// ICP EVM RPC Canister Integration
// Official integration with canister 7hfb6-caaaa-aaaar-qadga-cai

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use ic_cdk::api::call::call_raw;
use super::{EvmChain, EthereumError};

/// Official EVM RPC canister principal
pub const EVM_RPC_CANISTER_ID: &str = "7hfb6-caaaa-aaaar-qadga-cai";

/// EVM RPC Service for interacting with Ethereum and other EVM chains
#[derive(Debug, Clone)]
pub struct EvmRpcService {
    pub canister_id: Principal,
}

impl EvmRpcService {
    /// Create a new EVM RPC service instance
    pub fn new() -> Self {
        Self {
            canister_id: Principal::from_text(EVM_RPC_CANISTER_ID)
                .expect("Invalid EVM RPC canister principal"),
        }
    }

    /// Get balance of an Ethereum address
    pub async fn eth_get_balance(
        &self,
        address: &str,
        chain: &EvmChain,
        block_tag: Option<BlockTag>,
    ) -> Result<String, EthereumError> {
        let request = EthGetBalanceRequest {
            address: address.to_string(),
            block: block_tag.unwrap_or(BlockTag::Latest),
        };

        let rpc_config = self.get_rpc_config(chain);
        let call_result = self.make_rpc_call("eth_getBalance", &request, &rpc_config).await?;
        
        match call_result {
            RpcResult::Ok(balance) => Ok(balance),
            RpcResult::Err(error) => Err(EthereumError::RpcError(error.message)),
            RpcResult::Inconsistent(results) => {
                // Handle inconsistent results by taking the most common response
                self.resolve_inconsistent_balance_results(results)
            }
        }
    }

    /// Get transaction count (nonce) for an address
    pub async fn eth_get_transaction_count(
        &self,
        address: &str,
        chain: &EvmChain,
        block_tag: Option<BlockTag>,
    ) -> Result<u64, EthereumError> {
        let request = EthGetTransactionCountRequest {
            address: address.to_string(),
            block: block_tag.unwrap_or(BlockTag::Pending),
        };

        let rpc_config = self.get_rpc_config(chain);
        let call_result = self.make_rpc_call("eth_getTransactionCount", &request, &rpc_config).await?;
        
        match call_result {
            RpcResult::Ok(nonce_hex) => {
                let nonce = u64::from_str_radix(&nonce_hex[2..], 16)
                    .map_err(|_| EthereumError::SerializationError("Invalid nonce format".to_string()))?;
                Ok(nonce)
            },
            RpcResult::Err(error) => Err(EthereumError::RpcError(error.message)),
            RpcResult::Inconsistent(results) => {
                self.resolve_inconsistent_nonce_results(results)
            }
        }
    }

    /// Get gas price information
    pub async fn eth_gas_price(&self, chain: &EvmChain) -> Result<String, EthereumError> {
        let request = EthGasPriceRequest {};
        let rpc_config = self.get_rpc_config(chain);
        let call_result = self.make_rpc_call("eth_gasPrice", &request, &rpc_config).await?;
        
        match call_result {
            RpcResult::Ok(gas_price) => Ok(gas_price),
            RpcResult::Err(error) => Err(EthereumError::RpcError(error.message)),
            RpcResult::Inconsistent(results) => {
                self.resolve_inconsistent_gas_price_results(results)
            }
        }
    }

    /// Get fee history for EIP-1559 gas estimation
    pub async fn eth_fee_history(
        &self,
        chain: &EvmChain,
        block_count: u32,
        newest_block: BlockTag,
        reward_percentiles: Vec<f64>,
    ) -> Result<FeeHistory, EthereumError> {
        let request = EthFeeHistoryRequest {
            block_count,
            newest_block,
            reward_percentiles,
        };

        let rpc_config = self.get_rpc_config(chain);
        let call_result = self.make_rpc_call("eth_feeHistory", &request, &rpc_config).await?;
        
        match call_result {
            RpcResult::Ok(fee_history_json) => {
                serde_json::from_str(&fee_history_json)
                    .map_err(|e| EthereumError::SerializationError(format!("Fee history parse error: {}", e)))
            },
            RpcResult::Err(error) => Err(EthereumError::RpcError(error.message)),
            RpcResult::Inconsistent(results) => {
                self.resolve_inconsistent_fee_history_results(results)
            }
        }
    }

    /// Send raw transaction to the network
    pub async fn eth_send_raw_transaction(
        &self,
        chain: &EvmChain,
        raw_transaction: &str,
    ) -> Result<String, EthereumError> {
        let request = EthSendRawTransactionRequest {
            raw_transaction_hex: raw_transaction.to_string(),
        };

        let rpc_config = self.get_rpc_config(chain);
        let call_result = self.make_rpc_call("eth_sendRawTransaction", &request, &rpc_config).await?;
        
        match call_result {
            RpcResult::Ok(tx_hash) => Ok(tx_hash),
            RpcResult::Err(error) => Err(EthereumError::RpcError(error.message)),
            RpcResult::Inconsistent(_) => {
                // For transaction submission, inconsistency is an error
                Err(EthereumError::NetworkError("Inconsistent transaction submission results".to_string()))
            }
        }
    }

