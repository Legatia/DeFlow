// Chain Configuration for Multi-EVM Support
// Manages RPC endpoints, block explorers, and chain-specific parameters

use super::EvmChain;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a specific EVM chain
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain: EvmChain,
    pub chain_id: u64,
    pub name: String,
    pub native_token: String,
    pub rpc_endpoints: Vec<String>,
    pub block_explorer: String,
    pub is_testnet: bool,
    pub average_block_time_seconds: u64,
    pub finality_blocks: u64,
    pub min_gas_price: u64, // in wei
    pub max_gas_price: u64, // in wei
    pub typical_gas_price: u64, // in wei
    pub supports_eip1559: bool,
    pub bridge_contracts: HashMap<String, String>, // bridge_name -> contract_address
}

/// Chain configuration manager
#[derive(Debug, Clone)]
pub struct ChainConfigManager {
    configs: HashMap<EvmChain, ChainConfig>,
}

impl ChainConfigManager {
    /// Create a new chain configuration manager with default configurations
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        
        // Ethereum Mainnet
        configs.insert(EvmChain::Ethereum, ChainConfig {
            chain: EvmChain::Ethereum,
            chain_id: 1,
            name: "Ethereum".to_string(),
            native_token: "ETH".to_string(),
            rpc_endpoints: vec![
                "https://eth.llamarpc.com".to_string(),
                "https://rpc.ankr.com/eth".to_string(),
                "https://ethereum.publicnode.com".to_string(),
            ],
            block_explorer: "https://etherscan.io".to_string(),
            is_testnet: false,
            average_block_time_seconds: 12,
            finality_blocks: 32,
            min_gas_price: 1_000_000_000, // 1 gwei
            max_gas_price: 500_000_000_000, // 500 gwei
            typical_gas_price: 20_000_000_000, // 20 gwei
            supports_eip1559: true,
            bridge_contracts: HashMap::new(),
        });
        
        // Arbitrum One
        configs.insert(EvmChain::Arbitrum, ChainConfig {
            chain: EvmChain::Arbitrum,
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            native_token: "ETH".to_string(),
            rpc_endpoints: vec![
                "https://arb1.arbitrum.io/rpc".to_string(),
                "https://rpc.ankr.com/arbitrum".to_string(),
                "https://arbitrum.publicnode.com".to_string(),
            ],
            block_explorer: "https://arbiscan.io".to_string(),
            is_testnet: false,
            average_block_time_seconds: 1, // Very fast blocks
            finality_blocks: 1,
            min_gas_price: 100_000_000, // 0.1 gwei
            max_gas_price: 10_000_000_000, // 10 gwei
            typical_gas_price: 1_000_000_000, // 1 gwei
            supports_eip1559: true,
            bridge_contracts: {
                let mut bridges = HashMap::new();
                bridges.insert("official".to_string(), "0x8315177aB297bA92A06054cE80a67Ed4DBd7ed3a".to_string());
                bridges
            },
        });
        
        // Optimism
        configs.insert(EvmChain::Optimism, ChainConfig {
            chain: EvmChain::Optimism,
            chain_id: 10,
            name: "Optimism".to_string(),
            native_token: "ETH".to_string(),
            rpc_endpoints: vec![
                "https://mainnet.optimism.io".to_string(),
                "https://rpc.ankr.com/optimism".to_string(),
                "https://optimism.publicnode.com".to_string(),
            ],
            block_explorer: "https://optimistic.etherscan.io".to_string(),
            is_testnet: false,
            average_block_time_seconds: 2,
            finality_blocks: 1,
            min_gas_price: 1_000_000, // 0.001 gwei
            max_gas_price: 5_000_000_000, // 5 gwei
            typical_gas_price: 500_000_000, // 0.5 gwei
            supports_eip1559: true,
            bridge_contracts: {
                let mut bridges = HashMap::new();
                bridges.insert("official".to_string(), "0x99C9fc46f92E8a1c0deC1b1747d010903E884bE1".to_string());
                bridges
            },
        });
        
