// events.rs
use anchor_lang::prelude::*;

#[event]
pub struct FactoryInitialized {
    pub admin: Pubkey,
    pub fee_vault: Pubkey,
    pub min_collateral_ratio: u16,
    pub base_fee_rate: u16,
    pub protocol_version: u16,
    pub timestamp: i64,
}

#[event]
pub struct FactoryConfigUpdated {
    pub admin: Pubkey,
    pub fee_vault: Pubkey,
    pub min_collateral_ratio: u16,
    pub base_fee_rate: u16,
    pub protocol_version: u16,
    pub timestamp: i64,
}

#[event]
pub struct StablecoinCreated {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub bond_mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub target_currency: String,
    pub timestamp: i64,
}

#[event]
pub struct StablecoinMinted {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bond_amount: u64,
    pub mint_amount: u64,
    pub bond_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct StablecoinUpdated {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub timestamp: i64,
}

#[event]
pub struct StablecoinBurned {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub bond_amount: u64,
    pub stablecoin_amount: u64,
    pub bond_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct YieldCollected {
    pub collector: Pubkey,
    pub stablecoin: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct YieldDistributed {
    pub user: Pubkey,
    pub stablecoin: Pubkey,
    pub protocol_fee: u64,
    pub user_yield: u64,
    pub timestamp: i64,
}

#[event]
pub struct BondAdded {
    pub bond_mint: Pubkey,
    pub payment_mint: Pubkey,
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct BondConfigUpdated {
    pub bond_mint: Pubkey,
    pub is_enabled: bool,
    pub custom_fee_rate: Option<u16>,
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct BondRemoved {
    pub bond_mint: Pubkey,
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct StablecoinPaused {
    pub admin: Pubkey,
    pub stablecoin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct StablecoinResumed {
    pub admin: Pubkey,
    pub stablecoin: Pubkey,
    pub timestamp: i64,
}