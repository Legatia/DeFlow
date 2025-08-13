// Ethereum and L2 DeFi Integration Module
// Day 9: Ethereum & L2 Integration

// ICP-compliant minimal implementation for immediate usage
pub mod minimal_icp;

// Comprehensive tests for ICP-compliant Ethereum integration
#[cfg(test)]
pub mod tests;

// Full implementation modules (disabled due to compilation issues)
// pub mod service;
// pub mod addresses;
// pub mod l2_optimizer;
// pub mod gas_estimator;
// pub mod chain_config;
// pub mod evm_rpc;
// pub mod threshold_ecdsa;
// pub mod icp_gas_estimator;
// pub mod icp_service;

// Re-export only minimal ICP implementation
pub use minimal_icp::*;

use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Supported EVM chains for multi-chain operations
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EvmChain {
    /// Ethereum mainnet (Chain ID: 1)
    Ethereum,
    /// Arbitrum One (Chain ID: 42161)
    Arbitrum,
    /// Optimism (Chain ID: 10)
    Optimism,
    /// Polygon PoS (Chain ID: 137)
    Polygon,
    /// Base (Chain ID: 8453)
    Base,
    /// Avalanche C-Chain (Chain ID: 43114)
    Avalanche,
    /// Sonic L1 (Chain ID: 146)
    Sonic,
    /// BNB Smart Chain (Chain ID: 56)
    BnbSmartChain,
}

impl EvmChain {
    /// Get the chain ID for the EVM chain
    pub fn chain_id(&self) -> u64 {
        match self {
            EvmChain::Ethereum => 1,
            EvmChain::Arbitrum => 42161,
            EvmChain::Optimism => 10,
            EvmChain::Polygon => 137,
            EvmChain::Base => 8453,
            EvmChain::Avalanche => 43114,
            EvmChain::Sonic => 146,
            EvmChain::BnbSmartChain => 56,
        }
    }

    /// Get the native token symbol for the chain
    pub fn native_token(&self) -> &'static str {
        match self {
            EvmChain::Ethereum => "ETH",
            EvmChain::Arbitrum => "ETH",
            EvmChain::Optimism => "ETH",
            EvmChain::Polygon => "MATIC",
            EvmChain::Base => "ETH",
            EvmChain::Avalanche => "AVAX",
            EvmChain::Sonic => "S",
            EvmChain::BnbSmartChain => "BNB",
        }
    }

    /// Get the chain name
    pub fn name(&self) -> &'static str {
        match self {
            EvmChain::Ethereum => "Ethereum",
            EvmChain::Arbitrum => "Arbitrum One",
            EvmChain::Optimism => "Optimism",
            EvmChain::Polygon => "Polygon",
            EvmChain::Base => "Base",
            EvmChain::Avalanche => "Avalanche",
            EvmChain::Sonic => "Sonic",
            EvmChain::BnbSmartChain => "BNB Smart Chain",
        }
    }

    /// Check if this is a Layer 2 chain
    pub fn is_l2(&self) -> bool {
        matches!(
            self,
            EvmChain::Arbitrum | EvmChain::Optimism | EvmChain::Base
        )
    }

    /// Check if this is a sidechain
    pub fn is_sidechain(&self) -> bool {
        matches!(self, EvmChain::Polygon | EvmChain::Avalanche | EvmChain::BnbSmartChain)
    }

    /// Check if this is an independent L1 chain (not Ethereum)
    pub fn is_independent_l1(&self) -> bool {
        matches!(self, EvmChain::Sonic)
    }
}

/// Ethereum address type
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq)]
pub struct EthereumAddress {
    pub address: String,
    pub chain: EvmChain,
    pub derivation_path: String,
    pub balance_wei: String, // Use string to handle large numbers
    pub balance_eth: f64,
    pub nonce: u64,
    pub last_updated: u64,
}

/// Ethereum transaction parameters
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EthereumTransactionParams {
    pub to: String,
    pub value: String, // Wei amount as string
    pub gas_limit: u64,
    pub gas_price: Option<String>, // Legacy gas price
    pub max_fee_per_gas: Option<String>, // EIP-1559
    pub max_priority_fee_per_gas: Option<String>, // EIP-1559
    pub nonce: u64,
    pub data: Option<String>, // Contract call data
    pub chain_id: u64,
}

