use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct FeeConfig {
    pub mint_fee_bps: u16,
    pub redeem_fee_bps: u16,
    pub fee_collector: Pubkey,
}