use anchor_lang::{prelude::*, system_program};
use stablebond_sdk::find_kyc_pda;
use switchboard_solana::AggregatorAccountData;
use crate::{states::{factory_state::FactoryState, stablecoin::StablecoinState}, sol_fee_vault::SolFeeVault, user::UserState};
use anchor_spl::{
    associated_token::AssociatedToken, token::{self, Mint, Token, TokenAccount}
};
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct BurnStablecoin<'info> {
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

    #[account(
        mut,
        seeds = [
            USER_STATE_SEED,
            user.key().as_ref(),
            stablecoin_state.mint.as_ref()
        ],
        bump = user_state.bump
    )]
    pub user_state: Box<Account<'info, UserState>>,

    /// User's stablebond token account to receive bonds
    #[account(
        mut,
        constraint = user_bond_account.owner == user.key() @ StablecoinError::InvalidTokenAccountOwner,
        constraint = user_bond_account.mint == stablecoin_state.bond_mint @ StablecoinError::InvalidBondMint
    )]
    pub user_bond_account: Account<'info, TokenAccount>,

    /// User's stablecoin token account to burn from
    /// User's stablecoin token account
    #[account(
        mut,
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
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [SOL_FEE_VAULT_SEED],
        bump = sol_fee_vault.bump,
    )]
    pub sol_fee_vault: Account<'info, SolFeeVault>,

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

impl<'info> BurnStablecoin<'info> {
    pub fn validate(&self, amount: u64) -> Result<()> {
        msg!("Starting validation for burn amount: {}", amount);

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

        // 2. Check minimum redemption
        let bond_config = self.factory_state.get_bond_config(
            &self.stablecoin_state.bond_mint
        ).ok_or(StablecoinError::BondNotFound)?;

        require!(
            amount >= bond_config.min_redemption_amount,
            StablecoinError::RedeemAmountTooSmall
        );

        // 3. Verify oracle staleness
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp - self.stablecoin_state.last_price_update 
                <= ORACLE_STALENESS_THRESHOLD,
            StablecoinError::StaleOraclePrice
        );

        // 4. Verify user has enough stablecoins
        require!(
            self.user_stablecoin_account.amount >= amount,
            StablecoinError::InsufficientStablecoinBalance
        );