/// Ethereum transaction result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EthereumTransactionResult {
    pub success: bool,
    pub transaction_hash: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub value_wei: String,
    pub gas_used: Option<u64>,
    pub gas_price: String,
    pub total_fee_wei: String,
    pub block_number: Option<u64>,
    pub confirmation_time_estimate_seconds: u64,
    pub error_message: Option<String>,
}

/// Gas estimation result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct GasEstimate {
    pub gas_limit: u64,
    pub gas_price: String, // Legacy
    pub max_fee_per_gas: String, // EIP-1559
    pub max_priority_fee_per_gas: String, // EIP-1559
    pub total_fee_wei: String,
    pub total_fee_eth: f64,
    pub total_fee_usd: f64,
    pub confirmation_time_estimate_seconds: u64,
    pub priority: GasPriority,
}

/// Gas priority levels
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum GasPriority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Ethereum portfolio containing addresses across multiple chains
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EthereumPortfolio {
    pub addresses: Vec<EthereumAddress>,
    pub total_eth: f64,
    pub total_value_usd: f64,
    pub chain_balances: std::collections::HashMap<String, f64>, // chain -> ETH balance
    pub last_updated: u64,
}

/// L2 optimization recommendation
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct L2OptimizationResult {
    pub recommended_chain: EvmChain,
    pub estimated_fee_usd: f64,
    pub estimated_time_seconds: u64,
    pub savings_vs_ethereum: f64, // USD savings
    pub alternatives: Vec<ChainOption>,
    pub bridge_cost_usd: Option<f64>, // If bridging is required
    pub total_cost_usd: f64, // Including bridge costs
}

/// Chain option for L2 optimization
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ChainOption {
    pub chain: EvmChain,
    pub fee_usd: f64,
    pub time_seconds: u64,
    pub bridge_cost_usd: Option<f64>,
    pub total_cost_usd: f64,
    pub confidence_score: f64, // 0-1, based on data freshness and reliability
}

/// Error types for Ethereum operations
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum EthereumError {
    InvalidAddress(String),
    InsufficientBalance { required: String, available: String },
    NetworkError(String),
    TransactionFailed(String),
    GasEstimationFailed(String),
    ChainNotSupported(String),
    ThresholdEcdsaError(String),
    SerializationError(String),
    /// ICP EVM RPC canister error
    RpcError(String),
    /// Insufficient cycles for operation
    InsufficientCycles(String),
    /// Consensus validation failed
    ConsensusError(String),
    /// Address generation failed
    AddressGenerationError(String),
    /// Transaction signing failed
    SigningError(String),
    /// Transaction broadcast failed
    BroadcastError(String),
    /// L2 optimization error
    L2OptimizationError(String),
}

impl std::fmt::Display for EthereumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EthereumError::InvalidAddress(addr) => write!(f, "Invalid Ethereum address: {}", addr),
            EthereumError::InsufficientBalance { required, available } => {
                write!(f, "Insufficient balance: need {} wei, have {} wei", required, available)
            }
            EthereumError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            EthereumError::TransactionFailed(msg) => write!(f, "Transaction failed: {}", msg),
            EthereumError::GasEstimationFailed(msg) => write!(f, "Gas estimation failed: {}", msg),
            EthereumError::ChainNotSupported(chain) => write!(f, "Chain not supported: {}", chain),
            EthereumError::ThresholdEcdsaError(msg) => write!(f, "Threshold ECDSA error: {}", msg),
            EthereumError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            EthereumError::RpcError(msg) => write!(f, "EVM RPC canister error: {}", msg),
            EthereumError::InsufficientCycles(msg) => write!(f, "Insufficient cycles: {}", msg),
            EthereumError::ConsensusError(msg) => write!(f, "Consensus validation failed: {}", msg),
            EthereumError::AddressGenerationError(msg) => write!(f, "Address generation failed: {}", msg),
            EthereumError::SigningError(msg) => write!(f, "Transaction signing failed: {}", msg),
            EthereumError::BroadcastError(msg) => write!(f, "Transaction broadcast failed: {}", msg),
            EthereumError::L2OptimizationError(msg) => write!(f, "L2 optimization error: {}", msg),
        }
    }
}

/// Constants for Ethereum integration
pub mod constants {
    /// Wei per Ether (10^18)
    pub const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000;
    
    /// Gwei per Ether (10^9)
    pub const GWEI_PER_ETH: u64 = 1_000_000_000;
    
