// Cross-Chain Yield Optimizer
// Day 11: Advanced DeFi - Optimize yield across Bitcoin, Ethereum ecosystem, and Solana

use super::yield_farming::*;
use super::yield_engine::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::api::time;

/// Cross-chain yield optimizer orchestrating capital across multiple blockchains
#[derive(Debug, Clone)]
pub struct CrossChainYieldOptimizer {
    pub chain_optimizers: HashMap<ChainId, ChainSpecificOptimizer>,
    pub global_allocation: GlobalAllocationStrategy,
    pub cross_chain_bridges: Vec<CrossChainBridge>,
    pub gas_optimization: GasOptimizationEngine,
    pub portfolio_tracker: CrossChainPortfolioTracker,
    pub rebalancing_engine: CrossChainRebalancingEngine,
    pub risk_manager: CrossChainRiskManager,
    pub performance_monitor: PerformanceMonitor,
    pub last_optimization: u64,
}

impl CrossChainYieldOptimizer {
    pub fn new(global_strategy: GlobalAllocationStrategy) -> Self {
        Self {
            chain_optimizers: HashMap::new(),
            global_allocation: global_strategy,
            cross_chain_bridges: Vec::new(),
            gas_optimization: GasOptimizationEngine::new(),
            portfolio_tracker: CrossChainPortfolioTracker::new(),
            rebalancing_engine: CrossChainRebalancingEngine::new(),
            risk_manager: CrossChainRiskManager::new(),
            performance_monitor: PerformanceMonitor::new(),
            last_optimization: time(),
        }
    }

    /// Initialize chain-specific optimizers
    pub fn initialize_chain_optimizers(&mut self) {
        let supported_chains = vec![
            ChainId::Bitcoin,
            ChainId::Ethereum,
            ChainId::Arbitrum,  
            ChainId::Optimism,
            ChainId::Polygon,
            ChainId::Base,
            ChainId::Avalanche,
            ChainId::Sonic,
            ChainId::BSC,
            ChainId::Solana,
        ];

        for chain in supported_chains {
            let optimizer = ChainSpecificOptimizer::new(chain.clone());
            self.chain_optimizers.insert(chain, optimizer);
        }
    }

    /// Add cross-chain bridge for capital movement
    pub fn add_bridge(&mut self, bridge: CrossChainBridge) {
        self.cross_chain_bridges.push(bridge);
    }

    /// Optimize yield allocation across all chains
    pub async fn optimize_cross_chain_allocation(&mut self, total_capital_usd: u64) -> Result<CrossChainAllocationPlan, CrossChainOptimizationError> {
        // Step 1: Collect opportunities from all chains
        let mut all_opportunities = Vec::new();
        let mut chain_capacities = HashMap::new();

        for (chain_id, optimizer) in &self.chain_optimizers {
            let opportunities = optimizer.get_opportunities(total_capital_usd).await?;
            let capacity = optimizer.calculate_total_capacity();
            
            all_opportunities.extend(opportunities);
            chain_capacities.insert(chain_id.clone(), capacity);
        }

        // Step 2: Apply global allocation strategy
        let filtered_opportunities = self.apply_global_filters(&all_opportunities)?;

        // Step 3: Optimize allocation considering cross-chain constraints
        let allocation_plan = self.create_optimal_allocation(
            filtered_opportunities,
            total_capital_usd,
            &chain_capacities,
        )?;

        // Step 4: Optimize execution order and routing
        let optimized_plan = self.optimize_execution_routing(allocation_plan).await?;

        // Step 5: Validate risk constraints
        self.risk_manager.validate_allocation(&optimized_plan)?;

        // Step 6: Update tracking
        self.portfolio_tracker.update_planned_allocation(&optimized_plan);
        self.last_optimization = time();

        Ok(optimized_plan)
    }

    /// Apply global allocation filters and constraints
    fn apply_global_filters(&self, opportunities: &[YieldOpportunity]) -> Result<Vec<YieldOpportunity>, CrossChainOptimizationError> {
        let mut filtered = opportunities.to_vec();

        // Apply global risk limits
        filtered.retain(|opp| opp.risk_score <= self.global_allocation.max_risk_score);

        // Apply minimum yield threshold
        filtered.retain(|opp| opp.current_apy >= self.global_allocation.min_yield_threshold);

        // Apply chain allocation limits
        let mut chain_counts = HashMap::new();
        filtered.retain(|opp| {
            let count = chain_counts.entry(opp.chain.clone()).or_insert(0);
            *count += 1;
            *count <= self.global_allocation.max_strategies_per_chain
        });

        // Apply protocol diversification limits  
        let mut protocol_counts = HashMap::new();
        filtered.retain(|opp| {
            let count = protocol_counts.entry(opp.protocol.clone()).or_insert(0);
            *count += 1;
            *count <= self.global_allocation.max_strategies_per_protocol
        });

        if filtered.is_empty() {
            return Err(CrossChainOptimizationError::NoViableOpportunities);
        }

        Ok(filtered)
    }

