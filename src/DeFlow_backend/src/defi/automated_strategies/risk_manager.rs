// Strategy Risk Manager - Comprehensive risk management and limits
// Risk assessment, limits enforcement, and safety controls

use super::*;
use crate::defi::yield_farming::ChainId;

/// Comprehensive risk management system for automated strategies
#[derive(Debug, Clone)]
pub struct StrategyRiskManager {
    pub global_limits: GlobalRiskLimits,
    pub user_limits: HashMap<String, UserRiskLimits>,
    pub strategy_limits: HashMap<String, StrategyRiskLimits>,
    pub risk_models: RiskModels,
    pub monitoring_system: RiskMonitoringSystem,
    pub emergency_controls: EmergencyControls,
    pub compliance_checker: ComplianceChecker,
    pub last_risk_assessment: u64,
}

impl StrategyRiskManager {
    pub fn new() -> Self {
        Self {
            global_limits: GlobalRiskLimits::default(),
            user_limits: HashMap::new(),
            strategy_limits: HashMap::new(),
            risk_models: RiskModels::new(),
            monitoring_system: RiskMonitoringSystem::new(),
            emergency_controls: EmergencyControls::new(),
            compliance_checker: ComplianceChecker::new(),
            last_risk_assessment: 0,
        }
    }

    /// Initialize default risk limits and models
    pub fn initialize_default_limits(&mut self) {
        self.global_limits = GlobalRiskLimits::conservative_defaults();
        self.risk_models.initialize_models();
        self.monitoring_system.initialize();
        self.last_risk_assessment = self.get_current_time();
    }

    /// Validate capital allocation for strategy activation
    pub fn validate_capital_allocation(&mut self, strategy: &ActiveStrategy, capital_amount: f64) -> Result<(), StrategyError> {
        // Check global limits
        if capital_amount > self.global_limits.max_single_strategy_allocation {
            return Err(StrategyError::RiskLimitExceeded(format!(
                "Capital allocation ${:.2} exceeds global limit of ${:.2}",
                capital_amount, self.global_limits.max_single_strategy_allocation
            )));
        }

        // Check user limits
        if let Some(user_limits) = self.user_limits.get(&strategy.user_id) {
            let current_user_allocation = self.calculate_user_total_allocation(&strategy.user_id);
            if current_user_allocation + capital_amount > user_limits.max_total_allocation {
                return Err(StrategyError::RiskLimitExceeded(format!(
                    "User total allocation would exceed limit: ${:.2}",
                    user_limits.max_total_allocation
                )));
            }

            if capital_amount > user_limits.max_single_strategy_allocation {
                return Err(StrategyError::RiskLimitExceeded(format!(
                    "Single strategy allocation exceeds user limit: ${:.2}",
                    user_limits.max_single_strategy_allocation
                )));
            }
        }

        // Check strategy-specific limits
        let strategy_risk_score = self.calculate_strategy_risk_score(strategy)?;
        if strategy_risk_score > self.global_limits.max_strategy_risk_score {
            return Err(StrategyError::RiskLimitExceeded(format!(
                "Strategy risk score {} exceeds maximum allowed {}",
                strategy_risk_score, self.global_limits.max_strategy_risk_score
            )));
        }

        // Validate against risk models
        self.risk_models.validate_allocation(strategy, capital_amount)?;

        Ok(())
    }

    /// Pre-execution risk check
    pub fn pre_execution_check(&mut self, strategy: &ActiveStrategy) -> Result<(), StrategyError> {
        let current_time = self.get_current_time();

        // Check if strategy is within risk limits
        self.validate_strategy_risk_limits(strategy)?;

        // Check market conditions
        self.validate_market_conditions(strategy)?;

        // Check concentration risk
        self.check_concentration_risk(strategy)?;

        // Check liquidity risk
        self.check_liquidity_risk(strategy)?;

        // Check correlation risk
        self.check_correlation_risk(strategy)?;

        // Update monitoring
        self.monitoring_system.record_pre_execution_check(strategy, true);

        Ok(())
    }