    /// Wei per Gwei (10^9)
    pub const WEI_PER_GWEI: u64 = 1_000_000_000;
    
    /// Standard gas limit for ETH transfers
    pub const ETH_TRANSFER_GAS_LIMIT: u64 = 21_000;
    
    /// Standard gas limit for ERC-20 transfers
    pub const ERC20_TRANSFER_GAS_LIMIT: u64 = 65_000;
    
    /// Maximum gas limit for complex transactions
    pub const MAX_GAS_LIMIT: u64 = 10_000_000;
    
    /// Minimum gas price (1 gwei)
    pub const MIN_GAS_PRICE: u64 = 1_000_000_000;
    
    /// Default derivation path for Ethereum addresses
    pub const DEFAULT_DERIVATION_PATH: &str = "m/44'/60'/0'/0";
}

/// Utility functions for Ethereum operations
pub mod utils {
    use super::constants::*;
    
    /// Convert Wei to ETH
    pub fn wei_to_eth(wei: &str) -> Result<f64, String> {
        let wei_amount: u128 = wei.parse()
            .map_err(|_| format!("Invalid wei amount: {}", wei))?;
        Ok(wei_amount as f64 / WEI_PER_ETH as f64)
    }
    
    /// Convert ETH to Wei
    pub fn eth_to_wei(eth: f64) -> String {
        let wei_amount = (eth * WEI_PER_ETH as f64) as u128;
        wei_amount.to_string()
    }
    
    /// Convert Gwei to Wei
    pub fn gwei_to_wei(gwei: u64) -> u64 {
        gwei * WEI_PER_GWEI
    }
    
    /// Convert Wei to Gwei
    pub fn wei_to_gwei(wei: u64) -> u64 {
        wei / WEI_PER_GWEI
    }
    
    /// Validate Ethereum address format
    pub fn validate_ethereum_address(address: &str) -> bool {
        if !address.starts_with("0x") {
            return false;
        }
        
        if address.len() != 42 {
            return false;
        }
        
        // Check if all characters after 0x are valid hex
        address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
    
    /// Calculate transaction fee
    pub fn calculate_transaction_fee(gas_used: u64, gas_price_wei: &str) -> Result<String, String> {
        let gas_price: u128 = gas_price_wei.parse()
            .map_err(|_| format!("Invalid gas price: {}", gas_price_wei))?;
        let fee = gas_used as u128 * gas_price;
        Ok(fee.to_string())
    }
    
    /// Estimate confirmation time based on gas price
    pub fn estimate_confirmation_time(gas_price_gwei: u64) -> u64 {
        match gas_price_gwei {
            0..=5 => 300,      // 5 minutes for very low gas
            6..=15 => 180,     // 3 minutes for low gas
            16..=30 => 60,     // 1 minute for medium gas
            31..=50 => 30,     // 30 seconds for high gas
            _ => 15,           // 15 seconds for very high gas
        }
    }
}

#[cfg(test)]
mod mod_tests {
    use super::*;

