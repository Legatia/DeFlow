// Strategy Performance Tracker - Track and analyze strategy performance
// Comprehensive performance metrics, optimization, and reporting

use super::*;
use crate::defi::yield_farming::ChainId;

/// Comprehensive strategy performance tracking system
#[derive(Debug, Clone)]
pub struct StrategyPerformanceTracker {
    pub performance_history: HashMap<String, Vec<PerformanceRecord>>,
    pub benchmark_data: BenchmarkData,
    pub attribution_analyzer: PerformanceAttributionAnalyzer,
    pub risk_analyzer: RiskAnalyzer,
    pub optimization_engine: OptimizationEngine,
    pub reporting_engine: ReportingEngine,
    pub last_update: u64,
}

impl StrategyPerformanceTracker {
    pub fn new() -> Self {
        Self {
            performance_history: HashMap::new(),
            benchmark_data: BenchmarkData::new(),
            attribution_analyzer: PerformanceAttributionAnalyzer::new(),
            risk_analyzer: RiskAnalyzer::new(),
            optimization_engine: OptimizationEngine::new(),
            reporting_engine: ReportingEngine::new(),
            last_update: 0,
        }
    }

    /// Update strategy performance after execution
    pub fn update_strategy_performance(&mut self, strategy: &mut ActiveStrategy, execution_result: &StrategyExecutionResult) -> Result<(), StrategyError> {
        let current_time = self.get_current_time();
        
        // Create performance record
        let performance_record = PerformanceRecord {
            timestamp: execution_result.executed_at,
            execution_id: execution_result.execution_id.clone(),
            amount_invested: execution_result.amount_usd,
            expected_return: execution_result.expected_return,
            actual_return: execution_result.actual_return,
            gas_cost: execution_result.gas_cost_usd,
            execution_time_seconds: execution_result.execution_time_seconds,
            success: execution_result.success,
            pnl: execution_result.actual_return - execution_result.gas_cost_usd,
            roi_percentage: if execution_result.amount_usd > 0.0 {
                ((execution_result.actual_return - execution_result.gas_cost_usd) / execution_result.amount_usd) * 100.0
            } else {
                0.0
            },
            market_conditions: self.capture_market_conditions(),
        };
        
        // Add to performance history
        self.performance_history.entry(strategy.id.clone())
            .or_insert_with(Vec::new)
            .push(performance_record.clone());
        
        // Update strategy metrics
        self.update_strategy_metrics(strategy, &performance_record)?;
        
        // Update risk metrics
        self.risk_analyzer.update_risk_metrics(strategy, &performance_record)?;
        
        // Check for optimization opportunities
        let optimization_suggestions = self.optimization_engine.analyze_strategy_performance(strategy, &self.get_strategy_history(&strategy.id))?;
        
        if !optimization_suggestions.is_empty() {
            // Store optimization suggestions for later review
            self.store_optimization_suggestions(&strategy.id, optimization_suggestions);
        }
        
        self.last_update = current_time;
        Ok(())
    }

    /// Generate comprehensive performance summary
    pub fn generate_performance_summary(&self, strategy: &ActiveStrategy) -> Result<StrategyPerformanceSummary, StrategyError> {
        let history = self.get_strategy_history(&strategy.id);
        
        if history.is_empty() {
            return Ok(StrategyPerformanceSummary {
                strategy_id: strategy.id.clone(),
                name: strategy.config.name.clone(),
                performance_metrics: strategy.performance_metrics.clone(),
                risk_metrics: strategy.risk_metrics.clone(),
                recent_executions: strategy.execution_history.iter().rev().take(10).cloned().collect(),
                next_execution: strategy.next_execution,
                recommendations: vec!["Strategy has not been executed yet".to_string()],
            });
        }

        // Calculate advanced performance metrics
        let advanced_metrics = self.calculate_advanced_metrics(&history)?;
        let attribution_analysis = self.attribution_analyzer.analyze_performance_attribution(&history)?;
        let benchmark_comparison = self.benchmark_data.compare_to_benchmarks(&history)?;
        
        // Generate recommendations
        let recommendations = self.generate_performance_recommendations(strategy, &history, &advanced_metrics)?;
        
        Ok(StrategyPerformanceSummary {
            strategy_id: strategy.id.clone(),
            name: strategy.config.name.clone(),
            performance_metrics: StrategyPerformanceMetrics {
                total_executions: history.len() as u32,
                successful_executions: history.iter().filter(|r| r.success).count() as u32,
                total_pnl: history.iter().map(|r| r.pnl).sum(),
                roi_percentage: advanced_metrics.cumulative_roi,
                sharpe_ratio: advanced_metrics.sharpe_ratio,
                max_drawdown_percentage: advanced_metrics.max_drawdown,
                avg_execution_time_seconds: history.iter().map(|r| r.execution_time_seconds).sum::<u64>() as f64 / history.len() as f64,
                total_gas_spent_usd: history.iter().map(|r| r.gas_cost).sum(),
                win_rate_percentage: (history.iter().filter(|r| r.pnl > 0.0).count() as f64 / history.len() as f64) * 100.0,
            },
            risk_metrics: strategy.risk_metrics.clone(),
            recent_executions: strategy.execution_history.iter().rev().take(10).cloned().collect(),
            next_execution: strategy.next_execution,
            recommendations,
        })
    }

