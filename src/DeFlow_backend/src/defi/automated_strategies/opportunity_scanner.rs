// Opportunity Scanner - Discover and analyze DeFi opportunities
// Real-time scanning of yield farming, arbitrage, and other DeFi opportunities

use super::*;
use crate::defi::yield_farming::ChainId;
use crate::defi::yield_farming::{DeFiProtocol, YieldStrategy};
use crate::defi::arbitrage::ArbitrageOpportunity;
use crate::defi::price_oracle::{CrossChainPriceOracle, OracleError};
use crate::defi::protocol_integrations::{DeFiProtocolIntegrations, IntegrationError};

/// Comprehensive opportunity scanner for all DeFi strategies
#[derive(Debug, Clone)]
pub struct OpportunityScanner {
    pub yield_scanner: YieldOpportunityScanner,
    pub arbitrage_scanner: ArbitrageOpportunityScanner,
    pub liquidity_scanner: LiquidityOpportunityScanner,
    pub rebalancing_scanner: RebalancingOpportunityScanner,
    pub protocol_integrations: DeFiProtocolIntegrations,
    pub opportunity_cache: HashMap<String, CachedOpportunity>,
    pub scan_intervals: ScanIntervals,
    pub filters: OpportunityFilters,
    pub last_scan_times: HashMap<String, u64>,
}

impl OpportunityScanner {
    pub fn new() -> Self {
        Self {
            yield_scanner: YieldOpportunityScanner::new(),
            arbitrage_scanner: ArbitrageOpportunityScanner::new(),
            liquidity_scanner: LiquidityOpportunityScanner::new(),
            rebalancing_scanner: RebalancingOpportunityScanner::new(),
            protocol_integrations: DeFiProtocolIntegrations::new(),
            opportunity_cache: HashMap::new(),
            scan_intervals: ScanIntervals::default(),
            filters: OpportunityFilters::default(),
            last_scan_times: HashMap::new(),
        }
    }

    /// Initialize scanner with default configurations
    pub async fn initialize(&mut self) -> Result<(), StrategyError> {
        self.yield_scanner.initialize();
        self.arbitrage_scanner.initialize();
        self.liquidity_scanner.initialize();
        self.rebalancing_scanner.initialize();
        
        self.protocol_integrations.initialize().await
            .map_err(|e| StrategyError::ExecutionFailed(format!("Protocol integration initialization failed: {}", e)))?;
        
        Ok(())
    }

    /// Main opportunity scanning function
    pub async fn scan_opportunities(&mut self) -> Result<Vec<StrategyOpportunity>, StrategyError> {
        let current_time = self.get_current_time();
        let mut all_opportunities = Vec::new();

        // Scan yield farming opportunities using real protocol integrations
        if self.should_scan("yield_farming", current_time) {
            let live_yield_opportunities = self.protocol_integrations.get_yield_farming_opportunities().await
                .map_err(|e| StrategyError::ExecutionFailed(format!("Failed to get yield opportunities: {}", e)))?;
            
            let strategy_opportunities = self.convert_live_yield_to_strategy_opportunities(live_yield_opportunities);
            all_opportunities.extend(strategy_opportunities);
            self.last_scan_times.insert("yield_farming".to_string(), current_time);
        }

        // Scan arbitrage opportunities using real protocol integrations  
        if self.should_scan("arbitrage", current_time) {
            let live_arbitrage_opportunities = self.protocol_integrations.get_arbitrage_opportunities().await
                .map_err(|e| StrategyError::ExecutionFailed(format!("Failed to get arbitrage opportunities: {}", e)))?;
            
            let strategy_opportunities = self.convert_live_arbitrage_to_strategy_opportunities(live_arbitrage_opportunities);
            all_opportunities.extend(strategy_opportunities);
            self.last_scan_times.insert("arbitrage".to_string(), current_time);
        }

        // Scan liquidity mining opportunities (included in yield farming above)
        if self.should_scan("liquidity_mining", current_time) {
            // Liquidity mining opportunities are now included in yield farming scan above
            self.last_scan_times.insert("liquidity_mining".to_string(), current_time);
        }

        // Scan rebalancing opportunities (portfolio-based, uses existing logic)
        if self.should_scan("rebalancing", current_time) {
            let rebalancing_opportunities = self.rebalancing_scanner.scan_rebalancing_opportunities().await?;
            all_opportunities.extend(rebalancing_opportunities);
            self.last_scan_times.insert("rebalancing".to_string(), current_time);
        }

        // Filter and rank opportunities
        let filtered_opportunities = self.apply_filters(all_opportunities);
        let ranked_opportunities = self.rank_opportunities(filtered_opportunities);

        // Update cache
        self.update_opportunity_cache(&ranked_opportunities);

        Ok(ranked_opportunities)
    }

