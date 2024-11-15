use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use stablebond_sdk::{
    accounts::{Bond, PaymentFeed},
    find_bond_pda, find_payment_feed_pda,
};
use crate::states::factory_state::FactoryState;
use crate::errors::StablecoinError;
use crate::constants::*;
use crate::events::*;

#[derive(Accounts)]
pub struct AddSupportedBond<'info> {
    #[account(
        constraint = admin.key() == factory_state.admin @ StablecoinError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump,
    )]
    pub factory_state: Box<Account<'info, FactoryState>>,

    // The bond mint to verify
    pub bond_mint: Box<Account<'info, Mint>>,

    // The Etherfuse bond account
    /// CHECK: Verified in instruction
    pub bond_info: AccountInfo<'info>,

    // The payment feed from Etherfuse
    /// CHECK: Verified in instruction
    pub payment_feed_info: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn add_supported_bond(ctx: Context<AddSupportedBond>, min_creation_amount: u64, min_redemption_amount: u64) -> Result<()> {
    msg!("Adding supported bond: {}", ctx.accounts.bond_mint.key());

    // Verify it's a valid Etherfuse bond
    let (bond_pda, _) = find_bond_pda(ctx.accounts.bond_mint.key());
    require!(
        ctx.accounts.bond_info.key() == bond_pda,
        StablecoinError::InvalidBondAccount
    );

    // Get bond info
    let bond = Bond::try_from_slice(&ctx.accounts.bond_info.try_borrow_data()?)?;
    
    // Verify payment feed
    let feed_type = bond.payment_feed_type.clone();
    let (payment_feed_pda, _) = find_payment_feed_pda(feed_type);
    require!(
        ctx.accounts.payment_feed_info.key() == payment_feed_pda,
        StablecoinError::InvalidPaymentFeed
    );

    // Get payment mint from feed
    let payment_feed = PaymentFeed::try_from_slice(
        &ctx.accounts.payment_feed_info.try_borrow_data()?
    )?;

    // Add to factory's supported bonds
    ctx.accounts.factory_state.add_supported_bond(
        ctx.accounts.bond_mint.key(),
        payment_feed.payment_mint,
        min_creation_amount,
        min_redemption_amount,
    )?;

    emit!(BondAdded {
        bond_mint: ctx.accounts.bond_mint.key(),
        payment_mint: payment_feed.payment_mint,
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}