    /// Get strategy performance over time period
    pub fn get_performance_over_period(&self, strategy_id: &str, period_days: u32) -> Result<PerformanceOverTime, StrategyError> {
        let current_time = self.get_current_time();
        let period_nanos = period_days as u64 * 24 * 3600 * 1_000_000_000;
        let cutoff_time = current_time - period_nanos;
        
        let history = self.get_strategy_history(strategy_id);
        let period_history: Vec<&PerformanceRecord> = history.iter()
            .filter(|record| record.timestamp >= cutoff_time)
            .collect();
        
        if period_history.is_empty() {
            return Ok(PerformanceOverTime {
                strategy_id: strategy_id.to_string(),
                period_days,
                total_executions: 0,
                total_pnl: 0.0,
                total_volume: 0.0,
                average_roi: 0.0,
                win_rate: 0.0,
                best_execution: None,
                worst_execution: None,
                daily_returns: Vec::new(),
                cumulative_returns: Vec::new(),
            });
        }

        let total_pnl: f64 = period_history.iter().map(|r| r.pnl).sum();
        let total_volume: f64 = period_history.iter().map(|r| r.amount_invested).sum();
        let average_roi = if total_volume > 0.0 { (total_pnl / total_volume) * 100.0 } else { 0.0 };
        let win_rate = (period_history.iter().filter(|r| r.pnl > 0.0).count() as f64 / period_history.len() as f64) * 100.0;
        
        let best_execution = period_history.iter().max_by(|a, b| a.pnl.partial_cmp(&b.pnl).unwrap()).map(|r| r.execution_id.clone());
        let worst_execution = period_history.iter().min_by(|a, b| a.pnl.partial_cmp(&b.pnl).unwrap()).map(|r| r.execution_id.clone());
        
        // Calculate daily returns
        let daily_returns = self.calculate_daily_returns(&period_history);
        let cumulative_returns = self.calculate_cumulative_returns(&daily_returns);
        
        Ok(PerformanceOverTime {
            strategy_id: strategy_id.to_string(),
            period_days,
            total_executions: period_history.len(),
            total_pnl,
            total_volume,
            average_roi,
            win_rate,
            best_execution,
            worst_execution,
            daily_returns,
            cumulative_returns,
        })
    }

    /// Compare strategies performance
    pub fn compare_strategies(&self, strategy_ids: Vec<String>, period_days: u32) -> Result<StrategyComparison, StrategyError> {
        let mut strategy_performances = HashMap::new();
        
        for strategy_id in &strategy_ids {
            let performance = self.get_performance_over_period(strategy_id, period_days)?;
            strategy_performances.insert(strategy_id.clone(), performance);
        }
        
        // Find best and worst performers
        let best_performer = strategy_performances.iter()
            .max_by(|a, b| a.1.average_roi.partial_cmp(&b.1.average_roi).unwrap())
            .map(|(id, _)| id.clone());
            
        let worst_performer = strategy_performances.iter()
            .min_by(|a, b| a.1.average_roi.partial_cmp(&b.1.average_roi).unwrap())
            .map(|(id, _)| id.clone());
        
        // Calculate correlation matrix
        let correlation_matrix = self.calculate_strategy_correlations(&strategy_ids, period_days)?;
        
        Ok(StrategyComparison {
            strategy_performances,
            best_performer,
            worst_performer,
            correlation_matrix,
            period_days,
            comparison_date: self.get_current_time(),
        })
    }

    /// Get optimization suggestions for strategy
    pub fn get_optimization_suggestions(&self, strategy_id: &str) -> Vec<OptimizationSuggestion> {
        self.optimization_engine.get_stored_suggestions(strategy_id)
    }

