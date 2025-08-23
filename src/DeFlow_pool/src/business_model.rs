use crate::types::*;
use ic_cdk::api::time;
use candid::Principal;

pub struct DevTeamBusinessManager {
    // Business model configuration
    pub operating_cost_estimate: f64,
    pub distribution_reserve_ratio: f64,
}

impl DevTeamBusinessManager {
    pub fn new() -> Self {
        DevTeamBusinessManager {
            operating_cost_estimate: 15000.0, // $15K monthly operating costs
            distribution_reserve_ratio: 0.2,   // Keep 20% for business growth
        }
    }
    
    // =============================================================================
    // REVENUE TRACKING
    // =============================================================================
    
    pub fn add_transaction_fee_revenue(&mut self, pool_state: &mut PoolState, amount: f64) -> Result<(), String> {
        // SECURITY: Safe addition with overflow protection
        if !amount.is_finite() || amount < 0.0 {
            return Err("SECURITY: Invalid transaction fee amount".to_string());
        }
        
        let new_total = pool_state.dev_team_business.monthly_transaction_fees + amount;
        if !new_total.is_finite() || new_total > 1_000_000_000.0 { // $1B limit
            return Err("SECURITY: Transaction fee total exceeds reasonable limits".to_string());
        }
        
        pool_state.dev_team_business.monthly_transaction_fees = new_total;
        Ok(())
    }
    
    pub fn process_subscription_payment(&mut self, pool_state: &mut PoolState, _user: Principal, amount: f64) -> Result<(), String> {
        // SECURITY: Safe addition with overflow protection
        if !amount.is_finite() || amount < 0.0 {
            return Err("SECURITY: Invalid subscription payment amount".to_string());
        }
        
        let new_total = pool_state.dev_team_business.monthly_subscription_revenue + amount;
        if !new_total.is_finite() || new_total > 1_000_000_000.0 { // $1B limit
            return Err("SECURITY: Subscription revenue total exceeds reasonable limits".to_string());
        }
        
        pool_state.dev_team_business.monthly_subscription_revenue = new_total;
        
        // Check for profit distribution after each payment
        self.check_and_execute_profit_distribution(pool_state)?;
        
        Ok(())
    }
    
    pub fn add_enterprise_revenue(&mut self, pool_state: &mut PoolState, amount: f64) -> Result<(), String> {
        // SECURITY: Safe addition with overflow protection
        if !amount.is_finite() || amount < 0.0 {
            return Err("SECURITY: Invalid enterprise revenue amount".to_string());
        }
        
        let new_total = pool_state.dev_team_business.monthly_enterprise_revenue + amount;
        if !new_total.is_finite() || new_total > 1_000_000_000.0 { // $1B limit
            return Err("SECURITY: Enterprise revenue total exceeds reasonable limits".to_string());
        }
        
        pool_state.dev_team_business.monthly_enterprise_revenue = new_total;
        Ok(())
    }
    
    // =============================================================================
    // PROFIT DISTRIBUTION
    // =============================================================================
    
    pub fn check_and_execute_profit_distribution(&mut self, pool_state: &mut PoolState) -> Result<(), String> {
        let current_time = time();
        
        // Check if distribution period has passed
        if current_time - pool_state.dev_team_business.last_distribution_time >= pool_state.dev_team_business.distribution_frequency {
            self.execute_monthly_distribution(pool_state)?;
        }
        
        Ok(())
    }
    
    fn execute_monthly_distribution(&mut self, pool_state: &mut PoolState) -> Result<(), String> {
        let total_revenue = self.calculate_total_monthly_revenue(&pool_state.dev_team_business);
        let net_profit = total_revenue - pool_state.dev_team_business.monthly_operating_costs.max(self.operating_cost_estimate);
        
        if net_profit >= pool_state.dev_team_business.minimum_distribution_threshold {
            // Reserve portion for business growth
            let distributable = net_profit * (1.0 - self.distribution_reserve_ratio);
            let reserve_amount = net_profit * self.distribution_reserve_ratio;
            
            // 50/50 split between developers
            let _per_dev = distributable * 0.5;
            
            // Distribute earnings to all team members based on role
            let total_members = pool_state.dev_team_business.team_hierarchy.senior_managers.len() + 
                               pool_state.dev_team_business.team_hierarchy.operations_managers.len() + 
                               pool_state.dev_team_business.team_hierarchy.tech_managers.len() + 
                               pool_state.dev_team_business.team_hierarchy.developers.len() + 1; // +1 for owner
            
            if total_members > 0 {
                // For monthly profit distribution, assume it's distributed as ICP tokens
                // Convert USD profit to ICP tokens (placeholder conversion rate)
                let icp_tokens = (distributable * 100_000_000.0) as u64; // Convert USD to ICP atomic units
                
                // Use the new multi-token distribution system
                // TODO: Add ICP to Asset enum, using ETH as placeholder for now
                self.distribute_token_earnings(pool_state, crate::types::Asset::ETH, icp_tokens, distributable)?;
            }
            
            // Add to emergency fund
            pool_state.dev_team_business.emergency_fund += reserve_amount;
            
            // Reset monthly counters
            self.reset_monthly_profit_tracking(pool_state);
            
            // Update last distribution time
            pool_state.dev_team_business.last_distribution_time = time();
            
            Ok(())
        } else {
            // Not enough profit to distribute, carry over to next month
            Ok(())
        }
    }
    
