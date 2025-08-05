// Cross-Chain Arbitrage Engine
// Day 11: Advanced DeFi - Identify and execute profitable arbitrage opportunities across chains

use super::yield_farming::{ChainId, DeFiProtocol};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::api::time;

/// Cross-chain arbitrage engine for identifying and executing profitable opportunities
#[derive(Debug, Clone)]
pub struct CrossChainArbitrageEngine {
    pub price_oracles: HashMap<ChainId, PriceOracle>,
    pub dex_integrations: HashMap<ChainId, Vec<DexIntegration>>,
    pub bridge_costs: HashMap<(ChainId, ChainId), BridgeCostCalculator>,
    pub gas_estimators: HashMap<ChainId, GasEstimator>,
    pub arbitrage_config: ArbitrageConfiguration,
    pub profit_threshold: f64,
    pub last_scan: u64,
}

impl CrossChainArbitrageEngine {
    pub fn new(config: ArbitrageConfiguration) -> Self {
        Self {
            price_oracles: HashMap::new(),
            dex_integrations: HashMap::new(),
            bridge_costs: HashMap::new(),
            gas_estimators: HashMap::new(),
            arbitrage_config: config,
            profit_threshold: 0.005, // 0.5% minimum profit
            last_scan: 0, // Will be set when initialized properly
        }
    }

    /// Initialize with current time (for canister use)
    pub fn initialize(&mut self) {
        self.last_scan = time();
    }

