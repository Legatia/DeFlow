// L2 Optimization Framework
// Cross-L2 gas comparison, optimal chain selection, and bridge cost analysis

use super::{EvmChain, L2OptimizationResult, ChainOption, EthereumError, GasPriority, GasEstimator, ChainConfigManager};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// L2 optimization service for finding the most cost-effective chain
#[derive(Debug, Clone)]
pub struct L2Optimizer {
    gas_estimator: GasEstimator,
    chain_config: ChainConfigManager,
    bridge_cost_cache: std::cell::RefCell<HashMap<(EvmChain, EvmChain), CachedBridgeCost>>,
}

/// Cached bridge cost data
#[derive(Debug, Clone)]
struct CachedBridgeCost {
    cost_usd: f64,
    time_minutes: u64,
    timestamp: u64,
    ttl_seconds: u64,
}

/// Bridge route information
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BridgeRoute {
    pub from_chain: EvmChain,
    pub to_chain: EvmChain,
    pub bridge_type: BridgeType,
    pub estimated_cost_usd: f64,
    pub estimated_time_minutes: u64,
    pub requires_intermediate_chain: bool,
    pub intermediate_chain: Option<EvmChain>,
    pub total_hops: u32,
}

/// Types of bridges available
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum BridgeType {
    /// Official bridge (e.g., Arbitrum Bridge, Optimism Gateway)
    Official,
    /// Third-party bridge (e.g., Hop Protocol, Across)
    ThirdParty,
    /// Multi-hop via Ethereum
    MultiHop,
}

/// Transaction context for optimization
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TransactionContext {
    pub transaction_type: TransactionType,
    pub amount_usd: f64,
    pub urgency: GasPriority,
    pub current_chain: Option<EvmChain>,
    pub preferred_chains: Vec<EvmChain>,
    pub max_bridge_cost_usd: Option<f64>,
    pub max_total_time_minutes: Option<u64>,
}

/// Types of transactions for optimization
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum TransactionType {
    /// Simple ETH/native token transfer
    SimpleTransfer,
    /// ERC-20 token transfer
    TokenTransfer,
    /// DEX swap
    DexSwap,
    /// Lending protocol interaction
    Lending,
    /// NFT transaction
    Nft,
    /// Contract deployment
    ContractDeployment,
    /// Complex DeFi interaction
    ComplexDefi,
}