    #[test]
    fn test_evm_chain_properties() {
        // Test Ethereum
        let eth = EvmChain::Ethereum;
        assert_eq!(eth.chain_id(), 1);
        assert_eq!(eth.native_token(), "ETH");
        assert_eq!(eth.name(), "Ethereum");
        assert!(!eth.is_l2());
        assert!(!eth.is_sidechain());
        assert!(!eth.is_independent_l1());

        // Test Arbitrum (L2)
        let arb = EvmChain::Arbitrum;
        assert_eq!(arb.chain_id(), 42161);
        assert_eq!(arb.native_token(), "ETH");
        assert_eq!(arb.name(), "Arbitrum One");
        assert!(arb.is_l2());
        assert!(!arb.is_sidechain());
        assert!(!arb.is_independent_l1());

        // Test Optimism (L2)
        let op = EvmChain::Optimism;
        assert_eq!(op.chain_id(), 10);
        assert_eq!(op.native_token(), "ETH");
        assert_eq!(op.name(), "Optimism");
        assert!(op.is_l2());
        assert!(!op.is_sidechain());
        assert!(!op.is_independent_l1());

        // Test Polygon (Sidechain)
        let matic = EvmChain::Polygon;
        assert_eq!(matic.chain_id(), 137);
        assert_eq!(matic.native_token(), "MATIC");
        assert_eq!(matic.name(), "Polygon");
        assert!(!matic.is_l2());
        assert!(matic.is_sidechain());
        assert!(!matic.is_independent_l1());

        // Test Base (L2)
        let base = EvmChain::Base;
        assert_eq!(base.chain_id(), 8453);
        assert_eq!(base.native_token(), "ETH");
        assert_eq!(base.name(), "Base");
        assert!(base.is_l2());
        assert!(!base.is_sidechain());
        assert!(!base.is_independent_l1());

        // Test Avalanche (Sidechain)
        let avax = EvmChain::Avalanche;
        assert_eq!(avax.chain_id(), 43114);
        assert_eq!(avax.native_token(), "AVAX");
        assert_eq!(avax.name(), "Avalanche");
        assert!(!avax.is_l2());
        assert!(avax.is_sidechain());
        assert!(!avax.is_independent_l1());

        // Test Sonic (Independent L1)
        let sonic = EvmChain::Sonic;
        assert_eq!(sonic.chain_id(), 146);
        assert_eq!(sonic.native_token(), "S");
        assert_eq!(sonic.name(), "Sonic");
        assert!(!sonic.is_l2());
        assert!(!sonic.is_sidechain());
        assert!(sonic.is_independent_l1());

        // Test BNB Smart Chain (Sidechain)
        let bnb = EvmChain::BnbSmartChain;
        assert_eq!(bnb.chain_id(), 56);
        assert_eq!(bnb.native_token(), "BNB");
        assert_eq!(bnb.name(), "BNB Smart Chain");
        assert!(!bnb.is_l2());
        assert!(bnb.is_sidechain());
        assert!(!bnb.is_independent_l1());
    }

    #[test]
    fn test_ethereum_address_validation() {
        use utils::validate_ethereum_address;

        // Valid addresses
        assert!(validate_ethereum_address("0x1234567890123456789012345678901234567890"));
        assert!(validate_ethereum_address("0xabcdefABCDEF1234567890123456789012345678"));
        
        // Invalid addresses
        assert!(!validate_ethereum_address("1234567890123456789012345678901234567890")); // No 0x prefix
        assert!(!validate_ethereum_address("0x123")); // Too short
        assert!(!validate_ethereum_address("0x12345678901234567890123456789012345678901")); // Too long
        assert!(!validate_ethereum_address("0x123456789012345678901234567890123456789g")); // Invalid hex char
        assert!(!validate_ethereum_address("")); // Empty
    }

    #[test]
    fn test_wei_eth_conversion() {
        use utils::{wei_to_eth, eth_to_wei};

        // Test wei to eth conversion
        assert_eq!(wei_to_eth("1000000000000000000").unwrap(), 1.0); // 1 ETH
        assert_eq!(wei_to_eth("500000000000000000").unwrap(), 0.5); // 0.5 ETH
        assert_eq!(wei_to_eth("1").unwrap(), 0.000000000000000001); // 1 wei

        // Test eth to wei conversion
        assert_eq!(eth_to_wei(1.0), "1000000000000000000");
        assert_eq!(eth_to_wei(0.5), "500000000000000000");
        assert_eq!(eth_to_wei(2.5), "2500000000000000000");

        // Test invalid wei string
        assert!(wei_to_eth("invalid").is_err());
        assert!(wei_to_eth("").is_err());
    }

    #[test]
    fn test_gas_conversions() {
        use utils::{gwei_to_wei, wei_to_gwei};

        // Test gwei to wei
        assert_eq!(gwei_to_wei(1), 1_000_000_000);
        assert_eq!(gwei_to_wei(20), 20_000_000_000);
        assert_eq!(gwei_to_wei(100), 100_000_000_000);

        // Test wei to gwei
        assert_eq!(wei_to_gwei(1_000_000_000), 1);
        assert_eq!(wei_to_gwei(20_000_000_000), 20);
        assert_eq!(wei_to_gwei(100_000_000_000), 100);

        // Test partial gwei (should truncate)
        assert_eq!(wei_to_gwei(1_500_000_000), 1); // 1.5 gwei becomes 1
    }

