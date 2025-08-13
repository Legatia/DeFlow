// Solana DeFi Integration - ICP Chain Fusion Implementation  
// Day 10: Solana integration with high-performance blockchain support

pub mod service;
pub mod accounts;
pub mod programs;
pub mod tokens;
pub mod icp_solana;
// Temporarily commented out due to WASM compatibility issues with sol-rpc dependencies
// pub mod official_icp;
pub mod pure_icp;

use candid::{CandidType, Deserialize};
use serde::Serialize;
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument
};

// Re-export sub-modules
pub use service::SolanaDeFiService;
pub use accounts::SolanaAccountManager;
pub use programs::SolanaProgramManager;
pub use tokens::SolanaTokenManager;
// ICP-compliant service using official SOL RPC canister
pub use icp_solana::IcpSolanaService;
// Official ICP-compliant service based on dfinity/sol-rpc-canister example
// Temporarily commented out due to WASM compatibility issues
// pub use official_icp::OfficialIcpSolanaService;
// Pure ICP service using only built-in ICP capabilities
pub use pure_icp::PureIcpSolanaService;

/// Solana network types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SolanaNetwork {
    Mainnet,
    Devnet,
    Testnet,
}

impl SolanaNetwork {
    /// Get the RPC endpoint for the network
    pub fn rpc_endpoint(&self) -> &'static str {
        match self {
            SolanaNetwork::Mainnet => "https://api.mainnet-beta.solana.com",
            SolanaNetwork::Devnet => "https://api.devnet.solana.com", 
            SolanaNetwork::Testnet => "https://api.testnet.solana.com",
        }
    }

    /// Get the network name
    pub fn name(&self) -> &'static str {
        match self {
            SolanaNetwork::Mainnet => "Mainnet Beta",
            SolanaNetwork::Devnet => "Devnet",
            SolanaNetwork::Testnet => "Testnet",
        }
    }
}

/// Solana account representation  
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq)]
pub struct SolanaAccount {
    pub address: String, // Base58 encoded public key
    pub network: SolanaNetwork,
    pub derivation_path: String,
    pub balance_lamports: u64, // SOL balance in lamports (1 SOL = 1e9 lamports)
    pub balance_sol: f64,
    pub executable: bool,
    pub owner: String, // Program that owns this account
    pub rent_epoch: u64,
    pub last_updated: u64,
}

/// Solana transaction parameters
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SolanaTransactionParams {
    pub from: String,
    pub to: String,
    pub amount_lamports: u64,
    pub recent_blockhash: String,
    pub fee_payer: Option<String>,
    pub instructions: Vec<SolanaInstruction>,
}

/// Solana instruction
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SolanaInstruction {
    pub program_id: String,
    pub accounts: Vec<SolanaAccountMeta>,
    pub data: Vec<u8>,
}

/// Account metadata for instructions
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SolanaAccountMeta {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

/// Solana transaction result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SolanaTransactionResult {
    pub success: bool,
    pub signature: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub amount_lamports: u64,
    pub amount_sol: f64,
    pub fee_lamports: u64,
    pub block_height: Option<u64>,
    pub confirmation_status: SolanaConfirmationStatus,
    pub error_message: Option<String>,
}

/// Solana confirmation status
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum SolanaConfirmationStatus {
    Processed,
    Confirmed, 
    Finalized,
}

/// Solana portfolio containing SOL and SPL tokens
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SolanaPortfolio {
    pub accounts: Vec<SolanaAccount>,
    pub total_sol: f64,
    pub total_value_usd: f64,
    pub spl_tokens: Vec<SplTokenBalance>,
    pub last_updated: u64,
}

/// SPL Token balance
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SplTokenBalance {
    pub mint: String, // Token mint address
    pub symbol: String,
    pub name: String,
    pub balance: u64, // Raw token amount
    pub decimals: u8,
    pub balance_formatted: f64,
    pub value_usd: Option<f64>,
}

/// Solana program interaction result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SolanaProgramResult {
    pub success: bool,
    pub signature: Option<String>,
    pub program_id: String,
    pub instruction_data: Vec<u8>,
    pub accounts_used: Vec<String>,
    pub compute_units_consumed: Option<u64>,
    pub logs: Vec<String>,
    pub error_message: Option<String>,
}

