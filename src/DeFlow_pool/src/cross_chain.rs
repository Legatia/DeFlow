use crate::types::*;
use std::collections::HashMap;

pub struct CrossChainManager {
    // Supported chains and their capabilities
    pub chain_capabilities: HashMap<ChainId, ChainCapabilities>,
    
    // Price difference thresholds for arbitrage
    pub min_arbitrage_profit_threshold: f64,
    pub max_arbitrage_risk_score: f64,
}

impl CrossChainManager {
    pub fn new() -> Self {
        let mut chain_capabilities = HashMap::new();
        
        // Bitcoin capabilities
        chain_capabilities.insert(ChainId::Bitcoin, ChainCapabilities {
            native_assets: vec![Asset::BTC],
            supported_operations: vec!["send", "receive", "utxo_management"],
            avg_confirmation_time: 600, // 10 minutes
            finality_confidence: 0.99,
        });
        
        // Ethereum capabilities
        chain_capabilities.insert(ChainId::Ethereum, ChainCapabilities {
            native_assets: vec![Asset::ETH, Asset::USDC, Asset::USDT, Asset::DAI],
            supported_operations: vec!["send", "receive", "smart_contracts", "defi_protocols"],
            avg_confirmation_time: 180, // 3 minutes
            finality_confidence: 0.99,
        });
        
        // Solana capabilities
        chain_capabilities.insert(ChainId::Solana, ChainCapabilities {
            native_assets: vec![Asset::SOL],
            supported_operations: vec!["send", "receive", "programs", "spl_tokens"],
            avg_confirmation_time: 30, // 30 seconds
            finality_confidence: 0.95,
        });
        
        // Add other chains...
        
        CrossChainManager {
            chain_capabilities,
            min_arbitrage_profit_threshold: 0.005, // 0.5% minimum profit
            max_arbitrage_risk_score: 0.7, // Max 70% risk score
        }
    }
    
    // =============================================================================
    // ARBITRAGE DETECTION
    // =============================================================================
    
