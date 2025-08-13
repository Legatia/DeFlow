// Gas Estimation and Fee Management System
// Provides gas estimation for Ethereum and L2 chains with EIP-1559 support

use super::{EvmChain, GasEstimate, GasPriority, EthereumError, ChainConfigManager};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use ic_cdk::api::management_canister::http_request::{HttpRequest, HttpResponse, HttpMethod, TransformArgs};
use std::collections::HashMap;

/// Gas estimation service for multi-chain EVM operations
#[derive(Debug, Clone)]
pub struct GasEstimator {
    chain_config: ChainConfigManager,
    price_cache: std::cell::RefCell<HashMap<EvmChain, CachedGasPrice>>,
}

/// Cached gas price data
#[derive(Debug, Clone)]
struct CachedGasPrice {
    timestamp: u64,
    base_fee: u64,
    priority_fee: u64,
    gas_price: u64,
    ttl_seconds: u64,
}

/// Gas price data from RPC
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
struct GasPriceData {
    base_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
    gas_price: Option<String>,
}

/// EIP-1559 fee recommendation
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct EIP1559FeeRecommendation {
    pub base_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub max_fee_per_gas: String,
    pub estimated_confirmation_time_seconds: u64,
}

impl GasEstimator {
    /// Create a new gas estimator
    pub fn new() -> Self {
        Self {
            chain_config: ChainConfigManager::new(),
            price_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }
    
    /// Estimate gas for a transaction
    pub async fn estimate_gas(
        &self,
        chain: &EvmChain,
        to: Option<&str>,
        data: Option<&str>,
        value: Option<&str>,
        priority: GasPriority,
    ) -> Result<GasEstimate, EthereumError> {
        // Get current gas prices
        let gas_prices = self.get_gas_prices(chain).await?;
        
        // Estimate gas limit
        let gas_limit = self.estimate_gas_limit(chain, to, data, value).await?;
        
        // Calculate fees based on priority and chain support for EIP-1559
        let (gas_price, max_fee_per_gas, max_priority_fee_per_gas) = 
            self.calculate_gas_fees(chain, &gas_prices, priority)?;
        
        // Calculate total fees
        let total_fee_wei = if self.chain_config.supports_eip1559(chain) {
            (gas_limit as u128 * max_fee_per_gas.parse::<u128>()
                .map_err(|_| EthereumError::GasEstimationFailed("Invalid max_fee_per_gas".to_string()))?)
                .to_string()
        } else {
            (gas_limit as u128 * gas_price.parse::<u128>()
                .map_err(|_| EthereumError::GasEstimationFailed("Invalid gas_price".to_string()))?)
                .to_string()
        };
        
        let total_fee_eth = super::utils::wei_to_eth(&total_fee_wei)?;
        let total_fee_usd = self.convert_eth_to_usd(total_fee_eth, chain).await?;
        
        // Estimate confirmation time
        let confirmation_time = self.estimate_confirmation_time(chain, priority)?;
        
        Ok(GasEstimate {
            gas_limit,
            gas_price,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            total_fee_wei,
            total_fee_eth,
            total_fee_usd,
            confirmation_time_estimate_seconds: confirmation_time,
            priority,
        })
    }
    
    /// Get current gas prices for a chain
    async fn get_gas_prices(&self, chain: &EvmChain) -> Result<GasPriceData, EthereumError> {
        // Check cache first
        if let Some(cached) = self.get_cached_gas_price(chain) {
            return Ok(GasPriceData {
                base_fee_per_gas: Some(cached.base_fee.to_string()),
                max_priority_fee_per_gas: Some(cached.priority_fee.to_string()),
                gas_price: Some(cached.gas_price.to_string()),
            });
        }
        
        // Fetch from RPC
        let gas_prices = self.fetch_gas_prices_from_rpc(chain).await?;
        
        // Cache the result
        self.cache_gas_prices(chain, &gas_prices);
        
        Ok(gas_prices)
    }
    
    /// Fetch gas prices from RPC endpoint
    async fn fetch_gas_prices_from_rpc(&self, chain: &EvmChain) -> Result<GasPriceData, EthereumError> {
        let rpc_url = self.chain_config.get_primary_rpc(chain)
            .ok_or_else(|| EthereumError::ChainNotSupported(chain.name().to_string()))?;
        
        if self.chain_config.supports_eip1559(chain) {
            // Fetch EIP-1559 gas prices
            self.fetch_eip1559_gas_prices(&rpc_url).await
        } else {
            // Fetch legacy gas price
            self.fetch_legacy_gas_price(&rpc_url).await
        }
    }
    
    /// Fetch EIP-1559 gas prices
    async fn fetch_eip1559_gas_prices(&self, rpc_url: &str) -> Result<GasPriceData, EthereumError> {
        // Prepare RPC request for eth_feeHistory
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_feeHistory",
            "params": [4, "latest", [25, 50, 75]], // 4 blocks, 25th/50th/75th percentiles
            "id": 1
        });
        