    /// Initialize price oracles for all supported chains
    pub fn initialize_price_oracles(&mut self) {
        let chains = vec![
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

        for chain in chains {
            let oracle = PriceOracle::new(chain.clone());
            self.price_oracles.insert(chain, oracle);
        }
    }

    /// Add DEX integration for a specific chain
    pub fn add_dex_integration(&mut self, chain: ChainId, dex: DexIntegration) {
        self.dex_integrations.entry(chain).or_insert_with(Vec::new).push(dex);
    }

    /// Scan for arbitrage opportunities across all chains
    pub async fn scan_arbitrage_opportunities(
        &mut self,
        asset: String,
        min_profit_usd: f64,
        max_capital_usd: u64,
    ) -> Result<Vec<ArbitrageOpportunity>, ArbitrageError> {
        let mut opportunities = Vec::new();

        // Get prices from all chains
        let price_data = self.collect_cross_chain_prices(&asset).await?;
        
        if price_data.len() < 2 {
            return Err(ArbitrageError::InsufficientPriceData(asset));
        }

        // Find profitable price differences
        for (buy_chain, buy_price) in &price_data {
            for (sell_chain, sell_price) in &price_data {
                if buy_chain == sell_chain {
                    continue;
                }

                let price_diff = sell_price - buy_price;
                let profit_percentage = (price_diff / buy_price) * 100.0;

                if profit_percentage > self.profit_threshold * 100.0 {
                    // Calculate execution costs
                    let execution_cost = self.calculate_execution_cost(
                        buy_chain,
                        sell_chain,
                        max_capital_usd,
                        &asset,
                    ).await?;

                    let net_profit = (price_diff * max_capital_usd as f64) - execution_cost.total_cost;
                    
                    if net_profit >= min_profit_usd {
                        let opportunity = ArbitrageOpportunity {
                            asset: asset.clone(),
                            buy_chain: buy_chain.clone(),
                            sell_chain: sell_chain.clone(),
                            buy_price: *buy_price,
                            sell_price: *sell_price,
                            profit_percentage,
                            max_capital_usd,
                            net_profit_usd: net_profit,
                            execution_cost,
                            execution_time_estimate: self.estimate_execution_time(buy_chain, sell_chain),
                            confidence_score: self.calculate_confidence_score(buy_chain, sell_chain, &asset),
                            discovered_at: time(),
                        };
                        opportunities.push(opportunity);
                    }
                }
            }
        }

        // Sort by net profit (highest first)
        opportunities.sort_by(|a, b| b.net_profit_usd.partial_cmp(&a.net_profit_usd).unwrap());

        self.last_scan = time();
        Ok(opportunities)
    }

    /// Collect prices from all chains for a specific asset
    async fn collect_cross_chain_prices(&self, asset: &str) -> Result<HashMap<ChainId, f64>, ArbitrageError> {
        let mut prices = HashMap::new();

        for (chain, oracle) in &self.price_oracles {
            match oracle.get_asset_price(asset).await {
                Ok(price) => {
                    prices.insert(chain.clone(), price);
                }
                Err(_) => {
                    // Log error but continue with other chains
                    continue;
                }
            }
        }

        Ok(prices)
    }

    /// Calculate total execution cost for arbitrage
    async fn calculate_execution_cost(
        &self,
        buy_chain: &ChainId,
        sell_chain: &ChainId,
        amount_usd: u64,
        asset: &str,
    ) -> Result<ExecutionCost, ArbitrageError> {
        // Gas costs for buying
        let buy_gas_cost = self.gas_estimators
            .get(buy_chain)
            .map(|estimator| estimator.estimate_swap_cost(amount_usd))
            .unwrap_or(self.default_gas_cost(buy_chain));

        // Gas costs for selling
        let sell_gas_cost = self.gas_estimators
            .get(sell_chain)
            .map(|estimator| estimator.estimate_swap_cost(amount_usd))
            .unwrap_or(self.default_gas_cost(sell_chain));

        // Bridge costs
        let bridge_cost = self.bridge_costs
            .get(&(buy_chain.clone(), sell_chain.clone()))
            .map(|calculator| calculator.calculate_cost(amount_usd, asset))
            .unwrap_or_else(|| self.default_bridge_cost(buy_chain, sell_chain, amount_usd));

        // DEX fees
        let buy_dex_fee = self.calculate_dex_fee(buy_chain, amount_usd);
        let sell_dex_fee = self.calculate_dex_fee(sell_chain, amount_usd);

        Ok(ExecutionCost {
            buy_gas_cost,
            sell_gas_cost,
            bridge_cost,
            buy_dex_fee,
            sell_dex_fee,
            total_cost: buy_gas_cost + sell_gas_cost + bridge_cost + buy_dex_fee + sell_dex_fee,
        })
    }

    /// Calculate DEX trading fees
    fn calculate_dex_fee(&self, chain: &ChainId, amount_usd: u64) -> f64 {
        // Default DEX fees by chain (typical rates)
        let fee_rate = match chain {
            ChainId::Ethereum => 0.003,    // 0.3% Uniswap V2/V3
            ChainId::Arbitrum => 0.003,    // 0.3% Uniswap/SushiSwap
            ChainId::Optimism => 0.003,    // 0.3% Uniswap
            ChainId::Polygon => 0.003,     // 0.3% QuickSwap
            ChainId::Base => 0.003,        // 0.3% Uniswap
            ChainId::Avalanche => 0.003,   // 0.3% Trader Joe
            ChainId::BSC => 0.002,         // 0.2% PancakeSwap
            ChainId::Solana => 0.0025,     // 0.25% Raydium
            _ => 0.003,                    // Default 0.3%
        };

        amount_usd as f64 * fee_rate
    }

    /// Default gas cost estimates by chain
    fn default_gas_cost(&self, chain: &ChainId) -> f64 {
        match chain {
            ChainId::Ethereum => 80.0,     // High gas costs
            ChainId::Bitcoin => 15.0,      // Transaction fees
            ChainId::Arbitrum => 5.0,      // L2 efficiency
            ChainId::Optimism => 5.0,      // L2 efficiency
            ChainId::Polygon => 0.5,       // Very low costs
            ChainId::Base => 2.0,          // Base L2
            ChainId::Avalanche => 3.0,     // Avalanche C-Chain
            ChainId::BSC => 0.8,           // Low BSC fees
            ChainId::Solana => 0.002,      // Extremely low costs
            _ => 10.0,                     // Default estimate
        }
    }

    /// Default bridge cost calculation
    fn default_bridge_cost(&self, from_chain: &ChainId, to_chain: &ChainId, amount_usd: u64) -> f64 {
        let base_cost = match (from_chain, to_chain) {
            // Ethereum <-> L2s (cheaper)
            (ChainId::Ethereum, ChainId::Arbitrum) | (ChainId::Arbitrum, ChainId::Ethereum) => 15.0,
            (ChainId::Ethereum, ChainId::Optimism) | (ChainId::Optimism, ChainId::Ethereum) => 15.0,
            (ChainId::Ethereum, ChainId::Base) | (ChainId::Base, ChainId::Ethereum) => 12.0,
            (ChainId::Ethereum, ChainId::Polygon) | (ChainId::Polygon, ChainId::Ethereum) => 25.0,
            
            // Cross-chain (more expensive)
            (ChainId::Ethereum, ChainId::Solana) | (ChainId::Solana, ChainId::Ethereum) => 40.0,
            (ChainId::Ethereum, ChainId::BSC) | (ChainId::BSC, ChainId::Ethereum) => 35.0,
            (ChainId::Ethereum, ChainId::Avalanche) | (ChainId::Avalanche, ChainId::Ethereum) => 30.0,
            
            // L2 <-> L2 (via Ethereum)
            _ if from_chain.is_ethereum_l2() && to_chain.is_ethereum_l2() => 25.0,
            
            // Default cross-chain
            _ => 50.0,
        };

        let variable_rate = 0.001; // 0.1% of amount
        base_cost + (amount_usd as f64 * variable_rate)
    }

    /// Estimate total execution time
    fn estimate_execution_time(&self, buy_chain: &ChainId, sell_chain: &ChainId) -> u64 {
        let buy_time = self.chain_execution_time(buy_chain);
        let sell_time = self.chain_execution_time(sell_chain);
        let bridge_time = self.bridge_time(buy_chain, sell_chain);
        
        buy_time + bridge_time + sell_time
    }

    /// Chain-specific execution times
    fn chain_execution_time(&self, chain: &ChainId) -> u64 {
        match chain {
            ChainId::Ethereum => 300,      // 5 minutes (congestion)
            ChainId::Bitcoin => 1800,      // 30 minutes (confirmations)
            ChainId::Arbitrum => 60,       // 1 minute
            ChainId::Optimism => 120,      // 2 minutes (withdrawal delays)
            ChainId::Polygon => 60,        // 1 minute
            ChainId::Base => 60,           // 1 minute
            ChainId::Avalanche => 30,      // 30 seconds
            ChainId::BSC => 60,            // 1 minute
            ChainId::Solana => 5,          // 5 seconds
            _ => 120,                      // 2 minutes default
        }
    }

    /// Bridge-specific transfer times
    fn bridge_time(&self, from_chain: &ChainId, to_chain: &ChainId) -> u64 {
        match (from_chain, to_chain) {
            // Fast L2 <-> Ethereum
            (ChainId::Arbitrum, ChainId::Ethereum) => 420,  // 7 minutes
            (ChainId::Ethereum, ChainId::Arbitrum) => 600,  // 10 minutes
            (ChainId::Optimism, ChainId::Ethereum) => 1800, // 30 minutes (optimistic rollup)
            (ChainId::Ethereum, ChainId::Optimism) => 300,  // 5 minutes
            
            // Cross-chain bridges
            (_, ChainId::Solana) | (ChainId::Solana, _) => 900,     // 15 minutes
            (_, ChainId::BSC) | (ChainId::BSC, _) => 600,           // 10 minutes
            (_, ChainId::Avalanche) | (ChainId::Avalanche, _) => 480, // 8 minutes
            
            // Default bridge time
            _ => 600, // 10 minutes
        }
    }

    /// Calculate confidence score for arbitrage opportunity
    fn calculate_confidence_score(&self, buy_chain: &ChainId, sell_chain: &ChainId, asset: &str) -> f64 {
        let mut score: f64 = 0.5; // Base confidence

        // Chain reliability bonus
        let reliable_chains = vec![ChainId::Ethereum, ChainId::Arbitrum, ChainId::Optimism, ChainId::Polygon];
        if reliable_chains.contains(buy_chain) {
            score += 0.1;
        }
        if reliable_chains.contains(sell_chain) {
            score += 0.1;
        }

        // Asset liquidity bonus (major assets)
        let major_assets = vec!["USDC", "USDT", "ETH", "WETH", "BTC", "WBTC"];
        if major_assets.iter().any(|&major| asset.contains(major)) {
            score += 0.2;
        }

        // Bridge reliability
        if self.is_reliable_bridge_route(buy_chain, sell_chain) {
            score += 0.15;
        }

        score.min(1.0)
    }

    /// Check if bridge route is reliable
    fn is_reliable_bridge_route(&self, from_chain: &ChainId, to_chain: &ChainId) -> bool {
        match (from_chain, to_chain) {
            // Ethereum <-> L2s are most reliable
            (ChainId::Ethereum, _) | (_, ChainId::Ethereum) if to_chain.is_ethereum_l2() || from_chain.is_ethereum_l2() => true,
            
            // Major cross-chain routes
            (ChainId::Ethereum, ChainId::Solana) | (ChainId::Solana, ChainId::Ethereum) => true,
            (ChainId::Ethereum, ChainId::BSC) | (ChainId::BSC, ChainId::Ethereum) => true,
            (ChainId::Ethereum, ChainId::Avalanche) | (ChainId::Avalanche, ChainId::Ethereum) => true,
            
            _ => false,
        }
    }

    /// Execute arbitrage opportunity
    pub async fn execute_arbitrage(&mut self, opportunity: ArbitrageOpportunity) -> Result<ArbitrageExecutionResult, ArbitrageError> {
        // Pre-execution validation
        self.validate_opportunity(&opportunity)?;

        // Execute buy transaction
        let buy_result = self.execute_buy_transaction(&opportunity).await?;

        // Execute bridge transfer
        let bridge_result = self.execute_bridge_transfer(&opportunity, &buy_result).await?;

        // Execute sell transaction
        let sell_result = self.execute_sell_transaction(&opportunity, &bridge_result).await?;

        // Calculate actual profit
        let actual_profit = sell_result.amount_received - opportunity.max_capital_usd as f64 - buy_result.total_cost - bridge_result.cost - sell_result.total_cost;

        Ok(ArbitrageExecutionResult {
            opportunity_id: format!("arb_{}", opportunity.discovered_at),
            success: actual_profit > 0.0,
            buy_result,
            bridge_result,
            sell_result,
            actual_profit_usd: actual_profit,
            expected_profit_usd: opportunity.net_profit_usd,
            total_execution_time: time() - opportunity.discovered_at,
            gas_used_total: 0.0, // Would track actual gas usage
        })
    }

    /// Validate arbitrage opportunity before execution
    fn validate_opportunity(&self, opportunity: &ArbitrageOpportunity) -> Result<(), ArbitrageError> {
        // Check if opportunity is still valid (not expired)
        let age = time() - opportunity.discovered_at;
        if age > self.arbitrage_config.max_opportunity_age {
            return Err(ArbitrageError::OpportunityExpired(age));
        }

        // Check minimum profit threshold
        if opportunity.net_profit_usd < self.arbitrage_config.min_profit_usd {
            return Err(ArbitrageError::InsufficientProfit(opportunity.net_profit_usd));
        }

        Ok(())
    }

    /// Execute buy transaction (mock implementation)
    async fn execute_buy_transaction(&self, opportunity: &ArbitrageOpportunity) -> Result<TransactionResult, ArbitrageError> {
        // In production, this would interact with actual DEX contracts
        Ok(TransactionResult {
            transaction_hash: format!("0xbuy{:x}", time()),
            amount_in: opportunity.max_capital_usd as f64,
            amount_out: opportunity.max_capital_usd as f64 / opportunity.buy_price,
            amount_received: opportunity.max_capital_usd as f64 / opportunity.buy_price,
            gas_used: opportunity.execution_cost.buy_gas_cost,
            total_cost: opportunity.execution_cost.buy_gas_cost + opportunity.execution_cost.buy_dex_fee,
            success: true,
            error_message: None,
        })
    }

    /// Execute bridge transfer (mock implementation)
    async fn execute_bridge_transfer(&self, opportunity: &ArbitrageOpportunity, _buy_result: &TransactionResult) -> Result<BridgeResult, ArbitrageError> {
        // In production, this would interact with actual bridge contracts
        Ok(BridgeResult {
            bridge_hash: format!("0xbridge{:x}", time()),
            amount_bridged: opportunity.max_capital_usd as f64 / opportunity.buy_price,
            cost: opportunity.execution_cost.bridge_cost,
            time_taken: self.bridge_time(&opportunity.buy_chain, &opportunity.sell_chain),
            success: true,
            error_message: None,
        })
    }

    /// Execute sell transaction (mock implementation)
    async fn execute_sell_transaction(&self, opportunity: &ArbitrageOpportunity, _bridge_result: &BridgeResult) -> Result<TransactionResult, ArbitrageError> {
        // In production, this would interact with actual DEX contracts
        let tokens_to_sell = opportunity.max_capital_usd as f64 / opportunity.buy_price;
        let amount_received = tokens_to_sell * opportunity.sell_price;
        
        Ok(TransactionResult {
            transaction_hash: format!("0xsell{:x}", time()),
            amount_in: tokens_to_sell,
            amount_out: amount_received,
            amount_received,
            gas_used: opportunity.execution_cost.sell_gas_cost,
            total_cost: opportunity.execution_cost.sell_gas_cost + opportunity.execution_cost.sell_dex_fee,
            success: true,
            error_message: None,
        })
    }

    /// Get arbitrage statistics
    pub fn get_arbitrage_stats(&self) -> ArbitrageStats {
        ArbitrageStats {
            total_opportunities_found: 0, // Would track in production
            successful_arbitrages: 0,
            total_profit_usd: 0.0,
            average_execution_time: 0,
            supported_chains: self.price_oracles.len(),
            supported_assets: self.arbitrage_config.supported_assets.len(),
            last_scan_timestamp: self.last_scan,
        }
    }
}

/// Price oracle for fetching asset prices from different chains
#[derive(Debug, Clone)]
pub struct PriceOracle {
    pub chain: ChainId,
    pub price_cache: HashMap<String, CachedPrice>,
    pub cache_duration: u64,
}

impl PriceOracle {
    pub fn new(chain: ChainId) -> Self {
        Self {
            chain,
            price_cache: HashMap::new(),
            cache_duration: 300, // 5 minutes
        }
    }