/// Error types for Solana operations
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum SolanaError {
    InvalidAddress(String),
    InsufficientBalance { required: u64, available: u64 },
    NetworkError(String),
    TransactionFailed(String),
    ProgramError(String),
    AccountNotFound(String),
    InvalidInstruction(String),
    SerializationError(String),
    /// ICP-specific errors
    ThresholdEcdsaError(String),
    RpcError(String),
    InsufficientCycles(String),
    /// Solana-specific errors
    InvalidBlockhash(String),
    InsufficientRentExemption(String),
    InvalidProgramId(String),
    AccountDataTooLarge(String),
    ComputeBudgetExceeded(String),
}

impl std::fmt::Display for SolanaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolanaError::InvalidAddress(addr) => write!(f, "Invalid Solana address: {}", addr),
            SolanaError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: need {} lamports, have {} lamports", required, available)
            }
            SolanaError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            SolanaError::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            SolanaError::ProgramError(msg) => write!(f, "Program error: {}", msg),
            SolanaError::AccountNotFound(addr) => write!(f, "Account not found: {}", addr),
            SolanaError::InvalidInstruction(msg) => write!(f, "Invalid instruction: {}", msg),
            SolanaError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SolanaError::ThresholdEcdsaError(msg) => write!(f, "Threshold ECDSA error: {}", msg),
            SolanaError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            SolanaError::InsufficientCycles(msg) => write!(f, "Insufficient cycles: {}", msg),
            SolanaError::InvalidBlockhash(hash) => write!(f, "Invalid blockhash: {}", hash),
            SolanaError::InsufficientRentExemption(msg) => write!(f, "Insufficient rent exemption: {}", msg),
            SolanaError::InvalidProgramId(id) => write!(f, "Invalid program ID: {}", id),
            SolanaError::AccountDataTooLarge(msg) => write!(f, "Account data too large: {}", msg),
            SolanaError::ComputeBudgetExceeded(msg) => write!(f, "Compute budget exceeded: {}", msg),
        }
    }
}

/// Solana context for ICP Chain Fusion
#[derive(Clone, Debug)]
pub struct SolanaContext {
    pub network: SolanaNetwork,
    pub key_name: String,
}

impl SolanaContext {
    pub fn new(network: SolanaNetwork, key_name: String) -> Self {
        Self {
            network,
            key_name,
        }
    }
    
    pub fn ecdsa_key_id(&self) -> EcdsaKeyId {
        EcdsaKeyId {
            curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
            name: self.key_name.clone(),
        }
    }
}

/// Constants for Solana integration
pub mod constants {
    /// Lamports per SOL (10^9)
    pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
    
    /// Minimum rent-exempt balance for an account
    pub const MIN_RENT_EXEMPT_BALANCE: u64 = 890_880;
    
    /// System program ID
    pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111112";
    
    /// SPL Token program ID
    pub const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    
    /// Associated Token Account program ID
    pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    
    /// Maximum compute units per transaction
    pub const MAX_COMPUTE_UNITS: u32 = 1_400_000;
    
    /// Default compute unit price (micro-lamports)
    pub const DEFAULT_COMPUTE_UNIT_PRICE: u64 = 1;
}

/// Utility functions for Solana operations  
pub mod utils {
    use super::constants::*;
    
