// Cross-Chain Yield Farming API
// Day 11: Public canister endpoints for yield optimization

use super::yield_farming::*;
use super::yield_engine::*;
use super::cross_chain_optimizer::*;
use super::arbitrage::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use ic_cdk::{query, update, caller};
use std::collections::HashMap;

// Global yield optimizer instance
use std::cell::RefCell;
thread_local! {
    static YIELD_OPTIMIZER: RefCell<Option<CrossChainYieldOptimizer>> = RefCell::new(None);
    static YIELD_STRATEGIES: RefCell<HashMap<String, YieldStrategy>> = RefCell::new(HashMap::new());
    static STRATEGY_ENGINE: RefCell<Option<YieldStrategyEngine>> = RefCell::new(None);
    static ARBITRAGE_ENGINE: RefCell<Option<CrossChainArbitrageEngine>> = RefCell::new(None);
}

// Initialize the yield farming system
#[update]
pub async fn initialize_yield_farming() -> Result<String, String> {
    let global_strategy = GlobalAllocationStrategy::default();
    let mut optimizer = CrossChainYieldOptimizer::new(global_strategy);
    optimizer.initialize_chain_optimizers();
    
    // Initialize strategy engine
    let mut engine = YieldStrategyEngine::new(EvaluationCriteria::default());
    engine.initialize(); // Initialize with current time
    
    YIELD_OPTIMIZER.with(|opt| *opt.borrow_mut() = Some(optimizer));
    STRATEGY_ENGINE.with(|eng| *eng.borrow_mut() = Some(engine));
    
    Ok("Cross-chain yield farming system initialized successfully".to_string())
}

// Add a new yield strategy to the system
#[update]
pub async fn add_yield_strategy(
    strategy_id: String,
    protocol: DeFiProtocol,
    chain: ChainId,
    strategy_type: YieldStrategyType,
    current_apy: f64,
    risk_score: u8,
    liquidity_usd: u64,
    min_deposit_usd: u64,
) -> Result<String, String> {
    let mut strategy = YieldStrategy::new(strategy_id.clone(), protocol, chain, strategy_type);
    strategy.current_apy = current_apy;
    strategy.risk_score = risk_score;
    strategy.liquidity_usd = liquidity_usd;
    strategy.min_deposit_usd = min_deposit_usd;
    strategy.verified = true; // Auto-verify for demo
    strategy.initialize(); // Initialize with current time
    
    // Add to strategy engine
    STRATEGY_ENGINE.with(|eng| {
        if let Some(ref mut engine) = *eng.borrow_mut() {
            engine.update_strategy(strategy.clone());
        }
    });
    
    // Add to strategies collection
    YIELD_STRATEGIES.with(|strategies| {
        strategies.borrow_mut().insert(strategy_id.clone(), strategy);
    });
    
    Ok(format!("Yield strategy '{}' added successfully", strategy_id))
}

// Get all available yield opportunities
#[query]
pub fn get_yield_opportunities(capital_usd: u64, limit: Option<usize>) -> Result<Vec<YieldOpportunity>, String> {
    STRATEGY_ENGINE.with(|eng| {
        match &*eng.borrow() {
            Some(engine) => {
                let opportunities = engine.evaluate_yield_opportunities(capital_usd);
                let limit = limit.unwrap_or(20).min(100); // Cap at 100 results
                Ok(opportunities.into_iter().take(limit).collect())
            }
            None => Err("Yield farming system not initialized".to_string()),
        }
    })
}

// Get filtered yield opportunities
#[query]
pub fn get_filtered_opportunities(
    capital_usd: u64,
    filters: OpportunityFilters,
    limit: Option<usize>,
) -> Result<Vec<YieldOpportunity>, String> {
    STRATEGY_ENGINE.with(|eng| {
        match &*eng.borrow() {
            Some(engine) => {
                let limit = limit.unwrap_or(10).min(50);
                let opportunities = engine.get_top_opportunities(capital_usd, limit, filters);
                Ok(opportunities)
            }
            None => Err("Yield farming system not initialized".to_string()),
        }
    })
}

