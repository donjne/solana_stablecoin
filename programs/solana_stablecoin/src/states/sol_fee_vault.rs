use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct SolFeeVault {
    pub total_fees_collected: u64,
    pub last_collection: i64,
    pub admin: Pubkey,
    pub bump: u8,
}