    /// Create optimal allocation across filtered opportunities
    fn create_optimal_allocation(
        &self,
        opportunities: Vec<YieldOpportunity>,
        total_capital: u64,
        chain_capacities: &HashMap<ChainId, u64>,
    ) -> Result<CrossChainAllocationPlan, CrossChainOptimizationError> {
        
        // Sort opportunities by risk-adjusted score
        let mut sorted_opportunities = opportunities;
        sorted_opportunities.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).unwrap());

        let mut allocations = Vec::new();
        let mut remaining_capital = total_capital as f64;
        let mut chain_allocations: HashMap<ChainId, f64> = HashMap::new();
        let mut protocol_allocations: HashMap<DeFiProtocol, f64> = HashMap::new();

        // Track cross-chain distribution
        for opportunity in sorted_opportunities {
            if remaining_capital < opportunity.min_deposit_usd as f64 {
                continue;
            }

            // Calculate maximum allocation based on various constraints
            let max_by_global_chain = (total_capital as f64 * self.global_allocation.max_chain_allocation)
                - chain_allocations.get(&opportunity.chain).unwrap_or(&0.0);

            let max_by_global_protocol = (total_capital as f64 * self.global_allocation.max_protocol_allocation)  
                - protocol_allocations.get(&opportunity.protocol).unwrap_or(&0.0);

            let max_by_liquidity = opportunity.liquidity_usd as f64 * 0.1; // Max 10% of liquidity

            let max_by_capacity = *chain_capacities.get(&opportunity.chain).unwrap_or(&0) as f64;

            let max_by_single_strategy = total_capital as f64 * 0.2; // Max 20% in single strategy

            let max_allocation = [
                remaining_capital,
                max_by_global_chain,
                max_by_global_protocol,
                max_by_liquidity,
                max_by_capacity,
                max_by_single_strategy,
                opportunity.max_deposit_usd.unwrap_or(u64::MAX) as f64,
            ].iter().fold(f64::INFINITY, |a, &b| a.min(b));

            if max_allocation >= opportunity.min_deposit_usd as f64 {
                // Use Kelly criterion-inspired sizing based on expected return and risk
                let kelly_fraction = self.calculate_kelly_fraction(&opportunity);
                let kelly_size = (total_capital as f64 * kelly_fraction).min(max_allocation);
                
                let final_allocation = kelly_size.max(opportunity.min_deposit_usd as f64);

                let bridge_cost = self.calculate_bridge_cost(&opportunity.chain, final_allocation as u64);
                
                allocations.push(CrossChainAllocation {
                    opportunity: opportunity.clone(),
                    allocated_amount_usd: final_allocation as u64,
                    bridge_cost_usd: bridge_cost,
                    execution_priority: allocations.len() + 1,
                    estimated_deployment_time: self.estimate_deployment_time(&opportunity.chain),
                });

                remaining_capital -= final_allocation;
                *chain_allocations.entry(opportunity.chain.clone()).or_insert(0.0) += final_allocation;
                *protocol_allocations.entry(opportunity.protocol.clone()).or_insert(0.0) += final_allocation;

                if remaining_capital < 100.0 { // Stop if less than $100 remaining
                    break;
                }
            }
        }

        if allocations.is_empty() {
            return Err(CrossChainOptimizationError::AllocationOptimizationFailed("No viable allocations found".to_string()));
        }

        let total_allocated: u64 = allocations.iter().map(|a| a.allocated_amount_usd).sum();
        let total_bridge_costs: f64 = allocations.iter().map(|a| a.bridge_cost_usd).sum();
        
        let weighted_apy = allocations
            .iter()
            .map(|a| a.opportunity.expected_apy * (a.allocated_amount_usd as f64 / total_allocated as f64))
            .sum();

        let plan_allocations = allocations.clone();
        Ok(CrossChainAllocationPlan {
            allocations,
            total_capital,
            total_allocated,
            unallocated_amount: total_capital - total_allocated,
            total_bridge_costs,
            expected_weighted_apy: weighted_apy,
            estimated_total_gas_costs: self.estimate_total_gas_costs(&plan_allocations),
            diversification_metrics: self.calculate_diversification_metrics(&plan_allocations),
            risk_metrics: self.calculate_cross_chain_risk_metrics(&plan_allocations),
            execution_timeline: self.create_execution_timeline(&plan_allocations),
            created_at: time(),
        })
    }

    /// Calculate Kelly fraction for position sizing
    fn calculate_kelly_fraction(&self, opportunity: &YieldOpportunity) -> f64 {
        // Simplified Kelly criterion: f = (bp - q) / b
        // where b = odds (expected return), p = probability of success, q = probability of failure
        
        let expected_return = opportunity.expected_apy / 100.0;
        let win_probability = opportunity.confidence_level;
        let loss_probability = 1.0 - win_probability;
        
        // Conservative Kelly fraction with maximum cap
        let kelly = (expected_return * win_probability - loss_probability) / expected_return;
        kelly.max(0.0).min(0.1) // Cap at 10% for safety
    }

    /// Calculate bridge cost for moving capital to target chain
    fn calculate_bridge_cost(&self, target_chain: &ChainId, amount_usd: u64) -> f64 {
        let base_cost = match target_chain {
            ChainId::Ethereum => 25.0,     // High bridge costs to Ethereum
            ChainId::Bitcoin => 15.0,      // Moderate bridge costs
            ChainId::Arbitrum | ChainId::Optimism => 5.0, // L2 bridges
            ChainId::Polygon | ChainId::BSC => 3.0,       // Cheaper bridges
            ChainId::Solana => 8.0,        // Wormhole/other bridges
            _ => 10.0,                     // Default bridge cost
        };

        // Variable cost component (percentage of amount)
        let variable_rate = match target_chain {
            ChainId::Bitcoin => 0.001,     // 0.1%
            ChainId::Ethereum => 0.002,    // 0.2%  
            ChainId::Solana => 0.0015,     // 0.15%
            _ => 0.001,                    // 0.1% default
        };

        base_cost + (amount_usd as f64 * variable_rate)
    }

    /// Estimate deployment time for each chain
    fn estimate_deployment_time(&self, chain: &ChainId) -> u64 {
        match chain {
            ChainId::Bitcoin => 1800,      // 30 minutes (confirmation time)
            ChainId::Ethereum => 300,      // 5 minutes
            ChainId::Arbitrum | ChainId::Optimism => 120, // 2 minutes
            ChainId::Polygon => 60,        // 1 minute
            ChainId::Solana => 30,         // 30 seconds
            ChainId::BSC => 90,            // 1.5 minutes
            _ => 300,                      // 5 minutes default
        }
    }

    /// Optimize execution routing and order
    async fn optimize_execution_routing(&self, mut plan: CrossChainAllocationPlan) -> Result<CrossChainAllocationPlan, CrossChainOptimizationError> {
        // Sort by execution priority considering gas costs and timing
        plan.allocations.sort_by(|a, b| {
            let a_score = a.opportunity.overall_score - (a.bridge_cost_usd / a.allocated_amount_usd as f64);
            let b_score = b.opportunity.overall_score - (b.bridge_cost_usd / b.allocated_amount_usd as f64);
            b_score.partial_cmp(&a_score).unwrap()
        });

        // Update execution priorities
        for (i, allocation) in plan.allocations.iter_mut().enumerate() {
            allocation.execution_priority = i + 1;
        }

        // Recalculate execution timeline
        plan.execution_timeline = self.create_execution_timeline(&plan.allocations);

        Ok(plan)
    }

    /// Estimate total gas costs across all chains
    fn estimate_total_gas_costs(&self, allocations: &[CrossChainAllocation]) -> f64 {
        allocations
            .iter()
            .map(|allocation| {
                match allocation.opportunity.chain {
                    ChainId::Ethereum => 60.0,      // High gas costs
                    ChainId::Bitcoin => 8.0,        // Transaction fees
                    ChainId::Arbitrum | ChainId::Optimism => 4.0, // L2 efficiency
                    ChainId::Polygon | ChainId::BSC => 1.5,       // Very low costs
                    ChainId::Solana => 0.02,        // Extremely low costs
                    _ => 20.0,                      // Default estimate
                }
            })
            .sum()
    }

    /// Calculate diversification metrics across chains and protocols
    fn calculate_diversification_metrics(&self, allocations: &[CrossChainAllocation]) -> DiversificationMetrics {
        let total_amount: u64 = allocations.iter().map(|a| a.allocated_amount_usd).sum();
        
        // Chain diversification
        let mut chain_allocations = HashMap::new();
        for allocation in allocations {
            *chain_allocations.entry(allocation.opportunity.chain.clone()).or_insert(0u64) += allocation.allocated_amount_usd;
        }
        
        let chain_herfindahl = chain_allocations
            .values()
            .map(|amount| {
                let share = *amount as f64 / total_amount as f64;
                share * share
            })
            .sum::<f64>();
        
        // Protocol diversification
        let mut protocol_allocations = HashMap::new();
        for allocation in allocations {
            *protocol_allocations.entry(allocation.opportunity.protocol.clone()).or_insert(0u64) += allocation.allocated_amount_usd;
        }
        
        let protocol_herfindahl = protocol_allocations
            .values()
            .map(|amount| {
                let share = *amount as f64 / total_amount as f64;
                share * share
            })
            .sum::<f64>();

        DiversificationMetrics {
            num_chains: chain_allocations.len(),
            num_protocols: protocol_allocations.len(),
            num_strategies: allocations.len(),
            chain_herfindahl_index: chain_herfindahl,
            protocol_herfindahl_index: protocol_herfindahl,
            max_chain_allocation_pct: *chain_allocations.values().max().unwrap_or(&0) as f64 / total_amount as f64 * 100.0,
            max_protocol_allocation_pct: *protocol_allocations.values().max().unwrap_or(&0) as f64 / total_amount as f64 * 100.0,
        }
    }

    /// Calculate cross-chain risk metrics
    fn calculate_cross_chain_risk_metrics(&self, allocations: &[CrossChainAllocation]) -> CrossChainRiskMetrics {
        let total_amount: u64 = allocations.iter().map(|a| a.allocated_amount_usd).sum();
        
        // Weighted average risk score
        let weighted_risk_score = allocations
            .iter()
            .map(|a| a.opportunity.risk_score as f64 * (a.allocated_amount_usd as f64 / total_amount as f64))
            .sum();

        // Bridge risk exposure
        let total_bridge_costs: f64 = allocations.iter().map(|a| a.bridge_cost_usd).sum();
        let bridge_risk_ratio = total_bridge_costs / total_amount as f64;

        // Cross-chain correlation risk
        let correlation_risk = self.estimate_cross_chain_correlation_risk(allocations);

        CrossChainRiskMetrics {
            weighted_risk_score,
            bridge_risk_ratio,
            correlation_risk_score: correlation_risk,
            max_single_chain_exposure: self.calculate_max_chain_exposure(allocations),
            liquidity_risk_score: self.calculate_liquidity_risk_score(allocations),
        }
    }

    /// Estimate cross-chain correlation risk
    fn estimate_cross_chain_correlation_risk(&self, allocations: &[CrossChainAllocation]) -> f64 {
        // This would use historical correlation data in production
        // For now, provide estimates based on chain relationships
        
        let mut correlation_score = 0.0;
        let total_amount: u64 = allocations.iter().map(|a| a.allocated_amount_usd).sum();
        
        let mut chain_weights = HashMap::new();
        for allocation in allocations {
            *chain_weights.entry(allocation.opportunity.chain.clone()).or_insert(0.0) += 
                allocation.allocated_amount_usd as f64 / total_amount as f64;
        }

        // Ethereum ecosystem chains are highly correlated
        let eth_ecosystem_weight: f64 = chain_weights.iter()
            .filter(|(chain, _)| chain.is_ethereum_ecosystem())
            .map(|(_, weight)| *weight)
            .sum();
        
        if eth_ecosystem_weight > 0.7 {
            correlation_score += (eth_ecosystem_weight - 0.7) * 10.0; // Penalty for high correlation
        }

        correlation_score.min(10.0)
    }

    /// Calculate maximum single chain exposure percentage
    fn calculate_max_chain_exposure(&self, allocations: &[CrossChainAllocation]) -> f64 {
        let total_amount: u64 = allocations.iter().map(|a| a.allocated_amount_usd).sum();
        let mut chain_amounts = HashMap::new();
        
        for allocation in allocations {
            *chain_amounts.entry(allocation.opportunity.chain.clone()).or_insert(0u64) += allocation.allocated_amount_usd;
        }
        
        *chain_amounts.values().max().unwrap_or(&0) as f64 / total_amount as f64 * 100.0
    }

    /// Calculate aggregate liquidity risk score
    fn calculate_liquidity_risk_score(&self, allocations: &[CrossChainAllocation]) -> f64 {
        let total_amount: u64 = allocations.iter().map(|a| a.allocated_amount_usd).sum();
        
        allocations
            .iter()
            .map(|a| {
                let weight = a.allocated_amount_usd as f64 / total_amount as f64;
                let liquidity_risk = a.opportunity.risk_metrics.liquidity_risk_score;
                liquidity_risk * weight
            })
            .sum()
    }

    /// Create execution timeline
    fn create_execution_timeline(&self, allocations: &[CrossChainAllocation]) -> ExecutionTimeline {
        let mut total_time = 0u64;
        let mut milestones = Vec::new();
        
        for (i, allocation) in allocations.iter().enumerate() {
            total_time += allocation.estimated_deployment_time;
            
            milestones.push(ExecutionMilestone {
                step: i + 1,
                description: format!("Deploy ${} to {} on {}", 
                    allocation.allocated_amount_usd,
                    format!("{:?}", allocation.opportunity.protocol),
                    allocation.opportunity.chain.name()
                ),
                estimated_completion_time: total_time,
                estimated_gas_cost: allocation.opportunity.gas_cost_estimate,
                bridge_cost: allocation.bridge_cost_usd,
            });
        }

        ExecutionTimeline {
            total_estimated_time: total_time,
            milestones,
            parallel_execution_possible: self.check_parallel_execution_feasibility(allocations),
        }
    }

    /// Check if parallel execution is feasible
    fn check_parallel_execution_feasibility(&self, allocations: &[CrossChainAllocation]) -> bool {
        // Can execute in parallel if chains are different and bridges support it
        let unique_chains: std::collections::HashSet<_> = allocations
            .iter()
            .map(|a| &a.opportunity.chain)
            .collect();
        
        unique_chains.len() > 1 && allocations.len() > 1
    }

    /// Execute cross-chain allocation plan
    pub async fn execute_allocation_plan(
        &mut self,
        plan: CrossChainAllocationPlan,
    ) -> Result<ExecutionResult, CrossChainOptimizationError> {
        let mut execution_results = Vec::new();
        let start_time = time();

        for allocation in plan.allocations {
            let result = self.execute_single_allocation(&allocation).await?;
            execution_results.push(result);
            
            // Update portfolio tracking
            self.portfolio_tracker.update_executed_allocation(&allocation);
        }

        let total_deployed: u64 = execution_results.iter().map(|r| r.deployed_amount_usd).sum();
        let total_gas_spent: f64 = execution_results.iter().map(|r| r.actual_gas_cost).sum();

        Ok(ExecutionResult {
            plan_id: format!("plan_{}", start_time),
            execution_results: execution_results.clone(),
            total_deployed_usd: total_deployed,
            total_gas_spent,
            execution_time: time() - start_time,
            success_rate: execution_results.iter().filter(|r| r.success).count() as f64 / execution_results.len() as f64,
        })
    }

    /// Execute a single allocation
    async fn execute_single_allocation(
        &self,
        allocation: &CrossChainAllocation,
    ) -> Result<SingleExecutionResult, CrossChainOptimizationError> {
        // This would integrate with the actual DeFi services
        // For now, return a mock successful execution
        
        Ok(SingleExecutionResult {
            strategy_id: allocation.opportunity.strategy_id.clone(),
            chain: allocation.opportunity.chain.clone(),
            success: true,
            deployed_amount_usd: allocation.allocated_amount_usd,
            actual_gas_cost: allocation.opportunity.gas_cost_estimate,
            transaction_hash: format!("0x{:x}", time()),
            block_number: Some(1000000),
            execution_time: allocation.estimated_deployment_time,
            error_message: None,
        })
    }
}

