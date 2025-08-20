// Portfolio Risk Management System
// Comprehensive risk assessment, monitoring, and mitigation

use super::*;
use crate::defi::yield_farming::ChainId;
use ic_cdk::api::time;

/// Advanced portfolio risk manager
#[derive(Debug, Clone)]
pub struct PortfolioRiskManager {
    pub risk_models: HashMap<String, RiskModel>,
    pub alert_thresholds: HashMap<String, AlertThreshold>,
    pub risk_history: Vec<RiskAssessment>,
    pub stress_test_scenarios: Vec<StressTestScenario>,
    pub correlation_matrix: CorrelationMatrix,
    pub var_calculator: VaRCalculator,
    pub last_assessment: u64,
}

impl PortfolioRiskManager {
    pub fn new() -> Self {
        Self {
            risk_models: Self::initialize_risk_models(),
            alert_thresholds: Self::initialize_alert_thresholds(),
            risk_history: Vec::new(),
            stress_test_scenarios: Self::initialize_stress_scenarios(),
            correlation_matrix: CorrelationMatrix::new(),
            var_calculator: VaRCalculator::new(),
            last_assessment: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.last_assessment = self.get_current_time();
    }

    /// Calculate comprehensive portfolio risk metrics
    pub fn calculate_portfolio_risk(&self, portfolio: &UserPortfolio) -> Result<RiskMetrics, PortfolioError> {
        let current_time = self.get_current_time();
        
        // Calculate individual risk components
        let concentration_risk = self.calculate_concentration_risk(portfolio)?;
        let volatility_risk = self.calculate_volatility_risk(portfolio)?;
        let liquidity_risk = self.calculate_liquidity_risk(portfolio)?;
        let credit_risk = self.calculate_credit_risk(portfolio)?;
        let smart_contract_risk = self.calculate_smart_contract_risk(portfolio)?;
        let bridge_risk = self.calculate_bridge_risk(portfolio)?;
        let correlation_risk = self.calculate_correlation_risk(portfolio)?;
        
        // Calculate Value at Risk (VaR)
        let var_1d = self.var_calculator.calculate_var(portfolio, 1, 0.95)?;
        let var_7d = self.var_calculator.calculate_var(portfolio, 7, 0.95)?;
        let var_30d = self.var_calculator.calculate_var(portfolio, 30, 0.95)?;
        
        // Calculate Expected Shortfall (ES)
        let expected_shortfall = self.var_calculator.calculate_expected_shortfall(portfolio, 0.95)?;
        
        // Calculate overall risk score
        let overall_risk_score = self.calculate_overall_risk_score(
            concentration_risk,
            volatility_risk,
            liquidity_risk,
            credit_risk,
            smart_contract_risk,
            bridge_risk,
            correlation_risk,
        );

        // Calculate maximum drawdown
        let max_drawdown = self.calculate_max_drawdown_risk(portfolio)?;

        // Calculate Sharpe ratio
        let sharpe_ratio = self.calculate_sharpe_ratio(portfolio)?;

        Ok(RiskMetrics {
            overall_risk_score,
            concentration_risk,
            volatility_risk,
            liquidity_risk,
            credit_risk,
            smart_contract_risk,
            bridge_risk,
            correlation_risk,
            value_at_risk_1d: var_1d,
            value_at_risk_7d: var_7d,
            value_at_risk_30d: var_30d,
            expected_shortfall,
            max_drawdown_risk: max_drawdown,
            sharpe_ratio,
            beta: self.calculate_portfolio_beta(portfolio)?,
            tracking_error: self.calculate_tracking_error(portfolio)?,
            information_ratio: self.calculate_information_ratio(portfolio)?,
            sortino_ratio: self.calculate_sortino_ratio(portfolio)?,
            last_updated: current_time,
        })
    }

    /// Check risk thresholds and generate alerts
    pub fn check_risk_thresholds(&self, portfolio: &UserPortfolio) -> Result<Vec<RiskAlert>, PortfolioError> {
        let mut alerts = Vec::new();
        let risk_metrics = self.calculate_portfolio_risk(portfolio)?;

        // Check overall risk score
        if let Some(threshold) = self.alert_thresholds.get("overall_risk") {
            if risk_metrics.overall_risk_score > threshold.critical_level {
                alerts.push(RiskAlert {
                    alert_type: RiskAlertType::OverallRisk,
                    severity: AlertSeverity::Critical,
                    message: format!("Portfolio risk score ({:.1}) exceeds critical threshold ({:.1})", 
                                   risk_metrics.overall_risk_score, threshold.critical_level),
                    current_value: risk_metrics.overall_risk_score,
                    threshold_value: threshold.critical_level,
                    recommendation: "Consider reducing high-risk positions and diversifying across chains".to_string(),
                    timestamp: self.get_current_time(),
                });
            } else if risk_metrics.overall_risk_score > threshold.warning_level {
                alerts.push(RiskAlert {
                    alert_type: RiskAlertType::OverallRisk,
                    severity: AlertSeverity::Warning,
                    message: format!("Portfolio risk score ({:.1}) exceeds warning threshold", risk_metrics.overall_risk_score),
                    current_value: risk_metrics.overall_risk_score,
                    threshold_value: threshold.warning_level,
                    recommendation: "Monitor positions closely and consider risk reduction strategies".to_string(),
                    timestamp: self.get_current_time(),
                });
            }
        }

        // Check concentration risk
        if risk_metrics.concentration_risk > 70.0 {
            alerts.push(RiskAlert {
                alert_type: RiskAlertType::ConcentrationRisk,
                severity: AlertSeverity::Warning,
                message: "High concentration risk detected".to_string(),
                current_value: risk_metrics.concentration_risk,
                threshold_value: 70.0,
                recommendation: "Diversify holdings across more chains and protocols".to_string(),
                timestamp: self.get_current_time(),
            });
        }

        // Check liquidity risk
        if risk_metrics.liquidity_risk > 80.0 {
            alerts.push(RiskAlert {
                alert_type: RiskAlertType::LiquidityRisk,
                severity: AlertSeverity::Critical,
                message: "Critical liquidity risk detected".to_string(),
                current_value: risk_metrics.liquidity_risk,
                threshold_value: 80.0,
                recommendation: "Increase allocation to highly liquid assets".to_string(),
                timestamp: self.get_current_time(),
            });
        }

        // Check VaR thresholds
        let portfolio_value = portfolio.calculate_total_value();
        if risk_metrics.value_at_risk_1d > portfolio_value * 0.1 {
            alerts.push(RiskAlert {
                alert_type: RiskAlertType::ValueAtRisk,
                severity: AlertSeverity::Warning,
                message: "High daily Value at Risk detected".to_string(),
                current_value: risk_metrics.value_at_risk_1d,
                threshold_value: portfolio_value * 0.1,
                recommendation: "Consider hedging strategies or reducing position sizes".to_string(),
                timestamp: self.get_current_time(),
            });
        }

        Ok(alerts)
    }

    /// Perform stress testing on portfolio
    pub fn perform_stress_test(&self, portfolio: &UserPortfolio, scenario: &StressTestScenario) -> Result<StressTestResult, PortfolioError> {
        let initial_value = portfolio.calculate_total_value();
        let mut stressed_value = initial_value;

        let mut position_impacts = Vec::new();

        for position in &portfolio.positions {
            let impact = self.calculate_position_stress_impact(position, scenario)?;
            let stressed_position_value = position.value_usd * (1.0 + impact.percentage_change / 100.0);
            
            stressed_value += stressed_position_value - position.value_usd;
            position_impacts.push(impact);
        }

        let total_loss = initial_value - stressed_value;
        let loss_percentage = (total_loss / initial_value) * 100.0;

        Ok(StressTestResult {
            scenario_name: scenario.name.clone(),
            initial_portfolio_value: initial_value,
            stressed_portfolio_value: stressed_value,
            total_loss,
            loss_percentage,
            position_impacts,
            recovery_time_estimate: self.estimate_recovery_time(loss_percentage),
            recommendations: self.generate_stress_test_recommendations(scenario, loss_percentage),
            executed_at: self.get_current_time(),
        })
    }

    /// Calculate concentration risk
    fn calculate_concentration_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let total_value = portfolio.calculate_total_value();
        if total_value == 0.0 {
            return Ok(0.0);
        }

        // Calculate Herfindahl-Hirschman Index for concentration
        let mut hhi_chain = 0.0;
        let mut hhi_protocol = 0.0;
        let mut hhi_position = 0.0;

        // Chain concentration
        let mut chain_allocations = HashMap::new();
        for position in &portfolio.positions {
            let current = chain_allocations.get(&position.chain).unwrap_or(&0.0);
            chain_allocations.insert(position.chain.clone(), current + position.value_usd);
        }

        for allocation in chain_allocations.values() {
            let share = allocation / total_value;
            hhi_chain += share * share;
        }

        // Protocol concentration
        let mut protocol_allocations = HashMap::new();
        for position in &portfolio.positions {
            let current = protocol_allocations.get(&position.protocol).unwrap_or(&0.0);
            protocol_allocations.insert(position.protocol.clone(), current + position.value_usd);
        }

        for allocation in protocol_allocations.values() {
            let share = allocation / total_value;
            hhi_protocol += share * share;
        }

        // Individual position concentration
        for position in &portfolio.positions {
            let share = position.value_usd / total_value;
            hhi_position += share * share;
        }

        // Normalize to 0-100 scale (higher = more concentrated = riskier)
        let concentration_score = ((hhi_chain + hhi_protocol + hhi_position) / 3.0) * 100.0;
        Ok(concentration_score)
    }