    /// Post-execution risk assessment
    pub fn post_execution_assessment(&mut self, strategy: &ActiveStrategy, result: &StrategyExecutionResult) -> Result<(), StrategyError> {
        // Update risk metrics based on execution result
        self.update_strategy_risk_metrics(strategy, result)?;

        // Check if any risk thresholds were breached
        self.check_post_execution_risk_breaches(strategy, result)?;

        // Update risk models with new data
        self.risk_models.update_with_execution_data(strategy, result);

        // Check for emergency conditions
        if self.emergency_controls.should_trigger_emergency_stop(strategy, result)? {
            self.trigger_emergency_stop(&strategy.id, "Post-execution risk assessment triggered emergency stop".to_string())?;
        }

        Ok(())
    }

    /// Set user-specific risk limits
    pub fn set_user_risk_limits(&mut self, user_id: String, limits: UserRiskLimits) -> Result<(), StrategyError> {
        // Validate limits are within global bounds
        if limits.max_total_allocation > self.global_limits.max_user_total_allocation {
            return Err(StrategyError::RiskLimitExceeded("User limits exceed global maximums".to_string()));
        }

        self.user_limits.insert(user_id, limits);
        Ok(())
    }

    /// Set strategy-specific risk limits
    pub fn set_strategy_risk_limits(&mut self, strategy_id: String, limits: StrategyRiskLimits) -> Result<(), StrategyError> {
        self.strategy_limits.insert(strategy_id, limits);
        Ok(())
    }

    /// Calculate comprehensive risk score for strategy
    pub fn calculate_strategy_risk_score(&self, strategy: &ActiveStrategy) -> Result<u8, StrategyError> {
        let mut risk_score = 0.0;

        // Base risk from strategy configuration
        risk_score += strategy.config.risk_level as f64 * 0.3;

        // Chain risk
        let chain_risk = self.calculate_chain_risk(&strategy.config.target_chains);
        risk_score += chain_risk * 0.2;

        // Protocol risk
        let protocol_risk = self.calculate_protocol_risk(&strategy.config.target_protocols);
        risk_score += protocol_risk * 0.2;

        // Allocation risk
        let allocation_risk = self.calculate_allocation_risk(strategy.allocated_capital);
        risk_score += allocation_risk * 0.15;

        // Performance-based risk adjustment
        let performance_risk = self.calculate_performance_risk(&strategy.performance_metrics);
        risk_score += performance_risk * 0.15;

        // Clamp to 1-10 scale
        Ok(risk_score.clamp(1.0, 10.0) as u8)
    }

    /// Get comprehensive risk assessment for strategy
    pub fn get_strategy_risk_assessment(&self, strategy: &ActiveStrategy) -> Result<StrategyRiskAssessment, StrategyError> {
        let overall_risk_score = self.calculate_strategy_risk_score(strategy)?;
        
        let risk_breakdown = RiskBreakdown {
            market_risk: self.assess_market_risk(strategy)?,
            liquidity_risk: self.assess_liquidity_risk(strategy)?,
            smart_contract_risk: self.assess_smart_contract_risk(strategy)?,
            concentration_risk: self.assess_concentration_risk(strategy)?,
            correlation_risk: self.assess_correlation_risk(strategy)?,
            operational_risk: self.assess_operational_risk(strategy)?,
            bridge_risk: self.assess_bridge_risk(strategy)?,
        };

        let risk_factors = self.identify_risk_factors(strategy)?;
        let mitigation_suggestions = self.generate_mitigation_suggestions(strategy, &risk_breakdown)?;

        Ok(StrategyRiskAssessment {
            strategy_id: strategy.id.clone(),
            overall_risk_score,
            risk_level: self.categorize_risk_level(overall_risk_score),
            risk_breakdown,
            risk_factors,
            mitigation_suggestions,
            assessment_timestamp: self.get_current_time(),
            next_review_due: self.get_current_time() + (24 * 3600 * 1_000_000_000), // 24 hours
        })
    }

