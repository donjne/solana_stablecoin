use anchor_lang::prelude::*;
use crate::states::{factory_state::FactoryState, stablecoin::StablecoinState};
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct ResumeStablecoin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
        constraint = !factory_state.is_paused @ StablecoinError::FactoryPaused,
        constraint = factory_state.admin == admin.key() @ StablecoinError::Unauthorized
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    #[account(
        mut,
        constraint = stablecoin_state.creator == admin.key() @ StablecoinError::Unauthorized,
    )]
    pub stablecoin_state: Box<Account<'info, StablecoinState>>,

    pub system_program: Program<'info, System>,
}

pub fn resume_stablecoin(
    ctx: Context<ResumeStablecoin>,
) -> Result<()> {
    // Undo pause state
    let stablecoin = &mut ctx.accounts.stablecoin_state;
    require!(stablecoin.is_paused, StablecoinError::NotPaused);
    
    stablecoin.is_paused = false;
    stablecoin.last_updated = Clock::get()?.unix_timestamp;

    // Emit event
    emit!(StablecoinResumed {
        admin: ctx.accounts.admin.key(),
        stablecoin: ctx.accounts.stablecoin_state.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}