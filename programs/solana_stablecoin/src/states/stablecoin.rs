// states/stablecoin_state.rs
use anchor_lang::prelude::*;
use crate::states::user::UserShare;

#[account]
#[derive(InitSpace)]
pub struct StablecoinState {
    // Basic info
    #[max_len(32)]
    pub name: String,
    #[max_len(10)]
    pub symbol: String,
    #[max_len(10)]
    pub original_symbol: String,
    #[max_len(10)]
    pub target_currency: String,
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub last_updated: i64,

    // User info
    #[max_len(100)]  // Maximum users per stablecoin
    pub user_shares: Vec<UserShare>,
    
    // Collateral info
    pub stablebond_mint: Pubkey,        // The specific stablebond used as collateral
    pub collateral_vault: Pubkey,       // Vault holding stablebond tokens
    pub collateral_ratio: u16,          // u16 for basis points
    pub total_supply: u64,
    pub total_collateral: u64,

    // Yield/Rebase tracking
    pub last_rebase: i64,              // Last yield calculation & distribution
    pub total_rebase_amount: u64,      // Total amount of yield distributed via rebases
    pub last_yield_collection: i64,
    pub last_rate_update: i64,

    // Yield info
    pub yield_mint: Pubkey,
    pub total_yield_collected: u64,

    // Bond info
    pub bond_mint: Pubkey,
    
    // Price feeds
    pub fiat_oracle: Pubkey,            // Switchboard oracle for fiat price
    pub last_price_update: i64,         // Last time prices were checked
    
    // Protocol parameters
    pub is_paused: bool,
    pub fee_rate: u16,                  // In basis points
    pub last_fee_collection: i64,
    
    // Administrative
    pub bump: u8,
    pub reserved: [u8; 32],
}