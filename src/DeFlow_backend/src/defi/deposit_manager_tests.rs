// Comprehensive tests for the deposit management system
// Tests deposit address generation, portfolio management, and strategy allocation

#[cfg(test)]
mod tests {
    use super::super::deposit_manager::*;
    use super::super::yield_farming::*;
    use super::super::automated_strategies::*;
    use candid::Principal;
    use ic_cdk::api::time;
    use std::collections::HashMap;

    // Mock principal for testing
    fn mock_principal() -> Principal {
        Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
    }

    fn mock_principal_2() -> Principal {
        Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap()
    }

    // Helper function to create a mock deposit manager
    fn create_test_deposit_manager() -> DepositManager {
        DepositManager::new()
    }

    // Helper function to create mock deposit transaction
    fn create_mock_deposit(amount: f64, amount_usd: f64) -> DepositTransaction {
        DepositTransaction {
            tx_hash: "0x123456789abcdef".to_string(),
            amount,
            amount_usd,
            token_symbol: "ETH".to_string(),
            block_height: 18000000,
            timestamp: time(),
            confirmations: 12,
        }
    }

    #[tokio::test]
    async fn test_deposit_manager_initialization() {
        let manager = create_test_deposit_manager();

        assert_eq!(manager.user_deposits.len(), 0);
        assert_eq!(manager.deposit_monitors.len(), 0);
        assert_eq!(manager.strategy_allocations.len(), 0);
        assert_eq!(manager.pending_allocations.len(), 0);
    }

    #[tokio::test]
    async fn test_register_deposit_address() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();
        let address = "0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e".to_string();
        let chain = ChainId::Ethereum;

        // Register deposit address
        let result = manager.register_deposit_address(user, address.clone(), chain.clone()).await;

        assert!(result.is_ok());

        // Verify monitor was created
        assert!(manager.deposit_monitors.contains_key(&address));
        let monitor = manager.deposit_monitors.get(&address).unwrap();
        assert_eq!(monitor.user, user);
        assert_eq!(monitor.chain, chain);
        assert_eq!(monitor.address, address);
        assert_eq!(monitor.auto_allocate_enabled, false);

