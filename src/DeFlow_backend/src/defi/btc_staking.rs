// Bitcoin Staking via Stacks-ICP Wrapped Tokens
// Enables BTC holders to earn yield through stacking on Stacks L2
// Combines: Bitcoin → sBTC (Stacks) → Stacking rewards → ICP canister custody

use candid::{CandidType, Deserialize};
use serde::Serialize;
use super::stacks::*;
use super::yield_farming::ChainId;
use std::collections::HashMap;

/// Wrapped BTC staking manager
#[derive(Debug, Clone)]
pub struct BtcStakingManager {
    pub stacks_service: StacksService,
    pub staking_pools: Vec<StackingPool>,
    pub user_positions: HashMap<String, Vec<StakingPosition>>,
    pub total_staked_btc: u64, // in satoshis
}

/// Stacking pool on Stacks network
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StackingPool {
    pub pool_id: String,
    pub pool_name: String,
    pub pool_address: String,
    pub apy: f64,
    pub total_stacked: u64,        // Total BTC stacked (satoshis)
    pub min_stacking_amount: u64,   // Minimum amount to stack
    pub lock_period_cycles: u64,    // Lock period in Stacks cycles (~2 weeks per cycle)
    pub payout_frequency: PayoutFrequency,
    pub risk_score: f64,            // 0.0-1.0
    pub is_active: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PayoutFrequency {
    PerCycle,    // Every ~2 weeks (1 Stacks cycle)
    BiWeekly,
    Monthly,
}

/// User's staking position
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StakingPosition {
    pub position_id: String,
    pub pool_id: String,
    pub user: String,
    pub btc_amount: u64,            // Amount in satoshis
    pub sbtc_amount: u64,           // Wrapped sBTC amount
    pub stacking_address: String,   // Stacks address receiving rewards
    pub start_cycle: u64,
    pub lock_cycles: u64,
    pub current_cycle: u64,
    pub rewards_earned: u64,        // BTC rewards in satoshis
    pub status: StakingStatus,
    pub entered_at: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq)]
pub enum StakingStatus {
    Pending,        // Waiting for BTC → sBTC bridge
    Active,         // Currently stacking
    Unlocking,      // Lock period ended, can withdraw
    Withdrawn,
}

impl BtcStakingManager {
    pub fn new(network: StacksNetwork) -> Self {
        Self {
            stacks_service: StacksService::new(network),
            staking_pools: Vec::new(),
            user_positions: HashMap::new(),
            total_staked_btc: 0,
        }
    }

    /// Initialize with default stacking pools
    pub fn initialize_pools(&mut self) {
        self.staking_pools = vec![
            StackingPool {
                pool_id: "stacks_pool_1".to_string(),
                pool_name: "StackingDAO Pool".to_string(),
                pool_address: "SP000000000000000000002Q6VF78".to_string(),
                apy: 5.8,
                total_stacked: 0,
                min_stacking_amount: 100_000, // 0.001 BTC minimum
                lock_period_cycles: 2,         // 2 cycles = ~4 weeks
                payout_frequency: PayoutFrequency::PerCycle,
                risk_score: 0.2,
                is_active: true,
            },
            StackingPool {
                pool_id: "stacks_pool_2".to_string(),
                pool_name: "Xverse Stacking".to_string(),
                pool_address: "SP000000000000000000002Q6VF79".to_string(),
                apy: 6.2,
                total_stacked: 0,
                min_stacking_amount: 50_000,   // 0.0005 BTC
                lock_period_cycles: 3,          // 3 cycles = ~6 weeks
                payout_frequency: PayoutFrequency::BiWeekly,
                risk_score: 0.25,
                is_active: true,
            },
        ];
    }

    /// Stake BTC (converts to sBTC and stakes on Stacks)
    pub async fn stake_btc(
        &mut self,
        user: String,
        btc_amount: u64,
        btc_tx_id: String,
        pool_id: String,
        stacks_address: String,
    ) -> Result<StakingPosition, String> {
        // Find pool
        let pool = self.staking_pools
            .iter()
            .find(|p| p.pool_id == pool_id && p.is_active)
            .ok_or("Pool not found or inactive")?;

        // Validate amount
        if btc_amount < pool.min_stacking_amount {
            return Err(format!(
                "Amount too small. Minimum: {} satoshis",
                pool.min_stacking_amount
            ));
        }

        // Step 1: Bridge BTC to sBTC on Stacks
        let sbtc_tx = self.stacks_service
            .deposit_btc_to_sbtc(btc_tx_id.clone(), btc_amount, stacks_address.clone())
            .await?;

        // Step 2: Stack sBTC in the pool
        // In production, this would call the Stacks stacking contract
        let position_id = format!("stake_{}_{}", user, ic_cdk::api::time());
        let current_cycle = self.get_current_stacks_cycle();

        let position = StakingPosition {
            position_id,
            pool_id: pool.pool_id.clone(),
            user: user.clone(),
            btc_amount,
            sbtc_amount: btc_amount, // 1:1 for sBTC
            stacking_address: stacks_address,
            start_cycle: current_cycle,
            lock_cycles: pool.lock_period_cycles,
            current_cycle,
            rewards_earned: 0,
            status: StakingStatus::Pending,
            entered_at: ic_cdk::api::time(),
        };

        // Record position
        self.user_positions
            .entry(user)
            .or_insert_with(Vec::new)
            .push(position.clone());

        self.total_staked_btc += btc_amount;

        Ok(position)
    }

