// Integration tests for deposit management system
// Tests the complete flow from deposit address generation to strategy execution

#[cfg(test)]
mod integration_tests {
    use super::super::deposit_manager::*;
    use super::super::yield_farming::*;
    use super::super::automated_strategies::*;
    use super::super::types::*;
    use candid::Principal;
    use ic_cdk::api::time;
    use std::collections::HashMap;

    // Mock external service responses for testing
    struct MockBlockchainService {
        bitcoin_addresses: HashMap<Principal, String>,
        ethereum_addresses: HashMap<Principal, String>,
        solana_addresses: HashMap<Principal, String>,
        mock_deposits: HashMap<String, Vec<DepositTransaction>>,
    }

    impl MockBlockchainService {
        fn new() -> Self {
            Self {
                bitcoin_addresses: HashMap::new(),
                ethereum_addresses: HashMap::new(),
                solana_addresses: HashMap::new(),
                mock_deposits: HashMap::new(),
            }
        }

        async fn generate_bitcoin_address(&mut self, user: Principal) -> Result<String, String> {
            let address = format!("bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0w{:x}", user.as_slice()[0]);
            self.bitcoin_addresses.insert(user, address.clone());
            Ok(address)
        }

        async fn generate_ethereum_address(&mut self, user: Principal) -> Result<String, String> {
            let address = format!("0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE{:02x}{:02x}",
                                user.as_slice()[0], user.as_slice()[1]);
            self.ethereum_addresses.insert(user, address.clone());
            Ok(address)
        }

        async fn generate_solana_address(&mut self, user: Principal) -> Result<String, String> {
            let address = format!("DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG{}CNSKK",
                                char::from(65 + (user.as_slice()[0] % 26)));
            self.solana_addresses.insert(user, address.clone());
            Ok(address)
        }

        fn add_mock_deposit(&mut self, address: String, deposit: DepositTransaction) {
            self.mock_deposits.entry(address).or_insert_with(Vec::new).push(deposit);
        }

        async fn scan_deposits(&self, address: &str) -> Vec<DepositTransaction> {
            self.mock_deposits.get(address).cloned().unwrap_or_default()
        }
    }

    fn create_test_user() -> Principal {
        Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
    }

    fn create_test_user_2() -> Principal {
        Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap()
    }

    #[tokio::test]
    async fn test_complete_user_onboarding_flow() {
        let mut deposit_manager = DepositManager::new();
        let mut blockchain_service = MockBlockchainService::new();
        let user = create_test_user();

        // Step 1: User requests deposit addresses for multiple chains
        let btc_address = blockchain_service.generate_bitcoin_address(user).await.unwrap();
        let eth_address = blockchain_service.generate_ethereum_address(user).await.unwrap();
        let sol_address = blockchain_service.generate_solana_address(user).await.unwrap();

        // Step 2: Register addresses with deposit manager
        deposit_manager.register_deposit_address(user, btc_address.clone(), ChainId::Bitcoin).await.unwrap();
        deposit_manager.register_deposit_address(user, eth_address.clone(), ChainId::Ethereum).await.unwrap();
        deposit_manager.register_deposit_address(user, sol_address.clone(), ChainId::Solana).await.unwrap();

        // Step 3: Verify user portfolio is set up correctly
        let portfolio = deposit_manager.get_user_portfolio(&user);
        assert!(portfolio.is_some());

        let portfolio = portfolio.unwrap();
        assert_eq!(portfolio.user, user);
        assert_eq!(portfolio.deposit_addresses.len(), 3);
        assert_eq!(portfolio.total_value_usd, 0.0);
        assert_eq!(portfolio.available_for_strategies, 0.0);

        // Verify addresses are correct
        let btc_addr = portfolio.deposit_addresses.iter().find(|a| a.chain == ChainId::Bitcoin).unwrap();
        let eth_addr = portfolio.deposit_addresses.iter().find(|a| a.chain == ChainId::Ethereum).unwrap();
        let sol_addr = portfolio.deposit_addresses.iter().find(|a| a.chain == ChainId::Solana).unwrap();

        assert_eq!(btc_addr.address, btc_address);
        assert_eq!(btc_addr.native_token, "BTC");
        assert_eq!(eth_addr.address, eth_address);
        assert_eq!(eth_addr.native_token, "ETH");
        assert_eq!(sol_addr.address, sol_address);
        assert_eq!(sol_addr.native_token, "SOL");

        // Step 4: Verify monitoring is set up
        assert_eq!(deposit_manager.deposit_monitors.len(), 3);
        assert!(deposit_manager.deposit_monitors.contains_key(&btc_address));
        assert!(deposit_manager.deposit_monitors.contains_key(&eth_address));
        assert!(deposit_manager.deposit_monitors.contains_key(&sol_address));
    }