        // Polygon
        configs.insert(EvmChain::Polygon, ChainConfig {
            chain: EvmChain::Polygon,
            chain_id: 137,
            name: "Polygon".to_string(),
            native_token: "MATIC".to_string(),
            rpc_endpoints: vec![
                "https://polygon-rpc.com".to_string(),
                "https://rpc.ankr.com/polygon".to_string(),
                "https://polygon.publicnode.com".to_string(),
            ],
            block_explorer: "https://polygonscan.com".to_string(),
            is_testnet: false,
            average_block_time_seconds: 2,
            finality_blocks: 128,
            min_gas_price: 30_000_000_000, // 30 gwei (MATIC)
            max_gas_price: 500_000_000_000, // 500 gwei (MATIC)
            typical_gas_price: 50_000_000_000, // 50 gwei (MATIC)
            supports_eip1559: true,
            bridge_contracts: {
                let mut bridges = HashMap::new();
                bridges.insert("pos".to_string(), "0x40ec5B33f54e0E8A33A975908C5BA1c14e5BbbDf".to_string());
                bridges
            },
        });
        
        // Base
        configs.insert(EvmChain::Base, ChainConfig {
            chain: EvmChain::Base,
            chain_id: 8453,
            name: "Base".to_string(),
            native_token: "ETH".to_string(),
            rpc_endpoints: vec![
                "https://mainnet.base.org".to_string(),
                "https://rpc.ankr.com/base".to_string(),
                "https://base.publicnode.com".to_string(),
            ],
            block_explorer: "https://basescan.org".to_string(),
            is_testnet: false,
            average_block_time_seconds: 2,
            finality_blocks: 1,
            min_gas_price: 100_000_000, // 0.1 gwei
            max_gas_price: 10_000_000_000, // 10 gwei
            typical_gas_price: 1_000_000_000, // 1 gwei
            supports_eip1559: true,
            bridge_contracts: {
                let mut bridges = HashMap::new();
                bridges.insert("official".to_string(), "0x3154Cf16ccdb4C6d922629664174b904d80F2C35".to_string());
                bridges
            },
        });
        
        // Avalanche C-Chain
        configs.insert(EvmChain::Avalanche, ChainConfig {
            chain: EvmChain::Avalanche,
            chain_id: 43114,
            name: "Avalanche C-Chain".to_string(),
            native_token: "AVAX".to_string(),
            rpc_endpoints: vec![
                "https://api.avax.network/ext/bc/C/rpc".to_string(),
                "https://rpc.ankr.com/avalanche".to_string(),
                "https://avalanche.publicnode.com/ext/bc/C/rpc".to_string(),
            ],
            block_explorer: "https://snowtrace.io".to_string(),
            is_testnet: false,
            average_block_time_seconds: 2,
            finality_blocks: 1,
            min_gas_price: 25_000_000_000, // 25 gwei (AVAX)
            max_gas_price: 1_000_000_000_000, // 1000 gwei (AVAX)
            typical_gas_price: 50_000_000_000, // 50 gwei (AVAX)
            supports_eip1559: true,
            bridge_contracts: {
                let mut bridges = HashMap::new();
                bridges.insert("avalanche_bridge".to_string(), "0x8EB8a3b98659Cce290402893d0123abb75E3ab28".to_string());
                bridges
            },
        });
        