/// Chain-specific optimizer for individual blockchains
#[derive(Debug, Clone)]
pub struct ChainSpecificOptimizer {
    pub chain: ChainId,
    pub yield_engine: YieldStrategyEngine,
    pub gas_tracker: GasTracker,
    pub capacity_calculator: CapacityCalculator,
}

impl ChainSpecificOptimizer {
    pub fn new(chain: ChainId) -> Self {
        Self {
            chain: chain.clone(),
            yield_engine: YieldStrategyEngine::new(EvaluationCriteria::default()),
            gas_tracker: GasTracker::new(chain.clone()),
            capacity_calculator: CapacityCalculator::new(chain),
        }
    }

    pub async fn get_opportunities(&self, max_capital: u64) -> Result<Vec<YieldOpportunity>, CrossChainOptimizationError> {
        let opportunities = self.yield_engine.evaluate_yield_opportunities(max_capital);
        Ok(opportunities)
    }

    pub fn calculate_total_capacity(&self) -> u64 {
        self.capacity_calculator.calculate_total_capacity()
    }
}

/// Global allocation strategy configuration
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct GlobalAllocationStrategy {
    pub max_chain_allocation: f64,        // Max % of capital in single chain
    pub max_protocol_allocation: f64,     // Max % of capital in single protocol
    pub max_risk_score: u8,              // Maximum risk score to consider
    pub min_yield_threshold: f64,         // Minimum APY to consider
    pub max_strategies_per_chain: usize,  // Max number of strategies per chain
    pub max_strategies_per_protocol: usize, // Max number of strategies per protocol
    pub rebalancing_threshold: f64,       // Trigger rebalancing when allocation drifts by this %
    pub bridge_cost_threshold: f64,       // Max bridge cost as % of allocation
}

