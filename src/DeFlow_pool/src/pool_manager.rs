use crate::types::*;
use ic_cdk::api::time;
use std::collections::HashMap;

pub struct PoolManager {
    // Pool management configuration
    pub min_reserve_ratio: f64,
    pub max_utilization_ratio: f64,
    pub rebalancing_threshold: f64,
}

impl PoolManager {
    pub fn new() -> Self {
        PoolManager {
            min_reserve_ratio: 0.2,  // Always keep 20% in reserves
            max_utilization_ratio: 0.8,  // Never use more than 80%
            rebalancing_threshold: 0.1,  // Rebalance if 10% imbalance
        }
    }
    
    // =============================================================================
    // LIQUIDITY MANAGEMENT
    // =============================================================================
    
    pub fn add_to_reserves(&mut self, pool_state: &mut PoolState, asset: Asset, amount: u64) -> Result<(), String> {
        let chain_id = self.get_primary_chain_for_asset(&asset);
        
        // Get or create chain reserves
        let chain_reserves = pool_state.reserves.entry(chain_id).or_insert_with(HashMap::new);
        
        // Get or create asset reserve
        let asset_reserve = chain_reserves.entry(asset.clone()).or_insert_with(LiquidityReserve::default);
        
        // SECURITY: Update reserve with safe arithmetic to prevent overflow
        asset_reserve.total_amount = match asset_reserve.total_amount.checked_add(amount) {
            Some(new_total) => new_total,
            None => {
                ic_cdk::println!("SECURITY: Integer overflow in fee deposit - current: {}, adding: {}", 
                               asset_reserve.total_amount, amount);
                return Err("SECURITY: Integer overflow in reserve calculation".to_string());
            }
        };
        
        asset_reserve.fee_contributed_amount = match asset_reserve.fee_contributed_amount.checked_add(amount) {
            Some(new_fee_total) => new_fee_total,
            None => {
                ic_cdk::println!("SECURITY: Integer overflow in fee contribution - current: {}, adding: {}", 
                               asset_reserve.fee_contributed_amount, amount);
                // Rollback the total_amount change
                asset_reserve.total_amount = asset_reserve.total_amount.saturating_sub(amount);
                return Err("SECURITY: Integer overflow in fee contribution calculation".to_string());
            }
        };
        asset_reserve.last_updated = time();
        
        // Update daily growth rate
        self.update_growth_rate(asset_reserve);
        
        // Update total liquidity USD value
        self.update_total_liquidity_usd(pool_state);
        
        Ok(())
    }
    
    pub fn add_liquidity(&mut self, pool_state: &mut PoolState, chain_id: ChainId, asset: Asset, amount: u64) -> Result<(), String> {
        // Get or create chain reserves
        let chain_reserves = pool_state.reserves.entry(chain_id).or_insert_with(HashMap::new);
        
        // Get or create asset reserve
        let asset_reserve = chain_reserves.entry(asset.clone()).or_insert_with(LiquidityReserve::default);
        
        // SECURITY: Update reserve (external liquidity addition, not from fees) with safe arithmetic
        asset_reserve.total_amount = match asset_reserve.total_amount.checked_add(amount) {
            Some(new_total) => new_total,
            None => {
                ic_cdk::println!("SECURITY: Integer overflow in liquidity addition - current: {}, adding: {}", 
                               asset_reserve.total_amount, amount);
                return Err("SECURITY: Integer overflow in liquidity calculation".to_string());
            }
        };
        asset_reserve.last_updated = time();
        
        // Update growth rate and total liquidity
        self.update_growth_rate(asset_reserve);
        self.update_total_liquidity_usd(pool_state);
        
        Ok(())
    }
    
    pub fn withdraw_for_execution(&mut self, pool_state: &mut PoolState, asset: Asset, amount: u64) -> Result<String, String> {
        match &pool_state.phase {
            PoolPhase::Bootstrapping { .. } => {
                // SECURITY: ABSOLUTE BLOCK on withdrawals during bootstrap
                Err("SECURITY BLOCK: Pool withdrawals are PERMANENTLY DISABLED during bootstrap phase. No exceptions. Funds can only accumulate.".to_string())
            },
            PoolPhase::Active { .. } => {
                self.execute_withdrawal_with_reserves(pool_state, asset, amount)
            },
            PoolPhase::Emergency { reason, .. } => {
                Err(format!("Pool paused in emergency mode: {}", reason))
            },
            PoolPhase::Terminating { .. } => {
                Err("SECURITY BLOCK: Pool withdrawals disabled during termination process".to_string())
            },
            PoolPhase::Terminated { .. } => {
                Err("SECURITY BLOCK: Pool has been terminated, no withdrawals possible".to_string())
            }
        }
    }
    