    pub fn withdraw_dev_earnings_multi_token(
        &mut self, 
        pool_state: &mut PoolState, 
        caller: Principal, 
        option: crate::types::WithdrawalOption
    ) -> Result<Vec<crate::types::TokenTransfer>, String> {
        // Check if caller is a dev team member
        if !pool_state.dev_team_business.team_member_earnings.contains_key(&caller) {
            return Err("Unauthorized: Only dev team members can withdraw earnings".to_string());
        }
        
        let member_earnings = pool_state.dev_team_business.team_member_earnings
            .get(&caller)
            .cloned()
            .unwrap_or_default();
        
        if member_earnings.balances.is_empty() {
            return Err("No earnings available for withdrawal".to_string());
        }
        
        let mut transfers = Vec::new();
        
        // Process withdrawal based on option
        match option {
            crate::types::WithdrawalOption::OriginalTokens => {
                // Keep all tokens in original form
                for (asset, balance) in &member_earnings.balances {
                    if balance.amount > 0 {
                        transfers.push(crate::types::TokenTransfer {
                            asset: *asset,
                            amount: balance.amount,
                            recipient: caller,
                            transfer_type: crate::types::TransferType::OriginalToken { 
                                chain: self.get_primary_chain_for_asset(*asset) 
                            },
                        });
                    }
                }
            },
            crate::types::WithdrawalOption::ConvertToICP => {
                // Convert all tokens to ICP
                let total_usd_value = member_earnings.total_usd_value;
                if total_usd_value > 0.0 {
                    // TODO: Implement USD to ICP conversion rate lookup
                    let icp_amount = (total_usd_value * 100_000_000.0) as u64; // Placeholder conversion
                    transfers.push(crate::types::TokenTransfer {
                        asset: crate::types::Asset::ETH, // TODO: Add ICP to Asset enum
                        amount: icp_amount,
                        recipient: caller,
                        transfer_type: crate::types::TransferType::ConvertedToICP,
                    });
                }
            },
            crate::types::WithdrawalOption::Mixed { original_tokens, convert_to_icp } => {
                let mut total_conversion_usd = 0.0;
                
                // Process original tokens
                for asset in original_tokens {
                    if let Some(balance) = member_earnings.balances.get(&asset) {
                        if balance.amount > 0 {
                            transfers.push(crate::types::TokenTransfer {
                                asset,
                                amount: balance.amount,
                                recipient: caller,
                                transfer_type: crate::types::TransferType::OriginalToken { 
                                    chain: self.get_primary_chain_for_asset(asset) 
                                },
                            });
                        }
                    }
                }
                
                // Process tokens to convert to ICP
                for asset in convert_to_icp {
                    if let Some(balance) = member_earnings.balances.get(&asset) {
                        total_conversion_usd += balance.usd_value_at_time;
                    }
                }
                
                if total_conversion_usd > 0.0 {
                    let icp_amount = (total_conversion_usd * 100_000_000.0) as u64; // Placeholder conversion
                    transfers.push(crate::types::TokenTransfer {
                        asset: crate::types::Asset::ETH, // TODO: Add ICP to Asset enum
                        amount: icp_amount,
                        recipient: caller,
                        transfer_type: crate::types::TransferType::ConvertedToICP,
                    });
                }
            }
        }
        
        // Reset earnings after successful withdrawal preparation
        pool_state.dev_team_business.team_member_earnings.insert(caller, crate::types::MemberEarnings::default());
        
        // In production, execute actual token transfers here
        Ok(transfers)
    }
    
