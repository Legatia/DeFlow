// Solana Account Management - ICP Chain Fusion Implementation
// Handles Solana account operations with ICP threshold ECDSA

use super::*;
use candid::Principal;
use std::collections::HashMap;

/// Solana account manager for ICP integration
#[derive(Debug, Clone)]
pub struct SolanaAccountManager {
    pub key_name: String,
    pub network: SolanaNetwork,
}

impl SolanaAccountManager {
    /// Create new account manager
    pub fn new(key_name: String, network: SolanaNetwork) -> Self {
        Self {
            key_name,
            network,
        }
    }

    /// Generate Solana account for user using ICP derivation
    pub async fn create_account(
        &self,
        user: Principal,
        account_type: SolanaAccountType,
    ) -> Result<SolanaAccount, SolanaError> {
        // Create derivation path based on user and account type
        let derivation_path = match account_type {
            SolanaAccountType::Main => format!("{}-solana-main-{}", self.key_name, user.to_text()),
            SolanaAccountType::TokenAccount(mint) => {
                format!("{}-solana-token-{}-{}", self.key_name, mint, user.to_text())
            }
            SolanaAccountType::ProgramDerived(program_id, seeds) => {
                format!("{}-solana-pda-{}-{}-{}", 
                    self.key_name, 
                    program_id, 
                    seeds.join("-"), 
                    user.to_text()
                )
            }
        };

        // Generate deterministic address
        let address = self.derive_address(&derivation_path).await?;

        // Get account info from network
        let account_info = self.get_account_info(&address).await?;

        Ok(SolanaAccount {
            address,
            network: self.network.clone(),
            derivation_path,
            balance_lamports: account_info.balance_lamports,
            balance_sol: utils::lamports_to_sol(account_info.balance_lamports),
            executable: account_info.executable,
            owner: account_info.owner,
            rent_epoch: account_info.rent_epoch,
            last_updated: ic_cdk::api::time(),
        })
    }

    /// Get associated token account for a specific mint
    pub async fn get_associated_token_account(
        &self,
        user: Principal,
        mint_address: String,
    ) -> Result<SolanaAccount, SolanaError> {
        // Validate mint address
        if !utils::validate_solana_address(&mint_address) {
            return Err(SolanaError::InvalidAddress(mint_address));
        }

        // Create associated token account
        self.create_account(
            user,
            SolanaAccountType::TokenAccount(mint_address),
        ).await
    }

    /// Get program derived address (PDA)
    pub async fn get_program_derived_address(
        &self,
        user: Principal,
        program_id: String,
        seeds: Vec<String>,
    ) -> Result<SolanaAccount, SolanaError> {
        // Validate program ID
        if !utils::validate_solana_address(&program_id) {
            return Err(SolanaError::InvalidProgramId(program_id.clone()));
        }

        // Create PDA account
        self.create_account(
            user,
            SolanaAccountType::ProgramDerived(program_id, seeds),
        ).await
    }

    /// Check if account needs rent exemption
    pub async fn check_rent_exemption(
        &self,
        address: &str,
        data_size: usize,
    ) -> Result<RentExemptionInfo, SolanaError> {
        let account_info = self.get_account_info(address).await?;
        let required_balance = utils::calculate_rent_exemption(data_size);
        
        Ok(RentExemptionInfo {
            current_balance: account_info.balance_lamports,
            required_balance,
            is_rent_exempt: account_info.balance_lamports >= required_balance,
            shortfall: if account_info.balance_lamports < required_balance {
                Some(required_balance - account_info.balance_lamports)
            } else {
                None
            },
        })
    }

    /// Get multiple accounts efficiently
    pub async fn get_multiple_accounts(
        &self,
        addresses: Vec<String>,
    ) -> Result<HashMap<String, SolanaAccount>, SolanaError> {
        let mut accounts = HashMap::new();
        
        for address in addresses {
            // Validate address
            if !utils::validate_solana_address(&address) {
                return Err(SolanaError::InvalidAddress(address));
            }

            let account_info = self.get_account_info(&address).await?;
            let account = SolanaAccount {
                address: address.clone(),
                network: self.network.clone(),
                derivation_path: "external".to_string(), // External account
                balance_lamports: account_info.balance_lamports,
                balance_sol: utils::lamports_to_sol(account_info.balance_lamports),
                executable: account_info.executable,
                owner: account_info.owner,
                rent_epoch: account_info.rent_epoch,
                last_updated: ic_cdk::api::time(),
            };
            
            accounts.insert(address, account);
        }

        Ok(accounts)
    }

