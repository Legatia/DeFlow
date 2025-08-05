// Cross-Chain Yield Farming Module
// Day 11: Advanced DeFi Workflows - Yield Optimization across Bitcoin, Ethereum ecosystem, and Solana

use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::api::time;

/// Core yield farming types and structures
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChainId {
    Bitcoin,
    Ethereum,
    Arbitrum,
    Optimism,
    Polygon,
    Base,
    Avalanche,
    Sonic,
    BSC,
    Solana,
}

impl ChainId {
    pub fn name(&self) -> &'static str {
        match self {
            ChainId::Bitcoin => "Bitcoin",
            ChainId::Ethereum => "Ethereum",
            ChainId::Arbitrum => "Arbitrum",
            ChainId::Optimism => "Optimism",
            ChainId::Polygon => "Polygon",
            ChainId::Base => "Base",
            ChainId::Avalanche => "Avalanche",
            ChainId::Sonic => "Sonic",
            ChainId::BSC => "Binance Smart Chain",
            ChainId::Solana => "Solana",
        }
    }

    pub fn is_ethereum_ecosystem(&self) -> bool {
        matches!(self, 
            ChainId::Ethereum | ChainId::Arbitrum | ChainId::Optimism | 
            ChainId::Polygon | ChainId::Base | ChainId::Avalanche | 
            ChainId::Sonic | ChainId::BSC
        )
    }
}

