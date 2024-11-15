use anchor_lang::{prelude::*, system_program};
use stablebond_sdk::find_kyc_pda;
use switchboard_solana::AggregatorAccountData;
use crate::{states::{factory_state::FactoryState, stablecoin::StablecoinState}, user::{UserShare, UserState}, sol_fee_vault::SolFeeVault};
use anchor_spl::{
    associated_token::AssociatedToken, token::{self, Mint, Token, TokenAccount}
};
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct MintStablecoin<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = !stablecoin_state.is_paused @ StablecoinError::StablecoinPaused
    )]
    pub stablecoin_state: Box<Account<'info, StablecoinState>>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    /// User's state PDA
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserState::INIT_SPACE,
        seeds = [
            USER_STATE_SEED,
            user.key().as_ref(),
            stablecoin_mint.key().as_ref()
        ],
        bump
    )]
    pub user_state: Box<Account<'info, UserState>>,

    /// User's stablebond token account
    #[account(
        mut,
        constraint = user_bond_account.owner == user.key() @ StablecoinError::InvalidTokenAccountOwner,
        constraint = user_bond_account.mint == stablecoin_state.bond_mint @ StablecoinError::InvalidBondMint
    )]
    pub user_bond_account: Account<'info, TokenAccount>,

    /// User's stablecoin token account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = stablecoin_mint,
        associated_token::authority = user_state,
    )]
    pub user_stablecoin_account: Account<'info, TokenAccount>,

    /// The stablecoin mint
    #[account(
        mut,
        address = stablecoin_state.mint
    )]
    pub stablecoin_mint: Box<Account<'info, Mint>>,

    /// Collateral vault
    #[account(
        mut,
        address = stablecoin_state.collateral_vault
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [SOL_FEE_VAULT_SEED],
        bump = sol_fee_vault.bump,
    )]
    pub sol_fee_vault: Box<Account<'info, SolFeeVault>>,

    /// CHECK: Verified in logic
    pub kyc_info: AccountInfo<'info>,

    // Switchboard oracle accounts
    pub oracle: AccountLoader<'info, AggregatorAccountData>,

    // Programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> MintStablecoin<'info> {
    pub fn validate(&self, amount: u64) -> Result<()> {
        msg!("Starting validation for mint amount: {}", amount);

        // 1. Verify KYC using Etherfuse
        let (kyc_pda, _) = find_kyc_pda(self.user.key());
        require!(
            self.kyc_info.key() == kyc_pda,
            StablecoinError::InvalidKycAccount
        );

        // Verify the account exists and is owned by Etherfuse program        
        require!(
            self.kyc_info.owner == &stablebond_sdk::ID,
            StablecoinError::InvalidKycAccount
        );

        // 2. Check minimum deposit
        let bond_config = self.factory_state.get_bond_config(
            &self.stablecoin_state.bond_mint
        ).ok_or(StablecoinError::BondNotFound)?;

        require!(
            amount >= bond_config.min_creation_amount,
            StablecoinError::DepositTooSmall
        );

        // 3. Verify oracle staleness
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp - self.stablecoin_state.last_price_update 
                <= ORACLE_STALENESS_THRESHOLD,
            StablecoinError::StaleOraclePrice
        );

        Ok(())
    }
    
    pub fn get_oracle_price(&self) -> Result<u64> {
        msg!("Fetching oracle price");
        
        let feed = self.oracle.load()?;
        // Convert to absolute value since prices can't be negative
        let mantissa = feed.latest_confirmed_round.result.mantissa.abs() as u128;
        
        let price = mantissa
            .checked_mul(PRICE_SCALE)
            .ok_or(StablecoinError::MathOverflow)?;
    
        require!(price > 0, StablecoinError::InvalidOraclePrice);
        
        Ok(price as u64)
    }

    pub fn calculate_mint_amount(
        &self,
        bond_amount: u64,
        bond_price: u64,    // Price scaled by PRICE_SCALE
        collateral_ratio: u16,
    ) -> Result<u64> {
        msg!("Calculating mint amount for {} bonds at price {}", 
            bond_amount, bond_price);

        // Calculate total collateral value 
        let collateral_value = (bond_amount as u128)
            .checked_mul(bond_price as u128)
            .ok_or(StablecoinError::MathOverflow)?;
        
        // Calculate mint amount based on collateral ratio
        // collateral_value * BPS_SCALE / collateral_ratio
        let mint_amount = collateral_value
            .checked_mul(BPS_SCALE as u128)
            .ok_or(StablecoinError::MathOverflow)?
            .checked_div(collateral_ratio as u128)
            .ok_or(StablecoinError::MathOverflow)?;
            
        // Scale back down
        let mint_amount = mint_amount
            .checked_div(PRICE_SCALE)
            .ok_or(StablecoinError::MathOverflow)?;
            
        require!(
            mint_amount <= u64::MAX as u128,
            StablecoinError::ExcessivePriceDeviation
        );
            
        Ok(mint_amount as u64)
    }

    pub fn calculate_fee_amount(&self, amount: u64) -> Result<u64> {
        let fee_rate = self.factory_state
            .get_fee_rate(&self.stablecoin_state.bond_mint)?;
            
        let fee_amount = (amount as u128)
            .checked_mul(fee_rate as u128)
            .ok_or(StablecoinError::MathOverflow)?
            .checked_div(BPS_SCALE as u128)
            .ok_or(StablecoinError::MathOverflow)?;
    
        require!(
            fee_amount <= u64::MAX as u128,
            StablecoinError::FeeTooLarge
        );
            
        Ok(fee_amount as u64)
    }

    pub fn collect_fees(
        &self,
        ctx: &Context<MintStablecoin>,
        fee_amount: u64
    ) -> Result<()> {
        msg!("Collecting fee: {} lamports", fee_amount);

        // Transfer SOL fee to PDA
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: ctx.accounts.sol_fee_vault.to_account_info(),
                }
            ),
            fee_amount as u64
        )?;
    
        Ok(())
    }

    pub fn update_user_share(
        &mut self,
        user: Pubkey,
        bond_amount: u64,
        mint_amount: u64,
    ) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;

        if let Some(share) = self.stablecoin_state.user_shares
            .iter_mut()
            .find(|s| s.owner == user)
        {
            // Update existing position
            share.bond_amount = share.bond_amount
                .checked_add(bond_amount)
                .ok_or(StablecoinError::MathOverflow)?;
                
            share.mint_amount = share.mint_amount
                .checked_add(mint_amount)
                .ok_or(StablecoinError::MathOverflow)?;
                
            share.timestamp = timestamp;
        } else {
            // Create new position
            require!(
                self.stablecoin_state.user_shares.len() < 100,
                StablecoinError::TooManyUsers
            );
            
            self.stablecoin_state.user_shares.push(UserShare {
                owner: user,
                bond_amount,
                mint_amount,
                timestamp,
            });
        }

        Ok(())
    }
}

