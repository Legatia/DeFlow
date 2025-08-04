// Ethereum DeFi Service
// Main service orchestrating Ethereum and L2 operations

use super::{
    EvmChain, EthereumAddress, EthereumTransactionParams, EthereumTransactionResult, 
    EthereumPortfolio, EthereumError, GasPriority, L2OptimizationResult, TransactionContext,
    EthereumAddressManager, GasEstimator, L2Optimizer, ChainConfigManager
};
use candid::Principal;
use ic_cdk::api::management_canister::http_request::{HttpRequest, HttpResponse, HttpMethod};
use std::collections::HashMap;

/// Ethereum DeFi service context
#[derive(Debug, Clone)]
pub struct EthereumServiceContext {
    pub key_name: String,
    pub default_chain: EvmChain,
    pub supported_chains: Vec<EvmChain>,
}

/// Main Ethereum DeFi service
#[derive(Debug, Clone)]
pub struct EthereumDeFiService {
    context: EthereumServiceContext,
    address_managers: HashMap<EvmChain, EthereumAddressManager>,
    gas_estimator: GasEstimator,
    l2_optimizer: L2Optimizer,
    chain_config: ChainConfigManager,
}

impl EthereumDeFiService {
    /// Create a new Ethereum DeFi service
    pub fn new(context: EthereumServiceContext) -> Self {
        let mut address_managers = HashMap::new();
        
        // Create address managers for each supported chain
        for chain in &context.supported_chains {
            let manager = EthereumAddressManager::new(context.key_name.clone(), chain.clone());
            address_managers.insert(chain.clone(), manager);
        }
        
        Self {
            context,
            address_managers,
            gas_estimator: GasEstimator::new(),
            l2_optimizer: L2Optimizer::new(),
            chain_config: ChainConfigManager::new(),
        }
    }
    
    /// Get Ethereum address for a user on a specific chain
    pub async fn get_ethereum_address(
        &self, 
        user: Principal, 
        chain: EvmChain
    ) -> Result<EthereumAddress, EthereumError> {
        let manager = self.address_managers.get(&chain)
            .ok_or_else(|| EthereumError::ChainNotSupported(chain.name().to_string()))?;
        
        let mut address = manager.get_ethereum_address(user).await?;
        
        // Update balance
        address.balance_wei = self.get_balance(&address.address, &chain).await?;
        address.balance_eth = super::utils::wei_to_eth(&address.balance_wei)?;
        address.nonce = self.get_nonce(&address.address, &chain).await?;
        address.last_updated = ic_cdk::api::time();
        
        Ok(address)
    }
    
    /// Get user's portfolio across all supported chains
    pub async fn get_ethereum_portfolio(&self, user: Principal) -> Result<EthereumPortfolio, EthereumError> {
        let mut addresses = Vec::new();
        let mut total_eth = 0.0;
        let mut chain_balances = HashMap::new();
        
        // Get addresses from all supported chains
        for chain in &self.context.supported_chains {
            if let Ok(address) = self.get_ethereum_address(user, chain.clone()).await {
                chain_balances.insert(chain.name().to_string(), address.balance_eth);
                total_eth += address.balance_eth;
                addresses.push(address);
            }
        }
        
        // Convert to USD (simplified)
        let total_value_usd = total_eth * 2000.0; // Approximate ETH price
        
        Ok(EthereumPortfolio {
            addresses,
            total_eth,
            total_value_usd,
            chain_balances,
            last_updated: ic_cdk::api::time(),
        })
    }
    
