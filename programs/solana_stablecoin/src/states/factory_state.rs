// states/factory_state.rs
use anchor_lang::prelude::*;
use stablebond_sdk::{
    accounts::{Bond, PaymentFeed},
    find_bond_pda, find_payment_feed_pda,
};
use crate::states::{bond_config::StablebondConfig, bond_tracker::BondCollateralInfo};
use crate::errors::StablecoinError;
use crate::constants::*;

#[account]
#[derive(InitSpace)]
pub struct FactoryState {
    // Authority and control
    pub admin: Pubkey,                    // Account authorized to update initialize factory and update factory configs
    pub fee_vault: Pubkey,            // Account that holds the fees. Mint fees, Yield fees, Burn fees.
    pub is_paused: bool,                  // If the factory has been paused or not
    
    // Protocol parameters
    pub min_collateral_ratio: u16,        // The minimum collateral ratio needed to mint a stablecoin (e.g. 15000 = 150%)
    pub base_fee_rate: u16,               // The default fee percentage that goes to the fee_recipient account
    pub stablecoin_count: u32,           // tracks the total number of different stablecoins created 

    pub last_update: i64,                // Unix timestamp of the last protocol update

    // Stablebond configuration
    #[max_len(10)]
    pub allowed_bond_configs: Vec<StablebondConfig>, // list of validated stablebonds

    #[max_len(10)]
    pub bond_collateral_tracking: Vec<BondCollateralInfo>,

    #[max_len(5)]
    pub authorized_collectors: Vec<Pubkey>,
    
    // Admin controls
    pub protocol_version: u16,           // For tracking protocol upgrades
    pub bump: u8,                        // PDA bump
    pub reserved: [u8; 32],              // 32 bytes of free space to prevent account size issues and for future upgrades
}

impl FactoryState {

    pub fn is_authorized_collector(&self, collector: Pubkey) -> bool {
        self.authorized_collectors.contains(&collector)
    }

    pub fn add_collector(&mut self, collector: Pubkey) -> Result<()> {
        require!(
            !self.authorized_collectors.contains(&collector),
            StablecoinError::CollectorAlreadyExists
        );

        require!(
            self.authorized_collectors.len() < MAX_ALLOWED_COLLECTORS,
            StablecoinError::MaxCollectorsReached
        );

        self.authorized_collectors.push(collector);
        Ok(())
    }

    pub fn has_active_collateral(&self, bond_mint: &Pubkey) -> Result<bool> {
        if let Some(tracking) = self.bond_collateral_tracking
            .iter()
            .find(|t| t.bond_mint == *bond_mint)
        {
            Ok(tracking.total_collateral > 0)
        } else {
            Ok(false)
        }
    }

    // Update collateral tracking when minting/burning
    pub fn update_bond_collateral(
        &mut self,
        bond_mint: &Pubkey,
        amount: u64,
        is_deposit: bool
    ) -> Result<()> {
        let tracking = self.bond_collateral_tracking
            .iter_mut()
            .find(|t| t.bond_mint == *bond_mint)
            .ok_or(StablecoinError::BondNotFound)?;

        if is_deposit {
            tracking.total_collateral = tracking.total_collateral
                .checked_add(amount)
                .ok_or(StablecoinError::MathOverflow)?;
        } else {
            tracking.total_collateral = tracking.total_collateral
                .checked_sub(amount)
                .ok_or(StablecoinError::InsufficientCollateral)?;
        }

        Ok(())
    }

    pub fn add_supported_bond(
        &mut self,
        bond_mint: Pubkey,
        payment_mint: Pubkey,
        min_creation_amount: u64,
        min_redemption_amount: u64,
    ) -> Result<()> {
        // Ensure we don't exceed max bonds
        require!(
            self.allowed_bond_configs.len() < 10,  // max_len(10)
            StablecoinError::TooManyBonds
        );

        // Check if bond already exists
        require!(
            !self.allowed_bond_configs.iter().any(|c| c.bond_mint == bond_mint),
            StablecoinError::BondAlreadyExists
        );

        let config = StablebondConfig {
            bond_mint,
            payment_mint,
            admin: self.admin,
            min_creation_amount,
            min_redemption_amount,
            is_enabled: true,
            custom_fee_rate: None,
        };

        self.allowed_bond_configs.push(config);
        Ok(())
    }