impl Default for GlobalAllocationStrategy {
    fn default() -> Self {
        Self {
            max_chain_allocation: 0.4,     // Max 40% per chain
            max_protocol_allocation: 0.3,  // Max 30% per protocol
            max_risk_score: 7,             // Max risk score 7/10
            min_yield_threshold: 3.0,      // Min 3% APY
            max_strategies_per_chain: 5,   // Max 5 strategies per chain
            max_strategies_per_protocol: 3, // Max 3 strategies per protocol
            rebalancing_threshold: 0.05,   // 5% drift triggers rebalancing
            bridge_cost_threshold: 0.02,   // Max 2% bridge costs
        }
    }
}

/// Cross-chain bridge information
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CrossChainBridge {
    pub name: String,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub supported_assets: Vec<String>,
    pub min_amount_usd: u64,
    pub max_amount_usd: u64,
    pub base_fee_usd: f64,
    pub variable_fee_rate: f64,
    pub estimated_time_minutes: u64,
    pub security_rating: u8, // 1-10
    pub is_active: bool,
}

/// Cross-chain allocation within the optimization plan
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CrossChainAllocation {
    pub opportunity: YieldOpportunity,
    pub allocated_amount_usd: u64,
    pub bridge_cost_usd: f64,
    pub execution_priority: usize,
    pub estimated_deployment_time: u64,
}

