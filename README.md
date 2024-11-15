# Stable.fun - Complete Instruction Specifications

```bash
src/
├── lib.rs                 # Program entrypoint
├── instructions/          # Instruction processors
│   ├── mod.rs
│   ├── create.rs         # Stablecoin creation
│   ├── mint.rs           # Minting logic
│   ├── redeem.rs         # Redemption logic
│   └── admin.rs          # Admin functions
├── state/                # Program state definitions
│   ├── mod.rs
│   ├── config.rs         # Stablecoin configuration
│   └── factory.rs        # Global factory state
├── contexts/             # Account validation
│   ├── mod.rs
│   ├── create.rs
│   ├── mint.rs
│   └── redeem.rs
├── utils/                # Helper functions
│   ├── mod.rs
│   ├── math.rs          # Math operations
│   ├── validation.rs     # Input validation
│   └── oracle.rs        # Oracle interactions
├── errors.rs            # Custom errors
└── constants.rs         # Program constants
```

## Factory Management Instructions

### 1. `initialize_factory`

**Purpose**: First-time setup of the protocol
**Functionality**:

- Creates global factory state account
- Sets initial admin authority
- Configures base protocol parameters
- Initializes supported stablebond list
- Sets up fee collectors
- Establishes initial oracle configurations
**Parameters**:
- admin_authority: Pubkey
- min_collateral_ratio: u64
- base_fee_rate: u16
**When Used**: Once at protocol deployment

### 2. `update_factory_config`

**Purpose**: Update global protocol parameters
**Functionality**:

- Modify admin settings
- Update fee configurations
- Adjust global parameters
- Modify emergency settings
**Parameters**:
- new_admin: Option<Pubkey>
- new_fee_rate: Option<u16>
- new_fee_recipient: Option<Pubkey>
**When Used**: As needed by protocol admin

## Stablecoin Creation and Management

### 3. `create_stablecoin`

**Purpose**: Create new stablecoin instance
**Functionality**:

- Creates stablecoin configuration
- Initializes token mint
- Sets up oracle connections
- Configures initial parameters
- Establishes collateral vault
**Parameters**:
- name: String
- symbol: String
- target_currency: String
- initial_collateral_ratio: u64
- oracle_config: OracleConfig
**When Used**: When users want to create new stablecoin

### 4. `update_stablecoin_config`

**Purpose**: Modify stablecoin parameters
**Functionality**:

- Update collateral requirements
- Modify fee structure
- Adjust yield settings
- Update oracle configurations
**Parameters**:
- new_collateral_ratio: Option<u64>
- new_fees: Option<FeeConfig>
- new_oracle_settings: Option<OracleConfig>
**When Used**: When stablecoin parameters need adjustment

## Oracle Management

### 5. `add_oracle_source`

**Purpose**: Add new price oracle source
**Functionality**:

- Register new oracle feed
- Set oracle weight
- Configure price deviation limits
- Update aggregation logic
**Parameters**:
- oracle_pubkey: Pubkey
- oracle_weight: u8
- price_deviation_limit: u8
**When Used**: Adding redundancy/new price sources

### 6. `update_oracle_weights`

**Purpose**: Modify oracle importance
**Functionality**:

- Update oracle weightings
- Adjust minimum sources required
- Modify deviation parameters
**Parameters**:
- oracle_weights: Vec<(Pubkey, u8)>
- min_sources: u8
**When Used**: Rebalancing oracle importance

## Token Operations

### 7. `mint_tokens`

**Purpose**: Create new stablecoins
**Functionality**:

- Verify collateral deposit
- Check oracle prices
- Apply collateral ratio
- Mint new tokens
- Calculate/collect fees
**Parameters**:
- amount: u64
- slippage_tolerance: u8
**When Used**: Users minting new stablecoins

### 8. `redeem_tokens`

**Purpose**: Redeem stablecoins for collateral
**Functionality**:

- Burn stablecoins
- Return collateral
- Process fees
- Update metrics
**Parameters**:
- amount: u64
- min_collateral_out: u64
**When Used**: Users exiting positions

## Collateral Management

### 9. `auto_rebalance`

**Purpose**: Maintain collateral health
**Functionality**:

- Check collateral ratios
- Adjust requirements dynamically
- Process required actions
- Update state
**Parameters**:
- None (automated check)
**When Used**: Regular maintenance/price changes

### 10. `liquidate_position`

**Purpose**: Handle undercollateralized positions
**Functionality**:

- Check liquidation triggers
- Process liquidation
- Distribute rewards
- Update position state
**Parameters**:
- position_owner: Pubkey
- liquidation_amount: u64
**When Used**: Positions below safety threshold

## Stability Mechanisms

### 11. `maintain_peg`

**Purpose**: Ensure price stability
**Functionality**:

- Monitor price deviation
- Adjust supply/demand incentives
- Process stability fees
- Update parameters
**Parameters**:
- None (automated mechanism)
**When Used**: Price deviating from peg

### 12. `claim_stability_reward`

**Purpose**: Reward stability providers
**Functionality**:

- Calculate rewards
- Distribute incentives
- Update metrics
**Parameters**:
- reward_period: u64
**When Used**: After stability actions

## Yield Management

### 13. `collect_yield`

**Purpose**: Process stablebond yields
**Functionality**:

- Calculate yield
- Distribute rewards
- Update yield metrics
- Handle reinvestment
**Parameters**:
- yield_period: u64
**When Used**: Regular yield collection

### 14. `distribute_yield`

**Purpose**: Share yields with stakeholders
**Functionality**:

- Calculate shares
- Process distributions
- Update records
**Parameters**:
- distribution_period: u64
**When Used**: After yield collection

## Emergency Controls

### 15. `pause_stablecoin`

**Purpose**: Emergency protocol pause
**Functionality**:

- Suspend operations
- Secure assets
- Emit notifications
**Parameters**:
- pause_reason: String
**When Used**: Emergency situations

### 16. `resume_stablecoin`

**Purpose**: Restore operations
**Functionality**:

- Verify conditions
- Resume functionality
- Update states
**Parameters**:
- safety_confirmation: bool
**When Used**: After emergency resolution

## Parameter Governance

### 17. `propose_parameter_update`

**Purpose**: Suggest parameter changes
**Functionality**:

- Submit proposal
- Validate bounds
- Check cooldowns
- Update if valid
**Parameters**:
- parameter_type: ParameterType
- new_value: u64
**When Used**: Parameter adjustment needed

### 18. `add_supported_stablebond`

**Purpose**: Add new collateral type

**Functionality**:

- Verify stablebond
- Add to whitelist
- Configure parameters
- Set up tracking
**Parameters**:
- stablebond_mint: Pubkey
- initial_parameters: StablebondConfig
**When Used**: Adding new collateral options

### 19. `remove_supported_stablebond`

**Purpose**: Remove collateral type
**Functionality**:

- Handle existing positions
- Remove from whitelist
- Update configurations
**Parameters**:
- stablebond_mint: Pubkey
**When Used**: Deprecating collateral types