impl L2Optimizer {
    /// Create a new L2 optimizer
    pub fn new() -> Self {
        Self {
            gas_estimator: GasEstimator::new(),
            chain_config: ChainConfigManager::new(),
            bridge_cost_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }
    
    /// Find the optimal chain for a transaction
    pub async fn optimize_transaction(
        &self,
        context: &TransactionContext,
    ) -> Result<L2OptimizationResult, EthereumError> {
        // Get all viable chains based on preferences
        let candidate_chains = self.get_candidate_chains(context);
        
        // Calculate costs for each chain
        let mut chain_options = Vec::new();
        
        for chain in candidate_chains {
            if let Ok(option) = self.calculate_chain_option(&chain, context).await {
                chain_options.push(option);
            }
        }
        
        if chain_options.is_empty() {
            return Err(EthereumError::ChainNotSupported(
                "No viable chains found for transaction".to_string()
            ));
        }
        
        // Sort by total cost (including bridge costs)
        chain_options.sort_by(|a, b| a.total_cost_usd.partial_cmp(&b.total_cost_usd).unwrap());
        
        let recommended_option = chain_options[0].clone();
        let alternatives = chain_options[1..].to_vec();
        
        // Calculate savings vs Ethereum
        let ethereum_cost = self.calculate_ethereum_baseline_cost(context).await?;
        let savings_vs_ethereum = ethereum_cost - recommended_option.total_cost_usd;
        
        Ok(L2OptimizationResult {
            recommended_chain: recommended_option.chain,
            estimated_fee_usd: recommended_option.fee_usd,
            estimated_time_seconds: recommended_option.time_seconds,
            savings_vs_ethereum,
            alternatives,
            bridge_cost_usd: recommended_option.bridge_cost_usd,
            total_cost_usd: recommended_option.total_cost_usd,
        })
    }
    
    /// Get candidate chains based on transaction context
    fn get_candidate_chains(&self, context: &TransactionContext) -> Vec<EvmChain> {
        let mut candidates = if context.preferred_chains.is_empty() {
            // Use all supported chains if no preferences
            self.chain_config.get_supported_chains()
        } else {
            context.preferred_chains.clone()
        };
        
        // Filter by transaction type compatibility
        candidates.retain(|chain| self.is_chain_compatible_with_transaction(chain, &context.transaction_type));
        
        // Always include Ethereum as a baseline
        if !candidates.contains(&EvmChain::Ethereum) {
            candidates.push(EvmChain::Ethereum);
        }
        
        candidates
    }
    
    /// Check if a chain is compatible with a transaction type
    fn is_chain_compatible_with_transaction(&self, chain: &EvmChain, tx_type: &TransactionType) -> bool {
        match tx_type {
            TransactionType::SimpleTransfer => true, // All chains support transfers
            TransactionType::TokenTransfer => true, // All chains support ERC-20
            TransactionType::DexSwap => {
                // Most chains have DEXs, but check for major ones
                matches!(chain, 
                    EvmChain::Ethereum | EvmChain::Arbitrum | EvmChain::Optimism | 
                    EvmChain::Polygon | EvmChain::Base | EvmChain::Avalanche
                )
            },
            TransactionType::Lending => {
                // Major lending protocols available on these chains
                matches!(chain, 
                    EvmChain::Ethereum | EvmChain::Arbitrum | EvmChain::Optimism | 
                    EvmChain::Polygon | EvmChain::Avalanche
                )
            },
            TransactionType::Nft => {
                // NFT support on major chains
                matches!(chain, 
                    EvmChain::Ethereum | EvmChain::Arbitrum | EvmChain::Optimism | 
                    EvmChain::Polygon | EvmChain::Base
                )
            },
            TransactionType::ContractDeployment => true, // All chains support deployment
            TransactionType::ComplexDefi => {
                // Complex DeFi usually on major chains
                matches!(chain, 
                    EvmChain::Ethereum | EvmChain::Arbitrum | EvmChain::Optimism | 
                    EvmChain::Polygon
                )
            },
        }
    }
    
    /// Calculate option for a specific chain
    async fn calculate_chain_option(
        &self,
        chain: &EvmChain,
        context: &TransactionContext,
    ) -> Result<ChainOption, EthereumError> {
        // Get gas estimate for the transaction
        let gas_estimate = self.gas_estimator.estimate_gas(
            chain,
            None, // to address
            None, // data
            Some(&context.amount_usd.to_string()), // value (simplified)
            context.urgency.clone(),
        ).await?;
        
        // Calculate bridge cost if needed
        let bridge_cost_usd = if let Some(current_chain) = &context.current_chain {
            if current_chain != chain {
                Some(self.calculate_bridge_cost(current_chain, chain).await?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Apply transaction type multiplier to gas estimate
        let adjusted_fee_usd = gas_estimate.total_fee_usd * self.get_transaction_type_multiplier(&context.transaction_type);
        
        // Calculate total cost
        let total_cost_usd = adjusted_fee_usd + bridge_cost_usd.unwrap_or(0.0);
        
        // Calculate confidence score based on data freshness and chain reliability
        let confidence_score = self.calculate_confidence_score(chain, &gas_estimate);
        
        Ok(ChainOption {
            chain: chain.clone(),
            fee_usd: adjusted_fee_usd,
            time_seconds: gas_estimate.confirmation_time_estimate_seconds,
            bridge_cost_usd,
            total_cost_usd,
            confidence_score,
        })
    }
    
    /// Get multiplier for different transaction types (complexity factor)
    fn get_transaction_type_multiplier(&self, tx_type: &TransactionType) -> f64 {
        match tx_type {
            TransactionType::SimpleTransfer => 1.0,
            TransactionType::TokenTransfer => 2.5,
            TransactionType::DexSwap => 4.0,
            TransactionType::Lending => 3.5,
            TransactionType::Nft => 2.0,
            TransactionType::ContractDeployment => 10.0,
            TransactionType::ComplexDefi => 6.0,
        }
    }
    
    /// Calculate bridge cost between two chains
    async fn calculate_bridge_cost(&self, from_chain: &EvmChain, to_chain: &EvmChain) -> Result<f64, EthereumError> {
        // Check cache first
        if let Some(cached) = self.get_cached_bridge_cost(from_chain, to_chain) {
            return Ok(cached.cost_usd);
        }
        
        let cost = self.estimate_bridge_cost(from_chain, to_chain).await?;
        
        // Cache the result
        self.cache_bridge_cost(from_chain, to_chain, cost);
        
        Ok(cost)
    }
    
    /// Estimate bridge cost between chains
    async fn estimate_bridge_cost(&self, from_chain: &EvmChain, to_chain: &EvmChain) -> Result<f64, EthereumError> {
        if from_chain == to_chain {
            return Ok(0.0);
        }
        
        // Get bridge route
        let route = self.get_bridge_route(from_chain, to_chain)?;
        
        match route.bridge_type {
            BridgeType::Official => {
                // Official bridges typically cost gas on both chains
                let from_gas = self.gas_estimator.estimate_gas(
                    from_chain, None, None, None, GasPriority::Medium
                ).await?.total_fee_usd;
                
                let to_gas = self.gas_estimator.estimate_gas(
                    to_chain, None, None, None, GasPriority::Medium
                ).await?.total_fee_usd;
                
                Ok(from_gas * 2.0 + to_gas) // 2x for bridge transaction complexity
            },
            BridgeType::ThirdParty => {
                // Third-party bridges usually have fees + gas
                let base_gas = self.gas_estimator.estimate_gas(
                    from_chain, None, None, None, GasPriority::Medium
                ).await?.total_fee_usd;
                
                let bridge_fee = match (from_chain, to_chain) {
                    // L2 to L2 bridges
                    (EvmChain::Arbitrum, EvmChain::Optimism) => 3.0,
                    (EvmChain::Optimism, EvmChain::Arbitrum) => 3.0,
                    (EvmChain::Arbitrum, EvmChain::Base) => 2.5,
                    (EvmChain::Base, EvmChain::Arbitrum) => 2.5,
                    _ => 5.0, // Default third-party bridge fee
                };
                
                Ok(base_gas + bridge_fee)
            },
            BridgeType::MultiHop => {
                // Multi-hop via Ethereum
                let ethereum_gas = self.gas_estimator.estimate_gas(
                    &EvmChain::Ethereum, None, None, None, GasPriority::Medium
                ).await?.total_fee_usd;
                
                let from_gas = self.gas_estimator.estimate_gas(
                    from_chain, None, None, None, GasPriority::Medium
                ).await?.total_fee_usd;
                
                let to_gas = self.gas_estimator.estimate_gas(
                    to_chain, None, None, None, GasPriority::Medium
                ).await?.total_fee_usd;
                
                Ok(from_gas * 2.0 + ethereum_gas * 2.0 + to_gas * 2.0) // Multiple transactions
            },
        }
    }
    
    /// Get bridge route between two chains
    fn get_bridge_route(&self, from_chain: &EvmChain, to_chain: &EvmChain) -> Result<BridgeRoute, EthereumError> {
        if from_chain == to_chain {
            return Err(EthereumError::InvalidAddress("Same chain bridge".to_string()));
        }
        
        // Check if direct bridging is available
        if self.chain_config.are_chains_bridgeable(from_chain, to_chain) {
            let bridge_type = if self.has_official_bridge(from_chain, to_chain) {
                BridgeType::Official
            } else {
                BridgeType::ThirdParty
            };
            
            return Ok(BridgeRoute {
                from_chain: from_chain.clone(),
                to_chain: to_chain.clone(),
                bridge_type,
                estimated_cost_usd: 0.0, // Will be calculated separately
                estimated_time_minutes: self.estimate_bridge_time(from_chain, to_chain),
                requires_intermediate_chain: false,
                intermediate_chain: None,
                total_hops: 1,
            });
        }
        
        // Multi-hop via Ethereum
        Ok(BridgeRoute {
            from_chain: from_chain.clone(),
            to_chain: to_chain.clone(),
            bridge_type: BridgeType::MultiHop,
            estimated_cost_usd: 0.0,
            estimated_time_minutes: self.estimate_bridge_time(from_chain, &EvmChain::Ethereum) +
                                   self.estimate_bridge_time(&EvmChain::Ethereum, to_chain),
            requires_intermediate_chain: true,
            intermediate_chain: Some(EvmChain::Ethereum),
            total_hops: 2,
        })
    }
    
    /// Check if chains have official bridge
    fn has_official_bridge(&self, from_chain: &EvmChain, to_chain: &EvmChain) -> bool {
        match (from_chain, to_chain) {
            // L2s have official bridges to Ethereum
            (EvmChain::Arbitrum, EvmChain::Ethereum) => true,
            (EvmChain::Ethereum, EvmChain::Arbitrum) => true,
            (EvmChain::Optimism, EvmChain::Ethereum) => true,
            (EvmChain::Ethereum, EvmChain::Optimism) => true,
            (EvmChain::Base, EvmChain::Ethereum) => true,
            (EvmChain::Ethereum, EvmChain::Base) => true,
            // Polygon has POS bridge
            (EvmChain::Polygon, EvmChain::Ethereum) => true,
            (EvmChain::Ethereum, EvmChain::Polygon) => true,
            // Avalanche has official bridge
            (EvmChain::Avalanche, EvmChain::Ethereum) => true,
            (EvmChain::Ethereum, EvmChain::Avalanche) => true,
            _ => false,
        }
    }
    
    /// Estimate bridge time between chains
    fn estimate_bridge_time(&self, from_chain: &EvmChain, to_chain: &EvmChain) -> u64 {
        match (from_chain, to_chain) {
            // L2 to Ethereum (withdrawal periods)
            (EvmChain::Arbitrum, EvmChain::Ethereum) => 7 * 24 * 60, // 7 days
            (EvmChain::Optimism, EvmChain::Ethereum) => 7 * 24 * 60, // 7 days
            (EvmChain::Base, EvmChain::Ethereum) => 7 * 24 * 60, // 7 days
            
            // Ethereum to L2 (deposits)
            (EvmChain::Ethereum, EvmChain::Arbitrum) => 15, // ~15 minutes
            (EvmChain::Ethereum, EvmChain::Optimism) => 15, // ~15 minutes
            (EvmChain::Ethereum, EvmChain::Base) => 15, // ~15 minutes
            
            // Polygon (faster)
            (EvmChain::Polygon, EvmChain::Ethereum) => 3 * 60, // ~3 hours
            (EvmChain::Ethereum, EvmChain::Polygon) => 30, // ~30 minutes
            
            // Avalanche
            (EvmChain::Avalanche, EvmChain::Ethereum) => 60, // ~1 hour
            (EvmChain::Ethereum, EvmChain::Avalanche) => 30, // ~30 minutes
            
            // L2 to L2 (via third-party bridges)
            _ => 60, // ~1 hour default
        }
    }
    
    /// Calculate confidence score for optimization result
    fn calculate_confidence_score(&self, chain: &EvmChain, gas_estimate: &super::GasEstimate) -> f64 {
        let mut score: f64 = 1.0;
        
        // Reduce confidence for very high or very low gas prices (potential outliers)
        let gas_price_gwei = gas_estimate.gas_price.parse::<u64>().unwrap_or(0) / 1_000_000_000;
        if gas_price_gwei > 100 || gas_price_gwei < 1 {
            score *= 0.8;
        }
        
        // Higher confidence for established chains
        match chain {
            EvmChain::Ethereum | EvmChain::Arbitrum | EvmChain::Optimism => score *= 1.0,
            EvmChain::Polygon | EvmChain::Avalanche => score *= 0.95,
            EvmChain::Base => score *= 0.9,
        }
        
        // Reduce confidence if estimation seems too good to be true
        if gas_estimate.total_fee_usd < 0.01 {
            score *= 0.7;
        }
        
        score.min(1.0).max(0.1)
    }
    
    /// Calculate Ethereum baseline cost for comparison
    async fn calculate_ethereum_baseline_cost(&self, context: &TransactionContext) -> Result<f64, EthereumError> {
        let gas_estimate = self.gas_estimator.estimate_gas(
            &EvmChain::Ethereum,
            None,
            None,
            Some(&context.amount_usd.to_string()),
            context.urgency.clone(),
        ).await?;
        
        Ok(gas_estimate.total_fee_usd * self.get_transaction_type_multiplier(&context.transaction_type))
    }
    
    /// Get cached bridge cost
    fn get_cached_bridge_cost(&self, from_chain: &EvmChain, to_chain: &EvmChain) -> Option<CachedBridgeCost> {
        let cache = self.bridge_cost_cache.borrow();
        let key = (from_chain.clone(), to_chain.clone());
        
        if let Some(cached) = cache.get(&key) {
            let now = ic_cdk::api::time();
            if now - cached.timestamp < cached.ttl_seconds * 1_000_000_000 {
                return Some(cached.clone());
            }
        }
        None
    }
    
    /// Cache bridge cost
    fn cache_bridge_cost(&self, from_chain: &EvmChain, to_chain: &EvmChain, cost_usd: f64) {
        let cached = CachedBridgeCost {
            cost_usd,
            time_minutes: self.estimate_bridge_time(from_chain, to_chain),
            timestamp: ic_cdk::api::time(),
            ttl_seconds: 300, // Cache for 5 minutes
        };
        
        let key = (from_chain.clone(), to_chain.clone());
        self.bridge_cost_cache.borrow_mut().insert(key, cached);
    }
    
    /// Compare multiple L2 options
    pub async fn compare_l2_options(
        &self,
        l2_chains: &[EvmChain],
        context: &TransactionContext,
    ) -> Result<Vec<ChainOption>, EthereumError> {
        let mut options = Vec::new();
        
        for chain in l2_chains {
            if chain.is_l2() || chain.is_sidechain() {
                if let Ok(option) = self.calculate_chain_option(chain, context).await {
                    options.push(option);
                }
            }
        }
        
        // Sort by total cost
        options.sort_by(|a, b| a.total_cost_usd.partial_cmp(&b.total_cost_usd).unwrap());
        
        Ok(options)
    }
    
    /// Get bridge options between two chains
    pub async fn get_bridge_options(
        &self,
        from_chain: &EvmChain,
        to_chain: &EvmChain,
    ) -> Result<Vec<BridgeRoute>, EthereumError> {
        let mut routes = Vec::new();
        
        // Direct route if available
        if let Ok(direct_route) = self.get_bridge_route(from_chain, to_chain) {
            routes.push(direct_route);
        }
        
        // Alternative routes via Ethereum (if not already included)
        if from_chain != &EvmChain::Ethereum && to_chain != &EvmChain::Ethereum {
            let multi_hop_route = BridgeRoute {
                from_chain: from_chain.clone(),
                to_chain: to_chain.clone(),
                bridge_type: BridgeType::MultiHop,
                estimated_cost_usd: self.calculate_bridge_cost(from_chain, to_chain).await.unwrap_or(0.0),
                estimated_time_minutes: self.estimate_bridge_time(from_chain, to_chain),
                requires_intermediate_chain: true,
                intermediate_chain: Some(EvmChain::Ethereum),
                total_hops: 2,
            };
            routes.push(multi_hop_route);
        }
        
        Ok(routes)
    }
}

impl Default for L2Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// L2 optimization strategies
pub mod strategies {
    use super::*;
    
    /// Strategy for cost optimization
    pub struct CostOptimizationStrategy;
    
    impl CostOptimizationStrategy {
        /// Find the cheapest option regardless of time
        pub async fn optimize(
            optimizer: &L2Optimizer,
            context: &TransactionContext,
        ) -> Result<L2OptimizationResult, EthereumError> {
            let mut modified_context = context.clone();
            modified_context.urgency = GasPriority::Low; // Use lowest gas prices
            
            optimizer.optimize_transaction(&modified_context).await
        }
    }
    
    /// Strategy for speed optimization
    pub struct SpeedOptimizationStrategy;
    
    impl SpeedOptimizationStrategy {
        /// Find the fastest option
        pub async fn optimize(
            optimizer: &L2Optimizer,
            context: &TransactionContext,
        ) -> Result<L2OptimizationResult, EthereumError> {
            let mut modified_context = context.clone();
            modified_context.urgency = GasPriority::Urgent;
            // Prefer chains without bridge requirements
            modified_context.current_chain = None;
            
            optimizer.optimize_transaction(&modified_context).await
        }
    }
    
    /// Strategy for balanced optimization
    pub struct BalancedOptimizationStrategy;
    
    impl BalancedOptimizationStrategy {
        /// Find the best balance of cost and speed
        pub async fn optimize(
            optimizer: &L2Optimizer,
            context: &TransactionContext,
        ) -> Result<L2OptimizationResult, EthereumError> {
            let result = optimizer.optimize_transaction(context).await?;
            
            // Apply balanced scoring
            let mut best_option = result.alternatives[0].clone();
            let mut best_score = f64::MAX;
            
            for option in &result.alternatives {
                // Score = normalized_cost + normalized_time
                let cost_score = option.total_cost_usd / 100.0; // Normalize to reasonable range
                let time_score = option.time_seconds as f64 / 3600.0; // Normalize to hours
                let combined_score = cost_score + time_score;
                
                if combined_score < best_score {
                    best_score = combined_score;
                    best_option = option.clone();
                }
            }
            
            Ok(L2OptimizationResult {
                recommended_chain: best_option.chain,
                estimated_fee_usd: best_option.fee_usd,
                estimated_time_seconds: best_option.time_seconds,
                savings_vs_ethereum: result.savings_vs_ethereum,
                alternatives: result.alternatives,
                bridge_cost_usd: best_option.bridge_cost_usd,
                total_cost_usd: best_option.total_cost_usd,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transaction_type_multiplier() {
        let optimizer = L2Optimizer::new();
        
        assert_eq!(optimizer.get_transaction_type_multiplier(&TransactionType::SimpleTransfer), 1.0);
        assert!(optimizer.get_transaction_type_multiplier(&TransactionType::DexSwap) > 3.0);
        assert!(optimizer.get_transaction_type_multiplier(&TransactionType::ContractDeployment) > 5.0);
    }
    
    #[test]
    fn test_bridge_time_estimation() {
        let optimizer = L2Optimizer::new();
        
        // L2 withdrawals should take days
        let arb_to_eth_time = optimizer.estimate_bridge_time(&EvmChain::Arbitrum, &EvmChain::Ethereum);
        assert!(arb_to_eth_time > 1000); // More than 1000 minutes (>16 hours)
        
        // Deposits should be faster
        let eth_to_arb_time = optimizer.estimate_bridge_time(&EvmChain::Ethereum, &EvmChain::Arbitrum);
        assert!(eth_to_arb_time < 60); // Less than 1 hour
    }
    
    #[test]
    fn test_chain_compatibility() {
        let optimizer = L2Optimizer::new();
        
        // All chains should support simple transfers
        assert!(optimizer.is_chain_compatible_with_transaction(&EvmChain::Ethereum, &TransactionType::SimpleTransfer));
        assert!(optimizer.is_chain_compatible_with_transaction(&EvmChain::Arbitrum, &TransactionType::SimpleTransfer));
        
        // Complex DeFi should be limited to major chains
        assert!(optimizer.is_chain_compatible_with_transaction(&EvmChain::Ethereum, &TransactionType::ComplexDefi));
        assert!(optimizer.is_chain_compatible_with_transaction(&EvmChain::Arbitrum, &TransactionType::ComplexDefi));
    }
}