    /// Get user's total risk exposure
    pub fn get_user_risk_exposure(&self, user_id: &str, active_strategies: &[&ActiveStrategy]) -> UserRiskExposure {
        let user_strategies: Vec<&ActiveStrategy> = active_strategies.iter()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect();

        let total_allocation = user_strategies.iter().map(|s| s.allocated_capital).sum();
        let weighted_risk_score = if total_allocation > 0.0 {
            user_strategies.iter()
                .map(|s| (s.allocated_capital / total_allocation) * s.config.risk_level as f64)
                .sum::<f64>() as u8
        } else {
            0
        };

        let chain_exposure = self.calculate_chain_exposure(&user_strategies);
        let protocol_exposure = self.calculate_protocol_exposure(&user_strategies);

        UserRiskExposure {
            user_id: user_id.to_string(),
            total_allocation,
            weighted_risk_score,
            chain_exposure,
            protocol_exposure,
            active_strategies: user_strategies.len(),
            max_drawdown_risk: self.calculate_portfolio_max_drawdown_risk(&user_strategies),
            correlation_risk: self.calculate_portfolio_correlation_risk(&user_strategies),
            assessment_time: self.get_current_time(),
        }
    }

    /// Trigger emergency stop for strategy
    pub fn trigger_emergency_stop(&mut self, strategy_id: &str, reason: String) -> Result<(), StrategyError> {
        let reason_clone = reason.clone();
        self.emergency_controls.trigger_stop(strategy_id, reason, self.get_current_time())?;
        
        // Log emergency stop
        ic_cdk::println!("🚨 EMERGENCY STOP triggered for strategy {}: {}", strategy_id, reason_clone);
        
        Ok(())
    }

    /// Check if strategy should be emergency stopped
    pub fn should_emergency_stop(&self, strategy: &ActiveStrategy) -> Result<Option<String>, StrategyError> {
        self.emergency_controls.should_trigger_emergency_stop(strategy, &StrategyExecutionResult {
            execution_id: "risk_check".to_string(),
            strategy_id: strategy.id.clone(),
            user_id: strategy.user_id.clone(),
            opportunity_id: "risk_assessment".to_string(),
            action_type: "risk_check".to_string(),
            amount_usd: 0.0,
            expected_return: 0.0,
            actual_return: 0.0,
            gas_cost_usd: 0.0,
            execution_time_seconds: 0,
            success: true,
            error_message: None,
            transaction_hashes: vec![],
            executed_at: self.get_current_time(),
        }).map(|should_stop| if should_stop { Some("Risk assessment triggered emergency stop".to_string()) } else { None })
    }

    /// Get risk monitoring statistics
    pub fn get_risk_statistics(&self) -> RiskStatistics {
        self.monitoring_system.get_statistics()
    }

    // Private helper methods
    fn validate_strategy_risk_limits(&self, strategy: &ActiveStrategy) -> Result<(), StrategyError> {
        if let Some(limits) = self.strategy_limits.get(&strategy.id) {
            let current_risk_score = self.calculate_strategy_risk_score(strategy)?;
            if current_risk_score > limits.max_risk_score {
                return Err(StrategyError::RiskLimitExceeded(format!(
                    "Strategy risk score {} exceeds limit {}",
                    current_risk_score, limits.max_risk_score
                )));
            }
        }
        Ok(())
    }

    fn validate_market_conditions(&self, _strategy: &ActiveStrategy) -> Result<(), StrategyError> {
        // Mock market conditions validation
        let market_volatility = 25.8; // Mock VIX equivalent
        
        if market_volatility > self.global_limits.max_market_volatility {
            return Err(StrategyError::RiskLimitExceeded(format!(
                "Market volatility {} exceeds maximum {}",
                market_volatility, self.global_limits.max_market_volatility
            )));
        }
        
        Ok(())
    }

    fn check_concentration_risk(&self, strategy: &ActiveStrategy) -> Result<(), StrategyError> {
        let user_total = self.calculate_user_total_allocation(&strategy.user_id);
        if user_total > 0.0 {
            let concentration_percentage = (strategy.allocated_capital / user_total) * 100.0;
            if concentration_percentage > self.global_limits.max_single_strategy_percentage {
                return Err(StrategyError::RiskLimitExceeded(format!(
                    "Strategy concentration {}% exceeds limit {}%",
                    concentration_percentage, self.global_limits.max_single_strategy_percentage
                )));
            }
        }
        Ok(())
    }