    #[tokio::test]
    async fn test_deposit_detection_and_portfolio_update_flow() {
        let mut deposit_manager = DepositManager::new();
        let mut blockchain_service = MockBlockchainService::new();
        let user = create_test_user();

        // Set up deposit addresses
        let eth_address = blockchain_service.generate_ethereum_address(user).await.unwrap();
        deposit_manager.register_deposit_address(user, eth_address.clone(), ChainId::Ethereum).await.unwrap();

        // Simulate deposits
        let deposit1 = DepositTransaction {
            tx_hash: "0x123456789abcdef1".to_string(),
            amount: 2.5,
            amount_usd: 6250.0,
            token_symbol: "ETH".to_string(),
            block_height: 18000000,
            timestamp: time(),
            confirmations: 12,
        };

        let deposit2 = DepositTransaction {
            tx_hash: "0x123456789abcdef2".to_string(),
            amount: 1.0,
            amount_usd: 2500.0,
            token_symbol: "ETH".to_string(),
            block_height: 18000100,
            timestamp: time(),
            confirmations: 6,
        };

        blockchain_service.add_mock_deposit(eth_address.clone(), deposit1.clone());
        blockchain_service.add_mock_deposit(eth_address.clone(), deposit2.clone());

        // Simulate deposit scanning (in real system this would be automatic)
        let detected_deposits = blockchain_service.scan_deposits(&eth_address).await;
        assert_eq!(detected_deposits.len(), 2);

        // Update portfolio with detected deposits
        for deposit in detected_deposits {
            if let Some(portfolio) = deposit_manager.user_deposits.get_mut(&user) {
                if let Some(deposit_addr) = portfolio.deposit_addresses.iter_mut()
                    .find(|addr| addr.address == eth_address) {

                    deposit_addr.deposits.push(deposit.clone());
                    deposit_addr.balance += deposit.amount;
                    deposit_addr.balance_usd += deposit.amount_usd;

                    portfolio.total_value_usd += deposit.amount_usd;
                    portfolio.available_for_strategies += deposit.amount_usd;
                }
            }
        }

        // Verify portfolio was updated correctly
        let portfolio = deposit_manager.get_user_portfolio(&user).unwrap();
        assert_eq!(portfolio.total_value_usd, 8750.0);
        assert_eq!(portfolio.available_for_strategies, 8750.0);
        assert_eq!(portfolio.locked_in_strategies, 0.0);

        let eth_addr = portfolio.deposit_addresses.iter().find(|a| a.chain == ChainId::Ethereum).unwrap();
        assert_eq!(eth_addr.balance, 3.5);
        assert_eq!(eth_addr.balance_usd, 8750.0);
        assert_eq!(eth_addr.deposits.len(), 2);
    }

