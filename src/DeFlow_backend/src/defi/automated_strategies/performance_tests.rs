// Comprehensive Performance Tracker Test Suite
// Tests performance analytics, metrics calculation, and benchmarking

#[cfg(test)]
mod performance_tests {
    use super::super::*;
    use crate::defi::yield_farming::{ChainId, DeFiProtocol};
    use std::collections::HashMap;

    fn create_execution_result(
        execution_id: &str,
        strategy_id: &str,
        amount: f64,
        expected_return: f64,
        actual_return: f64,
        gas_cost: f64,
        success: bool,
        executed_at: u64,
    ) -> StrategyExecutionResult {
        StrategyExecutionResult {
            execution_id: execution_id.to_string(),
            strategy_id: strategy_id.to_string(),
            user_id: "test_user".to_string(),
            opportunity_id: format!("opp_{}", execution_id),
            action_type: "yield_farming".to_string(),
            amount_usd: amount,
            expected_return,
            actual_return,
            gas_cost_usd: gas_cost,
            execution_time_seconds: 30,
            success,
            error_message: if !success { Some("Execution failed".to_string()) } else { None },
            transaction_hashes: if success { vec![format!("0x{}", execution_id)] } else { vec![] },
            executed_at,
        }
    }

    fn create_diverse_execution_history() -> Vec<StrategyExecutionResult> {
        vec![
            // Successful executions
            create_execution_result("exec_1", "strategy_1", 1000.0, 50.0, 55.0, 25.0, true, 1234567890),
            create_execution_result("exec_2", "strategy_1", 1500.0, 75.0, 70.0, 30.0, true, 1234571490),
            create_execution_result("exec_3", "strategy_1", 2000.0, 100.0, 110.0, 35.0, true, 1234575090),
            create_execution_result("exec_4", "strategy_1", 800.0, 40.0, 45.0, 20.0, true, 1234578690),
            
            // Failed executions
            create_execution_result("exec_5", "strategy_1", 1200.0, 60.0, -30.0, 40.0, false, 1234582290),
            create_execution_result("exec_6", "strategy_1", 900.0, 45.0, -15.0, 35.0, false, 1234585890),
            
            // Mixed performance
            create_execution_result("exec_7", "strategy_1", 1800.0, 90.0, 85.0, 25.0, true, 1234589490),
            create_execution_result("exec_8", "strategy_1", 1300.0, 65.0, 70.0, 30.0, true, 1234593090),
            
            // Different strategy
            create_execution_result("exec_9", "strategy_2", 2500.0, 125.0, 130.0, 40.0, true, 1234596690),
            create_execution_result("exec_10", "strategy_2", 3000.0, 150.0, 140.0, 45.0, true, 1234600290),
        ]
    }

    #[tokio::test]
    async fn test_performance_tracker_initialization() {
        let tracker = StrategyPerformanceTracker::new();
        
        assert_eq!(tracker.execution_history.len(), 0);
        assert_eq!(tracker.performance_snapshots.len(), 0);
        assert_eq!(tracker.attribution_analyzer.factor_weights.len(), 0);
        assert_eq!(tracker.benchmark_data.benchmarks.len(), 0);
    }

    #[tokio::test]
    async fn test_execution_recording_single() {
        let mut tracker = StrategyPerformanceTracker::new();
        let execution = create_execution_result("test_1", "strategy_1", 1000.0, 50.0, 55.0, 25.0, true, 1234567890);
        
        let result = tracker.record_execution(&execution);
        assert!(result.is_ok());
        assert_eq!(tracker.execution_history.len(), 1);
        
        let recorded = &tracker.execution_history[0];
        assert_eq!(recorded.execution_id, "test_1");
        assert_eq!(recorded.strategy_id, "strategy_1");
        assert_eq!(recorded.actual_return, 55.0);
    }

    #[tokio::test]
    async fn test_execution_recording_multiple() {
        let mut tracker = StrategyPerformanceTracker::new();
        let executions = create_diverse_execution_history();
        
        for execution in executions.iter() {
            let result = tracker.record_execution(execution);
            assert!(result.is_ok());
        }
        
        assert_eq!(tracker.execution_history.len(), 10);
        
        // Test filtering by strategy
        let strategy_1_executions: Vec<_> = tracker.execution_history
            .iter()
            .filter(|e| e.strategy_id == "strategy_1")
            .collect();
        assert_eq!(strategy_1_executions.len(), 8);
        
        let strategy_2_executions: Vec<_> = tracker.execution_history
            .iter()
            .filter(|e| e.strategy_id == "strategy_2")
            .collect();
        assert_eq!(strategy_2_executions.len(), 2);
    }

