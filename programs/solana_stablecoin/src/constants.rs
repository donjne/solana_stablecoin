// constants.rs
use anchor_lang::prelude::*;

// Factory state seeds
pub const FACTORY_STATE_SEED: &[u8] = b"factory_state";
pub const STABLECOIN_SEED: &[u8] = b"stablecoin";
pub const USER_STATE_SEED: &[u8] = b"user_state";

pub const PRICE_SCALE: u128 = 1_000_000;  // 6 decimals of precision
pub const BPS_SCALE: u16 = 10_000;        // Basis points (100% = 10000)
pub const PROTOCOL_FEE_BPS: u16 = 1_000;

// Collateral ratio constants (in basis points)
pub const MIN_ALLOWED_COLLATERAL_RATIO: u16 = 12_000;  // 120% minimum collateral ratio
pub const MAX_ALLOWED_COLLATERAL_RATIO: u16 = 65_000; // 65% maximum collateral ratio
pub const DEFAULT_COLLATERAL_RATIO: u16 = 15_000;      // 150% default collateral ratio

// Fee configuration (in basis points)
pub const MIN_FEE_RATE_BPS: u16 = 10;                 // 0.01% minimum fee rate
pub const MAX_FEE_RATE_BPS: u16 = 10_000;          // 100% maximum fee rate
pub const DEFAULT_BASE_FEE_RATE_BPS: u16 = 30;      // 0.3% default fee rate

pub const BASE_INTEREST_RATE: u64 = 500; // 5% base interest rate

// For regular rate updates
pub const RATE_UPDATE_INTERVAL: i64 = 7 * 24 * 60 * 60;  // 1 week

// For checking if feed data is stale
pub const RATE_FRESHNESS_THRESHOLD: i64 = 15 * 60;  // 15 minutes

// Buffer sizes
pub const RESERVE_SPACE: usize = 32;               // Reserved space for future upgrades

// Maximum allowed bonds
pub const MAX_ALLOWED_BONDS: usize = 10;

// Maximum allowed bonds
pub const MAX_ALLOWED_COLLECTORS: usize = 5;

// Time calculation constants
pub const MIN_YIELD_COLLECTION_INTERVAL: i64 = 7 * 24 * 60 * 60; // 7 days in seconds
pub const REBASE_INTERVAL: i64 = 7 * 24 * 60 * 60; // 1 week in seconds
pub const YEAR_IN_SECONDS: i64 = 365 * 24 * 60 * 60;

// USDC mint
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

// Stablecoin limits
pub const STABLECOIN_DECIMALS: u8 = 6;           // Decimal places for stablecoin

pub const ORACLE_STALENESS_THRESHOLD: i64 = 300; // 5 minutes in seconds, 600 for 10 minutes

pub const SOL_FEE_VAULT_SEED: &[u8] = b"sol_fee_vault";

