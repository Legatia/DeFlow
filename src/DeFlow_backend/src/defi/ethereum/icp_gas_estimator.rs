// ICP-Compliant Gas Estimator
// Uses EVM RPC canister for consensus-validated gas price data

use super::{EthereumError, EvmChain, GasPriority, GasEstimate, EvmRpcService, BlockTag};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

/// ICP-compliant gas estimator using EVM RPC canister
#[derive(Debug, Clone)]
pub struct IcpGasEstimator {
    evm_rpc: EvmRpcService,
    gas_price_cache: std::cell::RefCell<HashMap<EvmChain, CachedGasPrice>>,
}

#[derive(Debug, Clone)]
struct CachedGasPrice {
    gas_price: String,
    timestamp: u64,
    ttl_seconds: u64,
}

impl IcpGasEstimator {
    /// Create a new ICP gas estimator
    pub fn new() -> Self {
        Self {
            evm_rpc: EvmRpcService::new(),
            gas_price_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }

    /// Estimate gas for a transaction using ICP consensus
    pub async fn estimate_gas(
        &self,
        chain: &EvmChain,
        to_address: Option<&str>,
        data: Option<&str>,
        value: Option<&str>,
        priority: GasPriority,
    ) -> Result<GasEstimate, EthereumError> {
        // Get gas limit estimate
        let gas_limit = if let Some(to) = to_address {
            self.evm_rpc.eth_estimate_gas(
                chain,
                to,
                None, // from address
                value,
                data,
            ).await.unwrap_or(21000) // Default to 21000 for simple transfer
        } else {
            21000 // Simple transfer gas limit
        };

        // Get current gas pricing information
        let (gas_price, max_fee_per_gas, max_priority_fee_per_gas) = 
            self.get_gas_pricing(chain, priority).await?;

        // Calculate total fees
        let total_fee_wei = if chain.supports_eip1559() {
            let max_fee: u128 = max_fee_per_gas.parse()
                .map_err(|_| EthereumError::GasEstimationFailed("Invalid max fee format".to_string()))?;
            (gas_limit as u128 * max_fee).to_string()
        } else {
            let gas_price_wei: u128 = gas_price.parse()
                .map_err(|_| EthereumError::GasEstimationFailed("Invalid gas price format".to_string()))?;
            (gas_limit as u128 * gas_price_wei).to_string()
        };

        // Convert to ETH
        let total_fee_eth = super::utils::wei_to_eth(&total_fee_wei)?;

        // Estimate USD value (simplified - in production you'd use price oracles)
        let eth_price_usd = self.get_eth_price_estimate().await;
        let total_fee_usd = total_fee_eth * eth_price_usd;

        // Estimate confirmation time based on priority and chain
        let confirmation_time_estimate_seconds = self.estimate_confirmation_time(chain, priority);

        Ok(GasEstimate {
            gas_limit,
            gas_price,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            total_fee_wei,
            total_fee_eth,
            total_fee_usd,
            confirmation_time_estimate_seconds,
        })
    }

    /// Get gas pricing using ICP consensus from multiple providers
    async fn get_gas_pricing(
        &self,
        chain: &EvmChain,
        priority: GasPriority,
    ) -> Result<(String, String, String), EthereumError> {
        if chain.supports_eip1559() {
            self.get_eip1559_pricing(chain, priority).await
        } else {
            self.get_legacy_pricing(chain, priority).await
        }
    }

    /// Get EIP-1559 gas pricing
    async fn get_eip1559_pricing(
        &self,
        chain: &EvmChain,
        priority: GasPriority,
    ) -> Result<(String, String, String), EthereumError> {
        // Get fee history for EIP-1559 calculation
        let fee_history = self.evm_rpc.eth_fee_history(
            chain,
            20, // Last 20 blocks
            BlockTag::Latest,
            vec![10.0, 50.0, 90.0], // 10th, 50th, 90th percentiles
        ).await?;

        // Calculate base fee and priority fees based on historical data
        let latest_base_fee = fee_history.base_fee_per_gas
            .last()
            .ok_or_else(|| EthereumError::GasEstimationFailed("No base fee data".to_string()))?;

        let base_fee_wei: u128 = u128::from_str_radix(&latest_base_fee[2..], 16)
            .map_err(|_| EthereumError::GasEstimationFailed("Invalid base fee format".to_string()))?;

        // Calculate priority fee based on historical data and desired priority
        let priority_fee_wei = self.calculate_priority_fee(&fee_history, priority)?;

        // Calculate max fee per gas (base fee + priority fee + buffer)
        let buffer_multiplier = match priority {
            GasPriority::Low => 1.1,
            GasPriority::Medium => 1.2,
            GasPriority::High => 1.3,
            GasPriority::Urgent => 1.5,
        };

        let max_fee_per_gas_wei = ((base_fee_wei + priority_fee_wei) as f64 * buffer_multiplier) as u128;

        Ok((
            format!("0x{:x}", max_fee_per_gas_wei), // gas_price (for compatibility)
            format!("0x{:x}", max_fee_per_gas_wei), // max_fee_per_gas
            format!("0x{:x}", priority_fee_wei),    // max_priority_fee_per_gas
        ))
    }

    /// Get legacy gas pricing
    async fn get_legacy_pricing(
        &self,
        chain: &EvmChain,
        priority: GasPriority,
    ) -> Result<(String, String, String), EthereumError> {
        // Check cache first
        if let Some(cached) = self.get_cached_gas_price(chain) {
            let base_price: u128 = u128::from_str_radix(&cached.gas_price[2..], 16)
                .map_err(|_| EthereumError::GasEstimationFailed("Invalid cached gas price".to_string()))?;
            
            let adjusted_price = self.adjust_gas_price_for_priority(base_price, priority);
            let gas_price_hex = format!("0x{:x}", adjusted_price);
            
            return Ok((
                gas_price_hex.clone(),
                gas_price_hex.clone(),
                "0x0".to_string(), // No priority fee for legacy
            ));
        }

        // Get current gas price from EVM RPC canister
        let current_gas_price = self.evm_rpc.eth_gas_price(chain).await?;
        
        // Cache the result
        self.cache_gas_price(chain, current_gas_price.clone());

        let base_price: u128 = u128::from_str_radix(&current_gas_price[2..], 16)
            .map_err(|_| EthereumError::GasEstimationFailed("Invalid gas price format".to_string()))?;

        let adjusted_price = self.adjust_gas_price_for_priority(base_price, priority);
        let gas_price_hex = format!("0x{:x}", adjusted_price);

        Ok((
            gas_price_hex.clone(),
            gas_price_hex.clone(),
            "0x0".to_string(), // No priority fee for legacy
        ))
    }

    /// Calculate priority fee from fee history
    fn calculate_priority_fee(
        &self,
        fee_history: &super::FeeHistory,
        priority: GasPriority,
    ) -> Result<u128, EthereumError> {
        let rewards = fee_history.reward.as_ref()
            .ok_or_else(|| EthereumError::GasEstimationFailed("No reward data in fee history".to_string()))?;

        if rewards.is_empty() {
            return Ok(1_000_000_000); // 1 gwei default
        }

        // Use the appropriate percentile based on priority
        let percentile_index = match priority {
            GasPriority::Low => 0,    // 10th percentile
            GasPriority::Medium => 1, // 50th percentile
            GasPriority::High => 2,   // 90th percentile
            GasPriority::Urgent => 2, // 90th percentile + multiplier
        };

        let recent_rewards = &rewards[rewards.len().saturating_sub(5)..]; // Last 5 blocks
        let mut priority_fees: Vec<u128> = Vec::new();

        for block_rewards in recent_rewards {
            if let Some(reward_hex) = block_rewards.get(percentile_index) {
                if let Ok(priority_fee) = u128::from_str_radix(&reward_hex[2..], 16) {
                    priority_fees.push(priority_fee);
                }
            }
        }

        if priority_fees.is_empty() {
            return Ok(1_000_000_000); // 1 gwei default
        }

        // Calculate average of recent priority fees
        let avg_priority_fee = priority_fees.iter().sum::<u128>() / priority_fees.len() as u128;

        // Apply urgency multiplier
        let final_priority_fee = match priority {
            GasPriority::Urgent => (avg_priority_fee as f64 * 1.5) as u128,
            _ => avg_priority_fee,
        };

        Ok(final_priority_fee.max(1_000_000_000)) // Minimum 1 gwei
    }

    /// Adjust gas price based on priority for legacy transactions
    fn adjust_gas_price_for_priority(&self, base_price: u128, priority: GasPriority) -> u128 {
        let multiplier = match priority {
            GasPriority::Low => 0.9,
            GasPriority::Medium => 1.0,
            GasPriority::High => 1.2,
            GasPriority::Urgent => 1.5,
        };

        ((base_price as f64) * multiplier) as u128
    }

    /// Estimate confirmation time based on chain and priority
    fn estimate_confirmation_time(&self, chain: &EvmChain, priority: GasPriority) -> u64 {
        let base_time = match chain {
            EvmChain::Ethereum => 60,    // ~1 minute per block
            EvmChain::Arbitrum => 15,    // ~15 seconds
            EvmChain::Optimism => 12,    // ~12 seconds
            EvmChain::Polygon => 3,      // ~3 seconds
            EvmChain::Base => 12,        // ~12 seconds
            EvmChain::Avalanche => 3,    // ~3 seconds
        };

        let blocks_to_confirm = match priority {
            GasPriority::Low => 3,
            GasPriority::Medium => 2,
            GasPriority::High => 1,
            GasPriority::Urgent => 1,
        };

        base_time * blocks_to_confirm
    }

    /// Get ETH price estimate (simplified - in production use price oracles)
    async fn get_eth_price_estimate(&self) -> f64 {
        // In production, this would call a price oracle
        // For now, return a reasonable estimate
        2000.0 // $2000 USD per ETH
    }

    /// Get cached gas price if still valid
    fn get_cached_gas_price(&self, chain: &EvmChain) -> Option<CachedGasPrice> {
        let cache = self.gas_price_cache.borrow();
        if let Some(cached) = cache.get(chain) {
            let now = ic_cdk::api::time();
            if now - cached.timestamp < cached.ttl_seconds * 1_000_000_000 {
                return Some(cached.clone());
            }
        }
        None
    }

    /// Cache gas price for future use
    fn cache_gas_price(&self, chain: &EvmChain, gas_price: String) {
        let cached = CachedGasPrice {
            gas_price,
            timestamp: ic_cdk::api::time(),
            ttl_seconds: 30, // Cache for 30 seconds
        };

        self.gas_price_cache.borrow_mut().insert(chain.clone(), cached);
    }

    /// Get detailed gas analysis for multiple chains
    pub async fn compare_gas_across_chains(
        &self,
        chains: &[EvmChain],
        priority: GasPriority,
    ) -> Result<Vec<ChainGasComparison>, EthereumError> {
        let mut comparisons = Vec::new();

        for chain in chains {
            match self.estimate_gas(chain, None, None, None, priority.clone()).await {
                Ok(estimate) => {
                    comparisons.push(ChainGasComparison {
                        chain: chain.clone(),
                        gas_estimate: estimate,
                        relative_cost_factor: 1.0, // Will be calculated after all estimates
                    });
                },
                Err(e) => {
                    // Continue with other chains
                }
            }
        }

        // Calculate relative cost factors
        if let Some(min_cost) = comparisons.iter().map(|c| c.gas_estimate.total_fee_usd).fold(None, |acc, x| {
            Some(match acc {
                None => x,
                Some(y) => x.min(y),
            })
        }) {
            for comparison in &mut comparisons {
                comparison.relative_cost_factor = comparison.gas_estimate.total_fee_usd / min_cost;
            }
        }

        // Sort by total cost
        comparisons.sort_by(|a, b| a.gas_estimate.total_fee_usd.partial_cmp(&b.gas_estimate.total_fee_usd).unwrap());

        Ok(comparisons)
    }
}

impl Default for IcpGasEstimator {
    fn default() -> Self {
        Self::new()
    }
}

/// Gas comparison across chains
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ChainGasComparison {
    pub chain: EvmChain,
    pub gas_estimate: GasEstimate,
    pub relative_cost_factor: f64, // How much more expensive than the cheapest option
}

/// Extension trait for EvmChain to check EIP-1559 support
trait Eip1559Support {
    fn supports_eip1559(&self) -> bool;
}

impl Eip1559Support for EvmChain {
    fn supports_eip1559(&self) -> bool {
        match self {
            EvmChain::Ethereum => true,
            EvmChain::Arbitrum => true,
            EvmChain::Optimism => true,
            EvmChain::Polygon => true,
            EvmChain::Base => true,
            EvmChain::Avalanche => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_estimator_creation() {
        let estimator = IcpGasEstimator::new();
        assert_eq!(estimator.evm_rpc.canister_id.to_text(), super::EVM_RPC_CANISTER_ID);
    }

    #[test]
    fn test_priority_adjustment() {
        let estimator = IcpGasEstimator::new();
        let base_price = 20_000_000_000u128; // 20 gwei
        
        assert_eq!(estimator.adjust_gas_price_for_priority(base_price, GasPriority::Low), 18_000_000_000);
        assert_eq!(estimator.adjust_gas_price_for_priority(base_price, GasPriority::Medium), 20_000_000_000);
        assert_eq!(estimator.adjust_gas_price_for_priority(base_price, GasPriority::High), 24_000_000_000);
        assert_eq!(estimator.adjust_gas_price_for_priority(base_price, GasPriority::Urgent), 30_000_000_000);
    }

    #[test]
    fn test_confirmation_time_estimation() {
        let estimator = IcpGasEstimator::new();
        
        // Ethereum should take longer than L2s
        let eth_time = estimator.estimate_confirmation_time(&EvmChain::Ethereum, GasPriority::Medium);
        let arb_time = estimator.estimate_confirmation_time(&EvmChain::Arbitrum, GasPriority::Medium);
        
        assert!(eth_time > arb_time);
        
        // Urgent should be faster than low priority
        let urgent_time = estimator.estimate_confirmation_time(&EvmChain::Ethereum, GasPriority::Urgent);
        let low_time = estimator.estimate_confirmation_time(&EvmChain::Ethereum, GasPriority::Low);
        
        assert!(urgent_time <= low_time);
    }

    #[test]
    fn test_eip1559_support() {
        assert!(EvmChain::Ethereum.supports_eip1559());
        assert!(EvmChain::Arbitrum.supports_eip1559());
        assert!(EvmChain::Polygon.supports_eip1559());
    }
}