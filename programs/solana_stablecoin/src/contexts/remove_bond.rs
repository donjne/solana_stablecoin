use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::states::factory_state::FactoryState;
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct RemoveBond<'info> {
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

pub fn remove_bond(ctx: Context<RemoveBond>) -> Result<()> {
    msg!("Removing bond: {}", ctx.accounts.bond_mint.key());

    let factory_state = &mut ctx.accounts.factory_state;
    
    // First check if bond has any active collateral
    // We're already tracking total collateral per bond type
    require!(
        !factory_state.has_active_collateral(&ctx.accounts.bond_mint.key())?,
        StablecoinError::ActiveCollateralExists
    );

    // Find and remove bond
    let position = factory_state.allowed_bond_configs
        .iter()
        .position(|c| c.bond_mint == ctx.accounts.bond_mint.key())
        .ok_or(StablecoinError::BondNotFound)?;

    factory_state.allowed_bond_configs.remove(position);

    emit!(BondRemoved {
        bond_mint: ctx.accounts.bond_mint.key(),
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
