// update_factory.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::states::factory_state::FactoryState;
use crate::errors::StablecoinError;
use crate::events::FactoryConfigUpdated;
use crate::constants::*;

/// UpdateFactoryConfig is the context for updating factory configuration parameters.
/// Only the current admin can execute this instruction.
#[derive(Accounts)]
pub struct UpdateFactoryConfig<'info> {
    /// The current admin who must sign to authorize changes
    #[account(
        constraint = factory_state.admin == admin.key() @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    /// Optional new admin if admin transfer is requested
    /// CHECK: Validated in instruction logic
    pub new_admin: Option<UncheckedAccount<'info>>,

    /// The factory state PDA containing configuration
    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    #[account(
        address = USDC_MINT @ StablecoinError::InvalidUSDCMint
    )]
    pub fee_mint: Account<'info, Mint>,

    /// The fee collection ATA, owned by factory PDA
    #[account(
        mut,
        associated_token::mint = fee_mint,
        associated_token::authority = factory_state,  // Factory PDA is the authority
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    /// Optional new collector to add
    /// CHECK: Validated in instruction logic
    pub new_collector: Option<UncheckedAccount<'info>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> UpdateFactoryConfig<'info> {
    /// Validates the update parameters
    pub fn validate(
        &self,
        new_min_collateral_ratio: Option<u16>,
        new_base_fee_rate: Option<u16>,
    ) -> Result<()> {
        // Validate new collateral ratio if provided
        if let Some(ratio) = new_min_collateral_ratio {
            require!(
                ratio >= MIN_ALLOWED_COLLATERAL_RATIO 
                    && ratio <= MAX_ALLOWED_COLLATERAL_RATIO,
                StablecoinError::InvalidCollateralRatio
            );
        }

        // Validate new fee rate if provided
        if let Some(fee) = new_base_fee_rate {
            require!(
                fee <= MAX_FEE_RATE_BPS,
                StablecoinError::InvalidFeeRate
            );
        }

        Ok(())
    }
}

pub fn update_factory_config(
    ctx: Context<UpdateFactoryConfig>,
    new_admin: Option<Pubkey>,
    new_min_collateral_ratio: Option<u16>,
    new_base_fee_rate: Option<u16>,
    new_fee_vault: Option<Pubkey>,
) -> Result<()> {
    // Validate input parameters if provided
    ctx.accounts.validate(new_min_collateral_ratio, new_base_fee_rate)?;

    let factory_state = &mut ctx.accounts.factory_state;

    // Add new collector if provided
    if let Some(collector_account) = &ctx.accounts.new_collector {
        let collector_key = collector_account.key();
        if !factory_state.authorized_collectors.contains(&collector_key) 
            && factory_state.authorized_collectors.len() < MAX_ALLOWED_COLLECTORS 
        {
            factory_state.authorized_collectors.push(collector_key);
        }
    }
    
    // Update admin if new admin is provided
    if let Some(new_admin) = new_admin {
        factory_state.admin = new_admin;
    }

    // Update collateral ratio if provided
    if let Some(ratio) = new_min_collateral_ratio {
        factory_state.min_collateral_ratio = ratio;
    }

    // Update base fee rate if provided
    if let Some(fee) = new_base_fee_rate {
        factory_state.base_fee_rate = fee;
    }

    // Update fee recipient if provided
    if let Some(vault) = new_fee_vault {
        factory_state.fee_vault = vault;
    }

    factory_state.protocol_version += 1;

    // Emit configuration update event
    emit!(FactoryConfigUpdated {
        admin: factory_state.admin,
        fee_vault: factory_state.fee_vault,
        min_collateral_ratio: factory_state.min_collateral_ratio,
        base_fee_rate: factory_state.base_fee_rate,
        protocol_version: factory_state.protocol_version,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}