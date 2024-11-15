use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserState {
    pub bump: u8,
    pub user: Pubkey,
    pub stablecoin: Pubkey,  // The stablecoin this user state belongs to
    pub total_yield_collected: u64,
    pub last_yield_collection: i64,
    pub bond_amount: u64,
    pub stablecoin_amount: u64,
    pub reserved: [u8; 32],  // Space for future fields
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UserShare {
    pub owner: Pubkey,
    pub bond_amount: u64,    // Amount of stablebonds deposited
    pub mint_amount: u64,    // Amount of stablecoins minted
    pub timestamp: i64,      // When position was last updated
}

impl anchor_lang::Space for UserShare {
    const INIT_SPACE: usize = 
        32 +    // Pubkey (bond_mint)
        8 +    // u64 (bond_amount)
        8 +     // u64 (mint_amount)
        8;      // i64 (timestamp)
}