/// Complete cross-chain allocation plan
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CrossChainAllocationPlan {
    pub allocations: Vec<CrossChainAllocation>,
    pub total_capital: u64,
    pub total_allocated: u64,
    pub unallocated_amount: u64,
    pub total_bridge_costs: f64,
    pub expected_weighted_apy: f64,
    pub estimated_total_gas_costs: f64,
    pub diversification_metrics: DiversificationMetrics,
    pub risk_metrics: CrossChainRiskMetrics,
    pub execution_timeline: ExecutionTimeline,
    pub created_at: u64,
}

/// Diversification metrics across chains and protocols
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DiversificationMetrics {
    pub num_chains: usize,
    pub num_protocols: usize,
    pub num_strategies: usize,
    pub chain_herfindahl_index: f64,     // Concentration index (lower = more diversified)
    pub protocol_herfindahl_index: f64,  // Concentration index (lower = more diversified)
    pub max_chain_allocation_pct: f64,
    pub max_protocol_allocation_pct: f64,
}

/// Cross-chain risk metrics
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CrossChainRiskMetrics {
    pub weighted_risk_score: f64,
    pub bridge_risk_ratio: f64,          // Bridge costs / total capital
    pub correlation_risk_score: f64,     // 0-10, higher = more correlated
    pub max_single_chain_exposure: f64,  // % of capital in largest chain allocation
    pub liquidity_risk_score: f64,       // Weighted average liquidity risk
}