    fn get_primary_chain_for_asset(&self, asset: crate::types::Asset) -> crate::types::ChainId {
        match asset {
            crate::types::Asset::BTC => crate::types::ChainId::Bitcoin,
            crate::types::Asset::ETH => crate::types::ChainId::Ethereum,
            crate::types::Asset::USDC => crate::types::ChainId::Ethereum, // Default to Ethereum for stablecoins
            crate::types::Asset::USDT => crate::types::ChainId::Ethereum,
            crate::types::Asset::DAI => crate::types::ChainId::Ethereum,
            crate::types::Asset::SOL => crate::types::ChainId::Solana,
            crate::types::Asset::MATIC => crate::types::ChainId::Polygon,
            crate::types::Asset::AVAX => crate::types::ChainId::Avalanche,
        }
    }

    // Legacy function for backward compatibility
    pub fn withdraw_dev_earnings(&mut self, pool_state: &mut PoolState, caller: Principal) -> Result<f64, String> {
        // Default to original tokens withdrawal
        let _transfers = self.withdraw_dev_earnings_multi_token(
            pool_state, 
            caller, 
            crate::types::WithdrawalOption::OriginalTokens
        )?;
        
        // Return total USD value for compatibility
        let total_usd = _transfers.iter()
            .map(|_| 1000.0) // Placeholder USD value per transfer
            .sum();
        Ok(total_usd)
    }
    
    // =============================================================================
    // BUSINESS ANALYTICS
    // =============================================================================
    
    pub fn calculate_total_monthly_revenue(&self, business_model: &DevTeamBusinessModel) -> f64 {
        business_model.monthly_subscription_revenue + 
        business_model.monthly_transaction_fees + 
        business_model.monthly_enterprise_revenue
    }
    
    pub fn get_monthly_profit(&self, business_model: &DevTeamBusinessModel) -> f64 {
        let total_revenue = self.calculate_total_monthly_revenue(business_model);
        total_revenue - business_model.monthly_operating_costs.max(self.operating_cost_estimate)
    }
    
    pub fn get_annual_projection(&self, business_model: &DevTeamBusinessModel) -> AnnualProjection {
        let monthly_profit = self.get_monthly_profit(business_model);
        let annual_profit = monthly_profit * 12.0;
        
        let total_members = business_model.team_hierarchy.senior_managers.len() + 
                           business_model.team_hierarchy.operations_managers.len() + 
                           business_model.team_hierarchy.tech_managers.len() + 
                           business_model.team_hierarchy.developers.len() + 1; // +1 for owner
        
        let distributable_annual = annual_profit * (1.0 - self.distribution_reserve_ratio);
        let per_member_annual = if total_members > 0 { 
            distributable_annual / total_members as f64 
        } else { 
            0.0 
        };
        
        AnnualProjection {
            projected_annual_revenue: self.calculate_total_monthly_revenue(business_model) * 12.0,
            projected_annual_profit: annual_profit,
            projected_dev_1_earnings: per_member_annual, // Owner earnings
            projected_dev_2_earnings: per_member_annual, // Average team member earnings
        }
    }
    
    pub fn assess_business_health(&self, business_model: &DevTeamBusinessModel) -> String {
        let monthly_profit = self.get_monthly_profit(business_model);
        
        if monthly_profit >= 100_000.0 {
            "Excellent".to_string()
        } else if monthly_profit >= 50_000.0 {
            "Very Good".to_string()
        } else if monthly_profit >= 20_000.0 {
            "Good".to_string()
        } else if monthly_profit >= 5_000.0 {
            "Fair".to_string()
        } else if monthly_profit >= 0.0 {
            "Breaking Even".to_string()
        } else {
            "Loss".to_string()
        }
    }
    
    // =============================================================================
    // CONFIGURATION MANAGEMENT
    // =============================================================================
    
    pub fn update_dev_principals(&mut self, _pool_state: &mut PoolState, _dev_1: Principal, _dev_2: Principal) -> Result<(), String> {
        // This function is deprecated - use team hierarchy management instead
        Err("Use team hierarchy management functions instead".to_string())
    }
    
    pub fn update_distribution_settings(&mut self, pool_state: &mut PoolState, threshold: f64, frequency: u64) -> Result<(), String> {
        if threshold < 0.0 {
            return Err("Distribution threshold must be positive".to_string());
        }
        
        if frequency < 86400 { // Less than 1 day
            return Err("Distribution frequency must be at least 1 day".to_string());
        }
        
        pool_state.dev_team_business.minimum_distribution_threshold = threshold;
        pool_state.dev_team_business.distribution_frequency = frequency;
        
        Ok(())
    }
    
    pub fn update_operating_costs(&mut self, pool_state: &mut PoolState, monthly_costs: f64) -> Result<(), String> {
        if monthly_costs < 0.0 {
            return Err("Operating costs cannot be negative".to_string());
        }
        
        pool_state.dev_team_business.monthly_operating_costs = monthly_costs;
        Ok(())
    }
    
