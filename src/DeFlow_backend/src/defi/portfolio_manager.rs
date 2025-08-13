// Advanced Portfolio Management System
// Day 11: Multi-chain portfolio tracking, rebalancing, and risk management

use super::yield_farming::{ChainId, YieldStrategy, DeFiProtocol};
use super::arbitrage::{ArbitrageOpportunity, ArbitrageExecutionResult};
use super::types::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::api::time;

/// Comprehensive portfolio manager for multi-chain DeFi positions
#[derive(Debug, Clone)]
pub struct AdvancedPortfolioManager {
    pub portfolios: HashMap<String, UserPortfolio>,
    pub rebalancing_engine: RebalancingEngine,
    pub risk_manager: PortfolioRiskManager,
    pub analytics_engine: PortfolioAnalyticsEngine,
    pub notification_system: NotificationSystem,
    pub auto_compound_settings: HashMap<String, AutoCompoundSettings>,
    pub last_update: u64,
}

impl AdvancedPortfolioManager {
    pub fn new() -> Self {
        Self {
            portfolios: HashMap::new(),
            rebalancing_engine: RebalancingEngine::new(),
            risk_manager: PortfolioRiskManager::new(),
            analytics_engine: PortfolioAnalyticsEngine::new(),
            notification_system: NotificationSystem::new(),
            auto_compound_settings: HashMap::new(),
            last_update: 0,
        }
    }

    /// Initialize with current time (for canister use)
    pub fn initialize(&mut self) {
        #[cfg(test)]
        {
            self.last_update = 1234567890_u64;
        }
        #[cfg(not(test))]
        {
            self.last_update = time();
        }
        self.rebalancing_engine.initialize();
        self.risk_manager.initialize();
        self.analytics_engine.initialize();
    }

    /// Create or update user portfolio
    pub fn create_portfolio(&mut self, user_id: String, config: PortfolioConfiguration) -> Result<(), PortfolioError> {
        let portfolio = UserPortfolio::new(user_id.clone(), config);
        self.portfolios.insert(user_id, portfolio);
        Ok(())
    }