/// Execution timeline with milestones
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ExecutionTimeline {
    pub total_estimated_time: u64,
    pub milestones: Vec<ExecutionMilestone>,
    pub parallel_execution_possible: bool,
}

/// Individual execution milestone
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ExecutionMilestone {
    pub step: usize,
    pub description: String,
    pub estimated_completion_time: u64,
    pub estimated_gas_cost: f64,
    pub bridge_cost: f64,
}

/// Gas optimization engine
#[derive(Debug, Clone)]
pub struct GasOptimizationEngine {
    pub gas_price_tracker: HashMap<ChainId, f64>,
    pub historical_gas_data: HashMap<ChainId, Vec<GasPriceSnapshot>>,
}

impl GasOptimizationEngine {
    pub fn new() -> Self {
        Self {
            gas_price_tracker: HashMap::new(),
            historical_gas_data: HashMap::new(),
        }
    }
}

/// Gas price snapshot for historical tracking
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct GasPriceSnapshot {
    pub timestamp: u64,
    pub gas_price: f64,
    pub network_congestion: f64, // 0-1, higher = more congested
}

/// Cross-chain portfolio tracker
#[derive(Debug, Clone)]
pub struct CrossChainPortfolioTracker {
    pub active_positions: HashMap<String, PortfolioPosition>,
    pub historical_performance: Vec<PortfolioSnapshot>, 
}

impl CrossChainPortfolioTracker {
    pub fn new() -> Self {
        Self {
            active_positions: HashMap::new(),
            historical_performance: Vec::new(),
        }
    }

