use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub nodes: Vec<WorkflowNode>,
    pub connections: Vec<NodeConnection>,
    pub triggers: Vec<WorkflowTrigger>,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WorkflowNode {
    pub id: String,
    pub node_type: String,
    pub position: NodePosition,
    pub configuration: NodeConfiguration,
    pub metadata: NodeMetadata,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodeConfiguration {
    pub parameters: HashMap<String, ConfigValue>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodeMetadata {
    pub label: String,
    pub description: Option<String>,
    pub version: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodeConnection {
    pub id: String,
    pub source_node_id: String,
    pub source_output: String,
    pub target_node_id: String,
    pub target_input: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum WorkflowTrigger {
    Manual,
    Schedule { cron: String },
    Webhook { path: String },
    Event { event_type: String, conditions: HashMap<String, ConfigValue> },
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub trigger_data: Option<HashMap<String, ConfigValue>>,
    pub node_executions: Vec<NodeExecution>,
    pub error_message: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodeExecution {
    pub node_id: String,
    pub status: ExecutionStatus,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub input_data: Option<HashMap<String, ConfigValue>>,
    pub output_data: Option<HashMap<String, ConfigValue>>,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodeDefinition {
    pub node_type: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub input_schema: Vec<ParameterSchema>,
    pub output_schema: Vec<ParameterSchema>,
    pub configuration_schema: Vec<ParameterSchema>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ParameterSchema {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub description: Option<String>,
    pub default_value: Option<ConfigValue>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodeInput {
    pub data: HashMap<String, ConfigValue>,
    pub context: ExecutionContext,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct NodeOutput {
    pub data: HashMap<String, ConfigValue>,
    pub next_nodes: Vec<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ExecutionContext {
    pub workflow_id: String,
    pub execution_id: String,
    pub user_id: String,
    pub timestamp: u64,
    pub global_variables: HashMap<String, ConfigValue>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum NodeError {
    ConfigurationError(String),
    ExecutionError(String),
    NetworkError(String),
    ValidationError(String),
    TimeoutError,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum ValidationError {
    MissingRequiredParameter(String),
    InvalidParameterType { parameter: String, expected: String, got: String },
    InvalidParameterValue(String),
    SchemaValidationFailed(String),
    DuplicateNodeId(String),
    InvalidNodeConfiguration(String),
    InvalidConnection { connection_id: String, reason: String },
    CycleDetected,
    InvalidTrigger(String),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
    pub retry_on_errors: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,
            retry_on_errors: vec![
                "NetworkError".to_string(),
                "TimeoutError".to_string(),
                "TemporaryError".to_string(),
            ],
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EventListener {
    pub id: String,
    pub workflow_id: String,
    pub event_type: String,
    pub conditions: HashMap<String, ConfigValue>,
    pub active: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ScheduledWorkflow {
    pub id: String,
    pub workflow_id: String,
    pub cron_expression: String,
    pub next_execution: u64,
    pub active: bool,
    pub timer_id: Option<String>,
}

// Enhanced persistent timer system
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ScheduledExecution {
    pub workflow_id: String,
    pub next_execution: u64,
    pub interval: Option<u64>, // for recurring workflows
    pub timer_id: Option<String>,
    pub schedule_type: ScheduleType,
    pub metadata: HashMap<String, ConfigValue>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum ScheduleType {
    Once,
    Interval { seconds: u64 },
    Cron { expression: String },
    Heartbeat, // Executed on every heartbeat
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WebhookEvent {
    pub event_type: String,
    pub data: HashMap<String, ConfigValue>,
    pub timestamp: u64,
    pub source: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WorkflowEvent {
    pub id: String,
    pub event_type: String,
    pub workflow_id: Option<String>,
    pub execution_id: Option<String>,
    pub data: HashMap<String, ConfigValue>,
    pub timestamp: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CircuitBreaker {
    pub node_type: String,
    pub failure_threshold: u32,
    pub recovery_timeout_ms: u64,
    pub current_failures: u32,
    pub state: CircuitBreakerState,
    pub last_failure_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum CircuitBreakerState {
    Closed,  // Normal operation
    Open,    // Failing, blocking requests
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new(node_type: String) -> Self {
        Self {
            node_type,
            failure_threshold: 5,
            recovery_timeout_ms: 60000, // 1 minute
            current_failures: 0,
            state: CircuitBreakerState::Closed,
            last_failure_time: None,
        }
    }
    
    pub fn can_execute(&mut self) -> bool {
        use ic_cdk::api;
        
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    let elapsed = api::time().saturating_sub(last_failure) / 1_000_000;
                    if elapsed > self.recovery_timeout_ms {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    pub fn on_success(&mut self) {
        self.current_failures = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure_time = None;
    }
    
    pub fn on_failure(&mut self) {
        use ic_cdk::api;
        
        self.current_failures += 1;
        self.last_failure_time = Some(api::time());
        
        if self.current_failures >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>, // (from, to)
}

// Zero-downtime architecture types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WorkflowState {
    pub active_workflows: Vec<(String, WorkflowExecution)>, // (workflow_id, execution)
    pub scheduled_executions: Vec<(u64, String)>, // (timestamp, workflow_id)
    pub user_balances: Vec<(String, PortfolioState)>, // (principal, portfolio)
    pub execution_history: Vec<ExecutionRecord>,
    pub system_health: SystemHealth,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PortfolioState {
    pub user_id: String,
    pub total_value_usd: f64,
    pub positions: Vec<Position>,
    pub last_updated: u64,
    pub risk_profile: RiskProfile,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Position {
    pub asset: String,
    pub amount: f64,
    pub value_usd: f64,
    pub platform: String,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RiskProfile {
    pub max_drawdown_percent: f64,
    pub stop_loss_enabled: bool,
    pub liquidation_threshold: f64,
    pub diversification_min: u32,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ExecutionRecord {
    pub id: String,
    pub workflow_id: String,
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub duration_ms: Option<u64>,
    pub gas_used: Option<u64>,
    pub error_message: Option<String>,
    pub node_count: u32,
    pub retry_count: u32,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SystemHealth {
    pub last_heartbeat: u64,
    pub active_workflows: u32,
    pub failed_executions_last_24h: u32,
    pub average_execution_time_ms: f64,
    pub chain_fusion_connectivity: Vec<(String, bool)>, // (chain, connected)
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            active_workflows: Vec::new(),
            scheduled_executions: Vec::new(),
            user_balances: Vec::new(),
            execution_history: Vec::new(),
            system_health: SystemHealth::default(),
        }
    }
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self {
            last_heartbeat: 0,
            active_workflows: 0,
            failed_executions_last_24h: 0,
            average_execution_time_ms: 0.0,
            chain_fusion_connectivity: vec![
                ("BTC".to_string(), false),
                ("ETH".to_string(), false),
                ("ICP".to_string(), true),
            ],
            memory_usage_percent: 0.0,
            cpu_usage_percent: 0.0,
        }
    }
}

// Recovery and fallback types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct WorkflowRecovery {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub fallback_strategy: Option<FallbackStrategy>,
    pub emergency_actions: Vec<EmergencyAction>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum FallbackStrategy {
    UseAlternativeNode { node_id: String },
    SkipNode,
    StopExecution,
    UseDefaultValue { value: ConfigValue },
    NotifyAndContinue,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum EmergencyAction {
    SendNotification { recipient: String, message: String },
    ExecuteWorkflow { workflow_id: String },
    LiquidatePosition { asset: String, percentage: f64 },
    PauseAllWorkflows,
    EnableSafeMode,
}

impl Default for WorkflowRecovery {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 5000,
            fallback_strategy: Some(FallbackStrategy::NotifyAndContinue),
            emergency_actions: vec![
                EmergencyAction::SendNotification {
                    recipient: "admin".to_string(),
                    message: "Workflow execution failed after maximum retries".to_string(),
                }
            ],
        }
    }
}

// User Management and Subscription System
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub principal_id: String,
    pub subscription_tier: SubscriptionTier,
    pub created_at: u64,
    pub updated_at: u64,
    pub monthly_volume: f64,
    pub total_volume: f64,
    pub active: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum SubscriptionTier {
    Standard,  // $0/month - Telegram & Discord only
    Premium,   // $19/month - All integrations
    Pro,       // $149/month - All integrations + advanced features
}

impl SubscriptionTier {
    pub fn allowed_node_types(&self) -> Vec<String> {
        match self {
            SubscriptionTier::Standard => vec![
                "telegram".to_string(),
                "discord".to_string(),
                // Core workflow nodes always available
                "delay".to_string(),
                "condition".to_string(),
                "transform".to_string(),
                "timer".to_string(),
            ],
            SubscriptionTier::Premium | SubscriptionTier::Pro => vec![
                "telegram".to_string(),
                "discord".to_string(),
                "twitter".to_string(),
                "facebook".to_string(),
                "email".to_string(),
                "linkedin".to_string(),
                "instagram".to_string(),
                "webhook".to_string(),
                "http_request".to_string(),
                // Core workflow nodes
                "delay".to_string(),
                "condition".to_string(),
                "transform".to_string(),
                "timer".to_string(),
                // DeFi nodes
                "bitcoin_portfolio".to_string(),
                "bitcoin_send".to_string(),
                "bitcoin_address".to_string(),
                "bitcoin_balance".to_string(),
                "ethereum_portfolio".to_string(),
                "ethereum_send".to_string(),
                "ethereum_address".to_string(),
                "ethereum_gas_estimate".to_string(),
                "l2_optimization".to_string(),
                "bridge_analysis".to_string(),
            ],
        }
    }
    
    pub fn monthly_fee(&self) -> f64 {
        match self {
            SubscriptionTier::Standard => 0.0,
            SubscriptionTier::Premium => 19.0,
            SubscriptionTier::Pro => 149.0,
        }
    }
    
    pub fn transaction_fee_rate(&self) -> f64 {
        match self {
            SubscriptionTier::Standard => 0.0085, // 0.85%
            SubscriptionTier::Premium => 0.0025,  // 0.25%
            SubscriptionTier::Pro => 0.001,       // 0.1%
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserSubscriptionInfo {
    pub user: User,
    pub payment_history: Vec<PaymentRecord>,
    pub usage_stats: UsageStats,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PaymentRecord {
    pub id: String,
    pub amount: f64,
    pub currency: String,
    pub payment_date: u64,
    pub subscription_tier: SubscriptionTier,
    pub status: PaymentStatus,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UsageStats {
    pub total_workflows_created: u32,
    pub total_executions: u32,
    pub monthly_executions: u32,
    pub last_activity: u64,
    pub preferred_node_types: Vec<String>,
}