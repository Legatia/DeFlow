// Cross-Chain Bridge Integrations for DeFlow
// Implements Wormhole, LayerZero, and Stargate for optimal yield farming

pub mod wormhole;
pub mod layerzero;
pub mod stargate;
pub mod router;
pub mod monitoring;

use candid::{CandidType, Deserialize};
use serde::Serialize;
use super::yield_farming::ChainId;

/// Supported bridge protocols
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BridgeProtocol {
    Wormhole,
    LayerZero,
    Stargate,
    IcpChainFusion,  // Native ICP integration
    PoolBased,       // DeFlow internal pool
}

/// Bridge transaction status
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum BridgeStatus {
    Pending,
    SourceConfirmed,
    Bridging,
    DestinationConfirmed,
    Completed,
    Failed(String),
}

/// Bridge transaction details
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub bridge_id: String,
    pub protocol: BridgeProtocol,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub asset: String,
    pub amount: u64,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub status: BridgeStatus,
    pub fee_paid: f64,
    pub estimated_arrival: u64,
    pub initiated_at: u64,
    pub completed_at: Option<u64>,
}

/// Bridge route option with costs
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BridgeRoute {
    pub protocol: BridgeProtocol,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub asset: String,
    pub estimated_fee: f64,
    pub estimated_time_seconds: u64,
    pub min_amount: u64,
    pub max_amount: u64,
    pub security_score: f64,      // 0.0-1.0
    pub success_rate_24h: f64,     // 0.0-1.0
    pub available: bool,
}

/// Bridge health metrics
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BridgeHealth {
    pub protocol: BridgeProtocol,
    pub is_operational: bool,
    pub liquidity_available: f64,
    pub avg_completion_time: u64,
    pub success_rate_7d: f64,
    pub last_failure: Option<u64>,
    pub maintenance_mode: bool,
}

/// Common bridge error types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum BridgeError {
    InsufficientLiquidity(String),
    UnsupportedRoute(ChainId, ChainId),
    UnsupportedAsset(String),
    AmountTooSmall(u64, u64),       // actual, minimum
    AmountTooLarge(u64, u64),       // actual, maximum
    BridgeOffline(BridgeProtocol),
    TransactionFailed(String),
    TimeoutExceeded(u64),
    InvalidAddress(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::InsufficientLiquidity(msg) => write!(f, "Insufficient liquidity: {}", msg),
            BridgeError::UnsupportedRoute(from, to) => {
                write!(f, "Unsupported route: {:?} -> {:?}", from, to)
            }
            BridgeError::UnsupportedAsset(asset) => write!(f, "Unsupported asset: {}", asset),
            BridgeError::AmountTooSmall(actual, min) => {
                write!(f, "Amount too small: {} (min: {})", actual, min)
            }
            BridgeError::AmountTooLarge(actual, max) => {
                write!(f, "Amount too large: {} (max: {})", actual, max)
            }
            BridgeError::BridgeOffline(protocol) => write!(f, "Bridge offline: {:?}", protocol),
            BridgeError::TransactionFailed(reason) => write!(f, "Transaction failed: {}", reason),
            BridgeError::TimeoutExceeded(elapsed) => {
                write!(f, "Timeout exceeded: {} seconds", elapsed)
            }
            BridgeError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
        }
    }
}

impl From<String> for BridgeError {
    fn from(error: String) -> Self {
        BridgeError::TransactionFailed(error)
    }
}