    pub fn update_planned_allocation(&mut self, _plan: &CrossChainAllocationPlan) {
        // Update tracking with planned allocation
    }

    pub fn update_executed_allocation(&mut self, _allocation: &CrossChainAllocation) {
        // Update tracking with executed allocation
    }
}

/// Individual portfolio position
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub strategy_id: String,
    pub chain: ChainId,
    pub protocol: DeFiProtocol,
    pub amount_usd: u64,
    pub entry_timestamp: u64,
    pub current_value_usd: u64,
    pub realized_return_usd: f64,
    pub unrealized_return_usd: f64,
}

/// Portfolio snapshot for performance tracking
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub timestamp: u64,
    pub total_value_usd: u64,
    pub total_return_usd: f64,
    pub apy_annualized: f64,
    pub positions_count: usize,
    pub chains_count: usize,
}

/// Cross-chain rebalancing engine
#[derive(Debug, Clone)]
pub struct CrossChainRebalancingEngine {
    pub rebalancing_rules: Vec<RebalancingRule>,
    pub last_rebalance: u64,
}

impl CrossChainRebalancingEngine {
    pub fn new() -> Self {
        Self {
            rebalancing_rules: Vec::new(),
            last_rebalance: time(),
        }
    }
}

/// Rebalancing rule definition
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RebalancingRule {
    pub name: String,
    pub trigger_condition: RebalancingTrigger,
    pub action: RebalancingAction,
    pub is_active: bool,
}

/// Rebalancing trigger conditions
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RebalancingTrigger {
    AllocationDrift(f64),              // Trigger when allocation drifts by %
    APYImprovement(f64),               // Trigger when better opportunity found
    RiskScoreChange(u8),               // Trigger when risk changes
    TimeInterval(u64),                 // Trigger after time interval
    MarketVolatility(f64),             // Trigger during high volatility
}

/// Rebalancing action to take
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RebalancingAction {
    Reallocate,                        // Move capital to better opportunities
    ReduceRisk,                        // Move to lower risk strategies
    TakeProfit,                        // Realize gains
    StopLoss,                          // Exit losing positions
}

/// Cross-chain risk manager
#[derive(Debug, Clone)]
pub struct CrossChainRiskManager {
    pub risk_limits: RiskLimits,
    pub monitoring_rules: Vec<RiskMonitoringRule>,
}

impl CrossChainRiskManager {
    pub fn new() -> Self {
        Self {
            risk_limits: RiskLimits::default(),
            monitoring_rules: Vec::new(),
        }
    }

    pub fn validate_allocation(&self, plan: &CrossChainAllocationPlan) -> Result<(), CrossChainOptimizationError> {
        // Validate against risk limits
        if plan.risk_metrics.weighted_risk_score > self.risk_limits.max_portfolio_risk_score {
            return Err(CrossChainOptimizationError::RiskLimitViolation(
                format!("Portfolio risk score {} exceeds limit {}", 
                    plan.risk_metrics.weighted_risk_score, 
                    self.risk_limits.max_portfolio_risk_score)
            ));
        }

        if plan.risk_metrics.max_single_chain_exposure > self.risk_limits.max_single_chain_exposure_pct {
            return Err(CrossChainOptimizationError::RiskLimitViolation(
                format!("Single chain exposure {}% exceeds limit {}%", 
                    plan.risk_metrics.max_single_chain_exposure, 
                    self.risk_limits.max_single_chain_exposure_pct)
            ));
        }

        Ok(())
    }
}

/// Risk limits configuration
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_portfolio_risk_score: f64,
    pub max_single_chain_exposure_pct: f64,
    pub max_single_protocol_exposure_pct: f64,
    pub max_correlation_risk_score: f64,
    pub max_bridge_cost_ratio: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_portfolio_risk_score: 6.0,
            max_single_chain_exposure_pct: 40.0,
            max_single_protocol_exposure_pct: 30.0,
            max_correlation_risk_score: 7.0,
            max_bridge_cost_ratio: 0.03, // 3%
        }
    }
}

/// Risk monitoring rule
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskMonitoringRule {
    pub name: String,
    pub condition: RiskCondition,
    pub action: RiskAction,
    pub is_active: bool,
}

/// Risk condition triggers
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskCondition {
    PortfolioLoss(f64),               // Portfolio loss exceeds %
    ChainDowntime(ChainId),           // Specific chain has issues
    ProtocolExploit(DeFiProtocol),    // Protocol security incident
    HighVolatility(f64),              // Market volatility exceeds threshold
}

/// Risk mitigation actions
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskAction {
    PauseNewAllocations,              // Stop new capital deployment
    ExitPosition(String),             // Exit specific strategy
    ReduceExposure(f64),              // Reduce exposure by %
    HedgePosition,                    // Add hedging positions
}

