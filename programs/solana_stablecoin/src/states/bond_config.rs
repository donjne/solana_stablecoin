// bond_config.rs
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StablebondConfig {
    // Essential bond identification
    pub bond_mint: Pubkey,              // The Stablebond SPL token mint
    pub payment_mint: Pubkey,           // The payment token mint (from Etherfuse)
    
    // Operational parameters
    pub admin: Pubkey,
    pub min_creation_amount: u64,
    pub min_redemption_amount: u64,
    pub is_enabled: bool,               // Admin control
    pub custom_fee_rate: Option<u16>,   // Optional custom fee rate
}

impl anchor_lang::Space for StablebondConfig {
    const INIT_SPACE: usize = 
        32 +    // Pubkey (bond_mint)
        32 +    // Pubkey (payment_mint)
        32 +     // Pubkey (admin)
        1 +     // bool (is_enabled)
        3;     // Option<u16> (custom_fee_rate)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BondConfigUpdate {
    pub is_enabled: Option<bool>,
    pub custom_fee_rate: Option<Option<u16>>, // Double Option: None = no change, Some(None) = remove custom rate
}