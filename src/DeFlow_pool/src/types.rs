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