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
    Terminating {
        initiated_at: u64,
        termination_request: PoolTerminationRequest,
    },
    Terminated {
        terminated_at: u64,
        final_asset_distribution: Vec<AssetDistribution>,
        termination_reason: String,
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
    
    // SECURITY: Enhanced rate limiting with separate counters for different operations
    pub last_team_change: u64,
    pub last_bootstrap_change: u64,
    pub last_configuration_change: u64,
    pub last_financial_operation: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum EarningsAllocation {
    Percentage(f64),           // e.g., 25.0 for 25% of profits
    FixedMonthlyUSD(f64),      // e.g., 5000.0 for $5,000/month
    FixedPerTransaction(f64),  // e.g., 10.0 for $10 per transaction
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemberEarningsConfig {
    pub allocation: EarningsAllocation,
    pub role: TeamRole,
    pub is_active: bool,           // Can be temporarily disabled
    pub vesting_cliff_months: u64, // Months before earning starts
    pub vesting_period_months: u64, // Total vesting period
    pub joined_timestamp: u64,
    pub last_modified_by: Principal,
    pub last_modified_time: u64,
}

impl Default for MemberEarningsConfig {
    fn default() -> Self {
        MemberEarningsConfig {
            allocation: EarningsAllocation::Percentage(0.0),
            role: TeamRole::Developer,
            is_active: true,
            vesting_cliff_months: 0,
            vesting_period_months: 12, // 1 year default vesting
            joined_timestamp: ic_cdk::api::time(),
            last_modified_by: Principal::anonymous(),
            last_modified_time: ic_cdk::api::time(),
        }
    }
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
    
    // Team earnings (multi-token support with original token preservation)
    pub team_member_earnings: HashMap<Principal, MemberEarnings>,
    pub total_distributed_profits: f64,
    
    // Business reserves
    pub emergency_fund: f64,
    pub reinvestment_fund: f64,
    
    // Distribution configuration
    pub minimum_distribution_threshold: f64,  // $5,000 minimum
    pub distribution_frequency: u64,          // Monthly (2,629,800 seconds)
    pub last_distribution_time: u64,
    pub member_earnings_config: HashMap<Principal, MemberEarningsConfig>, // Individual earnings allocation
}

impl Default for TeamHierarchy {
    fn default() -> Self {
        // SECURITY: Use a safe placeholder that will fail fast if not properly initialized
        // This ensures the canister cannot operate without proper owner initialization
        let placeholder_principal = Principal::anonymous();
        
        TeamHierarchy {
            owner_principal: placeholder_principal, // MUST be overridden in init() or canister will fail
            senior_managers: Vec::new(),
            operations_managers: Vec::new(), 
            tech_managers: Vec::new(),
            developers: Vec::new(),
            pending_approvals: Vec::new(),
            next_request_id: 1,
            // SECURITY: Initialize all rate limiting timestamps
            last_team_change: ic_cdk::api::time(),
            last_bootstrap_change: ic_cdk::api::time(),
            last_configuration_change: ic_cdk::api::time(),
            last_financial_operation: ic_cdk::api::time(),
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
            member_earnings_config: HashMap::new(), // No earnings allocated by default
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
    pub treasury_transactions: Vec<TreasuryTransaction>, // SECURITY: Consider bounded collection
    pub treasury_balances: Vec<TreasuryBalance>,
    pub withdrawal_requests: Vec<WithdrawalRequest>, // SECURITY: Consider bounded collection
    pub last_balance_update: u64,
    
    // SECURITY: Storage limits and monitoring
    pub storage_metrics: StorageMetrics,
    
    // POOL TERMINATION MANAGEMENT
    pub active_termination_request: Option<PoolTerminationRequest>,
    pub termination_history: Vec<PoolTerminationRequest>, // Track failed/cancelled attempts
    pub cofounder_principal: Option<Principal>, // Set during initialization
    
    // SECURITY: Race condition prevention (with upgrade compatibility)
    pub state_version: u64, // Incremented on every state change to prevent race conditions
    pub termination_nonce: u64, // Prevents replay attacks on termination operations
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
            
            // SECURITY: Initialize storage monitoring
            storage_metrics: StorageMetrics::default(),
            
            // Pool termination management
            active_termination_request: None,
            termination_history: Vec::new(),
            cofounder_principal: None, // Must be set via set_cofounder function
            
            // SECURITY: Race condition prevention
            state_version: 1, // Start at version 1
            termination_nonce: 0, // Start at 0
        }
    }
}

// =============================================================================
// MULTI-TOKEN EARNINGS SYSTEM
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenBalance {
    pub asset: Asset,
    pub amount: u64, // Atomic units (e.g., satoshis for BTC, wei for ETH)
    pub last_updated: u64,
    pub usd_value_at_time: f64, // For analytics/reporting
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MemberEarnings {
    pub balances: HashMap<Asset, TokenBalance>, // BTC, ETH, USDC, etc.
    pub total_usd_value: f64, // Calculated from all token balances
    pub last_distribution_time: u64,
    pub withdrawal_addresses: HashMap<ChainId, String>, // Chain-specific withdrawal addresses
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum WithdrawalOption {
    OriginalTokens, // Keep as received (BTC, ETH, USDC, etc.)
    ConvertToICP,   // Convert everything to ICP at withdrawal
    Mixed {         // Custom selection per token
        original_tokens: Vec<Asset>,
        convert_to_icp: Vec<Asset>,
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TokenTransfer {
    pub asset: Asset,
    pub amount: u64,
    pub recipient: Principal,
    pub transfer_type: TransferType,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum TransferType {
    OriginalToken { chain: ChainId },
    ConvertedToICP,
}

impl Default for MemberEarnings {
    fn default() -> Self {
        MemberEarnings {
            balances: HashMap::new(),
            total_usd_value: 0.0,
            last_distribution_time: 0,
            withdrawal_addresses: HashMap::new(),
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
    pub total_team_pending: f64,
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
    // SECURITY: Separate authorization lists to prevent circular dependencies
    pub authorized_fee_depositors: Vec<Principal>,   // who can deposit fees (separate from withdrawal)
    pub authorized_payment_processors: Vec<Principal>, // who can process payments (separate from withdrawal)
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
    PaymentReceived,          // User payments (USDC/USDT)
    RefundIssued,            // Refunds to users
}

// =============================================================================
// PAYMENT METHOD TYPES
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PaymentMethod {
    pub id: String,
    pub chain: ChainId,
    pub asset: Asset,
    pub canister_address: String,         // ICP Chain Fusion generated address
    pub token_address: Option<String>,    // ERC-20/SPL token contract address
    pub is_native_integration: bool,      // true if using ICP Chain Fusion
    pub key_derivation_path: Vec<Vec<u8>>, // derivation path for threshold cryptography
    pub enabled: bool,
    pub min_amount_usd: f64,
    pub max_amount_usd: f64,
    pub processing_fee_bps: u16,          // basis points (100 = 1%)
    pub confirmation_blocks: u32,
    pub estimated_settlement_time: u64,    // seconds
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Payment {
    pub id: String,
    pub user_principal: Principal,
    pub payment_method: PaymentMethod,
    pub amount: f64,                      // amount in token units
    pub amount_usd: f64,                  // USD value at payment time
    pub fee_amount: f64,                  // processing fee in token units
    pub fee_amount_usd: f64,              // processing fee in USD
    pub destination_address: String,       // our treasury address
    pub sender_address: String,           // user's wallet address
    pub tx_hash: Option<String>,
    pub status: PaymentStatus,
    pub initiated_at: u64,
    pub confirmed_at: Option<u64>,
    pub expires_at: u64,                  // payment expiration time
    pub purpose: PaymentPurpose,
    pub metadata: PaymentMetadata,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum PaymentStatus {
    Created,           // Payment request created
    WaitingConfirmation, // Transaction sent, waiting for confirmations
    Confirmed,         // Payment confirmed and processed
    Failed,           // Payment failed
    Expired,          // Payment expired
    Refunded,         // Payment refunded to user
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum PaymentPurpose {
    Subscription { plan: String, duration_months: u32 },
    WorkflowExecution { workflow_id: String, estimated_cost: f64 },
    PremiumFeatures { features: Vec<String> },
    TopUp { credits: f64 },
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PaymentMetadata {
    pub invoice_id: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub refund_policy: RefundPolicy,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum RefundPolicy {
    NoRefund,
    FullRefund { within_hours: u32 },
    PartialRefund { percentage: u8, within_hours: u32 },
    CustomTerms { terms: String },
}

// Stablecoin configurations for each supported chain
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct StablecoinConfig {
    pub usdc_ethereum: PaymentMethod,
    pub usdt_ethereum: PaymentMethod,
    pub usdc_polygon: PaymentMethod,
    pub usdt_polygon: PaymentMethod,
    pub usdc_arbitrum: PaymentMethod,
    pub usdt_arbitrum: PaymentMethod,
    pub usdc_base: PaymentMethod,
    pub usdt_base: PaymentMethod,
    pub usdc_solana: PaymentMethod,
    pub usdt_solana: PaymentMethod,
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

// SECURITY: Storage monitoring and limits
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct StorageMetrics {
    pub max_treasury_transactions: usize,
    pub max_withdrawal_requests: usize,
    pub max_payment_addresses: usize,
    pub current_memory_usage: u64,
    pub last_cleanup_time: u64,
    pub transactions_pruned: u64,
}

// =============================================================================
// POOL TERMINATION TYPES
// =============================================================================

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PoolTerminationRequest {
    pub id: String,
    pub initiated_by: Principal,
    pub reason: String,
    pub asset_distribution_plan: Vec<AssetDistribution>,
    pub owner_approval: Option<TerminationApproval>,
    pub cofounder_approval: Option<TerminationApproval>,
    pub created_at: u64,
    pub expires_at: u64, // 48 hours to get both approvals
    pub emergency_termination: bool, // Skip some validations for emergency
    pub expected_state_version: u64, // Expected pool state version when this request was created
    pub termination_nonce: u64, // Unique nonce for this termination attempt
    pub secure_confirmation_phrase: String, // Cryptographically secure confirmation phrase
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TerminationApproval {
    pub approver: Principal,
    pub approved_at: u64,
    pub signature_confirmation: String, // Extra confirmation string
    pub notes: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AssetDistribution {
    pub chain: String,
    pub asset: String,
    pub total_amount: f64,
    pub destination_address: String,
    pub estimated_usd_value: f64,
    pub status: DistributionStatus,
    pub tx_hash: Option<String>,
    pub executed_at: Option<u64>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum DistributionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct TerminationSummary {
    pub total_assets_distributed: f64, // USD value
    pub chains_processed: Vec<String>,
    pub successful_distributions: u32,
    pub failed_distributions: u32,
    pub termination_initiated_at: u64,
    pub termination_completed_at: Option<u64>,
    pub final_state_hash: String, // Hash of final state for auditing
}

impl Default for StorageMetrics {
    fn default() -> Self {
        StorageMetrics {
            // SECURITY: Realistic storage limits to prevent DoS attacks
            max_treasury_transactions: 1000,   // Max 1K transactions (was 10K)
            max_withdrawal_requests: 100,      // Max 100 withdrawal requests (was 1K)
            max_payment_addresses: 50,         // Max 50 payment addresses (was 100)
            current_memory_usage: 0,
            last_cleanup_time: ic_cdk::api::time(),
            transactions_pruned: 0,
        }
    }
}

impl Default for TreasuryConfig {
    fn default() -> Self {
        TreasuryConfig {
            payment_addresses: HashMap::new(),
            hot_wallet_limits: HashMap::new(),
            multi_sig_thresholds: HashMap::new(),
            withdrawal_approvers: Vec::new(),
            // SECURITY: Initialize separate authorization lists
            authorized_fee_depositors: Vec::new(),
            authorized_payment_processors: Vec::new(),
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
        // SECURITY: Handle upgrade from pre-security version
        match candid::decode_one::<Self>(&bytes) {
            Ok(state) => state,
            Err(_) => {
                // Try to decode as the old version without security fields
                ic_cdk::println!("SECURITY: Attempting migration from pre-security version");
                
                // For migration, we'll just return a default state and let post_upgrade handle it
                // This is a fallback - the actual migration should be handled in post_upgrade
                let default_state = Self::default();
                
                // Log the migration attempt
                ic_cdk::println!("SECURITY: Using default state for migration - will be corrected in post_upgrade");
                
                default_state
            }
        }
    }
}

// =============================================================================
// UPGRADE COMPATIBILITY FUNCTIONS
// =============================================================================

#[allow(dead_code)]
fn default_state_version() -> u64 {
    1 // Start at version 1 for upgraded canisters
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