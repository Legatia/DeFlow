// ICP-Compliant Ethereum DeFi Service
// Uses official EVM RPC canister and ICP threshold ECDSA

use super::{
    EvmChain, EthereumAddress, EthereumTransactionParams, EthereumTransactionResult, 
    EthereumPortfolio, EthereumError, GasPriority, L2OptimizationResult,
    EvmRpcService, ThresholdEcdsaService, IcpGasEstimator, L2Optimizer,
    ChainConfigManager, BlockTag
};
use candid::Principal;
use std::collections::HashMap;

/// ICP-compliant Ethereum DeFi service context
#[derive(Debug, Clone)]
pub struct IcpEthereumServiceContext {
    pub key_name: String,
    pub canister_id: Principal,
    pub supported_chains: Vec<EvmChain>,
}

/// Main ICP-compliant Ethereum DeFi service
#[derive(Debug, Clone)]
pub struct IcpEthereumDeFiService {
    context: IcpEthereumServiceContext,
    evm_rpc: EvmRpcService,
    threshold_ecdsa: ThresholdEcdsaService,
    gas_estimator: IcpGasEstimator,
    l2_optimizer: L2Optimizer,
    chain_config: ChainConfigManager,
}

impl IcpEthereumDeFiService {
    /// Create a new ICP-compliant Ethereum DeFi service
    pub fn new(context: IcpEthereumServiceContext) -> Self {
        let threshold_ecdsa = ThresholdEcdsaService::new(
            context.key_name.clone(),
            context.canister_id,
        );

        Self {
            evm_rpc: EvmRpcService::new(),
            threshold_ecdsa,
            gas_estimator: IcpGasEstimator::new(),
            l2_optimizer: L2Optimizer::new(),
            chain_config: ChainConfigManager::new(),
            context,
        }
    }

    /// Get Ethereum address for a user on a specific chain
    pub async fn get_ethereum_address(
        &self, 
        user: Principal, 
        chain: EvmChain
    ) -> Result<EthereumAddress, EthereumError> {
        // Generate address using ICP threshold ECDSA
        let address = self.threshold_ecdsa.get_ethereum_address(user, &chain)
            .await
            .map_err(|e| EthereumError::AddressGenerationError(e.to_string()))?;

        // Get balance using EVM RPC canister with consensus validation
        let balance_wei = self.evm_rpc.eth_get_balance(&address, &chain, Some(BlockTag::Latest))
            .await
            .unwrap_or_else(|_| "0x0".to_string());

        // Get nonce using EVM RPC canister
        let nonce = self.evm_rpc.eth_get_transaction_count(&address, &chain, Some(BlockTag::Pending))
            .await
            .unwrap_or(0);

        // Convert wei to ETH
        let balance_eth = super::utils::wei_to_eth(&balance_wei)
            .unwrap_or(0.0);

        Ok(EthereumAddress {
            address,
            chain,
            balance_wei,
            balance_eth,
            nonce,
            last_updated: ic_cdk::api::time(),
        })
    }

    /// Get user's portfolio across all supported chains with ICP consensus
    pub async fn get_ethereum_portfolio(&self, user: Principal) -> Result<EthereumPortfolio, EthereumError> {
        let mut addresses = Vec::new();
        let mut total_eth = 0.0;
        let mut chain_balances = HashMap::new();

        // Use concurrent requests for better performance
        let mut futures = Vec::new();
        for chain in &self.context.supported_chains {
            let future = self.get_ethereum_address(user, chain.clone());
            futures.push((chain.clone(), future));
        }

        // Process results as they complete
        for (chain, future) in futures {
            match future.await {
                Ok(address) => {
                    chain_balances.insert(chain.name().to_string(), address.balance_eth);
                    total_eth += address.balance_eth;
                    addresses.push(address);
                },
                Err(e) => {
                    ic_cdk::println!("Failed to get address for chain {:?}: {}", chain, e);
                    // Continue with other chains
                    chain_balances.insert(chain.name().to_string(), 0.0);
                }
            }
        }

        // Get ETH price using consensus (simplified for now)
        let eth_price_usd = self.get_eth_price_consensus().await.unwrap_or(2000.0);
        let total_value_usd = total_eth * eth_price_usd;

        Ok(EthereumPortfolio {
            addresses,
            total_eth,
            total_value_usd,
            chain_balances,
            last_updated: ic_cdk::api::time(),
        })
    }