    fn execute_withdrawal_with_reserves(&mut self, pool_state: &mut PoolState, asset: Asset, amount: u64) -> Result<String, String> {
        let chain_id = self.get_primary_chain_for_asset(&asset);
        
        // Check if we have sufficient reserves
        let available_amount = pool_state.reserves
            .get(&chain_id)
            .and_then(|chain_reserves| chain_reserves.get(&asset))
            .map(|reserve| (reserve.total_amount as f64 * self.max_utilization_ratio) as u64)
            .unwrap_or(0);
        
        if amount > available_amount {
            return Err(format!("Insufficient liquidity. Available: {}, Requested: {}", available_amount, amount));
        }
        
        // SECURITY: Execute withdrawal with safe arithmetic
        if let Some(chain_reserves) = pool_state.reserves.get_mut(&chain_id) {
            if let Some(asset_reserve) = chain_reserves.get_mut(&asset) {
                asset_reserve.total_amount = match asset_reserve.total_amount.checked_sub(amount) {
                    Some(new_total) => new_total,
                    None => {
                        ic_cdk::println!("SECURITY: Integer underflow in withdrawal - current: {}, removing: {}", 
                                       asset_reserve.total_amount, amount);
                        return Err("SECURITY: Insufficient funds for withdrawal".to_string());
                    }
                };
                asset_reserve.utilization_rate = 1.0 - (asset_reserve.total_amount as f64 / (asset_reserve.total_amount as f64 + amount as f64));
                asset_reserve.last_updated = time();
                
                // Update total liquidity
                self.update_total_liquidity_usd(pool_state);
                
                Ok(format!("Withdrew {} {} from pool", amount, asset.to_string()))
            } else {
                Err("Asset not found in reserves".to_string())
            }
        } else {
            Err("Chain not found in reserves".to_string())
        }
    }
    
    // =============================================================================
    // BOOTSTRAP MANAGEMENT
    // =============================================================================
    
    pub fn set_bootstrap_targets(&mut self, pool_state: &mut PoolState, targets: Vec<(Asset, u64)>) -> Result<(), String> {
        match &pool_state.phase {
            PoolPhase::Bootstrapping { .. } => {
                pool_state.bootstrap_targets = targets.into_iter().collect();
                Ok(())
            },
            _ => Err("Can only set bootstrap targets during bootstrapping phase".to_string())
        }
    }
    
    pub fn check_bootstrap_completion(&mut self, pool_state: &mut PoolState) -> Result<(), String> {
        match &pool_state.phase {
            PoolPhase::Bootstrapping { .. } => {
                let all_targets_met = pool_state.bootstrap_targets.iter().all(|(asset, target)| {
                    let current_amount = self.get_total_asset_amount(pool_state, asset);
                    current_amount >= *target
                });
                
                if all_targets_met {
                    self.transition_to_active_phase(pool_state)?;
                }
                
                Ok(())
            },
            _ => Ok(()) // Not in bootstrap phase
        }
    }
    
    pub fn get_bootstrap_progress(&self, pool_state: &PoolState) -> f64 {
        if pool_state.bootstrap_targets.is_empty() {
            return 1.0; // 100% if no targets set
        }
        
        let total_progress: f64 = pool_state.bootstrap_targets.iter()
            .map(|(asset, target)| {
                let current = self.get_total_asset_amount(pool_state, asset);
                (current as f64 / *target as f64).min(1.0)
            })
            .sum();
        
        total_progress / pool_state.bootstrap_targets.len() as f64
    }
    
    fn transition_to_active_phase(&mut self, pool_state: &mut PoolState) -> Result<(), String> {
        pool_state.phase = PoolPhase::Active {
            activated_at: time(),
            min_reserve_ratio: self.min_reserve_ratio,
            max_utilization: self.max_utilization_ratio,
        };
        
        Ok(())
    }
    
    // =============================================================================
    // POOL CONFIGURATION
    // =============================================================================
    