    #[tokio::test]
    async fn test_strategy_allocation_and_execution_flow() {
        let mut deposit_manager = DepositManager::new();
        let mut blockchain_service = MockBlockchainService::new();
        let user = create_test_user();

        // Set up user with deposits
        let eth_address = blockchain_service.generate_ethereum_address(user).await.unwrap();
        deposit_manager.register_deposit_address(user, eth_address.clone(), ChainId::Ethereum).await.unwrap();

        // Add initial deposit
        if let Some(portfolio) = deposit_manager.user_deposits.get_mut(&user) {
            portfolio.total_value_usd = 10000.0;
            portfolio.available_for_strategies = 10000.0;
            portfolio.deposit_addresses[0].balance = 4.0;
            portfolio.deposit_addresses[0].balance_usd = 10000.0;
        }

        // Create strategy allocation
        let strategy_config = StrategyConfig {
            strategy_id: "yield_farming_001".to_string(),
            strategy_type: StrategyType::YieldFarming,
            target_chain: ChainId::Ethereum,
            target_protocols: vec![DeFiProtocol::Aave],
            capital_allocation_usd: 5000.0,
            risk_parameters: RiskParameters {
                max_slippage: 0.02,
                max_gas_price_gwei: 100,
                stop_loss_threshold: -0.05,
                take_profit_threshold: 0.20,
            },
            execution_parameters: ExecutionParameters {
                frequency: ExecutionFrequency::Daily,
                rebalance_threshold: 0.05,
                compound_rewards: true,
                reinvest_threshold: 100.0,
            },
            constraints: StrategyConstraints {
                max_position_size_usd: 10000.0,
                max_daily_trades: 10,
                allowed_hours: vec![9, 10, 11, 14, 15, 16],
                blacklisted_tokens: vec![],
            },
            active: true,
            created_at: time(),
            updated_at: time(),
        };

        // Test allocation
        let allocation_result = deposit_manager.allocate_to_strategy(
            user,
            strategy_config.clone(),
            5000.0,
            Some(eth_address.clone())
        ).await;

        assert!(allocation_result.is_ok());
        let allocation_id = allocation_result.unwrap();
        assert!(!allocation_id.is_empty());

        // Verify pending allocation was created
        assert!(deposit_manager.pending_allocations.contains_key(&allocation_id));
        let pending = deposit_manager.pending_allocations.get(&allocation_id).unwrap();
        assert_eq!(pending.user, user);
        assert_eq!(pending.amount_usd, 5000.0);
        assert_eq!(pending.source_address, eth_address);
        assert_eq!(pending.target_chain, ChainId::Ethereum);

        // Simulate strategy execution completion
        let strategy_allocation = StrategyAllocation {
            strategy_id: strategy_config.strategy_id.clone(),
            protocol: DeFiProtocol::Aave,
            chain: ChainId::Ethereum,
            allocated_amount: 2.0, // 2 ETH
            allocated_amount_usd: 5000.0,
            current_value_usd: 5000.0, // Initially same as allocated
            pnl: 0.0,
            pnl_percentage: 0.0,
            status: AllocationStatus::Active,
            created_at: time(),
            last_updated: time(),
        };

        // Update portfolio with allocation
        if let Some(portfolio) = deposit_manager.user_deposits.get_mut(&user) {
            portfolio.available_for_strategies -= 5000.0;
            portfolio.locked_in_strategies += 5000.0;
            portfolio.deposit_addresses[0].strategy_allocations.push(strategy_allocation);
        }

        // Verify allocation is reflected in portfolio
        let portfolio = deposit_manager.get_user_portfolio(&user).unwrap();
        assert_eq!(portfolio.total_value_usd, 10000.0);
        assert_eq!(portfolio.available_for_strategies, 5000.0);
        assert_eq!(portfolio.locked_in_strategies, 5000.0);

        let eth_addr = portfolio.deposit_addresses.iter().find(|a| a.chain == ChainId::Ethereum).unwrap();
        assert_eq!(eth_addr.strategy_allocations.len(), 1);
        assert_eq!(eth_addr.strategy_allocations[0].allocated_amount_usd, 5000.0);
        assert_eq!(eth_addr.strategy_allocations[0].status, AllocationStatus::Active);
    }

    #[tokio::test]
    async fn test_auto_allocation_rule_execution() {
        let mut deposit_manager = DepositManager::new();
        let mut blockchain_service = MockBlockchainService::new();
        let user = create_test_user();

        // Set up deposit address
        let eth_address = blockchain_service.generate_ethereum_address(user).await.unwrap();
        deposit_manager.register_deposit_address(user, eth_address.clone(), ChainId::Ethereum).await.unwrap();

        // Set up auto-allocation rules
        let auto_rules = vec![
            AutoAllocationRule {
                strategy_type: "conservative_yield".to_string(),
                min_deposit_amount: 1000.0,
                allocation_percentage: 40.0,
                protocol_preference: vec![DeFiProtocol::Aave, DeFiProtocol::Compound],
                risk_tolerance: RiskTolerance::Conservative,
            },
            AutoAllocationRule {
                strategy_type: "moderate_yield".to_string(),
                min_deposit_amount: 2000.0,
                allocation_percentage: 30.0,
                protocol_preference: vec![DeFiProtocol::Uniswap(UniswapVersion::V3)],
                risk_tolerance: RiskTolerance::Moderate,
            }
        ];

        let setup_result = deposit_manager.setup_auto_allocation(user, eth_address.clone(), auto_rules);
        assert!(setup_result.is_ok());

        // Verify auto-allocation is enabled
        let monitor = deposit_manager.deposit_monitors.get(&eth_address).unwrap();
        assert!(monitor.auto_allocate_enabled);
        assert_eq!(monitor.auto_allocation_strategies.len(), 2);

        // Test rule triggering logic
        let test_deposits = vec![
            (500.0, 0), // Below minimum for any rule
            (1500.0, 1), // Triggers first rule only
            (3000.0, 2), // Triggers both rules
        ];

        for (deposit_amount, expected_triggers) in test_deposits {
            let triggered_rules: Vec<&AutoAllocationRule> = monitor.auto_allocation_strategies.iter()
                .filter(|rule| deposit_amount >= rule.min_deposit_amount)
                .collect();

            assert_eq!(triggered_rules.len(), expected_triggers,
                      "Deposit of {} should trigger {} rules", deposit_amount, expected_triggers);

            if expected_triggers > 0 {
                let total_allocation_percentage: f64 = triggered_rules.iter()
                    .map(|rule| rule.allocation_percentage)
                    .sum();

                let total_allocated = deposit_amount * (total_allocation_percentage / 100.0);

                if expected_triggers == 1 {
                    assert_eq!(total_allocated, deposit_amount * 0.40);
                } else if expected_triggers == 2 {
                    assert_eq!(total_allocated, deposit_amount * 0.70); // 40% + 30%
                }
            }
        }
    }