        let request = HttpRequest {
            url: rpc_url.to_string(),
            method: HttpMethod::POST,
            body: Some(request_body.to_string().into_bytes()),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            transform: Some(ic_cdk::api::management_canister::http_request::TransformContext::from_name(
                "transform_gas_price_response".to_string(), 
                serde_json::to_vec(&()).unwrap()
            )),
        };
        
        let (response,): (HttpResponse,) = ic_cdk::api::management_canister::http_request::http_request(request)
            .await
            .map_err(|e| EthereumError::NetworkError(format!("HTTP request failed: {:?}", e)))?;
        
        if response.status != 200 {
            return Err(EthereumError::NetworkError(format!("HTTP status: {}", response.status)));
        }
        
        let response_text = String::from_utf8(response.body)
            .map_err(|e| EthereumError::NetworkError(format!("Invalid UTF-8: {}", e)))?;
        
        let rpc_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| EthereumError::NetworkError(format!("JSON parse error: {}", e)))?;
        
        // Parse fee history response
        let result = rpc_response.get("result")
            .ok_or_else(|| EthereumError::NetworkError("No result in response".to_string()))?;
        
        let base_fee_per_gas = result.get("baseFeePerGas")
            .and_then(|fees| fees.as_array()?.last())
            .and_then(|fee| fee.as_str())
            .map(|fee| u64::from_str_radix(&fee[2..], 16).unwrap_or(0))
            .unwrap_or(20_000_000_000); // 20 gwei default
        
        let priority_fees = result.get("reward")
            .and_then(|rewards| rewards.as_array()?.last())
            .and_then(|reward| reward.as_array())
            .map(|percentiles| {
                percentiles.get(1) // 50th percentile
                    .and_then(|fee| fee.as_str())
                    .map(|fee| u64::from_str_radix(&fee[2..], 16).unwrap_or(0))
                    .unwrap_or(2_000_000_000) // 2 gwei default
            })
            .unwrap_or(2_000_000_000);
        
        Ok(GasPriceData {
            base_fee_per_gas: Some(base_fee_per_gas.to_string()),
            max_priority_fee_per_gas: Some(priority_fees.to_string()),
            gas_price: Some((base_fee_per_gas + priority_fees).to_string()),
        })
    }
    
    /// Fetch legacy gas price
    async fn fetch_legacy_gas_price(&self, rpc_url: &str) -> Result<GasPriceData, EthereumError> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });
        
        let request = HttpRequest {
            url: rpc_url.to_string(),
            method: HttpMethod::POST,
            body: Some(request_body.to_string().into_bytes()),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            transform: Some(ic_cdk::api::management_canister::http_request::TransformContext::from_name(
                "transform_gas_price_response".to_string(), 
                serde_json::to_vec(&()).unwrap()
            )),
        };
        
        let (response,): (HttpResponse,) = ic_cdk::api::management_canister::http_request::http_request(request)
            .await
            .map_err(|e| EthereumError::NetworkError(format!("HTTP request failed: {:?}", e)))?;
        
        if response.status != 200 {
            return Err(EthereumError::NetworkError(format!("HTTP status: {}", response.status)));
        }
        
        let response_text = String::from_utf8(response.body)
            .map_err(|e| EthereumError::NetworkError(format!("Invalid UTF-8: {}", e)))?;
        
        let rpc_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| EthereumError::NetworkError(format!("JSON parse error: {}", e)))?;
        
        let gas_price_hex = rpc_response.get("result")
            .and_then(|result| result.as_str())
            .ok_or_else(|| EthereumError::NetworkError("No gas price in response".to_string()))?;
        
        let gas_price = u64::from_str_radix(&gas_price_hex[2..], 16)
            .map_err(|_| EthereumError::NetworkError("Invalid gas price format".to_string()))?;
        
        Ok(GasPriceData {
            base_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            gas_price: Some(gas_price.to_string()),
        })
    }
    
    /// Estimate gas limit for a transaction
    async fn estimate_gas_limit(
        &self,
        chain: &EvmChain,
        to: Option<&str>,
        data: Option<&str>,
        value: Option<&str>,
    ) -> Result<u64, EthereumError> {
        // Use RPC eth_estimateGas if available, otherwise use defaults
        match (to, data) {
            (Some(_), None) => {
                // Simple ETH transfer
                Ok(super::constants::ETH_TRANSFER_GAS_LIMIT)
            },
            (Some(_), Some(data)) if data.len() > 10 => {
                // Contract interaction - estimate higher
                if data.starts_with("0xa9059cbb") { // ERC-20 transfer
                    Ok(super::constants::ERC20_TRANSFER_GAS_LIMIT)
                } else {
                    // Complex contract call
                    Ok(200_000)
                }
            },
            (None, Some(_)) => {
                // Contract deployment
                Ok(1_000_000)
            },
            _ => {
                // Default case
                Ok(super::constants::ETH_TRANSFER_GAS_LIMIT)
            }
        }
    }
    
    /// Calculate gas fees based on priority and chain capabilities
    fn calculate_gas_fees(
        &self,
        chain: &EvmChain,
        gas_prices: &GasPriceData,
        priority: GasPriority,
    ) -> Result<(String, String, String), EthereumError> {
        if self.chain_config.supports_eip1559(chain) {
            self.calculate_eip1559_fees(gas_prices, priority)
        } else {
            self.calculate_legacy_fees(gas_prices, priority)
        }
    }
    
    /// Calculate EIP-1559 fees
    fn calculate_eip1559_fees(
        &self,
        gas_prices: &GasPriceData,
        priority: GasPriority,
    ) -> Result<(String, String, String), EthereumError> {
        let base_fee = gas_prices.base_fee_per_gas.as_ref()
            .ok_or_else(|| EthereumError::GasEstimationFailed("No base fee".to_string()))?
            .parse::<u64>()
            .map_err(|_| EthereumError::GasEstimationFailed("Invalid base fee".to_string()))?;
        
        let current_priority_fee = gas_prices.max_priority_fee_per_gas.as_ref()
            .ok_or_else(|| EthereumError::GasEstimationFailed("No priority fee".to_string()))?
            .parse::<u64>()
            .map_err(|_| EthereumError::GasEstimationFailed("Invalid priority fee".to_string()))?;
        
        // Adjust fees based on priority
        let priority_multiplier = match priority {
            GasPriority::Low => 0.8,
            GasPriority::Medium => 1.0,
            GasPriority::High => 1.2,
            GasPriority::Urgent => 1.5,
        };
        
        let max_priority_fee_per_gas = ((current_priority_fee as f64) * priority_multiplier) as u64;
        let max_fee_per_gas = base_fee * 2 + max_priority_fee_per_gas; // Base fee buffer
        let gas_price = base_fee + max_priority_fee_per_gas;
        
        Ok((
            gas_price.to_string(),
            max_fee_per_gas.to_string(),
            max_priority_fee_per_gas.to_string(),
        ))
    }
    
    /// Calculate legacy fees
    fn calculate_legacy_fees(
        &self,
        gas_prices: &GasPriceData,
        priority: GasPriority,
    ) -> Result<(String, String, String), EthereumError> {
        let base_gas_price = gas_prices.gas_price.as_ref()
            .ok_or_else(|| EthereumError::GasEstimationFailed("No gas price".to_string()))?
            .parse::<u64>()
            .map_err(|_| EthereumError::GasEstimationFailed("Invalid gas price".to_string()))?;
        
        let priority_multiplier = match priority {
            GasPriority::Low => 0.9,
            GasPriority::Medium => 1.0,
            GasPriority::High => 1.1,
            GasPriority::Urgent => 1.25,
        };
        
        let adjusted_gas_price = ((base_gas_price as f64) * priority_multiplier) as u64;
        
        Ok((
            adjusted_gas_price.to_string(),
            adjusted_gas_price.to_string(), // Same as gas price for legacy
            "0".to_string(), // No priority fee in legacy
        ))
    }
    
    /// Estimate confirmation time based on priority
    fn estimate_confirmation_time(&self, chain: &EvmChain, priority: GasPriority) -> Result<u64, EthereumError> {
        let base_time = self.chain_config.get_config(chain)
            .map(|config| config.average_block_time_seconds)
            .unwrap_or(12); // Default to Ethereum block time
        
        let blocks_multiplier = match priority {
            GasPriority::Low => 5,
            GasPriority::Medium => 2,
            GasPriority::High => 1,
            GasPriority::Urgent => 1,
        };
        
        Ok(base_time * blocks_multiplier)
    }
    
    /// Convert ETH to USD (simplified - would use price oracle in production)
    async fn convert_eth_to_usd(&self, eth_amount: f64, chain: &EvmChain) -> Result<f64, EthereumError> {
        // Simplified conversion - in production, this would fetch from price oracle
        let eth_price_usd = match chain {
            EvmChain::Polygon => 0.8, // MATIC price approximation
            EvmChain::Avalanche => 25.0, // AVAX price approximation
            _ => 2000.0, // ETH price approximation
        };
        
        Ok(eth_amount * eth_price_usd)
    }
    
    /// Get cached gas price if available and fresh
    fn get_cached_gas_price(&self, chain: &EvmChain) -> Option<CachedGasPrice> {
        let cache = self.price_cache.borrow();
        if let Some(cached) = cache.get(chain) {
            let now = ic_cdk::api::time();
            if now - cached.timestamp < cached.ttl_seconds * 1_000_000_000 {
                return Some(cached.clone());
            }
        }
        None
    }
    
    /// Cache gas prices
    fn cache_gas_prices(&self, chain: &EvmChain, gas_prices: &GasPriceData) {
        let cached = CachedGasPrice {
            timestamp: ic_cdk::api::time(),
            base_fee: gas_prices.base_fee_per_gas.as_ref()
                .and_then(|fee| fee.parse().ok())
                .unwrap_or(0),
            priority_fee: gas_prices.max_priority_fee_per_gas.as_ref()
                .and_then(|fee| fee.parse().ok())
                .unwrap_or(0),
            gas_price: gas_prices.gas_price.as_ref()
                .and_then(|fee| fee.parse().ok())
                .unwrap_or(0),
            ttl_seconds: 30, // Cache for 30 seconds
        };
        
        self.price_cache.borrow_mut().insert(chain.clone(), cached);
    }
    
    /// Get EIP-1559 fee recommendation
    pub async fn get_eip1559_recommendation(
        &self,
        chain: &EvmChain,
        priority: GasPriority,
    ) -> Result<EIP1559FeeRecommendation, EthereumError> {
        if !self.chain_config.supports_eip1559(chain) {
            return Err(EthereumError::ChainNotSupported(
                format!("{} does not support EIP-1559", chain.name())
            ));
        }
        
        let gas_prices = self.get_gas_prices(chain).await?;
        let (_, max_fee_per_gas, max_priority_fee_per_gas) = 
            self.calculate_eip1559_fees(&gas_prices, priority)?;
        
        let confirmation_time = self.estimate_confirmation_time(chain, priority)?;
        
        Ok(EIP1559FeeRecommendation {
            base_fee_per_gas: gas_prices.base_fee_per_gas.unwrap_or_default(),
            max_priority_fee_per_gas,
            max_fee_per_gas,
            estimated_confirmation_time_seconds: confirmation_time,
        })
    }
}