    /// Generate performance report
    pub fn generate_performance_report(&self, strategy_id: &str, report_type: ReportType) -> Result<PerformanceReport, StrategyError> {
        self.reporting_engine.generate_report(strategy_id, report_type, self)
    }

    // Private helper methods
    fn update_strategy_metrics(&self, strategy: &mut ActiveStrategy, record: &PerformanceRecord) -> Result<(), StrategyError> {
        let metrics = &mut strategy.performance_metrics;
        
        // Update basic counters
        metrics.total_executions += 1;
        if record.success {
            metrics.successful_executions += 1;
        }
        
        // Update financial metrics
        metrics.total_pnl += record.pnl;
        metrics.total_gas_spent_usd += record.gas_cost;
        
        // Update ROI
        if strategy.allocated_capital > 0.0 {
            metrics.roi_percentage = (metrics.total_pnl / strategy.allocated_capital) * 100.0;
        }
        
        // Update win rate
        let history = self.get_strategy_history(&strategy.id);
        if !history.is_empty() {
            metrics.win_rate_percentage = (history.iter().filter(|r| r.pnl > 0.0).count() as f64 / history.len() as f64) * 100.0;
        }
        
        // Update execution time
        metrics.avg_execution_time_seconds = if metrics.total_executions > 0 {
            (metrics.avg_execution_time_seconds * (metrics.total_executions - 1) as f64 + record.execution_time_seconds as f64) / metrics.total_executions as f64
        } else {
            record.execution_time_seconds as f64
        };
        
        Ok(())
    }

    fn get_strategy_history(&self, strategy_id: &str) -> Vec<PerformanceRecord> {
        self.performance_history.get(strategy_id).cloned().unwrap_or_default()
    }

    fn calculate_advanced_metrics(&self, history: &[PerformanceRecord]) -> Result<AdvancedPerformanceMetrics, StrategyError> {
        if history.is_empty() {
            return Ok(AdvancedPerformanceMetrics::default());
        }

        let returns: Vec<f64> = history.iter().map(|r| r.roi_percentage).collect();
        
        // Calculate cumulative ROI
        let cumulative_roi = returns.iter().fold(0.0, |acc, &r| acc + r);
        
        // Calculate Sharpe ratio
        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let return_variance = returns.iter().map(|r| (r - avg_return).powi(2)).sum::<f64>() / returns.len() as f64;
        let return_std_dev = return_variance.sqrt();
        let sharpe_ratio = if return_std_dev > 0.0 { avg_return / return_std_dev } else { 0.0 };
        
        // Calculate maximum drawdown
        let mut running_max = returns[0];
        let mut max_drawdown = 0.0;
        for &return_val in &returns[1..] {
            running_max = running_max.max(return_val);
            let drawdown = (running_max - return_val) / running_max * 100.0;
            max_drawdown = (max_drawdown as f64).max(drawdown);
        }
        
        // Calculate Sortino ratio (downside deviation)
        let downside_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).cloned().collect();
        let downside_variance = if !downside_returns.is_empty() {
            downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / downside_returns.len() as f64
        } else {
            0.0
        };
        let downside_deviation = downside_variance.sqrt();
        let sortino_ratio = if downside_deviation > 0.0 { avg_return / downside_deviation } else { 0.0 };
        
        // Calculate Calmar ratio
        let calmar_ratio = if max_drawdown > 0.0 { avg_return / max_drawdown } else { 0.0 };
        
