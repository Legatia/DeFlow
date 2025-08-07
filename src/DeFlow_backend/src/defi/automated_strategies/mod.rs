// Automated DeFi Strategies Module - Complete integration
// Day 12: Multi-strategy coordination and automated execution

pub mod strategy_registry;
pub mod execution_engine;
pub mod opportunity_scanner;
pub mod performance_tracker;
pub mod risk_manager;
pub mod coordination_engine;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use strategy_registry::*;
pub use execution_engine::*;
pub use opportunity_scanner::*;
pub use performance_tracker::*;
pub use risk_manager::*;
pub use coordination_engine::*;

// Import necessary dependencies
use crate::defi::yield_farming::ChainId;
use crate::defi::yield_farming::{DeFiProtocol};
use crate::defi::protocol_integrations::{DeFiProtocolIntegrations, LiveYieldOpportunity, LiveArbitrageOpportunity};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

// Core strategy data types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ActiveStrategy {
    pub id: String,
    pub user_id: String,
    pub config: StrategyConfig,
    pub status: StrategyStatus,
    pub allocated_capital: f64,
    pub performance_metrics: StrategyPerformanceMetrics,
    pub risk_metrics: StrategyRiskMetrics,
    pub execution_history: Vec<StrategyExecutionResult>,
    pub next_execution: Option<u64>,
    pub created_at: u64,
    pub last_updated: u64,
    pub last_rebalance: Option<u64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub description: String,
    pub strategy_type: StrategyType,
    pub target_chains: Vec<ChainId>,
    pub target_protocols: Vec<DeFiProtocol>,
    pub risk_level: u8, // 1-10 scale
    pub max_allocation_usd: f64,
    pub min_return_threshold: f64,
    pub execution_interval_minutes: u64,
    pub gas_limit_usd: f64,
    pub auto_compound: bool,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq)]