        Self { configs }
    }
    
    /// Get configuration for a specific chain
    pub fn get_config(&self, chain: &EvmChain) -> Option<&ChainConfig> {
        self.configs.get(chain)
    }
    
    /// Get all supported chains
    pub fn get_supported_chains(&self) -> Vec<EvmChain> {
        self.configs.keys().cloned().collect()
    }
    
    /// Get primary RPC endpoint for a chain
    pub fn get_primary_rpc(&self, chain: &EvmChain) -> Option<String> {
        self.configs.get(chain)
            .and_then(|config| config.rpc_endpoints.first())
            .cloned()
    }
    
    /// Get all RPC endpoints for a chain (for failover)
    pub fn get_rpc_endpoints(&self, chain: &EvmChain) -> Vec<String> {
        self.configs.get(chain)
            .map(|config| config.rpc_endpoints.clone())
            .unwrap_or_default()
    }
    
    /// Get block explorer URL for a transaction
    pub fn get_tx_explorer_url(&self, chain: &EvmChain, tx_hash: &str) -> Option<String> {
        self.configs.get(chain)
            .map(|config| format!("{}/tx/{}", config.block_explorer, tx_hash))
    }
    
    /// Get block explorer URL for an address
    pub fn get_address_explorer_url(&self, chain: &EvmChain, address: &str) -> Option<String> {
        self.configs.get(chain)
            .map(|config| format!("{}/address/{}", config.block_explorer, address))
    }
    
    /// Check if chain supports EIP-1559
    pub fn supports_eip1559(&self, chain: &EvmChain) -> bool {
        self.configs.get(chain)
            .map(|config| config.supports_eip1559)
            .unwrap_or(false)
    }
    
    /// Get typical gas price for a chain
    pub fn get_typical_gas_price(&self, chain: &EvmChain) -> Option<u64> {
        self.configs.get(chain)
            .map(|config| config.typical_gas_price)
    }
    
    /// Get gas price bounds for a chain
    pub fn get_gas_price_bounds(&self, chain: &EvmChain) -> Option<(u64, u64)> {
        self.configs.get(chain)
            .map(|config| (config.min_gas_price, config.max_gas_price))
    }
    
    /// Calculate estimated confirmation time
    pub fn estimate_confirmation_time(&self, chain: &EvmChain, blocks: u64) -> Option<u64> {
        self.configs.get(chain)
            .map(|config| blocks * config.average_block_time_seconds)
    }
    
    /// Get finality requirements for a chain
    pub fn get_finality_blocks(&self, chain: &EvmChain) -> Option<u64> {
        self.configs.get(chain)
            .map(|config| config.finality_blocks)
    }
    
    /// Check if two chains are compatible for bridging
    pub fn are_chains_bridgeable(&self, from_chain: &EvmChain, to_chain: &EvmChain) -> bool {
        // All chains can bridge to/from Ethereum
        if *from_chain == EvmChain::Ethereum || *to_chain == EvmChain::Ethereum {
            return true;
        }
        
        // Some L2s have direct bridges
        match (from_chain, to_chain) {
            (EvmChain::Arbitrum, EvmChain::Optimism) => true,
            (EvmChain::Optimism, EvmChain::Arbitrum) => true,
            (EvmChain::Arbitrum, EvmChain::Base) => true,
            (EvmChain::Base, EvmChain::Arbitrum) => true,
            (EvmChain::Optimism, EvmChain::Base) => true,
            (EvmChain::Base, EvmChain::Optimism) => true,
            _ => false, // Require Ethereum as intermediate
        }
    }
    
    /// Get bridge contract address for a specific bridge
    pub fn get_bridge_contract(&self, chain: &EvmChain, bridge_name: &str) -> Option<String> {
        self.configs.get(chain)
            .and_then(|config| config.bridge_contracts.get(bridge_name))
            .cloned()
    }
    
    /// Update RPC endpoints for a chain (for load balancing)
    pub fn update_rpc_endpoints(&mut self, chain: EvmChain, endpoints: Vec<String>) {
        if let Some(config) = self.configs.get_mut(&chain) {
            config.rpc_endpoints = endpoints;
        }
    }
    
    /// Add a new chain configuration
    pub fn add_chain_config(&mut self, chain: EvmChain, config: ChainConfig) {
        self.configs.insert(chain, config);
    }
    
    /// Get chains sorted by typical gas costs (lowest first)
    pub fn get_chains_by_cost(&self) -> Vec<(EvmChain, u64)> {
        let mut chains: Vec<_> = self.configs.iter()
            .map(|(chain, config)| (*chain, config.typical_gas_price))
            .collect();
        
        chains.sort_by_key(|(_, gas_price)| *gas_price);
        chains
    }
    
    /// Get L2 chains only
    pub fn get_l2_chains(&self) -> Vec<EvmChain> {
        self.configs.keys()
            .cloned()
            .filter(|chain| chain.is_l2())
            .collect()
    }
    
    /// Get sidechain chains only
    pub fn get_sidechain_chains(&self) -> Vec<EvmChain> {
        self.configs.keys()
            .cloned()
            .filter(|chain| chain.is_sidechain())
            .collect()
    }
}

impl Default for ChainConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Testnet configurations for development
pub struct TestnetChainConfig;

impl TestnetChainConfig {
    /// Get testnet configuration manager
    pub fn new() -> ChainConfigManager {
        let mut configs = HashMap::new();
        
        // Ethereum Sepolia
        configs.insert(EvmChain::Ethereum, ChainConfig {
            chain: EvmChain::Ethereum,
            chain_id: 11155111,
            name: "Ethereum Sepolia".to_string(),
            native_token: "ETH".to_string(),
            rpc_endpoints: vec![
                "https://rpc.sepolia.org".to_string(),
                "https://rpc.ankr.com/eth_sepolia".to_string(),
            ],
            block_explorer: "https://sepolia.etherscan.io".to_string(),
            is_testnet: true,
            average_block_time_seconds: 12,
            finality_blocks: 32,
            min_gas_price: 1_000_000_000,
            max_gas_price: 100_000_000_000,
            typical_gas_price: 5_000_000_000,
            supports_eip1559: true,
            bridge_contracts: HashMap::new(),
        });
        
        // Add other testnet configs as needed...
        
        ChainConfigManager { configs }
    }
}