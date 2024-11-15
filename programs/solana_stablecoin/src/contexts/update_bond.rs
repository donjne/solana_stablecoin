use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::states::{factory_state::FactoryState, bond_config::BondConfigUpdate};
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct UpdateBondConfig<'info> {
    #[account(
        constraint = admin.key() == factory_state.admin @ StablecoinError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    pub bond_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

pub fn update_bond_config(
    ctx: Context<UpdateBondConfig>, 
    updates: BondConfigUpdate
) -> Result<()> {
    msg!("Updating config for bond: {}", ctx.accounts.bond_mint.key());

    let factory_state = &mut ctx.accounts.factory_state;
    
    // Find bond config
    let config = factory_state.allowed_bond_configs
        .iter_mut()
        .find(|c| c.bond_mint == ctx.accounts.bond_mint.key())
        .ok_or(StablecoinError::BondNotFound)?;

    // Apply updates
    if let Some(is_enabled) = updates.is_enabled {
        msg!("Updating enabled status to: {}", is_enabled);
        config.is_enabled = is_enabled;
    }

    if let Some(custom_fee) = updates.custom_fee_rate {
        msg!("Updating custom fee rate to: {:?}", custom_fee);
        if let Some(fee) = custom_fee {
            require!(
                fee <= MAX_FEE_RATE_BPS,
                StablecoinError::InvalidFeeRate
            );
        }
        config.custom_fee_rate = custom_fee;
    }

    emit!(BondConfigUpdated {
        bond_mint: ctx.accounts.bond_mint.key(),
        is_enabled: config.is_enabled,
        custom_fee_rate: config.custom_fee_rate,
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}