// Optimize allocation across chains
#[update]
pub async fn optimize_yield_allocation(total_capital_usd: u64) -> Result<CrossChainAllocationPlan, String> {
    // Create a simple allocation plan for demo purposes
    // In production, this would use the full cross-chain optimizer
    
    let demo_opportunities = STRATEGY_ENGINE.with(|eng| {
        match &*eng.borrow() {
            Some(engine) => engine.evaluate_yield_opportunities(total_capital_usd),
            None => Vec::new(),
        }
    });
    
    if demo_opportunities.is_empty() {
        return Err("No yield opportunities available".to_string());
    }
    
    // Create allocations for top 3 opportunities
    let mut allocations = Vec::new();
    let mut remaining_capital = total_capital_usd;
    
    for (i, opp) in demo_opportunities.iter().take(3).enumerate() {
        let allocation_amount = (remaining_capital / 3).max(opp.min_deposit_usd);
        if allocation_amount <= remaining_capital {
            allocations.push(CrossChainAllocation {
                opportunity: opp.clone(),
                allocated_amount_usd: allocation_amount,
                bridge_cost_usd: 10.0, // Mock bridge cost
                execution_priority: i + 1,
                estimated_deployment_time: 300, // 5 minutes
            });
            remaining_capital -= allocation_amount;
        }
    }
    
    let total_allocated: u64 = allocations.iter().map(|a| a.allocated_amount_usd).sum();
    let weighted_apy = allocations
        .iter()
        .map(|a| a.opportunity.expected_apy * (a.allocated_amount_usd as f64 / total_allocated as f64))
        .sum();
    
    Ok(CrossChainAllocationPlan {
        allocations: allocations.clone(),
        total_capital: total_capital_usd,
        total_allocated,
        unallocated_amount: total_capital_usd - total_allocated,
        total_bridge_costs: allocations.iter().map(|a| a.bridge_cost_usd).sum(),
        expected_weighted_apy: weighted_apy,
        estimated_total_gas_costs: 50.0, // Mock gas costs
        diversification_metrics: DiversificationMetrics {
            num_chains: allocations.iter().map(|a| &a.opportunity.chain).collect::<std::collections::HashSet<_>>().len(),
            num_protocols: allocations.iter().map(|a| &a.opportunity.protocol).collect::<std::collections::HashSet<_>>().len(),
            num_strategies: allocations.len(),
            chain_herfindahl_index: 0.5,
            protocol_herfindahl_index: 0.5,
            max_chain_allocation_pct: 50.0,
            max_protocol_allocation_pct: 50.0,
        },
        risk_metrics: CrossChainRiskMetrics {
            weighted_risk_score: 5.0,
            bridge_risk_ratio: 0.01,
            correlation_risk_score: 3.0,
            max_single_chain_exposure: 40.0,
            liquidity_risk_score: 0.2,
        },
        execution_timeline: ExecutionTimeline {
            total_estimated_time: 900, // 15 minutes
            milestones: allocations.iter().enumerate().map(|(i, allocation)| {
                ExecutionMilestone {
                    step: i + 1,
                    description: format!("Deploy ${} to {} on {}", 
                        allocation.allocated_amount_usd,
                        format!("{:?}", allocation.opportunity.protocol),
                        allocation.opportunity.chain.name()
                    ),
                    estimated_completion_time: (i as u64 + 1) * 300,
                    estimated_gas_cost: allocation.opportunity.gas_cost_estimate,
                    bridge_cost: allocation.bridge_cost_usd,
                }
            }).collect(),
            parallel_execution_possible: true,
        },
        created_at: ic_cdk::api::time(),
    })
}

// Execute allocation plan
#[update]
pub async fn execute_allocation_plan(plan: CrossChainAllocationPlan) -> Result<ExecutionResult, String> {
    // Since we need async execution, we'll create a mock result
    // In production, this would actually execute the allocation plan
    Ok(ExecutionResult {
        plan_id: format!("plan_{}", ic_cdk::api::time()),
        execution_results: plan.allocations.iter().map(|allocation| {
            SingleExecutionResult {
                strategy_id: allocation.opportunity.strategy_id.clone(),
                chain: allocation.opportunity.chain.clone(),
                success: true,
                deployed_amount_usd: allocation.allocated_amount_usd,
                actual_gas_cost: allocation.opportunity.gas_cost_estimate,
                transaction_hash: format!("0x{:x}", allocation.allocated_amount_usd),
                block_number: Some(1000000),
                execution_time: allocation.estimated_deployment_time,
                error_message: None,
            }
        }).collect(),
        total_deployed_usd: plan.total_allocated,
        total_gas_spent: plan.estimated_total_gas_costs,
        execution_time: plan.execution_timeline.total_estimated_time,
        success_rate: 1.0, // 100% success for demo
    })
}