    /// Get cached opportunities for quick access
    pub fn get_cached_opportunities(&self, strategy_type: Option<&str>) -> Vec<StrategyOpportunity> {
        let current_time = self.get_current_time();
        
        self.opportunity_cache.values()
            .filter(|cached| {
                // Filter by strategy type if specified
                if let Some(s_type) = strategy_type {
                    if !self.opportunity_matches_strategy_type(&cached.opportunity, s_type) {
                        return false;
                    }
                }
                
                // Check if opportunity is still valid
                current_time - cached.cached_at < 300_000_000_000 // 5 minutes cache
            })
            .map(|cached| cached.opportunity.clone())
            .collect()
    }

    /// Get opportunities for specific chain
    pub fn get_opportunities_by_chain(&self, chain: &ChainId) -> Vec<StrategyOpportunity> {
        self.opportunity_cache.values()
            .filter(|cached| &cached.opportunity.chain == chain)
            .map(|cached| cached.opportunity.clone())
            .collect()
    }

    /// Get top opportunities by expected return
    pub fn get_top_opportunities(&self, limit: usize) -> Vec<StrategyOpportunity> {
        let mut opportunities: Vec<StrategyOpportunity> = self.opportunity_cache.values()
            .map(|cached| cached.opportunity.clone())
            .collect();
        
        opportunities.sort_by(|a, b| b.expected_return_percentage.partial_cmp(&a.expected_return_percentage).unwrap());
        opportunities.truncate(limit);
        
        opportunities
    }

    /// Set custom scanning intervals
    pub fn set_scan_intervals(&mut self, intervals: ScanIntervals) {
        self.scan_intervals = intervals;
    }

    /// Set opportunity filters
    pub fn set_filters(&mut self, filters: OpportunityFilters) {
        self.filters = filters;
    }

    /// Get scanning statistics
    pub fn get_scan_statistics(&self) -> ScanStatistics {
        let current_time = self.get_current_time();
        let total_cached = self.opportunity_cache.len();
        
        let by_type = self.group_opportunities_by_type();
        let by_chain = self.group_opportunities_by_chain();
        
        let avg_return = if total_cached > 0 {
            self.opportunity_cache.values()
                .map(|cached| cached.opportunity.expected_return_percentage)
                .sum::<f64>() / total_cached as f64
        } else {
            0.0
        };

        ScanStatistics {
            total_opportunities: total_cached,
            opportunities_by_type: by_type,
            opportunities_by_chain: by_chain,
            average_expected_return: avg_return,
            last_scan_times: self.last_scan_times.clone(),
            cache_hit_rate: 85.2, // Mock value
            scan_duration_ms: 150, // Mock value
            last_updated: current_time,
        }
    }