    #[tokio::test]
    async fn test_multi_user_isolation() {
        let mut deposit_manager = DepositManager::new();
        let mut blockchain_service = MockBlockchainService::new();
        let user1 = create_test_user();
        let user2 = create_test_user_2();

        // Set up addresses for both users
        let user1_eth = blockchain_service.generate_ethereum_address(user1).await.unwrap();
        let user2_eth = blockchain_service.generate_ethereum_address(user2).await.unwrap();

        deposit_manager.register_deposit_address(user1, user1_eth.clone(), ChainId::Ethereum).await.unwrap();
        deposit_manager.register_deposit_address(user2, user2_eth.clone(), ChainId::Ethereum).await.unwrap();

        // Add different balances
        if let Some(portfolio1) = deposit_manager.user_deposits.get_mut(&user1) {
            portfolio1.total_value_usd = 5000.0;
            portfolio1.available_for_strategies = 5000.0;
        }

        if let Some(portfolio2) = deposit_manager.user_deposits.get_mut(&user2) {
            portfolio2.total_value_usd = 15000.0;
            portfolio2.available_for_strategies = 15000.0;
        }

        // Verify isolation
        let portfolio1 = deposit_manager.get_user_portfolio(&user1).unwrap();
        let portfolio2 = deposit_manager.get_user_portfolio(&user2).unwrap();

        assert_eq!(portfolio1.user, user1);
        assert_eq!(portfolio2.user, user2);
        assert_eq!(portfolio1.total_value_usd, 5000.0);
        assert_eq!(portfolio2.total_value_usd, 15000.0);

        // Test that user1 cannot access user2's funds
        let strategy_config = create_test_strategy_config();
        let allocation_result = deposit_manager.allocate_to_strategy(
            user1,
            strategy_config,
            10000.0, // More than user1 has
            None
        ).await;

        assert!(allocation_result.is_err());
        assert!(allocation_result.unwrap_err().contains("Insufficient funds"));

        // Verify user2 can access their own funds
        let strategy_config2 = create_test_strategy_config();
        let allocation_result2 = deposit_manager.allocate_to_strategy(
            user2,
            strategy_config2,
            10000.0, // Within user2's balance
            None
        ).await;

        assert!(allocation_result2.is_ok());
    }