    /// Unstake and withdraw BTC
    pub async fn unstake_btc(
        &mut self,
        user: String,
        position_id: String,
    ) -> Result<String, String> {
        // Check if lock period ended first
        let current_cycle = self.get_current_stacks_cycle();

        // Find position
        let user_positions = self.user_positions
            .get_mut(&user)
            .ok_or("No positions found for user")?;

        let position = user_positions
            .iter_mut()
            .find(|p| p.position_id == position_id)
            .ok_or("Position not found")?;

        if current_cycle < position.start_cycle + position.lock_cycles {
            return Err(format!(
                "Still locked. Unlocks in {} cycles",
                (position.start_cycle + position.lock_cycles) - current_cycle
            ));
        }

        // Unstake from Stacks pool
        // In production, call Stacks contract to unstake

        position.status = StakingStatus::Unlocking;

        // Bridge sBTC back to BTC
        let btc_address = "bc1q..."; // User's Bitcoin address
        let withdraw_tx = self.stacks_service
            .withdraw_sbtc_to_btc(
                position.sbtc_amount + position.rewards_earned,
                btc_address.to_string(),
                position.stacking_address.clone(),
            )
            .await?;

        position.status = StakingStatus::Withdrawn;
        self.total_staked_btc -= position.btc_amount;

        Ok(withdraw_tx.tx_id)
    }

    /// Claim stacking rewards
    pub async fn claim_rewards(
        &mut self,
        user: String,
        position_id: String,
    ) -> Result<u64, String> {
        // Get current cycle first to avoid borrow conflicts
        let current_cycle = self.get_current_stacks_cycle();

        let user_positions = self.user_positions
            .get_mut(&user)
            .ok_or("No positions found for user")?;

        let position = user_positions
            .iter_mut()
            .find(|p| p.position_id == position_id)
            .ok_or("Position not found")?;

        if position.status != StakingStatus::Active {
            return Err("Position not active".to_string());
        }

        // Calculate pending rewards
        let pool = self.staking_pools
            .iter()
            .find(|p| p.pool_id == position.pool_id)
            .ok_or("Pool not found")?;

        let cycles_stacked = current_cycle - position.start_cycle;

        let annual_rewards = (position.btc_amount as f64 * pool.apy / 100.0) as u64;
        let cycle_duration_days = 14; // ~2 weeks per cycle
        let rewards_per_cycle = annual_rewards * cycle_duration_days / 365;
        let pending_rewards = rewards_per_cycle * cycles_stacked;

        // In production, claim from Stacks contract
        position.rewards_earned += pending_rewards;
        position.current_cycle = current_cycle;

        Ok(pending_rewards)
    }

    /// Get user's total staking positions
    pub fn get_user_positions(&self, user: &str) -> Vec<StakingPosition> {
        self.user_positions
            .get(user)
            .cloned()
            .unwrap_or_default()
    }

    /// Get total staking stats
    pub fn get_staking_stats(&self) -> StakingStats {
        let total_positions: usize = self.user_positions.values().map(|v| v.len()).sum();

        let total_rewards: u64 = self.user_positions
            .values()
            .flatten()
            .map(|p| p.rewards_earned)
            .sum();

        let avg_apy = if !self.staking_pools.is_empty() {
            self.staking_pools.iter().map(|p| p.apy).sum::<f64>() / self.staking_pools.len() as f64
        } else {
            0.0
        };

        StakingStats {
            total_staked_btc: self.total_staked_btc,
            total_staked_usd: (self.total_staked_btc as f64 / 100_000_000.0) * 45000.0, // Assuming $45k BTC
            total_positions,
            total_rewards_paid: total_rewards,
            average_apy: avg_apy,
            active_pools: self.staking_pools.iter().filter(|p| p.is_active).count(),
        }
    }

    /// Get best pool for amount
    pub fn get_best_pool(&self, amount: u64) -> Option<&StackingPool> {
        self.staking_pools
            .iter()
            .filter(|p| p.is_active && amount >= p.min_stacking_amount)
            .max_by(|a, b| a.apy.partial_cmp(&b.apy).unwrap())
    }

    /// Get current Stacks cycle
    fn get_current_stacks_cycle(&self) -> u64 {
        // Stacks cycles are ~2 weeks each
        // In production, query Stacks blockchain for current cycle
        let time_since_genesis = ic_cdk::api::time() - 1_600_000_000_000_000_000; // Approximate
        let seconds_per_cycle = 14 * 24 * 60 * 60; // 2 weeks
        let nanos_per_cycle = seconds_per_cycle * 1_000_000_000;

        time_since_genesis / nanos_per_cycle
    }
}

/// Staking statistics
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StakingStats {
    pub total_staked_btc: u64,
    pub total_staked_usd: f64,
    pub total_positions: usize,
    pub total_rewards_paid: u64,
    pub average_apy: f64,
    pub active_pools: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_manager_creation() {
        let mut manager = BtcStakingManager::new(StacksNetwork::Testnet);
        manager.initialize_pools();

        assert_eq!(manager.staking_pools.len(), 2);
        assert_eq!(manager.total_staked_btc, 0);
    }

    #[test]
    fn test_get_best_pool() {
        let mut manager = BtcStakingManager::new(StacksNetwork::Testnet);
        manager.initialize_pools();

        let best = manager.get_best_pool(100_000).unwrap();
        assert!(best.apy > 5.0);
        assert!(best.is_active);
    }

    #[test]
    fn test_min_stacking_amount() {
        let mut manager = BtcStakingManager::new(StacksNetwork::Testnet);
        manager.initialize_pools();

        let pool = &manager.staking_pools[0];
        assert!(pool.min_stacking_amount > 0);
    }
}