    fn check_liquidity_risk(&self, strategy: &ActiveStrategy) -> Result<(), StrategyError> {
        // Mock liquidity risk assessment
        for chain in &strategy.config.target_chains {
            let liquidity_score = match chain {
                ChainId::Ethereum => 9.5,
                ChainId::Arbitrum => 8.2,
                ChainId::Polygon => 7.8,
                ChainId::Solana => 8.5,
                _ => 6.0,
            };
            
            if liquidity_score < self.global_limits.min_liquidity_score {
                return Err(StrategyError::RiskLimitExceeded(format!(
                    "Chain {} liquidity score {} below minimum {}",
                    chain.name(), liquidity_score, self.global_limits.min_liquidity_score
                )));
            }
        }
        Ok(())
    }

    fn check_correlation_risk(&self, _strategy: &ActiveStrategy) -> Result<(), StrategyError> {
        // Mock correlation risk check
        // In production, this would analyze correlations with other active strategies
        Ok(())
    }

    fn update_strategy_risk_metrics(&self, _strategy: &ActiveStrategy, _result: &StrategyExecutionResult) -> Result<(), StrategyError> {
        // Update risk metrics based on execution results
        // This would update volatility, drawdown, and other risk measures
        Ok(())
    }

    fn check_post_execution_risk_breaches(&self, strategy: &ActiveStrategy, result: &StrategyExecutionResult) -> Result<(), StrategyError> {
        // Check if execution caused any risk limit breaches
        if !result.success {
            let failure_rate = self.calculate_recent_failure_rate(strategy);
            if failure_rate > self.global_limits.max_failure_rate {
                return Err(StrategyError::RiskLimitExceeded(format!(
                    "Strategy failure rate {}% exceeds limit {}%",
                    failure_rate * 100.0, self.global_limits.max_failure_rate * 100.0
                )));
            }
        }

        // Check drawdown
        let current_drawdown = self.calculate_current_drawdown(strategy);
        if current_drawdown > self.global_limits.max_drawdown_percentage {
            return Err(StrategyError::RiskLimitExceeded(format!(
                "Strategy drawdown {}% exceeds limit {}%",
                current_drawdown, self.global_limits.max_drawdown_percentage
            )));
        }

        Ok(())
    }

    fn calculate_user_total_allocation(&self, _user_id: &str) -> f64 {
        // Mock calculation - in production would sum all user's active strategies
        50000.0
    }

    fn calculate_chain_risk(&self, chains: &[ChainId]) -> f64 {
        let mut total_risk = 0.0;
        for chain in chains {
            let chain_risk = match chain {
                ChainId::Ethereum => 3.0, // Established, lower risk
                ChainId::Bitcoin => 2.0,   // Most established
                ChainId::Arbitrum => 4.0,  // L2, moderate risk
                ChainId::Polygon => 4.5,   // Sidechain, moderate-high risk
                ChainId::Solana => 5.0,    // Different architecture, higher risk
                _ => 6.0,                  // Unknown/new chains, highest risk
            };
            total_risk += chain_risk;
        }
        (total_risk / chains.len() as f64).min(10.0)
    }

    fn calculate_protocol_risk(&self, protocols: &[crate::defi::yield_farming::DeFiProtocol]) -> f64 {
        let mut total_risk = 0.0;
        for protocol in protocols {
            let protocol_risk = match protocol {
                crate::defi::yield_farming::DeFiProtocol::Aave => 2.5,
                crate::defi::yield_farming::DeFiProtocol::Compound => 2.8,
                crate::defi::yield_farming::DeFiProtocol::Uniswap(_) => 3.2,
                _ => 5.0,
            };
            total_risk += protocol_risk;
        }
        (total_risk / protocols.len() as f64).min(10.0)
    }

    fn calculate_allocation_risk(&self, allocation: f64) -> f64 {
        // Higher allocations have higher risk due to capital concentration
        match allocation {
            a if a < 1000.0 => 1.0,
            a if a < 10000.0 => 2.0,
            a if a < 50000.0 => 3.0,
            a if a < 100000.0 => 5.0,
            _ => 8.0,
        }
    }

