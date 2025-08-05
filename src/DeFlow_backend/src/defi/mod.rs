// DeFlow DeFi Module - Multi-Chain DeFi Integration
// Day 8: Bitcoin Integration Foundation
// Day 9: Ethereum & L2 Integration

pub mod bitcoin;
pub mod ethereum;
pub mod solana;
pub mod types;
pub mod api;
// Day 11: Advanced DeFi Workflows - Cross-chain yield farming and arbitrage
pub mod yield_farming;
pub mod yield_engine;
pub mod cross_chain_optimizer;
pub mod yield_api;
pub mod arbitrage;
// Day 11: Advanced Portfolio Management System
pub mod portfolio_manager;
pub mod portfolio_api;

use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

// Re-export core DeFi types
pub use types::*;
// API functions are exposed as canister endpoints in the api module

// Core DeFi chain context
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DeFiChainContext {
    pub bitcoin: BitcoinDeFiContext,
    pub ethereum_chains: HashMap<ChainId, EVMDeFiContext>,
    pub solana: Option<SolanaDeFiContext>,
    pub active_chains: Vec<ChainId>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinDeFiContext {
    pub network: BitcoinNetwork,
    pub address_types: Vec<BitcoinAddressType>,
    pub ordinals_support: bool,
    pub runes_support: bool,
    pub brc20_support: bool,
    pub key_name: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EVMDeFiContext {
    pub chain_id: ChainId,
    pub chain_name: String,
    pub rpc_endpoints: Vec<String>,
    pub gas_optimization: bool,
    pub l2_type: Option<L2Type>,
    pub native_token: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SolanaDeFiContext {
    pub network: SolanaNetwork,
    pub spl_tokens: Vec<String>,
    pub jupiter_integration: bool,
    pub key_name: String,
}

impl Default for DeFiChainContext {
    fn default() -> Self {
        Self {
            bitcoin: BitcoinDeFiContext::default(),
            ethereum_chains: HashMap::new(),
            solana: None,
            active_chains: vec![ChainId::Bitcoin],
        }
    }
}

impl Default for BitcoinDeFiContext {
    fn default() -> Self {
        Self {
            network: BitcoinNetwork::Regtest, // Start with regtest for development
            address_types: vec![
                BitcoinAddressType::P2PKH,
                BitcoinAddressType::P2WPKH,
                BitcoinAddressType::P2TR,
            ],
            ordinals_support: true,
            runes_support: true,
            brc20_support: true,
            key_name: "deflow_bitcoin_key".to_string(),
        }
    }
}

// DeFi Chain Manager - Central coordinator for all chains
pub struct DeFiChainManager {
    context: DeFiChainContext,
    bitcoin_service: Option<bitcoin::BitcoinDeFiService>,
    ethereum_service: Option<ethereum::minimal_icp::MinimalIcpEthereumService>,
    solana_service: Option<solana::icp_solana::IcpSolanaService>,
}

impl DeFiChainManager {
    pub fn new() -> Self {
        Self {
            context: DeFiChainContext::default(),
            bitcoin_service: None,
            ethereum_service: None,
            solana_service: None,
        }
    }
    
    #[allow(dead_code)]
    pub async fn initialize_bitcoin(&mut self) -> Result<(), String> {
        let bitcoin_service = bitcoin::BitcoinDeFiService::new(
            self.context.bitcoin.network.clone(),
            self.context.bitcoin.key_name.clone()
        ).await?;
        
        self.bitcoin_service = Some(bitcoin_service);
        ic_cdk::println!("Bitcoin DeFi service initialized successfully");
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn initialize_ethereum(&mut self) -> Result<(), String> {
        let ethereum_service = ethereum::minimal_icp::MinimalIcpEthereumService::new(
            "deflow_ethereum_key".to_string(),
            ic_cdk::api::id(),
        );
        
        self.ethereum_service = Some(ethereum_service);
        ic_cdk::println!("ICP-compliant Ethereum DeFi service initialized successfully");
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn initialize_solana(&mut self) -> Result<(), String> {
        let solana_service = solana::icp_solana::IcpSolanaService::new(
            "deflow_solana_key".to_string(),
            ic_cdk::api::id(),
            solana::SolanaNetwork::Devnet, // Start with Devnet for development
        );
        
        self.solana_service = Some(solana_service);
        ic_cdk::println!("ICP-compliant Solana DeFi service initialized successfully");
        Ok(())
    }
    
    pub fn get_active_chains(&self) -> Vec<ChainId> {
        self.context.active_chains.clone()
    }
    
    pub fn is_chain_active(&self, chain_id: &ChainId) -> bool {
        self.context.active_chains.contains(chain_id)
    }
}

// Global DeFi chain manager instance
use std::cell::RefCell;
thread_local! {
    static DEFI_CHAIN_MANAGER: RefCell<DeFiChainManager> = RefCell::new(DeFiChainManager::new());
}

pub fn with_defi_manager<R>(f: impl FnOnce(&DeFiChainManager) -> R) -> R {
    DEFI_CHAIN_MANAGER.with(|manager| f(&manager.borrow()))
}

pub fn with_defi_manager_mut<R>(f: impl FnOnce(&mut DeFiChainManager) -> R) -> R {
    DEFI_CHAIN_MANAGER.with(|manager| f(&mut manager.borrow_mut()))
}

// Initialize DeFi system
pub async fn initialize_defi_system() -> Result<(), String> {
    ic_cdk::println!("Initializing DeFlow DeFi system...");
    
    // Bitcoin service initialization will be handled on-demand in API calls
    // This avoids complex async lifetime issues during initialization
    
    ic_cdk::println!("DeFi system components ready");
    Ok(())
}