    // =============================================================================
    // MULTI-TOKEN EARNINGS MANAGEMENT
    // =============================================================================
    
    pub fn add_token_earnings(&self, pool_state: &mut PoolState, principal: Principal, asset: Asset, amount: u64, usd_value: f64) -> Result<(), String> {
        // SECURITY: Validate inputs
        if amount == 0 {
            return Err("SECURITY: Amount must be greater than zero".to_string());
        }
        if !usd_value.is_finite() || usd_value < 0.0 {
            return Err("SECURITY: Invalid USD value".to_string());
        }
        
        // Get or create member earnings
        let member_earnings = pool_state.dev_team_business.team_member_earnings
            .entry(principal)
            .or_insert_with(|| crate::types::MemberEarnings::default());
        
        // Update token balance
        let token_balance = member_earnings.balances
            .entry(asset)
            .or_insert_with(|| crate::types::TokenBalance {
                asset,
                amount: 0,
                last_updated: ic_cdk::api::time(),
                usd_value_at_time: 0.0,
            });
        
        // Safe addition with overflow protection
        let new_amount = token_balance.amount.saturating_add(amount);
        if new_amount < token_balance.amount {
            return Err("SECURITY: Token balance overflow prevented".to_string());
        }
        
        token_balance.amount = new_amount;
        token_balance.last_updated = ic_cdk::api::time();
        token_balance.usd_value_at_time = usd_value;
        
        // Update total USD value for the member
        self.recalculate_member_usd_value(member_earnings);
        
        Ok(())
    }
    
    fn recalculate_member_usd_value(&self, member_earnings: &mut crate::types::MemberEarnings) {
        member_earnings.total_usd_value = member_earnings.balances
            .values()
            .map(|balance| balance.usd_value_at_time)
            .sum();
    }
    
    pub fn distribute_token_earnings(&self, pool_state: &mut PoolState, asset: Asset, total_amount: u64, usd_value: f64) -> Result<(), String> {
        // Collect all principals first to avoid borrow checker issues
        let mut all_principals = Vec::new();
        
        // Add owner
        all_principals.push(pool_state.dev_team_business.team_hierarchy.owner_principal);
        
        // Add all team members
        all_principals.extend(&pool_state.dev_team_business.team_hierarchy.senior_managers);
        all_principals.extend(&pool_state.dev_team_business.team_hierarchy.operations_managers);
        all_principals.extend(&pool_state.dev_team_business.team_hierarchy.tech_managers);
        all_principals.extend(&pool_state.dev_team_business.team_hierarchy.developers);
        
        let total_members = all_principals.len();
        
        if total_members == 0 {
            return Err("No team members to distribute to".to_string());
        }
        
        let per_member_amount = total_amount / total_members as u64;
        let per_member_usd = usd_value / total_members as f64;
        
        if per_member_amount == 0 {
            return Err("Amount too small to distribute".to_string());
        }
        
        // Distribute to all collected principals
        for principal in all_principals {
            self.add_token_earnings(pool_state, principal, asset, per_member_amount, per_member_usd)?;
        }
        
        Ok(())
    }

    // =============================================================================
    // UTILITY FUNCTIONS
    // =============================================================================
    
    fn reset_monthly_profit_tracking(&self, pool_state: &mut PoolState) {
        pool_state.dev_team_business.monthly_subscription_revenue = 0.0;
        pool_state.dev_team_business.monthly_transaction_fees = 0.0;
        pool_state.dev_team_business.monthly_enterprise_revenue = 0.0;
    }
    
    pub fn get_distribution_schedule(&self, business_model: &DevTeamBusinessModel) -> DistributionSchedule {
        let current_time = time();
        let time_since_last = current_time - business_model.last_distribution_time;
        let time_until_next = if time_since_last >= business_model.distribution_frequency {
            0 // Overdue
        } else {
            business_model.distribution_frequency - time_since_last
        };
        
        DistributionSchedule {
            last_distribution: business_model.last_distribution_time,
            next_distribution: current_time + time_until_next,
            distribution_frequency_days: business_model.distribution_frequency / 86_400_000_000_000u64,
            is_overdue: time_until_next == 0,
        }
    }
}

// =============================================================================
// SUPPORTING TYPES
// =============================================================================

#[derive(Clone, Debug)]
pub struct AnnualProjection {
    pub projected_annual_revenue: f64,
    pub projected_annual_profit: f64,
    pub projected_dev_1_earnings: f64,
    pub projected_dev_2_earnings: f64,
}

#[derive(Clone, Debug)]
pub struct DistributionSchedule {
    pub last_distribution: u64,
    pub next_distribution: u64,
    pub distribution_frequency_days: u64,
    pub is_overdue: bool,
}