    fn calculate_performance_risk(&self, metrics: &StrategyPerformanceMetrics) -> f64 {
        let mut risk = 0.0;
        
        // Win rate risk
        if metrics.win_rate_percentage < 50.0 {
            risk += (50.0 - metrics.win_rate_percentage) / 10.0;
        }
        
        // Drawdown risk
        if metrics.max_drawdown_percentage > 10.0 {
            risk += metrics.max_drawdown_percentage / 5.0;
        }
        
        risk.min(10.0)
    }

    fn assess_market_risk(&self, _strategy: &ActiveStrategy) -> Result<f64, StrategyError> {
        // Mock market risk assessment
        Ok(4.2)
    }

    fn assess_liquidity_risk(&self, _strategy: &ActiveStrategy) -> Result<f64, StrategyError> {
        // Mock liquidity risk assessment
        Ok(3.1)
    }

    fn assess_smart_contract_risk(&self, _strategy: &ActiveStrategy) -> Result<f64, StrategyError> {
        // Mock smart contract risk assessment
        Ok(3.8)
    }

    fn assess_concentration_risk(&self, _strategy: &ActiveStrategy) -> Result<f64, StrategyError> {
        // Mock concentration risk assessment
        Ok(2.5)
    }

    fn assess_correlation_risk(&self, _strategy: &ActiveStrategy) -> Result<f64, StrategyError> {
        // Mock correlation risk assessment
        Ok(4.1)
    }

    fn assess_operational_risk(&self, _strategy: &ActiveStrategy) -> Result<f64, StrategyError> {
        // Mock operational risk assessment
        Ok(2.8)
    }

    fn assess_bridge_risk(&self, _strategy: &ActiveStrategy) -> Result<f64, StrategyError> {
        // Mock bridge risk assessment
        Ok(5.2)
    }

    fn identify_risk_factors(&self, _strategy: &ActiveStrategy) -> Result<Vec<String>, StrategyError> {
        Ok(vec![
            "High market volatility detected".to_string(),
            "Concentrated exposure to single protocol".to_string(),
            "Cross-chain bridge risk present".to_string(),
        ])
    }

    fn generate_mitigation_suggestions(&self, _strategy: &ActiveStrategy, _breakdown: &RiskBreakdown) -> Result<Vec<String>, StrategyError> {
        Ok(vec![
            "Consider diversifying across more protocols".to_string(),
            "Reduce position size during high volatility periods".to_string(),
            "Implement dynamic stop-loss based on VaR calculations".to_string(),
        ])
    }

    fn categorize_risk_level(&self, risk_score: u8) -> RiskLevel {
        match risk_score {
            1..=3 => RiskLevel::Low,
            4..=6 => RiskLevel::Medium,
            7..=8 => RiskLevel::High,
            9..=10 => RiskLevel::Critical,
            _ => RiskLevel::Medium,
        }
    }

    fn calculate_chain_exposure(&self, strategies: &[&ActiveStrategy]) -> HashMap<String, f64> {
        let mut exposure = HashMap::new();
        let total_allocation: f64 = strategies.iter().map(|s| s.allocated_capital).sum();
        
        if total_allocation > 0.0 {
            for strategy in strategies {
                for chain in &strategy.config.target_chains {
                    let chain_allocation = strategy.allocated_capital / strategy.config.target_chains.len() as f64;
                    let percentage = (chain_allocation / total_allocation) * 100.0;
                    *exposure.entry(chain.name().to_string()).or_insert(0.0) += percentage;
                }
            }
        }
        
        exposure
    }

    fn calculate_protocol_exposure(&self, strategies: &[&ActiveStrategy]) -> HashMap<String, f64> {
        let mut exposure = HashMap::new();
        let total_allocation: f64 = strategies.iter().map(|s| s.allocated_capital).sum();
        
        if total_allocation > 0.0 {
            for strategy in strategies {
                for protocol in &strategy.config.target_protocols {
                    let protocol_allocation = strategy.allocated_capital / strategy.config.target_protocols.len() as f64;
                    let percentage = (protocol_allocation / total_allocation) * 100.0;
                    *exposure.entry(format!("{:?}", protocol)).or_insert(0.0) += percentage;
                }
            }
        }
        
        exposure
    }