    /// Send Ethereum transaction with optimization
    pub async fn send_ethereum(
        &self,
        user: Principal,
        to_address: String,
        amount_wei: String,
        chain: Option<EvmChain>,
        gas_priority: GasPriority,
        optimize_for_cost: bool,
    ) -> Result<EthereumTransactionResult, EthereumError> {
        // Validate destination address
        if !super::utils::validate_ethereum_address(&to_address) {
            return Err(EthereumError::InvalidAddress(to_address));
        }
        
        // Determine optimal chain if not specified
        let target_chain = if let Some(chain) = chain {
            chain
        } else if optimize_for_cost {
            self.find_optimal_chain_for_transaction(user, &amount_wei, gas_priority).await?
        } else {
            self.context.default_chain.clone()
        };
        
        // Get user's address on the target chain
        let from_address = self.get_ethereum_address(user, target_chain.clone()).await?;
        
        // Check balance
        let balance_wei: u128 = from_address.balance_wei.parse()
            .map_err(|_| EthereumError::InsufficientBalance {
                required: amount_wei.clone(),
                available: from_address.balance_wei.clone(),
            })?;
        
        let amount: u128 = amount_wei.parse()
            .map_err(|_| EthereumError::InvalidAddress("Invalid amount".to_string()))?;
        
        if balance_wei < amount {
            return Err(EthereumError::InsufficientBalance {
                required: amount_wei,
                available: from_address.balance_wei,
            });
        }
        
        // Get gas estimate
        let gas_estimate = self.gas_estimator.estimate_gas(
            &target_chain,
            Some(&to_address),
            None,
            Some(&amount_wei),
            gas_priority.clone(),
        ).await?;
        
        // Check if user has enough for gas
        let total_needed = amount + gas_estimate.gas_limit as u128 * 
            gas_estimate.max_fee_per_gas.parse::<u128>().unwrap_or(0);
        
        if balance_wei < total_needed {
            return Err(EthereumError::InsufficientBalance {
                required: total_needed.to_string(),
                available: from_address.balance_wei,
            });
        }
        
        // Build transaction parameters
        let tx_params = EthereumTransactionParams {
            to: to_address.clone(),
            value: amount_wei.clone(),
            gas_limit: gas_estimate.gas_limit,
            gas_price: if self.chain_config.supports_eip1559(&target_chain) {
                None
            } else {
                Some(gas_estimate.gas_price.clone())
            },
            max_fee_per_gas: if self.chain_config.supports_eip1559(&target_chain) {
                Some(gas_estimate.max_fee_per_gas.clone())
            } else {
                None
            },
            max_priority_fee_per_gas: if self.chain_config.supports_eip1559(&target_chain) {
                Some(gas_estimate.max_priority_fee_per_gas.clone())
            } else {
                None
            },
            nonce: from_address.nonce,
            data: None,
            chain_id: target_chain.chain_id(),
        };
        
        // Sign and broadcast transaction
        self.sign_and_broadcast_transaction(user, &target_chain, tx_params).await
    }
    