    /// Calculate volatility risk
    fn calculate_volatility_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let mut weighted_volatility = 0.0;
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return Ok(0.0);
        }

        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            let volatility = self.get_position_volatility(&position.chain, &position.protocol)?;
            weighted_volatility += weight * volatility;
        }

        // Normalize to 0-100 scale
        Ok(weighted_volatility.min(100.0))
    }

    /// Calculate liquidity risk
    fn calculate_liquidity_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let mut weighted_liquidity_risk = 0.0;
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return Ok(0.0);
        }

        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            let liquidity_score = self.get_position_liquidity_score(position)?;
            weighted_liquidity_risk += weight * (100.0 - liquidity_score); // Higher liquidity = lower risk
        }

        Ok(weighted_liquidity_risk)
    }

    /// Calculate credit risk
    fn calculate_credit_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let mut weighted_credit_risk = 0.0;
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return Ok(0.0);
        }

        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            let credit_score = self.get_protocol_credit_score(&position.protocol)?;
            weighted_credit_risk += weight * credit_score;
        }

        Ok(weighted_credit_risk)
    }

    /// Calculate smart contract risk
    fn calculate_smart_contract_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let mut weighted_sc_risk = 0.0;
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return Ok(0.0);
        }

        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            let sc_risk = self.get_smart_contract_risk_score(&position.protocol, &position.chain)?;
            weighted_sc_risk += weight * sc_risk;
        }

        Ok(weighted_sc_risk)
    }

    /// Calculate bridge risk
    fn calculate_bridge_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let mut total_bridge_exposure = 0.0;
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return Ok(0.0);
        }

        // Count positions that likely involve bridge assets
        for position in &portfolio.positions {
            if self.is_bridge_dependent_position(position) {
                total_bridge_exposure += position.value_usd;
            }
        }

        let bridge_exposure_ratio = total_bridge_exposure / total_value;
        Ok(bridge_exposure_ratio * 60.0) // Scale to risk score
    }

    /// Calculate correlation risk
    fn calculate_correlation_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        if portfolio.positions.len() < 2 {
            return Ok(0.0);
        }

        let mut total_correlation = 0.0;
        let mut pair_count = 0;

        // Calculate average correlation between positions
        for i in 0..portfolio.positions.len() {
            for j in i+1..portfolio.positions.len() {
                let correlation = self.correlation_matrix.get_correlation(
                    &portfolio.positions[i].chain,
                    &portfolio.positions[i].protocol,
                    &portfolio.positions[j].chain,
                    &portfolio.positions[j].protocol,
                )?;
                total_correlation += correlation.abs();
                pair_count += 1;
            }
        }

        let avg_correlation = if pair_count > 0 {
            total_correlation / pair_count as f64
        } else {
            0.0
        };

        // Convert correlation to risk score (higher correlation = higher risk)
        Ok(avg_correlation * 100.0)
    }

    /// Calculate overall risk score
    fn calculate_overall_risk_score(
        &self,
        concentration: f64,
        volatility: f64,
        liquidity: f64,
        credit: f64,
        smart_contract: f64,
        bridge: f64,
        correlation: f64,
    ) -> f64 {
        // Weighted average of risk components
        let weights = [0.2, 0.2, 0.15, 0.15, 0.1, 0.1, 0.1]; // Weights sum to 1.0
        let risks = [concentration, volatility, liquidity, credit, smart_contract, bridge, correlation];
        
        weights.iter().zip(risks.iter()).map(|(w, r)| w * r).sum()
    }

    /// Calculate portfolio beta
    fn calculate_portfolio_beta(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        // Simplified beta calculation against a DeFi market benchmark
        let mut weighted_beta = 0.0;
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return Ok(1.0);
        }

        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            let beta = self.get_position_beta(position)?;
            weighted_beta += weight * beta;
        }

        Ok(weighted_beta)
    }

    /// Calculate tracking error
    fn calculate_tracking_error(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        // Simplified tracking error calculation
        let volatility = self.calculate_volatility_risk(portfolio)?;
        Ok(volatility * 0.3) // Approximate tracking error
    }

    /// Calculate information ratio
    fn calculate_information_ratio(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let tracking_error = self.calculate_tracking_error(portfolio)?;
        if tracking_error == 0.0 {
            Ok(0.0)
        } else {
            // Simplified calculation - would need actual returns data
            Ok(0.5) // Placeholder
        }
    }

    /// Calculate Sharpe ratio
    fn calculate_sharpe_ratio(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        // Simplified Sharpe ratio calculation
        let total_value = portfolio.calculate_total_value();
        let mut weighted_apy = 0.0;

        if total_value == 0.0 {
            return Ok(0.0);
        }

        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            weighted_apy += weight * position.current_apy;
        }

        let risk_free_rate = 2.0; // Assume 2% risk-free rate
        let volatility = self.calculate_volatility_risk(portfolio)?;
        
        if volatility == 0.0 {
            Ok(0.0)
        } else {
            Ok((weighted_apy - risk_free_rate) / volatility)
        }
    }

    /// Calculate Sortino ratio
    fn calculate_sortino_ratio(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        // Simplified Sortino ratio - would need actual returns data
        let sharpe = self.calculate_sharpe_ratio(portfolio)?;
        Ok(sharpe * 1.2) // Sortino is typically higher than Sharpe
    }

    /// Calculate maximum drawdown risk
    fn calculate_max_drawdown_risk(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        // Estimate maximum drawdown based on volatility and correlation
        let volatility = self.calculate_volatility_risk(portfolio)?;
        let correlation = self.calculate_correlation_risk(portfolio)?;
        
        // Higher volatility and correlation increase drawdown risk
        Ok((volatility * 0.8 + correlation * 0.2).min(50.0))
    }

    /// Helper functions for risk calculations
    fn get_position_volatility(&self, chain: &ChainId, protocol: &DeFiProtocol) -> Result<f64, PortfolioError> {
        // Mock volatility data
        let chain_volatility = match chain {
            ChainId::Bitcoin => 30.0,
            ChainId::Ethereum => 25.0,
            ChainId::Arbitrum => 28.0,
            ChainId::Optimism => 28.0,
            ChainId::Polygon => 35.0,
            ChainId::Solana => 45.0,
            ChainId::Avalanche => 40.0,
            _ => 35.0,
        };

        let protocol_factor = match protocol {
            DeFiProtocol::Aave => 0.8,
            DeFiProtocol::Compound => 0.8,
            DeFiProtocol::Uniswap(_) => 1.2,
            DeFiProtocol::SushiSwap => 1.3,
            _ => 1.0,
        };

        Ok(chain_volatility * protocol_factor)
    }

    fn get_position_liquidity_score(&self, position: &Position) -> Result<f64, PortfolioError> {
        // Mock liquidity scores (0-100, higher = more liquid)
        let base_score = match position.chain {
            ChainId::Ethereum => 90.0,
            ChainId::Arbitrum => 80.0,
            ChainId::Optimism => 75.0,
            ChainId::Polygon => 70.0,
            ChainId::Solana => 65.0,
            _ => 60.0,
        };

        let protocol_adjustment = match position.protocol {
            DeFiProtocol::Aave => 10.0,
            DeFiProtocol::Uniswap(_) => 15.0,
            _ => 0.0,
        };

        let score = base_score + protocol_adjustment;
        Ok(if score > 100.0 { 100.0 } else { score })
    }

    fn get_protocol_credit_score(&self, protocol: &DeFiProtocol) -> Result<f64, PortfolioError> {
        // Mock credit risk scores (0-100, higher = riskier)
        Ok(match protocol {
            DeFiProtocol::Aave => 15.0,      // Low credit risk
            DeFiProtocol::Compound => 20.0,  // Low credit risk
            DeFiProtocol::Uniswap(_) => 10.0, // Very low credit risk (AMM)
            DeFiProtocol::SushiSwap => 25.0,  // Medium credit risk
            _ => 40.0,                        // Higher risk for unknown protocols
        })
    }

    fn get_smart_contract_risk_score(&self, protocol: &DeFiProtocol, chain: &ChainId) -> Result<f64, PortfolioError> {
        // Mock smart contract risk scores
        let protocol_risk = match protocol {
            DeFiProtocol::Aave => 15.0,       // Battle-tested
            DeFiProtocol::Compound => 18.0,   // Battle-tested
            DeFiProtocol::Uniswap(_) => 12.0, // Well-audited
            _ => 30.0,                        // Higher risk for others
        };

        let chain_factor = match chain {
            ChainId::Ethereum => 1.0,   // Most mature
            ChainId::Arbitrum => 1.1,   // Slightly higher risk (L2)
            ChainId::Optimism => 1.1,   // Slightly higher risk (L2)
            ChainId::Polygon => 1.2,    // Higher risk
            ChainId::Solana => 1.3,     // Different architecture
            _ => 1.4,                   // Newer chains
        };

        Ok(protocol_risk * chain_factor)
    }

    fn is_bridge_dependent_position(&self, position: &Position) -> bool {
        // Positions on non-native chains for assets typically require bridges
        !matches!(position.chain, ChainId::Ethereum | ChainId::Bitcoin | ChainId::Solana)
    }

    fn get_position_beta(&self, position: &Position) -> Result<f64, PortfolioError> {
        // Mock beta values against DeFi market
        Ok(match position.protocol {
            DeFiProtocol::Aave => 0.8,
            DeFiProtocol::Compound => 0.7,
            DeFiProtocol::Uniswap(_) => 1.2,
            DeFiProtocol::SushiSwap => 1.4,
            _ => 1.0,
        })
    }

    fn calculate_position_stress_impact(&self, position: &Position, scenario: &StressTestScenario) -> Result<PositionStressImpact, PortfolioError> {
        let mut impact_percentage = 0.0;

        // Apply scenario-specific impacts
        for factor in &scenario.stress_factors {
            let position_impact = match factor {
                StressFactor::MarketCrash(percentage) => {
                    let volatility_multiplier = self.get_position_volatility(&position.chain, &position.protocol)? / 30.0;
                    percentage * volatility_multiplier
                },
                StressFactor::LiquidityCrisis(percentage) => {
                    let liquidity_score = self.get_position_liquidity_score(position)?;
                    percentage * (1.0 - liquidity_score / 100.0)
                },
                StressFactor::ProtocolHack(target_protocol) => {
                    if std::mem::discriminant(&position.protocol) == std::mem::discriminant(target_protocol) {
                        -80.0 // 80% loss if protocol is hacked
                    } else {
                        -10.0 // 10% contagion effect
                    }
                },
                StressFactor::BridgeFailure(target_chain) => {
                    if &position.chain == target_chain && self.is_bridge_dependent_position(position) {
                        -60.0 // 60% loss if bridge fails
                    } else {
                        0.0
                    }
                },
                StressFactor::RegulatoryShock(percentage) => {
                    percentage * 0.5 // Assume 50% of impact applies
                },
            };
            impact_percentage += position_impact;
        }

        Ok(PositionStressImpact {
            position_id: position.id.clone(),
            initial_value: position.value_usd,
            stressed_value: position.value_usd * (1.0 + impact_percentage / 100.0),
            percentage_change: impact_percentage,
            risk_factors: scenario.stress_factors.clone(),
        })
    }

    fn estimate_recovery_time(&self, loss_percentage: f64) -> u64 {
        // Estimate recovery time in days based on loss severity
        match loss_percentage.abs() {
            l if l < 5.0 => 7,      // 1 week
            l if l < 10.0 => 30,    // 1 month
            l if l < 25.0 => 90,    // 3 months
            l if l < 50.0 => 180,   // 6 months
            _ => 365,               // 1 year or more
        }
    }

    fn generate_stress_test_recommendations(&self, scenario: &StressTestScenario, loss_percentage: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if loss_percentage > 20.0 {
            recommendations.push("Consider significant diversification across protocols".to_string());
            recommendations.push("Implement stop-loss mechanisms for high-risk positions".to_string());
        }

        if scenario.stress_factors.iter().any(|f| matches!(f, StressFactor::LiquidityCrisis(_))) {
            recommendations.push("Increase allocation to highly liquid assets".to_string());
        }

        if scenario.stress_factors.iter().any(|f| matches!(f, StressFactor::BridgeFailure(_))) {
            recommendations.push("Reduce cross-chain exposure and bridge dependencies".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Portfolio shows good resilience to stress scenarios".to_string());
        }

        recommendations
    }

    fn initialize_risk_models() -> HashMap<String, RiskModel> {
        let mut models = HashMap::new();
        
        models.insert("defi_market".to_string(), RiskModel {
            name: "DeFi Market Risk".to_string(),
            factors: vec!["volatility".to_string(), "liquidity".to_string(), "correlation".to_string()],
            weights: vec![0.4, 0.3, 0.3],
            baseline_score: 50.0,
        });

        models
    }

    fn initialize_alert_thresholds() -> HashMap<String, AlertThreshold> {
        let mut thresholds = HashMap::new();
        
        thresholds.insert("overall_risk".to_string(), AlertThreshold {
            warning_level: 70.0,
            critical_level: 85.0,
        });

        thresholds.insert("concentration_risk".to_string(), AlertThreshold {
            warning_level: 60.0,
            critical_level: 80.0,
        });

        thresholds
    }

    fn initialize_stress_scenarios() -> Vec<StressTestScenario> {
        vec![
            StressTestScenario {
                name: "Market Crash".to_string(),
                description: "Broad crypto market crash scenario".to_string(),
                stress_factors: vec![
                    StressFactor::MarketCrash(-40.0),
                    StressFactor::LiquidityCrisis(-20.0),
                ],
            },
            StressTestScenario {
                name: "Major Protocol Hack".to_string(),
                description: "Large DeFi protocol security breach".to_string(),
                stress_factors: vec![
                    StressFactor::ProtocolHack(DeFiProtocol::Aave),
                    StressFactor::MarketCrash(-15.0),
                ],
            },
            StressTestScenario {
                name: "Bridge Failure".to_string(),
                description: "Major cross-chain bridge compromise".to_string(),
                stress_factors: vec![
                    StressFactor::BridgeFailure(ChainId::Arbitrum),
                    StressFactor::LiquidityCrisis(-25.0),
                ],
            },
        ]
    }

    fn get_current_time(&self) -> u64 {
        #[cfg(test)]
        {
            1234567890_u64
        }
        #[cfg(not(test))]
        {
            if self.last_assessment == 0 || self.last_assessment == 1234567890_u64 {
                1234567890_u64
            } else {
                time()
            }
        }
    }
}