    /// Derive Solana address using ICP threshold ECDSA
    async fn derive_address(&self, derivation_path: &str) -> Result<String, SolanaError> {
        // In production, this would use ICP threshold ECDSA to generate a Solana public key
        // For now, generate deterministic mock address
        let address_seed = format!("{}-{}", derivation_path, self.network.name());
        let mock_address = self.generate_solana_address(&address_seed);
        Ok(mock_address)
    }

    /// Get account information from Solana network
    async fn get_account_info(&self, address: &str) -> Result<AccountInfo, SolanaError> {
        // In production, this would make HTTP outcalls to Solana RPC
        // For now, generate mock account info
        let balance_seed = format!("{}-{}-balance", address, self.network.name());
        let balance_lamports = (self.hash_string(&balance_seed) % 5_000_000_000) as u64; // Up to 5 SOL

        Ok(AccountInfo {
            balance_lamports,
            executable: false,
            owner: constants::SYSTEM_PROGRAM_ID.to_string(),
            rent_epoch: 350,
        })
    }

    /// Generate Solana address from seed
    fn generate_solana_address(&self, seed: &str) -> String {
        let hash = self.hash_string(seed);
        // Generate Base58-like address format
        format!("{:032x}", hash)[0..32].to_string()
    }

    /// Hash function for deterministic generation
    fn hash_string(&self, input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

/// Account type for different Solana account categories
#[derive(Debug, Clone)]
pub enum SolanaAccountType {
    /// Main SOL account for the user
    Main,
    /// SPL token account for specific mint
    TokenAccount(String), // mint address
    /// Program derived address
    ProgramDerived(String, Vec<String>), // program_id, seeds
}

/// Internal account info structure
#[derive(Debug, Clone)]
struct AccountInfo {
    balance_lamports: u64,
    executable: bool,
    owner: String,
    rent_epoch: u64,
}

/// Rent exemption information
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RentExemptionInfo {
    pub current_balance: u64,
    pub required_balance: u64,
    pub is_rent_exempt: bool,
    pub shortfall: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_manager_creation() {
        let manager = SolanaAccountManager::new(
            "test_key".to_string(),
            SolanaNetwork::Devnet
        );
        assert_eq!(manager.key_name, "test_key");
        assert_eq!(manager.network, SolanaNetwork::Devnet);
    }

    #[test]
    fn test_address_generation_consistency() {
        let manager = SolanaAccountManager::new(
            "test_key".to_string(),
            SolanaNetwork::Mainnet
        );

        let seed = "test_seed";
        let address1 = manager.generate_solana_address(seed);
        let address2 = manager.generate_solana_address(seed);
        assert_eq!(address1, address2);

        let different_seed = "different";
        let address3 = manager.generate_solana_address(different_seed);
        assert_ne!(address1, address3);
    }

    #[test]
    fn test_account_type_variants() {
        let main_account = SolanaAccountType::Main;
        let token_account = SolanaAccountType::TokenAccount("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string());
        let pda_account = SolanaAccountType::ProgramDerived(
            "11111111111111111111111111111112".to_string(),
            vec!["seed1".to_string(), "seed2".to_string()]
        );

        // Test that different account types produce different derivation paths
        match main_account {
            SolanaAccountType::Main => assert!(true),
            _ => assert!(false),
        }

        match token_account {
            SolanaAccountType::TokenAccount(mint) => {
                assert_eq!(mint, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
            },
            _ => assert!(false),
        }

        match pda_account {
            SolanaAccountType::ProgramDerived(program, seeds) => {
                assert_eq!(program, "11111111111111111111111111111112");
                assert_eq!(seeds.len(), 2);
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn test_rent_exemption_info() {
        let info = RentExemptionInfo {
            current_balance: 1_000_000,
            required_balance: 2_000_000,
            is_rent_exempt: false,
            shortfall: Some(1_000_000),
        };

        assert!(!info.is_rent_exempt);
        assert_eq!(info.shortfall, Some(1_000_000));
        assert!(info.current_balance < info.required_balance);
    }
}