    /// Checks if a bond mint is supported and valid
    /// Returns Ok(true) if the bond is supported and enabled
    pub fn is_bond_supported<'info>(
        &self,
        bond_mint: &Pubkey,
        bond_info: &AccountInfo<'info>,
    ) -> Result<bool> {
        msg!("Checking if bond {} is supported...", bond_mint);
    
        // 1. Check if bond is in our allowed configs
        if let Some(config) = self.allowed_bond_configs
            .iter()
            .find(|config| config.bond_mint == *bond_mint)
        {
            msg!("Found bond config in allowed list");
    
            // 2. Check if bond is enabled
            if !config.is_enabled {
                msg!("Bond is disabled in config");
                return Ok(false);
            }
            
            // 3. Verify the account provided matches Etherfuse's PDA
            let (bond_pda, _) = find_bond_pda(*bond_mint);
            msg!("Expected bond PDA: {}", bond_pda);
            msg!("Provided bond account: {}", bond_info.key());
            
            require!(
                bond_info.key() == bond_pda,
                StablecoinError::InvalidBondAccount
            );
    
            // 4. Verify we can deserialize the bond data
            let _bond = Bond::try_from_slice(&bond_info.try_borrow_data()?)?;
            msg!("Successfully deserialized bond data");
            
            Ok(true)
        } else {
            msg!("Bond not found in allowed configs");
            Ok(false)
        }
    }

    pub fn validate_bond<'info>(
        &self,
        bond_mint: &Pubkey,
        bond_info: &AccountInfo<'info>,
        payment_feed_info: &AccountInfo<'info>,
    ) -> Result<()> {
        msg!("Starting detailed bond validation for {}", bond_mint);
    
        // 1. Get and verify bond config exists
        let config = self.allowed_bond_configs
            .iter()
            .find(|config| config.bond_mint == *bond_mint)
            .ok_or_else(|| {
                msg!("Bond not found in allowed configs");
                StablecoinError::UnsupportedBond
            })?;
    
        // 2. Check if bond is enabled
        require!(
            config.is_enabled,
            StablecoinError::BondDisabled
        );
        msg!("Bond is enabled");
    
        // 3. Deserialize and validate bond data
        msg!("Deserializing bond data");
        let bond = Bond::try_from_slice(&bond_info.try_borrow_data()?)?;
        
        // 4. Get and verify payment feed
        let feed_type = bond.payment_feed_type.clone();
        let (expected_payment_pda, _) = find_payment_feed_pda(feed_type);
        
        msg!("Expected payment feed: {}", expected_payment_pda);
        msg!("Provided payment feed: {}", payment_feed_info.key());
        
        require!(
            payment_feed_info.key() == expected_payment_pda,
            StablecoinError::InvalidPaymentFeed
        );
    
        // 5. Verify payment feed data
        let _payment_feed = PaymentFeed::try_from_slice(
            &payment_feed_info.try_borrow_data()?
        )?;
        msg!("Successfully deserialized payment feed");
    
        Ok(())
    }

    pub fn get_bond_config(&self, bond_mint: &Pubkey) -> Option<&StablebondConfig> {
        msg!("Looking up config for bond: {}", bond_mint);
        let config = self.allowed_bond_configs
            .iter()
            .find(|config| config.bond_mint == *bond_mint);
        
        if config.is_some() {
            msg!("Found config for bond");
        } else {
            msg!("No config found for bond");
        }
        
        config
    }

    pub fn get_fee_rate(&self, bond_mint: &Pubkey) -> Result<u16> {
        let config = self.get_bond_config(bond_mint)
            .ok_or(StablecoinError::BondNotFound)?;
            
        Ok(config.custom_fee_rate.unwrap_or(self.base_fee_rate))
    }
}