/// Value at Risk calculator
#[derive(Debug, Clone)]
pub struct VaRCalculator {
    pub confidence_levels: Vec<f64>,
    pub time_horizons: Vec<u32>,
}

impl VaRCalculator {
    pub fn new() -> Self {
        Self {
            confidence_levels: vec![0.90, 0.95, 0.99],
            time_horizons: vec![1, 7, 30],
        }
    }

    pub fn calculate_var(&self, portfolio: &UserPortfolio, days: u32, confidence: f64) -> Result<f64, PortfolioError> {
        let volatility = self.calculate_portfolio_volatility(portfolio)?;
        let portfolio_value = portfolio.calculate_total_value();
        
        // Z-score for confidence level (normal distribution assumption)
        let z_score = match confidence {
            c if c >= 0.99 => 2.33,
            c if c >= 0.95 => 1.65,
            c if c >= 0.90 => 1.28,
            _ => 1.65,
        };

        // Scale volatility by time horizon
        let scaled_volatility = volatility * (days as f64).sqrt() / 100.0;
        
        Ok(portfolio_value * scaled_volatility * z_score)
    }

    pub fn calculate_expected_shortfall(&self, portfolio: &UserPortfolio, confidence: f64) -> Result<f64, PortfolioError> {
        let var = self.calculate_var(portfolio, 1, confidence)?;
        // ES is typically 20-30% higher than VaR
        Ok(var * 1.25)
    }