    /// Add position to user portfolio
    pub fn add_position(&mut self, user_id: &str, position: Position) -> Result<(), PortfolioError> {
        let current_time = self.get_current_time();
        
        let portfolio = self.portfolios.get_mut(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        portfolio.add_position(position);
        portfolio.last_updated = current_time;
        
        self.update_portfolio_metrics(user_id)?;
        
        Ok(())
    }

    /// Remove position from user portfolio
    pub fn remove_position(&mut self, user_id: &str, position_id: &str) -> Result<Position, PortfolioError> {
        let current_time = self.get_current_time();
        
        let portfolio = self.portfolios.get_mut(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        let position = portfolio.remove_position(position_id)?;
        portfolio.last_updated = current_time;
        
        self.update_portfolio_metrics(user_id)?;
        
        Ok(position)
    }

    /// Update position value and performance
    pub fn update_position(&mut self, user_id: &str, position_id: &str, update: PositionUpdate) -> Result<(), PortfolioError> {
        let current_time = self.get_current_time();
        
        let portfolio = self.portfolios.get_mut(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        portfolio.update_position(position_id, update)?;
        portfolio.last_updated = current_time;
        
        self.update_portfolio_metrics(user_id)?;
        
        Ok(())
    }

    /// Get comprehensive portfolio summary
    pub fn get_portfolio_summary(&self, user_id: &str) -> Result<PortfolioSummary, PortfolioError> {
        let portfolio = self.portfolios.get(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        let analytics = self.analytics_engine.generate_portfolio_analytics(portfolio)?;
        let risk_metrics = self.risk_manager.calculate_portfolio_risk(portfolio)?;
        
        Ok(PortfolioSummary {
            user_id: user_id.to_string(),
            total_value_usd: portfolio.calculate_total_value(),
            positions: portfolio.positions.clone(),
            performance_metrics: analytics.performance_metrics,
            risk_metrics,
            allocation_breakdown: analytics.allocation_breakdown,
            chain_distribution: analytics.chain_distribution,
            protocol_distribution: analytics.protocol_distribution,
            yield_summary: analytics.yield_summary,
            last_updated: portfolio.last_updated,
        })
    }

    /// Check if portfolio needs rebalancing
    pub fn check_rebalancing_needs(&self, user_id: &str) -> Result<Vec<RebalancingRecommendation>, PortfolioError> {
        let portfolio = self.portfolios.get(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        self.rebalancing_engine.analyze_rebalancing_needs(portfolio)
    }

    /// Execute portfolio rebalancing
    pub async fn execute_rebalancing(&mut self, user_id: &str, plan: RebalancingPlan) -> Result<RebalancingResult, PortfolioError> {
        let current_time = self.get_current_time();
        
        let result = {
            let portfolio = self.portfolios.get_mut(user_id)
                .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
            
            self.rebalancing_engine.execute_rebalancing(portfolio, plan).await?
        };
        
        // Update portfolio timestamp
        if let Some(portfolio) = self.portfolios.get_mut(user_id) {
            portfolio.last_updated = current_time;
        }
        
        // Update analytics after rebalancing
        self.update_portfolio_metrics(user_id)?;
        
        Ok(result)
    }

    /// Set up automatic compound settings
    pub fn setup_auto_compound(&mut self, user_id: String, settings: AutoCompoundSettings) -> Result<(), PortfolioError> {
        self.auto_compound_settings.insert(user_id, settings);
        Ok(())
    }

    /// Process automatic compounding for eligible positions
    pub async fn process_auto_compounding(&mut self) -> Result<Vec<AutoCompoundResult>, PortfolioError> {
        let mut results = Vec::new();
        let current_time = self.get_current_time();
        
        // Collect user IDs and settings to avoid borrowing issues
        let user_settings: Vec<_> = self.auto_compound_settings.iter()
            .map(|(user_id, settings)| (user_id.clone(), settings.clone()))
            .collect();
        
        for (user_id, settings) in user_settings {
            // Process each portfolio separately to avoid borrowing conflicts
            let portfolio_positions: Vec<_> = self.portfolios.get(&user_id)
                .map(|p| p.positions.clone())
                .unwrap_or_default();
                
            for (idx, position) in portfolio_positions.iter().enumerate() {
                if self.should_auto_compound(position, &settings, current_time) {
                    if let Some(portfolio) = self.portfolios.get_mut(&user_id) {
                        if let Some(pos) = portfolio.positions.get_mut(idx) {
                            let result = Self::execute_auto_compound_static(pos, &settings, current_time).await?;
                            results.push(result);
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }

    /// Update portfolio metrics and analytics
    fn update_portfolio_metrics(&mut self, user_id: &str) -> Result<(), PortfolioError> {
        let portfolio = self.portfolios.get(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        // Update analytics
        let analytics = self.analytics_engine.generate_portfolio_analytics(portfolio)?;
        self.analytics_engine.store_analytics(user_id, analytics);
        
        // Check risk thresholds
        let risk_alerts = self.risk_manager.check_risk_thresholds(portfolio)?;
        if !risk_alerts.is_empty() {
            self.notification_system.send_risk_alerts(user_id, risk_alerts);
        }
        
        Ok(())
    }

    /// Check if position should be auto-compounded
    fn should_auto_compound(&self, position: &Position, settings: &AutoCompoundSettings, current_time: u64) -> bool {
        if !settings.enabled {
            return false;
        }
        
        // Check if enough time has passed since last compound
        let time_since_last = current_time.saturating_sub(position.last_compound_time);
        if time_since_last < settings.frequency_hours * 3600 * 1_000_000_000 {
            return false;
        }
        
        // Check if rewards meet minimum threshold
        if position.pending_rewards_usd < settings.min_rewards_threshold {
            return false;
        }
        
        // Check if position type supports auto-compounding
        matches!(position.position_type, 
            PositionType::YieldFarming { auto_compound: true, .. } | 
            PositionType::LiquidityProvision { auto_compound: true, .. })
    }

    /// Execute auto-compounding for a position
    async fn execute_auto_compound(&self, position: &mut Position, settings: &AutoCompoundSettings) -> Result<AutoCompoundResult, PortfolioError> {
        let current_time = self.get_current_time();
        Self::execute_auto_compound_static(position, settings, current_time).await
    }
    
    /// Static version of auto-compound execution to avoid borrowing issues
    async fn execute_auto_compound_static(position: &mut Position, settings: &AutoCompoundSettings, current_time: u64) -> Result<AutoCompoundResult, PortfolioError> {
        let compound_amount = position.pending_rewards_usd;
        let gas_cost = Self::estimate_compound_gas_cost_static(&position.chain, compound_amount);
        
        // Check if compounding is profitable after gas costs
        if compound_amount <= gas_cost * settings.max_gas_ratio {
            return Ok(AutoCompoundResult {
                position_id: position.id.clone(),
                success: false,
                compound_amount: 0.0,
                gas_cost,
                error_message: Some("Gas cost too high relative to rewards".to_string()),
                executed_at: current_time,
            });
        }
        
        // Execute compound (mock implementation)
        position.value_usd += compound_amount - gas_cost;
        position.pending_rewards_usd = 0.0;
        position.last_compound_time = current_time;
        position.total_compounded_usd += compound_amount;
        
        Ok(AutoCompoundResult {
            position_id: position.id.clone(),
            success: true,
            compound_amount,
            gas_cost,
            error_message: None,
            executed_at: current_time,
        })
    }

    /// Estimate gas cost for compounding
    fn estimate_compound_gas_cost(&self, chain: &ChainId, _amount: f64) -> f64 {
        Self::estimate_compound_gas_cost_static(chain, _amount)
    }
    
    /// Static version of gas cost estimation
    fn estimate_compound_gas_cost_static(chain: &ChainId, _amount: f64) -> f64 {
        match chain {
            ChainId::Ethereum => 25.0,     // High gas
            ChainId::Bitcoin => 15.0,      // Transaction fees
            ChainId::Arbitrum => 3.0,      // L2 efficiency
            ChainId::Optimism => 3.0,      // L2 efficiency
            ChainId::Polygon => 0.5,       // Very low
            ChainId::Base => 2.0,          // Base L2
            ChainId::Avalanche => 2.5,     // Avalanche
            ChainId::BSC => 1.0,           // BSC
            ChainId::Solana => 0.01,       // Extremely low
            _ => 5.0,                      // Default
        }
    }

    /// Get current time for portfolio operations
    fn get_current_time(&self) -> u64 {
        // In test mode, always return a fixed timestamp to avoid canister issues
        #[cfg(test)]
        {
            1234567890_u64
        }
        #[cfg(not(test))]
        {
            if self.last_update == 0 || self.last_update == 1234567890_u64 {
                // Test mode or not initialized
                1234567890_u64
            } else {
                time()
            }
        }
    }

    /// Get portfolio performance over time period
    pub fn get_portfolio_performance(&self, user_id: &str, period_days: u32) -> Result<PortfolioPerformance, PortfolioError> {
        let portfolio = self.portfolios.get(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        self.analytics_engine.calculate_performance(portfolio, period_days)
    }

    /// Get detailed position analytics
    pub fn get_position_analytics(&self, user_id: &str, position_id: &str) -> Result<PositionAnalytics, PortfolioError> {
        let portfolio = self.portfolios.get(user_id)
            .ok_or(PortfolioError::PortfolioNotFound(user_id.to_string()))?;
        
        let position = portfolio.get_position(position_id)?;
        self.analytics_engine.analyze_position(position)
    }
}

/// User portfolio containing all DeFi positions
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserPortfolio {
    pub user_id: String,
    pub positions: Vec<Position>,
    pub configuration: PortfolioConfiguration,
    pub total_value_usd: f64,
    pub last_updated: u64,
    pub created_at: u64,
}

impl UserPortfolio {
    pub fn new(user_id: String, config: PortfolioConfiguration) -> Self {
        let current_time = 1234567890_u64; // Will be set properly when initialized
        Self {
            user_id,
            positions: Vec::new(),
            configuration: config,
            total_value_usd: 0.0,
            last_updated: current_time,
            created_at: current_time,
        }
    }

    pub fn add_position(&mut self, position: Position) {
        self.positions.push(position);
        self.update_total_value();
    }

    pub fn remove_position(&mut self, position_id: &str) -> Result<Position, PortfolioError> {
        let index = self.positions.iter().position(|p| p.id == position_id)
            .ok_or(PortfolioError::PositionNotFound(position_id.to_string()))?;
        
        let position = self.positions.remove(index);
        self.update_total_value();
        Ok(position)
    }

    pub fn update_position(&mut self, position_id: &str, update: PositionUpdate) -> Result<(), PortfolioError> {
        let position = self.positions.iter_mut()
            .find(|p| p.id == position_id)
            .ok_or(PortfolioError::PositionNotFound(position_id.to_string()))?;
        
        if let Some(new_value) = update.value_usd {
            position.value_usd = new_value;
        }
        if let Some(new_rewards) = update.pending_rewards_usd {
            position.pending_rewards_usd = new_rewards;
        }
        if let Some(new_apy) = update.current_apy {
            position.current_apy = new_apy;
        }
        
        self.update_total_value();
        Ok(())
    }

    pub fn get_position(&self, position_id: &str) -> Result<&Position, PortfolioError> {
        self.positions.iter()
            .find(|p| p.id == position_id)
            .ok_or(PortfolioError::PositionNotFound(position_id.to_string()))
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.positions.iter().map(|p| p.value_usd + p.pending_rewards_usd).sum()
    }

    fn update_total_value(&mut self) {
        self.total_value_usd = self.calculate_total_value();
    }

    pub fn get_positions_by_chain(&self, chain: &ChainId) -> Vec<&Position> {
        self.positions.iter().filter(|p| &p.chain == chain).collect()
    }

    pub fn get_positions_by_protocol(&self, protocol: &DeFiProtocol) -> Vec<&Position> {
        self.positions.iter().filter(|p| &p.protocol == protocol).collect()
    }
}

/// Individual DeFi position in a portfolio
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub user_id: String,
    pub chain: ChainId,
    pub protocol: DeFiProtocol,
    pub position_type: PositionType,
    pub value_usd: f64,
    pub initial_investment_usd: f64,
    pub pending_rewards_usd: f64,
    pub current_apy: f64,
    pub risk_score: u8,
    pub created_at: u64,
    pub last_updated: u64,
    pub last_compound_time: u64,
    pub total_compounded_usd: f64,
    pub metadata: PositionMetadata,
}

impl Position {
    pub fn new(
        id: String,
        user_id: String,
        chain: ChainId,
        protocol: DeFiProtocol,
        position_type: PositionType,
        initial_investment_usd: f64,
    ) -> Self {
        let current_time = 1234567890_u64; // Will be set properly when initialized
        Self {
            id,
            user_id,
            chain,
            protocol,
            position_type,
            value_usd: initial_investment_usd,
            initial_investment_usd,
            pending_rewards_usd: 0.0,
            current_apy: 0.0,
            risk_score: 5,
            created_at: current_time,
            last_updated: current_time,
            last_compound_time: current_time,
            total_compounded_usd: 0.0,
            metadata: PositionMetadata::default(),
        }
    }

    pub fn calculate_pnl(&self) -> f64 {
        (self.value_usd + self.pending_rewards_usd) - self.initial_investment_usd
    }

    pub fn calculate_pnl_percentage(&self) -> f64 {
        if self.initial_investment_usd == 0.0 {
            0.0
        } else {
            (self.calculate_pnl() / self.initial_investment_usd) * 100.0
        }
    }

    pub fn days_since_creation(&self, current_time: u64) -> u64 {
        (current_time.saturating_sub(self.created_at)) / (24 * 3600 * 1_000_000_000)
    }
}

/// Types of DeFi positions
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PositionType {
    YieldFarming {
        lp_token: String,
        reward_tokens: Vec<String>,
        auto_compound: bool,
    },
    Lending {
        asset: String,
        variable_rate: bool,
        collateral_factor: f64,
    },
    LiquidityProvision {
        pool_address: String,
        token_a: String,
        token_b: String,
        fee_tier: u32,
        auto_compound: bool,
    },
    Staking {
        validator: String,
        lock_period_days: u32,
        slash_risk: bool,
    },
    Arbitrage {
        strategy_id: String,
        frequency: ArbitrageFrequency,
    },
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum ArbitrageFrequency {
    Manual,
    Automatic,
    Triggered,
}

/// Portfolio configuration and preferences
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioConfiguration {
    pub risk_tolerance: RiskTolerance,
    pub rebalancing_strategy: RebalancingStrategy,
    pub auto_compound_enabled: bool,
    pub notification_preferences: NotificationPreferences,
    pub chain_preferences: Vec<ChainId>,
    pub protocol_preferences: Vec<DeFiProtocol>,
    pub max_positions: u32,
    pub min_position_size_usd: f64,
    pub max_single_position_percentage: f64,
}

impl Default for PortfolioConfiguration {
    fn default() -> Self {
        Self {
            risk_tolerance: RiskTolerance::Moderate,
            rebalancing_strategy: RebalancingStrategy::Threshold(5.0), // 5% threshold
            auto_compound_enabled: true,
            notification_preferences: NotificationPreferences::default(),
            chain_preferences: vec![ChainId::Ethereum, ChainId::Arbitrum, ChainId::Polygon],
            protocol_preferences: vec![DeFiProtocol::Aave, DeFiProtocol::Uniswap(super::yield_farming::UniswapVersion::V3)],
            max_positions: 20,
            min_position_size_usd: 100.0,
            max_single_position_percentage: 25.0,
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskTolerance {
    Conservative,
    Moderate,
    Aggressive,
    Custom {
        max_risk_score: u8,
        max_volatility: f64,
        max_drawdown: f64,
    },
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RebalancingStrategy {
    Threshold(f64),        // Rebalance when allocation drifts by X%
    Periodic(u64),         // Rebalance every X hours
    Manual,                // Manual rebalancing only
    Dynamic {              // Advanced dynamic rebalancing
        volatility_factor: f64,
        momentum_factor: f64,
        risk_adjusted: bool,
    },
}

/// Position metadata for additional information
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PositionMetadata {
    pub tags: Vec<String>,
    pub notes: String,
    pub transaction_hashes: Vec<String>,
    pub external_ids: HashMap<String, String>,
    pub performance_benchmarks: Vec<String>,
}

impl Default for PositionMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            notes: String::new(),
            transaction_hashes: Vec::new(),
            external_ids: HashMap::new(),
            performance_benchmarks: Vec::new(),
        }
    }
}

/// Position update structure
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub value_usd: Option<f64>,
    pub pending_rewards_usd: Option<f64>,
    pub current_apy: Option<f64>,
    pub risk_score: Option<u8>,
    pub metadata: Option<PositionMetadata>,
}

/// Comprehensive portfolio summary
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub user_id: String,
    pub total_value_usd: f64,
    pub positions: Vec<Position>,
    pub performance_metrics: PerformanceMetrics,
    pub risk_metrics: RiskMetrics,
    pub allocation_breakdown: AllocationBreakdown,
    pub chain_distribution: HashMap<ChainId, f64>,
    pub protocol_distribution: HashMap<DeFiProtocol, f64>,
    pub yield_summary: YieldSummary,
    pub last_updated: u64,
}

/// Portfolio errors
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PortfolioError {
    PortfolioNotFound(String),
    PositionNotFound(String),
    InsufficientBalance(f64),
    RiskLimitExceeded(String),
    RebalancingFailed(String),
    InvalidConfiguration(String),
    NotificationError(String),
    AnalyticsError(String),
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortfolioError::PortfolioNotFound(id) => write!(f, "Portfolio not found: {}", id),
            PortfolioError::PositionNotFound(id) => write!(f, "Position not found: {}", id),
            PortfolioError::InsufficientBalance(amount) => write!(f, "Insufficient balance: ${:.2}", amount),
            PortfolioError::RiskLimitExceeded(msg) => write!(f, "Risk limit exceeded: {}", msg),
            PortfolioError::RebalancingFailed(msg) => write!(f, "Rebalancing failed: {}", msg),
            PortfolioError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            PortfolioError::NotificationError(msg) => write!(f, "Notification error: {}", msg),
            PortfolioError::AnalyticsError(msg) => write!(f, "Analytics error: {}", msg),
        }
    }
}

// Include other portfolio management components
mod rebalancing;
mod risk_management;
mod analytics;
mod notifications;

pub use rebalancing::*;
pub use risk_management::*;
pub use analytics::*;
pub use notifications::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defi::yield_farming::DeFiProtocol;

    #[test]
    fn test_portfolio_manager_creation() {
        let mut portfolio_manager = AdvancedPortfolioManager::new();
        
        // Should be able to initialize
        portfolio_manager.initialize();
        
        // Should start with no portfolios
        assert_eq!(portfolio_manager.portfolios.len(), 0);
    }

    #[test]
    fn test_create_portfolio() {
        let mut portfolio_manager = AdvancedPortfolioManager::new();
        portfolio_manager.initialize();
        
        let config = PortfolioConfiguration::default();
        let result = portfolio_manager.create_portfolio("test_user".to_string(), config);
        
        assert!(result.is_ok());
        assert_eq!(portfolio_manager.portfolios.len(), 1);
        assert!(portfolio_manager.portfolios.contains_key("test_user"));
    }

    #[test]
    fn test_add_position() {
        let mut portfolio_manager = AdvancedPortfolioManager::new();
        portfolio_manager.initialize();
        
        // Create portfolio first
        let config = PortfolioConfiguration::default();
        portfolio_manager.create_portfolio("test_user".to_string(), config).unwrap();
        
        // Create a test position
        let position = Position::new(
            "pos_1".to_string(),
            "test_user".to_string(),
            ChainId::Ethereum,
            DeFiProtocol::Aave,
            PositionType::Lending {
                asset: "USDC".to_string(),
                variable_rate: true,
                collateral_factor: 0.8,
            },
            1000.0, // $1000 initial investment
        );
        
        let result = portfolio_manager.add_position("test_user", position);
        assert!(result.is_ok());
        
        // Check that position was added
        let portfolio = portfolio_manager.portfolios.get("test_user").unwrap();
        assert_eq!(portfolio.positions.len(), 1);
        assert_eq!(portfolio.positions[0].id, "pos_1");
        assert_eq!(portfolio.positions[0].value_usd, 1000.0);
    }

    #[test]
    fn test_portfolio_metrics() {
        let mut portfolio_manager = AdvancedPortfolioManager::new();
        portfolio_manager.initialize();
        
        // Create portfolio and add position
        let config = PortfolioConfiguration::default();
        portfolio_manager.create_portfolio("test_user".to_string(), config).unwrap();
        
        let position = Position::new(
            "pos_1".to_string(),
            "test_user".to_string(),
            ChainId::Ethereum,
            DeFiProtocol::Aave,
            PositionType::Lending {
                asset: "USDC".to_string(),
                variable_rate: true,
                collateral_factor: 0.8,
            },
            1000.0,
        );
        
        portfolio_manager.add_position("test_user", position).unwrap();
        
        // Test portfolio summary
        let summary = portfolio_manager.get_portfolio_summary("test_user");
        assert!(summary.is_ok());
        
        let summary = summary.unwrap();
        assert_eq!(summary.user_id, "test_user");
        assert_eq!(summary.total_value_usd, 1000.0);
        assert_eq!(summary.positions.len(), 1);
    }

    #[test]
    fn test_auto_compound_settings() {
        let mut portfolio_manager = AdvancedPortfolioManager::new();
        portfolio_manager.initialize();
        
        let settings = AutoCompoundSettings::default();
        let result = portfolio_manager.setup_auto_compound("test_user".to_string(), settings);
        
        assert!(result.is_ok());
        assert!(portfolio_manager.auto_compound_settings.contains_key("test_user"));
    }
}