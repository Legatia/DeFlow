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
        pool_state.dev_team_business.monthly_transaction_fees += amount;
        Ok(())
    }
    
    pub fn process_subscription_payment(&mut self, pool_state: &mut PoolState, _user: Principal, amount: f64) -> Result<(), String> {
        pool_state.dev_team_business.monthly_subscription_revenue += amount;
        
        // Check for profit distribution after each payment
        self.check_and_execute_profit_distribution(pool_state)?;
        
        Ok(())
    }
    
    pub fn add_enterprise_revenue(&mut self, pool_state: &mut PoolState, amount: f64) -> Result<(), String> {
        pool_state.dev_team_business.monthly_enterprise_revenue += amount;
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
                let per_member = distributable / total_members as f64;
                
                // Owner gets their share
                *pool_state.dev_team_business.team_member_earnings.entry(pool_state.dev_team_business.team_hierarchy.owner_principal).or_insert(0.0) += per_member;
                
                // Distribute to all team members
                for principal in &pool_state.dev_team_business.team_hierarchy.senior_managers {
                    *pool_state.dev_team_business.team_member_earnings.entry(*principal).or_insert(0.0) += per_member;
                }
                for principal in &pool_state.dev_team_business.team_hierarchy.operations_managers {
                    *pool_state.dev_team_business.team_member_earnings.entry(*principal).or_insert(0.0) += per_member;
                }
                for principal in &pool_state.dev_team_business.team_hierarchy.tech_managers {
                    *pool_state.dev_team_business.team_member_earnings.entry(*principal).or_insert(0.0) += per_member;
                }
                for principal in &pool_state.dev_team_business.team_hierarchy.developers {
                    *pool_state.dev_team_business.team_member_earnings.entry(*principal).or_insert(0.0) += per_member;
                }
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
    
    pub fn withdraw_dev_earnings(&mut self, pool_state: &mut PoolState, caller: Principal) -> Result<f64, String> {
        // Check if caller is a dev team member
        if !pool_state.dev_team_business.team_member_earnings.contains_key(&caller) {
            return Err("Unauthorized: Only dev team members can withdraw earnings".to_string());
        }
        
        let earnings = pool_state.dev_team_business.team_member_earnings
            .get(&caller)
            .copied()
            .unwrap_or(0.0);
        
        if earnings > 0.0 {
            // Reset earnings to 0 after withdrawal
            pool_state.dev_team_business.team_member_earnings.insert(caller, 0.0);
            // In production, this would transfer ICP tokens to the dev wallet
            Ok(earnings)
        } else {
            Err("No earnings available for withdrawal".to_string())
        }
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