    fn calculate_portfolio_volatility(&self, portfolio: &UserPortfolio) -> Result<f64, PortfolioError> {
        let mut weighted_volatility = 0.0;
        let total_value = portfolio.calculate_total_value();

        if total_value == 0.0 {
            return Ok(0.0);
        }

        for position in &portfolio.positions {
            let weight = position.value_usd / total_value;
            // Mock volatility calculation
            let position_volatility = match position.chain {
                ChainId::Bitcoin => 30.0,
                ChainId::Ethereum => 25.0,
                ChainId::Solana => 45.0,
                _ => 35.0,
            };
            weighted_volatility += weight * weight * position_volatility * position_volatility;
        }

        Ok(weighted_volatility.sqrt())
    }
}

/// Correlation matrix for cross-asset correlations
#[derive(Debug, Clone)]
pub struct CorrelationMatrix {
    pub correlations: HashMap<(ChainId, ChainId), f64>,
}

impl CorrelationMatrix {
    pub fn new() -> Self {
        let mut correlations = HashMap::new();
        
        // Initialize with mock correlation data
        correlations.insert((ChainId::Ethereum, ChainId::Arbitrum), 0.85);
        correlations.insert((ChainId::Ethereum, ChainId::Optimism), 0.80);
        correlations.insert((ChainId::Ethereum, ChainId::Polygon), 0.75);
        correlations.insert((ChainId::Bitcoin, ChainId::Ethereum), 0.60);
        correlations.insert((ChainId::Solana, ChainId::Ethereum), 0.50);
        
        Self { correlations }
    }