    pub fn activate_pool(&mut self, pool_state: &mut PoolState) -> Result<String, String> {
        match &pool_state.phase {
            PoolPhase::Bootstrapping { .. } => {
                // SECURITY: Strict bootstrap completion requirements
                let bootstrap_progress = self.get_bootstrap_progress(pool_state);
                
                // Must meet ALL bootstrap targets (100%), not just 80%
                if bootstrap_progress < 1.0 {
                    return Err(format!("Bootstrap incomplete: {:.1}%. ALL targets must be met (100%) before activation.", bootstrap_progress * 100.0));
                }
                
                // SECURITY: Additional safety check - verify total liquidity meets minimum threshold
                if pool_state.total_liquidity_usd < 400_000.0 { // $400K minimum
                    return Err(format!("Insufficient total liquidity: ${:.2}. Minimum $400,000 required.", pool_state.total_liquidity_usd));
                }
                
                self.transition_to_active_phase(pool_state)?;
                Ok("Pool activated successfully - all bootstrap targets met".to_string())
            },
            PoolPhase::Active { .. } => {
                Err("Pool already active".to_string())
            },
            PoolPhase::Emergency { .. } => {
                Err("Cannot activate pool while in emergency mode".to_string())
            },
            PoolPhase::Terminating { .. } => {
                Err("Cannot activate pool during termination process".to_string())
            },
            PoolPhase::Terminated { .. } => {
                Err("Cannot activate terminated pool".to_string())
            }
        }
    }
    
    pub fn emergency_pause(&mut self, pool_state: &mut PoolState, reason: String) -> Result<String, String> {
        pool_state.phase = PoolPhase::Emergency {
            paused_at: time(),
            reason: reason.clone(),
        };
        
        Ok(format!("Pool paused: {}", reason))
    }
    
    // =============================================================================
    // UTILITY FUNCTIONS
    // =============================================================================
    
    fn get_primary_chain_for_asset(&self, asset: &Asset) -> ChainId {
        match asset {
            Asset::BTC => ChainId::Bitcoin,
            Asset::ETH => ChainId::Ethereum,
            Asset::USDC => ChainId::Ethereum, // Primary USDC on Ethereum
            Asset::USDT => ChainId::Ethereum, // Primary USDT on Ethereum
            Asset::DAI => ChainId::Ethereum,
            Asset::SOL => ChainId::Solana,
            Asset::MATIC => ChainId::Polygon,
            Asset::AVAX => ChainId::Avalanche,
            Asset::FLOW => ChainId::Ethereum, // FLOW token deployed on Ethereum/ICP
        }
    }
    
    fn get_total_asset_amount(&self, pool_state: &PoolState, asset: &Asset) -> u64 {
        pool_state.reserves.values()
            .flat_map(|chain_reserves| chain_reserves.get(asset))
            .map(|reserve| reserve.total_amount)
            .sum()
    }
    
    fn update_growth_rate(&self, reserve: &mut LiquidityReserve) {
        // Simple growth rate calculation based on recent additions
        // In production, this would be more sophisticated
        let time_diff = (time() - reserve.last_updated) as f64 / 86_400_000_000_000f64; // Convert to days (24*60*60*1e9)
        if time_diff > 0.0 {
            let amount_diff = reserve.fee_contributed_amount as f64;
            reserve.daily_growth_rate = amount_diff / reserve.total_amount.max(1) as f64 / time_diff;
        }
    }
    
    fn update_total_liquidity_usd(&self, pool_state: &mut PoolState) {
        // Simplified USD calculation - in production, would use price oracles
        let mut total_usd = 0.0;
        
        for (_chain_id, chain_reserves) in &pool_state.reserves {
            for (asset, reserve) in chain_reserves {
                let usd_value = self.estimate_asset_usd_value(asset, reserve.total_amount);
                total_usd += usd_value;
            }
        }
        
        pool_state.total_liquidity_usd = total_usd;
    }
    
    fn estimate_asset_usd_value(&self, asset: &Asset, amount: u64) -> f64 {
        // Simplified price estimates - in production, would use real price oracles
        let (price_usd, decimals) = match asset {
            Asset::BTC => (45000.0, 8),
            Asset::ETH => (2500.0, 18),
            Asset::USDC => (1.0, 6),
            Asset::USDT => (1.0, 6),
            Asset::DAI => (1.0, 18),
            Asset::SOL => (100.0, 9),
            Asset::MATIC => (0.8, 18),
            Asset::AVAX => (25.0, 18),
            Asset::FLOW => (0.10, 8), // $0.10 per FLOW token
        };
        
        let normalized_amount = amount as f64 / 10_u64.pow(decimals) as f64;
        normalized_amount * price_usd
    }
}