        Ok(AdvancedPerformanceMetrics {
            cumulative_roi,
            sharpe_ratio,
            sortino_ratio,
            calmar_ratio,
            max_drawdown,
            volatility: return_std_dev,
            avg_return,
            total_trades: history.len(),
        })
    }

    fn generate_performance_recommendations(&self, strategy: &ActiveStrategy, history: &[PerformanceRecord], metrics: &AdvancedPerformanceMetrics) -> Result<Vec<String>, StrategyError> {
        let mut recommendations = Vec::new();
        
        // ROI-based recommendations
        if metrics.cumulative_roi < 0.0 {
            recommendations.push("Consider reducing position size or pausing strategy due to negative returns".to_string());
        } else if metrics.cumulative_roi > 50.0 {
            recommendations.push("Strong performance - consider increasing allocation".to_string());
        }
        
        // Risk-based recommendations
        if metrics.max_drawdown > 20.0 {
            recommendations.push("High drawdown detected - consider implementing tighter stop-loss".to_string());
        }
        
        if metrics.volatility > 15.0 {
            recommendations.push("High volatility - consider reducing position size or frequency".to_string());
        }
        
        // Sharpe ratio recommendations
        if metrics.sharpe_ratio < 0.5 {
            recommendations.push("Low risk-adjusted returns - review strategy parameters".to_string());
        } else if metrics.sharpe_ratio > 2.0 {
            recommendations.push("Excellent risk-adjusted returns - strategy is performing well".to_string());
        }
        
        // Execution-based recommendations
        if strategy.performance_metrics.win_rate_percentage < 40.0 {
            recommendations.push("Low win rate - consider adjusting entry criteria".to_string());
        }
        
        let avg_gas_cost = history.iter().map(|r| r.gas_cost).sum::<f64>() / history.len() as f64;
        if avg_gas_cost > strategy.config.gas_limit_usd * 0.8 {
            recommendations.push("High gas costs - consider optimizing for gas efficiency".to_string());
        }
        
        // Time-based recommendations
        if metrics.avg_return < 0.5 {
            recommendations.push("Low average returns - consider increasing minimum profit thresholds".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Strategy is performing within normal parameters".to_string());
        }
        
        Ok(recommendations)
    }

    fn capture_market_conditions(&self) -> MarketConditions {
        // Mock market conditions capture
        MarketConditions {
            btc_price: 43250.0,
            eth_price: 2245.0,
            vix_equivalent: 25.8,
            total_market_cap: 1_800_000_000_000.0,
            fear_greed_index: 65,
            gas_prices: {
                let mut gas_prices = HashMap::new();
                gas_prices.insert(ChainId::Ethereum, 25.0);
                gas_prices.insert(ChainId::Arbitrum, 0.2);
                gas_prices.insert(ChainId::Polygon, 0.1);
                gas_prices
            },
        }
    }

    fn calculate_daily_returns(&self, records: &[&PerformanceRecord]) -> Vec<DailyReturn> {
        // Group records by day and calculate daily returns
        let mut daily_groups: std::collections::BTreeMap<u64, Vec<&PerformanceRecord>> = std::collections::BTreeMap::new();
        
        for record in records {
            let day = record.timestamp / (24 * 3600 * 1_000_000_000); // Convert to days
            daily_groups.entry(day).or_insert_with(Vec::new).push(record);
        }
        
        daily_groups.into_iter().map(|(day, day_records)| {
            let total_pnl: f64 = day_records.iter().map(|r| r.pnl).sum();
            let total_volume: f64 = day_records.iter().map(|r| r.amount_invested).sum();
            let return_pct = if total_volume > 0.0 { (total_pnl / total_volume) * 100.0 } else { 0.0 };
            
            DailyReturn {
                date: day * 24 * 3600 * 1_000_000_000, // Convert back to timestamp
                return_percentage: return_pct,
                volume: total_volume,
                executions: day_records.len(),
            }
        }).collect()
    }

    fn calculate_cumulative_returns(&self, daily_returns: &[DailyReturn]) -> Vec<CumulativeReturn> {
        let mut cumulative = 0.0;
        daily_returns.iter().map(|daily| {
            cumulative += daily.return_percentage;
            CumulativeReturn {
                date: daily.date,
                cumulative_return_percentage: cumulative,
            }
        }).collect()
    }

    fn calculate_strategy_correlations(&self, strategy_ids: &[String], period_days: u32) -> Result<HashMap<String, HashMap<String, f64>>, StrategyError> {
        let mut correlation_matrix = HashMap::new();
        
        // Get daily returns for each strategy
        let mut strategy_returns = HashMap::new();
        for strategy_id in strategy_ids {
            let performance = self.get_performance_over_period(strategy_id, period_days)?;
            let returns: Vec<f64> = performance.daily_returns.iter().map(|r| r.return_percentage).collect();
            strategy_returns.insert(strategy_id.clone(), returns);
        }
        
        // Calculate pairwise correlations
        for strategy_a in strategy_ids {
            let mut row = HashMap::new();
            for strategy_b in strategy_ids {
                let correlation = if strategy_a == strategy_b {
                    1.0
                } else {
                    self.calculate_correlation(
                        strategy_returns.get(strategy_a).unwrap(),
                        strategy_returns.get(strategy_b).unwrap()
                    )
                };
                row.insert(strategy_b.clone(), correlation);
            }
            correlation_matrix.insert(strategy_a.clone(), row);
        }
        
        Ok(correlation_matrix)
    }

    fn calculate_correlation(&self, returns_a: &[f64], returns_b: &[f64]) -> f64 {
        if returns_a.len() != returns_b.len() || returns_a.is_empty() {
            return 0.0;
        }
        
        let mean_a = returns_a.iter().sum::<f64>() / returns_a.len() as f64;
        let mean_b = returns_b.iter().sum::<f64>() / returns_b.len() as f64;
        
        let numerator: f64 = returns_a.iter().zip(returns_b.iter())
            .map(|(a, b)| (a - mean_a) * (b - mean_b))
            .sum();
        
        let sum_sq_a: f64 = returns_a.iter().map(|a| (a - mean_a).powi(2)).sum();
        let sum_sq_b: f64 = returns_b.iter().map(|b| (b - mean_b).powi(2)).sum();
        
        let denominator = (sum_sq_a * sum_sq_b).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn store_optimization_suggestions(&self, _strategy_id: &str, _suggestions: Vec<OptimizationSuggestion>) {
        // In production, this would store suggestions in persistent storage
        // For now, we just acknowledge the storage request
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

// Supporting structures and analyzers

/// Performance attribution analyzer
#[derive(Debug, Clone)]
pub struct PerformanceAttributionAnalyzer;

impl PerformanceAttributionAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_performance_attribution(&self, _history: &[PerformanceRecord]) -> Result<AttributionAnalysis, StrategyError> {
        // Mock attribution analysis
        Ok(AttributionAnalysis {
            chain_attribution: {
                let mut chain_attr = HashMap::new();
                chain_attr.insert("Ethereum".to_string(), 45.2);
                chain_attr.insert("Arbitrum".to_string(), 32.1);
                chain_attr.insert("Polygon".to_string(), 22.7);
                chain_attr
            },
            protocol_attribution: {
                let mut protocol_attr = HashMap::new();
                protocol_attr.insert("Uniswap".to_string(), 38.5);
                protocol_attr.insert("Aave".to_string(), 28.3);
                protocol_attr.insert("Compound".to_string(), 33.2);
                protocol_attr
            },
            time_attribution: {
                let mut time_attr = HashMap::new();
                time_attr.insert("Morning".to_string(), 35.8);
                time_attr.insert("Afternoon".to_string(), 42.1);
                time_attr.insert("Evening".to_string(), 22.1);
                time_attr
            },
        })
    }
}

/// Risk analyzer for strategy performance
#[derive(Debug, Clone)]
pub struct RiskAnalyzer;

impl RiskAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn update_risk_metrics(&self, strategy: &mut ActiveStrategy, record: &PerformanceRecord) -> Result<(), StrategyError> {
        // Update risk metrics based on performance record
        strategy.risk_metrics.volatility_score = self.calculate_volatility_score(record);
        strategy.risk_metrics.liquidity_risk_score = self.calculate_liquidity_risk(record);
        
        Ok(())
    }

    fn calculate_volatility_score(&self, _record: &PerformanceRecord) -> f64 {
        // Mock volatility calculation
        5.5
    }

    fn calculate_liquidity_risk(&self, _record: &PerformanceRecord) -> f64 {
        // Mock liquidity risk calculation
        3.2
    }
}

/// Optimization engine for strategy improvement suggestions
#[derive(Debug, Clone)]
pub struct OptimizationEngine;

impl OptimizationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_strategy_performance(&self, _strategy: &ActiveStrategy, _history: &[PerformanceRecord]) -> Result<Vec<OptimizationSuggestion>, StrategyError> {
        // Mock optimization suggestions
        Ok(vec![
            OptimizationSuggestion {
                suggestion_type: OptimizationType::ParameterAdjustment,
                description: "Consider increasing minimum APY threshold by 1%".to_string(),
                expected_improvement: 8.5,
                confidence_score: 0.75,
                implementation_complexity: 2,
            },
            OptimizationSuggestion {
                suggestion_type: OptimizationType::RiskManagement,
                description: "Implement dynamic stop-loss based on volatility".to_string(),
                expected_improvement: 12.3,
                confidence_score: 0.65,
                implementation_complexity: 6,
            },
        ])
    }

    pub fn get_stored_suggestions(&self, _strategy_id: &str) -> Vec<OptimizationSuggestion> {
        // Mock stored suggestions retrieval
        vec![]
    }
}