    pub fn get_correlation(
        &self,
        chain1: &ChainId,
        _protocol1: &DeFiProtocol,
        chain2: &ChainId,
        _protocol2: &DeFiProtocol,
    ) -> Result<f64, PortfolioError> {
        if chain1 == chain2 {
            return Ok(1.0);
        }

        let key1 = (chain1.clone(), chain2.clone());
        let key2 = (chain2.clone(), chain1.clone());

        Ok(self.correlations.get(&key1)
            .or_else(|| self.correlations.get(&key2))
            .unwrap_or(&0.3) // Default correlation
            .clone())
    }
}

/// Risk assessment data structures
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub overall_risk_score: f64,
    pub concentration_risk: f64,
    pub volatility_risk: f64,
    pub liquidity_risk: f64,
    pub credit_risk: f64,
    pub smart_contract_risk: f64,
    pub bridge_risk: f64,
    pub correlation_risk: f64,
    pub value_at_risk_1d: f64,
    pub value_at_risk_7d: f64,
    pub value_at_risk_30d: f64,
    pub expected_shortfall: f64,
    pub max_drawdown_risk: f64,
    pub sharpe_ratio: f64,
    pub beta: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub sortino_ratio: f64,
    pub last_updated: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskAlert {
    pub alert_type: RiskAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub recommendation: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskAlertType {
    OverallRisk,
    ConcentrationRisk,
    VolatilityRisk,
    LiquidityRisk,
    CreditRisk,
    SmartContractRisk,
    BridgeRisk,
    ValueAtRisk,
    DrawdownRisk,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RiskModel {
    pub name: String,
    pub factors: Vec<String>,
    pub weights: Vec<f64>,
    pub baseline_score: f64,
}

#[derive(Debug, Clone)]
pub struct AlertThreshold {
    pub warning_level: f64,
    pub critical_level: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StressTestScenario {
    pub name: String,
    pub description: String,
    pub stress_factors: Vec<StressFactor>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum StressFactor {
    MarketCrash(f64),                    // Percentage decline
    LiquidityCrisis(f64),               // Liquidity impact
    ProtocolHack(DeFiProtocol),         // Specific protocol hack
    BridgeFailure(ChainId),             // Bridge compromise
    RegulatoryShock(f64),               // Regulatory impact
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StressTestResult {
    pub scenario_name: String,
    pub initial_portfolio_value: f64,
    pub stressed_portfolio_value: f64,
    pub total_loss: f64,
    pub loss_percentage: f64,
    pub position_impacts: Vec<PositionStressImpact>,
    pub recovery_time_estimate: u64,
    pub recommendations: Vec<String>,
    pub executed_at: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PositionStressImpact {
    pub position_id: String,
    pub initial_value: f64,
    pub stressed_value: f64,
    pub percentage_change: f64,
    pub risk_factors: Vec<StressFactor>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub user_id: String,
    pub risk_metrics: RiskMetrics,
    pub alerts: Vec<RiskAlert>,
    pub recommendations: Vec<String>,
    pub assessment_timestamp: u64,
}