impl Default for GasEstimator {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP response transform function for gas price requests
#[ic_cdk::query]
fn transform_gas_price_response(raw: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: raw.response.status,
        body: raw.response.body,
        headers: vec![],
    }
}

/// Gas optimization utilities
pub mod optimization {
    use super::*;
    
    /// Find the most cost-effective chain for a transaction
    pub async fn find_optimal_chain(
        estimator: &GasEstimator,
        chains: &[EvmChain],
        priority: GasPriority,
        amount_usd: f64,
    ) -> Result<EvmChain, EthereumError> {
        let mut best_chain = chains[0];
        let mut best_cost = f64::MAX;
        
        for chain in chains {
            if let Ok(estimate) = estimator.estimate_gas(chain, None, None, None, priority.clone()).await {
                // Consider both absolute cost and percentage of transaction amount
                let cost_score = estimate.total_fee_usd + (estimate.total_fee_usd / amount_usd) * 100.0;
                
                if cost_score < best_cost {
                    best_cost = cost_score;
                    best_chain = *chain;
                }
            }
        }
        
        Ok(best_chain)
    }
    
    /// Calculate potential savings by using L2 instead of Ethereum
    pub async fn calculate_l2_savings(
        estimator: &GasEstimator,
        l2_chain: &EvmChain,
        priority: GasPriority,
    ) -> Result<f64, EthereumError> {
        let ethereum_estimate = estimator.estimate_gas(&EvmChain::Ethereum, None, None, None, priority.clone()).await?;
        let l2_estimate = estimator.estimate_gas(l2_chain, None, None, None, priority).await?;
        
        Ok(ethereum_estimate.total_fee_usd - l2_estimate.total_fee_usd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gas_priority_multiplier() {
        let estimator = GasEstimator::new();
        let gas_prices = GasPriceData {
            base_fee_per_gas: Some("20000000000".to_string()), // 20 gwei
            max_priority_fee_per_gas: Some("2000000000".to_string()), // 2 gwei
            gas_price: Some("22000000000".to_string()), // 22 gwei
        };
        
        let (_, max_fee, priority_fee) = estimator.calculate_eip1559_fees(&gas_prices, GasPriority::High).unwrap();
        
        // High priority should increase fees
        assert!(priority_fee.parse::<u64>().unwrap() > 2000000000);
        assert!(max_fee.parse::<u64>().unwrap() > 42000000000); // base * 2 + priority
    }
    
    #[test]
    fn test_gas_limit_estimation() {
        // ETH transfer should use standard gas limit
        assert_eq!(super::constants::ETH_TRANSFER_GAS_LIMIT, 21_000);
        
        // ERC-20 transfer should use higher limit
        assert_eq!(super::constants::ERC20_TRANSFER_GAS_LIMIT, 65_000);
    }
}