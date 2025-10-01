// Stacks Bitcoin Yield Strategies
// Integrates sBTC yield opportunities into DeFlow's multi-chain yield optimization engine

use super::stacks::*;
use super::yield_farming::{ChainId, DeFiProtocol, YieldStrategy};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

/// Stacks-specific yield strategy implementation
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StacksBtcYieldStrategy {
    pub strategy_id: String,
    pub strategy_name: String,
    pub protocol: StacksProtocol,
    pub current_apy: f64,
    pub tvl_usd: f64,
    pub user_deposits: HashMap<String, u64>, // user_address -> amount in satoshis
    pub total_deposited: u64,                 // Total sBTC deposited in satoshis
    pub rewards_earned: u64,                  // Total rewards in satoshis
    pub risk_level: RiskLevel,
    pub is_active: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,       // 0.0-0.3
    Medium,    // 0.3-0.6
    High,      // 0.6-1.0
}

impl StacksBtcYieldStrategy {
    pub fn new(protocol: StacksProtocol, strategy_name: String) -> Self {
        let risk_level = match protocol.protocol_type {
            StacksProtocolType::LiquidStaking => RiskLevel::Low,
            StacksProtocolType::Lending => RiskLevel::Medium,
            StacksProtocolType::Dex | StacksProtocolType::YieldAggregator => RiskLevel::Medium,
        };

        Self {
            strategy_id: format!("stacks_{}", strategy_name.to_lowercase().replace(' ', "_")),
            strategy_name,
            protocol,
            current_apy: 0.0,
            tvl_usd: 0.0,
            user_deposits: HashMap::new(),
            total_deposited: 0,
            rewards_earned: 0,
            risk_level,
            is_active: true,
        }
    }

    /// Update APY from real-time data
    pub fn update_apy(&mut self, new_apy: f64) {
        self.current_apy = new_apy;
    }

    /// Record user deposit
    pub fn add_deposit(&mut self, user: String, amount: u64) {
        *self.user_deposits.entry(user).or_insert(0) += amount;
        self.total_deposited += amount;
    }

    /// Record user withdrawal
    pub fn remove_deposit(&mut self, user: String, amount: u64) -> Result<(), String> {
        let balance = self.user_deposits.get(&user).copied().unwrap_or(0);

        if balance < amount {
            return Err(format!("Insufficient balance: has {} satoshis, requested {}", balance, amount));
        }

        *self.user_deposits.get_mut(&user).unwrap() -= amount;
        self.total_deposited -= amount;

        Ok(())
    }

    /// Get user's deposited amount
    pub fn get_user_balance(&self, user: &str) -> u64 {
        self.user_deposits.get(user).copied().unwrap_or(0)
    }

    /// Calculate estimated daily rewards for a deposit amount
    pub fn calculate_daily_rewards(&self, amount: u64) -> u64 {
        let daily_rate = self.current_apy / 365.0 / 100.0;
        (amount as f64 * daily_rate) as u64
    }

    /// Calculate estimated annual rewards for a deposit amount
    pub fn calculate_annual_rewards(&self, amount: u64) -> u64 {
        (amount as f64 * self.current_apy / 100.0) as u64
    }
}

/// Stacks Bitcoin yield manager
#[derive(Debug, Clone)]
pub struct StacksBtcYieldManager {
    pub stacks_service: StacksService,
    pub strategies: Vec<StacksBtcYieldStrategy>,
    pub auto_compound_enabled: bool,
    pub rebalance_threshold: f64, // APY difference to trigger rebalance
}

impl StacksBtcYieldManager {
    pub fn new(network: StacksNetwork) -> Self {
        Self {
            stacks_service: StacksService::new(network),
            strategies: Vec::new(),
            auto_compound_enabled: true,
            rebalance_threshold: 2.0, // 2% APY difference
        }
    }

    /// Initialize with default Stacks DeFi strategies
    pub async fn initialize_strategies(&mut self) -> Result<(), String> {
        let opportunities = self.stacks_service.get_yield_opportunities().await?;

        for opp in opportunities {
            let strategy = StacksBtcYieldStrategy::new(
                opp.protocol.clone(),
                opp.strategy_name.clone(),
            );
            self.strategies.push(strategy);
        }

        Ok(())
    }

    /// Get best yield strategy based on APY and risk
    pub fn get_best_strategy(&self, max_risk: RiskLevel) -> Option<&StacksBtcYieldStrategy> {
        self.strategies
            .iter()
            .filter(|s| s.is_active && self.matches_risk_level(&s.risk_level, &max_risk))
            .max_by(|a, b| a.current_apy.partial_cmp(&b.current_apy).unwrap())
    }

    /// Check if strategy risk level is acceptable
    fn matches_risk_level(&self, strategy_risk: &RiskLevel, max_risk: &RiskLevel) -> bool {
        match max_risk {
            RiskLevel::Low => matches!(strategy_risk, RiskLevel::Low),
            RiskLevel::Medium => matches!(strategy_risk, RiskLevel::Low | RiskLevel::Medium),
            RiskLevel::High => true, // All risk levels acceptable
        }
    }