    /// Estimate gas for a transaction
    pub async fn eth_estimate_gas(
        &self,
        chain: &EvmChain,
        to: &str,
        from: Option<&str>,
        value: Option<&str>,
        data: Option<&str>,
    ) -> Result<u64, EthereumError> {
        let request = EthEstimateGasRequest {
            to: to.to_string(),
            from: from.map(|s| s.to_string()),
            value: value.map(|s| s.to_string()),
            data: data.map(|s| s.to_string()),
        };

        let rpc_config = self.get_rpc_config(chain);
        let call_result = self.make_rpc_call("eth_estimateGas", &request, &rpc_config).await?;
        
        match call_result {
            RpcResult::Ok(gas_hex) => {
                let gas = u64::from_str_radix(&gas_hex[2..], 16)
                    .map_err(|_| EthereumError::SerializationError("Invalid gas format".to_string()))?;
                Ok(gas)
            },
            RpcResult::Err(error) => Err(EthereumError::RpcError(error.message)),
            RpcResult::Inconsistent(results) => {
                self.resolve_inconsistent_gas_estimate_results(results)
            }
        }
    }

    /// Make RPC call to the EVM RPC canister
    async fn make_rpc_call<T: CandidType>(
        &self,
        method: &str,
        request: &T,
        config: &RpcConfig,
    ) -> Result<RpcResult<String>, EthereumError> {
        let cycles = self.calculate_cycles_for_call(method);
        
        let rpc_request = RpcRequest {
            method: method.to_string(),
            params: candid::encode_one(request)
                .map_err(|e| EthereumError::SerializationError(format!("Candid encoding error: {}", e)))?,
            config: config.clone(),
        };

        let call_result = call_raw(
            self.canister_id,
            "request",
            &candid::encode_one(&rpc_request)
                .map_err(|e| EthereumError::SerializationError(format!("Request encoding error: {}", e)))?,
            cycles,
        ).await;

        match call_result {
            Ok(response_bytes) => {
                candid::decode_one(&response_bytes)
                    .map_err(|e| EthereumError::SerializationError(format!("Response decoding error: {}", e)))
            },
            Err((code, msg)) => {
                Err(EthereumError::NetworkError(format!("EVM RPC call failed: {} - {}", code as u8, msg)))
            }
        }
    }

    /// Get RPC configuration for a specific chain
    fn get_rpc_config(&self, chain: &EvmChain) -> RpcConfig {
        RpcConfig {
            response_consensus: Some(ResponseConsensus {
                max_response_bytes: 1024 * 1024, // 1MB
                num_providers: 3,
                min_consensus: 2,
            }),
            providers: Some(self.get_default_providers(chain)),
        }
    }

    /// Get default RPC providers for a chain
    fn get_default_providers(&self, chain: &EvmChain) -> Vec<RpcProvider> {
        match chain {
            EvmChain::Ethereum => vec![
                RpcProvider::Alchemy,
                RpcProvider::Cloudflare,
                RpcProvider::Ankr,
            ],
            EvmChain::Arbitrum => vec![
                RpcProvider::Alchemy,
                RpcProvider::Ankr,
            ],
            EvmChain::Optimism => vec![
                RpcProvider::Alchemy,
                RpcProvider::Ankr,
            ],
            EvmChain::Polygon => vec![
                RpcProvider::Alchemy,
                RpcProvider::Ankr,
            ],
            EvmChain::Base => vec![
                RpcProvider::Alchemy,
            ],
            EvmChain::Avalanche => vec![
                RpcProvider::Ankr,
            ],
        }
    }

    /// Calculate cycles needed for RPC call
    fn calculate_cycles_for_call(&self, method: &str) -> u64 {
        match method {
            "eth_getBalance" | "eth_getTransactionCount" | "eth_gasPrice" => 1_000_000_000, // 1B cycles
            "eth_feeHistory" => 2_000_000_000, // 2B cycles
            "eth_estimateGas" => 1_500_000_000, // 1.5B cycles
            "eth_sendRawTransaction" => 3_000_000_000, // 3B cycles
            _ => 1_000_000_000, // Default 1B cycles
        }
    }

    // Consensus resolution methods for handling inconsistent results