/// Reporting engine for generating performance reports
#[derive(Debug, Clone)]
pub struct ReportingEngine;

impl ReportingEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_report(&self, _strategy_id: &str, _report_type: ReportType, _tracker: &StrategyPerformanceTracker) -> Result<PerformanceReport, StrategyError> {
        // Mock report generation
        Ok(PerformanceReport {
            report_id: format!("report_{:x}", ic_cdk::api::time()),
            strategy_id: _strategy_id.to_string(),
            report_type: _report_type,
            generated_at: ic_cdk::api::time(),
            summary: "Strategy performance summary".to_string(),
            detailed_analysis: "Detailed performance analysis would be here".to_string(),
            recommendations: vec!["Sample recommendation".to_string()],
            charts_data: HashMap::new(),
        })
    }
}

/// Benchmark data for strategy comparison
#[derive(Debug, Clone)]
pub struct BenchmarkData;

impl BenchmarkData {
    pub fn new() -> Self {
        Self
    }

    pub fn compare_to_benchmarks(&self, _history: &[PerformanceRecord]) -> Result<BenchmarkComparison, StrategyError> {
        // Mock benchmark comparison
        Ok(BenchmarkComparison {
            btc_comparison: 8.5,  // Strategy outperformed BTC by 8.5%
            eth_comparison: 12.3, // Strategy outperformed ETH by 12.3%
            sp500_comparison: 25.1, // Strategy outperformed S&P 500 by 25.1%
            defi_index_comparison: 3.2, // Strategy outperformed DeFi index by 3.2%
        })
    }
}