    /// Deposit sBTC into a strategy
    pub async fn deposit_to_strategy(
        &mut self,
        strategy_id: &str,
        user: String,
        amount: u64,
    ) -> Result<String, String> {
        // Find strategy
        let strategy = self.strategies
            .iter_mut()
            .find(|s| s.strategy_id == strategy_id)
            .ok_or_else(|| format!("Strategy not found: {}", strategy_id))?;

        // Check minimum deposit
        if amount < 50_000 {
            return Err("Minimum deposit is 50,000 satoshis (0.0005 BTC)".to_string());
        }

        // Record deposit
        strategy.add_deposit(user.clone(), amount);

        // In production, would interact with actual Stacks protocol contract
        let tx_id = format!("stacks_deposit_{}_{}", strategy_id, ic_cdk::api::time());

        Ok(tx_id)
    }

    /// Withdraw sBTC from a strategy
    pub async fn withdraw_from_strategy(
        &mut self,
        strategy_id: &str,
        user: String,
        amount: u64,
    ) -> Result<String, String> {
        // Find strategy
        let strategy = self.strategies
            .iter_mut()
            .find(|s| s.strategy_id == strategy_id)
            .ok_or_else(|| format!("Strategy not found: {}", strategy_id))?;

        // Remove deposit
        strategy.remove_deposit(user.clone(), amount)?;

        // In production, would interact with actual Stacks protocol contract
        let tx_id = format!("stacks_withdrawal_{}_{}", strategy_id, ic_cdk::api::time());

        Ok(tx_id)
    }

    /// Update all strategy APYs from live data
    pub async fn refresh_apys(&mut self) -> Result<(), String> {
        let opportunities = self.stacks_service.get_yield_opportunities().await?;

        for opp in opportunities {
            if let Some(strategy) = self.strategies.iter_mut().find(|s|
                s.protocol.name == opp.protocol.name
            ) {
                strategy.update_apy(opp.apy);
                strategy.tvl_usd = opp.protocol.tvl_usd;
            }
        }

        Ok(())
    }

    /// Check if rebalancing is beneficial
    pub fn should_rebalance(&self, current_strategy_id: &str, user_amount: u64) -> Option<String> {
        let current_strategy = self.strategies.iter().find(|s| s.strategy_id == current_strategy_id)?;
        let best_strategy = self.get_best_strategy(current_strategy.risk_level.clone())?;

        // Rebalance if APY difference exceeds threshold
        if best_strategy.current_apy - current_strategy.current_apy >= self.rebalance_threshold {
            // Calculate if gas costs are worth it
            let annual_gain = user_amount as f64 * (best_strategy.current_apy - current_strategy.current_apy) / 100.0;
            let estimated_cost = 5_000.0; // Estimated cost in satoshis

            if annual_gain > estimated_cost * 12.0 { // At least 12x gas cost in annual gains
                return Some(best_strategy.strategy_id.clone());
            }
        }

        None
    }

    /// Get all active strategies
    pub fn get_active_strategies(&self) -> Vec<&StacksBtcYieldStrategy> {
        self.strategies.iter().filter(|s| s.is_active).collect()
    }

    /// Get strategy by ID
    pub fn get_strategy(&self, strategy_id: &str) -> Option<&StacksBtcYieldStrategy> {
        self.strategies.iter().find(|s| s.strategy_id == strategy_id)
    }

    /// Get user's total balance across all strategies
    pub fn get_user_total_balance(&self, user: &str) -> u64 {
        self.strategies
            .iter()
            .map(|s| s.get_user_balance(user))
            .sum()
    }