/// DeFi protocols supported for yield farming
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DeFiProtocol {
    // Ethereum Ecosystem
    Uniswap(UniswapVersion),
    Aave,
    Compound,
    Curve,
    Balancer,
    Convex,
    Yearn,
    
    // Layer 2 Specific
    QuickSwap,      // Polygon
    SushiSwap,      // Multi-chain
    PancakeSwap,    // BSC
    SpookySwap,     // Fantom
    TraderJoe,      // Avalanche
    
    // Solana Ecosystem
    Raydium,
    Serum,
    Mango,
    Orca,
    Marinade,
    
    // Bitcoin
    LightningNetwork,
    StacksDefi,
    RootStock,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniswapVersion {
    V2,
    V3,
    V4,
}

/// Yield strategy types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq)]
pub enum YieldStrategyType {
    LiquidityProvision {
        pool_address: String,
        token_a: String,
        token_b: String,
        fee_tier: u32,
    },
    Lending {
        asset: String,
        variable_rate: bool,
    },
    Staking {
        asset: String,
        validator: Option<String>,
        lock_period: Option<u64>,
    },
    YieldFarming {
        lp_token: String,
        reward_tokens: Vec<String>,
    },
    Vault {
        vault_address: String,
        strategy_type: String,
    },
}

/// Core yield strategy structure
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct YieldStrategy {
    pub id: String,
    pub protocol: DeFiProtocol,
    pub chain: ChainId,
    pub strategy_type: YieldStrategyType,
    pub current_apy: f64,
    pub historical_apy_7d: f64,
    pub historical_apy_30d: f64,
    pub risk_score: u8, // 1-10, where 1 is lowest risk
    pub liquidity_usd: u64,
    pub min_deposit_usd: u64,
    pub max_deposit_usd: Option<u64>,
    pub deposit_fee: f64,
    pub withdrawal_fee: f64,
    pub performance_fee: f64,
    pub lock_period: Option<u64>, // seconds
    pub auto_compound: bool,
    pub verified: bool,
    pub last_updated: u64,
}

impl YieldStrategy {
    pub fn new(
        id: String,
        protocol: DeFiProtocol,
        chain: ChainId,
        strategy_type: YieldStrategyType,
    ) -> Self {
        Self {
            id,
            protocol,
            chain,
            strategy_type,
            current_apy: 0.0,
            historical_apy_7d: 0.0,
            historical_apy_30d: 0.0,
            risk_score: 5,
            liquidity_usd: 0,
            min_deposit_usd: 100,
            max_deposit_usd: None,
            deposit_fee: 0.0,
            withdrawal_fee: 0.0,
            performance_fee: 0.0,
            lock_period: None,
            auto_compound: false,
            verified: false,
            last_updated: 0, // Will be set when initialized properly
        }
    }

    /// Initialize with current time (for canister use)
    pub fn initialize(&mut self) {
        self.last_updated = time();
    }

    /// Calculate risk-adjusted return (Sharpe ratio approximation)
    pub fn risk_adjusted_return(&self) -> f64 {
        let risk_penalty = self.risk_score as f64 / 10.0;
        self.current_apy * (1.0 - risk_penalty * 0.1)
    }

    /// Calculate effective APY after fees
    pub fn effective_apy(&self, _deposit_amount_usd: u64, holding_period_days: u64) -> f64 {
        let annual_return = self.current_apy / 100.0;
        let deposit_fee_cost = self.deposit_fee / 100.0;
        let withdrawal_fee_cost = self.withdrawal_fee / 100.0;
        let performance_fee_cost = self.performance_fee / 100.0;
        
        // Calculate net return after fees
        let gross_return = annual_return * (holding_period_days as f64 / 365.0);
        let fee_adjusted_return = gross_return * (1.0 - performance_fee_cost) - deposit_fee_cost - withdrawal_fee_cost;
        
        // Annualize the return
        (fee_adjusted_return * 365.0 / holding_period_days as f64) * 100.0
    }

    /// Check if strategy is suitable for given parameters
    pub fn is_suitable(&self, deposit_amount_usd: u64, max_risk_score: u8, min_liquidity_ratio: f64) -> bool {
        // Check deposit limits
        if deposit_amount_usd < self.min_deposit_usd {
            return false;
        }
        
        if let Some(max_deposit) = self.max_deposit_usd {
            if deposit_amount_usd > max_deposit {
                return false;
            }
        }
        
        // Check risk tolerance
        if self.risk_score > max_risk_score {
            return false;
        }
        
        // Check liquidity (deposit should not be more than X% of total liquidity)
        let liquidity_ratio = deposit_amount_usd as f64 / self.liquidity_usd as f64;
        if liquidity_ratio > min_liquidity_ratio {
            return false;
        }
        
        // Must be verified for production
        self.verified
    }
}

/// Risk management parameters
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RiskParameters {
    pub max_allocation_per_protocol: f64, // 0.0 - 1.0
    pub max_allocation_per_chain: f64,
    pub max_risk_score: u8,
    pub min_liquidity_ratio: f64,
    pub max_correlation_exposure: f64,
    pub stop_loss_threshold: f64, // -0.05 = 5% loss triggers exit
    pub rebalance_threshold: f64, // 0.02 = 2% deviation triggers rebalance
    pub min_yield_threshold: f64, // Minimum APY to consider
}

impl Default for RiskParameters {
    fn default() -> Self {
        Self {
            max_allocation_per_protocol: 0.3, // Max 30% in any single protocol
            max_allocation_per_chain: 0.4,    // Max 40% in any single chain
            max_risk_score: 7,                // Max risk score of 7/10
            min_liquidity_ratio: 0.1,         // Deposit max 10% of pool liquidity
            max_correlation_exposure: 0.6,    // Max 60% in correlated assets
            stop_loss_threshold: -0.1,        // Stop loss at 10% loss
            rebalance_threshold: 0.05,        // Rebalance at 5% deviation
            min_yield_threshold: 3.0,         // Minimum 3% APY
        }
    }
}

/// Rebalancing rules and triggers
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RebalancingRules {
    pub enabled: bool,
    pub frequency: RebalanceFrequency,
    pub triggers: Vec<RebalanceTrigger>,
    pub max_gas_cost_usd: f64,
    pub min_improvement_threshold: f64, // Minimum improvement to justify rebalancing
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RebalanceFrequency {
    Never,
    Daily,
    Weekly,
    Monthly,
    OnDemand,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RebalanceTrigger {
    APYDivergence(f64),        // Trigger when APY difference exceeds threshold
    AllocationDrift(f64),      // Trigger when allocation drifts from target
    RiskScoreChange(u8),       // Trigger when risk score changes
    LiquidityChange(f64),      // Trigger when liquidity changes significantly
    MarketVolatility(f64),     // Trigger during high volatility
}

/// Yield optimizer core structure
#[derive(Debug, Clone)]
pub struct YieldOptimizer {
    pub strategies: Vec<YieldStrategy>,
    pub risk_parameters: RiskParameters,
    pub rebalancing_rules: RebalancingRules,
    pub chain_preferences: HashMap<ChainId, f64>, // Weight preferences for chains
    pub protocol_blacklist: Vec<DeFiProtocol>,
    pub last_optimization: u64,
}

impl YieldOptimizer {
    pub fn new(risk_parameters: RiskParameters, rebalancing_rules: RebalancingRules) -> Self {
        Self {
            strategies: Vec::new(),
            risk_parameters,
            rebalancing_rules,
            chain_preferences: HashMap::new(),
            protocol_blacklist: Vec::new(),
            last_optimization: 0, // Will be set when initialized properly
        }
    }

    /// Initialize with current time (for canister use)
    pub fn initialize(&mut self) {
        self.last_optimization = time();
    }

    /// Add yield strategy to the optimizer
    pub fn add_strategy(&mut self, strategy: YieldStrategy) {
        self.strategies.push(strategy);
    }

    /// Remove strategy by ID
    pub fn remove_strategy(&mut self, strategy_id: &str) -> bool {
        let initial_len = self.strategies.len();
        self.strategies.retain(|s| s.id != strategy_id);
        self.strategies.len() < initial_len
    }

    /// Get all available strategies filtered by risk and suitability
    pub fn get_suitable_strategies(&self, deposit_amount_usd: u64) -> Vec<&YieldStrategy> {
        self.strategies
            .iter()
            .filter(|strategy| {
                strategy.is_suitable(
                    deposit_amount_usd,
                    self.risk_parameters.max_risk_score,
                    self.risk_parameters.min_liquidity_ratio,
                ) && strategy.current_apy >= self.risk_parameters.min_yield_threshold
                  && !self.protocol_blacklist.contains(&strategy.protocol)
            })
            .collect()
    }

    /// Find optimal allocation across strategies
    pub fn optimize_allocation(&self, total_capital_usd: u64) -> Result<AllocationPlan, YieldOptimizationError> {
        let suitable_strategies = self.get_suitable_strategies(total_capital_usd);
        
        if suitable_strategies.is_empty() {
            return Err(YieldOptimizationError::NoSuitableStrategies);
        }

        // Score each strategy based on risk-adjusted returns and constraints
        let mut scored_strategies: Vec<_> = suitable_strategies
            .into_iter()
            .map(|strategy| {
                let base_score = strategy.risk_adjusted_return();
                let chain_preference = self.chain_preferences
                    .get(&strategy.chain)
                    .unwrap_or(&1.0);
                let final_score = base_score * chain_preference;
                
                (strategy, final_score)
            })
            .collect();

        // Sort by score (highest first)
        scored_strategies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Allocate capital using greedy approach with constraints
        let mut allocations = Vec::new();
        let mut remaining_capital = total_capital_usd as f64;
        let mut chain_allocations: HashMap<ChainId, f64> = HashMap::new();
        let mut protocol_allocations: HashMap<DeFiProtocol, f64> = HashMap::new();

        for (strategy, _score) in scored_strategies {
            if remaining_capital < strategy.min_deposit_usd as f64 {
                continue;
            }

            // Calculate maximum allocation based on constraints
            let max_by_protocol = (total_capital_usd as f64 * self.risk_parameters.max_allocation_per_protocol)
                - protocol_allocations.get(&strategy.protocol).unwrap_or(&0.0);
            
            let max_by_chain = (total_capital_usd as f64 * self.risk_parameters.max_allocation_per_chain)
                - chain_allocations.get(&strategy.chain).unwrap_or(&0.0);

            let max_by_liquidity = strategy.liquidity_usd as f64 * self.risk_parameters.min_liquidity_ratio;

            let max_allocation = [
                remaining_capital,
                max_by_protocol,
                max_by_chain,
                max_by_liquidity,
                strategy.max_deposit_usd.unwrap_or(u64::MAX) as f64,
            ].iter().fold(f64::INFINITY, |a, &b| a.min(b));

            if max_allocation >= strategy.min_deposit_usd as f64 {
                let allocation_amount = max_allocation.min(remaining_capital * 0.3); // Don't put more than 30% in single strategy
                
                allocations.push(StrategyAllocation {
                    strategy_id: strategy.id.clone(),
                    chain: strategy.chain.clone(),
                    protocol: strategy.protocol.clone(),
                    amount_usd: allocation_amount as u64,
                    expected_apy: strategy.current_apy,
                    risk_score: strategy.risk_score,
                });

                remaining_capital -= allocation_amount;
                *chain_allocations.entry(strategy.chain.clone()).or_insert(0.0) += allocation_amount;
                *protocol_allocations.entry(strategy.protocol.clone()).or_insert(0.0) += allocation_amount;

                if remaining_capital < 100.0 { // Stop if less than $100 remaining
                    break;
                }
            }
        }

        if allocations.is_empty() {
            return Err(YieldOptimizationError::AllocationFailed("No viable allocations found".to_string()));
        }

        let allocated_amount: u64 = allocations.iter().map(|a| a.amount_usd).sum();
        let weighted_apy = allocations
            .iter()
            .map(|a| a.expected_apy * (a.amount_usd as f64 / allocated_amount as f64))
            .sum();

        let plan_allocations = allocations.clone();
        Ok(AllocationPlan {
            allocations,
            total_allocated_usd: allocated_amount,
            unallocated_usd: total_capital_usd - allocated_amount,
            expected_weighted_apy: weighted_apy,
            risk_score: self.calculate_portfolio_risk_score(&plan_allocations),
            diversification_score: self.calculate_diversification_score(&plan_allocations),
            estimated_gas_cost_usd: self.estimate_deployment_cost(&plan_allocations),
            created_at: 0, // Will be set when plan is created properly
        })
    }

    /// Calculate portfolio-level risk score
    fn calculate_portfolio_risk_score(&self, allocations: &[StrategyAllocation]) -> f64 {
        if allocations.is_empty() {
            return 0.0;
        }

        let total_amount: u64 = allocations.iter().map(|a| a.amount_usd).sum();
        allocations
            .iter()
            .map(|a| (a.risk_score as f64) * (a.amount_usd as f64 / total_amount as f64))
            .sum()
    }

    /// Calculate diversification score (higher is better)
    fn calculate_diversification_score(&self, allocations: &[StrategyAllocation]) -> f64 {
        if allocations.is_empty() {
            return 0.0;
        }

        let num_strategies = allocations.len() as f64;
        let num_chains = allocations.iter().map(|a| &a.chain).collect::<std::collections::HashSet<_>>().len() as f64;
        let num_protocols = allocations.iter().map(|a| &a.protocol).collect::<std::collections::HashSet<_>>().len() as f64;

        // Diversification score based on number of strategies, chains, and protocols
        (num_strategies * 0.4 + num_chains * 0.4 + num_protocols * 0.2).min(10.0)
    }

    /// Estimate gas costs for deploying capital across strategies
    fn estimate_deployment_cost(&self, allocations: &[StrategyAllocation]) -> f64 {
        allocations
            .iter()
            .map(|allocation| {
                match allocation.chain {
                    ChainId::Ethereum => 50.0,      // High gas costs
                    ChainId::Bitcoin => 10.0,       // Lower fees
                    ChainId::Arbitrum | ChainId::Optimism => 5.0, // L2 efficiency
                    ChainId::Polygon | ChainId::BSC => 1.0,       // Very low costs
                    ChainId::Solana => 0.01,        // Extremely low costs
                    _ => 20.0,                      // Default estimate
                }
            })
            .sum()
    }
}

/// Strategy allocation within optimization plan
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyAllocation {
    pub strategy_id: String,
    pub chain: ChainId,
    pub protocol: DeFiProtocol,
    pub amount_usd: u64,
    pub expected_apy: f64,
    pub risk_score: u8,
}

/// Complete allocation plan from optimizer
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AllocationPlan {
    pub allocations: Vec<StrategyAllocation>,
    pub total_allocated_usd: u64,
    pub unallocated_usd: u64,
    pub expected_weighted_apy: f64,
    pub risk_score: f64,
    pub diversification_score: f64,
    pub estimated_gas_cost_usd: f64,
    pub created_at: u64,
}

impl AllocationPlan {
    /// Get allocation summary by chain
    pub fn get_chain_allocation_summary(&self) -> HashMap<ChainId, ChainAllocationSummary> {
        let mut summary = HashMap::new();
        
        for allocation in &self.allocations {
            let entry = summary.entry(allocation.chain.clone()).or_insert(ChainAllocationSummary {
                chain: allocation.chain.clone(),
                total_amount_usd: 0,
                strategy_count: 0,
                weighted_apy: 0.0,
                avg_risk_score: 0.0,
            });
            
            entry.total_amount_usd += allocation.amount_usd;
            entry.strategy_count += 1;
        }
        
        // Calculate weighted averages
        for (_, summary_item) in summary.iter_mut() {
            let chain_allocations: Vec<_> = self.allocations
                .iter()
                .filter(|a| a.chain == summary_item.chain)
                .collect();
            
            summary_item.weighted_apy = chain_allocations
                .iter()
                .map(|a| a.expected_apy * (a.amount_usd as f64 / summary_item.total_amount_usd as f64))
                .sum();
            
            summary_item.avg_risk_score = chain_allocations
                .iter()
                .map(|a| a.risk_score as f64)
                .sum::<f64>() / chain_allocations.len() as f64;
        }
        
        summary
    }
}

/// Chain allocation summary
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ChainAllocationSummary {
    pub chain: ChainId,
    pub total_amount_usd: u64,
    pub strategy_count: usize,
    pub weighted_apy: f64,
    pub avg_risk_score: f64,
}

/// Yield optimization errors
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum YieldOptimizationError {
    NoSuitableStrategies,
    InsufficientCapital(u64), // Minimum required
    AllocationFailed(String),
    RiskConstraintViolation(String),
    InvalidStrategy(String),
    ChainNotSupported(String),
}

impl std::fmt::Display for YieldOptimizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YieldOptimizationError::NoSuitableStrategies => {
                write!(f, "No suitable yield strategies found matching risk criteria")
            }
            YieldOptimizationError::InsufficientCapital(min) => {
                write!(f, "Insufficient capital: minimum {} USD required", min)
            }
            YieldOptimizationError::AllocationFailed(reason) => {
                write!(f, "Allocation optimization failed: {}", reason)
            }
            YieldOptimizationError::RiskConstraintViolation(constraint) => {
                write!(f, "Risk constraint violation: {}", constraint)
            }
            YieldOptimizationError::InvalidStrategy(strategy) => {
                write!(f, "Invalid strategy: {}", strategy)
            }
            YieldOptimizationError::ChainNotSupported(chain) => {
                write!(f, "Chain not supported: {}", chain)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock time function for tests
    fn mock_time() -> u64 {
        1234567890_u64
    }

    fn create_test_strategy(id: &str, chain: ChainId, apy: f64, risk: u8) -> YieldStrategy {
        let mut strategy = YieldStrategy::new(
            id.to_string(),
            DeFiProtocol::Aave,
            chain,
            YieldStrategyType::Lending {
                asset: "USDC".to_string(),
                variable_rate: true,
            },
        );
        strategy.current_apy = apy;
        strategy.risk_score = risk;
        strategy.liquidity_usd = 1_000_000;
        strategy.verified = true;
        strategy.last_updated = mock_time(); // Set test time manually
        strategy
    }

    #[test]
    fn test_yield_strategy_creation() {
        let strategy = create_test_strategy("test1", ChainId::Ethereum, 8.5, 4);
        assert_eq!(strategy.id, "test1");
        assert_eq!(strategy.chain, ChainId::Ethereum);
        assert_eq!(strategy.current_apy, 8.5);
        assert_eq!(strategy.risk_score, 4);
    }

    #[test]
    fn test_risk_adjusted_return() {
        let strategy = create_test_strategy("test1", ChainId::Ethereum, 10.0, 8);
        let risk_adjusted = strategy.risk_adjusted_return();
        // Risk score 8 should reduce APY: 10.0 * (1.0 - 0.8 * 0.1) = 9.2
        assert!((risk_adjusted - 9.2).abs() < 0.01);
    }

    #[test]
    fn test_effective_apy_calculation() {
        let mut strategy = create_test_strategy("test1", ChainId::Ethereum, 12.0, 5);
        strategy.deposit_fee = 0.5;    // 0.5%
        strategy.withdrawal_fee = 0.3; // 0.3%
        strategy.performance_fee = 10.0; // 10%
        
        let effective_apy = strategy.effective_apy(10000, 365);
        // Should account for all fees over the year
        assert!(effective_apy < 12.0);
        assert!(effective_apy > 8.0); // Should still be positive after fees
    }

    #[test]
    fn test_strategy_suitability() {
        let strategy = create_test_strategy("test1", ChainId::Ethereum, 8.0, 6);
        
        // Should be suitable with reasonable parameters
        assert!(strategy.is_suitable(1000, 8, 0.1));
        
        // Should fail with high risk aversion
        assert!(!strategy.is_suitable(1000, 4, 0.1));
        
        // Should fail with low deposit
        assert!(!strategy.is_suitable(50, 8, 0.1));
    }

    #[test]
    fn test_yield_optimizer_creation() {
        let risk_params = RiskParameters::default();
        let rebalance_rules = RebalancingRules {
            enabled: true,
            frequency: RebalanceFrequency::Weekly,
            triggers: vec![],
            max_gas_cost_usd: 100.0,
            min_improvement_threshold: 0.02,
        };
        
        let mut optimizer = YieldOptimizer::new(risk_params, rebalance_rules);
        optimizer.last_optimization = mock_time(); // Set test time manually
        assert_eq!(optimizer.strategies.len(), 0);
        assert!(optimizer.rebalancing_rules.enabled);
    }

    #[test]
    fn test_optimizer_add_remove_strategies() {
        let mut optimizer = YieldOptimizer::new(RiskParameters::default(), RebalancingRules {
            enabled: false,
            frequency: RebalanceFrequency::Never,
            triggers: vec![],
            max_gas_cost_usd: 0.0,
            min_improvement_threshold: 0.0,
        });
        optimizer.last_optimization = mock_time(); // Set test time manually
        
        let strategy = create_test_strategy("test1", ChainId::Ethereum, 8.0, 5);
        optimizer.add_strategy(strategy);
        assert_eq!(optimizer.strategies.len(), 1);
        
        assert!(optimizer.remove_strategy("test1"));
        assert_eq!(optimizer.strategies.len(), 0);
        
        assert!(!optimizer.remove_strategy("nonexistent"));
    }

    #[test]
    fn test_chain_id_properties() {
        assert_eq!(ChainId::Ethereum.name(), "Ethereum");
        assert_eq!(ChainId::Solana.name(), "Solana");
        
        assert!(ChainId::Ethereum.is_ethereum_ecosystem());
        assert!(ChainId::Arbitrum.is_ethereum_ecosystem());
        assert!(!ChainId::Bitcoin.is_ethereum_ecosystem());
        assert!(!ChainId::Solana.is_ethereum_ecosystem());
    }

    #[test]
    fn test_optimization_with_multiple_strategies() {
        let mut optimizer = YieldOptimizer::new(RiskParameters::default(), RebalancingRules {
            enabled: false,
            frequency: RebalanceFrequency::Never,
            triggers: vec![],
            max_gas_cost_usd: 0.0,
            min_improvement_threshold: 0.0,
        });
        optimizer.last_optimization = mock_time(); // Set test time manually
        
        // Add diverse strategies
        optimizer.add_strategy(create_test_strategy("eth_aave", ChainId::Ethereum, 8.0, 4));
        optimizer.add_strategy(create_test_strategy("arb_uniswap", ChainId::Arbitrum, 12.0, 6));
        optimizer.add_strategy(create_test_strategy("sol_raydium", ChainId::Solana, 15.0, 7));
        
        let result = optimizer.optimize_allocation(10000);
        assert!(result.is_ok());
        
        let plan = result.unwrap();
        assert!(!plan.allocations.is_empty());
        assert!(plan.total_allocated_usd > 0);
        assert!(plan.expected_weighted_apy > 0.0);
        assert!(plan.diversification_score > 0.0);
    }
}