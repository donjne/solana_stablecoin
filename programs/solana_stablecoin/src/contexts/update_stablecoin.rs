use anchor_lang::prelude::*;
use crate::states::{factory_state::FactoryState, stablecoin::StablecoinState};
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct UpdateStablecoin<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Factory state PDA
    #[account(
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
        constraint = !factory_state.is_paused @ StablecoinError::FactoryPaused
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    /// Stablecoin state PDA that we want to update
    #[account(
        mut,
        seeds = [
            STABLECOIN_SEED,
            stablecoin_state.creator.key().as_ref(),
            stablecoin_state.original_symbol.as_bytes()
        ],
        bump = stablecoin_state.bump,
        constraint = stablecoin_state.creator == authority.key() @ StablecoinError::UnauthorizedAccess
    )]
    pub stablecoin_state: Box<Account<'info, StablecoinState>>,

    pub system_program: Program<'info, System>,
}

impl<'info> UpdateStablecoin<'info> {
    pub fn validate(
        &self,
        name: &Option<String>,
        symbol: &Option<String>,
    ) -> Result<()> {
        // Validate name if provided
        if let Some(name) = name {
            require!(!name.is_empty() && name.len() <= 32, StablecoinError::InvalidName);
        }

        // Validate symbol if provided
        if let Some(symbol) = symbol {
            require!(!symbol.is_empty() && symbol.len() <= 10, StablecoinError::InvalidSymbol);
        }

        Ok(())
    }
}

pub fn update_stablecoin(
    ctx: Context<UpdateStablecoin>,
    name: Option<String>,
    symbol: Option<String>,
) -> Result<()> {
    ctx.accounts.validate(&name, &symbol)?;
    
    let stablecoin = &mut ctx.accounts.stablecoin_state;
    let current_timestamp = Clock::get()?.unix_timestamp;
    
    // Update name if provided
    if let Some(new_name) = name.clone() {
        stablecoin.name = new_name;
    }

    // Update symbol if provided
    if let Some(new_symbol) = symbol.clone() {
        stablecoin.symbol = new_symbol;
    }

    // Update the last_updated timestamp
    stablecoin.last_updated = current_timestamp;

    emit!(StablecoinUpdated {
        authority: ctx.accounts.authority.key(),
        mint: stablecoin.mint,
        name: name,
        symbol: symbol,
        timestamp: current_timestamp,
    });

    Ok(())
}