    /// Get user's portfolio breakdown
    pub fn get_user_portfolio(&self, user: &str) -> Vec<UserStrategyPosition> {
        self.strategies
            .iter()
            .filter_map(|s| {
                let balance = s.get_user_balance(user);
                if balance > 0 {
                    Some(UserStrategyPosition {
                        strategy_id: s.strategy_id.clone(),
                        strategy_name: s.strategy_name.clone(),
                        protocol_name: s.protocol.name.clone(),
                        deposited_amount: balance,
                        current_apy: s.current_apy,
                        estimated_daily_rewards: s.calculate_daily_rewards(balance),
                        estimated_annual_rewards: s.calculate_annual_rewards(balance),
                        risk_level: s.risk_level.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// User's position in a strategy
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserStrategyPosition {
    pub strategy_id: String,
    pub strategy_name: String,
    pub protocol_name: String,
    pub deposited_amount: u64,
    pub current_apy: f64,
    pub estimated_daily_rewards: u64,
    pub estimated_annual_rewards: u64,
    pub risk_level: RiskLevel,
}

/// Convert Stacks strategy to generic YieldStrategy for cross-chain optimization
pub fn stacks_to_yield_strategy(stacks_strategy: &StacksBtcYieldStrategy) -> YieldStrategy {
    use super::yield_farming::{YieldStrategy, YieldStrategyType};

    YieldStrategy {
        id: stacks_strategy.strategy_id.clone(),
        protocol: DeFiProtocol::StacksDefi,
        chain: ChainId::Stacks,
        strategy_type: YieldStrategyType::Lending {
            asset: "sBTC".to_string(),
            variable_rate: true,
        },
        current_apy: stacks_strategy.current_apy,
        historical_apy_7d: stacks_strategy.current_apy * 0.95, // Estimate
        historical_apy_30d: stacks_strategy.current_apy * 0.9, // Estimate
        risk_score: match stacks_strategy.risk_level {
            RiskLevel::Low => 2,
            RiskLevel::Medium => 5,
            RiskLevel::High => 8,
        },
        liquidity_usd: stacks_strategy.tvl_usd as u64,
        min_deposit_usd: 50, // ~$50 minimum (0.0005 BTC)
        max_deposit_usd: None,
        deposit_fee: 0.0,
        withdrawal_fee: 0.001, // 0.1% withdrawal fee
        performance_fee: 0.0,
        lock_period: None,
        auto_compound: true,
        verified: true,
        last_updated: ic_cdk::api::time(),
        entry_apy: None,
        entry_timestamp: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_protocol() -> StacksProtocol {
        StacksProtocol {
            name: "Test Protocol".to_string(),
            contract_address: "SP000000000000000000002Q6VF78".to_string(),
            contract_name: "test-vault".to_string(),
            tvl_usd: 1_000_000.0,
            apy: 8.5,
            protocol_type: StacksProtocolType::Lending,
        }
    }

    #[test]
    fn test_strategy_creation() {
        let protocol = create_test_protocol();
        let strategy = StacksBtcYieldStrategy::new(protocol, "Test Strategy".to_string());

        assert_eq!(strategy.strategy_name, "Test Strategy");
        assert_eq!(strategy.total_deposited, 0);
        assert!(matches!(strategy.risk_level, RiskLevel::Medium));
        assert!(strategy.is_active);
    }

    #[test]
    fn test_deposit_and_withdrawal() {
        let protocol = create_test_protocol();
        let mut strategy = StacksBtcYieldStrategy::new(protocol, "Test Strategy".to_string());

        // Deposit
        strategy.add_deposit("user1".to_string(), 100_000);
        assert_eq!(strategy.get_user_balance("user1"), 100_000);
        assert_eq!(strategy.total_deposited, 100_000);

        // Another deposit from same user
        strategy.add_deposit("user1".to_string(), 50_000);
        assert_eq!(strategy.get_user_balance("user1"), 150_000);
        assert_eq!(strategy.total_deposited, 150_000);

        // Withdrawal
        let result = strategy.remove_deposit("user1".to_string(), 75_000);
        assert!(result.is_ok());
        assert_eq!(strategy.get_user_balance("user1"), 75_000);
        assert_eq!(strategy.total_deposited, 75_000);

        // Withdrawal exceeding balance
        let result = strategy.remove_deposit("user1".to_string(), 100_000);
        assert!(result.is_err());
    }

    #[test]
    fn test_rewards_calculation() {
        let protocol = create_test_protocol();
        let mut strategy = StacksBtcYieldStrategy::new(protocol, "Test Strategy".to_string());
        strategy.update_apy(10.0); // 10% APY

        let amount = 100_000_000; // 1 BTC in satoshis

        // Daily rewards: 1 BTC * 10% / 365 = ~27,397 satoshis
        let daily = strategy.calculate_daily_rewards(amount);
        assert!(daily > 27_000 && daily < 28_000);

        // Annual rewards: 1 BTC * 10% = 10,000,000 satoshis
        let annual = strategy.calculate_annual_rewards(amount);
        assert_eq!(annual, 10_000_000);
    }

    #[test]
    fn test_risk_level_matching() {
        let manager = StacksBtcYieldManager::new(StacksNetwork::Testnet);

        assert!(manager.matches_risk_level(&RiskLevel::Low, &RiskLevel::Low));
        assert!(manager.matches_risk_level(&RiskLevel::Low, &RiskLevel::Medium));
        assert!(manager.matches_risk_level(&RiskLevel::Low, &RiskLevel::High));

        assert!(!manager.matches_risk_level(&RiskLevel::Medium, &RiskLevel::Low));
        assert!(manager.matches_risk_level(&RiskLevel::Medium, &RiskLevel::Medium));
        assert!(manager.matches_risk_level(&RiskLevel::Medium, &RiskLevel::High));

        assert!(!manager.matches_risk_level(&RiskLevel::High, &RiskLevel::Low));
        assert!(!manager.matches_risk_level(&RiskLevel::High, &RiskLevel::Medium));
        assert!(manager.matches_risk_level(&RiskLevel::High, &RiskLevel::High));
    }
}