    /// Get asset price from the chain
    pub async fn get_asset_price(&self, asset: &str) -> Result<f64, ArbitrageError> {
        // Check cache first
        if let Some(cached) = self.price_cache.get(asset) {
            if time() - cached.timestamp < self.cache_duration {
                return Ok(cached.price);
            }
        }

        // Fetch from chain (mock implementation)
        let price = self.fetch_price_from_chain(asset).await?;
        Ok(price)
    }

    /// Fetch price from chain (mock implementation)
    async fn fetch_price_from_chain(&self, asset: &str) -> Result<f64, ArbitrageError> {
        // Mock prices with slight variations across chains
        let base_price = match asset {
            "USDC" | "USDT" => 1.0,
            "ETH" | "WETH" => 2400.0,
            "BTC" | "WBTC" => 45000.0,
            "SOL" => 95.0,
            "AVAX" => 25.0,
            "MATIC" => 0.85,
            "BNB" => 310.0,
            _ => return Err(ArbitrageError::AssetNotSupported(asset.to_string())),
        };

        // Add chain-specific variations (simulate price differences)
        let variation = match self.chain {
            ChainId::Ethereum => 1.0,      // Base price
            ChainId::Arbitrum => 0.998,    // Slightly lower
            ChainId::Optimism => 0.999,    // Slightly lower
            ChainId::Polygon => 1.002,     // Slightly higher
            ChainId::BSC => 1.001,         // Slightly higher
            ChainId::Solana => 0.997,      // Lower (more volatile)
            ChainId::Avalanche => 1.0015,  // Slightly higher
            _ => 1.0,
        };

        Ok(base_price * variation)
    }
}

/// DEX integration for executing trades
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DexIntegration {
    pub name: String,
    pub chain: ChainId,
    pub contract_address: String,
    pub fee_rate: f64,
    pub supported_assets: Vec<String>,
    pub liquidity_threshold: u64,
}

/// Bridge cost calculator
#[derive(Debug, Clone)]
pub struct BridgeCostCalculator {
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub base_fee: f64,
    pub variable_rate: f64,
}

impl BridgeCostCalculator {
    pub fn calculate_cost(&self, amount_usd: u64, _asset: &str) -> f64 {
        self.base_fee + (amount_usd as f64 * self.variable_rate)
    }
}

/// Gas estimator for chain-specific gas costs
#[derive(Debug, Clone)]
pub struct GasEstimator {
    pub chain: ChainId,
    pub current_gas_price: f64,
    pub swap_gas_limit: u64,
}

impl GasEstimator {
    pub fn estimate_swap_cost(&self, _amount_usd: u64) -> f64 {
        // Simplified gas cost calculation
        self.current_gas_price * (self.swap_gas_limit as f64 / 1_000_000.0)
    }
}

/// Cached price data
#[derive(Debug, Clone)]
pub struct CachedPrice {
    pub price: f64,
    pub timestamp: u64,
}

/// Arbitrage configuration
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ArbitrageConfiguration {
    pub supported_assets: Vec<String>,
    pub max_opportunity_age: u64,      // seconds
    pub min_profit_usd: f64,
    pub max_capital_per_trade: u64,
    pub max_slippage_tolerance: f64,
    pub enable_mev_protection: bool,
}

impl Default for ArbitrageConfiguration {
    fn default() -> Self {
        Self {
            supported_assets: vec![
                "USDC".to_string(),
                "USDT".to_string(),
                "ETH".to_string(),
                "WETH".to_string(),
                "BTC".to_string(),
                "WBTC".to_string(),
                "SOL".to_string(),
            ],
            max_opportunity_age: 60,        // 1 minute
            min_profit_usd: 10.0,           // $10 minimum profit
            max_capital_per_trade: 100_000, // $100k max per trade
            max_slippage_tolerance: 0.02,   // 2% max slippage
            enable_mev_protection: true,
        }
    }
}

/// Arbitrage opportunity
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub asset: String,
    pub buy_chain: ChainId,
    pub sell_chain: ChainId,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub max_capital_usd: u64,
    pub net_profit_usd: f64,
    pub execution_cost: ExecutionCost,
    pub execution_time_estimate: u64,
    pub confidence_score: f64,
    pub discovered_at: u64,
}