    /// Convert lamports to SOL
    pub fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / LAMPORTS_PER_SOL as f64
    }
    
    /// Convert SOL to lamports
    pub fn sol_to_lamports(sol: f64) -> u64 {
        (sol * LAMPORTS_PER_SOL as f64) as u64
    }
    
    /// Validate Solana address format (Base58)
    pub fn validate_solana_address(address: &str) -> bool {
        // Basic validation - should be 32-44 characters, Base58 encoded
        if address.len() < 32 || address.len() > 44 {
            return false;
        }
        
        // Check if all characters are valid Base58
        const BASE58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        address.chars().all(|c| BASE58_ALPHABET.contains(&(c as u8)))
    }
    
    /// Calculate rent exemption amount
    pub fn calculate_rent_exemption(data_size: usize) -> u64 {
        // Simplified calculation - in practice this would call Solana RPC
        let base_cost = MIN_RENT_EXEMPT_BALANCE;
        let data_cost = (data_size as u64) * 6960; // ~6960 lamports per byte
        base_cost + data_cost
    }
    
    /// Estimate transaction fee
    pub fn estimate_transaction_fee(num_signatures: u8, compute_units: u32) -> u64 {
        let signature_fee = (num_signatures as u64) * 5000; // 5000 lamports per signature
        let compute_fee = (compute_units as u64) * DEFAULT_COMPUTE_UNIT_PRICE;
        signature_fee + compute_fee
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_network_properties() {
        let mainnet = SolanaNetwork::Mainnet;
        assert_eq!(mainnet.name(), "Mainnet Beta");
        assert_eq!(mainnet.rpc_endpoint(), "https://api.mainnet-beta.solana.com");

        let devnet = SolanaNetwork::Devnet;
        assert_eq!(devnet.name(), "Devnet");
        assert_eq!(devnet.rpc_endpoint(), "https://api.devnet.solana.com");
    }

    #[test]
    fn test_lamports_sol_conversion() {
        use utils::{lamports_to_sol, sol_to_lamports};

        // Test lamports to SOL conversion
        assert_eq!(lamports_to_sol(1_000_000_000), 1.0); // 1 SOL
        assert_eq!(lamports_to_sol(500_000_000), 0.5); // 0.5 SOL
        assert_eq!(lamports_to_sol(1), 0.000000001); // 1 lamport

        // Test SOL to lamports conversion
        assert_eq!(sol_to_lamports(1.0), 1_000_000_000);
        assert_eq!(sol_to_lamports(0.5), 500_000_000);
        assert_eq!(sol_to_lamports(2.5), 2_500_000_000);
    }

    #[test]
    fn test_solana_address_validation() {
        use utils::validate_solana_address;

        // Valid addresses (typical Solana format)
        assert!(validate_solana_address("11111111111111111111111111111112")); // System program
        assert!(validate_solana_address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")); // SPL Token program
        
        // Invalid addresses
        assert!(!validate_solana_address("short")); // Too short
        assert!(!validate_solana_address("this_is_way_too_long_to_be_a_valid_solana_address_format")); // Too long
        assert!(!validate_solana_address("InvalidChars0OIl")); // Contains invalid Base58 chars (0, O, I, l)
        assert!(!validate_solana_address("")); // Empty
    }

    #[test]
    fn test_solana_error_display() {
        let error1 = SolanaError::InvalidAddress("invalid".to_string());
        assert!(error1.to_string().contains("Invalid Solana address"));

        let error2 = SolanaError::InsufficientBalance {
            required: 1000,
            available: 500,
        };
        assert!(error2.to_string().contains("Insufficient balance"));
        assert!(error2.to_string().contains("1000"));
        assert!(error2.to_string().contains("500"));

        let error3 = SolanaError::ProgramError("Custom program error".to_string());
        assert!(error3.to_string().contains("Program error"));
    }

    #[test]
    fn test_constants() {
        use constants::*;

        assert_eq!(LAMPORTS_PER_SOL, 1_000_000_000);
        assert_eq!(SYSTEM_PROGRAM_ID, "11111111111111111111111111111112");
        assert_eq!(SPL_TOKEN_PROGRAM_ID, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
        assert!(MIN_RENT_EXEMPT_BALANCE > 0);
        assert!(MAX_COMPUTE_UNITS > 1_000_000);
    }

    #[test]
    fn test_confirmation_status_variants() {
        let statuses = [
            SolanaConfirmationStatus::Processed,
            SolanaConfirmationStatus::Confirmed,
            SolanaConfirmationStatus::Finalized,
        ];

        // Test that all variants are different
        for i in 0..statuses.len() {
            for j in i + 1..statuses.len() {
                assert_ne!(format!("{:?}", statuses[i]), format!("{:?}", statuses[j]));
            }
        }
    }

    #[test]
    fn test_rent_exemption_calculation() {
        use utils::calculate_rent_exemption;

        let base_account = calculate_rent_exemption(0);
        assert_eq!(base_account, constants::MIN_RENT_EXEMPT_BALANCE);

        let token_account = calculate_rent_exemption(165); // Typical SPL token account size
        assert!(token_account > base_account);
    }

    #[test]
    fn test_transaction_fee_estimation() {
        use utils::estimate_transaction_fee;

        // Simple transfer (1 signature)
        let simple_fee = estimate_transaction_fee(1, 150_000);
        assert!(simple_fee >= 5000); // At least the signature fee

        // Multi-sig transaction (3 signatures)
        let multisig_fee = estimate_transaction_fee(3, 200_000);
        assert!(multisig_fee > simple_fee);
    }
}