use anchor_lang::prelude::*;
use stablebond_sdk::types::PaymentFeedType;
use crate::{states::{factory_state::FactoryState, stablecoin::StablecoinState}, user::UserState};
use anchor_spl::{
    associated_token::AssociatedToken, token::{self, Mint, Token, TokenAccount}
};
use crate::errors::StablecoinError;
use switchboard_solana::AggregatorAccountData;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct DistributeYield<'info> {
    #[account(mut)]
    pub distributor: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump,
        constraint = !factory_state.is_paused @ StablecoinError::FactoryPaused,
        constraint = factory_state.is_authorized_collector(distributor.key()) @ StablecoinError::Unauthorized
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    #[account(
        mut,
        constraint = !stablecoin_state.is_paused @ StablecoinError::StablecoinPaused
    )]
    pub stablecoin_state: Box<Account<'info, StablecoinState>>,

    #[account(
        mut,
        seeds = [
            USER_STATE_SEED,
            user_state.user.as_ref(),
            stablecoin_state.mint.as_ref()
        ],
        bump = user_state.bump
    )]
    pub user_state: Box<Account<'info, UserState>>,

    /// Bond oracle for price feed
    pub oracle: AccountLoader<'info, AggregatorAccountData>,

    /// CHECK: Verified through find_bond_pda
    pub bond_info: AccountInfo<'info>,

    /// The bond mint 
    #[account(address = stablecoin_state.bond_mint)]
    pub bond_mint: Box<Account<'info, Mint>>,

    /// Protocol fee vault
    #[account(
        mut,
        associated_token::mint = yield_mint,
        associated_token::authority = factory_state
    )]
    pub usdc_fee_vault: Account<'info, TokenAccount>,

    #[account(
        address = USDC_MINT @ StablecoinError::InvalidUSDCMint
    )]
    pub yield_mint: Box<Account<'info, Mint>>,

    /// User's yield token account
    #[account(
        init_if_needed,
        payer = distributor,
        associated_token::mint = yield_mint,
        associated_token::authority = user_state,
    )]
    pub user_yield_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> DistributeYield<'info> {
    pub fn validate(&self) -> Result<()> {
        // Verify bond PDA
        let (bond_pda, _) = stablebond_sdk::find_bond_pda(self.bond_mint.key());
        require!(
            bond_pda == self.bond_info.key(),
            StablecoinError::InvalidBondAccount
        );

        let bond = stablebond_sdk::accounts::Bond::try_from_slice(
            &self.bond_info.try_borrow_data()?
        )?;

        // Verify bond mint matches
        require!(
            bond.mint == self.bond_mint.key(),
            StablecoinError::BondMintMismatch
        );

        // Verify rebase interval has passed
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time - self.stablecoin_state.last_rebase >= REBASE_INTERVAL,
            StablecoinError::RebaseTooEarly 
        );

        // Verify user has a position
        require!(
            self.stablecoin_state.user_shares.iter()
                .any(|share| share.owner == self.user_state.user),
            StablecoinError::NoUserPosition
        );

        require!(
            current_time - self.stablecoin_state.last_rate_update <= RATE_UPDATE_INTERVAL,
            StablecoinError::StaleError
        );

        Ok(())
    }

    pub fn get_conversion_rate(&self, feed_type: &PaymentFeedType) -> Result<u128> {
        let current_time = Clock::get()?.unix_timestamp;
        let feed = self.oracle.load()?;

        // Verify feed freshness
        require!(
            current_time - feed.latest_confirmed_round.round_open_timestamp <= RATE_FRESHNESS_THRESHOLD,
            StablecoinError::StaleError
        );

        match feed_type {
            // USD pairs 
            PaymentFeedType::UsdcMxn | 
            PaymentFeedType::SwitchboardUsdcMxn | 
            PaymentFeedType::SwitchboardUsdcBrl | 
            PaymentFeedType::SwitchboardUsdcEur | 
            PaymentFeedType::SwitchboardUsdcGbp => {
                msg!("Using oracle conversion rate for {:?}", feed_type);
                Ok(feed.latest_confirmed_round.result.mantissa.abs() as u128)
            },

            // USD/USD pairs
            PaymentFeedType::UsdcUsd |
            PaymentFeedType::SwitchboardUsdcUsd => {
                msg!("Using 1:1 USD conversion");
                Ok(PRICE_SCALE)
            },

            PaymentFeedType::Stub => {
                msg!("Using stub conversion rate");
                Ok(PRICE_SCALE)
            }
        }
    }

    pub fn calculate_rebase_yield(&self) -> Result<(u64, u64)> {
        let bond = stablebond_sdk::accounts::Bond::try_from_slice(
            &self.bond_info.try_borrow_data()?
        )?;

        // Get current price and conversion rate
        let base_price = self.oracle.load()?.latest_confirmed_round.result.mantissa.abs() as u128;
        let conversion_rate = self.get_conversion_rate(&bond.payment_feed_type)?;

        let current_price = base_price
        .checked_mul(conversion_rate)
        .ok_or(StablecoinError::MathOverflow)?
        .checked_div(PRICE_SCALE)
        .ok_or(StablecoinError::MathOverflow)?;

        self.calculate_yield_from_price(current_price)
    }

    fn calculate_yield_from_price(&self, current_price: u128) -> Result<(u64, u64)> {
        // Calculate time elapsed since last rebase
        let current_time = Clock::get()?.unix_timestamp;
        let time_elapsed = (current_time - self.stablecoin_state.last_rebase) as u128;

        let user_share = self.stablecoin_state.user_shares
            .iter()
            .find(|share| share.owner == self.user_state.user)
            .ok_or(StablecoinError::NoUserPosition)?;

        // Calculate yield based on price and share
        let base_amount = user_share.mint_amount as u128;
        
        let yield_rate = current_price
            .checked_mul(time_elapsed)
            .ok_or(StablecoinError::MathOverflow)?
            .checked_div(YEAR_IN_SECONDS as u128)
            .ok_or(StablecoinError::MathOverflow)?;

        let total_yield = base_amount
            .checked_mul(yield_rate)
            .ok_or(StablecoinError::MathOverflow)?
            .checked_div(PRICE_SCALE as u128)
            .ok_or(StablecoinError::MathOverflow)?;

        let protocol_fee = total_yield
            .checked_mul(PROTOCOL_FEE_BPS as u128)
            .ok_or(StablecoinError::MathOverflow)?
            .checked_div(BPS_SCALE as u128)
            .ok_or(StablecoinError::MathOverflow)?;

        let user_yield = total_yield
            .checked_sub(protocol_fee)
            .ok_or(StablecoinError::MathOverflow)?;

        require!(
            protocol_fee <= u64::MAX as u128 && user_yield <= u64::MAX as u128,
            StablecoinError::MathOverflow
        );

        Ok((protocol_fee as u64, user_yield as u64))
    }
}