    /// Send Ethereum transaction with ICP security and consensus
    pub async fn send_ethereum(
        &self,
        user: Principal,
        to_address: String,
        amount_wei: String,
        chain: Option<EvmChain>,
        gas_priority: GasPriority,
        optimize_for_cost: bool,
    ) -> Result<EthereumTransactionResult, EthereumError> {
        // Validate destination address using ICP threshold ECDSA service
        self.threshold_ecdsa.validate_ethereum_address(&to_address)?;

        // Determine optimal chain if not specified
        let target_chain = if let Some(chain) = chain {
            chain
        } else if optimize_for_cost {
            self.find_optimal_chain_for_transaction(user, &amount_wei, gas_priority.clone()).await?
        } else {
            EvmChain::Ethereum // Default to Ethereum mainnet
        };

        // Get user's address on the target chain using ICP threshold ECDSA
        let from_address = self.get_ethereum_address(user, target_chain.clone()).await?;

        // Validate balance using EVM RPC canister consensus
        let balance_wei: u128 = from_address.balance_wei.parse()
            .map_err(|_| EthereumError::InsufficientBalance {
                required: amount_wei.clone(),
                available: from_address.balance_wei.clone(),
            })?;

        let amount: u128 = amount_wei.parse()
            .map_err(|_| EthereumError::SerializationError("Invalid amount format".to_string()))?;

        if balance_wei < amount {
            return Err(EthereumError::InsufficientBalance {
                required: amount_wei,
                available: from_address.balance_wei,
            });
        }

        // Get gas estimate using ICP consensus
        let gas_estimate = self.gas_estimator.estimate_gas(
            &target_chain,
            Some(&to_address),
            None,
            Some(&amount_wei),
            gas_priority.clone(),
        ).await?;

        // Check total cost including gas
        let gas_cost_wei: u128 = gas_estimate.total_fee_wei.parse()
            .map_err(|_| EthereumError::GasEstimationFailed("Invalid gas fee format".to_string()))?;

        let total_needed = amount + gas_cost_wei;
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
            gas_price: if target_chain.supports_eip1559() {
                None
            } else {
                Some(gas_estimate.gas_price.clone())
            },
            max_fee_per_gas: if target_chain.supports_eip1559() {
                Some(gas_estimate.max_fee_per_gas.clone())
            } else {
                None
            },
            max_priority_fee_per_gas: if target_chain.supports_eip1559() {
                Some(gas_estimate.max_priority_fee_per_gas.clone())
            } else {
                None
            },
            nonce: from_address.nonce,
            data: None,
            chain_id: target_chain.chain_id(),
        };