    #[tokio::test]
    async fn test_performance_metrics_calculation_profitable() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Add profitable executions
        let executions = vec![
            create_execution_result("exec_1", "profitable_strategy", 1000.0, 50.0, 55.0, 10.0, true, 1234567890),
            create_execution_result("exec_2", "profitable_strategy", 2000.0, 100.0, 110.0, 15.0, true, 1234571490),
            create_execution_result("exec_3", "profitable_strategy", 1500.0, 75.0, 80.0, 12.0, true, 1234575090),
            create_execution_result("exec_4", "profitable_strategy", 800.0, 40.0, 45.0, 8.0, true, 1234578690),
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let result = tracker.calculate_performance_metrics("profitable_strategy");
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert_eq!(metrics.total_executions, 4);
        assert_eq!(metrics.successful_executions, 4);
        assert_eq!(metrics.win_rate_percentage, 100.0);
        assert!(metrics.total_return_usd > 0.0);
        assert!(metrics.total_return_percentage > 0.0);
        assert!(metrics.sharpe_ratio > 0.0);
        assert!(metrics.max_drawdown_percentage >= 0.0);
    }

    #[tokio::test]
    async fn test_performance_metrics_calculation_mixed() {
        let mut tracker = StrategyPerformanceTracker::new();
        let executions = create_diverse_execution_history();
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let result = tracker.calculate_performance_metrics("strategy_1");
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert_eq!(metrics.total_executions, 8);
        assert_eq!(metrics.successful_executions, 6);
        assert_eq!(metrics.win_rate_percentage, 75.0);
        
        // Check that total return accounts for both profits and losses
        let expected_total = 55.0 + 70.0 + 110.0 + 45.0 - 30.0 - 15.0 + 85.0 + 70.0;
        assert!((metrics.total_return_usd - expected_total).abs() < 0.01);
        
        // Check fees are summed correctly
        let expected_fees = 25.0 + 30.0 + 35.0 + 20.0 + 40.0 + 35.0 + 25.0 + 30.0;
        assert!((metrics.total_fees_paid - expected_fees).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_performance_metrics_losing_strategy() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Add losing executions
        let executions = vec![
            create_execution_result("loss_1", "losing_strategy", 1000.0, 50.0, -30.0, 25.0, false, 1234567890),
            create_execution_result("loss_2", "losing_strategy", 2000.0, 100.0, -80.0, 40.0, false, 1234571490),
            create_execution_result("loss_3", "losing_strategy", 1500.0, 75.0, -45.0, 35.0, false, 1234575090),
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let result = tracker.calculate_performance_metrics("losing_strategy");
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert_eq!(metrics.total_executions, 3);
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.win_rate_percentage, 0.0);
        assert!(metrics.total_return_usd < 0.0);
        assert!(metrics.total_return_percentage < 0.0);
        assert!(metrics.sharpe_ratio < 0.0);
        assert!(metrics.max_drawdown_percentage > 0.0);
    }

    #[tokio::test]
    async fn test_annualized_return_calculation() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Create executions spanning different time periods
        let start_time = 1234567890;
        let one_year_later = start_time + (365 * 24 * 3600);
        
        let executions = vec![
            create_execution_result("year_1", "annual_strategy", 10000.0, 500.0, 520.0, 50.0, true, start_time),
            create_execution_result("year_2", "annual_strategy", 10000.0, 500.0, 480.0, 50.0, true, one_year_later),
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let metrics = tracker.calculate_performance_metrics("annual_strategy").unwrap();
        assert!(metrics.annualized_return_percentage > 0.0);
        
        // For a total return of 1000 over approximately 1 year on 20000 invested,
        // annualized return should be around 5%
        assert!(metrics.annualized_return_percentage > 3.0 && metrics.annualized_return_percentage < 7.0);
    }

    #[tokio::test]
    async fn test_sharpe_ratio_calculation() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Create consistent profitable executions (low volatility, good Sharpe)
        let executions = vec![
            create_execution_result("consistent_1", "consistent_strategy", 1000.0, 50.0, 52.0, 10.0, true, 1234567890),
            create_execution_result("consistent_2", "consistent_strategy", 1000.0, 50.0, 51.0, 10.0, true, 1234571490),
            create_execution_result("consistent_3", "consistent_strategy", 1000.0, 50.0, 53.0, 10.0, true, 1234575090),
            create_execution_result("consistent_4", "consistent_strategy", 1000.0, 50.0, 52.5, 10.0, true, 1234578690),
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let metrics = tracker.calculate_performance_metrics("consistent_strategy").unwrap();
        
        // Consistent profits should yield positive Sharpe ratio
        assert!(metrics.sharpe_ratio > 0.0);
        
        // Add volatile executions
        let volatile_executions = vec![
            create_execution_result("volatile_1", "volatile_strategy", 1000.0, 50.0, 100.0, 10.0, true, 1234567890),
            create_execution_result("volatile_2", "volatile_strategy", 1000.0, 50.0, 10.0, 10.0, true, 1234571490),
            create_execution_result("volatile_3", "volatile_strategy", 1000.0, 50.0, 90.0, 10.0, true, 1234575090),
            create_execution_result("volatile_4", "volatile_strategy", 1000.0, 50.0, 20.0, 10.0, true, 1234578690),
        ];
        
        for execution in volatile_executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let volatile_metrics = tracker.calculate_performance_metrics("volatile_strategy").unwrap();
        
        // Higher volatility should generally result in lower Sharpe ratio
        // (though both strategies are profitable)
        assert!(volatile_metrics.sharpe_ratio >= 0.0);
    }

    #[tokio::test]
    async fn test_max_drawdown_calculation() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Create a sequence with significant drawdown
        let executions = vec![
            create_execution_result("dd_1", "drawdown_strategy", 1000.0, 50.0, 60.0, 10.0, true, 1234567890),  // +60, total: 60
            create_execution_result("dd_2", "drawdown_strategy", 1000.0, 50.0, 40.0, 10.0, true, 1234571490),  // +40, total: 100
            create_execution_result("dd_3", "drawdown_strategy", 1000.0, 50.0, -30.0, 20.0, false, 1234575090), // -30, total: 70 (drawdown from 100)
            create_execution_result("dd_4", "drawdown_strategy", 1000.0, 50.0, -20.0, 15.0, false, 1234578690), // -20, total: 50 (max drawdown)
            create_execution_result("dd_5", "drawdown_strategy", 1000.0, 50.0, 80.0, 10.0, true, 1234582290),  // +80, total: 130 (recovery)
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let metrics = tracker.calculate_performance_metrics("drawdown_strategy").unwrap();
        
        // Max drawdown should be calculated as percentage from peak
        // Peak was 100, lowest was 50, so drawdown is 50%
        assert!(metrics.max_drawdown_percentage > 0.0);
        assert!(metrics.max_drawdown_percentage <= 100.0);
    }

    #[tokio::test]
    async fn test_comprehensive_analytics_generation() {
        let mut tracker = StrategyPerformanceTracker::new();
        let executions = create_diverse_execution_history();
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let result = tracker.generate_comprehensive_analytics("strategy_1");
        assert!(result.is_ok());
        
        let analytics = result.unwrap();
        assert_eq!(analytics.strategy_id, "strategy_1");
        assert!(analytics.performance_metrics.total_executions > 0);
        assert!(analytics.attribution_analysis.factor_contributions.len() > 0);
        assert!(analytics.benchmark_comparison.comparisons.len() > 0);
        
        // Check that all time periods are covered
        assert!(analytics.performance_over_time.daily_returns.len() > 0);
        assert!(analytics.performance_over_time.monthly_returns.len() > 0);
        
        // Verify risk metrics
        assert!(analytics.risk_metrics.value_at_risk_95 > 0.0);
        assert!(analytics.risk_metrics.expected_shortfall > 0.0);
        assert!(analytics.risk_metrics.maximum_drawdown_duration > 0);
    }

    #[tokio::test]
    async fn test_performance_snapshots() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Add some executions
        let executions = vec![
            create_execution_result("snap_1", "snapshot_strategy", 1000.0, 50.0, 55.0, 10.0, true, 1234567890),
            create_execution_result("snap_2", "snapshot_strategy", 1200.0, 60.0, 65.0, 12.0, true, 1234571490),
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        // Create performance snapshot
        let result = tracker.create_performance_snapshot("snapshot_strategy");
        assert!(result.is_ok());
        
        let snapshot_id = result.unwrap();
        assert!(!snapshot_id.is_empty());
        assert_eq!(tracker.performance_snapshots.len(), 1);
        
        let snapshot = tracker.performance_snapshots.get(&snapshot_id).unwrap();
        assert_eq!(snapshot.strategy_id, "snapshot_strategy");
        assert!(snapshot.metrics.total_executions > 0);
        assert!(snapshot.timestamp > 0);
    }

    #[tokio::test]
    async fn test_benchmark_comparison() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Initialize benchmarks
        tracker.benchmark_data.initialize();
        assert!(tracker.benchmark_data.benchmarks.len() > 0);
        
        // Add executions for comparison
        let executions = create_diverse_execution_history();
        for execution in &executions[..5] { // Use first 5 executions
            tracker.record_execution(execution).unwrap();
        }
        
        let analytics = tracker.generate_comprehensive_analytics("strategy_1").unwrap();
        let benchmark_comparison = &analytics.benchmark_comparison;
        
        assert!(benchmark_comparison.comparisons.len() > 0);
        
        // Check that benchmarks have reasonable values
        for comparison in &benchmark_comparison.comparisons {
            assert!(!comparison.benchmark_name.is_empty());
            assert!(comparison.strategy_return.is_finite());
            assert!(comparison.benchmark_return.is_finite());
            assert!(comparison.excess_return.is_finite());
            assert!(comparison.correlation >= -1.0 && comparison.correlation <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_attribution_analysis() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Initialize attribution analyzer
        tracker.attribution_analyzer.initialize();
        assert!(tracker.attribution_analyzer.factor_weights.len() > 0);
        
        let executions = create_diverse_execution_history();
        for execution in &executions[..6] {
            tracker.record_execution(execution).unwrap();
        }
        
        let analytics = tracker.generate_comprehensive_analytics("strategy_1").unwrap();
        let attribution = &analytics.attribution_analysis;
        
        assert!(attribution.factor_contributions.len() > 0);
        assert!(attribution.total_return.is_finite());
        
        // Check that factor contributions sum to reasonable total
        let sum_contributions: f64 = attribution.factor_contributions.values().sum();
        assert!(sum_contributions.is_finite());
        
        // Verify that all major factors are present
        let factors: Vec<&String> = attribution.factor_contributions.keys().collect();
        assert!(factors.len() >= 3); // Should have at least 3 attribution factors
    }

    #[tokio::test]
    async fn test_time_series_analysis() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Create executions over different days
        let base_time = 1234567890;
        let day_seconds = 24 * 3600;
        
        let executions = vec![
            create_execution_result("day_1", "time_strategy", 1000.0, 50.0, 52.0, 10.0, true, base_time),
            create_execution_result("day_2", "time_strategy", 1000.0, 50.0, 48.0, 10.0, true, base_time + day_seconds),
            create_execution_result("day_3", "time_strategy", 1000.0, 50.0, 55.0, 10.0, true, base_time + 2 * day_seconds),
            create_execution_result("day_4", "time_strategy", 1000.0, 50.0, 53.0, 10.0, true, base_time + 3 * day_seconds),
            create_execution_result("day_5", "time_strategy", 1000.0, 50.0, 49.0, 10.0, true, base_time + 4 * day_seconds),
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let analytics = tracker.generate_comprehensive_analytics("time_strategy").unwrap();
        let time_series = &analytics.performance_over_time;
        
        assert!(time_series.daily_returns.len() > 0);
        assert!(time_series.cumulative_returns.len() > 0);
        assert!(time_series.rolling_sharpe_ratios.len() > 0);
        
        // Verify time series data makes sense
        for daily_return in &time_series.daily_returns {
            assert!(daily_return.is_finite());
        }
        
        // Cumulative returns should generally be increasing for profitable strategy
        let first_cumulative = time_series.cumulative_returns[0];
        let last_cumulative = time_series.cumulative_returns.last().unwrap();
        assert!(last_cumulative >= &first_cumulative); // Should be non-decreasing overall
    }

    #[tokio::test]
    async fn test_risk_metrics_calculation() {
        let mut tracker = StrategyPerformanceTracker::new();
        
        // Create diverse returns for risk calculation
        let executions = vec![
            create_execution_result("risk_1", "risk_strategy", 1000.0, 50.0, 60.0, 10.0, true, 1234567890),
            create_execution_result("risk_2", "risk_strategy", 1000.0, 50.0, 40.0, 10.0, true, 1234571490),
            create_execution_result("risk_3", "risk_strategy", 1000.0, 50.0, 70.0, 10.0, true, 1234575090),
            create_execution_result("risk_4", "risk_strategy", 1000.0, 50.0, 30.0, 10.0, true, 1234578690),
            create_execution_result("risk_5", "risk_strategy", 1000.0, 50.0, -10.0, 20.0, false, 1234582290),
            create_execution_result("risk_6", "risk_strategy", 1000.0, 50.0, 80.0, 10.0, true, 1234585890),
        ];
        
        for execution in executions {
            tracker.record_execution(&execution).unwrap();
        }
        
        let analytics = tracker.generate_comprehensive_analytics("risk_strategy").unwrap();
        let risk_metrics = &analytics.risk_metrics;
        
        // VaR should be positive (representing potential loss)
        assert!(risk_metrics.value_at_risk_95 > 0.0);
        assert!(risk_metrics.value_at_risk_99 > 0.0);
        
        // Expected shortfall should be >= VaR
        assert!(risk_metrics.expected_shortfall >= risk_metrics.value_at_risk_95);
        
        // Volatility should be positive
        assert!(risk_metrics.volatility > 0.0);
        
        // Maximum drawdown duration should be reasonable
        assert!(risk_metrics.maximum_drawdown_duration >= 0);
        
        // Downside deviation should be non-negative
        assert!(risk_metrics.downside_deviation >= 0.0);
    }
}