pub fn distribute_yield(ctx: Context<DistributeYield>) -> Result<()> {
    // 1. Validate state and check rebase interval
    ctx.accounts.validate()?;

    // 2. Calculate rebase yield shares
    let (protocol_fee, user_yield) = ctx.accounts.calculate_rebase_yield()?;

    // 3. Transfer protocol fee
    if protocol_fee > 0 {
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.yield_mint.to_account_info(),
                    to: ctx.accounts.usdc_fee_vault.to_account_info(),
                    authority: ctx.accounts.stablecoin_state.to_account_info(),
                },
                &[&[
                    STABLECOIN_SEED,
                    ctx.accounts.stablecoin_state.creator.as_ref(),
                    ctx.accounts.stablecoin_state.original_symbol.as_bytes(),
                    &[ctx.accounts.stablecoin_state.bump],
                ]]
            ),
            protocol_fee
        )?;
    }

    // 4. Mint user yield
    if user_yield > 0 {
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.yield_mint.to_account_info(),
                    to: ctx.accounts.user_yield_account.to_account_info(),
                    authority: ctx.accounts.stablecoin_state.to_account_info(),
                },
                &[&[
                    STABLECOIN_SEED,
                    ctx.accounts.stablecoin_state.creator.as_ref(),
                    ctx.accounts.stablecoin_state.original_symbol.as_bytes(),
                    &[ctx.accounts.stablecoin_state.bump],
                ]]
            ),
            user_yield
        )?;

        // Update user state tracking
        let user_state = &mut ctx.accounts.user_state;
        user_state.total_yield_collected = user_state.total_yield_collected
            .checked_add(user_yield)
            .ok_or(StablecoinError::MathOverflow)?;
        user_state.last_yield_collection = Clock::get()?.unix_timestamp;
    }

    // Update rebase timestamp
    ctx.accounts.stablecoin_state.last_rebase = Clock::get()?.unix_timestamp;
    ctx.accounts.stablecoin_state.total_rebase_amount += protocol_fee.checked_add(user_yield).ok_or(StablecoinError::MathOverflow)?;
    

    // 5. Emit event
    emit!(YieldDistributed {
        user: ctx.accounts.user_state.user,
        stablecoin: ctx.accounts.stablecoin_state.key(),
        protocol_fee,
        user_yield,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}