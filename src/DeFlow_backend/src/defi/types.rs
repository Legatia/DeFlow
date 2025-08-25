// DeFlow DeFi Types - Core type definitions for multi-chain DeFi

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;

// Chain identification
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChainId {
    Bitcoin,
    Ethereum,
    Arbitrum,
    Optimism,
    Polygon,
    Avalanche,
    Base,
    Solana,
    Custom(String),
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainId::Bitcoin => write!(f, "Bitcoin"),
            ChainId::Ethereum => write!(f, "Ethereum"),
            ChainId::Arbitrum => write!(f, "Arbitrum"),
            ChainId::Optimism => write!(f, "Optimism"),
            ChainId::Polygon => write!(f, "Polygon"),
            ChainId::Avalanche => write!(f, "Avalanche"),
            ChainId::Base => write!(f, "Base"),
            ChainId::Solana => write!(f, "Solana"),
            ChainId::Custom(name) => write!(f, "{}", name),
        }
    }
}

// Bitcoin network types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum BitcoinNetwork {
    Mainnet,
    Testnet,
    Regtest,
}

// Bitcoin address types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum BitcoinAddressType {
    P2PKH,   // Legacy (1...)
    P2SH,    // Script Hash (3...)
    P2WPKH,  // SegWit (bc1q...)
    P2TR,    // Taproot (bc1p...)
}

// Solana network types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum SolanaNetwork {
    Mainnet,
    Devnet,
    Testnet,
}

// L2 types for Ethereum
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum L2Type {
    Optimistic,  // Optimism, Arbitrum
    ZKRollup,    // Polygon zkEVM
    Sidechain,   // Polygon PoS
    StateChannel,
}

// Asset representation across chains
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Asset {
    pub symbol: String,
    pub name: String,
    pub chain: ChainId,
    pub contract_address: Option<String>, // None for native assets
    pub decimals: u8,
    pub is_native: bool,
}

// Position in a DeFi protocol
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Position {
    pub id: String,
    pub chain: ChainId,
    pub asset: Asset,
    pub amount: u64, // In smallest unit (satoshis, wei, lamports)
    pub value_usd: f64,
    pub percentage: f64, // % of total portfolio
    pub protocol: Option<String>, // Uniswap, Aave, Jupiter, etc.
    pub yield_apy: Option<f64>,
    pub risk_score: u8, // 1-10 scale
    pub last_updated: u64,
}

// Cross-chain portfolio
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CrossChainPortfolio {
    pub user_id: Principal,
    pub total_value_usd: f64,
    pub positions: Vec<Position>,
    pub target_allocation: AllocationStrategy,
    pub rebalance_threshold: f64,
    pub auto_rebalance: bool,
    pub risk_profile: RiskProfile,
    pub last_rebalance: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

// Allocation strategies
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum AllocationStrategy {
    FixedPercentage(HashMap<ChainId, f64>),
    MarketCapWeighted,
    VolatilityWeighted,
    YieldOptimized { min_apy: f64 },
    RiskAdjusted { max_risk: u8 },
    Custom(String), // Custom strategy name
}

// Risk profile
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RiskProfile {
    pub risk_tolerance: u8, // 1-10 scale
    pub max_position_size_percentage: f64,
    pub max_chain_exposure_percentage: f64,
    pub min_stablecoin_percentage: f64,
    pub stop_loss_percentage: f64,
    pub emergency_exit_enabled: bool,
}

impl Default for RiskProfile {
    fn default() -> Self {
        Self {
            risk_tolerance: 5,
            max_position_size_percentage: 20.0,
            max_chain_exposure_percentage: 60.0,
            min_stablecoin_percentage: 10.0,
            stop_loss_percentage: 15.0,
            emergency_exit_enabled: true,
        }
    }
}

// Bitcoin-specific types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinAddress {
    pub address: String,
    pub address_type: BitcoinAddressType,
    pub derivation_path: String,
    pub balance_satoshis: u64,
    pub utxo_count: u32,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinUTXO {
    pub txid: String,
    pub vout: u32,
    pub value_satoshis: u64,
    pub script_pubkey: String,
    pub confirmations: u32,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinPortfolio {
    pub addresses: Vec<BitcoinAddress>,
    pub total_btc: f64,
    pub total_satoshis: u64,
    pub total_value_usd: f64,
    pub utxos: Vec<BitcoinUTXO>,
    pub last_updated: u64,
}

// Transaction types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DeFiTransaction {
    pub id: String,
    pub chain: ChainId,
    pub transaction_type: TransactionType,
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub asset: Asset,
    pub gas_fee: Option<u64>,
    pub status: TransactionStatus,
    pub created_at: u64,
    pub confirmed_at: Option<u64>,
    pub block_height: Option<u64>,
    pub transaction_hash: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum TransactionType {
    Send,
    Receive,
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Stake,
    Unstake,
    Lend,
    Borrow,
    Repay,
    Claim,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

// DeFi execution result
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DeFiExecutionResult {
    pub success: bool,
    pub transaction_hash: Option<String>,
    pub gas_used: Option<u64>,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub chain: ChainId,
}

// Price information
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PriceInfo {
    pub asset: Asset,
    pub price_usd: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub market_cap: Option<f64>,
    pub last_updated: u64,
    pub source: String, // Chainlink, Pyth, etc.
}

// Gas/Fee information
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct GasInfo {
    pub chain: ChainId,
    pub gas_price: u64,
    pub priority_fee: Option<u64>,
    pub estimated_cost_usd: f64,
    pub confirmation_time_seconds: u32,
    pub last_updated: u64,
}

// Helper functions
#[allow(dead_code)]
impl Asset {
    pub fn bitcoin() -> Self {
        Self {
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            chain: ChainId::Bitcoin,
            contract_address: None,
            decimals: 8,
            is_native: true,
        }
    }
    
    pub fn ethereum() -> Self {
        Self {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            chain: ChainId::Ethereum,
            contract_address: None,
            decimals: 18,
            is_native: true,
        }
    }
    
    pub fn solana() -> Self {
        Self {
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            chain: ChainId::Solana,
            contract_address: None,
            decimals: 9,
            is_native: true,
        }
    }
}

impl ChainId {
    pub fn evm_chain_id(&self) -> Option<u64> {
        match self {
            ChainId::Ethereum => Some(1),
            ChainId::Arbitrum => Some(42161),
            ChainId::Optimism => Some(10),
            ChainId::Polygon => Some(137),
            ChainId::Avalanche => Some(43114),
            ChainId::Base => Some(8453),
            _ => None,
        }
    }
    
    pub fn is_evm_chain(&self) -> bool {
        self.evm_chain_id().is_some()
    }
    
    pub fn is_l2(&self) -> bool {
        matches!(self, ChainId::Arbitrum | ChainId::Optimism | ChainId::Polygon | ChainId::Base)
    }
}