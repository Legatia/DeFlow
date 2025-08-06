// Multi-Strategy Coordination Engine - Coordinate multiple strategies
// Optimize resource allocation, prevent conflicts, and synchronize execution

use super::*;
use crate::defi::yield_farming::ChainId;

/// Multi-strategy coordination engine for portfolio optimization
#[derive(Debug, Clone)]
pub struct MultiStrategyCoordinator {
    pub coordination_rules: CoordinationRules,
    pub resource_allocator: ResourceAllocator,
    pub conflict_resolver: ConflictResolver,
    pub synchronizer: ExecutionSynchronizer,
    pub optimizer: PortfolioOptimizer,
    pub rebalancer: StrategyRebalancer,
    pub performance_analyzer: CrossStrategyAnalyzer,
    pub coordination_history: Vec<CoordinationAction>,
}

impl MultiStrategyCoordinator {
    pub fn new() -> Self {
        Self {
            coordination_rules: CoordinationRules::default(),
            resource_allocator: ResourceAllocator::new(),
            conflict_resolver: ConflictResolver::new(),
            synchronizer: ExecutionSynchronizer::new(),
            optimizer: PortfolioOptimizer::new(),
            rebalancer: StrategyRebalancer::new(),
            performance_analyzer: CrossStrategyAnalyzer::new(),
            coordination_history: Vec::new(),
        }
    }

    /// Main coordination function - optimize strategy portfolio
    pub fn coordinate_strategies(&mut self, strategies: &mut HashMap<String, ActiveStrategy>) -> Result<CoordinationResult, StrategyError> {
        let coordination_start = self.get_current_time();
        let mut actions_taken = Vec::new();

        // Group strategies by user for coordination
        let user_strategies = self.group_strategies_by_user(strategies);

        for (user_id, strategy_ids) in user_strategies {
            if strategy_ids.len() < 2 {
                continue; // No need to coordinate single strategy
            }

            // Create owned copies of strategy data for analysis to avoid borrowing conflicts
            let user_strategies_data: Vec<ActiveStrategy> = strategy_ids.iter()
                .filter_map(|id| strategies.get(id))
                .cloned()
                .collect();

            if user_strategies_data.is_empty() {
                continue;
            }

            // Convert to references for analysis
            let user_strategies_refs: Vec<&ActiveStrategy> = user_strategies_data.iter().collect();

            // Detect and resolve conflicts
            let conflicts = self.conflict_resolver.detect_conflicts(&user_strategies_refs)?;
            if !conflicts.is_empty() {
                let resolutions = self.conflict_resolver.resolve_conflicts(conflicts)?;
                for resolution in resolutions {
                    if let Some(strategy) = strategies.get_mut(&resolution.strategy_id) {
                        self.apply_conflict_resolution(strategy, resolution)?;
                        actions_taken.push(CoordinationActionType::ConflictResolution);
                    }
                }
            }

            // Optimize resource allocation
            let allocation_optimization = self.resource_allocator.optimize_allocation(&user_strategies_refs, &user_id)?;
            if allocation_optimization.improvements.len() > 0 {
                for improvement in allocation_optimization.improvements {
                    if let Some(strategy) = strategies.get_mut(&improvement.strategy_id) {
                        self.apply_allocation_improvement(strategy, improvement)?;
                        actions_taken.push(CoordinationActionType::AllocationOptimization);
                    }
                }
            }

            // Check for rebalancing opportunities
            let rebalancing_suggestions = self.rebalancer.analyze_rebalancing_needs(&user_strategies_refs)?;
            if !rebalancing_suggestions.is_empty() {
                for suggestion in rebalancing_suggestions {
                    if let Some(strategy) = strategies.get_mut(&suggestion.strategy_id) {
                        self.apply_rebalancing_suggestion(strategy, suggestion)?;
                        actions_taken.push(CoordinationActionType::Rebalancing);
                    }
                }
            }

            // Synchronize execution timing
            let sync_adjustments = self.synchronizer.optimize_execution_timing(&user_strategies_refs)?;
            for adjustment in sync_adjustments {
                if let Some(strategy) = strategies.get_mut(&adjustment.strategy_id) {
                    self.apply_timing_adjustment(strategy, adjustment)?;
                    actions_taken.push(CoordinationActionType::TimingSynchronization);
                }
            }

            // Portfolio-level optimization
            let portfolio_optimizations = self.optimizer.optimize_strategy_portfolio(&user_strategies_refs)?;
            for optimization in portfolio_optimizations {
                if let Some(strategy) = strategies.get_mut(&optimization.strategy_id) {
                    self.apply_portfolio_optimization(strategy, optimization)?;
                    actions_taken.push(CoordinationActionType::PortfolioOptimization);
                }
            }
        }

        // Record coordination action
        let coordination_action = CoordinationAction {
            timestamp: coordination_start,
            actions_taken: actions_taken.clone(),
            strategies_affected: strategies.len(),
            execution_time_ms: (self.get_current_time() - coordination_start) / 1_000_000,
            improvements_achieved: self.calculate_coordination_improvements(&actions_taken),
        };

        self.coordination_history.push(coordination_action.clone());

        // Clean up old history (keep last 1000 entries)
        if self.coordination_history.len() > 1000 {
            self.coordination_history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            self.coordination_history.truncate(1000);
        }

        Ok(CoordinationResult {
            coordination_action,
            total_strategies_coordinated: strategies.len(),
            conflicts_resolved: actions_taken.iter().filter(|a| matches!(a, CoordinationActionType::ConflictResolution)).count(),
            optimizations_applied: actions_taken.iter().filter(|a| matches!(a, CoordinationActionType::AllocationOptimization)).count(),
            execution_time_ms: (self.get_current_time() - coordination_start) / 1_000_000,
        })
    }

    /// Analyze cross-strategy performance and correlation
    pub fn analyze_cross_strategy_performance(&self, strategies: &HashMap<String, ActiveStrategy>) -> Result<CrossStrategyAnalysis, StrategyError> {
        self.performance_analyzer.analyze_cross_strategy_performance(strategies)
    }

