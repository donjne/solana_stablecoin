// errors.rs
use anchor_lang::prelude::*;

#[error_code]
pub enum StablecoinError {
    #[msg("Invalid collateral ratio specified")]
    InvalidCollateralRatio,

    #[msg("Collateral ratio below minimum required")]
    CollateralRatioTooLow,

    #[msg("The USDC Mint is incorrect")]
    InvalidUSDCMint,

    #[msg("Invalid fee rate specified")]
    InvalidFeeRate,

    #[msg("Factory is already initialized")]
    FactoryAlreadyInitialized,

    #[msg("Factory is currently paused")]
    FactoryPaused,

    #[msg("Invalid admin authority")]
    InvalidAdminAuthority,

    #[msg("Operation exceeds maximum allowed value")]
    ExceedsMaximumValue,

    #[msg("Unauthorized access")]
    Unauthorized,

    #[msg("Invalid stablecoin name length")]
    InvalidNameLength,

    #[msg("Invalid payout account")]
    InvalidPayoutAccount,

    #[msg("No collateral to collect from")]
    NoCollateralToCollectFrom,

    #[msg("Failed to deserialize payout")]
    PayoutDeserializationError,

    #[msg("Yield collect is too frequent")]
    YieldCollectionTooFrequent,

    #[msg("Invalid stablecoin symbol length")]
    InvalidSymbolLength,

    #[msg("Does not meet requirements")]
    StaleError,

    #[msg("Stablecoin is already paused")]
    AlreadyPaused,

    #[msg("Stablecoin is not paused")]
    NotPaused,

    #[msg("Bond mint mismatch")]
    BondMintMismatch,

    #[msg("Rebase is too early")]
    RebaseTooEarly,

    #[msg("Maximum number of stablecoins reached")]
    MaxStablecoinsReached,

    #[msg("Invalid oracle configuration")]
    InvalidOracleConfig,

    #[msg("Oracle price is too stale")]
    StaleOraclePrice,

    #[msg("Oracle price is not valid")]
    InvalidOraclePrice,

    #[msg("Oracle confidence is too low")]
    LowOracleConfidence,

    #[msg("Invalid target currency")]
    InvalidTargetCurrency,

    #[msg("Insufficient collateral provided")]
    InsufficientCollateral,

    #[msg("Invalid symbol format")]
    InvalidSymbolFormat,

    #[msg("Collateral ratio exceeds maximum allowed")]
    CollateralRatioTooHigh,

    #[msg("PDA bump not found")]
    BumpNotFound,

    #[msg("Math overflow occurred")]
    MathOverflow,

    #[msg("Invalid oracle staleness configuration")]
    InvalidOracleStaleness,

    #[msg("Invalid oracle confidence interval")]
    InvalidOracleConfidence,

    #[msg("Invalid oracle update count")]
    InvalidOracleUpdateCount,

    #[msg("Invalid price deviation threshold")]
    InvalidPriceDeviation,

    #[msg("Insufficient oracle updates")]
    InsufficientOracleUpdates,

    #[msg("Price deviation exceeds maximum allowed")]
    ExcessivePriceDeviation,

    #[msg("Invalid KYC account")]
    InvalidKycAccount,

    #[msg("Fee is too large")]
    FeeTooLarge,

    #[msg("Maximum amount of users reached")]
    TooManyUsers,

    #[msg("Dividing by zero is not allowed")]
    DivideByZero,

    #[msg("The timestamp is invalid")]
    InvalidTimestamp,

    #[msg("No oracle account identified")]
    MissingOracleAccount,

    #[msg("Not enough stablecoins")]
    InsufficientStablecoinBalance,

    #[msg("Stablecoin paused")]
    StablecoinPaused,

    #[msg("Redeem amount is too small")]
    RedeemAmountTooSmall,

    #[msg("User has insufficient share")]
    InsufficientUserShare,
    
    #[msg("TUser share was not found")]
    UserShareNotFound,

    #[msg("Invalid mint amount")]
    InvalidMintAmount,

    #[msg("Invalid price")]
    InvalidPrice,

    #[msg("No yield to distribute")]
    NoYieldToDistribute,

    #[msg("No user position")]

    NoUserPosition,

    #[msg("No total supply")]
    NoTotalSupply,

    #[msg("Excessive slippage")]
    ExcessiveSlippage,

    #[msg("Supply cap exceeded")]
    SupplyCapExceeded,

    #[msg("Bond is not supported")]
    UnsupportedBond,

    #[msg("Only admin can perform this action")]
    UnauthorizedAdmin,

    #[msg("Only authority can perform this action")]
    UnauthorizedAccess,

    #[msg("Maximum number of supported bonds reached")]
    TooManyBonds,

    #[msg("Not the owner of token account")]
    InvalidTokenAccountOwner,

    #[msg("Bond is invalid")]
    InvalidBondMint,

    #[msg("Bond is already supported")]
    BondAlreadyExists,

    #[msg("Yield collector already exists")]
    CollectorAlreadyExists,

    #[msg("Yield collectors has reached limit")]
    MaxCollectorsReached,

    #[msg("Invalid bond account")]
    InvalidBondAccount,

    #[msg("Invalid payment feed")]
    InvalidPaymentFeed,
    
    #[msg("Bond is currently disabled")]
    BondDisabled,
    
    #[msg("Bond has active collateral and cannot be removed")]
    ActiveCollateralExists,

    #[msg("Bond not found")]
    BondNotFound,

    #[msg("Deposit amount below minimum required")]
    DepositTooSmall,

    #[msg("Name is invalid")]
    InvalidName,

    #[msg("Symbol is invalid")]
    InvalidSymbol,
}