// Get yield farming portfolio summary
#[query]
pub fn get_yield_portfolio_summary() -> Result<YieldPortfolioSummary, String> {
    let user = caller();
    
    // This would track user positions in production
    Ok(YieldPortfolioSummary {
        user_principal: user,
        total_value_usd: 0.0,
        total_allocated_usd: 0,
        weighted_apy: 0.0,
        active_positions: 0,
        chains_count: 0,
        protocols_count: 0,
        total_earned_usd: 0.0,
        last_updated: ic_cdk::api::time(),
    })
}

// Get strategy performance metrics
#[query]
pub fn get_strategy_performance(strategy_id: String) -> Result<StrategyPerformanceMetrics, String> {
    YIELD_STRATEGIES.with(|strategies| {
        match strategies.borrow().get(&strategy_id) {
            Some(strategy) => {
                Ok(StrategyPerformanceMetrics {
                    strategy_id: strategy_id.clone(),
                    current_apy: strategy.current_apy,
                    historical_apy_7d: strategy.historical_apy_7d,
                    historical_apy_30d: strategy.historical_apy_30d,
                    total_value_locked: strategy.liquidity_usd,
                    risk_score: strategy.risk_score,
                    success_rate: 0.95, // Mock data
                    avg_execution_time: 300, // 5 minutes
                    total_users: 0,
                    last_updated: strategy.last_updated,
                })
            }
            None => Err(format!("Strategy '{}' not found", strategy_id)),
        }
    })
}


// Get cross-chain bridge options for yield farming
#[query]
pub fn get_yield_bridge_options(
    from_chain: ChainId,
    to_chain: ChainId,
    asset: String,
    amount_usd: u64,
) -> Vec<BridgeOption> {
    // Mock bridge options - would integrate with actual bridges in production
    vec![
        BridgeOption {
            bridge_name: "Wormhole".to_string(),
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            asset: asset.clone(),
            min_amount_usd: 50,
            max_amount_usd: 100000,
            fee_usd: amount_usd as f64 * 0.003, // 0.3% fee
            estimated_time_minutes: 15,
            security_rating: 8,
        },
        BridgeOption {
            bridge_name: "LayerZero".to_string(),
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            asset: asset.clone(),
            min_amount_usd: 100,
            max_amount_usd: 50000,
            fee_usd: amount_usd as f64 * 0.005, // 0.5% fee
            estimated_time_minutes: 10,
            security_rating: 9,
        }
    ]
}

// Update strategy APY (admin function)
#[update]
pub async fn update_strategy_apy(strategy_id: String, new_apy: f64) -> Result<String, String> {
    YIELD_STRATEGIES.with(|strategies| {
        match strategies.borrow_mut().get_mut(&strategy_id) {
            Some(strategy) => {
                strategy.current_apy = new_apy;
                strategy.last_updated = ic_cdk::api::time();
                
                // Update in strategy engine as well
                STRATEGY_ENGINE.with(|eng| {
                    if let Some(ref mut engine) = *eng.borrow_mut() {
                        engine.update_strategy(strategy.clone());
                    }
                });
                
                Ok(format!("Strategy '{}' APY updated to {}%", strategy_id, new_apy))
            }
            None => Err(format!("Strategy '{}' not found", strategy_id)),
        }
    })
}