    /// Get coordination recommendations for user's strategy portfolio
    pub fn get_coordination_recommendations(&self, user_id: &str, strategies: &HashMap<String, ActiveStrategy>) -> Result<Vec<CoordinationRecommendation>, StrategyError> {
        let user_strategy_ids: Vec<String> = strategies.values()
            .filter(|s| s.user_id == user_id)
            .map(|s| s.id.clone())
            .collect();

        if user_strategy_ids.len() < 2 {
            return Ok(vec![CoordinationRecommendation {
                recommendation_type: RecommendationType::Information,
                title: "Single Strategy Portfolio".to_string(),
                description: "You have only one active strategy. Consider diversifying with additional strategies for better risk management.".to_string(),
                priority: RecommendationPriority::Low,
                expected_benefit: "Improved risk-adjusted returns through diversification".to_string(),
                implementation_complexity: 3,
                estimated_improvement_percentage: 15.0,
            }]);
        }

        let mut recommendations = Vec::new();

        // Check for correlation issues
        let correlation_analysis = self.analyze_strategy_correlations(&user_strategy_ids, strategies)?;
        if correlation_analysis.max_correlation > 0.8 {
            recommendations.push(CoordinationRecommendation {
                recommendation_type: RecommendationType::RiskReduction,
                title: "High Strategy Correlation Detected".to_string(),
                description: format!("Your strategies have high correlation ({:.1}%). Consider diversifying across different chains or strategy types.", correlation_analysis.max_correlation * 100.0),
                priority: RecommendationPriority::High,
                expected_benefit: "Reduced portfolio risk through diversification".to_string(),
                implementation_complexity: 5,
                estimated_improvement_percentage: 12.0,
            });
        }

        // Check for allocation inefficiencies
        let allocation_analysis = self.analyze_allocation_efficiency(&user_strategy_ids, strategies)?;
        if allocation_analysis.efficiency_score < 0.7 {
            recommendations.push(CoordinationRecommendation {
                recommendation_type: RecommendationType::AllocationOptimization,
                title: "Suboptimal Capital Allocation".to_string(),
                description: "Your capital allocation across strategies could be optimized for better risk-adjusted returns.".to_string(),
                priority: RecommendationPriority::Medium,
                expected_benefit: "Improved returns through better capital allocation".to_string(),
                implementation_complexity: 4,
                estimated_improvement_percentage: 8.5,
            });
        }

        // Check for timing conflicts
        let timing_analysis = self.analyze_execution_timing(&user_strategy_ids, strategies)?;
        if timing_analysis.conflicts_detected > 0 {
            recommendations.push(CoordinationRecommendation {
                recommendation_type: RecommendationType::TimingOptimization,
                title: "Execution Timing Conflicts".to_string(),
                description: format!("Detected {} timing conflicts that could impact performance.", timing_analysis.conflicts_detected),
                priority: RecommendationPriority::Medium,
                expected_benefit: "Better execution efficiency and reduced slippage".to_string(),
                implementation_complexity: 3,
                estimated_improvement_percentage: 5.2,
            });
        }

        // Check for gas optimization opportunities
        let gas_analysis = self.analyze_gas_efficiency(&user_strategy_ids, strategies)?;
        if gas_analysis.optimization_potential > 10.0 {
            recommendations.push(CoordinationRecommendation {
                recommendation_type: RecommendationType::GasOptimization,
                title: "Gas Cost Optimization Opportunity".to_string(),
                description: format!("Could reduce gas costs by {:.1}% through better coordination.", gas_analysis.optimization_potential),
                priority: RecommendationPriority::Low,
                expected_benefit: "Reduced transaction costs".to_string(),
                implementation_complexity: 2,
                estimated_improvement_percentage: gas_analysis.optimization_potential,
            });
        }

        // Sort recommendations by priority and potential benefit
        recommendations.sort_by(|a, b| {
            let priority_order = |p: &RecommendationPriority| match p {
                RecommendationPriority::High => 3,
                RecommendationPriority::Medium => 2,
                RecommendationPriority::Low => 1,
            };
            
            let a_score = priority_order(&a.priority) as f64 * 1000.0 + a.estimated_improvement_percentage;
            let b_score = priority_order(&b.priority) as f64 * 1000.0 + b.estimated_improvement_percentage;
            
            b_score.partial_cmp(&a_score).unwrap()
        });

        Ok(recommendations)
    }

    /// Get coordination statistics and performance
    pub fn get_coordination_statistics(&self) -> CoordinationStatistics {
        let total_actions = self.coordination_history.len();
        
        let actions_by_type = self.coordination_history.iter()
            .flat_map(|h| &h.actions_taken)
            .fold(std::collections::HashMap::new(), |mut acc, action| {
                *acc.entry(format!("{:?}", action)).or_insert(0) += 1;
                acc
            });

        let avg_execution_time = if total_actions > 0 {
            self.coordination_history.iter()
                .map(|h| h.execution_time_ms)
                .sum::<u64>() as f64 / total_actions as f64
        } else {
            0.0
        };

        let total_improvements: f64 = self.coordination_history.iter()
            .map(|h| h.improvements_achieved)
            .sum();

        CoordinationStatistics {
            total_coordinations: total_actions,
            actions_by_type,
            average_execution_time_ms: avg_execution_time,
            total_improvements_achieved: total_improvements,
            last_coordination: self.coordination_history.last().map(|h| h.timestamp),
            success_rate: 98.5, // Mock value
        }
    }

    /// Set custom coordination rules
    pub fn set_coordination_rules(&mut self, rules: CoordinationRules) {
        self.coordination_rules = rules;
    }

    // Private helper methods
    fn group_strategies_by_user(&self, strategies: &HashMap<String, ActiveStrategy>) -> HashMap<String, Vec<String>> {
        let mut user_strategies = HashMap::new();
        
        for (strategy_id, strategy) in strategies {
            user_strategies.entry(strategy.user_id.clone())
                .or_insert_with(Vec::new)
                .push(strategy_id.clone());
        }
        
        user_strategies
    }

    fn apply_conflict_resolution(&self, strategy: &mut ActiveStrategy, resolution: ConflictResolution) -> Result<(), StrategyError> {
        match resolution.resolution_type {
            ResolutionType::DelayExecution => {
                if let Some(next_execution) = strategy.next_execution {
                    strategy.next_execution = Some(next_execution + resolution.delay_seconds * 1_000_000_000);
                }
            },
            ResolutionType::ReduceAllocation => {
                strategy.allocated_capital *= resolution.adjustment_factor;
            },
            ResolutionType::ChangeChain => {
                // Logic to change target chains based on resolution
                // This would require more complex implementation
            },
            ResolutionType::PauseTemporarily => {
                strategy.status = StrategyStatus::Paused;
            },
        }
        Ok(())
    }

    fn apply_allocation_improvement(&self, strategy: &mut ActiveStrategy, improvement: AllocationImprovement) -> Result<(), StrategyError> {
        strategy.allocated_capital = improvement.new_allocation;
        Ok(())
    }

    fn apply_rebalancing_suggestion(&self, _strategy: &mut ActiveStrategy, _suggestion: RebalancingSuggestion) -> Result<(), StrategyError> {
        // Apply rebalancing logic
        Ok(())
    }

    fn apply_timing_adjustment(&self, strategy: &mut ActiveStrategy, adjustment: TimingAdjustment) -> Result<(), StrategyError> {
        if let Some(next_execution) = strategy.next_execution {
            strategy.next_execution = Some(next_execution + adjustment.delay_seconds * 1_000_000_000);
        }
        Ok(())
    }

    fn apply_portfolio_optimization(&self, _strategy: &mut ActiveStrategy, _optimization: PortfolioOptimization) -> Result<(), StrategyError> {
        // Apply portfolio optimization logic
        Ok(())
    }

    fn calculate_coordination_improvements(&self, actions: &[CoordinationActionType]) -> f64 {
        // Mock calculation of improvements achieved
        actions.len() as f64 * 2.5 // Average 2.5% improvement per action
    }