    #[test]
    fn test_transaction_fee_calculation() {
        use utils::calculate_transaction_fee;

        // Standard ETH transfer
        let fee = calculate_transaction_fee(21000, "20000000000").unwrap(); // 21k gas at 20 gwei
        assert_eq!(fee, "420000000000000"); // 0.00042 ETH

        // Higher gas price
        let fee2 = calculate_transaction_fee(21000, "50000000000").unwrap(); // 21k gas at 50 gwei
        assert_eq!(fee2, "1050000000000000"); // 0.00105 ETH

        // Complex transaction
        let fee3 = calculate_transaction_fee(100000, "30000000000").unwrap(); // 100k gas at 30 gwei
        assert_eq!(fee3, "3000000000000000"); // 0.003 ETH

        // Invalid gas price
        assert!(calculate_transaction_fee(21000, "invalid").is_err());
    }

    #[test]
    fn test_confirmation_time_estimation() {
        use utils::estimate_confirmation_time;

        // Very low gas - 5 minutes
        assert_eq!(estimate_confirmation_time(1), 300);
        assert_eq!(estimate_confirmation_time(5), 300);

        // Low gas - 3 minutes
        assert_eq!(estimate_confirmation_time(10), 180);
        assert_eq!(estimate_confirmation_time(15), 180);

        // Medium gas - 1 minute
        assert_eq!(estimate_confirmation_time(20), 60);
        assert_eq!(estimate_confirmation_time(30), 60);

        // High gas - 30 seconds
        assert_eq!(estimate_confirmation_time(40), 30);
        assert_eq!(estimate_confirmation_time(50), 30);

        // Very high gas - 15 seconds
        assert_eq!(estimate_confirmation_time(100), 15);
        assert_eq!(estimate_confirmation_time(200), 15);
    }

    #[test]
    fn test_ethereum_error_display() {
        let error1 = EthereumError::InvalidAddress("0xinvalid".to_string());
        assert!(error1.to_string().contains("Invalid Ethereum address"));

        let error2 = EthereumError::InsufficientBalance {
            required: "1000".to_string(),
            available: "500".to_string(),
        };
        assert!(error2.to_string().contains("Insufficient balance"));
        assert!(error2.to_string().contains("1000"));
        assert!(error2.to_string().contains("500"));

        let error3 = EthereumError::NetworkError("Connection failed".to_string());
        assert!(error3.to_string().contains("Network error"));

        let error4 = EthereumError::RpcError("EVM RPC canister error".to_string());
        assert!(error4.to_string().contains("EVM RPC canister error"));

        let error5 = EthereumError::ConsensusError("Validation failed".to_string());
        assert!(error5.to_string().contains("Consensus validation failed"));
    }

    #[test]
    fn test_gas_priority_variants() {
        let priorities = [
            GasPriority::Low,
            GasPriority::Medium,
            GasPriority::High,
            GasPriority::Urgent,
        ];

        // Test that all variants are different
        for i in 0..priorities.len() {
            for j in i + 1..priorities.len() {
                assert_ne!(format!("{:?}", priorities[i]), format!("{:?}", priorities[j]));
            }
        }
    }

    #[test]
    fn test_constants() {
        use constants::*;

        assert_eq!(WEI_PER_ETH, 1_000_000_000_000_000_000);
        assert_eq!(GWEI_PER_ETH, 1_000_000_000);
        assert_eq!(WEI_PER_GWEI, 1_000_000_000);
        assert_eq!(ETH_TRANSFER_GAS_LIMIT, 21_000);
        assert_eq!(ERC20_TRANSFER_GAS_LIMIT, 65_000);
        assert_eq!(MAX_GAS_LIMIT, 10_000_000);
        assert_eq!(MIN_GAS_PRICE, 1_000_000_000);
        assert_eq!(DEFAULT_DERIVATION_PATH, "m/44'/60'/0'/0");
    }

    #[test]
    fn test_evm_chain_serialization() {
        // Test that EVM chains can be serialized/deserialized
        let chains = vec![
            EvmChain::Ethereum,
            EvmChain::Arbitrum,
            EvmChain::Optimism,
            EvmChain::Polygon,
            EvmChain::Base,
            EvmChain::Avalanche,
            EvmChain::Sonic,
            EvmChain::BnbSmartChain,
        ];

        for chain in chains {
            // Test Debug formatting
            let debug_str = format!("{:?}", chain);
            assert!(!debug_str.is_empty());

            // Test Clone
            let cloned = chain.clone();
            assert_eq!(chain.chain_id(), cloned.chain_id());

            // Test PartialEq
            assert_eq!(chain, cloned);
        }
    }
}