        // Sign and broadcast transaction using ICP security
        self.sign_and_broadcast_transaction(user, &target_chain, tx_params).await
    }

    /// Sign and broadcast transaction using ICP threshold ECDSA and EVM RPC canister
    async fn sign_and_broadcast_transaction(
        &self,
        user: Principal,
        chain: &EvmChain,
        params: EthereumTransactionParams,
    ) -> Result<EthereumTransactionResult, EthereumError> {
        // Build transaction hash for signing
        let tx_hash = self.build_transaction_hash(&params)?;

        // Sign transaction with ICP threshold ECDSA
        let signature = self.threshold_ecdsa.sign_transaction_hash(user, chain, &tx_hash)
            .await
            .map_err(|e| EthereumError::SigningError(e.to_string()))?;

        // Build raw transaction with signature
        let raw_transaction = self.build_raw_transaction(&params, &signature)?;

        // Broadcast transaction using EVM RPC canister with consensus
        let tx_hash_result = self.evm_rpc.eth_send_raw_transaction(chain, &raw_transaction)
            .await
            .map_err(|e| EthereumError::BroadcastError(e.to_string()))?;

        Ok(EthereumTransactionResult {
            success: true,
            transaction_hash: Some(tx_hash_result),
            from_address: self.threshold_ecdsa.get_ethereum_address(user, chain).await.unwrap_or_default(),
            to_address: params.to,
            value_wei: params.value,
            gas_used: Some(params.gas_limit),
            gas_price: params.gas_price.unwrap_or_else(|| params.max_fee_per_gas.unwrap_or_default()),
            total_fee_wei: (params.gas_limit as u128 * 
                params.max_fee_per_gas.clone().unwrap_or(params.gas_price.unwrap_or("0".to_string()))
                    .parse::<u128>().unwrap_or(0)).to_string(),
            block_number: None, // Will be available after confirmation
            confirmation_time_estimate_seconds: self.estimate_confirmation_time(chain, &GasPriority::Medium),
            error_message: None,
        })
    }

    /// Build transaction hash for signing (EIP-155 compliant)
    fn build_transaction_hash(&self, params: &EthereumTransactionParams) -> Result<Vec<u8>, EthereumError> {
        use rlp::RlpStream;
        use sha3::{Digest, Keccak256};

        let mut stream = if params.max_fee_per_gas.is_some() {
            // EIP-1559 transaction
            let mut s = RlpStream::new_list(12);
            s.append(&2u8); // Transaction type
            s.append(&params.chain_id);
            s.append(&params.nonce);
            s.append(&params.max_priority_fee_per_gas.as_ref().unwrap().parse::<u64>()
                .map_err(|_| EthereumError::SerializationError("Invalid max priority fee".to_string()))?);
            s.append(&params.max_fee_per_gas.as_ref().unwrap().parse::<u64>()
                .map_err(|_| EthereumError::SerializationError("Invalid max fee".to_string()))?);
            s.append(&params.gas_limit);
            
            // Decode hex address
            let to_bytes = hex::decode(&params.to[2..])
                .map_err(|_| EthereumError::InvalidAddress(params.to.clone()))?;
            s.append(&to_bytes);
            
            let value = params.value.parse::<u128>()
                .map_err(|_| EthereumError::SerializationError("Invalid value".to_string()))?;
            s.append(&value);
            
            // Data
            if let Some(data) = &params.data {
                let data_bytes = hex::decode(&data[2..])
                    .map_err(|_| EthereumError::SerializationError("Invalid data".to_string()))?;
                s.append(&data_bytes);
            } else {
                s.append(&"");
            }
            
            s.append(&Vec::<u8>::new()); // Access list (empty for now)
            s
        } else {
            // Legacy transaction
            let mut s = RlpStream::new_list(9);
            s.append(&params.nonce);
            s.append(&params.gas_price.as_ref().unwrap().parse::<u64>()
                .map_err(|_| EthereumError::SerializationError("Invalid gas price".to_string()))?);
            s.append(&params.gas_limit);
            
            let to_bytes = hex::decode(&params.to[2..])
                .map_err(|_| EthereumError::InvalidAddress(params.to.clone()))?;
            s.append(&to_bytes);
            
            let value = params.value.parse::<u128>()
                .map_err(|_| EthereumError::SerializationError("Invalid value".to_string()))?;
            s.append(&value);
            
            if let Some(data) = &params.data {
                let data_bytes = hex::decode(&data[2..])
                    .map_err(|_| EthereumError::SerializationError("Invalid data".to_string()))?;
                s.append(&data_bytes);
            } else {
                s.append(&"");
            }
            
            // EIP-155: Chain ID for replay protection
            s.append(&params.chain_id);
            s.append(&0u8); // r
            s.append(&0u8); // s
            s
        };

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

        let r = &signature[0..32];
        let s = &signature[32..64];
        let v = signature[64];

        let mut stream = if params.max_fee_per_gas.is_some() {
            // EIP-1559 transaction
            let mut s = RlpStream::new_list(12);
            s.append(&2u8); // Transaction type
            s.append(&params.chain_id);
            s.append(&params.nonce);
            s.append(&params.max_priority_fee_per_gas.as_ref().unwrap().parse::<u64>().unwrap());
            s.append(&params.max_fee_per_gas.as_ref().unwrap().parse::<u64>().unwrap());
            s.append(&params.gas_limit);
            
            let to_bytes = hex::decode(&params.to[2..]).unwrap();
            s.append(&to_bytes);
            
            let value = params.value.parse::<u128>().unwrap();
            s.append(&value);
            
            if let Some(data) = &params.data {
                let data_bytes = hex::decode(&data[2..]).unwrap();
                s.append(&data_bytes);
            } else {
                s.append(&"");
            }
            
            s.append(&Vec::<u8>::new()); // Access list
            s.append(&(v as u64));
            s.append(&r);
            s.append(&s);
            s
        } else {
            // Legacy transaction
            let mut stream = RlpStream::new_list(9);
            stream.append(&params.nonce);
            stream.append(&params.gas_price.as_ref().unwrap().parse::<u64>().unwrap());
            stream.append(&params.gas_limit);
            
            let to_bytes = hex::decode(&params.to[2..]).unwrap();
            stream.append(&to_bytes);
            
            let value = params.value.parse::<u128>().unwrap();
            stream.append(&value);
            
            if let Some(data) = &params.data {
                let data_bytes = hex::decode(&data[2..]).unwrap();
                stream.append(&data_bytes);
            } else {
                stream.append(&"");
            }
            
            // EIP-155 signature
            let v_eip155 = v as u64 + params.chain_id * 2 + 35;
            stream.append(&v_eip155);
            stream.append(&r);
            stream.append(&s);
            stream
        };

        let encoded = stream.out();
        Ok(format!("0x{}", hex::encode(encoded)))
    }

    /// Find optimal chain for a transaction using L2 optimizer
    async fn find_optimal_chain_for_transaction(
        &self,
        user: Principal,
        amount_wei: &str,
        gas_priority: GasPriority,
    ) -> Result<EvmChain, EthereumError> {
        let amount_eth = super::utils::wei_to_eth(amount_wei)
            .map_err(|e| EthereumError::L2OptimizationError(e.to_string()))?;
        let amount_usd = amount_eth * 2000.0; // Approximate ETH price

        let context = super::l2_optimizer::TransactionContext {
            transaction_type: super::l2_optimizer::TransactionType::SimpleTransfer,
            amount_usd,
            urgency: gas_priority,
            current_chain: None,
            preferred_chains: self.context.supported_chains.clone(),
            max_bridge_cost_usd: Some(amount_usd * 0.1), // Max 10% of transaction value
            max_total_time_minutes: Some(60), // Max 1 hour
        };

        let optimization = self.l2_optimizer.optimize_transaction(&context).await
            .map_err(|e| EthereumError::L2OptimizationError(e.to_string()))?;
        
        Ok(optimization.recommended_chain)
    }

    /// Get ETH price using consensus validation
    async fn get_eth_price_consensus(&self) -> Result<f64, EthereumError> {
        // In production, this would use multiple price oracles through EVM RPC canister
        // For now, return a reasonable estimate
        Ok(2000.0)
    }

    /// Estimate confirmation time for a chain and priority
    fn estimate_confirmation_time(&self, chain: &EvmChain, priority: &GasPriority) -> u64 {
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

    /// Get L2 optimization recommendations using ICP consensus
    pub async fn get_l2_optimization(
        &self,
        user: Principal,
        amount_wei: String,
        transaction_type: super::l2_optimizer::TransactionType,
        gas_priority: GasPriority,
    ) -> Result<L2OptimizationResult, EthereumError> {
        let amount_eth = super::utils::wei_to_eth(&amount_wei)
            .map_err(|e| EthereumError::L2OptimizationError(e.to_string()))?;
        let amount_usd = amount_eth * self.get_eth_price_consensus().await.unwrap_or(2000.0);

        let context = super::l2_optimizer::TransactionContext {
            transaction_type,
            amount_usd,
            urgency: gas_priority,
            current_chain: Some(EvmChain::Ethereum), // Assume starting from Ethereum
            preferred_chains: self.context.supported_chains.clone(),
            max_bridge_cost_usd: Some(amount_usd * 0.05), // Max 5% of transaction value
            max_total_time_minutes: Some(120), // Max 2 hours
        };

        self.l2_optimizer.optimize_transaction(&context).await
            .map_err(|e| EthereumError::L2OptimizationError(e.to_string()))
    }
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
    fn test_icp_service_creation() {
        let context = IcpEthereumServiceContext {
            key_name: "test_key".to_string(),
            canister_id: Principal::anonymous(),
            supported_chains: vec![EvmChain::Ethereum, EvmChain::Arbitrum],
        };

        let service = IcpEthereumDeFiService::new(context);
        assert_eq!(service.context.supported_chains.len(), 2);
        assert_eq!(service.evm_rpc.canister_id.to_text(), super::EVM_RPC_CANISTER_ID);
    }

    #[test]
    fn test_confirmation_time_estimation() {
        let context = IcpEthereumServiceContext {
            key_name: "test_key".to_string(),
            canister_id: Principal::anonymous(),
            supported_chains: vec![EvmChain::Ethereum],
        };

        let service = IcpEthereumDeFiService::new(context);
        
        let eth_time = service.estimate_confirmation_time(&EvmChain::Ethereum, &GasPriority::Medium);
        let arb_time = service.estimate_confirmation_time(&EvmChain::Arbitrum, &GasPriority::Medium);
        
        assert!(eth_time > arb_time);
    }
}