    fn calculate_portfolio_max_drawdown_risk(&self, strategies: &[&ActiveStrategy]) -> f64 {
        strategies.iter()
            .map(|s| s.performance_metrics.max_drawdown_percentage)
            .fold(0.0, f64::max)
    }

    fn calculate_portfolio_correlation_risk(&self, _strategies: &[&ActiveStrategy]) -> f64 {
        // Mock portfolio correlation risk
        6.5
    }

    fn calculate_recent_failure_rate(&self, _strategy: &ActiveStrategy) -> f64 {
        // Mock failure rate calculation
        0.05 // 5%
    }

    fn calculate_current_drawdown(&self, _strategy: &ActiveStrategy) -> f64 {
        // Mock current drawdown calculation
        8.5
    }

    fn get_current_time(&self) -> u64 {
        #[cfg(test)]
        {
            1234567890_u64
        }
        #[cfg(not(test))]
        {
            ic_cdk::api::time()
        }
    }
}

// Supporting structures

/// Global risk limits for the entire system
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct GlobalRiskLimits {
    pub max_single_strategy_allocation: f64,
    pub max_user_total_allocation: f64,
    pub max_strategy_risk_score: u8,
    pub max_market_volatility: f64,
    pub max_single_strategy_percentage: f64,
    pub min_liquidity_score: f64,
    pub max_failure_rate: f64,
    pub max_drawdown_percentage: f64,
    pub max_gas_cost_percentage: f64,
}

impl Default for GlobalRiskLimits {
    fn default() -> Self {
        Self::conservative_defaults()
    }
}

impl GlobalRiskLimits {
    pub fn conservative_defaults() -> Self {
        Self {
            max_single_strategy_allocation: 100000.0,
            max_user_total_allocation: 500000.0,
            max_strategy_risk_score: 8,
            max_market_volatility: 40.0,
            max_single_strategy_percentage: 25.0,
            min_liquidity_score: 6.0,
            max_failure_rate: 0.15, // 15%
            max_drawdown_percentage: 20.0,
            max_gas_cost_percentage: 5.0,
        }
    }
}

/// User-specific risk limits
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserRiskLimits {
    pub max_total_allocation: f64,
    pub max_single_strategy_allocation: f64,
    pub max_risk_score: u8,
    pub max_strategies: u32,
    pub emergency_stop_drawdown: f64,
}

/// Strategy-specific risk limits
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyRiskLimits {
    pub max_risk_score: u8,
    pub max_allocation: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
    pub max_daily_executions: u32,
}

/// Risk models for different assessments
#[derive(Debug, Clone)]
pub struct RiskModels {
    pub var_model: VaRModel,
    pub stress_test_model: StressTestModel,
    pub correlation_model: CorrelationModel,
}

impl RiskModels {
    pub fn new() -> Self {
        Self {
            var_model: VaRModel::new(),
            stress_test_model: StressTestModel::new(),
            correlation_model: CorrelationModel::new(),
        }
    }

    pub fn initialize_models(&mut self) {
        self.var_model.initialize();
        self.stress_test_model.initialize();
        self.correlation_model.initialize();
    }

    pub fn validate_allocation(&self, _strategy: &ActiveStrategy, _amount: f64) -> Result<(), StrategyError> {
        // Mock validation
        Ok(())
    }

    pub fn update_with_execution_data(&mut self, _strategy: &ActiveStrategy, _result: &StrategyExecutionResult) {
        // Update models with new execution data
    }
}

/// Value at Risk model
#[derive(Debug, Clone)]
pub struct VaRModel;

impl VaRModel {
    pub fn new() -> Self { Self }
    pub fn initialize(&mut self) {}
}

/// Stress testing model
#[derive(Debug, Clone)]
pub struct StressTestModel;