pub fn mint_stablecoin(
    ctx: Context<MintStablecoin>, 
    bond_amount: u64
) -> Result<()> {
    msg!("Starting stablecoin mint process");

    // 1. Validate all conditions
    ctx.accounts.validate(bond_amount)?;

    // 2. Get oracle price
    let bond_price = ctx.accounts.get_oracle_price()?;
    msg!("Current bond price (scaled): {}", bond_price);

    // 3. Calculate mint amount
    let mint_amount = ctx.accounts.calculate_mint_amount(
        bond_amount,
        bond_price,
        ctx.accounts.stablecoin_state.collateral_ratio,
    )?;
    msg!("Calculated mint amount: {}", mint_amount);

    let fee_amount = ctx.accounts.calculate_fee_amount(mint_amount)?;

    // 4. Collect fees
    ctx.accounts.collect_fees(&ctx, fee_amount)?;

    // Update fee tracking
    let sol_fee_vault = &mut ctx.accounts.sol_fee_vault;
    sol_fee_vault.total_fees_collected = sol_fee_vault
        .total_fees_collected
        .checked_add(fee_amount as u64)
        .ok_or(StablecoinError::MathOverflow)?;
    sol_fee_vault.last_collection = Clock::get()?.unix_timestamp;

    // 5. Transfer bonds to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_bond_account.to_account_info(),
                to: ctx.accounts.collateral_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            }
        ),
        bond_amount
    )?;

    // 6. Update user share
    ctx.accounts.update_user_share(
        ctx.accounts.user.key(),
        bond_amount,
        mint_amount,
    )?;

    // 7. Mint stablecoins to user
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                to: ctx.accounts.user_stablecoin_account.to_account_info(),
                authority: ctx.accounts.stablecoin_state.to_account_info(),
            },
            &[&[
                STABLECOIN_SEED,
                ctx.accounts.stablecoin_state.creator.as_ref(),
                ctx.accounts.stablecoin_state.original_symbol.as_bytes(),
                &[ctx.accounts.stablecoin_state.bump],
            ]]
        ),
        mint_amount
    )?;

    // 8. Update state
    let stablecoin = &mut ctx.accounts.stablecoin_state;
    stablecoin.total_supply = stablecoin.total_supply
        .checked_add(mint_amount)
        .ok_or(StablecoinError::MathOverflow)?;
        
    stablecoin.total_collateral = stablecoin.total_collateral
        .checked_add(bond_amount)
        .ok_or(StablecoinError::MathOverflow)?;
        
    stablecoin.last_updated = Clock::get()?.unix_timestamp;

    // 9. Emit event with scaled price
    emit!(StablecoinMinted {
        user: ctx.accounts.user.key(),
        mint: ctx.accounts.stablecoin_mint.key(),
        bond_amount,
        mint_amount,
        bond_price,  // a scaled integer
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}