// Get yield farming statistics
#[query]
pub fn get_yield_farming_stats() -> YieldFarmingStats {
    let strategy_count = YIELD_STRATEGIES.with(|s| s.borrow().len());
    
    let (total_tvl, avg_apy, chains_active) = YIELD_STRATEGIES.with(|strategies| {
        let strategies = strategies.borrow();
        let total_tvl: u64 = strategies.values().map(|s| s.liquidity_usd).sum();
        let avg_apy = if !strategies.is_empty() {
            strategies.values().map(|s| s.current_apy).sum::<f64>() / strategies.len() as f64
        } else {
            0.0
        };
        let chains: std::collections::HashSet<_> = strategies.values().map(|s| &s.chain).collect();
        (total_tvl, avg_apy, chains.len())
    });
    
    YieldFarmingStats {
        total_strategies: strategy_count,
        total_tvl_usd: total_tvl,
        average_apy: avg_apy,
        active_chains: chains_active,
        total_users: 0, // Mock data
        total_volume_24h: 0.0, // Mock data
        last_updated: ic_cdk::api::time(),
    }
}

// Supporting data structures for API responses
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct YieldPortfolioSummary {
    pub user_principal: candid::Principal,
    pub total_value_usd: f64,
    pub total_allocated_usd: u64,
    pub weighted_apy: f64,
    pub active_positions: usize,
    pub chains_count: usize,
    pub protocols_count: usize,
    pub total_earned_usd: f64,
    pub last_updated: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct StrategyPerformanceMetrics {
    pub strategy_id: String,
    pub current_apy: f64,
    pub historical_apy_7d: f64,
    pub historical_apy_30d: f64,
    pub total_value_locked: u64,
    pub risk_score: u8,
    pub success_rate: f64,
    pub avg_execution_time: u64,
    pub total_users: usize,
    pub last_updated: u64,
}


#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct BridgeOption {
    pub bridge_name: String,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub asset: String,
    pub min_amount_usd: u64,
    pub max_amount_usd: u64,
    pub fee_usd: f64,
    pub estimated_time_minutes: u64,
    pub security_rating: u8,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct YieldFarmingStats {
    pub total_strategies: usize,
    pub total_tvl_usd: u64,
    pub average_apy: f64,
    pub active_chains: usize,
    pub total_users: usize,
    pub total_volume_24h: f64,
    pub last_updated: u64,
}

// Demo data initialization function
#[update]
pub async fn initialize_demo_strategies() -> Result<String, String> {
    // Initialize system first
    initialize_yield_farming().await?;
    
    // Add demo strategies across different chains and protocols
    let demo_strategies = vec![
        ("aave_eth_usdc", DeFiProtocol::Aave, ChainId::Ethereum, 
         YieldStrategyType::Lending { asset: "USDC".to_string(), variable_rate: true }, 
         4.5, 3, 50_000_000, 100),
        
        ("uniswap_arb_ethusdc", DeFiProtocol::Uniswap(UniswapVersion::V3), ChainId::Arbitrum,
         YieldStrategyType::LiquidityProvision { 
             pool_address: "0x123".to_string(), 
             token_a: "ETH".to_string(), 
             token_b: "USDC".to_string(), 
             fee_tier: 500 
         }, 
         12.3, 6, 10_000_000, 1000),
        
        ("raydium_sol_solusdc", DeFiProtocol::Raydium, ChainId::Solana,
         YieldStrategyType::YieldFarming { 
             lp_token: "RAY-USDC-LP".to_string(), 
             reward_tokens: vec!["RAY".to_string()] 
         }, 
         18.7, 7, 5_000_000, 50),
        
        ("compound_poly_dai", DeFiProtocol::Compound, ChainId::Polygon,
         YieldStrategyType::Lending { asset: "DAI".to_string(), variable_rate: false }, 
         6.2, 4, 8_000_000, 200),
        
    ];
    
    for (id, protocol, chain, strategy_type, apy, risk, liquidity, min_deposit) in demo_strategies {
        add_yield_strategy(
            id.to_string(),
            protocol,
            chain,
            strategy_type,
            apy,
            risk,
            liquidity,
            min_deposit,
        ).await?;
    }
    
    Ok("Demo yield strategies initialized successfully".to_string())
}

// ================================
// CROSS-CHAIN ARBITRAGE API ENDPOINTS
// Day 11: Arbitrage opportunity detection and execution
// ================================

// Initialize the arbitrage engine
#[update]
pub async fn initialize_arbitrage_engine() -> Result<String, String> {
    let config = ArbitrageConfiguration::default();
    let mut engine = CrossChainArbitrageEngine::new(config);
    engine.initialize();
    engine.initialize_price_oracles();
    
    ARBITRAGE_ENGINE.with(|arb| *arb.borrow_mut() = Some(engine));
    
    Ok("Cross-chain arbitrage engine initialized successfully".to_string())
}

// Scan for arbitrage opportunities
#[update]
pub async fn scan_cross_chain_arbitrage(
    asset: String,
    min_profit_usd: f64,
    max_capital_usd: u64,
) -> Result<Vec<super::arbitrage::ArbitrageOpportunity>, String> {
    ARBITRAGE_ENGINE.with(|arb| {
        if let Some(ref mut engine) = *arb.borrow_mut() {
            // Since we can't use async in this context directly, create mock opportunities
            // In production, this would use the actual async scan method
            Ok(create_mock_arbitrage_opportunities(asset, min_profit_usd, max_capital_usd))
        } else {
            Err("Arbitrage engine not initialized".to_string())
        }
    })
}

// Get arbitrage engine statistics
#[query]
pub fn get_cross_chain_arbitrage_stats() -> Result<ArbitrageStats, String> {
    ARBITRAGE_ENGINE.with(|arb| {
        match &*arb.borrow() {
            Some(engine) => Ok(engine.get_arbitrage_stats()),
            None => Err("Arbitrage engine not initialized".to_string()),
        }
    })
}

// Execute arbitrage opportunity (simplified for canister interface)
#[update]
pub async fn execute_arbitrage_opportunity(
    opportunity_id: String,
    asset: String,
    buy_chain: ChainId,
    sell_chain: ChainId,
    capital_usd: u64,
) -> Result<ArbitrageExecutionSummary, String> {
    // Simplified execution for demonstration
    // In production, this would use the full arbitrage execution engine
    
    let execution_time = ic_cdk::api::time();
    let mock_profit = capital_usd as f64 * 0.015; // 1.5% profit
    
    Ok(ArbitrageExecutionSummary {
        opportunity_id,
        asset,
        buy_chain,
        sell_chain,
        capital_used_usd: capital_usd,
        profit_usd: mock_profit,
        execution_time_seconds: 120, // 2 minutes
        success: true,
        transaction_hashes: vec![
            format!("0xbuy{:x}", execution_time),
            format!("0xbridge{:x}", execution_time + 1),
            format!("0xsell{:x}", execution_time + 2),
        ],
        total_gas_cost: 25.0,
        bridge_cost: 15.0,
        net_profit_usd: mock_profit - 40.0, // Subtract costs
        executed_at: execution_time,
    })
}

// Get supported arbitrage assets
#[query]
pub fn get_cross_chain_arbitrage_assets() -> Vec<String> {
    vec![
        "USDC".to_string(),
        "USDT".to_string(),
        "ETH".to_string(),
        "WETH".to_string(),
        "BTC".to_string(),
        "WBTC".to_string(),
        "SOL".to_string()
    ]
}

// Get arbitrage bridge options
#[query]
pub fn get_cross_chain_arbitrage_bridges(
    from_chain: ChainId,
    to_chain: ChainId,
    asset: String,
) -> Vec<ArbitrageBridgeOption> {
    vec![
        ArbitrageBridgeOption {
            bridge_name: "Wormhole".to_string(),
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            asset: asset.clone(),
            base_fee_usd: 10.0,
            variable_rate: 0.003, // 0.3%
            estimated_time_minutes: 15,
            confidence_score: 0.9,
            supported: true,
        },
        ArbitrageBridgeOption {
            bridge_name: "LayerZero".to_string(),
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            asset: asset.clone(),
            base_fee_usd: 8.0,
            variable_rate: 0.005, // 0.5%
            estimated_time_minutes: 10,
            confidence_score: 0.85,
            supported: is_layerzero_supported(&from_chain, &to_chain),
        },
        ArbitrageBridgeOption {
            bridge_name: "Stargate".to_string(),
            from_chain,
            to_chain,
            asset,
            base_fee_usd: 12.0,
            variable_rate: 0.002, // 0.2%
            estimated_time_minutes: 8,
            confidence_score: 0.95,
            supported: true,
        },
    ]
}

// Helper function to create mock arbitrage opportunities
fn create_mock_arbitrage_opportunities(
    asset: String,
    min_profit_usd: f64,
    max_capital_usd: u64,
) -> Vec<super::arbitrage::ArbitrageOpportunity> {
    let current_time = ic_cdk::api::time();
    let base_price = match asset.as_str() {
        "USDC" | "USDT" => 1.0,
        "ETH" | "WETH" => 2400.0,
        "BTC" | "WBTC" => 45000.0,
        "SOL" => 95.0,
        _ => 100.0,
    };

    // Create 3 mock opportunities with different chains and profit margins
    let opportunities = vec![
        (ChainId::Ethereum, ChainId::Arbitrum, 0.8, 1.5), // ETH -> ARB
        (ChainId::Polygon, ChainId::Avalanche, 1.2, 2.1), // POLY -> AVAX  
        (ChainId::Solana, ChainId::Avalanche, 0.7, 1.8),  // SOL -> AVAX
    ];

    opportunities
        .into_iter()
        .enumerate()
        .filter_map(|(i, (buy_chain, sell_chain, buy_discount, sell_premium))| {
            let buy_price = base_price * (1.0 - buy_discount / 100.0);
            let sell_price = base_price * (1.0 + sell_premium / 100.0);
            let profit_percentage = ((sell_price - buy_price) / buy_price) * 100.0;
            
            // Calculate execution costs
            let buy_gas = 20.0;
            let sell_gas = 15.0;
            let bridge_cost = 25.0;
            let dex_fees = max_capital_usd as f64 * 0.006; // 0.6% total DEX fees
            let total_cost = buy_gas + sell_gas + bridge_cost + dex_fees;
            
            let gross_profit = (sell_price - buy_price) * (max_capital_usd as f64 / buy_price);
            let net_profit = gross_profit - total_cost;
            
            if net_profit >= min_profit_usd {
                Some(super::arbitrage::ArbitrageOpportunity {
                    asset: asset.clone(),
                    buy_chain: buy_chain.clone(),
                    sell_chain: sell_chain.clone(),
                    buy_price,
                    sell_price,
                    profit_percentage,
                    max_capital_usd,
                    net_profit_usd: net_profit,
                    execution_cost: ExecutionCost {
                        buy_gas_cost: buy_gas,
                        sell_gas_cost: sell_gas,
                        bridge_cost,
                        buy_dex_fee: dex_fees / 2.0,
                        sell_dex_fee: dex_fees / 2.0,
                        total_cost,
                    },
                    execution_time_estimate: 300 + (i as u64 * 120), // 5-9 minutes
                    confidence_score: 0.8 - (i as f64 * 0.1),
                    discovered_at: current_time,
                })
            } else {
                None
            }
        })
        .collect()
}

// Helper function to check LayerZero support
fn is_layerzero_supported(from_chain: &ChainId, to_chain: &ChainId) -> bool {
    let supported_chains = vec![
        ChainId::Ethereum,
        ChainId::Arbitrum,
        ChainId::Optimism,
        ChainId::Polygon,
        ChainId::Avalanche,
    ];
    
    supported_chains.contains(from_chain) && supported_chains.contains(to_chain)
}

// Supporting data structures for arbitrage API responses
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ArbitrageExecutionSummary {
    pub opportunity_id: String,
    pub asset: String,
    pub buy_chain: ChainId,
    pub sell_chain: ChainId,
    pub capital_used_usd: u64,
    pub profit_usd: f64,
    pub execution_time_seconds: u64,
    pub success: bool,
    pub transaction_hashes: Vec<String>,
    pub total_gas_cost: f64,
    pub bridge_cost: f64,
    pub net_profit_usd: f64,
    pub executed_at: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ArbitrageBridgeOption {
    pub bridge_name: String,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub asset: String,
    pub base_fee_usd: f64,
    pub variable_rate: f64,
    pub estimated_time_minutes: u64,
    pub confidence_score: f64,
    pub supported: bool,
}