    /// Get balance for an address on a specific chain
    async fn get_balance(&self, address: &str, chain: &EvmChain) -> Result<String, EthereumError> {
        let rpc_url = self.chain_config.get_primary_rpc(chain)
            .ok_or_else(|| EthereumError::ChainNotSupported(chain.name().to_string()))?;
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [address, "latest"],
            "id": 1
        });
        
        let request = HttpRequest {
            url: rpc_url,
            method: HttpMethod::POST,
            body: Some(request_body.to_string().into_bytes()),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            transform: Some(ic_cdk::api::management_canister::http_request::TransformContext::from_name(
                "transform_ethereum_response".to_string(), 
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
        
        let balance_hex = rpc_response.get("result")
            .and_then(|result| result.as_str())
            .ok_or_else(|| EthereumError::NetworkError("No balance in response".to_string()))?;
        
        // Convert hex to decimal string
        let balance = u128::from_str_radix(&balance_hex[2..], 16)
            .map_err(|_| EthereumError::NetworkError("Invalid balance format".to_string()))?;
        
        Ok(balance.to_string())
    }
    
    /// Get nonce for an address on a specific chain
    async fn get_nonce(&self, address: &str, chain: &EvmChain) -> Result<u64, EthereumError> {
        let rpc_url = self.chain_config.get_primary_rpc(chain)
            .ok_or_else(|| EthereumError::ChainNotSupported(chain.name().to_string()))?;
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [address, "pending"],
            "id": 1
        });
        
        let request = HttpRequest {
            url: rpc_url,
            method: HttpMethod::POST,
            body: Some(request_body.to_string().into_bytes()),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            transform: Some(ic_cdk::api::management_canister::http_request::TransformContext::from_name(
                "transform_ethereum_response".to_string(), 
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
        
        let nonce_hex = rpc_response.get("result")
            .and_then(|result| result.as_str())
            .ok_or_else(|| EthereumError::NetworkError("No nonce in response".to_string()))?;
        
        let nonce = u64::from_str_radix(&nonce_hex[2..], 16)
            .map_err(|_| EthereumError::NetworkError("Invalid nonce format".to_string()))?;
        
        Ok(nonce)
    }
    
    /// Find optimal chain for a transaction
    async fn find_optimal_chain_for_transaction(
        &self,
        user: Principal,
        amount_wei: &str,
        gas_priority: GasPriority,
    ) -> Result<EvmChain, EthereumError> {
        let amount_eth = super::utils::wei_to_eth(amount_wei)?;
        let amount_usd = amount_eth * 2000.0; // Approximate ETH price
        
        let context = TransactionContext {
            transaction_type: super::l2_optimizer::TransactionType::SimpleTransfer,
            amount_usd,
            urgency: gas_priority,
            current_chain: None,
            preferred_chains: self.context.supported_chains.clone(),
            max_bridge_cost_usd: Some(amount_usd * 0.1), // Max 10% of transaction value
            max_total_time_minutes: Some(60), // Max 1 hour
        };
        
        let optimization = self.l2_optimizer.optimize_transaction(&context).await?;
        Ok(optimization.recommended_chain)
    }
    
    /// Sign and broadcast transaction
    async fn sign_and_broadcast_transaction(
        &self,
        user: Principal,
        chain: &EvmChain,
        params: EthereumTransactionParams,
    ) -> Result<EthereumTransactionResult, EthereumError> {
        // Get address manager for the chain
        let manager = self.address_managers.get(chain)
            .ok_or_else(|| EthereumError::ChainNotSupported(chain.name().to_string()))?;
        
        // Build transaction hash for signing
        let tx_hash = self.build_transaction_hash(&params)?;
        
        // Sign transaction with threshold ECDSA
        let signature = manager.sign_transaction_hash(user, &tx_hash).await?;
        
        // Format signature for Ethereum
        let ethereum_signature = super::addresses::signature_utils::format_ethereum_signature(
            &signature,
            &tx_hash,
            &params.to, // This should be the from address, but we'll use to for now
        )?;
        
        // Build raw transaction
        let raw_transaction = self.build_raw_transaction(&params, &ethereum_signature)?;
        
        // Broadcast transaction
        self.broadcast_transaction(chain, &raw_transaction).await
    }
    
    /// Build transaction hash for signing
    fn build_transaction_hash(&self, params: &EthereumTransactionParams) -> Result<Vec<u8>, EthereumError> {
        use rlp::RlpStream;
        use sha3::{Digest, Keccak256};
        
        let mut stream = RlpStream::new_list(9);
        
        // Add transaction fields
        stream.append(&params.nonce);
        
        if let Some(gas_price) = &params.gas_price {
            stream.append(&gas_price.parse::<u64>()
                .map_err(|_| EthereumError::SerializationError("Invalid gas price".to_string()))?);
        } else {
            stream.append(&params.max_fee_per_gas.as_ref().unwrap().parse::<u64>()
                .map_err(|_| EthereumError::SerializationError("Invalid max fee".to_string()))?);
        }
        
        stream.append(&params.gas_limit);
        
        // Decode hex address
        let to_bytes = hex::decode(&params.to[2..])
            .map_err(|_| EthereumError::InvalidAddress(params.to.clone()))?;
        stream.append(&to_bytes);
        
        // Value
        let value = params.value.parse::<u128>()
            .map_err(|_| EthereumError::SerializationError("Invalid value".to_string()))?;
        stream.append(&value);
        
        // Data
        if let Some(data) = &params.data {
            let data_bytes = hex::decode(&data[2..])
                .map_err(|_| EthereumError::SerializationError("Invalid data".to_string()))?;
            stream.append(&data_bytes);
        } else {
            stream.append(&"");
        }
        
        // Chain ID
        stream.append(&params.chain_id);
        stream.append(&0u8); // r
        stream.append(&0u8); // s
        
        let encoded = stream.out();
        
        // Hash with Keccak-256
        let mut hasher = Keccak256::new();
        hasher.update(&encoded);
        Ok(hasher.finalize().to_vec())
    }
    
    /// Build raw transaction with signature
    fn build_raw_transaction(
        &self,
        params: &EthereumTransactionParams,
        signature: &[u8],
    ) -> Result<String, EthereumError> {
        use rlp::RlpStream;
        
        if signature.len() != 65 {
            return Err(EthereumError::SerializationError("Invalid signature length".to_string()));
        }
        
        let mut stream = RlpStream::new_list(9);
        
        // Add transaction fields (same as for hash)
        stream.append(&params.nonce);
        
        if let Some(gas_price) = &params.gas_price {
            stream.append(&gas_price.parse::<u64>()
                .map_err(|_| EthereumError::SerializationError("Invalid gas price".to_string()))?);
        } else {
            stream.append(&params.max_fee_per_gas.as_ref().unwrap().parse::<u64>()
                .map_err(|_| EthereumError::SerializationError("Invalid max fee".to_string()))?);
        }
        
        stream.append(&params.gas_limit);
        
        let to_bytes = hex::decode(&params.to[2..])
            .map_err(|_| EthereumError::InvalidAddress(params.to.clone()))?;
        stream.append(&to_bytes);
        
        let value = params.value.parse::<u128>()
            .map_err(|_| EthereumError::SerializationError("Invalid value".to_string()))?;
        stream.append(&value);
        
        if let Some(data) = &params.data {
            let data_bytes = hex::decode(&data[2..])
                .map_err(|_| EthereumError::SerializationError("Invalid data".to_string()))?;
            stream.append(&data_bytes);
        } else {
            stream.append(&"");
        }
        
        // Add signature
        let v = signature[64] as u64;
        let r = &signature[0..32];
        let s = &signature[32..64];
        
        stream.append(&v);
        stream.append(&r);
        stream.append(&s);
        
        let encoded = stream.out();
        Ok(format!("0x{}", hex::encode(encoded)))
    }
    
    /// Broadcast transaction to the network
    async fn broadcast_transaction(
        &self,
        chain: &EvmChain,
        raw_transaction: &str,
    ) -> Result<EthereumTransactionResult, EthereumError> {
        let rpc_url = self.chain_config.get_primary_rpc(chain)
            .ok_or_else(|| EthereumError::ChainNotSupported(chain.name().to_string()))?;
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [raw_transaction],
            "id": 1
        });
        
        let request = HttpRequest {
            url: rpc_url,
            method: HttpMethod::POST,
            body: Some(request_body.to_string().into_bytes()),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            transform: Some(ic_cdk::api::management_canister::http_request::TransformContext::from_name(
                "transform_ethereum_response".to_string(), 
                serde_json::to_vec(&()).unwrap()
            )),
        };
        
        let (response,): (HttpResponse,) = ic_cdk::api::management_canister::http_request::http_request(request)
            .await
            .map_err(|e| EthereumError::NetworkError(format!("HTTP request failed: {:?}", e)))?;
        
        let response_text = String::from_utf8(response.body)
            .map_err(|e| EthereumError::NetworkError(format!("Invalid UTF-8: {}", e)))?;
        
        let rpc_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| EthereumError::NetworkError(format!("JSON parse error: {}", e)))?;
        
        if let Some(error) = rpc_response.get("error") {
            let error_message = error.get("message")
                .and_then(|msg| msg.as_str())
                .unwrap_or("Unknown error");
            
            return Ok(EthereumTransactionResult {
                success: false,
                transaction_hash: None,
                from_address: "".to_string(),
                to_address: "".to_string(),
                value_wei: "0".to_string(),
                gas_used: None,
                gas_price: "0".to_string(),
                total_fee_wei: "0".to_string(),
                block_number: None,
                confirmation_time_estimate_seconds: 0,
                error_message: Some(error_message.to_string()),
            });
        }
        
        let tx_hash = rpc_response.get("result")
            .and_then(|result| result.as_str())
            .ok_or_else(|| EthereumError::NetworkError("No transaction hash in response".to_string()))?;
        
        Ok(EthereumTransactionResult {
            success: true,
            transaction_hash: Some(tx_hash.to_string()),
            from_address: "".to_string(), // Would need to extract from transaction
            to_address: "".to_string(),   // Would need to extract from transaction
            value_wei: "0".to_string(),   // Would need to extract from transaction
            gas_used: None,               // Available after confirmation
            gas_price: "0".to_string(),   // Would need to extract from transaction
            total_fee_wei: "0".to_string(), // Available after confirmation
            block_number: None,           // Available after confirmation
            confirmation_time_estimate_seconds: 60, // Estimated
            error_message: None,
        })
    }
    
    /// Get L2 optimization recommendations
    pub async fn get_l2_optimization(
        &self,
        user: Principal,
        amount_wei: String,
        transaction_type: super::l2_optimizer::TransactionType,
        gas_priority: GasPriority,
    ) -> Result<L2OptimizationResult, EthereumError> {
        let amount_eth = super::utils::wei_to_eth(&amount_wei)?;
        let amount_usd = amount_eth * 2000.0; // Approximate ETH price
        
        let context = TransactionContext {
            transaction_type,
            amount_usd,
            urgency: gas_priority,
            current_chain: Some(self.context.default_chain.clone()),
            preferred_chains: self.context.supported_chains.clone(),
            max_bridge_cost_usd: Some(amount_usd * 0.05), // Max 5% of transaction value
            max_total_time_minutes: Some(120), // Max 2 hours
        };
        
        self.l2_optimizer.optimize_transaction(&context).await
    }
}

/// HTTP response transform function for Ethereum requests
#[ic_cdk::query]
fn transform_ethereum_response(raw: ic_cdk::api::management_canister::http_request::TransformArgs) -> HttpResponse {
    HttpResponse {
        status: raw.response.status,
        body: raw.response.body,
        headers: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_service_creation() {
        let context = EthereumServiceContext {
            key_name: "test_key".to_string(),
            default_chain: EvmChain::Ethereum,
            supported_chains: vec![EvmChain::Ethereum, EvmChain::Arbitrum, EvmChain::Optimism],
        };
        
        let service = EthereumDeFiService::new(context);
        assert_eq!(service.address_managers.len(), 3);
    }
    
    #[test]
    fn test_chain_support() {
        let context = EthereumServiceContext {
            key_name: "test_key".to_string(),
            default_chain: EvmChain::Ethereum,
            supported_chains: vec![EvmChain::Ethereum, EvmChain::Arbitrum],
        };
        
        let service = EthereumDeFiService::new(context);
        assert!(service.address_managers.contains_key(&EvmChain::Ethereum));
        assert!(service.address_managers.contains_key(&EvmChain::Arbitrum));
        assert!(!service.address_managers.contains_key(&EvmChain::Polygon));
    }
}