pub enum StrategyStatus {
    Created,
    Active,
    Paused,
    Stopped,
    Error,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum StrategyType {
    YieldFarming(YieldFarmingConfig),
    Arbitrage(ArbitrageConfig),
    Rebalancing(RebalancingConfig),
    LiquidityMining(LiquidityMiningConfig),
    DCA(DCAConfig),
    Composite(Vec<CompositeStrategyConfig>),
}

// Strategy-specific configurations
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct YieldFarmingConfig {
    pub min_apy_threshold: f64,
    pub preferred_tokens: Vec<String>,
    pub max_impermanent_loss_percentage: f64,
    pub auto_harvest_rewards: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ArbitrageConfig {
    pub min_profit_percentage: f64,
    pub max_execution_time_seconds: u64,
    pub max_slippage_percentage: f64,
    pub preferred_dex_pairs: Vec<(String, String)>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RebalancingConfig {
    pub target_allocation: HashMap<String, f64>, // asset -> percentage
    pub rebalance_threshold_percentage: f64,
    pub rebalance_frequency_hours: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct LiquidityMiningConfig {
    pub min_apr_threshold: f64,
    pub preferred_pairs: Vec<(String, String)>,
    pub max_pool_concentration_percentage: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DCAConfig {
    pub target_token: String,
    pub amount_per_execution: f64,
    pub price_threshold_percentage: Option<f64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CompositeStrategyConfig {
    pub sub_strategy: StrategyType,
    pub allocation_percentage: f64,
    pub priority: u8,
}

// Performance metrics
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyPerformanceMetrics {
    pub total_executions: u32,
    pub successful_executions: u32,
    pub total_pnl: f64,
    pub roi_percentage: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_percentage: f64,
    pub avg_execution_time_seconds: f64,
    pub total_gas_spent_usd: f64,
    pub win_rate_percentage: f64,
}

impl Default for StrategyPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            total_pnl: 0.0,
            roi_percentage: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown_percentage: 0.0,
            avg_execution_time_seconds: 0.0,
            total_gas_spent_usd: 0.0,
            win_rate_percentage: 0.0,
        }
    }
}

// Risk metrics
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyRiskMetrics {
    pub volatility_score: f64,
    pub liquidity_risk_score: f64,
    pub concentration_risk_score: f64,
    pub smart_contract_risk_score: f64,
    pub last_risk_assessment: u64,
}

impl Default for StrategyRiskMetrics {
    fn default() -> Self {
        Self {
            volatility_score: 5.0,
            liquidity_risk_score: 5.0,
            concentration_risk_score: 5.0,
            smart_contract_risk_score: 5.0,
            last_risk_assessment: 0,
        }
    }
}

// Execution result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyExecutionResult {
    pub execution_id: String,
    pub strategy_id: String,
    pub user_id: String,
    pub opportunity_id: String,
    pub action_type: String,
    pub amount_usd: f64,
    pub expected_return: f64,
    pub actual_return: f64,
    pub gas_cost_usd: f64,
    pub execution_time_seconds: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub transaction_hashes: Vec<String>,
    pub executed_at: u64,
}

// Opportunity types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyOpportunity {
    pub id: String,
    pub opportunity_type: OpportunityType,
    pub chain: ChainId,
    pub protocol: DeFiProtocol,
    pub expected_return_percentage: f64,
    pub risk_score: u8,
    pub estimated_gas_cost: f64,
    pub liquidity_score: f64,
    pub time_sensitivity_minutes: u64,
    pub discovered_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum OpportunityType {
    YieldFarming { 
        apy: f64, 
        tokens: Vec<String>, 
        pool_address: String 
    },
    Arbitrage { 
        profit_percentage: f64, 
        token_pair: (String, String), 
        dex_pair: (String, String) 
    },
    LiquidityMining { 
        apr: f64, 
        reward_tokens: Vec<String>, 
        pool_info: String 
    },
    Rebalancing { 
        current_allocation: HashMap<String, f64>, 
        target_allocation: HashMap<String, f64> 
    },
}

// Performance summary
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyPerformanceSummary {
    pub strategy_id: String,
    pub name: String,
    pub performance_metrics: StrategyPerformanceMetrics,
    pub risk_metrics: StrategyRiskMetrics,
    pub recent_executions: Vec<StrategyExecutionResult>,
    pub next_execution: Option<u64>,
    pub recommendations: Vec<String>,
}

/// Main automated strategy management system
#[derive(Debug, Clone)]
pub struct AutomatedStrategyManager {
    pub strategy_registry: StrategyRegistry,
    pub execution_engine: StrategyExecutionEngine,
    pub opportunity_scanner: OpportunityScanner,
    pub performance_tracker: StrategyPerformanceTracker,
    pub risk_manager: StrategyRiskManager,
    pub coordination_engine: MultiStrategyCoordinator,
    pub protocol_integrations: DeFiProtocolIntegrations,
    pub active_strategies: HashMap<String, ActiveStrategy>,
    pub user_preferences: HashMap<String, UserPreferences>,
    pub last_execution: u64,
    pub last_scan: u64,
}

impl AutomatedStrategyManager {
    pub fn new() -> Self {
        Self {
            strategy_registry: StrategyRegistry::new(),
            execution_engine: StrategyExecutionEngine::new(),
            opportunity_scanner: OpportunityScanner::new(),
            performance_tracker: StrategyPerformanceTracker::new(),
            risk_manager: StrategyRiskManager::new(),
            coordination_engine: MultiStrategyCoordinator::new(),
            protocol_integrations: DeFiProtocolIntegrations::new(),
            active_strategies: HashMap::new(),
            user_preferences: HashMap::new(),
            last_execution: 0,
            last_scan: 0,
        }
    }

    /// Initialize the automated strategy system
    pub fn initialize(&mut self) {
        // Initialize all components
        self.strategy_registry.initialize();
        ic_cdk::spawn(async move {
            // Note: opportunity_scanner.initialize() returns a Future that needs to be awaited
            // For now, we'll handle this in a spawned task
        });
        self.risk_manager.initialize_default_limits();

        ic_cdk::println!("Automated Strategy Manager initialized successfully");
    }

    /// Create a new strategy for a user
    pub fn create_strategy(&mut self, user_id: String, config: StrategyConfig) -> Result<String, StrategyError> {
        // Validate strategy configuration
        self.validate_strategy_config(&config)?;

        // Generate unique strategy ID
        let strategy_id = format!("strategy_{}_{:x}", user_id, ic_cdk::api::time());

        // Create active strategy
        let active_strategy = ActiveStrategy {
            id: strategy_id.clone(),
            user_id: user_id.clone(),
            config: config.clone(),
            status: StrategyStatus::Created,
            allocated_capital: 0.0,
            performance_metrics: StrategyPerformanceMetrics::default(),
            risk_metrics: StrategyRiskMetrics::default(),
            execution_history: Vec::new(),
            next_execution: None,
            created_at: ic_cdk::api::time(),
            last_updated: ic_cdk::api::time(),
            last_rebalance: None,
        };

        // Store the strategy
        self.active_strategies.insert(strategy_id.clone(), active_strategy);

        // Initialize user preferences if not exists
        if !self.user_preferences.contains_key(&user_id) {
            self.user_preferences.insert(user_id.clone(), UserPreferences::default());
        }

        ic_cdk::println!("Created new strategy {} for user {}", strategy_id, user_id);
        Ok(strategy_id)
    }

    /// Activate a strategy with capital allocation
    pub fn activate_strategy(&mut self, strategy_id: &str, capital_amount: f64) -> Result<(), StrategyError> {
        let strategy = self.active_strategies.get_mut(strategy_id)
            .ok_or(StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        // Validate capital allocation with risk manager
        self.risk_manager.validate_capital_allocation(strategy, capital_amount)?;

        // Update strategy
        strategy.allocated_capital = capital_amount;
        strategy.status = StrategyStatus::Active;
        strategy.last_updated = ic_cdk::api::time();
        strategy.next_execution = Some(ic_cdk::api::time() + strategy.config.execution_interval_minutes * 60 * 1_000_000_000);

        ic_cdk::println!("Activated strategy {} with ${:.2} capital", strategy_id, capital_amount);
        Ok(())
    }

    /// Execute all eligible strategies
    pub async fn execute_strategies(&mut self) -> Result<Vec<StrategyExecutionResult>, StrategyError> {
        let current_time = ic_cdk::api::time();
        let mut results = Vec::new();

        // Get eligible strategies for execution
        let eligible_strategy_ids: Vec<String> = self.active_strategies
            .iter()
            .filter(|(_, strategy)| self.is_strategy_eligible_for_execution(strategy, current_time))
            .map(|(id, _)| id.clone())
            .collect();

        if eligible_strategy_ids.is_empty() {
            return Ok(results);
        }

        // Coordinate strategies before execution
        let coordination_result = self.coordination_engine.coordinate_strategies(&mut self.active_strategies)?;
        ic_cdk::println!("Strategy coordination completed: {} optimizations applied", 
            coordination_result.optimizations_applied);

        // Scan for opportunities
        let opportunities = self.opportunity_scanner.scan_opportunities().await?;
        ic_cdk::println!("Found {} opportunities for strategy execution", opportunities.len());

        // Execute each eligible strategy
        for strategy_id in eligible_strategy_ids {
            let strategy = self.active_strategies.get(&strategy_id).unwrap().clone();
            
            // Pre-execution risk check
            if let Err(e) = self.risk_manager.pre_execution_check(&strategy) {
                ic_cdk::println!("Pre-execution risk check failed for strategy {}: {}", strategy_id, e);
                continue;
            }

            // Find suitable opportunity for this strategy
            let suitable_opportunity = self.find_suitable_opportunity(&strategy, &opportunities);
            
            if let Some(opportunity) = suitable_opportunity {
                // Execute the strategy
                let execution_result = self.execute_single_strategy(&strategy, opportunity).await;
                
                match execution_result {
                    Ok(mut result) => {
                        // Post-execution risk assessment
                        if let Err(risk_error) = self.risk_manager.post_execution_assessment(&strategy, &result) {
                            ic_cdk::println!("Post-execution risk assessment warning: {}", risk_error);
                        }

                        // Update performance tracker
                        if let Some(strategy_mut) = self.active_strategies.get_mut(&strategy_id) {
                            if let Err(perf_error) = self.performance_tracker.update_strategy_performance(strategy_mut, &result) {
                                ic_cdk::println!("Performance tracking error: {}", perf_error);
                            }
                            
                            // Update strategy execution history
                            strategy_mut.execution_history.push(result.clone());
                            strategy_mut.last_updated = current_time;
                            
                            // Schedule next execution
                            strategy_mut.next_execution = Some(current_time + strategy_mut.config.execution_interval_minutes * 60 * 1_000_000_000);
                        }

                        results.push(result);
                    }
                    Err(e) => {
                        ic_cdk::println!("Strategy {} execution failed: {}", strategy_id, e);
                        // Record failed execution
                        if let Some(strategy_mut) = self.active_strategies.get_mut(&strategy_id) {
                            let failed_result = StrategyExecutionResult {
                                execution_id: format!("failed_{:x}", current_time),
                                strategy_id: strategy_id.clone(),
                                user_id: strategy.user_id.clone(),
                                opportunity_id: "error".to_string(),
                                action_type: "failed_execution".to_string(),
                                amount_usd: 0.0,
                                expected_return: 0.0,
                                actual_return: 0.0,
                                gas_cost_usd: 0.0,
                                execution_time_seconds: 0,
                                success: false,
                                error_message: Some(e.to_string()),
                                transaction_hashes: vec![],
                                executed_at: current_time,
                            };
                            
                            strategy_mut.execution_history.push(failed_result.clone());
                            results.push(failed_result);
                        }
                    }
                }
            } else {
                ic_cdk::println!("No suitable opportunity found for strategy {}", strategy_id);
            }
        }

        self.last_execution = current_time;
        Ok(results)
    }

    /// Get user's strategies
    pub fn get_user_strategies(&self, user_id: &str) -> Vec<&ActiveStrategy> {
        self.active_strategies
            .values()
            .filter(|strategy| strategy.user_id == user_id)
            .collect()
    }

    /// Get strategy performance summary
    pub fn get_strategy_performance(&self, strategy_id: &str) -> Result<StrategyPerformanceSummary, StrategyError> {
        let strategy = self.active_strategies.get(strategy_id)
            .ok_or(StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        self.performance_tracker.generate_performance_summary(strategy)
    }

    /// Pause a strategy
    pub fn pause_strategy(&mut self, strategy_id: &str) -> Result<(), StrategyError> {
        let strategy = self.active_strategies.get_mut(strategy_id)
            .ok_or(StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        strategy.status = StrategyStatus::Paused;
        strategy.last_updated = ic_cdk::api::time();
        strategy.next_execution = None;

        ic_cdk::println!("Paused strategy {}", strategy_id);
        Ok(())
    }

    /// Resume a paused strategy
    pub fn resume_strategy(&mut self, strategy_id: &str) -> Result<(), StrategyError> {
        let strategy = self.active_strategies.get_mut(strategy_id)
            .ok_or(StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        if strategy.status == StrategyStatus::Paused {
            strategy.status = StrategyStatus::Active;
            strategy.last_updated = ic_cdk::api::time();
            strategy.next_execution = Some(ic_cdk::api::time() + strategy.config.execution_interval_minutes * 60 * 1_000_000_000);

            ic_cdk::println!("Resumed strategy {}", strategy_id);
        }

        Ok(())
    }

    /// Stop a strategy permanently
    pub fn stop_strategy(&mut self, strategy_id: &str) -> Result<(), StrategyError> {
        let strategy = self.active_strategies.get_mut(strategy_id)
            .ok_or(StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        strategy.status = StrategyStatus::Stopped;
        strategy.last_updated = ic_cdk::api::time();
        strategy.next_execution = None;

        ic_cdk::println!("Stopped strategy {}", strategy_id);
        Ok(())
    }

    /// Update strategy configuration
    pub fn update_strategy_config(&mut self, strategy_id: &str, new_config: StrategyConfig) -> Result<(), StrategyError> {
        // Validate new configuration
        self.validate_strategy_config(&new_config)?;

        let strategy = self.active_strategies.get_mut(strategy_id)
            .ok_or(StrategyError::StrategyNotFound(strategy_id.to_string()))?;

        strategy.config = new_config;
        strategy.last_updated = ic_cdk::api::time();

        ic_cdk::println!("Updated configuration for strategy {}", strategy_id);
        Ok(())
    }

    /// Get comprehensive strategy analytics
    pub fn get_strategy_analytics(&self, user_id: &str) -> StrategyAnalytics {
        let user_strategies: Vec<&ActiveStrategy> = self.get_user_strategies(user_id);
        
        let total_strategies = user_strategies.len();
        let active_strategies = user_strategies.iter()
            .filter(|s| s.status == StrategyStatus::Active)
            .count();
        
        let total_allocated_capital = user_strategies.iter()
            .map(|s| s.allocated_capital)
            .sum();

        let total_pnl = user_strategies.iter()
            .map(|s| s.performance_metrics.total_pnl)
            .sum();

        let weighted_roi = if total_allocated_capital > 0.0 {
            user_strategies.iter()
                .map(|s| (s.allocated_capital / total_allocated_capital) * s.performance_metrics.roi_percentage)
                .sum()
        } else {
            0.0
        };

        StrategyAnalytics {
            user_id: user_id.to_string(),
            total_strategies,
            active_strategies,
            total_allocated_capital,
            total_pnl,
            weighted_roi,
            best_performing_strategy: user_strategies.iter()
                .max_by(|a, b| a.performance_metrics.roi_percentage.partial_cmp(&b.performance_metrics.roi_percentage).unwrap())
                .map(|s| s.id.clone()),
            worst_performing_strategy: user_strategies.iter()
                .min_by(|a, b| a.performance_metrics.roi_percentage.partial_cmp(&b.performance_metrics.roi_percentage).unwrap())
                .map(|s| s.id.clone()),
            average_execution_time: user_strategies.iter()
                .map(|s| s.performance_metrics.avg_execution_time_seconds)
                .sum::<f64>() / user_strategies.len().max(1) as f64,
            total_gas_spent: user_strategies.iter()
                .map(|s| s.performance_metrics.total_gas_spent_usd)
                .sum(),
        }
    }

    // Private helper methods

    fn validate_strategy_config(&self, config: &StrategyConfig) -> Result<(), StrategyError> {
        if config.name.is_empty() {
            return Err(StrategyError::ValidationFailed("Strategy name cannot be empty".to_string()));
        }

        if config.target_chains.is_empty() {
            return Err(StrategyError::ValidationFailed("At least one target chain must be specified".to_string()));
        }

        if config.target_protocols.is_empty() {
            return Err(StrategyError::ValidationFailed("At least one target protocol must be specified".to_string()));
        }

        if config.max_allocation_usd <= 0.0 {
            return Err(StrategyError::ValidationFailed("Max allocation must be positive".to_string()));
        }

        if config.execution_interval_minutes < 1 {
            return Err(StrategyError::ValidationFailed("Execution interval must be at least 1 minute".to_string()));
        }

        Ok(())
    }

    fn is_strategy_eligible_for_execution(&self, strategy: &ActiveStrategy, current_time: u64) -> bool {
        strategy.status == StrategyStatus::Active &&
        strategy.allocated_capital > 0.0 &&
        strategy.next_execution.map_or(false, |next_time| current_time >= next_time)
    }

    fn find_suitable_opportunity(&self, strategy: &ActiveStrategy, opportunities: &[StrategyOpportunity]) -> Option<StrategyOpportunity> {
        for opportunity in opportunities {
            if self.is_opportunity_suitable_for_strategy(strategy, opportunity) {
                return Some(opportunity.clone());
            }
        }
        None
    }

    fn is_opportunity_suitable_for_strategy(&self, strategy: &ActiveStrategy, opportunity: &StrategyOpportunity) -> bool {
        // Check if opportunity matches strategy type
        let type_match = match (&strategy.config.strategy_type, &opportunity.opportunity_type) {
            (StrategyType::YieldFarming(_), OpportunityType::YieldFarming { .. }) => true,
            (StrategyType::Arbitrage(_), OpportunityType::Arbitrage { .. }) => true,
            (StrategyType::LiquidityMining(_), OpportunityType::LiquidityMining { .. }) => true,
            (StrategyType::Rebalancing(_), OpportunityType::Rebalancing { .. }) => true,
            _ => false,
        };

        if !type_match {
            return false;
        }

        // Check if chain is supported
        if !strategy.config.target_chains.contains(&opportunity.chain) {
            return false;
        }

        // Check if protocol is supported
        if !strategy.config.target_protocols.contains(&opportunity.protocol) {
            return false;
        }

        // Check minimum return threshold
        if opportunity.expected_return_percentage < strategy.config.min_return_threshold {
            return false;
        }

        // Check risk tolerance
        if opportunity.risk_score > strategy.config.risk_level {
            return false;
        }

        true
    }

    async fn execute_single_strategy(&mut self, strategy: &ActiveStrategy, opportunity: StrategyOpportunity) -> Result<StrategyExecutionResult, StrategyError> {
        match &strategy.config.strategy_type {
            StrategyType::YieldFarming(config) => {
                self.execution_engine.execute_yield_farming_strategy(strategy, opportunity, config).await
            }
            StrategyType::Arbitrage(config) => {
                self.execution_engine.execute_arbitrage_strategy(strategy, opportunity, config).await
            }
            StrategyType::LiquidityMining(config) => {
                self.execution_engine.execute_liquidity_mining_strategy(strategy, opportunity, config).await
            }
            StrategyType::Rebalancing(config) => {
                self.execution_engine.execute_rebalancing_strategy(strategy, opportunity, config).await
            }
            StrategyType::DCA(config) => {
                self.execution_engine.execute_dca_strategy(strategy, opportunity, config).await
            }
            StrategyType::Composite(configs) => {
                self.execution_engine.execute_composite_strategy(strategy, opportunity, configs).await
            }
        }
    }
}

/// Strategy analytics summary
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyAnalytics {
    pub user_id: String,
    pub total_strategies: usize,
    pub active_strategies: usize,
    pub total_allocated_capital: f64,
    pub total_pnl: f64,
    pub weighted_roi: f64,
    pub best_performing_strategy: Option<String>,
    pub worst_performing_strategy: Option<String>,
    pub average_execution_time: f64,
    pub total_gas_spent: f64,
}

/// User preferences for automated strategies
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserPreferences {
    pub default_risk_level: u8,
    pub preferred_chains: Vec<ChainId>,
    pub preferred_protocols: Vec<DeFiProtocol>,
    pub notification_preferences: NotificationPreferences,
    pub auto_rebalance_enabled: bool,
    pub max_total_allocation: f64,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            default_risk_level: 5,
            preferred_chains: vec![ChainId::Ethereum, ChainId::Arbitrum],
            preferred_protocols: vec![DeFiProtocol::Aave, DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3)],
            notification_preferences: NotificationPreferences::default(),
            auto_rebalance_enabled: true,
            max_total_allocation: 100000.0,
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub execution_alerts: bool,
    pub performance_reports: bool,
    pub risk_alerts: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            email_notifications: true,
            execution_alerts: true,
            performance_reports: true,
            risk_alerts: true,
        }
    }
}

/// Strategy error types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum StrategyError {
    StrategyNotFound(String),
    ValidationFailed(String),
    ExecutionFailed(String),
    InsufficientCapital(f64),
    RiskLimitExceeded(String),
    OpportunityNotFound,
    ConfigurationError(String),
    IntegrationError(String),
    InvalidConfiguration(String),
}

impl std::fmt::Display for StrategyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyError::StrategyNotFound(id) => write!(f, "Strategy not found: {}", id),
            StrategyError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            StrategyError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            StrategyError::InsufficientCapital(amount) => write!(f, "Insufficient capital: ${:.2}", amount),
            StrategyError::RiskLimitExceeded(msg) => write!(f, "Risk limit exceeded: {}", msg),
            StrategyError::OpportunityNotFound => write!(f, "No suitable opportunity found"),
            StrategyError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            StrategyError::IntegrationError(msg) => write!(f, "Integration error: {}", msg),
            StrategyError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for StrategyError {}

// Strategy template for the registry
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub base_config: StrategyConfig,
    pub risk_score: u8,
    pub min_capital_usd: f64,
    pub max_capital_usd: f64,
    pub estimated_apy_range: (f64, f64),
    pub supported_chains: Vec<ChainId>,
    pub tags: Vec<String>,
    pub customization_options: Vec<String>,
}