/// Execution cost breakdown
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ExecutionCost {
    pub buy_gas_cost: f64,
    pub sell_gas_cost: f64,
    pub bridge_cost: f64,
    pub buy_dex_fee: f64,
    pub sell_dex_fee: f64,
    pub total_cost: f64,
}

/// Transaction execution result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TransactionResult {
    pub transaction_hash: String,
    pub amount_in: f64,
    pub amount_out: f64,
    pub amount_received: f64,
    pub gas_used: f64,
    pub total_cost: f64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Bridge transfer result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BridgeResult {
    pub bridge_hash: String,
    pub amount_bridged: f64,
    pub cost: f64,
    pub time_taken: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Complete arbitrage execution result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ArbitrageExecutionResult {
    pub opportunity_id: String,
    pub success: bool,
    pub buy_result: TransactionResult,
    pub bridge_result: BridgeResult,
    pub sell_result: TransactionResult,
    pub actual_profit_usd: f64,
    pub expected_profit_usd: f64,
    pub total_execution_time: u64,
    pub gas_used_total: f64,
}

/// Arbitrage statistics
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ArbitrageStats {
    pub total_opportunities_found: u64,
    pub successful_arbitrages: u64,
    pub total_profit_usd: f64,
    pub average_execution_time: u64,
    pub supported_chains: usize,
    pub supported_assets: usize,
    pub last_scan_timestamp: u64,
}

/// Extension trait for ChainId
trait ChainIdExt {
    fn is_ethereum_l2(&self) -> bool;
}

impl ChainIdExt for ChainId {
    fn is_ethereum_l2(&self) -> bool {
        matches!(self, 
            ChainId::Arbitrum | 
            ChainId::Optimism | 
            ChainId::Polygon | 
            ChainId::Base
        )
    }
}

/// Arbitrage-specific errors
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum ArbitrageError {
    InsufficientPriceData(String),
    AssetNotSupported(String),
    OpportunityExpired(u64),
    InsufficientProfit(f64),
    ExecutionFailed(String),
    BridgeNotAvailable(String),
    InsufficientLiquidity(String),
    SlippageExceeded(f64),
    MEVAttackDetected,
    NetworkCongestion(String),
}

impl std::fmt::Display for ArbitrageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArbitrageError::InsufficientPriceData(asset) => write!(f, "Insufficient price data for asset: {}", asset),
            ArbitrageError::AssetNotSupported(asset) => write!(f, "Asset not supported: {}", asset),
            ArbitrageError::OpportunityExpired(age) => write!(f, "Opportunity expired (age: {} seconds)", age),
            ArbitrageError::InsufficientProfit(profit) => write!(f, "Insufficient profit: ${:.2}", profit),
            ArbitrageError::ExecutionFailed(reason) => write!(f, "Execution failed: {}", reason),
            ArbitrageError::BridgeNotAvailable(route) => write!(f, "Bridge not available for route: {}", route),
            ArbitrageError::InsufficientLiquidity(pool) => write!(f, "Insufficient liquidity in pool: {}", pool),
            ArbitrageError::SlippageExceeded(slippage) => write!(f, "Slippage exceeded: {:.2}%", slippage * 100.0),
            ArbitrageError::MEVAttackDetected => write!(f, "MEV attack detected, transaction cancelled"),
            ArbitrageError::NetworkCongestion(chain) => write!(f, "Network congestion on {}", chain),
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

    fn create_test_arbitrage_engine() -> CrossChainArbitrageEngine {
        let config = ArbitrageConfiguration::default();
        let mut engine = CrossChainArbitrageEngine::new(config);
        // Set test time manually to avoid canister-only function calls
        engine.last_scan = mock_time();
        engine
    }

    #[test]
    fn test_arbitrage_engine_creation() {
        let engine = create_test_arbitrage_engine();
        // Price oracles not initialized in tests to avoid canister-only functions
        assert_eq!(engine.profit_threshold, 0.005);
        assert_eq!(engine.arbitrage_config.min_profit_usd, 10.0);
        assert_eq!(engine.arbitrage_config.max_capital_per_trade, 100_000);
    }

    #[test]
    fn test_execution_cost_calculation() {
        let engine = create_test_arbitrage_engine();
        
        let buy_chain = ChainId::Ethereum;
        let sell_chain = ChainId::Arbitrum;
        let amount = 10000;
        
        let buy_gas = engine.default_gas_cost(&buy_chain);
        let sell_gas = engine.default_gas_cost(&sell_chain);
        let bridge_cost = engine.default_bridge_cost(&buy_chain, &sell_chain, amount);
        
        assert!(buy_gas > 0.0);
        assert!(sell_gas > 0.0);
        assert!(bridge_cost > 0.0);
    }

    #[test]
    fn test_chain_execution_times() {
        let engine = create_test_arbitrage_engine();
        
        assert_eq!(engine.chain_execution_time(&ChainId::Solana), 5);     // Fastest
        assert_eq!(engine.chain_execution_time(&ChainId::Bitcoin), 1800); // Slowest
        assert_eq!(engine.chain_execution_time(&ChainId::Ethereum), 300); // Medium
    }

    #[test]
    fn test_confidence_score_calculation() {
        let engine = create_test_arbitrage_engine();
        
        let score_eth_arb = engine.calculate_confidence_score(&ChainId::Ethereum, &ChainId::Arbitrum, "USDC");
        let score_exotic = engine.calculate_confidence_score(&ChainId::Sonic, &ChainId::BSC, "UNKNOWN");
        
        assert!(score_eth_arb > score_exotic);
        assert!(score_eth_arb <= 1.0);
        assert!(score_exotic >= 0.0);
    }

    #[test]
    fn test_dex_fee_calculation() {
        let engine = create_test_arbitrage_engine();
        
        let eth_fee = engine.calculate_dex_fee(&ChainId::Ethereum, 10000);
        let sol_fee = engine.calculate_dex_fee(&ChainId::Solana, 10000);
        
        assert_eq!(eth_fee, 30.0); // 0.3% of $10,000
        assert_eq!(sol_fee, 25.0); // 0.25% of $10,000
    }

    #[test]
    fn test_bridge_cost_variations() {
        let engine = create_test_arbitrage_engine();
        
        let eth_to_arb = engine.default_bridge_cost(&ChainId::Ethereum, &ChainId::Arbitrum, 1000);
        let eth_to_sol = engine.default_bridge_cost(&ChainId::Ethereum, &ChainId::Solana, 1000);
        
        assert!(eth_to_sol > eth_to_arb); // Cross-chain more expensive than L2
    }

    #[test]
    fn test_arbitrage_configuration_defaults() {
        let config = ArbitrageConfiguration::default();
        
        assert!(!config.supported_assets.is_empty());
        assert!(config.supported_assets.contains(&"USDC".to_string()));
        assert!(config.supported_assets.contains(&"ETH".to_string()));
        assert_eq!(config.max_opportunity_age, 60);
        assert_eq!(config.min_profit_usd, 10.0);
    }
}