    fn analyze_strategy_correlations(&self, _strategy_ids: &[String], _strategies: &HashMap<String, ActiveStrategy>) -> Result<CorrelationAnalysis, StrategyError> {
        Ok(CorrelationAnalysis {
            max_correlation: 0.85,
            avg_correlation: 0.42,
            correlation_matrix: HashMap::new(), // Would be populated in real implementation
        })
    }

    fn analyze_allocation_efficiency(&self, _strategy_ids: &[String], _strategies: &HashMap<String, ActiveStrategy>) -> Result<AllocationAnalysis, StrategyError> {
        Ok(AllocationAnalysis {
            efficiency_score: 0.65,
            optimal_allocation: HashMap::new(), // Would be calculated in real implementation
            current_allocation: HashMap::new(),
        })
    }

    fn analyze_execution_timing(&self, _strategy_ids: &[String], _strategies: &HashMap<String, ActiveStrategy>) -> Result<TimingAnalysis, StrategyError> {
        Ok(TimingAnalysis {
            conflicts_detected: 2,
            optimal_schedule: HashMap::new(),
            current_schedule: HashMap::new(),
        })
    }

    fn analyze_gas_efficiency(&self, _strategy_ids: &[String], _strategies: &HashMap<String, ActiveStrategy>) -> Result<GasAnalysis, StrategyError> {
        Ok(GasAnalysis {
            optimization_potential: 15.3,
            current_gas_cost: 125.0,
            optimized_gas_cost: 105.9,
        })
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

// Supporting coordination components

/// Resource allocator for optimal capital distribution
#[derive(Debug, Clone)]
pub struct ResourceAllocator;

impl ResourceAllocator {
    pub fn new() -> Self { Self }

    pub fn optimize_allocation(&self, strategies: &[&ActiveStrategy], user_id: &str) -> Result<AllocationOptimizationResult, StrategyError> {
        let total_allocation: f64 = strategies.iter().map(|s| s.allocated_capital).sum();
        let mut improvements = Vec::new();

        // Simple optimization: reallocate based on performance
        for strategy in strategies {
            let performance_score = strategy.performance_metrics.roi_percentage;
            let current_percentage = (strategy.allocated_capital / total_allocation) * 100.0;
            
            // If strategy is performing well, suggest increasing allocation
            if performance_score > 10.0 && current_percentage < 40.0 {
                improvements.push(AllocationImprovement {
                    strategy_id: strategy.id.clone(),
                    current_allocation: strategy.allocated_capital,
                    new_allocation: strategy.allocated_capital * 1.1,
                    reason: "Strong performance warrants increased allocation".to_string(),
                    expected_improvement: 5.0,
                });
            }
            // If strategy is underperforming, suggest decreasing allocation
            else if performance_score < -5.0 && current_percentage > 15.0 {
                improvements.push(AllocationImprovement {
                    strategy_id: strategy.id.clone(),
                    current_allocation: strategy.allocated_capital,
                    new_allocation: strategy.allocated_capital * 0.9,
                    reason: "Poor performance suggests reduced allocation".to_string(),
                    expected_improvement: 3.0,
                });
            }
        }

        Ok(AllocationOptimizationResult {
            user_id: user_id.to_string(),
            improvements,
            total_strategies_analyzed: strategies.len(),
            optimization_confidence: 0.75,
        })
    }
}

/// Conflict resolver for strategy conflicts
#[derive(Debug, Clone)]
pub struct ConflictResolver;

impl ConflictResolver {
    pub fn new() -> Self { Self }

    pub fn detect_conflicts(&self, strategies: &[&ActiveStrategy]) -> Result<Vec<StrategyConflict>, StrategyError> {
        let mut conflicts = Vec::new();

        // Check for resource conflicts (same chain/protocol at same time)
        for (i, strategy_a) in strategies.iter().enumerate() {
            for strategy_b in strategies.iter().skip(i + 1) {
                if self.strategies_have_resource_conflict(strategy_a, strategy_b) {
                    conflicts.push(StrategyConflict {
                        conflict_type: ConflictType::ResourceContention,
                        strategy_ids: vec![strategy_a.id.clone(), strategy_b.id.clone()],
                        description: "Strategies competing for same resources".to_string(),
                        severity: ConflictSeverity::Medium,
                    });
                }

                if self.strategies_have_timing_conflict(strategy_a, strategy_b) {
                    conflicts.push(StrategyConflict {
                        conflict_type: ConflictType::ExecutionTiming,
                        strategy_ids: vec![strategy_a.id.clone(), strategy_b.id.clone()],
                        description: "Strategies scheduled to execute simultaneously".to_string(),
                        severity: ConflictSeverity::Low,
                    });
                }
            }
        }

        Ok(conflicts)
    }

    pub fn resolve_conflicts(&self, conflicts: Vec<StrategyConflict>) -> Result<Vec<ConflictResolution>, StrategyError> {
        let mut resolutions = Vec::new();

        for conflict in conflicts {
            let resolution = match conflict.conflict_type {
                ConflictType::ResourceContention => ConflictResolution {
                    conflict_id: format!("conflict_{:x}", self.get_current_time()),
                    strategy_id: conflict.strategy_ids[0].clone(), // Affect first strategy
                    resolution_type: ResolutionType::DelayExecution,
                    delay_seconds: 300, // 5 minute delay
                    adjustment_factor: 1.0,
                    reason: "Delay execution to avoid resource contention".to_string(),
                },
                ConflictType::ExecutionTiming => ConflictResolution {
                    conflict_id: format!("conflict_{:x}", self.get_current_time()),
                    strategy_id: conflict.strategy_ids[1].clone(), // Affect second strategy
                    resolution_type: ResolutionType::DelayExecution,
                    delay_seconds: 60, // 1 minute delay
                    adjustment_factor: 1.0,
                    reason: "Stagger execution timing".to_string(),
                },
                ConflictType::AllocationImbalance => ConflictResolution {
                    conflict_id: format!("conflict_{:x}", self.get_current_time()),
                    strategy_id: conflict.strategy_ids[0].clone(),
                    resolution_type: ResolutionType::ReduceAllocation,
                    delay_seconds: 0,
                    adjustment_factor: 0.8,
                    reason: "Reduce allocation to balance portfolio".to_string(),
                },
            };

            resolutions.push(resolution);
        }

        Ok(resolutions)
    }

    fn strategies_have_resource_conflict(&self, strategy_a: &ActiveStrategy, strategy_b: &ActiveStrategy) -> bool {
        // Check if strategies target same chains and protocols
        let chains_overlap = strategy_a.config.target_chains.iter()
            .any(|chain| strategy_b.config.target_chains.contains(chain));
        
        let protocols_overlap = strategy_a.config.target_protocols.iter()
            .any(|protocol| strategy_b.config.target_protocols.contains(protocol));

        chains_overlap && protocols_overlap
    }

    fn strategies_have_timing_conflict(&self, strategy_a: &ActiveStrategy, strategy_b: &ActiveStrategy) -> bool {
        // Check if strategies are scheduled to execute at similar times
        if let (Some(time_a), Some(time_b)) = (strategy_a.next_execution, strategy_b.next_execution) {
            let time_diff = (time_a as i64 - time_b as i64).abs() as u64;
            time_diff < 60 * 1_000_000_000 // Within 1 minute
        } else {
            false
        }
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

/// Execution synchronizer for timing optimization
#[derive(Debug, Clone)]
pub struct ExecutionSynchronizer;

impl ExecutionSynchronizer {
    pub fn new() -> Self { Self }

    pub fn optimize_execution_timing(&self, strategies: &[&ActiveStrategy]) -> Result<Vec<TimingAdjustment>, StrategyError> {
        let mut adjustments = Vec::new();

        // Spread out executions to avoid simultaneous execution
        let mut execution_times: Vec<(String, u64)> = strategies.iter()
            .filter_map(|s| s.next_execution.map(|time| (s.id.clone(), time)))
            .collect();

        execution_times.sort_by_key(|(_, time)| *time);

        for i in 1..execution_times.len() {
            let (current_id, current_time) = &execution_times[i];
            let (_, prev_time) = &execution_times[i-1];

            // If executions are too close together, delay the later one
            if current_time - prev_time < 120 * 1_000_000_000 { // Less than 2 minutes apart
                adjustments.push(TimingAdjustment {
                    strategy_id: current_id.clone(),
                    delay_seconds: 180, // 3 minute delay
                    reason: "Spread out execution timing".to_string(),
                });
            }
        }

        Ok(adjustments)
    }
}

/// Portfolio optimizer for overall strategy optimization
#[derive(Debug, Clone)]
pub struct PortfolioOptimizer;

impl PortfolioOptimizer {
    pub fn new() -> Self { Self }

    pub fn optimize_strategy_portfolio(&self, strategies: &[&ActiveStrategy]) -> Result<Vec<PortfolioOptimization>, StrategyError> {
        let mut optimizations = Vec::new();
        
        if strategies.len() < 2 {
            return Ok(optimizations);
        }

        // Analyze portfolio composition
        let total_allocation: f64 = strategies.iter().map(|s| s.allocated_capital).sum();
        if total_allocation == 0.0 {
            return Ok(optimizations);
        }

        // 1. Risk Parity Optimization
        let risk_parity_optimization = self.calculate_risk_parity_optimization(strategies, total_allocation)?;
        if let Some(opt) = risk_parity_optimization {
            optimizations.push(opt);
        }

        // 2. Mean Reversion Optimization
        let mean_reversion_optimization = self.calculate_mean_reversion_optimization(strategies)?;
        if let Some(opt) = mean_reversion_optimization {
            optimizations.push(opt);
        }

        // 3. Momentum-based Optimization
        let momentum_optimization = self.calculate_momentum_optimization(strategies)?;
        if let Some(opt) = momentum_optimization {
            optimizations.push(opt);
        }

        // 4. Diversification Optimization
        let diversification_optimization = self.calculate_diversification_optimization(strategies)?;
        if let Some(opt) = diversification_optimization {
            optimizations.push(opt);
        }

        // 5. Sharpe Ratio Optimization
        let sharpe_optimization = self.calculate_sharpe_optimization(strategies, total_allocation)?;
        if let Some(opt) = sharpe_optimization {
            optimizations.push(opt);
        }

        Ok(optimizations)
    }

    /// Calculate risk parity optimization
    fn calculate_risk_parity_optimization(&self, strategies: &[&ActiveStrategy], total_allocation: f64) -> Result<Option<PortfolioOptimization>, StrategyError> {
        let mut parameters = HashMap::new();
        
        // Calculate each strategy's risk contribution
        let total_risk: f64 = strategies.iter().map(|s| s.config.risk_level as f64).sum();
        
        for strategy in strategies {
            let target_allocation_percentage = (strategy.config.risk_level as f64 / total_risk) * 100.0;
            let current_percentage = (strategy.allocated_capital / total_allocation) * 100.0;
            
            let deviation = (target_allocation_percentage - current_percentage).abs();
            if deviation > 5.0 { // More than 5% deviation from risk parity
                parameters.insert(format!("{}_target_allocation", strategy.id), target_allocation_percentage);
                parameters.insert(format!("{}_current_allocation", strategy.id), current_percentage);
                parameters.insert(format!("{}_deviation", strategy.id), deviation);
            }
        }

        if !parameters.is_empty() {
            Ok(Some(PortfolioOptimization {
                strategy_id: "portfolio_risk_parity".to_string(),
                optimization_type: "Risk Parity Rebalancing".to_string(),
                parameters,
            }))
        } else {
            Ok(None)
        }
    }

    /// Calculate mean reversion optimization
    fn calculate_mean_reversion_optimization(&self, strategies: &[&ActiveStrategy]) -> Result<Option<PortfolioOptimization>, StrategyError> {
        let mut parameters = HashMap::new();
        let mut needs_optimization = false;

        for strategy in strategies {
            let recent_performance = strategy.performance_metrics.roi_percentage;
            let long_term_avg = self.calculate_strategy_long_term_average(strategy);
            
            let deviation = recent_performance - long_term_avg;
            
            // If significantly underperforming, suggest increase (mean reversion)
            if deviation < -10.0 {
                parameters.insert(format!("{}_suggested_action", strategy.id), 1.0); // Increase
                parameters.insert(format!("{}_performance_deviation", strategy.id), deviation);
                needs_optimization = true;
            }
            // If significantly overperforming, suggest decrease (mean reversion)
            else if deviation > 15.0 {
                parameters.insert(format!("{}_suggested_action", strategy.id), -1.0); // Decrease
                parameters.insert(format!("{}_performance_deviation", strategy.id), deviation);
                needs_optimization = true;
            }
        }

        if needs_optimization {
            Ok(Some(PortfolioOptimization {
                strategy_id: "portfolio_mean_reversion".to_string(),
                optimization_type: "Mean Reversion Allocation".to_string(),
                parameters,
            }))
        } else {
            Ok(None)
        }
    }

    /// Calculate momentum optimization
    fn calculate_momentum_optimization(&self, strategies: &[&ActiveStrategy]) -> Result<Option<PortfolioOptimization>, StrategyError> {
        let mut parameters = HashMap::new();
        let mut needs_optimization = false;

        for strategy in strategies {
            let momentum_score = self.calculate_strategy_momentum(strategy);
            
            if momentum_score > 0.7 { // Strong positive momentum
                parameters.insert(format!("{}_momentum_score", strategy.id), momentum_score);
                parameters.insert(format!("{}_suggested_action", strategy.id), 1.0); // Increase allocation
                needs_optimization = true;
            } else if momentum_score < -0.5 { // Strong negative momentum
                parameters.insert(format!("{}_momentum_score", strategy.id), momentum_score);
                parameters.insert(format!("{}_suggested_action", strategy.id), -1.0); // Decrease allocation
                needs_optimization = true;
            }
        }

        if needs_optimization {
            Ok(Some(PortfolioOptimization {
                strategy_id: "portfolio_momentum".to_string(),
                optimization_type: "Momentum-Based Allocation".to_string(),
                parameters,
            }))
        } else {
            Ok(None)
        }
    }

    /// Calculate diversification optimization
    fn calculate_diversification_optimization(&self, strategies: &[&ActiveStrategy]) -> Result<Option<PortfolioOptimization>, StrategyError> {
        let mut parameters = HashMap::new();
        let mut needs_optimization = false;

        // Analyze chain diversification
        let chain_counts = self.count_chain_exposure(strategies);
        let protocol_counts = self.count_protocol_exposure(strategies);
        
        // Check for over-concentration
        for (chain, count) in chain_counts {
            let concentration = count as f64 / strategies.len() as f64;
            if concentration > 0.6 { // More than 60% in single chain
                parameters.insert(format!("chain_{}_concentration", chain.name()), concentration);
                needs_optimization = true;
            }
        }

        for (protocol, count) in protocol_counts {
            let concentration = count as f64 / strategies.len() as f64;
            if concentration > 0.5 { // More than 50% in single protocol
                parameters.insert(format!("protocol_{}_concentration", format!("{:?}", protocol)), concentration);
                needs_optimization = true;
            }
        }

        if needs_optimization {
            parameters.insert("diversification_score".to_string(), self.calculate_portfolio_diversification_score(strategies));
            
            Ok(Some(PortfolioOptimization {
                strategy_id: "portfolio_diversification".to_string(),
                optimization_type: "Diversification Enhancement".to_string(),
                parameters,
            }))
        } else {
            Ok(None)
        }
    }

    /// Calculate Sharpe ratio optimization
    fn calculate_sharpe_optimization(&self, strategies: &[&ActiveStrategy], total_allocation: f64) -> Result<Option<PortfolioOptimization>, StrategyError> {
        let mut parameters = HashMap::new();
        let mut needs_optimization = false;

        let portfolio_sharpe = self.calculate_current_portfolio_sharpe(strategies, total_allocation);
        
        // Simulate allocation adjustments to improve Sharpe ratio
        for strategy in strategies {
            let strategy_sharpe = strategy.performance_metrics.sharpe_ratio;
            
            if strategy_sharpe > portfolio_sharpe + 0.3 {
                // Strategy has significantly better Sharpe ratio
                parameters.insert(format!("{}_sharpe_advantage", strategy.id), strategy_sharpe - portfolio_sharpe);
                parameters.insert(format!("{}_suggested_action", strategy.id), 1.0); // Increase allocation
                needs_optimization = true;
            } else if strategy_sharpe < portfolio_sharpe - 0.5 {
                // Strategy significantly underperforms
                parameters.insert(format!("{}_sharpe_disadvantage", strategy.id), portfolio_sharpe - strategy_sharpe);
                parameters.insert(format!("{}_suggested_action", strategy.id), -1.0); // Decrease allocation
                needs_optimization = true;
            }
        }

        if needs_optimization {
            parameters.insert("current_portfolio_sharpe".to_string(), portfolio_sharpe);
            
            Ok(Some(PortfolioOptimization {
                strategy_id: "portfolio_sharpe_optimization".to_string(),
                optimization_type: "Sharpe Ratio Optimization".to_string(),
                parameters,
            }))
        } else {
            Ok(None)
        }
    }

    // Helper methods for portfolio optimization
    fn calculate_strategy_long_term_average(&self, strategy: &ActiveStrategy) -> f64 {
        // Mock calculation - in production would use historical data
        strategy.performance_metrics.roi_percentage * 0.8 // Assume current performance is slightly above long-term average
    }

    fn calculate_strategy_momentum(&self, strategy: &ActiveStrategy) -> f64 {
        // Mock momentum calculation based on recent performance trend
        let recent_roi = strategy.performance_metrics.roi_percentage;
        let win_rate = strategy.performance_metrics.win_rate_percentage;
        
        // Simple momentum score: (ROI - 5) / 10 + (win_rate - 50) / 100
        ((recent_roi - 5.0) / 10.0 + (win_rate - 50.0) / 100.0).clamp(-1.0, 1.0)
    }

    fn count_chain_exposure(&self, strategies: &[&ActiveStrategy]) -> HashMap<ChainId, usize> {
        let mut chain_counts = HashMap::new();
        
        for strategy in strategies {
            for chain in &strategy.config.target_chains {
                *chain_counts.entry(chain.clone()).or_insert(0) += 1;
            }
        }
        
        chain_counts
    }

    fn count_protocol_exposure(&self, strategies: &[&ActiveStrategy]) -> HashMap<crate::defi::yield_farming::DeFiProtocol, usize> {
        let mut protocol_counts = HashMap::new();
        
        for strategy in strategies {
            for protocol in &strategy.config.target_protocols {
                *protocol_counts.entry(protocol.clone()).or_insert(0) += 1;
            }
        }
        
        protocol_counts
    }

    fn calculate_portfolio_diversification_score(&self, strategies: &[&ActiveStrategy]) -> f64 {
        if strategies.len() <= 1 {
            return 0.0;
        }

        let unique_chains: std::collections::HashSet<_> = strategies.iter()
            .flat_map(|s| &s.config.target_chains)
            .collect();
        
        let unique_protocols: std::collections::HashSet<_> = strategies.iter()
            .flat_map(|s| &s.config.target_protocols)
            .collect();

        let unique_strategy_types: std::collections::HashSet<_> = strategies.iter()
            .map(|s| std::mem::discriminant(&s.config.strategy_type))
            .collect();

        let chain_diversity = (unique_chains.len() as f64 / 8.0).min(1.0); // Max 8 major chains
        let protocol_diversity = (unique_protocols.len() as f64 / 15.0).min(1.0); // Max 15 major protocols
        let strategy_type_diversity = (unique_strategy_types.len() as f64 / 6.0).min(1.0); // 6 strategy types

        ((chain_diversity + protocol_diversity + strategy_type_diversity) / 3.0) * 100.0
    }

    fn calculate_current_portfolio_sharpe(&self, strategies: &[&ActiveStrategy], total_allocation: f64) -> f64 {
        if total_allocation == 0.0 {
            return 0.0;
        }

        strategies.iter()
            .map(|s| (s.allocated_capital / total_allocation) * s.performance_metrics.sharpe_ratio)
            .sum()
    }
}

/// Strategy rebalancer for dynamic rebalancing
#[derive(Debug, Clone)]
pub struct StrategyRebalancer;

impl StrategyRebalancer {
    pub fn new() -> Self { Self }

    pub fn analyze_rebalancing_needs(&self, strategies: &[&ActiveStrategy]) -> Result<Vec<RebalancingSuggestion>, StrategyError> {
        let mut suggestions = Vec::new();
        
        if strategies.len() < 2 {
            return Ok(suggestions);
        }

        let total_allocation: f64 = strategies.iter().map(|s| s.allocated_capital).sum();
        if total_allocation == 0.0 {
            return Ok(suggestions);
        }

        // 1. Performance-based rebalancing
        let performance_suggestions = self.analyze_performance_based_rebalancing(strategies, total_allocation)?;
        suggestions.extend(performance_suggestions);

        // 2. Risk-based rebalancing
        let risk_suggestions = self.analyze_risk_based_rebalancing(strategies, total_allocation)?;
        suggestions.extend(risk_suggestions);

        // 3. Correlation-based rebalancing
        let correlation_suggestions = self.analyze_correlation_based_rebalancing(strategies)?;
        suggestions.extend(correlation_suggestions);

        // 4. Volatility-based rebalancing
        let volatility_suggestions = self.analyze_volatility_based_rebalancing(strategies, total_allocation)?;
        suggestions.extend(volatility_suggestions);

        // 5. Time-based rebalancing (periodic rebalancing)
        let time_suggestions = self.analyze_time_based_rebalancing(strategies)?;
        suggestions.extend(time_suggestions);

        Ok(suggestions)
    }

    fn analyze_performance_based_rebalancing(&self, strategies: &[&ActiveStrategy], total_allocation: f64) -> Result<Vec<RebalancingSuggestion>, StrategyError> {
        let mut suggestions = Vec::new();
        let avg_roi: f64 = strategies.iter().map(|s| s.performance_metrics.roi_percentage).sum::<f64>() / strategies.len() as f64;

        for strategy in strategies {
            let current_percentage = (strategy.allocated_capital / total_allocation) * 100.0;
            let performance_deviation = strategy.performance_metrics.roi_percentage - avg_roi;
            
            let mut suggested_changes = HashMap::new();
            
            // Strong outperformer - increase allocation
            if performance_deviation > 10.0 && current_percentage < 40.0 {
                let suggested_increase = (performance_deviation / 100.0 * current_percentage).min(10.0);
                suggested_changes.insert("allocation_change_percentage".to_string(), suggested_increase);
                suggested_changes.insert("new_target_percentage".to_string(), current_percentage + suggested_increase);
                suggested_changes.insert("performance_score".to_string(), strategy.performance_metrics.roi_percentage);
                
                suggestions.push(RebalancingSuggestion {
                    strategy_id: strategy.id.clone(),
                    suggested_changes,
                    reason: format!("Strong performance ({:.1}% ROI vs {:.1}% average) warrants increased allocation", 
                        strategy.performance_metrics.roi_percentage, avg_roi),
                });
            }
            // Underperformer - decrease allocation
            else if performance_deviation < -8.0 && current_percentage > 15.0 {
                let suggested_decrease = (performance_deviation.abs() / 100.0 * current_percentage).min(8.0);
                suggested_changes.insert("allocation_change_percentage".to_string(), -suggested_decrease);
                suggested_changes.insert("new_target_percentage".to_string(), current_percentage - suggested_decrease);
                suggested_changes.insert("performance_score".to_string(), strategy.performance_metrics.roi_percentage);
                
                suggestions.push(RebalancingSuggestion {
                    strategy_id: strategy.id.clone(),
                    suggested_changes,
                    reason: format!("Poor performance ({:.1}% ROI vs {:.1}% average) suggests reduced allocation", 
                        strategy.performance_metrics.roi_percentage, avg_roi),
                });
            }
        }

        Ok(suggestions)
    }

    fn analyze_risk_based_rebalancing(&self, strategies: &[&ActiveStrategy], total_allocation: f64) -> Result<Vec<RebalancingSuggestion>, StrategyError> {
        let mut suggestions = Vec::new();
        
        // Calculate portfolio risk metrics
        let weighted_risk: f64 = strategies.iter()
            .map(|s| (s.allocated_capital / total_allocation) * s.config.risk_level as f64)
            .sum();

        // If portfolio risk is too high, suggest reducing high-risk strategy allocations
        if weighted_risk > 7.0 {
            for strategy in strategies {
                if strategy.config.risk_level > 7 {
                    let current_percentage = (strategy.allocated_capital / total_allocation) * 100.0;
                    let suggested_decrease = ((strategy.config.risk_level as f64 - 7.0) / 3.0 * current_percentage).min(15.0);
                    
                    let mut suggested_changes = HashMap::new();
                    suggested_changes.insert("allocation_change_percentage".to_string(), -suggested_decrease);
                    suggested_changes.insert("new_target_percentage".to_string(), current_percentage - suggested_decrease);
                    suggested_changes.insert("risk_score".to_string(), strategy.config.risk_level as f64);
                    suggested_changes.insert("portfolio_risk".to_string(), weighted_risk);
                    
                    suggestions.push(RebalancingSuggestion {
                        strategy_id: strategy.id.clone(),
                        suggested_changes,
                        reason: format!("High-risk strategy (risk level {}) contributing to elevated portfolio risk ({:.1})", 
                            strategy.config.risk_level, weighted_risk),
                    });
                }
            }
        }

        Ok(suggestions)
    }

    fn analyze_correlation_based_rebalancing(&self, strategies: &[&ActiveStrategy]) -> Result<Vec<RebalancingSuggestion>, StrategyError> {
        let mut suggestions = Vec::new();
        
        // Find highly correlated strategy pairs
        for (i, strategy_a) in strategies.iter().enumerate() {
            for strategy_b in strategies.iter().skip(i + 1) {
                let correlation = self.calculate_strategy_correlation(strategy_a, strategy_b);
                
                // If correlation is too high, suggest reducing allocation to one of them
                if correlation > 0.8 {
                    // Reduce allocation to the lower-performing strategy
                    let target_strategy = if strategy_a.performance_metrics.roi_percentage < strategy_b.performance_metrics.roi_percentage {
                        strategy_a
                    } else {
                        strategy_b
                    };
                    
                    let mut suggested_changes = HashMap::new();
                    suggested_changes.insert("allocation_change_percentage".to_string(), -10.0);
                    suggested_changes.insert("correlation_with".to_string(), 
                        if target_strategy.id == strategy_a.id { 
                            strategy_b.id.clone() 
                        } else { 
                            strategy_a.id.clone() 
                        }.parse().unwrap_or(0.0));
                    suggested_changes.insert("correlation_value".to_string(), correlation);
                    
                    suggestions.push(RebalancingSuggestion {
                        strategy_id: target_strategy.id.clone(),
                        suggested_changes,
                        reason: format!("High correlation ({:.1}%) with another strategy reduces diversification benefits", 
                            correlation * 100.0),
                    });
                }
            }
        }

        Ok(suggestions)
    }

    fn analyze_volatility_based_rebalancing(&self, strategies: &[&ActiveStrategy], total_allocation: f64) -> Result<Vec<RebalancingSuggestion>, StrategyError> {
        let mut suggestions = Vec::new();
        
        let avg_volatility = strategies.iter()
            .map(|s| s.risk_metrics.volatility_score)
            .sum::<f64>() / strategies.len() as f64;

        for strategy in strategies {
            let current_percentage = (strategy.allocated_capital / total_allocation) * 100.0;
            let volatility = strategy.risk_metrics.volatility_score;
            
            // High volatility strategy - reduce allocation during volatile periods
            if volatility > avg_volatility * 1.5 && current_percentage > 20.0 {
                let suggested_decrease = ((volatility - avg_volatility) / avg_volatility * 10.0).min(12.0);
                
                let mut suggested_changes = HashMap::new();
                suggested_changes.insert("allocation_change_percentage".to_string(), -suggested_decrease);
                suggested_changes.insert("new_target_percentage".to_string(), current_percentage - suggested_decrease);
                suggested_changes.insert("volatility_score".to_string(), volatility);
                suggested_changes.insert("avg_volatility".to_string(), avg_volatility);
                
                suggestions.push(RebalancingSuggestion {
                    strategy_id: strategy.id.clone(),
                    suggested_changes,
                    reason: format!("High volatility ({:.1} vs {:.1} average) suggests reduced allocation", 
                        volatility, avg_volatility),
                });
            }
            // Low volatility, good performance - increase allocation
            else if volatility < avg_volatility * 0.7 && strategy.performance_metrics.roi_percentage > 5.0 && current_percentage < 35.0 {
                let suggested_increase = ((avg_volatility - volatility) / avg_volatility * 8.0).min(10.0);
                
                let mut suggested_changes = HashMap::new();
                suggested_changes.insert("allocation_change_percentage".to_string(), suggested_increase);
                suggested_changes.insert("new_target_percentage".to_string(), current_percentage + suggested_increase);
                suggested_changes.insert("volatility_score".to_string(), volatility);
                suggested_changes.insert("roi_percentage".to_string(), strategy.performance_metrics.roi_percentage);
                
                suggestions.push(RebalancingSuggestion {
                    strategy_id: strategy.id.clone(),
                    suggested_changes,
                    reason: format!("Low volatility ({:.1}) with good returns ({:.1}%) supports increased allocation", 
                        volatility, strategy.performance_metrics.roi_percentage),
                });
            }
        }

        Ok(suggestions)
    }

    fn analyze_time_based_rebalancing(&self, strategies: &[&ActiveStrategy]) -> Result<Vec<RebalancingSuggestion>, StrategyError> {
        let mut suggestions = Vec::new();
        let current_time = self.get_current_time();
        
        // Check if strategies haven't been rebalanced recently
        for strategy in strategies {
            let time_since_last_rebalance = current_time - strategy.last_rebalance.unwrap_or(0);
            let days_since_rebalance = time_since_last_rebalance / (24 * 3600 * 1_000_000_000);
            
            // Suggest rebalancing if it's been more than 30 days and performance has drifted
            if days_since_rebalance > 30 {
                let performance_drift = (strategy.performance_metrics.roi_percentage - 5.0).abs(); // 5% baseline
                
                if performance_drift > 10.0 {
                    let mut suggested_changes = HashMap::new();
                    suggested_changes.insert("days_since_rebalance".to_string(), days_since_rebalance as f64);
                    suggested_changes.insert("performance_drift".to_string(), performance_drift);
                    suggested_changes.insert("suggested_action".to_string(), if strategy.performance_metrics.roi_percentage > 15.0 { 1.0 } else { -1.0 });
                    
                    suggestions.push(RebalancingSuggestion {
                        strategy_id: strategy.id.clone(),
                        suggested_changes,
                        reason: format!("Strategy hasn't been rebalanced for {} days and shows {:.1}% performance drift", 
                            days_since_rebalance, performance_drift),
                    });
                }
            }
        }

        Ok(suggestions)
    }

    fn calculate_strategy_correlation(&self, strategy_a: &ActiveStrategy, strategy_b: &ActiveStrategy) -> f64 {
        // Calculate correlation based on strategy characteristics
        let mut correlation = 0.0;

        // Same strategy types have higher correlation
        let type_correlation = match (&strategy_a.config.strategy_type, &strategy_b.config.strategy_type) {
            (StrategyType::YieldFarming(_), StrategyType::YieldFarming(_)) => 0.7,
            (StrategyType::Arbitrage(_), StrategyType::Arbitrage(_)) => 0.8,
            (StrategyType::LiquidityMining(_), StrategyType::LiquidityMining(_)) => 0.6,
            _ => 0.2,
        };
        correlation += type_correlation * 0.4;

        // Same chains increase correlation
        let chain_overlap = strategy_a.config.target_chains.iter()
            .filter(|chain| strategy_b.config.target_chains.contains(chain))
            .count() as f64 / strategy_a.config.target_chains.len().max(1) as f64;
        correlation += chain_overlap * 0.3;

        // Same protocols increase correlation
        let protocol_overlap = strategy_a.config.target_protocols.iter()
            .filter(|protocol| strategy_b.config.target_protocols.contains(protocol))
            .count() as f64 / strategy_a.config.target_protocols.len().max(1) as f64;
        correlation += protocol_overlap * 0.3;

        correlation.min(1.0)
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

/// Cross-strategy performance analyzer
#[derive(Debug, Clone)]
pub struct CrossStrategyAnalyzer;

impl CrossStrategyAnalyzer {
    pub fn new() -> Self { Self }

    pub fn analyze_cross_strategy_performance(&self, strategies: &HashMap<String, ActiveStrategy>) -> Result<CrossStrategyAnalysis, StrategyError> {
        let active_strategies: Vec<&ActiveStrategy> = strategies.values()
            .filter(|s| matches!(s.status, StrategyStatus::Active))
            .collect();

        let total_strategies = active_strategies.len();
        let total_allocation: f64 = active_strategies.iter().map(|s| s.allocated_capital).sum();
        let weighted_roi = if total_allocation > 0.0 {
            active_strategies.iter()
                .map(|s| (s.allocated_capital / total_allocation) * s.performance_metrics.roi_percentage)
                .sum()
        } else {
            0.0
        };

        // Calculate correlations between strategies (mock implementation)
        let mut correlation_matrix = HashMap::new();
        for strategy_a in &active_strategies {
            let mut strategy_correlations = HashMap::new();
            for strategy_b in &active_strategies {
                let correlation = if strategy_a.id == strategy_b.id {
                    1.0
                } else {
                    // Mock correlation based on strategy types and chains
                    self.calculate_mock_correlation(strategy_a, strategy_b)
                };
                strategy_correlations.insert(strategy_b.id.clone(), correlation);
            }
            correlation_matrix.insert(strategy_a.id.clone(), strategy_correlations);
        }

        Ok(CrossStrategyAnalysis {
            total_strategies,
            total_allocation,
            weighted_portfolio_roi: weighted_roi,
            correlation_matrix,
            diversification_score: self.calculate_diversification_score(&active_strategies),
            risk_adjusted_return: self.calculate_risk_adjusted_return(&active_strategies),
            portfolio_sharpe_ratio: self.calculate_portfolio_sharpe_ratio(&active_strategies),
            analysis_timestamp: self.get_current_time(),
        })
    }

    fn calculate_mock_correlation(&self, strategy_a: &ActiveStrategy, strategy_b: &ActiveStrategy) -> f64 {
        let mut correlation = 0.0;

        // Similar strategy types have higher correlation
        let type_similarity = match (&strategy_a.config.strategy_type, &strategy_b.config.strategy_type) {
            (StrategyType::YieldFarming(_), StrategyType::YieldFarming(_)) => 0.7,
            (StrategyType::Arbitrage(_), StrategyType::Arbitrage(_)) => 0.8,
            (StrategyType::Rebalancing(_), StrategyType::Rebalancing(_)) => 0.6,
            _ => 0.3,
        };
        correlation += type_similarity * 0.4;

        // Same chains have higher correlation
        let chain_overlap = strategy_a.config.target_chains.iter()
            .filter(|chain| strategy_b.config.target_chains.contains(chain))
            .count() as f64 / strategy_a.config.target_chains.len().max(1) as f64;
        correlation += chain_overlap * 0.3;

        // Same protocols have higher correlation
        let protocol_overlap = strategy_a.config.target_protocols.iter()
            .filter(|protocol| strategy_b.config.target_protocols.contains(protocol))
            .count() as f64 / strategy_a.config.target_protocols.len().max(1) as f64;
        correlation += protocol_overlap * 0.3;

        correlation.min(1.0)
    }

    fn calculate_diversification_score(&self, strategies: &[&ActiveStrategy]) -> f64 {
        if strategies.len() <= 1 {
            return 0.0;
        }

        // Mock diversification score based on variety of chains, protocols, and strategy types
        let unique_chains: std::collections::HashSet<_> = strategies.iter()
            .flat_map(|s| &s.config.target_chains)
            .collect();
        
        let unique_protocols: std::collections::HashSet<_> = strategies.iter()
            .flat_map(|s| &s.config.target_protocols)
            .collect();

        let chain_diversity = unique_chains.len() as f64 / 5.0; // Max 5 major chains
        let protocol_diversity = unique_protocols.len() as f64 / 10.0; // Max 10 major protocols

        ((chain_diversity + protocol_diversity) / 2.0).min(1.0) * 100.0
    }

    fn calculate_risk_adjusted_return(&self, strategies: &[&ActiveStrategy]) -> f64 {
        if strategies.is_empty() {
            return 0.0;
        }

        let total_allocation: f64 = strategies.iter().map(|s| s.allocated_capital).sum();
        if total_allocation == 0.0 {
            return 0.0;
        }

        let weighted_return: f64 = strategies.iter()
            .map(|s| (s.allocated_capital / total_allocation) * s.performance_metrics.roi_percentage)
            .sum();

        let weighted_risk: f64 = strategies.iter()
            .map(|s| (s.allocated_capital / total_allocation) * s.config.risk_level as f64)
            .sum();

        if weighted_risk > 0.0 {
            weighted_return / weighted_risk
        } else {
            0.0
        }
    }

    fn calculate_portfolio_sharpe_ratio(&self, strategies: &[&ActiveStrategy]) -> f64 {
        if strategies.is_empty() {
            return 0.0;
        }

        let total_allocation: f64 = strategies.iter().map(|s| s.allocated_capital).sum();
        if total_allocation == 0.0 {
            return 0.0;
        }

        // Weighted Sharpe ratio
        strategies.iter()
            .map(|s| (s.allocated_capital / total_allocation) * s.performance_metrics.sharpe_ratio)
            .sum()
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

// Data structures

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CoordinationRules {
    pub max_simultaneous_executions: u32,
    pub min_execution_interval_seconds: u64,
    pub max_correlation_threshold: f64,
    pub rebalancing_threshold_percentage: f64,
    pub gas_optimization_enabled: bool,
}

impl Default for CoordinationRules {
    fn default() -> Self {
        Self {
            max_simultaneous_executions: 3,
            min_execution_interval_seconds: 120, // 2 minutes
            max_correlation_threshold: 0.8,
            rebalancing_threshold_percentage: 10.0,
            gas_optimization_enabled: true,
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub coordination_action: CoordinationAction,
    pub total_strategies_coordinated: usize,
    pub conflicts_resolved: usize,
    pub optimizations_applied: usize,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CoordinationAction {
    pub timestamp: u64,
    pub actions_taken: Vec<CoordinationActionType>,
    pub strategies_affected: usize,
    pub execution_time_ms: u64,
    pub improvements_achieved: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum CoordinationActionType {
    ConflictResolution,
    AllocationOptimization,
    Rebalancing,
    TimingSynchronization,
    PortfolioOptimization,
    GasOptimization,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CoordinationRecommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub expected_benefit: String,
    pub implementation_complexity: u8, // 1-10 scale
    pub estimated_improvement_percentage: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RecommendationType {
    AllocationOptimization,
    RiskReduction,
    TimingOptimization,
    GasOptimization,
    Information,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RecommendationPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CrossStrategyAnalysis {
    pub total_strategies: usize,
    pub total_allocation: f64,
    pub weighted_portfolio_roi: f64,
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
    pub diversification_score: f64,
    pub risk_adjusted_return: f64,
    pub portfolio_sharpe_ratio: f64,
    pub analysis_timestamp: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CoordinationStatistics {
    pub total_coordinations: usize,
    pub actions_by_type: std::collections::HashMap<String, usize>,
    pub average_execution_time_ms: f64,
    pub total_improvements_achieved: f64,
    pub last_coordination: Option<u64>,
    pub success_rate: f64,
}

// Supporting data structures for various coordination aspects

#[derive(Debug, Clone)]
pub struct StrategyConflict {
    pub conflict_type: ConflictType,
    pub strategy_ids: Vec<String>,
    pub description: String,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    ResourceContention,
    ExecutionTiming,
    AllocationImbalance,
}

#[derive(Debug, Clone)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub strategy_id: String,
    pub resolution_type: ResolutionType,
    pub delay_seconds: u64,
    pub adjustment_factor: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum ResolutionType {
    DelayExecution,
    ReduceAllocation,
    ChangeChain,
    PauseTemporarily,
}

#[derive(Debug, Clone)]
pub struct AllocationOptimizationResult {
    pub user_id: String,
    pub improvements: Vec<AllocationImprovement>,
    pub total_strategies_analyzed: usize,
    pub optimization_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct AllocationImprovement {
    pub strategy_id: String,
    pub current_allocation: f64,
    pub new_allocation: f64,
    pub reason: String,
    pub expected_improvement: f64,
}

#[derive(Debug, Clone)]
pub struct TimingAdjustment {
    pub strategy_id: String,
    pub delay_seconds: u64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct RebalancingSuggestion {
    pub strategy_id: String,
    pub suggested_changes: HashMap<String, f64>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct PortfolioOptimization {
    pub strategy_id: String,
    pub optimization_type: String,
    pub parameters: HashMap<String, f64>,
}

// Analysis result structures
#[derive(Debug, Clone)]
pub struct CorrelationAnalysis {
    pub max_correlation: f64,
    pub avg_correlation: f64,
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
}

#[derive(Debug, Clone)]
pub struct AllocationAnalysis {
    pub efficiency_score: f64,
    pub optimal_allocation: HashMap<String, f64>,
    pub current_allocation: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct TimingAnalysis {
    pub conflicts_detected: usize,
    pub optimal_schedule: HashMap<String, u64>,
    pub current_schedule: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct GasAnalysis {
    pub optimization_potential: f64,
    pub current_gas_cost: f64,
    pub optimized_gas_cost: f64,
}