impl StressTestModel {
    pub fn new() -> Self { Self }
    pub fn initialize(&mut self) {}
}

/// Correlation model
#[derive(Debug, Clone)]
pub struct CorrelationModel;

impl CorrelationModel {
    pub fn new() -> Self { Self }
    pub fn initialize(&mut self) {}
}

/// Risk monitoring system
#[derive(Debug, Clone)]
pub struct RiskMonitoringSystem {
    pub alerts_sent: u32,
    pub risk_checks_performed: u32,
    pub emergency_stops_triggered: u32,
}

impl RiskMonitoringSystem {
    pub fn new() -> Self {
        Self {
            alerts_sent: 0,
            risk_checks_performed: 0,
            emergency_stops_triggered: 0,
        }
    }

    pub fn initialize(&mut self) {}

    pub fn record_pre_execution_check(&mut self, _strategy: &ActiveStrategy, _passed: bool) {
        self.risk_checks_performed += 1;
    }

    pub fn get_statistics(&self) -> RiskStatistics {
        RiskStatistics {
            total_risk_checks: self.risk_checks_performed,
            total_alerts_sent: self.alerts_sent,
            total_emergency_stops: self.emergency_stops_triggered,
            risk_check_success_rate: 95.5, // Mock value
            average_risk_score: 4.2,       // Mock value
            last_updated: ic_cdk::api::time(),
        }
    }
}

/// Emergency control system
#[derive(Debug, Clone)]
pub struct EmergencyControls {
    pub emergency_stops: HashMap<String, EmergencyStop>,
    pub global_circuit_breaker: bool,
}

impl EmergencyControls {
    pub fn new() -> Self {
        Self {
            emergency_stops: HashMap::new(),
            global_circuit_breaker: false,
        }
    }

    pub fn should_trigger_emergency_stop(&self, strategy: &ActiveStrategy, result: &StrategyExecutionResult) -> Result<bool, StrategyError> {
        // Check for emergency stop conditions
        if !result.success && result.gas_cost_usd > strategy.config.gas_limit_usd {
            return Ok(true);
        }

        if strategy.performance_metrics.max_drawdown_percentage > 25.0 {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn trigger_stop(&mut self, strategy_id: &str, reason: String, timestamp: u64) -> Result<(), StrategyError> {
        let emergency_stop = EmergencyStop {
            strategy_id: strategy_id.to_string(),
            reason,
            triggered_at: timestamp,
            triggered_by: "risk_manager".to_string(),
        };

        self.emergency_stops.insert(strategy_id.to_string(), emergency_stop);
        Ok(())
    }
}

/// Compliance checker
#[derive(Debug, Clone)]
pub struct ComplianceChecker;

impl ComplianceChecker {
    pub fn new() -> Self { Self }
}

// Data structures

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyRiskAssessment {
    pub strategy_id: String,
    pub overall_risk_score: u8,
    pub risk_level: RiskLevel,
    pub risk_breakdown: RiskBreakdown,
    pub risk_factors: Vec<String>,
    pub mitigation_suggestions: Vec<String>,
    pub assessment_timestamp: u64,
    pub next_review_due: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskBreakdown {
    pub market_risk: f64,
    pub liquidity_risk: f64,
    pub smart_contract_risk: f64,
    pub concentration_risk: f64,
    pub correlation_risk: f64,
    pub operational_risk: f64,
    pub bridge_risk: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserRiskExposure {
    pub user_id: String,
    pub total_allocation: f64,
    pub weighted_risk_score: u8,
    pub chain_exposure: HashMap<String, f64>,
    pub protocol_exposure: HashMap<String, f64>,
    pub active_strategies: usize,
    pub max_drawdown_risk: f64,
    pub correlation_risk: f64,
    pub assessment_time: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskStatistics {
    pub total_risk_checks: u32,
    pub total_alerts_sent: u32,
    pub total_emergency_stops: u32,
    pub risk_check_success_rate: f64,
    pub average_risk_score: f64,
    pub last_updated: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EmergencyStop {
    pub strategy_id: String,
    pub reason: String,
    pub triggered_at: u64,
    pub triggered_by: String,
}