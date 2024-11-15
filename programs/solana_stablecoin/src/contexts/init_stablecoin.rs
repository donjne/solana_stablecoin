// contexts/create_stablecoin.rs
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::states::{factory_state::FactoryState, stablecoin::StablecoinState, user::UserState};
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    target_currency: String,
)]
pub struct CreateStablecoin<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Factory state PDA
    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
        constraint = !factory_state.is_paused @ StablecoinError::FactoryPaused
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    /// Stablecoin state PDA
    #[account(
        init,
        payer = creator,
        space = 8 + StablecoinState::INIT_SPACE,
        seeds = [
            STABLECOIN_SEED,
            creator.key().as_ref(),
            symbol.as_bytes()
        ],
        bump
    )]
    pub stablecoin_state: Box<Account<'info, StablecoinState>>,

    #[account(
        init,
        payer = creator,
        space = 8 + UserState::INIT_SPACE,
        seeds = [
            USER_STATE_SEED,
            creator.key().as_ref(),
            mint.key().as_ref()
        ],
        bump
    )]
    pub creator_state: Box<Account<'info, UserState>>,

    /// The mint for the stablecoin
    #[account(
        init,
        payer = creator,
        mint::decimals = STABLECOIN_DECIMALS,
        mint::authority = stablecoin_state,
    )]
    pub mint: Box<Account<'info, Mint>>,

    /// Yield mint
    #[account(
        address = USDC_MINT @ StablecoinError::InvalidUSDCMint
    )]
    pub yield_mint: Box<Account<'info, Mint>>,

    /// Collateral vault for stablebonds
    #[account(
        init,
        payer = creator,
        associated_token::mint = bond_mint,
        associated_token::authority = stablecoin_state,
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,

    /// The stablebond mint we're using as collateral
    pub bond_mint: Box<Account<'info, Mint>>,

    /// CHECK: Validated in is_bond_supported using Etherfuse PDA
    pub bond_info: AccountInfo<'info>,

    // Required programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateStablecoin<'info> {
    pub fn validate(
        &self,
        name: &str,
        symbol: &str,
        _target_currency: &str,
    ) -> Result<()> {
        // Basic validation
        require!(!name.is_empty() && name.len() <= 32, StablecoinError::InvalidName);
        require!(!symbol.is_empty() && symbol.len() <= 10, StablecoinError::InvalidSymbol);

        require!(
            self.factory_state.is_bond_supported(
                &self.bond_mint.key(),
                &self.bond_info
            )?, 
            StablecoinError::UnsupportedBond
        );

        Ok(())
    }
}

pub fn create_stablecoin(
    ctx: Context<CreateStablecoin>,
    name: String,
    symbol: String,
    target_currency: String,
) -> Result<()> {

    // bond_mint: The stablebond token mint (e.g., CETES, USTRY)
    // bond_info: The Etherfuse bond PDA
    // payment_feed_info: The Etherfuse payment feed PDA
    
    ctx.accounts.validate(&name, &symbol, &target_currency)?;
    let stablecoin = &mut ctx.accounts.stablecoin_state;
    
    // Initialize basic info
    stablecoin.name = name.clone();
    stablecoin.symbol = symbol.clone();
    stablecoin.original_symbol = symbol.clone();
    stablecoin.target_currency = target_currency.clone();
    stablecoin.creator = ctx.accounts.creator.key();
    stablecoin.mint = ctx.accounts.mint.key();
    stablecoin.yield_mint = ctx.accounts.yield_mint.key();
    stablecoin.total_yield_collected = 0;
    stablecoin.collateral_vault = ctx.accounts.collateral_vault.key();
    stablecoin.bond_mint = ctx.accounts.bond_mint.key();
    stablecoin.last_updated = Clock::get()?.unix_timestamp;
    stablecoin.last_rebase = Clock::get()?.unix_timestamp;
    stablecoin.total_rebase_amount = 0;

    stablecoin.user_shares = Vec::with_capacity(100);
    
    // Initialize tracking
    stablecoin.total_supply = 0;
    stablecoin.total_collateral = 0;
    stablecoin.bump = ctx.bumps.stablecoin_state;

    // Update factory
    let factory = &mut ctx.accounts.factory_state;
    factory.stablecoin_count += 1;
    msg!("Stablecoin created! Now initializing user state...");

    // Initialize creator's user state
    let creator_state = &mut ctx.accounts.creator_state;
    creator_state.bump = ctx.bumps.creator_state;
    creator_state.user = ctx.accounts.creator.key();
    creator_state.stablecoin = ctx.accounts.mint.key();
    creator_state.total_yield_collected = 0;
    creator_state.last_yield_collection = Clock::get()?.unix_timestamp;
    creator_state.bond_amount = 0;
    creator_state.stablecoin_amount = 0;
    msg!("User state successfully initialized!");

    emit!(StablecoinCreated {
        creator: ctx.accounts.creator.key(),
        mint: ctx.accounts.mint.key(),
        bond_mint: ctx.accounts.bond_mint.key(),
        name: name.clone(),
        symbol: symbol.clone(),
        target_currency: target_currency.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}