    #[tokio::test]
    async fn test_cross_chain_fund_management() {
        let mut deposit_manager = DepositManager::new();
        let mut blockchain_service = MockBlockchainService::new();
        let user = create_test_user();

        // Set up addresses on multiple chains
        let btc_address = blockchain_service.generate_bitcoin_address(user).await.unwrap();
        let eth_address = blockchain_service.generate_ethereum_address(user).await.unwrap();
        let sol_address = blockchain_service.generate_solana_address(user).await.unwrap();

        deposit_manager.register_deposit_address(user, btc_address.clone(), ChainId::Bitcoin).await.unwrap();
        deposit_manager.register_deposit_address(user, eth_address.clone(), ChainId::Ethereum).await.unwrap();
        deposit_manager.register_deposit_address(user, sol_address.clone(), ChainId::Solana).await.unwrap();

        // Add balances to different chains
        if let Some(portfolio) = deposit_manager.user_deposits.get_mut(&user) {
            portfolio.total_value_usd = 20000.0;
            portfolio.available_for_strategies = 20000.0;

            // Bitcoin: 0.2 BTC at $50,000 = $10,000
            portfolio.deposit_addresses[0].balance_usd = 10000.0;
            // Ethereum: 4 ETH at $2,500 = $10,000
            portfolio.deposit_addresses[1].balance_usd = 6000.0;
            // Solana: 40 SOL at $100 = $4,000
            portfolio.deposit_addresses[2].balance_usd = 4000.0;
        }

        // Test same-chain allocation preference
        let eth_strategy = StrategyConfig {
            strategy_id: "eth_yield".to_string(),
            strategy_type: StrategyType::YieldFarming,
            target_chain: ChainId::Ethereum,
            target_protocols: vec![DeFiProtocol::Aave],
            capital_allocation_usd: 5000.0,
            risk_parameters: RiskParameters {
                max_slippage: 0.02,
                max_gas_price_gwei: 100,
                stop_loss_threshold: -0.05,
                take_profit_threshold: 0.20,
            },
            execution_parameters: ExecutionParameters {
                frequency: ExecutionFrequency::Daily,
                rebalance_threshold: 0.05,
                compound_rewards: true,
                reinvest_threshold: 100.0,
            },
            constraints: StrategyConstraints {
                max_position_size_usd: 10000.0,
                max_daily_trades: 10,
                allowed_hours: vec![9, 10, 11, 14, 15, 16],
                blacklisted_tokens: vec![],
            },
            active: true,
            created_at: time(),
            updated_at: time(),
        };

        // Should prefer Ethereum address for Ethereum strategy
        let best_source = deposit_manager.find_best_source_address(&user, &ChainId::Ethereum, 5000.0);
        assert!(best_source.is_ok());
        assert_eq!(best_source.unwrap(), eth_address);

        // Test cross-chain fallback when same-chain insufficient
        let large_strategy = StrategyConfig {
            strategy_id: "large_yield".to_string(),
            strategy_type: StrategyType::YieldFarming,
            target_chain: ChainId::Ethereum,
            target_protocols: vec![DeFiProtocol::Aave],
            capital_allocation_usd: 8000.0, // More than ETH balance alone
            risk_parameters: RiskParameters {
                max_slippage: 0.02,
                max_gas_price_gwei: 100,
                stop_loss_threshold: -0.05,
                take_profit_threshold: 0.20,
            },
            execution_parameters: ExecutionParameters {
                frequency: ExecutionFrequency::Daily,
                rebalance_threshold: 0.05,
                compound_rewards: true,
                reinvest_threshold: 100.0,
            },
            constraints: StrategyConstraints {
                max_position_size_usd: 15000.0,
                max_daily_trades: 10,
                allowed_hours: vec![9, 10, 11, 14, 15, 16],
                blacklisted_tokens: vec![],
            },
            active: true,
            created_at: time(),
            updated_at: time(),
        };

        // Should fall back to Bitcoin address (highest balance)
        let cross_chain_source = deposit_manager.find_best_source_address(&user, &ChainId::Ethereum, 8000.0);
        assert!(cross_chain_source.is_ok());
        assert_eq!(cross_chain_source.unwrap(), btc_address);
    }

    // Helper function to create test strategy config
    fn create_test_strategy_config() -> StrategyConfig {
        StrategyConfig {
            strategy_id: "test_strategy".to_string(),
            strategy_type: StrategyType::YieldFarming,
            target_chain: ChainId::Ethereum,
            target_protocols: vec![DeFiProtocol::Aave],
            capital_allocation_usd: 1000.0,
            risk_parameters: RiskParameters {
                max_slippage: 0.02,
                max_gas_price_gwei: 100,
                stop_loss_threshold: -0.05,
                take_profit_threshold: 0.20,
            },
            execution_parameters: ExecutionParameters {
                frequency: ExecutionFrequency::Daily,
                rebalance_threshold: 0.05,
                compound_rewards: true,
                reinvest_threshold: 100.0,
            },
            constraints: StrategyConstraints {
                max_position_size_usd: 10000.0,
                max_daily_trades: 10,
                allowed_hours: vec![9, 10, 11, 14, 15, 16],
                blacklisted_tokens: vec![],
            },
            active: true,
            created_at: time(),
            updated_at: time(),
        }
    }
}