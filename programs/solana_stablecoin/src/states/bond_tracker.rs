use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BondCollateralInfo {
    pub bond_mint: Pubkey,
    pub total_collateral: u64,          // Total amount locked across all stablecoins
    pub num_stablecoins: u32,           // Number of stablecoins using this bond
}

impl anchor_lang::Space for BondCollateralInfo {
    const INIT_SPACE: usize = 
        32 +    // Pubkey (bond_mint)
        8 +    // u64 (total_collateral)
        4;     // u32 (num_stablecoins)
}
