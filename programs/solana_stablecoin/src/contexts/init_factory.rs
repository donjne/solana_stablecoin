// init_factory.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::states::{factory_state::FactoryState, sol_fee_vault::SolFeeVault};
use crate::errors::StablecoinError;
use crate::events::FactoryInitialized;
use crate::constants::*;

/// InitializeFactory is the context for creating a new stablecoin factory.
/// This is the first instruction that must be called to set up the protocol.
/// The admin will have control over protocol parameters and emergency functions.
#[derive(Accounts)]
pub struct InitializeFactory<'info> {
    /// The admin who will have authority over the factory
    /// This account must be a signer as they're establishing control of the protocol
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The factory state PDA that stores all protocol configurations
    /// Seeds: ["factory_state"]
    /// This account is initialized here and will store all factory-wide parameters
    #[account(
        init,
        payer = admin,
        space = 8 + FactoryState::INIT_SPACE,
        seeds = [FACTORY_STATE_SEED],
        bump,
        constraint = factory_state.to_account_info().data_is_empty() 
        @ StablecoinError::FactoryAlreadyInitialized
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    #[account(
        address = USDC_MINT @ StablecoinError::InvalidUSDCMint
    )]
    pub fee_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + SolFeeVault::INIT_SPACE,
        seeds = [SOL_FEE_VAULT_SEED],
        bump
    )]
    pub sol_fee_vault: Box<Account<'info, SolFeeVault>>,

    /// The fee collection ATA, owned by factory PDA
    #[account(
        init,
        payer = admin,
        associated_token::mint = fee_mint,
        associated_token::authority = factory_state,  // Factory PDA is the authority
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializeFactory<'info> {
    pub fn validate(
        &self, 
        min_collateral_ratio: u16,
        base_fee_rate: u16,
    ) -> Result<()> {
        // Ensure the collateral ratio is within acceptable bounds
        require!(
            min_collateral_ratio >= MIN_ALLOWED_COLLATERAL_RATIO 
                && min_collateral_ratio <= MAX_ALLOWED_COLLATERAL_RATIO,
            StablecoinError::InvalidCollateralRatio
        );

        // Ensure the fee rate doesn't exceed maximum allowed (65% = 65000 basis points)
        require!(
            base_fee_rate <= MAX_FEE_RATE_BPS,
            StablecoinError::InvalidFeeRate
        );

        Ok(())
    }
}


pub fn initialize_factory(
    ctx: Context<InitializeFactory>,
    min_collateral_ratio: u16,
    base_fee_rate: u16,
) -> Result<()> {
    ctx.accounts.validate(min_collateral_ratio, base_fee_rate)?;

    let admin_key = ctx.accounts.admin.key();

    let factory_state = &mut ctx.accounts.factory_state;
    let sol_fee_vault = &mut ctx.accounts.sol_fee_vault;
    
    // Ensure vector capacity doesn't exceed max
    factory_state.allowed_bond_configs = Vec::with_capacity(MAX_ALLOWED_BONDS);
    factory_state.bond_collateral_tracking = Vec::with_capacity(MAX_ALLOWED_BONDS);
    factory_state.authorized_collectors = Vec::with_capacity(MAX_ALLOWED_COLLECTORS);
    
    // Initialize with default values where appropriate
    factory_state.admin = admin_key;
    factory_state.authorized_collectors.push(admin_key); 
    factory_state.fee_vault = ctx.accounts.fee_vault.key();
    factory_state.is_paused = false;
    factory_state.min_collateral_ratio = min_collateral_ratio;
    factory_state.base_fee_rate = base_fee_rate;
    factory_state.stablecoin_count = 0;
    factory_state.last_update = Clock::get()?.unix_timestamp;
    factory_state.protocol_version = 1;
    factory_state.bump = ctx.bumps.factory_state;
    factory_state.reserved = [0; RESERVE_SPACE];

    // Initialize sol fee vault
    sol_fee_vault.total_fees_collected = 0;
    sol_fee_vault.last_collection = Clock::get()?.unix_timestamp;
    sol_fee_vault.admin = ctx.accounts.admin.key();
    sol_fee_vault.bump = ctx.bumps.sol_fee_vault;

    emit!(FactoryInitialized {
        admin: ctx.accounts.admin.key(),
        fee_vault: ctx.accounts.fee_vault.key(),
        min_collateral_ratio,
        base_fee_rate,
        protocol_version: factory_state.protocol_version,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}