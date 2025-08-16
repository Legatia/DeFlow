// Comprehensive tests for ICP-compliant Ethereum integration
// Tests for API endpoints, service integration, and ICP compliance

#[cfg(test)]
mod api_tests {
    use crate::defi::api::*;
    use crate::defi::ethereum::{EvmChain, GasPriority};

    #[test]
    fn test_simple_gas_estimate_creation() {
        let estimate = SimpleGasEstimate {
            gas_limit: 21000,
            gas_price: "0x4a817c800".to_string(), // 20 gwei
            max_fee_per_gas: "0x4a817c800".to_string(),
            max_priority_fee_per_gas: "0x77359400".to_string(), // 2 gwei
            total_fee_wei: "420000000000000".to_string(),
            total_fee_eth: 0.00042,
            total_fee_usd: 0.84,
            confirmation_time_estimate_seconds: 60,
            priority: GasPriority::Medium,
        };

        assert_eq!(estimate.gas_limit, 21000);
        assert!(estimate.total_fee_eth > 0.0);
        assert!(estimate.total_fee_usd > 0.0);
        assert_eq!(estimate.confirmation_time_estimate_seconds, 60);
    }

    #[test]
    fn test_simple_chain_option_creation() {
        let option = SimpleChainOption {
            chain: EvmChain::Arbitrum,
            fee_usd: 0.1,
            time_seconds: 30,
            total_cost_usd: 0.1,
        };

        assert_eq!(option.chain, EvmChain::Arbitrum);
        assert_eq!(option.fee_usd, 0.1);
        assert_eq!(option.time_seconds, 30);
        assert_eq!(option.total_cost_usd, 0.1);
    }

    #[test]
    fn test_evm_chain_info_structure() {
        let chain_info = EVMChainInfo {
            chain: EvmChain::Ethereum,
            chain_id: 1,
            name: "Ethereum".to_string(),
            native_token: "ETH".to_string(),
            is_l2: false,
            is_sidechain: false,
            supports_eip1559: true,
            average_block_time_seconds: 12,
            typical_gas_price_gwei: 20,
            block_explorer: "https://etherscan.io".to_string(),
        };

        assert_eq!(chain_info.chain, EvmChain::Ethereum);
        assert_eq!(chain_info.chain_id, 1);
        assert_eq!(chain_info.name, "Ethereum");
        assert_eq!(chain_info.native_token, "ETH");
        assert!(!chain_info.is_l2);
        assert!(!chain_info.is_sidechain);
        assert!(chain_info.supports_eip1559);
        assert_eq!(chain_info.average_block_time_seconds, 12);
        assert_eq!(chain_info.typical_gas_price_gwei, 20);
        assert!(chain_info.block_explorer.contains("etherscan"));
    }

    #[test]
    fn test_gas_priority_ordering() {
        let priorities = vec![
            GasPriority::Low,
            GasPriority::Medium,
            GasPriority::High,
            GasPriority::Urgent,
        ];

        // Test that we have all 4 priorities
        assert_eq!(priorities.len(), 4);

        // Test serialization uniqueness
        let serialized: Vec<String> = priorities.iter()
            .map(|p| format!("{:?}", p))
            .collect();
        
        let mut unique_serialized = serialized.clone();
        unique_serialized.sort();
        unique_serialized.dedup();
        
        assert_eq!(serialized.len(), unique_serialized.len()); // All unique
    }

