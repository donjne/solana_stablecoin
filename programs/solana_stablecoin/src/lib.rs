use anchor_lang::prelude::*;

pub mod contexts;
pub mod states;
pub mod errors;
pub mod constants;
pub mod events;


use contexts::*;
use states::*;

declare_id!("7gFZNhBQidDAqbzqbuFFzetrnordQLLpibV8hX5S2taU");

#[program]
pub mod solana_stablecoin {
    use super::*;

    // Factory Management Instructions
    pub fn initialize_factory(
        ctx: Context<InitializeFactory>,
        min_collateral_ratio: u16,
        base_fee_rate: u16,
    ) -> Result<()> {
        contexts::initialize_factory(ctx, min_collateral_ratio, base_fee_rate)
    }

    pub fn update_factory_config(
        ctx: Context<UpdateFactoryConfig>,
        new_admin: Option<Pubkey>,
        new_min_collateral_ratio: Option<u16>,
        new_base_fee_rate: Option<u16>,
        new_fee_recipient: Option<Pubkey>,
        
    ) -> Result<()> {
        contexts::update_factory_config(ctx, new_admin, new_min_collateral_ratio, new_base_fee_rate, new_fee_recipient)
    }

    // Stablecoin Creation and Management
    pub fn create_stablecoin(
        ctx: Context<CreateStablecoin>,
        name: String,
        symbol: String,
        target_currency: String,
    ) -> Result<()> {
        contexts::create_stablecoin(ctx, name, symbol, target_currency)
    }

    pub fn update_stablecoin(
        ctx: Context<UpdateStablecoin>,
        name: Option<String>,
        symbol: Option<String>,
    ) -> Result<()> {
        contexts::update_stablecoin(ctx, name, symbol)
    }

    pub fn add_supported_bond(ctx: Context<AddSupportedBond>, min_creation_amount: u64, min_redemption_amount: u64) -> Result<()> {
        contexts::add_supported_bond(ctx, min_creation_amount, min_redemption_amount)
    }

    pub fn remove_bond(ctx: Context<RemoveBond>) -> Result<()> {
        contexts::remove_bond(ctx)
    }

    pub fn update_bond_config(
        ctx: Context<UpdateBondConfig>, 
        updates: BondConfigUpdate
    ) -> Result<()> {
        contexts::update_bond_config(ctx, updates)
    }

    // Token Operations
    pub fn mint_tokens(
        ctx: Context<MintStablecoin>, 
        bond_amount: u64
    ) -> Result<()> {
        contexts::mint_stablecoin(ctx, bond_amount)
    }

    pub fn burn_tokens(
        ctx: Context<BurnStablecoin>,
        stablecoin_amount: u64,
    ) -> Result<()> {
        contexts::burn_stablecoin(ctx, stablecoin_amount)
    }

    // Yield Management
    pub fn distribute_yield(
        ctx: Context<DistributeYield>,
    ) -> Result<()> {
        contexts::distribute_yield(ctx)
    }

    // Emergency Controls
    pub fn pause_stablecoin(
        ctx: Context<PauseStablecoin>,
    ) -> Result<()> {
        contexts::pause_stablecoin(ctx)
    }

    pub fn resume_stablecoin(
        ctx: Context<ResumeStablecoin>,
    ) -> Result<()> {
        contexts::resume_stablecoin(ctx)
    }
}