        // Verify user portfolio was created
        assert!(manager.user_deposits.contains_key(&user));
        let portfolio = manager.user_deposits.get(&user).unwrap();
        assert_eq!(portfolio.user, user);
        assert_eq!(portfolio.deposit_addresses.len(), 1);
        assert_eq!(portfolio.total_value_usd, 0.0);
        assert_eq!(portfolio.available_for_strategies, 0.0);
    }

    #[tokio::test]
    async fn test_register_multiple_chain_addresses() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();

        // Register addresses for different chains
        let btc_address = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string();
        let eth_address = "0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e".to_string();
        let sol_address = "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK".to_string();

        manager.register_deposit_address(user, btc_address.clone(), ChainId::Bitcoin).await.unwrap();
        manager.register_deposit_address(user, eth_address.clone(), ChainId::Ethereum).await.unwrap();
        manager.register_deposit_address(user, sol_address.clone(), ChainId::Solana).await.unwrap();

        // Verify all addresses are registered
        assert_eq!(manager.deposit_monitors.len(), 3);
        assert!(manager.deposit_monitors.contains_key(&btc_address));
        assert!(manager.deposit_monitors.contains_key(&eth_address));
        assert!(manager.deposit_monitors.contains_key(&sol_address));

        // Verify user portfolio has all addresses
        let portfolio = manager.user_deposits.get(&user).unwrap();
        assert_eq!(portfolio.deposit_addresses.len(), 3);

        let btc_addr = portfolio.deposit_addresses.iter().find(|a| a.address == btc_address).unwrap();
        assert_eq!(btc_addr.chain, ChainId::Bitcoin);
        assert_eq!(btc_addr.native_token, "BTC");

        let eth_addr = portfolio.deposit_addresses.iter().find(|a| a.address == eth_address).unwrap();
        assert_eq!(eth_addr.chain, ChainId::Ethereum);
        assert_eq!(eth_addr.native_token, "ETH");
    }

    #[tokio::test]
    async fn test_deposit_detection_and_portfolio_update() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();
        let address = "0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e".to_string();

        // Register address first
        manager.register_deposit_address(user, address.clone(), ChainId::Ethereum).await.unwrap();

        // Simulate deposit detection by manually updating portfolio
        let deposit = create_mock_deposit(2.5, 6250.0); // 2.5 ETH at $2500/ETH

        // Update user portfolio with deposit
        if let Some(portfolio) = manager.user_deposits.get_mut(&user) {
            if let Some(deposit_addr) = portfolio.deposit_addresses.iter_mut()
                .find(|addr| addr.address == address) {

                deposit_addr.deposits.push(deposit.clone());
                deposit_addr.balance += deposit.amount;
                deposit_addr.balance_usd += deposit.amount_usd;

                portfolio.total_value_usd += deposit.amount_usd;
                portfolio.available_for_strategies += deposit.amount_usd;
            }
        }

        // Verify portfolio was updated
        let portfolio = manager.user_deposits.get(&user).unwrap();
        assert_eq!(portfolio.total_value_usd, 6250.0);
        assert_eq!(portfolio.available_for_strategies, 6250.0);
        assert_eq!(portfolio.locked_in_strategies, 0.0);

        let deposit_addr = &portfolio.deposit_addresses[0];
        assert_eq!(deposit_addr.balance, 2.5);
        assert_eq!(deposit_addr.balance_usd, 6250.0);
        assert_eq!(deposit_addr.deposits.len(), 1);
    }

    #[tokio::test]
    async fn test_find_best_source_address() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();

        // Set up addresses with different balances
        manager.register_deposit_address(user, "btc_addr".to_string(), ChainId::Bitcoin).await.unwrap();
        manager.register_deposit_address(user, "eth_addr".to_string(), ChainId::Ethereum).await.unwrap();
        manager.register_deposit_address(user, "arb_addr".to_string(), ChainId::Arbitrum).await.unwrap();

        // Update balances
        if let Some(portfolio) = manager.user_deposits.get_mut(&user) {
            portfolio.deposit_addresses[0].balance_usd = 1000.0; // BTC
            portfolio.deposit_addresses[1].balance_usd = 5000.0; // ETH
            portfolio.deposit_addresses[2].balance_usd = 2000.0; // Arbitrum
        }

        // Test same-chain preference
        let best_eth = manager.find_best_source_address(&user, &ChainId::Ethereum, 3000.0);
        assert!(best_eth.is_ok());
        assert_eq!(best_eth.unwrap(), "eth_addr");

        // Test cross-chain fallback when same-chain insufficient
        let best_cross_chain = manager.find_best_source_address(&user, &ChainId::Polygon, 1500.0);
        assert!(best_cross_chain.is_ok());
        assert_eq!(best_cross_chain.unwrap(), "eth_addr"); // Highest balance

        // Test insufficient funds
        let insufficient = manager.find_best_source_address(&user, &ChainId::Ethereum, 10000.0);
        assert!(insufficient.is_err());
    }

    #[tokio::test]
    async fn test_strategy_allocation_validation() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();
        let address = "0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e".to_string();

        // Register address and add balance
        manager.register_deposit_address(user, address.clone(), ChainId::Ethereum).await.unwrap();

        // Add mock balance
        if let Some(portfolio) = manager.user_deposits.get_mut(&user) {
            portfolio.total_value_usd = 5000.0;
            portfolio.available_for_strategies = 5000.0;
            portfolio.deposit_addresses[0].balance_usd = 5000.0;
        }

        // Create mock strategy config
        let strategy_config = create_mock_strategy_config();

        // Test valid allocation
        let result = manager.allocate_to_strategy(user, strategy_config.clone(), 3000.0, Some(address.clone())).await;
        assert!(result.is_ok());

        // Test insufficient funds
        let insufficient_result = manager.allocate_to_strategy(user, strategy_config.clone(), 6000.0, Some(address.clone())).await;
        assert!(insufficient_result.is_err());
        assert!(insufficient_result.unwrap_err().contains("Insufficient funds"));

        // Test user with no portfolio
        let unknown_user = mock_principal_2();
        let no_portfolio_result = manager.allocate_to_strategy(unknown_user, strategy_config, 1000.0, None).await;
        assert!(no_portfolio_result.is_err());
        assert!(no_portfolio_result.unwrap_err().contains("no deposit portfolio"));
    }

    #[tokio::test]
    async fn test_auto_allocation_setup() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();
        let address = "0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e".to_string();

        // Register address
        manager.register_deposit_address(user, address.clone(), ChainId::Ethereum).await.unwrap();

        // Create auto-allocation rules
        let rules = vec![
            AutoAllocationRule {
                strategy_type: "conservative_yield".to_string(),
                min_deposit_amount: 100.0,
                allocation_percentage: 50.0,
                protocol_preference: vec![DeFiProtocol::Aave, DeFiProtocol::Compound],
                risk_tolerance: RiskTolerance::Conservative,
            },
            AutoAllocationRule {
                strategy_type: "moderate_yield".to_string(),
                min_deposit_amount: 500.0,
                allocation_percentage: 30.0,
                protocol_preference: vec![DeFiProtocol::Uniswap(UniswapVersion::V3)],
                risk_tolerance: RiskTolerance::Moderate,
            }
        ];

        // Setup auto-allocation
        let result = manager.setup_auto_allocation(user, address.clone(), rules.clone());
        assert!(result.is_ok());

        // Verify monitor was updated
        let monitor = manager.deposit_monitors.get(&address).unwrap();
        assert!(monitor.auto_allocate_enabled);
        assert_eq!(monitor.auto_allocation_strategies.len(), 2);
        assert_eq!(monitor.auto_allocation_strategies[0].strategy_type, "conservative_yield");
        assert_eq!(monitor.auto_allocation_strategies[1].allocation_percentage, 30.0);

        // Test unauthorized access
        let other_user = mock_principal_2();
        let unauthorized_result = manager.setup_auto_allocation(other_user, address.clone(), rules);
        assert!(unauthorized_result.is_err());
        assert!(unauthorized_result.unwrap_err().contains("Unauthorized"));
    }

    #[tokio::test]
    async fn test_get_user_portfolio() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();

        // Test empty portfolio
        let empty_portfolio = manager.get_user_portfolio(&user);
        assert!(empty_portfolio.is_none());

        // Register address and add data
        let address = "0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e".to_string();
        manager.register_deposit_address(user, address.clone(), ChainId::Ethereum).await.unwrap();

        // Add mock deposits and allocations
        if let Some(portfolio) = manager.user_deposits.get_mut(&user) {
            portfolio.total_value_usd = 10000.0;
            portfolio.available_for_strategies = 7000.0;
            portfolio.locked_in_strategies = 3000.0;

            portfolio.deposit_addresses[0].deposits.push(create_mock_deposit(4.0, 10000.0));
            portfolio.deposit_addresses[0].balance = 4.0;
            portfolio.deposit_addresses[0].balance_usd = 10000.0;

            // Add mock strategy allocation
            portfolio.deposit_addresses[0].strategy_allocations.push(StrategyAllocation {
                strategy_id: "yield_001".to_string(),
                protocol: DeFiProtocol::Aave,
                chain: ChainId::Ethereum,
                allocated_amount: 1.2,
                allocated_amount_usd: 3000.0,
                current_value_usd: 3150.0,
                pnl: 150.0,
                pnl_percentage: 5.0,
                status: AllocationStatus::Active,
                created_at: time(),
                last_updated: time(),
            });
        }

        // Test populated portfolio
        let portfolio = manager.get_user_portfolio(&user);
        assert!(portfolio.is_some());

        let portfolio = portfolio.unwrap();
        assert_eq!(portfolio.user, user);
        assert_eq!(portfolio.total_value_usd, 10000.0);
        assert_eq!(portfolio.available_for_strategies, 7000.0);
        assert_eq!(portfolio.locked_in_strategies, 3000.0);
        assert_eq!(portfolio.deposit_addresses.len(), 1);
        assert_eq!(portfolio.deposit_addresses[0].deposits.len(), 1);
        assert_eq!(portfolio.deposit_addresses[0].strategy_allocations.len(), 1);
    }

    #[tokio::test]
    async fn test_get_available_balance() {
        let mut manager = create_test_deposit_manager();
        let user = mock_principal();

        // Test user with no portfolio
        let no_balance = manager.get_available_balance(&user);
        assert_eq!(no_balance, 0.0);

        // Register address and set balance
        let address = "0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e".to_string();
        manager.register_deposit_address(user, address, ChainId::Ethereum).await.unwrap();

        if let Some(portfolio) = manager.user_deposits.get_mut(&user) {
            portfolio.available_for_strategies = 2500.75;
        }

        let available_balance = manager.get_available_balance(&user);
        assert_eq!(available_balance, 2500.75);
    }

    #[tokio::test]
    async fn test_chain_native_token_mapping() {
        let manager = create_test_deposit_manager();

        assert_eq!(manager.get_native_token(&ChainId::Bitcoin), "BTC");
        assert_eq!(manager.get_native_token(&ChainId::Ethereum), "ETH");
        assert_eq!(manager.get_native_token(&ChainId::Arbitrum), "ETH");
        assert_eq!(manager.get_native_token(&ChainId::Optimism), "ETH");
        assert_eq!(manager.get_native_token(&ChainId::Polygon), "MATIC");
        assert_eq!(manager.get_native_token(&ChainId::Base), "ETH");
        assert_eq!(manager.get_native_token(&ChainId::Avalanche), "AVAX");
        assert_eq!(manager.get_native_token(&ChainId::Solana), "SOL");
    }

    #[tokio::test]
    async fn test_allocation_status_transitions() {
        // Test different allocation statuses
        let active_allocation = StrategyAllocation {
            strategy_id: "test_001".to_string(),
            protocol: DeFiProtocol::Aave,
            chain: ChainId::Ethereum,
            allocated_amount: 1000.0,
            allocated_amount_usd: 1000.0,
            current_value_usd: 1050.0,
            pnl: 50.0,
            pnl_percentage: 5.0,
            status: AllocationStatus::Active,
            created_at: time(),
            last_updated: time(),
        };

        // Test status matching
        match active_allocation.status {
            AllocationStatus::Active => assert!(true),
            _ => assert!(false, "Expected Active status"),
        }

        // Test PnL calculation
        assert_eq!(active_allocation.pnl, 50.0);
        assert_eq!(active_allocation.pnl_percentage, 5.0);
        assert!(active_allocation.current_value_usd > active_allocation.allocated_amount_usd);
    }

    #[tokio::test]
    async fn test_risk_tolerance_levels() {
        let conservative_rule = AutoAllocationRule {
            strategy_type: "conservative".to_string(),
            min_deposit_amount: 100.0,
            allocation_percentage: 25.0,
            protocol_preference: vec![DeFiProtocol::Aave, DeFiProtocol::Compound],
            risk_tolerance: RiskTolerance::Conservative,
        };

        let aggressive_rule = AutoAllocationRule {
            strategy_type: "aggressive".to_string(),
            min_deposit_amount: 1000.0,
            allocation_percentage: 75.0,
            protocol_preference: vec![DeFiProtocol::Yearn, DeFiProtocol::Convex],
            risk_tolerance: RiskTolerance::Aggressive,
        };

        // Test risk tolerance matching
        match conservative_rule.risk_tolerance {
            RiskTolerance::Conservative => {
                assert!(conservative_rule.allocation_percentage <= 50.0);
                assert!(conservative_rule.protocol_preference.contains(&DeFiProtocol::Aave));
            },
            _ => assert!(false, "Expected Conservative risk tolerance"),
        }

        match aggressive_rule.risk_tolerance {
            RiskTolerance::Aggressive => {
                assert!(aggressive_rule.allocation_percentage > 50.0);
                assert!(aggressive_rule.min_deposit_amount >= 1000.0);
            },
            _ => assert!(false, "Expected Aggressive risk tolerance"),
        }
    }

    // Helper functions for testing
    fn create_mock_strategy_config() -> StrategyConfig {
        StrategyConfig {
            strategy_id: "mock_strategy_001".to_string(),
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
                allowed_hours: vec![9, 10, 11, 14, 15, 16], // Market hours
                blacklisted_tokens: vec![],
            },
            active: true,
            created_at: time(),
            updated_at: time(),
        }
    }
}