        // 5. Verify vault has enough bonds
        require!(
            self.collateral_vault.amount >= self.calculate_bond_return(
                amount,
                self.get_oracle_price()?,
                self.stablecoin_state.collateral_ratio,
            )?,
            StablecoinError::InsufficientCollateral
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

    pub fn calculate_bond_return(
        &self,
        stablecoin_amount: u64,
        bond_price: u64,    // Price scaled by PRICE_SCALE
        collateral_ratio: u16,
    ) -> Result<u64> {
        msg!("Calculating bond return for {} stablecoins at price {}", 
            stablecoin_amount, bond_price);

        // Calculate bond amount to return
        // bond_amount = (stablecoin_amount * collateral_ratio * PRICE_SCALE) / (bond_price * BPS_SCALE)
        
        // First multiply stablecoin amount by collateral ratio
        let collateral_needed = (stablecoin_amount as u128)
            .checked_mul(collateral_ratio as u128)
            .ok_or(StablecoinError::MathOverflow)?;

        // Multiply by PRICE_SCALE to match bond price scaling
        let scaled_collateral = collateral_needed
            .checked_mul(PRICE_SCALE as u128)
            .ok_or(StablecoinError::MathOverflow)?;

        // Divide by bond price and BPS_SCALE
        let bond_amount = scaled_collateral
            .checked_div(bond_price as u128)
            .ok_or(StablecoinError::MathOverflow)?
            .checked_div(BPS_SCALE as u128)
            .ok_or(StablecoinError::MathOverflow)?;

        require!(
            bond_amount <= u64::MAX as u128,
            StablecoinError::ExcessivePriceDeviation
        );

        Ok(bond_amount as u64)
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
        ctx: &Context<BurnStablecoin>,
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

        // Find user's position
        if let Some(index) = self.stablecoin_state.user_shares
            .iter()
            .position(|s| s.owner == user)
        {
            let share = &mut self.stablecoin_state.user_shares[index];
            
            // Verify user has enough balance to burn
            require!(
                share.bond_amount >= bond_amount &&
                share.mint_amount >= mint_amount,
                StablecoinError::InsufficientUserShare
            );
            
            // Update position
            share.bond_amount = share.bond_amount
                .checked_sub(bond_amount)
                .ok_or(StablecoinError::MathOverflow)?;
                
            share.mint_amount = share.mint_amount
                .checked_sub(mint_amount)
                .ok_or(StablecoinError::MathOverflow)?;
                
            share.timestamp = timestamp;
            
            // If position is completely empty, remove it
            if share.bond_amount == 0 && share.mint_amount == 0 {
                self.stablecoin_state.user_shares.remove(index);
            }
        } else {
            return Err(StablecoinError::UserShareNotFound.into());
        }

        Ok(())
    }
}

pub fn burn_stablecoin(
    ctx: Context<BurnStablecoin>,
    stablecoin_amount: u64,
) -> Result<()> {
    msg!("Starting stablecoin burn process");

    // 1. Validate all conditions
    ctx.accounts.validate(stablecoin_amount)?;

    // 2. Get oracle price
    let bond_price = ctx.accounts.get_oracle_price()?;
    msg!("Current bond price (scaled): {}", bond_price);

    // 3. Calculate bond amount to return
    // This is reverse of mint calculation:
    // bond_amount = (stablecoin_amount * collateral_ratio * PRICE_SCALE) / (bond_price * BPS_SCALE)
    let bond_amount = ctx.accounts.calculate_bond_return(
        stablecoin_amount,
        bond_price,
        ctx.accounts.stablecoin_state.collateral_ratio,
    )?;
    msg!("Calculated bond return amount: {}", bond_amount);

    // 4. Calculate and collect redemption fees
    let fee_amount = ctx.accounts.calculate_fee_amount(stablecoin_amount)?;
    ctx.accounts.collect_fees(&ctx, fee_amount)?;

    // 5. Burn stablecoins from user
    token::burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.stablecoin_mint.to_account_info(),
                from: ctx.accounts.user_stablecoin_account.to_account_info(),
                authority: ctx.accounts.user_state.to_account_info(),
            },
            &[&[
                USER_STATE_SEED,
                ctx.accounts.user.key().as_ref(),
                ctx.accounts.stablecoin_mint.key().as_ref(),
                &[ctx.accounts.user_state.bump],
            ]]
        ),
        stablecoin_amount
    )?;

    // 6. Transfer bonds from vault to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_bond_account.to_account_info(),
                authority: ctx.accounts.stablecoin_state.to_account_info(),
            },
            &[&[
                STABLECOIN_SEED,
                ctx.accounts.stablecoin_state.creator.as_ref(),
                ctx.accounts.stablecoin_state.original_symbol.as_bytes(),
                &[ctx.accounts.stablecoin_state.bump],
            ]]
        ),
        bond_amount
    )?;

    // 7. Update user share (decrease position)
    ctx.accounts.update_user_share(
        ctx.accounts.user.key(),
        bond_amount,
        stablecoin_amount,
    )?;

    // 8. Update state
    let stablecoin = &mut ctx.accounts.stablecoin_state;
    stablecoin.total_supply = stablecoin.total_supply
        .checked_sub(stablecoin_amount)
        .ok_or(StablecoinError::MathOverflow)?;
        
    stablecoin.total_collateral = stablecoin.total_collateral
        .checked_sub(bond_amount)
        .ok_or(StablecoinError::MathOverflow)?;
        
    stablecoin.last_updated = Clock::get()?.unix_timestamp;

    // 9. Emit event
    emit!(StablecoinBurned {
        user: ctx.accounts.user.key(),
        mint: ctx.accounts.stablecoin_mint.key(),
        bond_amount,
        stablecoin_amount,
        bond_price,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}