    fn resolve_inconsistent_balance_results(&self, results: Vec<ProviderResult>) -> Result<String, EthereumError> {
        let mut balance_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        for result in results {
            if let ProviderResult::Ok(balance) = result {
                *balance_counts.entry(balance).or_insert(0) += 1;
            }
        }

        balance_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(balance, _)| balance)
            .ok_or_else(|| EthereumError::NetworkError("No valid balance results".to_string()))
    }

    fn resolve_inconsistent_nonce_results(&self, results: Vec<ProviderResult>) -> Result<u64, EthereumError> {
        let mut nonce_counts: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
        
        for result in results {
            if let ProviderResult::Ok(nonce_hex) = result {
                if let Ok(nonce) = u64::from_str_radix(&nonce_hex[2..], 16) {
                    *nonce_counts.entry(nonce).or_insert(0) += 1;
                }
            }
        }

        nonce_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(nonce, _)| nonce)
            .ok_or_else(|| EthereumError::NetworkError("No valid nonce results".to_string()))
    }

    fn resolve_inconsistent_gas_price_results(&self, results: Vec<ProviderResult>) -> Result<String, EthereumError> {
        let mut gas_prices: Vec<u128> = Vec::new();
        
        for result in results {
            if let ProviderResult::Ok(gas_price_hex) = result {
                if let Ok(gas_price) = u128::from_str_radix(&gas_price_hex[2..], 16) {
                    gas_prices.push(gas_price);
                }
            }
        }

        if gas_prices.is_empty() {
            return Err(EthereumError::NetworkError("No valid gas price results".to_string()));
        }

        // Use median gas price
        gas_prices.sort();
        let median_gas_price = gas_prices[gas_prices.len() / 2];
        Ok(format!("0x{:x}", median_gas_price))
    }

    fn resolve_inconsistent_gas_estimate_results(&self, results: Vec<ProviderResult>) -> Result<u64, EthereumError> {
        let mut gas_estimates: Vec<u64> = Vec::new();
        
        for result in results {
            if let ProviderResult::Ok(gas_hex) = result {
                if let Ok(gas) = u64::from_str_radix(&gas_hex[2..], 16) {
                    gas_estimates.push(gas);
                }
            }
        }

        if gas_estimates.is_empty() {
            return Err(EthereumError::NetworkError("No valid gas estimate results".to_string()));
        }

        // Use maximum gas estimate for safety
        match gas_estimates.iter().max() {
            Some(max_gas) => Ok(*max_gas),
            None => {
                // DEMO: Fallback to reasonable default gas limit
                Ok(21000) // Standard ETH transfer gas limit
            }
        }
    }

    fn resolve_inconsistent_fee_history_results(&self, results: Vec<ProviderResult>) -> Result<FeeHistory, EthereumError> {
        // For fee history, we'll take the first valid result
        // In production, more sophisticated consensus logic would be needed
        for result in results {
            if let ProviderResult::Ok(fee_history_json) = result {
                if let Ok(fee_history) = serde_json::from_str::<FeeHistory>(&fee_history_json) {
                    return Ok(fee_history);
                }
            }
        }
        
        Err(EthereumError::NetworkError("No valid fee history results".to_string()))
    }
}

impl Default for EvmRpcService {
    fn default() -> Self {
        Self::new()
    }
}

// EVM RPC Canister Types (based on official Candid interface)

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RpcRequest {
    pub method: String,
    pub params: Vec<u8>,
    pub config: RpcConfig,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RpcConfig {
    pub response_consensus: Option<ResponseConsensus>,
    pub providers: Option<Vec<RpcProvider>>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ResponseConsensus {
    pub max_response_bytes: u64,
    pub num_providers: u32,
    pub min_consensus: u32,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum RpcProvider {
    Alchemy,
    Ankr,
    BlockPi,
    Cloudflare,
    LlamaNodes,
    PublicNode,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum RpcResult<T> {
    Ok(T),
    Err(RpcError),
    Inconsistent(Vec<ProviderResult>),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ProviderResult {
    Ok(String),
    Err(RpcError),
}

// Request/Response Types

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EthGetBalanceRequest {
    pub address: String,
    pub block: BlockTag,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EthGetTransactionCountRequest {
    pub address: String,
    pub block: BlockTag,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EthGasPriceRequest {}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EthFeeHistoryRequest {
    pub block_count: u32,
    pub newest_block: BlockTag,
    pub reward_percentiles: Vec<f64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EthEstimateGasRequest {
    pub to: String,
    pub from: Option<String>,
    pub value: Option<String>,
    pub data: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EthSendRawTransactionRequest {
    pub raw_transaction_hex: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum BlockTag {
    Latest,
    Earliest,
    Pending,
    Safe,
    Finalized,
    Number(u64),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FeeHistory {
    pub base_fee_per_gas: Vec<String>,
    pub gas_used_ratio: Vec<f64>,
    pub oldest_block: String,
    pub reward: Option<Vec<Vec<String>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_rpc_service_creation() {
        let service = EvmRpcService::new();
        assert_eq!(service.canister_id.to_text(), EVM_RPC_CANISTER_ID);
    }

    #[test]
    fn test_cycle_calculation() {
        let service = EvmRpcService::new();
        assert_eq!(service.calculate_cycles_for_call("eth_getBalance"), 1_000_000_000);
        assert_eq!(service.calculate_cycles_for_call("eth_sendRawTransaction"), 3_000_000_000);
    }

    #[test]
    fn test_provider_selection() {
        let service = EvmRpcService::new();
        let ethereum_providers = service.get_default_providers(&EvmChain::Ethereum);
        assert_eq!(ethereum_providers.len(), 3);
        assert!(ethereum_providers.contains(&RpcProvider::Alchemy));
        assert!(ethereum_providers.contains(&RpcProvider::Cloudflare));
    }
}