    pub fn detect_arbitrage_opportunities(&self, pool_state: &PoolState) -> Result<Vec<ArbitrageOpportunity>, String> {
        let mut opportunities = Vec::new();
        
        // Get all assets that exist on multiple chains
        let multi_chain_assets = self.get_multi_chain_assets();
        
        for asset in multi_chain_assets {
            if let Some(opportunity) = self.analyze_asset_for_arbitrage(&asset, pool_state) {
                if opportunity.expected_profit >= self.min_arbitrage_profit_threshold {
                    opportunities.push(opportunity);
                }
            }
        }
        
        // Sort by profit potential
        opportunities.sort_by(|a, b| b.expected_profit.partial_cmp(&a.expected_profit).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(opportunities)
    }
    
    fn analyze_asset_for_arbitrage(&self, asset: &Asset, pool_state: &PoolState) -> Option<ArbitrageOpportunity> {
        // Get secure price data across chains
        let price_data = match self.get_secure_price_data(asset) {
            Ok(data) => data,
            Err(error) => {
                ic_cdk::println!("SECURITY: Price data error for {:?}: {}", asset, error);
                return None;
            }
        };
        
        // Find the best buy and sell opportunities
        let cheapest = price_data.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?;
        let most_expensive = price_data.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?;
        
        if cheapest.0 == most_expensive.0 {
            return None; // Same chain
        }
        
        let price_difference = (most_expensive.1 - cheapest.1) / cheapest.1;
        
        if price_difference >= self.min_arbitrage_profit_threshold {
            let required_capital = self.estimate_required_capital(asset, &cheapest.0, pool_state);
            let expected_profit = required_capital * price_difference * 0.8; // Account for fees
            
            Some(ArbitrageOpportunity {
                asset_pair: (asset.to_string(), "USD".to_string()),
                buy_chain: cheapest.0.clone(),
                sell_chain: most_expensive.0.clone(),
                price_difference,
                expected_profit,
                required_capital,
                confidence_score: self.calculate_confidence_score(&cheapest.0, &most_expensive.0),
            })
        } else {
            None
        }
    }
    
    fn get_secure_price_data(&self, asset: &Asset) -> Result<Vec<(ChainId, f64)>, String> {
        // SECURITY: This should call real price oracles in production
        // TODO: Implement Pyth, Chainlink, or other secure oracle integration
        
        // For now, return controlled mock data with validation
        let price_data = match asset {
            Asset::ETH => vec![
                (ChainId::Ethereum, self.validate_price(2500.0, "ETH")?),
                (ChainId::Arbitrum, self.validate_price(2498.0, "ETH")?),
                (ChainId::Polygon, self.validate_price(2502.0, "ETH")?),
            ],
            Asset::USDC => vec![
                (ChainId::Ethereum, self.validate_price(1.000, "USDC")?),
                (ChainId::Arbitrum, self.validate_price(0.999, "USDC")?),
                (ChainId::Polygon, self.validate_price(1.001, "USDC")?),
                (ChainId::Solana, self.validate_price(0.998, "USDC")?),
            ],
            Asset::BTC => vec![
                (ChainId::Bitcoin, self.validate_price(45000.0, "BTC")?),
                (ChainId::Ethereum, self.validate_price(44950.0, "WBTC")?),
            ],
            _ => return Err(format!("Asset {:?} not supported for price data", asset)),
        };
        
        // SECURITY: Validate we have at least 2 chains for arbitrage
        if price_data.len() < 2 {
            return Err(format!("Insufficient price data for asset {:?}", asset));
        }
        
        Ok(price_data)
    }
    
    fn validate_price(&self, price: f64, asset_name: &str) -> Result<f64, String> {
        // SECURITY: Validate price data
        if price <= 0.0 || !price.is_finite() {
            return Err(format!("Invalid price for {}: {}", asset_name, price));
        }
        
        // SECURITY: Basic sanity checks for major assets
        match asset_name {
            "USDC" | "USDT" | "DAI" => {
                if price < 0.95 || price > 1.05 {
                    return Err(format!("Stablecoin price out of range: {} = {}", asset_name, price));
                }
            },
            "ETH" => {
                if price < 100.0 || price > 50000.0 {
                    return Err(format!("ETH price out of reasonable range: {}", price));
                }
            },
            "BTC" | "WBTC" => {
                if price < 1000.0 || price > 200000.0 {
                    return Err(format!("BTC price out of reasonable range: {}", price));
                }
            },
            _ => {} // Other assets - basic validation only
        }
        
        Ok(price)
    }
    
    // =============================================================================
    // CROSS-CHAIN EXECUTION
    // =============================================================================
    
    pub fn execute_cross_chain_trade(&mut self, 
        pool_state: &mut PoolState, 
        source_chain: ChainId, 
        dest_chain: ChainId, 
        asset: Asset, 
        amount: u64
    ) -> Result<String, String> {
        
        // Validate chains support the asset
        self.validate_chain_asset_support(&source_chain, &asset)?;
        self.validate_chain_asset_support(&dest_chain, &asset)?;
        
        // Check if we have sufficient liquidity on source chain
        let available_liquidity = self.get_available_liquidity(pool_state, &source_chain, &asset);
        if amount > available_liquidity {
            return Err(format!("Insufficient liquidity on {:?}. Available: {}, Requested: {}", 
                source_chain, available_liquidity, amount));
        }
        
        // Execute the cross-chain trade
        match pool_state.phase {
            PoolPhase::Bootstrapping { .. } => {
                self.execute_external_cross_chain_trade(source_chain, dest_chain, asset, amount)
            },
            PoolPhase::Active { .. } => {
                self.execute_hybrid_cross_chain_trade(pool_state, source_chain, dest_chain, asset, amount)
            },
            PoolPhase::Emergency { .. } => {
                Err("Cross-chain operations disabled during emergency".to_string())
            }
        }
    }
    
    fn execute_external_cross_chain_trade(&self, 
        source_chain: ChainId, 
        dest_chain: ChainId, 
        asset: Asset, 
        amount: u64
    ) -> Result<String, String> {
        // During bootstrap: use external bridges/DEXs only
        let execution_plan = format!(
            "External cross-chain trade: {} {} from {:?} to {:?} via bridge",
            amount, asset.to_string(), source_chain, dest_chain
        );
        
        // In production, this would:
        // 1. Select optimal bridge (LayerZero, Wormhole, etc.)
        // 2. Execute bridge transaction
        // 3. Monitor confirmation
        // 4. Handle any failures
        
        Ok(execution_plan)
    }
    
    fn execute_hybrid_cross_chain_trade(&self, 
        pool_state: &mut PoolState, 
        source_chain: ChainId, 
        dest_chain: ChainId, 
        asset: Asset, 
        amount: u64
    ) -> Result<String, String> {
        // During active phase: optimize between pool and external options
        let pool_liquidity = self.get_available_liquidity(pool_state, &dest_chain, &asset);
        
        if pool_liquidity >= amount {
            // Use pool for better execution
            self.execute_pool_based_trade(pool_state, source_chain, dest_chain, asset, amount)
        } else {
            // Fall back to external bridges
            self.execute_external_cross_chain_trade(source_chain, dest_chain, asset, amount)
        }
    }
    
    fn execute_pool_based_trade(&self, 
        _pool_state: &mut PoolState, 
        source_chain: ChainId, 
        dest_chain: ChainId, 
        asset: Asset, 
        amount: u64
    ) -> Result<String, String> {
        // Pool-based cross-chain trade with better rates
        let execution_plan = format!(
            "Pool-based cross-chain trade: {} {} from {:?} to {:?} via DeFlow pool",
            amount, asset.to_string(), source_chain, dest_chain
        );
        
        // In production, this would:
        // 1. Reserve liquidity on destination chain
        // 2. Execute source chain transaction
        // 3. Release liquidity on destination chain
        // 4. Update pool balances
        
        Ok(execution_plan)
    }
    
    // =============================================================================
    // UTILITY FUNCTIONS
    // =============================================================================
    
    fn get_multi_chain_assets(&self) -> Vec<Asset> {
        vec![Asset::ETH, Asset::USDC, Asset::USDT, Asset::BTC]
    }
    
    fn validate_chain_asset_support(&self, chain: &ChainId, asset: &Asset) -> Result<(), String> {
        if let Some(capabilities) = self.chain_capabilities.get(chain) {
            if capabilities.native_assets.contains(asset) || self.asset_supported_via_bridge(chain, asset) {
                Ok(())
            } else {
                Err(format!("Asset {:?} not supported on chain {:?}", asset, chain))
            }
        } else {
            Err(format!("Chain {:?} not supported", chain))
        }
    }
    
    fn asset_supported_via_bridge(&self, chain: &ChainId, asset: &Asset) -> bool {
        // Check if asset can be bridged to this chain
        match (chain, asset) {
            (ChainId::Ethereum, Asset::BTC) => true, // WBTC
            (ChainId::Arbitrum, Asset::ETH) => true, // Bridged ETH
            (ChainId::Arbitrum, Asset::USDC) => true, // Bridged USDC
            (ChainId::Polygon, Asset::ETH) => true, // Bridged ETH
            (ChainId::Polygon, Asset::USDC) => true, // Bridged USDC
            _ => false,
        }
    }
    
    fn get_available_liquidity(&self, pool_state: &PoolState, chain: &ChainId, asset: &Asset) -> u64 {
        pool_state.reserves
            .get(chain)
            .and_then(|chain_reserves| chain_reserves.get(asset))
            .map(|reserve| (reserve.total_amount as f64 * 0.8) as u64) // 80% utilization max
            .unwrap_or(0)
    }
    
    fn estimate_required_capital(&self, _asset: &Asset, _chain: &ChainId, _pool_state: &PoolState) -> f64 {
        // Simplified capital estimation
        10000.0 // $10K default
    }
    
    fn calculate_confidence_score(&self, buy_chain: &ChainId, sell_chain: &ChainId) -> f64 {
        // Calculate confidence based on chain reliability, liquidity, etc.
        let buy_reliability = self.get_chain_reliability(buy_chain);
        let sell_reliability = self.get_chain_reliability(sell_chain);
        
        (buy_reliability + sell_reliability) / 2.0
    }
    
    fn get_chain_reliability(&self, chain: &ChainId) -> f64 {
        match chain {
            ChainId::Bitcoin => 0.95,
            ChainId::Ethereum => 0.90,
            ChainId::Arbitrum => 0.85,
            ChainId::Optimism => 0.85,
            ChainId::Polygon => 0.80,
            ChainId::Base => 0.85,
            ChainId::Solana => 0.75,
            ChainId::Avalanche => 0.80,
        }
    }
}

// =============================================================================
// SUPPORTING TYPES
// =============================================================================

#[derive(Clone, Debug)]
pub struct ChainCapabilities {
    pub native_assets: Vec<Asset>,
    pub supported_operations: Vec<&'static str>,
    pub avg_confirmation_time: u64, // seconds
    pub finality_confidence: f64,   // 0.0 to 1.0
}