    /// Convert live yield opportunities to strategy opportunities
    fn convert_live_yield_to_strategy_opportunities(&self, live_opportunities: Vec<crate::defi::protocol_integrations::LiveYieldOpportunity>) -> Vec<StrategyOpportunity> {
        live_opportunities.into_iter().map(|live_opp| {
            let opportunity_type = match live_opp.opportunity_type {
                crate::defi::protocol_integrations::YieldOpportunityType::YieldFarming => {
                    OpportunityType::YieldFarming {
                        apy: live_opp.apy,
                        tokens: live_opp.tokens,
                        pool_address: live_opp.pool_address,
                    }
                },
                crate::defi::protocol_integrations::YieldOpportunityType::LiquidityMining => {
                    OpportunityType::LiquidityMining {
                        apr: live_opp.apy,
                        reward_tokens: live_opp.tokens.clone(),
                        pool_info: format!("{} on {}", live_opp.pool_address, live_opp.protocol.to_string()),
                    }
                },
                crate::defi::protocol_integrations::YieldOpportunityType::Lending => {
                    OpportunityType::YieldFarming {
                        apy: live_opp.apy,
                        tokens: live_opp.tokens,
                        pool_address: live_opp.pool_address,
                    }
                },
                crate::defi::protocol_integrations::YieldOpportunityType::Staking => {
                    OpportunityType::YieldFarming {
                        apy: live_opp.apy,
                        tokens: live_opp.tokens,
                        pool_address: live_opp.pool_address,
                    }
                },
            };

            StrategyOpportunity {
                id: live_opp.id,
                opportunity_type,
                chain: self.convert_chain_id(&live_opp.chain),
                protocol: live_opp.protocol,
                expected_return_percentage: live_opp.apy,
                risk_score: self.calculate_risk_score_from_factors(&live_opp.risk_factors),
                estimated_gas_cost: live_opp.gas_cost_estimate_usd,
                liquidity_score: self.calculate_liquidity_score(live_opp.total_liquidity_usd),
                time_sensitivity_minutes: 60, // Default for yield opportunities
                discovered_at: live_opp.last_updated,
                expires_at: live_opp.last_updated + (60 * 60 * 1_000_000_000), // 1 hour expiry
            }
        }).collect()
    }