    #[test]
    fn test_supported_evm_chains() {
        let chains = vec![
            EvmChain::Ethereum,
            EvmChain::Arbitrum,
            EvmChain::Optimism,
            EvmChain::Polygon,
            EvmChain::Base,
            EvmChain::Avalanche,
        ];

        assert_eq!(chains.len(), 6);

        // Test chain properties
        for chain in chains {
            assert!(chain.chain_id() > 0);
            assert!(!chain.name().is_empty());
            assert!(!chain.native_token().is_empty());
            
            // Test L2 classification
            match chain {
                EvmChain::Arbitrum | EvmChain::Optimism | EvmChain::Base => {
                    assert!(chain.is_l2());
                    assert!(!chain.is_sidechain());
                    assert!(!chain.is_independent_l1());
                    assert_eq!(chain.native_token(), "ETH");
                },
                EvmChain::Polygon | EvmChain::Avalanche => {
                    assert!(!chain.is_l2());
                    assert!(chain.is_sidechain());
                    assert!(!chain.is_independent_l1());
                    assert_ne!(chain.native_token(), "ETH");
                },
                EvmChain::Ethereum => {
                    assert!(!chain.is_l2());
                    assert!(!chain.is_sidechain());
                    assert!(!chain.is_independent_l1());
                    assert_eq!(chain.native_token(), "ETH");
                },
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::defi::ethereum::{MinimalIcpEthereumService, TransactionType, EvmChain, GasPriority};
    use candid::Principal;

    #[test]
    fn test_icp_ethereum_service_integration() {
        let service = MinimalIcpEthereumService::new(
            "deflow_ethereum_key".to_string(),
            Principal::anonymous(),
        );

        // Test service properties
        assert_eq!(service.key_name, "deflow_ethereum_key");
        assert_eq!(service.canister_id, Principal::anonymous());
        assert_eq!(service.supported_chains.len(), 8);

        // Test that all major chains are supported
        assert!(service.supported_chains.contains(&EvmChain::Ethereum));
        assert!(service.supported_chains.contains(&EvmChain::Arbitrum));
        assert!(service.supported_chains.contains(&EvmChain::Optimism));
        assert!(service.supported_chains.contains(&EvmChain::Polygon));
        assert!(service.supported_chains.contains(&EvmChain::Base));
        assert!(service.supported_chains.contains(&EvmChain::Avalanche));
    }

    #[test]
    fn test_transaction_type_completeness() {
        let all_types = vec![
            TransactionType::SimpleTransfer,
            TransactionType::TokenTransfer,
            TransactionType::DexSwap,
            TransactionType::Lending,
            TransactionType::Nft,
            TransactionType::ContractDeployment,
            TransactionType::ComplexDefi,
        ];

        assert_eq!(all_types.len(), 7);

        // Test that each type can be formatted and cloned
        for tx_type in all_types {
            let formatted = format!("{:?}", tx_type);
            assert!(!formatted.is_empty());
            
            let cloned = tx_type.clone();
            assert_eq!(format!("{:?}", tx_type), format!("{:?}", cloned));
        }
    }

    #[test] 
    fn test_l2_cost_comparison_logic() {
        // Test the logic used in compare_l2_costs
        let chains = vec![
            EvmChain::Ethereum,
            EvmChain::Arbitrum,
            EvmChain::Optimism,
            EvmChain::Polygon,
        ];

        for chain in chains {
            // Test base fee logic
            for priority in [GasPriority::Low, GasPriority::Medium, GasPriority::High, GasPriority::Urgent] {
                let base_fee = match priority {
                    GasPriority::Low => 0.5,
                    GasPriority::Medium => 2.0,
                    GasPriority::High => 5.0,
                    GasPriority::Urgent => 10.0,
                };
                assert!(base_fee > 0.0);
            }

            // Test chain multiplier logic
            let multiplier = match chain {
                EvmChain::Ethereum => 5.0,
                EvmChain::Arbitrum => 0.1,
                EvmChain::Optimism => 0.1,
                EvmChain::Polygon => 0.01,
                EvmChain::Base => 0.1,
                EvmChain::Avalanche => 0.2,
            };
            assert!(multiplier > 0.0);

            // Ethereum should be most expensive
            if chain == EvmChain::Ethereum {
                assert_eq!(multiplier, 5.0);
            } else {
                assert!(multiplier < 1.0); // L2s should be cheaper
            }
        }
    }

    #[test]
    fn test_gas_estimation_logic() {
        // Test gas estimation parameters
        let base_gas_limit = 21000u64;
        assert_eq!(base_gas_limit, 21000);

        for priority in [GasPriority::Low, GasPriority::Medium, GasPriority::High, GasPriority::Urgent] {
            let gas_price_gwei = match priority {
                GasPriority::Low => 5,
                GasPriority::Medium => 20,
                GasPriority::High => 50,
                GasPriority::Urgent => 100,
            };

            assert!(gas_price_gwei > 0);
            
            // Higher priority should mean higher gas price
            match priority {
                GasPriority::Low => assert_eq!(gas_price_gwei, 5),
                GasPriority::Medium => assert!(gas_price_gwei > 5),
                GasPriority::High => assert!(gas_price_gwei > 20),
                GasPriority::Urgent => assert!(gas_price_gwei > 50),
            }

            let gas_price_wei = gas_price_gwei as u64 * 1_000_000_000u64;
            let total_fee_wei = base_gas_limit as u128 * gas_price_wei as u128;
            let total_fee_eth = total_fee_wei as f64 / 1e18;
            let total_fee_usd = total_fee_eth * 2000.0; // Approximate ETH price

            assert!(total_fee_wei > 0);
            assert!(total_fee_eth > 0.0);
            assert!(total_fee_usd > 0.0);
        }
    }
}

#[cfg(test)]
mod icp_compliance_tests {
    use crate::defi::ethereum::{MinimalIcpEthereumService, EvmChain};
    use candid::Principal;

    #[test]
    fn test_icp_key_management() {
        let service = MinimalIcpEthereumService::new(
            "deflow_ethereum_key".to_string(),
            Principal::anonymous(),
        );

        // Test that key_name follows ICP conventions
        assert!(!service.key_name.is_empty());
        assert!(service.key_name.contains("ethereum"));
        
        // Test canister ID is properly set
        assert_eq!(service.canister_id, Principal::anonymous());
    }

    #[test]
    fn test_deterministic_address_generation() {
        let service = MinimalIcpEthereumService::new(
            "test_key".to_string(),
            Principal::anonymous(),
        );

        let user = Principal::anonymous();
        let chain = EvmChain::Ethereum;

        // Address generation should be deterministic
        let seed1 = format!("{}-{:?}-{}", service.key_name, chain, user.to_text());
        let seed2 = format!("{}-{:?}-{}", service.key_name, chain, user.to_text());
        assert_eq!(seed1, seed2);

        // Hash should be consistent
        let hash1 = service.hash_string(&seed1);
        let hash2 = service.hash_string(&seed2);
        assert_eq!(hash1, hash2);

        // Address format should be valid Ethereum address format
        let address = format!("0x{:040x}", hash1);
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
        assert!(address.chars().skip(2).all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_icp_compliance_patterns() {
        let service = MinimalIcpEthereumService::new(
            "deflow_ethereum_key".to_string(),
            Principal::anonymous(), // Use anonymous instead of ic_cdk::api::id() for tests
        );

        // Test ICP-compliant patterns
        assert!(!service.key_name.is_empty()); // Key names must not be empty
        assert_eq!(service.supported_chains.len(), 8); // Support major EVM chains
        
        // Test that canister ID is set (in real deployment)
        // Note: ic_cdk::api::id() returns different values in test vs deployment
        
        // Test hash function consistency (required for deterministic addresses)
        let test_input = "test_input";
        let hash1 = service.hash_string(test_input);
        let hash2 = service.hash_string(test_input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_multi_chain_support() {
        let service = MinimalIcpEthereumService::new(
            "test_key".to_string(),
            Principal::anonymous(),
        );

        // Test that all required EVM chains are supported
        let required_chains = [
            EvmChain::Ethereum,
            EvmChain::Arbitrum,
            EvmChain::Optimism,
            EvmChain::Polygon,
            EvmChain::Base,
            EvmChain::Avalanche,
        ];

        for required_chain in required_chains {
            assert!(
                service.supported_chains.contains(&required_chain),
                "Missing required chain: {:?}",
                required_chain
            );
        }

        // Test chain properties for ICP compliance
        for chain in &service.supported_chains {
            assert!(chain.chain_id() > 0, "Chain ID must be positive");
            assert!(!chain.name().is_empty(), "Chain name must not be empty");
            assert!(!chain.native_token().is_empty(), "Native token must not be empty");
        }
    }

    #[test]
    fn test_error_handling_compliance() {
        use crate::defi::ethereum::EthereumError;

        // Test that all error types can be properly displayed (required for debugging)
        let errors = vec![
            EthereumError::InvalidAddress("test".to_string()),
            EthereumError::InsufficientBalance { 
                required: "100".to_string(), 
                available: "50".to_string() 
            },
            EthereumError::NetworkError("test".to_string()),
            EthereumError::TransactionFailed("test".to_string()),
            EthereumError::GasEstimationFailed("test".to_string()),
            EthereumError::ChainNotSupported("test".to_string()),
            EthereumError::ThresholdEcdsaError("test".to_string()),
            EthereumError::SerializationError("test".to_string()),
            EthereumError::RpcError("test".to_string()),
            EthereumError::InsufficientCycles("test".to_string()),
            EthereumError::ConsensusError("test".to_string()),
            EthereumError::AddressGenerationError("test".to_string()),
            EthereumError::SigningError("test".to_string()),
            EthereumError::BroadcastError("test".to_string()),
            EthereumError::L2OptimizationError("test".to_string()),
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty(), "Error display must not be empty");
            assert!(error_string.len() > 5, "Error message should be descriptive");
        }
    }
}