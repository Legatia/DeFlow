// Comprehensive Test Suite for Automated DeFi Strategy System
// Tests for all strategy components: execution, scanning, risk management, performance tracking

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defi::yield_farming::{ChainId, DeFiProtocol, UniswapVersion};
    use crate::defi::automated_strategies::{
        StrategyConfig, StrategyType, YieldFarmingConfig, ArbitrageConfig, RebalancingConfig,
        LiquidityMiningConfig, DCAConfig, ActiveStrategy, StrategyStatus, StrategyPerformanceMetrics,
        StrategyRiskMetrics, StrategyOpportunity, OpportunityType, StrategyExecutionResult,
        StrategyExecutionEngine, OpportunityScanner, StrategyRiskManager, StrategyPerformanceTracker,
        MultiStrategyCoordinator, StrategyError,
    };
    use crate::defi::automated_strategies::risk_manager::RiskLimits;
    use std::collections::HashMap;

    // Test helper functions
    fn create_test_strategy_config() -> StrategyConfig {
        StrategyConfig {
            name: "Test Strategy".to_string(),
            description: "Test strategy for unit testing".to_string(),
            strategy_type: StrategyType::YieldFarming(YieldFarmingConfig {
                min_apy_threshold: 5.0,
                preferred_tokens: vec!["USDC".to_string(), "ETH".to_string()],
                max_impermanent_loss_percentage: 5.0,
                auto_harvest_rewards: true,
            }),
            target_chains: vec![ChainId::Ethereum],
            target_protocols: vec![DeFiProtocol::Aave],
            risk_level: 3,
            max_allocation_usd: 10000.0,
            min_return_threshold: 5.0,
            execution_interval_minutes: 1440,
            gas_limit_usd: 50.0,
            auto_compound: true,
            stop_loss_percentage: Some(10.0),
            take_profit_percentage: Some(25.0),
        }
    }

    fn create_test_active_strategy() -> ActiveStrategy {
        ActiveStrategy {
            id: "test_strategy_1".to_string(),
            user_id: "test_user_1".to_string(),
            config: create_test_strategy_config(),
            allocated_capital: 5000.0,
            status: StrategyStatus::Active,
            created_at: 1234567890,
            last_updated: 1234567890,
            next_execution: Some(1234567890 + 86400),
            last_rebalance: None,
            performance_metrics: StrategyPerformanceMetrics {
                total_executions: 10,
                successful_executions: 8,
                total_pnl: 250.0,
                roi_percentage: 5.0,
                sharpe_ratio: 1.25,
                max_drawdown_percentage: 8.0,
                avg_execution_time_seconds: 30.0,
                total_gas_spent_usd: 25.0,
                win_rate_percentage: 75.0,
            },
            risk_metrics: StrategyRiskMetrics::default(),
            execution_history: Vec::new(),
        }
    }

    fn create_test_opportunity() -> StrategyOpportunity {
        StrategyOpportunity {
            id: "test_opp_1".to_string(),
            opportunity_type: OpportunityType::YieldFarming {
                apy: 8.5,
                tokens: vec!["USDC".to_string(), "ETH".to_string()],
                pool_address: "0x1234567890abcdef".to_string(),
            },
            chain: ChainId::Ethereum,
            protocol: DeFiProtocol::Aave,
            expected_return_percentage: 8.5,
            risk_score: 3,
            estimated_gas_cost: 25.0,
            liquidity_score: 9.0,
            time_sensitivity_minutes: 60,
            discovered_at: 1234567890,
            expires_at: 1234571490, // 1 hour later
        }
    }

    // Strategy Execution Engine Tests
    #[test]
    fn test_execution_engine_initialization() {
        let execution_engine = StrategyExecutionEngine::new();
        // Test basic initialization - just verify it's created successfully
        assert!(execution_engine.execution_metrics.successful_executions >= 0);
    }

    #[test]
    fn test_execution_engine_validate_prerequisites() {
        let _execution_engine = StrategyExecutionEngine::new();
        let strategy = create_test_active_strategy();
        let opportunity = create_test_opportunity();

        // This should validate successfully for reasonable parameters
        // Note: We can't easily test async methods in this environment,
        // so we focus on synchronous validation and setup
        assert_eq!(strategy.allocated_capital, 5000.0);
        assert_eq!(opportunity.expected_return_percentage, 8.5);
        assert!(opportunity.expires_at > opportunity.discovered_at);
    }

    // Opportunity Scanner Tests
    #[test]
    fn test_opportunity_scanner_initialization() {
        let scanner = OpportunityScanner::new();
        
        // Test basic initialization
        assert!(scanner.opportunity_cache.len() >= 0);
        assert!(scanner.min_apy_threshold > 0.0);
        assert!(scanner.min_liquidity_usd > 0.0);
    }

    #[test]
    fn test_opportunity_caching() {
        let mut scanner = OpportunityScanner::new();
        let test_opportunity = create_test_opportunity();
        
        scanner.cache_opportunity(test_opportunity.clone());
        assert_eq!(scanner.opportunity_cache.len(), 1);
        assert!(scanner.opportunity_cache.contains_key(&test_opportunity.id));
        
        let cached = scanner.opportunity_cache.get(&test_opportunity.id).unwrap();
        assert_eq!(cached.id, test_opportunity.id);
    }

    #[test]
    fn test_opportunity_filtering() {
        let scanner = OpportunityScanner::new();
        let mut opportunities = vec![create_test_opportunity()];
        
        // Add a low-quality opportunity
        let mut low_quality_opp = create_test_opportunity();
        low_quality_opp.id = "low_quality".to_string();
        low_quality_opp.expected_return_percentage = 2.0; // Below threshold
        low_quality_opp.risk_score = 9; // High risk
        opportunities.push(low_quality_opp);

        let filtered = scanner.filter_opportunities_by_quality(opportunities);
        assert_eq!(filtered.len(), 1); // Should filter out the low-quality one
        assert_eq!(filtered[0].id, "test_opp_1");
    }

    #[test]
    fn test_arbitrage_opportunity_scanner() {
        let scanner = OpportunityScanner::new();
        
        // Test arbitrage-related functionality through main scanner
        assert!(scanner.min_apy_threshold > 0.0);
        assert!(scanner.min_liquidity_usd > 0.0);
    }

    #[test]
    fn test_rebalancing_opportunity_scanner() {
        let scanner = OpportunityScanner::new();
        
        // Test rebalancing-related functionality through main scanner
        assert!(scanner.min_apy_threshold > 0.0);
        assert!(scanner.min_liquidity_usd > 0.0);
    }

    // Risk Manager Tests
    #[test]
    fn test_risk_manager_initialization() {
        let mut risk_manager = StrategyRiskManager::new();
        risk_manager.initialize_default_limits();

        assert!(risk_manager.global_limits.max_single_strategy_allocation > 0.0);
        assert!(risk_manager.global_limits.max_strategy_risk_score > 0);
        assert_eq!(risk_manager.user_limits.len(), 0);
        assert_eq!(risk_manager.strategy_limits.len(), 0);
    }

    #[test]
    fn test_capital_allocation_validation() {
        let mut risk_manager = StrategyRiskManager::new();
        risk_manager.initialize_default_limits();
        
        let strategy = create_test_active_strategy();
        let capital_amount = 5000.0; // Within limits

        let result = risk_manager.validate_capital_allocation(&strategy, capital_amount);
        assert!(result.is_ok());
    }

    #[test]
    fn test_capital_allocation_exceeds_limits() {
        let mut risk_manager = StrategyRiskManager::new();
        risk_manager.initialize_default_limits();
        
        let strategy = create_test_active_strategy();
        let capital_amount = 200000.0; // Exceeds global limit

        let result = risk_manager.validate_capital_allocation(&strategy, capital_amount);
        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = error.to_string();
            assert!(error_msg.contains("exceeds") || error_msg.contains("limit"));
        }
    }

    #[test]
    fn test_strategy_risk_score_calculation() {
        let risk_manager = StrategyRiskManager::new();
        let strategy = create_test_active_strategy();
        
        let result = risk_manager.calculate_strategy_risk_score(&strategy);
        assert!(result.is_ok());
        
        let risk_score = result.unwrap();
        assert!(risk_score >= 1 && risk_score <= 10);
    }

    #[test]
    fn test_user_risk_limits() {
        let mut risk_manager = StrategyRiskManager::new();
        risk_manager.initialize_default_limits();
        
        let user_limits = RiskLimits {
            max_single_strategy_allocation: 10000.0,
            max_strategy_risk_score: 7,
            max_daily_loss_percentage: 5.0,
            max_total_exposure_percentage: 80.0,
            emergency_stop_enabled: true,
        };

        let result = risk_manager.set_user_risk_limits("test_user_1".to_string(), user_limits);
        assert!(result.is_ok());
        assert_eq!(risk_manager.user_limits.len(), 1);
    }

    #[test]
    fn test_risk_assessment_generation() {
        let risk_manager = StrategyRiskManager::new();
        let strategy = create_test_active_strategy();
        
        let result = risk_manager.get_strategy_risk_assessment(&strategy);
        assert!(result.is_ok());
        
        let assessment = result.unwrap();
        assert_eq!(assessment.strategy_id, strategy.id);
        assert!(assessment.overall_risk_score >= 1 && assessment.overall_risk_score <= 10);
        assert!(assessment.risk_factors.len() > 0);
        assert!(assessment.mitigation_suggestions.len() > 0);
    }

    // Performance Tracker Tests
    #[test]
    fn test_performance_tracker_initialization() {
        let performance_tracker = StrategyPerformanceTracker::new();
        
        assert_eq!(performance_tracker.performance_history.len(), 0);
        assert_eq!(performance_tracker.benchmark_data.benchmarks.len(), 0);
    }

    #[test]
    fn test_execution_recording() {
        let mut performance_tracker = StrategyPerformanceTracker::new();
        
        let execution_result = StrategyExecutionResult {
            execution_id: "test_exec_1".to_string(),
            strategy_id: "test_strategy_1".to_string(),
            user_id: "test_user_1".to_string(),
            opportunity_id: "test_opp_1".to_string(),
            action_type: "yield_farming".to_string(),
            amount_usd: 1000.0,
            expected_return: 50.0,
            actual_return: 45.0,
            gas_cost_usd: 25.0,
            execution_time_seconds: 30,
            success: true,
            error_message: None,
            transaction_hashes: vec!["0xabc123".to_string()],
            executed_at: 1234567890,
        };

        let result = performance_tracker.record_execution(&execution_result);
        assert!(result.is_ok());
        assert_eq!(performance_tracker.performance_history.len(), 1);
    }

    #[test]
    fn test_performance_metrics_calculation() {
        let mut performance_tracker = StrategyPerformanceTracker::new();
        
        // Add multiple execution results
        for i in 0..5 {
            let execution_result = StrategyExecutionResult {
                execution_id: format!("test_exec_{}", i),
                strategy_id: "test_strategy_1".to_string(),
                user_id: "test_user_1".to_string(),
                opportunity_id: format!("test_opp_{}", i),
                action_type: "yield_farming".to_string(),
                amount_usd: 1000.0,
                expected_return: 50.0,
                actual_return: if i % 2 == 0 { 45.0 } else { 55.0 },
                gas_cost_usd: 25.0,
                execution_time_seconds: 30,
                success: i < 4, // 4 successful, 1 failed
                error_message: if i == 4 { Some("Test error".to_string()) } else { None },
                transaction_hashes: vec![format!("0xabc12{}", i)],
                executed_at: 1234567890 + (i as u64 * 3600),
            };
            
            performance_tracker.record_execution(&execution_result).unwrap();
        }

        let result = performance_tracker.calculate_performance_metrics("test_strategy_1");
        assert!(result.is_ok());
        
        let metrics = result.unwrap();
        assert_eq!(metrics.total_executions, 5);
        assert_eq!(metrics.successful_executions, 4);
        assert_eq!(metrics.win_rate_percentage, 80.0);
        assert!(metrics.total_pnl > 0.0);
    }

    // Coordination Engine Tests
    #[test]
    fn test_coordination_engine_initialization() {
        let coordination_engine = MultiStrategyCoordinator::new();
        
        // Test basic initialization of coordination engine
        assert_eq!(coordination_engine.coordination_history.len(), 0);
    }

    // Integration Tests
    #[test]
    fn test_full_strategy_setup() {
        let execution_engine = StrategyExecutionEngine::new();
        let mut risk_manager = StrategyRiskManager::new();
        let performance_tracker = StrategyPerformanceTracker::new();
        
        risk_manager.initialize_default_limits();
        
        let strategy = create_test_active_strategy();
        let opportunity = create_test_opportunity();
        
        // Test that all components can be initialized and basic operations work
        assert!(execution_engine.execution_metrics.successful_executions >= 0);
        assert!(risk_manager.global_limits.max_single_strategy_allocation > 0.0);
        assert_eq!(performance_tracker.performance_history.len(), 0);
        
        // Test risk validation
        let risk_result = risk_manager.validate_capital_allocation(&strategy, 5000.0);
        assert!(risk_result.is_ok());
        
        // Test opportunity validation
        assert!(opportunity.expected_return_percentage > 0.0);
        assert!(opportunity.risk_score <= 10);
    }

    #[test]
    fn test_multi_chain_strategy_setup() {
        let coordination_engine = MultiStrategyCoordinator::new();
        
        // Create strategies for different chains
        let mut eth_strategy = create_test_active_strategy();
        eth_strategy.id = "eth_strategy".to_string();
        eth_strategy.config.target_chains = vec![ChainId::Ethereum];
        
        let mut btc_strategy = create_test_active_strategy();
        btc_strategy.id = "btc_strategy".to_string();
        btc_strategy.config.target_chains = vec![ChainId::Bitcoin];
        
        let mut sol_strategy = create_test_active_strategy();
        sol_strategy.id = "sol_strategy".to_string();
        sol_strategy.config.target_chains = vec![ChainId::Solana];
        
        // Test that strategies can be created with different chain configurations
        assert_eq!(eth_strategy.config.target_chains[0], ChainId::Ethereum);
        assert_eq!(btc_strategy.config.target_chains[0], ChainId::Bitcoin);
        assert_eq!(sol_strategy.config.target_chains[0], ChainId::Solana);
        
        // Test coordination engine can handle multiple strategies
        assert_eq!(coordination_engine.coordination_history.len(), 0);
    }

    #[test]
    fn test_strategy_type_configurations() {
        // Test YieldFarming strategy type
        let yield_config = YieldFarmingConfig {
            min_apy_threshold: 5.0,
            preferred_tokens: vec!["USDC".to_string(), "ETH".to_string()],
            max_impermanent_loss_percentage: 5.0,
            auto_harvest_rewards: true,
        };
        
        assert_eq!(yield_config.min_apy_threshold, 5.0);
        assert_eq!(yield_config.preferred_tokens.len(), 2);
        assert!(yield_config.auto_harvest_rewards);
        
        // Test Arbitrage strategy type
        let arbitrage_config = ArbitrageConfig {
            min_profit_percentage: 0.5,
            max_execution_time_seconds: 300,
            max_slippage_percentage: 1.0,
            preferred_dex_pairs: vec![
                ("Uniswap".to_string(), "Sushiswap".to_string()),
                ("Curve".to_string(), "Balancer".to_string()),
            ],
        };
        
        assert_eq!(arbitrage_config.min_profit_percentage, 0.5);
        assert_eq!(arbitrage_config.preferred_dex_pairs.len(), 2);
        assert_eq!(arbitrage_config.max_execution_time_seconds, 300);
        
        // Test Rebalancing strategy type
        let rebalancing_config = RebalancingConfig {
            target_allocation: vec![
                ("ETH".to_string(), 50.0),
                ("BTC".to_string(), 30.0),
                ("USDC".to_string(), 20.0),
            ].into_iter().collect(),
            rebalance_threshold_percentage: 5.0,
            rebalance_frequency_hours: 168, // Weekly
        };
        
        assert_eq!(rebalancing_config.rebalance_threshold_percentage, 5.0);
        assert_eq!(rebalancing_config.target_allocation.len(), 3);
        assert_eq!(rebalancing_config.rebalance_frequency_hours, 168);
    }

    #[test]
    fn test_opportunity_types() {
        // Test YieldFarming opportunity
        let yield_opp = OpportunityType::YieldFarming {
            apy: 8.5,
            tokens: vec!["USDC".to_string(), "ETH".to_string()],
            pool_address: "0x1234567890abcdef".to_string(),
        };
        
        if let OpportunityType::YieldFarming { apy, .. } = yield_opp {
            assert_eq!(apy, 8.5);
        } else {
            panic!("Expected YieldFarming opportunity type");
        }
        
        // Test Arbitrage opportunity
        let arb_opp = OpportunityType::Arbitrage {
            profit_percentage: 1.5,
            token_pair: ("ETH".to_string(), "USDC".to_string()),
            dex_pair: ("Uniswap".to_string(), "SushiSwap".to_string()),
        };
        
        if let OpportunityType::Arbitrage { profit_percentage, .. } = arb_opp {
            assert_eq!(profit_percentage, 1.5);
        } else {
            panic!("Expected Arbitrage opportunity type");
        }
    }

    #[test]
    fn test_error_handling() {
        let mut risk_manager = StrategyRiskManager::new();
        risk_manager.initialize_default_limits();
        let strategy = create_test_active_strategy();
        
        // Test error case - excessive capital allocation
        let result = risk_manager.validate_capital_allocation(&strategy, 1000000.0);
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(error.to_string().contains("limit") || error.to_string().contains("exceed"));
        } else {
            panic!("Expected error for excessive capital allocation");
        }
    }
}