// Data structures

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub timestamp: u64,
    pub execution_id: String,
    pub amount_invested: f64,
    pub expected_return: f64,
    pub actual_return: f64,
    pub gas_cost: f64,
    pub execution_time_seconds: u64,
    pub success: bool,
    pub pnl: f64,
    pub roi_percentage: f64,
    pub market_conditions: MarketConditions,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct MarketConditions {
    pub btc_price: f64,
    pub eth_price: f64,
    pub vix_equivalent: f64,
    pub total_market_cap: f64,
    pub fear_greed_index: u8,
    pub gas_prices: HashMap<ChainId, f64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AdvancedPerformanceMetrics {
    pub cumulative_roi: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub max_drawdown: f64,
    pub volatility: f64,
    pub avg_return: f64,
    pub total_trades: usize,
}

impl Default for AdvancedPerformanceMetrics {
    fn default() -> Self {
        Self {
            cumulative_roi: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            max_drawdown: 0.0,
            volatility: 0.0,
            avg_return: 0.0,
            total_trades: 0,
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PerformanceOverTime {
    pub strategy_id: String,
    pub period_days: u32,
    pub total_executions: usize,
    pub total_pnl: f64,
    pub total_volume: f64,
    pub average_roi: f64,
    pub win_rate: f64,
    pub best_execution: Option<String>,
    pub worst_execution: Option<String>,
    pub daily_returns: Vec<DailyReturn>,
    pub cumulative_returns: Vec<CumulativeReturn>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DailyReturn {
    pub date: u64,
    pub return_percentage: f64,
    pub volume: f64,
    pub executions: usize,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CumulativeReturn {
    pub date: u64,
    pub cumulative_return_percentage: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyComparison {
    pub strategy_performances: HashMap<String, PerformanceOverTime>,
    pub best_performer: Option<String>,
    pub worst_performer: Option<String>,
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
    pub period_days: u32,
    pub comparison_date: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: OptimizationType,
    pub description: String,
    pub expected_improvement: f64, // Percentage improvement expected
    pub confidence_score: f64,     // 0-1 confidence in the suggestion
    pub implementation_complexity: u8, // 1-10 scale
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum OptimizationType {
    ParameterAdjustment,
    RiskManagement,
    ExecutionTiming,
    CapitalAllocation,
    GasOptimization,
    ChainSelection,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AttributionAnalysis {
    pub chain_attribution: HashMap<String, f64>,
    pub protocol_attribution: HashMap<String, f64>,
    pub time_attribution: HashMap<String, f64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub btc_comparison: f64,
    pub eth_comparison: f64,
    pub sp500_comparison: f64,
    pub defi_index_comparison: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum ReportType {
    Daily,
    Weekly,
    Monthly,
    Custom { start_date: u64, end_date: u64 },
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub report_id: String,
    pub strategy_id: String,
    pub report_type: ReportType,
    pub generated_at: u64,
    pub summary: String,
    pub detailed_analysis: String,
    pub recommendations: Vec<String>,
    pub charts_data: HashMap<String, Vec<f64>>,
}