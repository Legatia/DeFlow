use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::Storable;
use serde::{Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

// =============================================================================
// CORE TYPES
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum ChainId {
    Bitcoin,
    Ethereum,
    Arbitrum,
    Optimism,
    Polygon,
    Base,
    Solana,
    Avalanche,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum Asset {
    BTC,
    ETH,
    USDC,
    USDT,
    DAI,
    SOL,
    MATIC,
    AVAX,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum PoolPhase {
    Bootstrapping {
        started_at: u64,
        target_liquidity: HashMap<Asset, u64>,
        estimated_completion: u64,
    },
    Active {
        activated_at: u64,
        min_reserve_ratio: f64,
        max_utilization: f64,
    },
    Emergency {
        paused_at: u64,
        reason: String,
    },
}

impl Default for PoolPhase {
    fn default() -> Self {
        PoolPhase::Bootstrapping {
            started_at: ic_cdk::api::time(),
            target_liquidity: HashMap::new(),
            estimated_completion: ic_cdk::api::time() + (365 * 24 * 60 * 60 * 1_000_000_000), // 1 year from now
        }
    }
}

// =============================================================================
// LIQUIDITY MANAGEMENT
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct LiquidityReserve {
    pub total_amount: u64,
    pub fee_contributed_amount: u64,
    pub last_updated: u64,
    pub daily_growth_rate: f64,
    pub utilization_rate: f64,
}

impl Default for LiquidityReserve {
    fn default() -> Self {
        LiquidityReserve {
            total_amount: 0,
            fee_contributed_amount: 0,
            last_updated: ic_cdk::api::time(),
            daily_growth_rate: 0.0,
            utilization_rate: 0.0,
        }
    }
}

// =============================================================================
// BUSINESS MODEL INTEGRATION
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum TeamRole {
    Owner,
    SeniorManager,
    OperationsManager,  
    TechManager,
    Developer,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum TeamChangeType {
    AddMember,
    RemoveMember,
    PromoteMember,
    DemoteMember,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TeamChangeRequest {
    pub request_type: TeamChangeType,
    pub requester: Principal,
    pub target_principal: Principal,
    pub new_role: TeamRole,
    pub requires_owner_approval: bool,
    pub timestamp: u64,
    pub approved: bool,
    pub request_id: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TeamHierarchy {
    // INTERNET IDENTITY INTEGRATION: Owner (project deployer)
    pub owner_principal: Principal,
    
    // Management hierarchy
    pub senior_managers: Vec<Principal>,
    pub operations_managers: Vec<Principal>, 
    pub tech_managers: Vec<Principal>,
    pub developers: Vec<Principal>,
    
    // Approval system
    pub pending_approvals: Vec<TeamChangeRequest>,
    pub next_request_id: u64,
    
    // SECURITY: Rate limiting
    pub last_team_change: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DevTeamBusinessModel {
    // TEAM HIERARCHY: Internet Identity based team management
    pub team_hierarchy: TeamHierarchy,
    
    // Real-time profit tracking
    pub monthly_subscription_revenue: f64,
    pub monthly_transaction_fees: f64,
    pub monthly_enterprise_revenue: f64,
    pub monthly_operating_costs: f64,
    
    // Team earnings (distributed by role and contribution)
    pub team_member_earnings: HashMap<Principal, f64>,
    pub total_distributed_profits: f64,
    
    // Business reserves
    pub emergency_fund: f64,
    pub reinvestment_fund: f64,
    
    // Distribution configuration
    pub minimum_distribution_threshold: f64,  // $5,000 minimum
    pub distribution_frequency: u64,          // Monthly (2,629,800 seconds)
    pub last_distribution_time: u64,
}

impl Default for TeamHierarchy {
    fn default() -> Self {
        TeamHierarchy {
            owner_principal: Principal::anonymous(),
            senior_managers: Vec::new(),
            operations_managers: Vec::new(), 
            tech_managers: Vec::new(),
            developers: Vec::new(),
            pending_approvals: Vec::new(),
            next_request_id: 1,
            last_team_change: ic_cdk::api::time(),
        }
    }
}

impl Default for DevTeamBusinessModel {
    fn default() -> Self {
        DevTeamBusinessModel {
            team_hierarchy: TeamHierarchy::default(),
            monthly_subscription_revenue: 0.0,
            monthly_transaction_fees: 0.0,
            monthly_enterprise_revenue: 0.0,
            monthly_operating_costs: 0.0,
            team_member_earnings: HashMap::new(),
            total_distributed_profits: 0.0,
            emergency_fund: 0.0,
            reinvestment_fund: 0.0,
            minimum_distribution_threshold: 5000.0,
            distribution_frequency: 2_629_800, // 30 days
            last_distribution_time: ic_cdk::api::time(),
        }
    }
}

// =============================================================================
// POOL STATE
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PoolState {
    // Pool operational state
    pub phase: PoolPhase,
    
    // Cross-chain liquidity reserves (built from fees)
    pub reserves: HashMap<ChainId, HashMap<Asset, LiquidityReserve>>,
    
    // Integrated dev team business model
    pub dev_team_business: DevTeamBusinessModel,
    
    // Pool metrics
    pub total_liquidity_usd: f64,
    pub monthly_volume: f64,
    pub fee_collection_rate: f64,
    
    // Bootstrap targets
    pub bootstrap_targets: HashMap<Asset, u64>,
    
    // ===== TREASURY MANAGEMENT FIELDS =====
    pub treasury_config: TreasuryConfig,
    pub payment_addresses: Vec<PaymentAddress>,
    pub treasury_transactions: Vec<TreasuryTransaction>,
    pub treasury_balances: Vec<TreasuryBalance>,
    pub withdrawal_requests: Vec<WithdrawalRequest>,
    pub last_balance_update: u64,
}

impl Default for PoolState {
    fn default() -> Self {
        let mut bootstrap_targets = HashMap::new();
        bootstrap_targets.insert(Asset::USDC, 200_000_000_000); // $200K USDC (6 decimals)
        bootstrap_targets.insert(Asset::USDT, 100_000_000_000); // $100K USDT (6 decimals)
        bootstrap_targets.insert(Asset::ETH, 60_000_000_000_000_000_000_u128.min(u64::MAX as u128) as u64); // 60 ETH (capped at u64::MAX)
        bootstrap_targets.insert(Asset::BTC, 3_00_000_000); // 3 BTC (8 decimals)
        bootstrap_targets.insert(Asset::SOL, 2000_000_000_000); // 2000 SOL (9 decimals)
        
        PoolState {
            phase: PoolPhase::default(),
            reserves: HashMap::new(),
            dev_team_business: DevTeamBusinessModel::default(),
            total_liquidity_usd: 0.0,
            monthly_volume: 0.0,
            fee_collection_rate: 0.004, // 0.4% pool accumulation rate
            bootstrap_targets,
            
            // Treasury management defaults
            treasury_config: TreasuryConfig::default(),
            payment_addresses: Vec::new(),
            treasury_transactions: Vec::new(),
            treasury_balances: Vec::new(),
            withdrawal_requests: Vec::new(),
            last_balance_update: ic_cdk::api::time(),
        }
    }
}

// =============================================================================
// ANALYTICS & REPORTING
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FinancialOverview {
    // Pool metrics
    pub total_liquidity: f64,
    pub monthly_pool_growth: f64,
    pub bootstrap_progress: f64,
    
    // Business metrics
    pub monthly_revenue: f64,
    pub dev_1_pending: f64,
    pub dev_2_pending: f64,
    pub emergency_fund: f64,
    
    // Health indicators
    pub pool_health: String,
    pub business_health: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ArbitrageOpportunity {
    pub asset_pair: (String, String),
    pub buy_chain: ChainId,
    pub sell_chain: ChainId,
    pub price_difference: f64,
    pub expected_profit: f64,
    pub required_capital: f64,
    pub confidence_score: f64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PremiumAccess {
    pub principal: Principal,
    pub tier: String, // "Premium+" for all dev team members
    pub fee_rate: f64, // 0.001 (0.1%) for dev team
    pub granted_by: Principal, // Who granted this access
    pub granted_at: u64,
    pub expires_at: Option<u64>, // None = permanent for dev team
}

// =============================================================================
// TREASURY MANAGEMENT TYPES
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TreasuryConfig {
    pub payment_addresses: HashMap<String, String>, // chain_asset -> address
    pub hot_wallet_limits: HashMap<String, f64>,    // chain_asset -> max_amount_usd
    pub multi_sig_thresholds: HashMap<String, f64>, // chain_asset -> threshold_usd
    pub withdrawal_approvers: Vec<Principal>,        // who can approve withdrawals
    pub auto_transfer_enabled: bool,                 // auto transfer to cold storage
    pub cold_storage_threshold: f64,                 // amount to trigger cold transfer
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PaymentAddress {
    pub chain: String,           // "ethereum", "polygon", "bitcoin"
    pub asset: String,           // "usdc", "usdt", "eth", "btc"
    pub address: String,         // wallet address
    pub address_type: AddressType, // Hot, Warm, Cold
    pub max_balance_usd: Option<f64>, // max USD amount before transfer
    pub created_at: u64,
    pub last_used: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum AddressType {
    Hot,      // For automated operations (daily use)
    Warm,     // For business operations (multi-sig)
    Cold,     // For long-term storage (offline)
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TreasuryTransaction {
    pub id: String,
    pub transaction_type: TreasuryTransactionType,
    pub chain: String,
    pub asset: String,
    pub amount: f64,
    pub amount_usd: f64,                    // USD value at time of transaction
    pub from_address: String,
    pub to_address: String,
    pub tx_hash: Option<String>,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub initiated_by: Principal,
    pub notes: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum TreasuryTransactionType {
    SubscriptionPayment,
    TransactionFeeRevenue,    // 30% of platform transaction fees
    WithdrawalToTeam,
    TransferToCold,
    TransferToWarm,
    Rebalancing,
    EmergencyWithdrawal,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    RequiresApproval,
    Cancelled,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TreasuryBalance {
    pub chain: String,
    pub asset: String,
    pub amount: f64,
    pub amount_usd: f64,
    pub last_updated: u64,
    pub last_tx_hash: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WithdrawalRequest {
    pub id: String,
    pub requested_by: Principal,
    pub chain: String,
    pub asset: String,
    pub amount: f64,
    pub amount_usd: f64,
    pub destination_address: String,
    pub reason: String,
    pub status: WithdrawalStatus,
    pub required_approvals: u32,
    pub current_approvals: Vec<Principal>,
    pub created_at: u64,
    pub approved_at: Option<u64>,
    pub executed_at: Option<u64>,
    pub tx_hash: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum WithdrawalStatus {
    PendingApproval,
    Approved,
    Executed,
    Rejected,
    Expired,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TreasuryHealthReport {
    pub total_usd_value: f64,
    pub total_assets: usize,
    pub balances_over_limit: Vec<String>,
    pub last_payment_timestamp: Option<u64>,
    pub pending_withdrawals: usize,
    pub hot_wallet_utilization: f64,        // percentage of limits used
    pub largest_single_balance: f64,         // largest balance in USD
    pub diversification_score: f64,          // how spread across assets/chains
    pub security_alerts: Vec<String>,
}

impl Default for TreasuryConfig {
    fn default() -> Self {
        TreasuryConfig {
            payment_addresses: HashMap::new(),
            hot_wallet_limits: HashMap::new(),
            multi_sig_thresholds: HashMap::new(),
            withdrawal_approvers: Vec::new(),
            auto_transfer_enabled: false,
            cold_storage_threshold: 50000.0, // $50K default
        }
    }
}

// =============================================================================
// STABLE STORAGE IMPLEMENTATIONS
// =============================================================================

impl Storable for PoolState {
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded {
        max_size: 1024 * 1024, // 1MB max size
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

impl Asset {
    pub fn to_string(&self) -> String {
        match self {
            Asset::BTC => "BTC".to_string(),
            Asset::ETH => "ETH".to_string(),
            Asset::USDC => "USDC".to_string(),
            Asset::USDT => "USDT".to_string(),
            Asset::DAI => "DAI".to_string(),
            Asset::SOL => "SOL".to_string(),
            Asset::MATIC => "MATIC".to_string(),
            Asset::AVAX => "AVAX".to_string(),
        }
    }
    
    pub fn decimals(&self) -> u8 {
        match self {
            Asset::BTC => 8,
            Asset::ETH => 18,
            Asset::USDC => 6,
            Asset::USDT => 6,
            Asset::DAI => 18,
            Asset::SOL => 9,
            Asset::MATIC => 18,
            Asset::AVAX => 18,
        }
    }
}

impl ChainId {
    pub fn to_string(&self) -> String {
        match self {
            ChainId::Bitcoin => "Bitcoin".to_string(),
            ChainId::Ethereum => "Ethereum".to_string(),
            ChainId::Arbitrum => "Arbitrum".to_string(),
            ChainId::Optimism => "Optimism".to_string(),
            ChainId::Polygon => "Polygon".to_string(),
            ChainId::Base => "Base".to_string(),
            ChainId::Solana => "Solana".to_string(),
            ChainId::Avalanche => "Avalanche".to_string(),
        }
    }
}