/// Performance monitoring system
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    pub performance_history: Vec<PerformanceSnapshot>,
    pub benchmarks: HashMap<String, f64>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            performance_history: Vec::new(),
            benchmarks: HashMap::new(),
        }
    }
}

/// Gas tracker for chain-specific gas optimization
#[derive(Debug, Clone)]
pub struct GasTracker {
    pub chain: ChainId,
    pub current_gas_price: f64,
    pub gas_history: Vec<GasPriceSnapshot>,
}

impl GasTracker {
    pub fn new(chain: ChainId) -> Self {
        Self {
            chain,
            current_gas_price: 0.0,
            gas_history: Vec::new(),
        }
    }
}

/// Capacity calculator for chain liquidity limits
#[derive(Debug, Clone)]  
pub struct CapacityCalculator {
    pub chain: ChainId,
    pub total_liquidity: u64,
    pub utilization_rate: f64,
}

impl CapacityCalculator {
    pub fn new(chain: ChainId) -> Self {
        Self {
            chain,
            total_liquidity: 0,
            utilization_rate: 0.0,
        }
    }

    pub fn calculate_total_capacity(&self) -> u64 {
        (self.total_liquidity as f64 * (1.0 - self.utilization_rate)) as u64
    }
}

/// Execution result for allocation plan
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub plan_id: String,
    pub execution_results: Vec<SingleExecutionResult>,
    pub total_deployed_usd: u64,
    pub total_gas_spent: f64,
    pub execution_time: u64,
    pub success_rate: f64,
}

/// Result of executing a single allocation
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SingleExecutionResult {
    pub strategy_id: String,
    pub chain: ChainId,
    pub success: bool,
    pub deployed_amount_usd: u64,
    pub actual_gas_cost: f64,
    pub transaction_hash: String,
    pub block_number: Option<u64>,
    pub execution_time: u64,
    pub error_message: Option<String>,
}

/// Cross-chain optimization errors
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum CrossChainOptimizationError {
    NoViableOpportunities,
    AllocationOptimizationFailed(String),
    RiskLimitViolation(String),
    BridgeNotAvailable(String),
    InsufficientLiquidity(String),
    ExecutionFailed(String),
    ChainUnavailable(ChainId),
}

impl std::fmt::Display for CrossChainOptimizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrossChainOptimizationError::NoViableOpportunities => {
                write!(f, "No viable yield opportunities found across all chains")
            }
            CrossChainOptimizationError::AllocationOptimizationFailed(reason) => {
                write!(f, "Cross-chain allocation optimization failed: {}", reason)
            }
            CrossChainOptimizationError::RiskLimitViolation(violation) => {
                write!(f, "Risk limit violation: {}", violation)
            }
            CrossChainOptimizationError::BridgeNotAvailable(bridge) => {
                write!(f, "Bridge not available: {}", bridge)
            }
            CrossChainOptimizationError::InsufficientLiquidity(chain) => {
                write!(f, "Insufficient liquidity on chain: {}", chain)
            }
            CrossChainOptimizationError::ExecutionFailed(reason) => {
                write!(f, "Execution failed: {}", reason)
            }
            CrossChainOptimizationError::ChainUnavailable(chain) => {
                write!(f, "Chain unavailable: {:?}", chain)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_chain_optimizer_creation() {
        let optimizer = CrossChainYieldOptimizer::new(GlobalAllocationStrategy::default());
        assert_eq!(optimizer.chain_optimizers.len(), 0);
        assert!(optimizer.global_allocation.max_chain_allocation > 0.0);
    }

    #[test]
    fn test_chain_specific_optimizer() {
        let optimizer = ChainSpecificOptimizer::new(ChainId::Ethereum);
        assert_eq!(optimizer.chain, ChainId::Ethereum);
        assert!(optimizer.capacity_calculator.calculate_total_capacity() >= 0);
    }

    #[test]
    fn test_global_allocation_strategy_defaults() {
        let strategy = GlobalAllocationStrategy::default();
        assert_eq!(strategy.max_chain_allocation, 0.4);
        assert_eq!(strategy.max_protocol_allocation, 0.3);
        assert_eq!(strategy.max_risk_score, 7);
        assert_eq!(strategy.min_yield_threshold, 3.0);
    }

    #[test]
    fn test_risk_limits_validation() {
        let limits = RiskLimits::default();
        assert!(limits.max_portfolio_risk_score > 0.0);
        assert!(limits.max_single_chain_exposure_pct > 0.0);
        assert!(limits.max_bridge_cost_ratio > 0.0);
    }
}