    /// Convert live arbitrage opportunities to strategy opportunities
    fn convert_live_arbitrage_to_strategy_opportunities(&self, live_opportunities: Vec<crate::defi::protocol_integrations::LiveArbitrageOpportunity>) -> Vec<StrategyOpportunity> {
        live_opportunities.into_iter().map(|live_opp| {
            StrategyOpportunity {
                id: live_opp.id,
                opportunity_type: OpportunityType::Arbitrage {
                    profit_percentage: live_opp.profit_percentage,
                    token_pair: live_opp.token_pair,
                    dex_pair: (live_opp.buy_dex, live_opp.sell_dex),
                },
                chain: self.convert_chain_id(&live_opp.chain),
                protocol: DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3), // Use Uniswap as default
                expected_return_percentage: live_opp.profit_percentage,
                risk_score: self.calculate_risk_score_from_confidence(live_opp.confidence_score),
                estimated_gas_cost: live_opp.estimated_gas_cost_usd,
                liquidity_score: self.calculate_liquidity_score_from_trade_size(live_opp.max_trade_size_usd),
                time_sensitivity_minutes: (live_opp.time_sensitivity_seconds / 60) as u64,
                discovered_at: live_opp.last_updated,
                expires_at: live_opp.last_updated + (live_opp.time_sensitivity_seconds as u64 * 1_000_000_000),
            }
        }).collect()
    }

    /// Calculate risk score from risk factors
    fn calculate_risk_score_from_factors(&self, risk_factors: &[String]) -> u8 {
        let base_risk = 5u8;
        let additional_risk = risk_factors.len() as u8;
        (base_risk + additional_risk).min(10)
    }

    /// Calculate liquidity score from total liquidity
    fn calculate_liquidity_score(&self, total_liquidity_usd: f64) -> f64 {
        // Scale liquidity to 0-10 score
        if total_liquidity_usd > 100_000_000.0 {
            10.0
        } else if total_liquidity_usd > 10_000_000.0 {
            8.0
        } else if total_liquidity_usd > 1_000_000.0 {
            6.0
        } else if total_liquidity_usd > 100_000.0 {
            4.0
        } else {
            2.0
        }
    }

    /// Calculate risk score from confidence score
    fn calculate_risk_score_from_confidence(&self, confidence_score: f64) -> u8 {
        // Inverse relationship: higher confidence = lower risk
        let risk_score = 10.0 - (confidence_score / 10.0);
        risk_score.max(1.0).min(10.0) as u8
    }

    /// Calculate liquidity score from max trade size
    fn calculate_liquidity_score_from_trade_size(&self, max_trade_size_usd: f64) -> f64 {
        // Scale trade size to 0-10 score
        (max_trade_size_usd / 10_000.0).min(10.0).max(1.0)
    }

    /// Helper method to match chains (handles ChainId::Custom variants)
    fn chains_match(&self, chain1: &ChainId, chain2: &ChainId) -> bool {
        match (chain1, chain2) {
            // Remove custom chain handling since ChainId doesn't have Custom variant
            _ => chain1 == chain2,
        }
    }

    /// Helper method to get chain name 
    fn get_chain_name(&self, chain: &ChainId) -> String {
        chain.name().to_string()
    }

    // Private helper methods
    fn should_scan(&self, scan_type: &str, current_time: u64) -> bool {
        let last_scan = self.last_scan_times.get(scan_type).unwrap_or(&0);
        let interval = match scan_type {
            "yield_farming" => self.scan_intervals.yield_farming_seconds,
            "arbitrage" => self.scan_intervals.arbitrage_seconds,
            "liquidity_mining" => self.scan_intervals.liquidity_mining_seconds,
            "rebalancing" => self.scan_intervals.rebalancing_seconds,
            _ => 300, // 5 minutes default
        };
        
        current_time - last_scan > interval * 1_000_000_000
    }

    fn apply_filters(&self, opportunities: Vec<StrategyOpportunity>) -> Vec<StrategyOpportunity> {
        opportunities.into_iter()
            .filter(|opp| {
                // Minimum return filter
                if opp.expected_return_percentage < self.filters.min_expected_return {
                    return false;
                }
                
                // Maximum risk filter
                if opp.risk_score > self.filters.max_risk_score {
                    return false;
                }
                
                // Chain filter
                if !self.filters.allowed_chains.is_empty() && !self.filters.allowed_chains.contains(&opp.chain) {
                    return false;
                }
                
                // Protocol filter
                if !self.filters.allowed_protocols.is_empty() && !self.filters.allowed_protocols.contains(&opp.protocol) {
                    return false;
                }
                
                // Minimum liquidity filter
                if opp.liquidity_score < self.filters.min_liquidity_score {
                    return false;
                }
                
                true
            })
            .collect()
    }

    fn rank_opportunities(&self, mut opportunities: Vec<StrategyOpportunity>) -> Vec<StrategyOpportunity> {
        opportunities.sort_by(|a, b| {
            let score_a = self.calculate_opportunity_score(a);
            let score_b = self.calculate_opportunity_score(b);
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        opportunities
    }

    fn calculate_opportunity_score(&self, opportunity: &StrategyOpportunity) -> f64 {
        let mut score = 0.0;
        
        // Expected return weight (40%)
        score += opportunity.expected_return_percentage * 0.4;
        
        // Risk adjustment (30%)
        let risk_adjustment = (10.0 - opportunity.risk_score as f64) / 10.0;
        score += risk_adjustment * 30.0;
        
        // Liquidity score weight (20%)
        score += opportunity.liquidity_score * 0.2;
        
        // Time sensitivity bonus (10%)
        let time_bonus = if opportunity.time_sensitivity_minutes < 30 {
            10.0
        } else if opportunity.time_sensitivity_minutes < 60 {
            5.0
        } else {
            0.0
        };
        score += time_bonus * 0.1;
        
        score
    }

    fn update_opportunity_cache(&mut self, opportunities: &[StrategyOpportunity]) {
        let current_time = self.get_current_time();
        
        // Clear expired opportunities
        self.opportunity_cache.retain(|_, cached| {
            current_time - cached.cached_at < 600_000_000_000 // 10 minutes max cache
        });
        
        // Add new opportunities
        for opportunity in opportunities {
            self.opportunity_cache.insert(
                opportunity.id.clone(),
                CachedOpportunity {
                    opportunity: opportunity.clone(),
                    cached_at: current_time,
                    access_count: 0,
                }
            );
        }
    }

    fn opportunity_matches_strategy_type(&self, opportunity: &StrategyOpportunity, strategy_type: &str) -> bool {
        match (&opportunity.opportunity_type, strategy_type) {
            (OpportunityType::YieldFarming { .. }, "yield_farming") => true,
            (OpportunityType::Arbitrage { .. }, "arbitrage") => true,
            (OpportunityType::LiquidityMining { .. }, "liquidity_mining") => true,
            (OpportunityType::Rebalancing { .. }, "rebalancing") => true,
            _ => false,
        }
    }

    fn group_opportunities_by_type(&self) -> HashMap<String, usize> {
        let mut by_type = HashMap::new();
        
        for cached in self.opportunity_cache.values() {
            let type_name = match &cached.opportunity.opportunity_type {
                OpportunityType::YieldFarming { .. } => "yield_farming",
                OpportunityType::Arbitrage { .. } => "arbitrage",
                OpportunityType::LiquidityMining { .. } => "liquidity_mining",
                OpportunityType::Rebalancing { .. } => "rebalancing",
            };
            
            *by_type.entry(type_name.to_string()).or_insert(0) += 1;
        }
        
        by_type
    }

    fn group_opportunities_by_chain(&self) -> HashMap<String, usize> {
        let mut by_chain = HashMap::new();
        
        for cached in self.opportunity_cache.values() {
            let chain_name = cached.opportunity.chain.name().to_string();
            *by_chain.entry(chain_name).or_insert(0) += 1;
        }
        
        by_chain
    }

    /// Convert defi::types::ChainId to yield_farming::ChainId
    fn convert_chain_id(&self, chain: &crate::defi::types::ChainId) -> ChainId {
        match chain {
            crate::defi::types::ChainId::Bitcoin => ChainId::Bitcoin,
            crate::defi::types::ChainId::Ethereum => ChainId::Ethereum,
            crate::defi::types::ChainId::Arbitrum => ChainId::Arbitrum,
            crate::defi::types::ChainId::Optimism => ChainId::Optimism,
            crate::defi::types::ChainId::Polygon => ChainId::Polygon,
            crate::defi::types::ChainId::Base => ChainId::Base,
            crate::defi::types::ChainId::Avalanche => ChainId::Avalanche,
            crate::defi::types::ChainId::Custom(name) if name == "Sonic" => ChainId::Sonic,
            crate::defi::types::ChainId::Custom(name) if name == "BSC" => ChainId::BSC,
            crate::defi::types::ChainId::Custom(_) => ChainId::Ethereum, // Default fallback
            crate::defi::types::ChainId::Solana => ChainId::Solana,
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

/// Yield farming opportunity scanner
#[derive(Debug, Clone)]
pub struct YieldOpportunityScanner {
    pub monitored_protocols: Vec<DeFiProtocol>,
    pub min_apy_threshold: f64,
    pub preferred_tokens: Vec<String>,
}

impl YieldOpportunityScanner {
    pub fn new() -> Self {
        Self {
            monitored_protocols: Vec::new(),
            min_apy_threshold: 3.0,
            preferred_tokens: Vec::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.monitored_protocols = vec![
            DeFiProtocol::Aave,
            DeFiProtocol::Compound,
            DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3),
        ];
        
        self.preferred_tokens = vec![
            "USDC".to_string(),
            "USDT".to_string(),
            "DAI".to_string(),
            "ETH".to_string(),
            "BTC".to_string(),
        ];
    }

    pub async fn scan_yield_opportunities(&self) -> Result<Vec<StrategyOpportunity>, StrategyError> {
        let mut opportunities = Vec::new();
        let current_time = self.get_current_time();

        // Scan each supported chain
        for chain in &[ChainId::Ethereum, ChainId::Arbitrum, ChainId::Polygon, ChainId::Solana] {
            for protocol in &self.monitored_protocols {
                let yield_opportunities = self.scan_protocol_yields(chain, protocol).await?;
                opportunities.extend(yield_opportunities);
            }
        }

        // Filter by minimum APY
        opportunities.retain(|opp| {
            if let OpportunityType::YieldFarming { apy, .. } = &opp.opportunity_type {
                *apy >= self.min_apy_threshold
            } else {
                false
            }
        });

        Ok(opportunities)
    }

    async fn scan_protocol_yields(&self, chain: &ChainId, protocol: &DeFiProtocol) -> Result<Vec<StrategyOpportunity>, StrategyError> {
        let mut opportunities = Vec::new();
        let current_time = self.get_current_time();

        // Mock yield farming opportunities (in production, this would query actual protocols)
        let mock_opportunities = vec![
            // Ethereum Aave USDC
            (15.2, vec!["USDC".to_string()], "0xaave_usdc_pool", ChainId::Ethereum, DeFiProtocol::Aave, 3, 8.5, 120),
            // Arbitrum Uniswap V3 ETH/USDC
            (22.8, vec!["ETH".to_string(), "USDC".to_string()], "0xuniswap_eth_usdc", ChainId::Arbitrum, DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3), 5, 7.2, 60),
            // Polygon Compound DAI
            (8.9, vec!["DAI".to_string()], "0xcompound_dai", ChainId::Polygon, DeFiProtocol::Compound, 2, 9.1, 240),
        ];

        for (apy, tokens, pool_address, opp_chain, opp_protocol, risk_score, liquidity_score, time_sensitivity) in mock_opportunities {
            if opp_chain == *chain && opp_protocol == *protocol {
                let opportunity = StrategyOpportunity {
                    id: format!("yf_{}_{}_{}_{:x}", chain.name(), format!("{:?}", protocol), tokens.join("_"), current_time),
                    opportunity_type: OpportunityType::YieldFarming {
                        apy,
                        tokens: tokens.clone(),
                        pool_address: pool_address.to_string(),
                    },
                    chain: chain.clone(),
                    protocol: protocol.clone(),
                    expected_return_percentage: apy,
                    risk_score,
                    estimated_gas_cost: self.estimate_gas_cost(chain),
                    liquidity_score,
                    time_sensitivity_minutes: time_sensitivity,
                    discovered_at: current_time,
                    expires_at: current_time + (time_sensitivity as u64 * 60 * 1_000_000_000),
                };
                
                opportunities.push(opportunity);
            }
        }

        Ok(opportunities)
    }

    fn estimate_gas_cost(&self, chain: &ChainId) -> f64 {
        match chain {
            ChainId::Ethereum => 75.0,
            ChainId::Arbitrum => 15.0,
            ChainId::Polygon => 8.0,
            ChainId::Solana => 2.0,
            _ => 25.0,
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

    fn get_chain_name(&self, chain: &ChainId) -> String {
        chain.name().to_string()
    }
}

/// Arbitrage opportunity scanner
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunityScanner {
    pub monitored_dexes: Vec<String>,
    pub token_pairs: Vec<(String, String)>,
    pub min_profit_threshold: f64,
}

impl ArbitrageOpportunityScanner {
    pub fn new() -> Self {
        Self {
            monitored_dexes: Vec::new(),
            token_pairs: Vec::new(),
            min_profit_threshold: 0.5,
        }
    }

    pub fn initialize(&mut self) {
        self.monitored_dexes = vec![
            "Uniswap".to_string(),
            "SushiSwap".to_string(),
            "PancakeSwap".to_string(),
            "QuickSwap".to_string(),
        ];
        
        self.token_pairs = vec![
            ("ETH".to_string(), "USDC".to_string()),
            ("BTC".to_string(), "USDC".to_string()),
            ("ETH".to_string(), "BTC".to_string()),
            ("MATIC".to_string(), "USDC".to_string()),
        ];
    }

    pub async fn scan_arbitrage_opportunities(&self) -> Result<Vec<StrategyOpportunity>, StrategyError> {
        let mut opportunities = Vec::new();
        let current_time = self.get_current_time();

        // Scan each chain for arbitrage opportunities
        for chain in &[ChainId::Ethereum, ChainId::Arbitrum, ChainId::BSC, ChainId::Polygon] {
            for token_pair in &self.token_pairs {
                let arb_opportunities = self.scan_token_pair_arbitrage(chain, token_pair).await?;
                opportunities.extend(arb_opportunities);
            }
        }

        // Filter by minimum profit
        opportunities.retain(|opp| {
            if let OpportunityType::Arbitrage { profit_percentage, .. } = &opp.opportunity_type {
                *profit_percentage >= self.min_profit_threshold
            } else {
                false
            }
        });

        Ok(opportunities)
    }

    async fn scan_token_pair_arbitrage(&self, chain: &ChainId, token_pair: &(String, String)) -> Result<Vec<StrategyOpportunity>, StrategyError> {
        let mut opportunities = Vec::new();
        let current_time = self.get_current_time();

        // Mock arbitrage opportunities (in production, this would query actual DEX prices)
        let mock_opportunities = vec![
            // ETH/USDC arbitrage opportunities
            (1.8, ("ETH".to_string(), "USDC".to_string()), ("Uniswap".to_string(), "SushiSwap".to_string()), ChainId::Ethereum, 4, 25.0, 8.9, 15),
            (2.3, ("BTC".to_string(), "USDC".to_string()), ("Uniswap".to_string(), "QuickSwap".to_string()), ChainId::Polygon, 5, 5.0, 7.5, 10),
            (0.9, ("MATIC".to_string(), "USDC".to_string()), ("QuickSwap".to_string(), "SushiSwap".to_string()), ChainId::Polygon, 3, 3.0, 9.2, 20),
        ];

        for (profit_pct, opp_token_pair, dex_pair, opp_chain, risk_score, gas_cost, liquidity_score, time_sensitivity) in mock_opportunities {
            if opp_chain == *chain && opp_token_pair == *token_pair {
                let opportunity = StrategyOpportunity {
                    id: format!("arb_{}_{}_{}_{}_{:x}", chain.name(), token_pair.0, token_pair.1, dex_pair.0, current_time),
                    opportunity_type: OpportunityType::Arbitrage {
                        profit_percentage: profit_pct,
                        token_pair: token_pair.clone(),
                        dex_pair: dex_pair.clone(),
                    },
                    chain: chain.clone(),
                    protocol: DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3), // Default to Uniswap
                    expected_return_percentage: profit_pct,
                    risk_score,
                    estimated_gas_cost: gas_cost,
                    liquidity_score,
                    time_sensitivity_minutes: time_sensitivity,
                    discovered_at: current_time,
                    expires_at: current_time + (time_sensitivity as u64 * 60 * 1_000_000_000),
                };
                
                opportunities.push(opportunity);
            }
        }

        Ok(opportunities)
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

/// Liquidity mining opportunity scanner
#[derive(Debug, Clone)]
pub struct LiquidityOpportunityScanner {
    pub min_apr_threshold: f64,
    pub monitored_pools: Vec<String>,
    pub impermanent_loss_tolerance: f64,
}

impl LiquidityOpportunityScanner {
    pub fn new() -> Self {
        Self {
            min_apr_threshold: 8.0,
            monitored_pools: Vec::new(),
            impermanent_loss_tolerance: 10.0,
        }
    }

    pub fn initialize(&mut self) {
        self.monitored_pools = vec![
            "ETH_USDC_3000".to_string(),
            "ETH_USDC_500".to_string(),
            "BTC_ETH_3000".to_string(),
            "MATIC_USDC_500".to_string(),
        ];
    }

    pub async fn scan_liquidity_opportunities(&self) -> Result<Vec<StrategyOpportunity>, StrategyError> {
        let mut opportunities = Vec::new();
        let current_time = self.get_current_time();

        // Mock liquidity mining opportunities
        let mock_opportunities = vec![
            (28.5, vec!["LM".to_string(), "REWARDS".to_string()], "ETH_USDC_LP_Pool", ChainId::Ethereum, 6, 45.0, 8.2, 180),
            (35.2, vec!["MATIC".to_string(), "QUICK".to_string()], "MATIC_USDC_LP_Pool", ChainId::Polygon, 7, 8.0, 7.8, 120),
            (19.8, vec!["ARB".to_string()], "ETH_ARB_LP_Pool", ChainId::Arbitrum, 5, 12.0, 8.9, 240),
        ];

        for (apr, reward_tokens, pool_info, chain, risk_score, gas_cost, liquidity_score, time_sensitivity) in mock_opportunities {
            if apr >= self.min_apr_threshold {
                let opportunity = StrategyOpportunity {
                    id: format!("lm_{}_{:x}", pool_info, current_time),
                    opportunity_type: OpportunityType::LiquidityMining {
                        apr,
                        reward_tokens: reward_tokens.clone(),
                        pool_info: pool_info.to_string(),
                    },
                    chain,
                    protocol: DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3),
                    expected_return_percentage: apr,
                    risk_score,
                    estimated_gas_cost: gas_cost,
                    liquidity_score,
                    time_sensitivity_minutes: time_sensitivity,
                    discovered_at: current_time,
                    expires_at: current_time + (time_sensitivity as u64 * 60 * 1_000_000_000),
                };
                
                opportunities.push(opportunity);
            }
        }

        Ok(opportunities)
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

/// Portfolio rebalancing opportunity scanner
#[derive(Debug, Clone)]
pub struct RebalancingOpportunityScanner {
    pub drift_threshold: f64,
    pub monitored_allocations: Vec<String>,
}

impl RebalancingOpportunityScanner {
    pub fn new() -> Self {
        Self {
            drift_threshold: 5.0, // 5% drift threshold
            monitored_allocations: Vec::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.monitored_allocations = vec![
            "BTC".to_string(),
            "ETH".to_string(),
            "Stablecoins".to_string(),
            "Alts".to_string(),
        ];
    }

    pub async fn scan_rebalancing_opportunities(&self) -> Result<Vec<StrategyOpportunity>, StrategyError> {
        let mut opportunities = Vec::new();
        let current_time = self.get_current_time();

        // Mock rebalancing opportunities based on market conditions
        let mock_opportunities = vec![
            // Portfolio drifted due to ETH price movement
            (
                {
                    let mut current = HashMap::new();
                    current.insert("BTC".to_string(), 35.0);
                    current.insert("ETH".to_string(), 40.0);  // Target was 30%
                    current.insert("Stablecoins".to_string(), 20.0);
                    current.insert("Alts".to_string(), 5.0);
                    current
                },
                {
                    let mut target = HashMap::new();
                    target.insert("BTC".to_string(), 40.0);
                    target.insert("ETH".to_string(), 30.0);
                    target.insert("Stablecoins".to_string(), 20.0);
                    target.insert("Alts".to_string(), 10.0);
                    target
                },
                ChainId::Ethereum,
                3,
                35.0,
                9.1,
                360
            ),
        ];

        for (current_allocation, target_allocation, chain, risk_score, gas_cost, liquidity_score, time_sensitivity) in mock_opportunities {
            // Calculate drift
            let max_drift = target_allocation.iter()
                .map(|(asset, target_pct)| {
                    let current_pct = current_allocation.get(asset).unwrap_or(&0.0);
                    ((*target_pct - *current_pct) as f64).abs()
                })
                .fold(0.0, f64::max);

            if max_drift >= self.drift_threshold {
                let opportunity = StrategyOpportunity {
                    id: format!("reb_{}_drift_{:.1}_{:x}", chain.name(), max_drift, current_time),
                    opportunity_type: OpportunityType::Rebalancing {
                        current_allocation,
                        target_allocation,
                    },
                    chain,
                    protocol: DeFiProtocol::Uniswap(crate::defi::yield_farming::UniswapVersion::V3),
                    expected_return_percentage: 0.0, // Rebalancing doesn't generate direct returns
                    risk_score,
                    estimated_gas_cost: gas_cost,
                    liquidity_score,
                    time_sensitivity_minutes: time_sensitivity,
                    discovered_at: current_time,
                    expires_at: current_time + (time_sensitivity as u64 * 60 * 1_000_000_000),
                };
                
                opportunities.push(opportunity);
            }
        }

        Ok(opportunities)
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

// MarketDataProvider removed - now using real protocol integrations via DeFiProtocolIntegrations

// Supporting data structures

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ScanIntervals {
    pub yield_farming_seconds: u64,
    pub arbitrage_seconds: u64,
    pub liquidity_mining_seconds: u64,
    pub rebalancing_seconds: u64,
}

impl Default for ScanIntervals {
    fn default() -> Self {
        Self {
            yield_farming_seconds: 300,  // 5 minutes
            arbitrage_seconds: 30,       // 30 seconds
            liquidity_mining_seconds: 600, // 10 minutes
            rebalancing_seconds: 3600,   // 1 hour
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct OpportunityFilters {
    pub min_expected_return: f64,
    pub max_risk_score: u8,
    pub min_liquidity_score: f64,
    pub allowed_chains: Vec<ChainId>,
    pub allowed_protocols: Vec<DeFiProtocol>,
    pub max_gas_cost_usd: f64,
}

impl Default for OpportunityFilters {
    fn default() -> Self {
        Self {
            min_expected_return: 3.0,
            max_risk_score: 8,
            min_liquidity_score: 5.0,
            allowed_chains: vec![],  // Empty means all allowed
            allowed_protocols: vec![], // Empty means all allowed
            max_gas_cost_usd: 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedOpportunity {
    pub opportunity: StrategyOpportunity,
    pub cached_at: u64,
    pub access_count: u32,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ScanStatistics {
    pub total_opportunities: usize,
    pub opportunities_by_type: HashMap<String, usize>,
    pub opportunities_by_chain: HashMap<String, usize>,
    pub average_expected_return: f64,
    pub last_scan_times: HashMap<String, u64>,
    pub cache_hit_rate: f64,
    pub scan_duration_ms: u64,
    pub last_updated: u64,
}

// TokenPrice and LiquidityInfo removed - now using real price oracle data