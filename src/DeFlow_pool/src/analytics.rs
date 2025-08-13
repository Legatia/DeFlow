use crate::types::*;
use std::collections::HashMap;

pub struct PoolAnalytics {
    // Analytics configuration
    pub metrics_retention_days: u64,
    pub health_check_thresholds: HealthThresholds,
}

impl PoolAnalytics {
    pub fn new() -> Self {
        PoolAnalytics {
            metrics_retention_days: 90, // Keep 90 days of metrics
            health_check_thresholds: HealthThresholds {
                excellent_liquidity_threshold: 1_000_000.0,  // $1M+
                good_liquidity_threshold: 500_000.0,         // $500K+
                fair_liquidity_threshold: 100_000.0,         // $100K+
                min_bootstrap_progress: 0.8,                 // 80% minimum
                max_utilization_ratio: 0.85,                 // 85% max utilization
            },
        }
    }
    
    // =============================================================================
    // FINANCIAL OVERVIEW
    // =============================================================================
    
    pub fn get_financial_overview(&self, pool_state: &PoolState) -> Result<FinancialOverview, String> {
        Ok(FinancialOverview {
            // Pool metrics
            total_liquidity: pool_state.total_liquidity_usd,
            monthly_pool_growth: self.calculate_monthly_pool_growth(pool_state),
            bootstrap_progress: self.calculate_bootstrap_progress(pool_state),
            
            // Business metrics
            monthly_revenue: self.calculate_monthly_revenue(&pool_state.dev_team_business),
            dev_1_pending: pool_state.dev_team_business.team_member_earnings.get(&pool_state.dev_team_business.team_hierarchy.owner_principal).copied().unwrap_or(0.0),
            dev_2_pending: pool_state.dev_team_business.total_distributed_profits, // Show total distributed to all team members
            emergency_fund: pool_state.dev_team_business.emergency_fund,
            
            // Health indicators
            pool_health: self.assess_pool_health(pool_state),
            business_health: self.assess_business_health(&pool_state.dev_team_business),
        })
    }
    
    // =============================================================================
    // POOL ANALYTICS
    // =============================================================================
    
    pub fn calculate_monthly_pool_growth(&self, pool_state: &PoolState) -> f64 {
        // Calculate total fee contributions in the current period
        let mut _total_fee_growth = 0.0;
        
        for (_chain_id, chain_reserves) in &pool_state.reserves {
            for (_asset, reserve) in chain_reserves {
                _total_fee_growth += reserve.fee_contributed_amount as f64 * self.estimate_usd_conversion_rate(&_asset);
            }
        }
        
        // Estimate monthly growth based on daily growth rates
        let mut estimated_monthly_growth = 0.0;
        for (_chain_id, chain_reserves) in &pool_state.reserves {
            for (_asset, reserve) in chain_reserves {
                let asset_value = reserve.total_amount as f64 * self.estimate_usd_conversion_rate(&_asset);
                estimated_monthly_growth += asset_value * reserve.daily_growth_rate * 30.0; // 30 days
            }
        }
        
        estimated_monthly_growth
    }
    
    pub fn calculate_bootstrap_progress(&self, pool_state: &PoolState) -> f64 {
        if pool_state.bootstrap_targets.is_empty() {
            return 1.0; // 100% if no targets
        }
        
        let total_progress: f64 = pool_state.bootstrap_targets.iter()
            .map(|(asset, target)| {
                let current = self.get_total_asset_amount(pool_state, asset);
                (current as f64 / *target as f64).min(1.0)
            })
            .sum();
        
        total_progress / pool_state.bootstrap_targets.len() as f64
    }
    
    pub fn get_chain_distribution(&self, pool_state: &PoolState) -> Vec<(ChainId, f64)> {
        let mut distribution = Vec::new();
        
        for (chain_id, chain_reserves) in &pool_state.reserves {
            let mut chain_value = 0.0;
            
            for (asset, reserve) in chain_reserves {
                let asset_usd_value = reserve.total_amount as f64 * self.estimate_usd_conversion_rate(asset);
                chain_value += asset_usd_value;
            }
            
            if chain_value > 0.0 {
                distribution.push((chain_id.clone(), chain_value));
            }
        }
        
        // Sort by value, descending
        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        distribution
    }
    
    pub fn get_asset_distribution(&self, pool_state: &PoolState) -> Vec<(Asset, f64, f64)> {
        let mut asset_totals: HashMap<Asset, f64> = HashMap::new();
        
        // Calculate total value per asset across all chains
        for (_chain_id, chain_reserves) in &pool_state.reserves {
            for (asset, reserve) in chain_reserves {
                let asset_usd_value = reserve.total_amount as f64 * self.estimate_usd_conversion_rate(asset);
                *asset_totals.entry(asset.clone()).or_insert(0.0) += asset_usd_value;
            }
        }
        
        // Calculate percentages
        let total_value: f64 = asset_totals.values().sum();
        let mut distribution: Vec<(Asset, f64, f64)> = asset_totals.into_iter()
            .map(|(asset, value)| {
                let percentage = if total_value > 0.0 { value / total_value * 100.0 } else { 0.0 };
                (asset, value, percentage)
            })
            .collect();
        
        // Sort by value, descending
        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        distribution
    }
    
    // =============================================================================
    // BUSINESS ANALYTICS
    // =============================================================================
    
    pub fn calculate_monthly_revenue(&self, business_model: &DevTeamBusinessModel) -> f64 {
        business_model.monthly_subscription_revenue + 
        business_model.monthly_transaction_fees + 
        business_model.monthly_enterprise_revenue
    }
    
    pub fn calculate_monthly_profit(&self, business_model: &DevTeamBusinessModel) -> f64 {
        let total_revenue = self.calculate_monthly_revenue(business_model);
        total_revenue - business_model.monthly_operating_costs
    }
    
    pub fn get_revenue_breakdown(&self, business_model: &DevTeamBusinessModel) -> RevenueBreakdown {
        let total_revenue = self.calculate_monthly_revenue(business_model);
        
        RevenueBreakdown {
            subscription_revenue: business_model.monthly_subscription_revenue,
            subscription_percentage: if total_revenue > 0.0 { 
                business_model.monthly_subscription_revenue / total_revenue * 100.0 
            } else { 0.0 },
            
            transaction_fee_revenue: business_model.monthly_transaction_fees,
            transaction_fee_percentage: if total_revenue > 0.0 { 
                business_model.monthly_transaction_fees / total_revenue * 100.0 
            } else { 0.0 },
            
            enterprise_revenue: business_model.monthly_enterprise_revenue,
            enterprise_percentage: if total_revenue > 0.0 { 
                business_model.monthly_enterprise_revenue / total_revenue * 100.0 
            } else { 0.0 },
            
            total_revenue,
        }
    }
    
    // =============================================================================
    // HEALTH ASSESSMENT
    // =============================================================================
    
    pub fn assess_pool_health(&self, pool_state: &PoolState) -> String {
        let total_liquidity = pool_state.total_liquidity_usd;
        let bootstrap_progress = self.calculate_bootstrap_progress(pool_state);
        let utilization = self.calculate_average_utilization(pool_state);
        
        // Check emergency conditions first
        if let PoolPhase::Emergency { .. } = pool_state.phase {
            return "Emergency".to_string();
        }
        
        // Assess based on phase and metrics
        match pool_state.phase {
            PoolPhase::Bootstrapping { .. } => {
                if bootstrap_progress >= 0.9 {
                    "Ready for Activation".to_string()
                } else if bootstrap_progress >= self.health_check_thresholds.min_bootstrap_progress {
                    "Good Progress".to_string()
                } else if bootstrap_progress >= 0.5 {
                    "Fair Progress".to_string()
                } else {
                    "Early Stage".to_string()
                }
            },
            PoolPhase::Active { .. } => {
                if total_liquidity >= self.health_check_thresholds.excellent_liquidity_threshold && utilization < 0.7 {
                    "Excellent".to_string()
                } else if total_liquidity >= self.health_check_thresholds.good_liquidity_threshold && utilization < 0.8 {
                    "Good".to_string()
                } else if total_liquidity >= self.health_check_thresholds.fair_liquidity_threshold && utilization < self.health_check_thresholds.max_utilization_ratio {
                    "Fair".to_string()
                } else {
                    "Stressed".to_string()
                }
            },
            PoolPhase::Emergency { .. } => "Emergency".to_string(),
        }
    }
    
    pub fn assess_business_health(&self, business_model: &DevTeamBusinessModel) -> String {
        let monthly_profit = self.calculate_monthly_profit(business_model);
        let total_pending: f64 = business_model.team_member_earnings.values().sum();
        
        if monthly_profit >= 100_000.0 {
            "Excellent".to_string()
        } else if monthly_profit >= 50_000.0 && total_pending < 200_000.0 {
            "Very Good".to_string()
        } else if monthly_profit >= 20_000.0 && total_pending < 100_000.0 {
            "Good".to_string()
        } else if monthly_profit >= 5_000.0 {
            "Fair".to_string()
        } else if monthly_profit >= 0.0 {
            "Breaking Even".to_string()
        } else {
            "Needs Attention".to_string()
        }
    }
    
    // =============================================================================
    // REPORTING
    // =============================================================================
    
    pub fn generate_analytics_report(&self, pool_state: &PoolState) -> String {
        let financial_overview = self.get_financial_overview(pool_state).unwrap();
        let chain_distribution = self.get_chain_distribution(pool_state);
        let asset_distribution = self.get_asset_distribution(pool_state);
        let revenue_breakdown = self.get_revenue_breakdown(&pool_state.dev_team_business);
        
        format!(
            r#"
=== DeFlow Pool Analytics Report ===

POOL OVERVIEW:
- Phase: {:?}
- Total Liquidity: ${:.2}
- Pool Health: {}
- Bootstrap Progress: {:.1}%

BUSINESS METRICS:
- Monthly Revenue: ${:.2}
- Business Health: {}
- Dev 1 Pending: ${:.2}
- Dev 2 Pending: ${:.2}
- Emergency Fund: ${:.2}

CHAIN DISTRIBUTION:
{}

ASSET DISTRIBUTION:
{}

REVENUE BREAKDOWN:
- Subscriptions: ${:.2} ({:.1}%)
- Transaction Fees: ${:.2} ({:.1}%)
- Enterprise: ${:.2} ({:.1}%)
- Total: ${:.2}

Generated at: {}
            "#,
            pool_state.phase,
            financial_overview.total_liquidity,
            financial_overview.pool_health,
            financial_overview.bootstrap_progress * 100.0,
            
            financial_overview.monthly_revenue,
            financial_overview.business_health,
            financial_overview.dev_1_pending,
            financial_overview.dev_2_pending,
            financial_overview.emergency_fund,
            
            chain_distribution.iter()
                .map(|(chain, value)| format!("  {:?}: ${:.2}", chain, value))
                .collect::<Vec<_>>()
                .join("\n"),
                
            asset_distribution.iter()
                .map(|(asset, value, percentage)| format!("  {:?}: ${:.2} ({:.1}%)", asset, value, percentage))
                .collect::<Vec<_>>()
                .join("\n"),
                
            revenue_breakdown.subscription_revenue,
            revenue_breakdown.subscription_percentage,
            revenue_breakdown.transaction_fee_revenue,
            revenue_breakdown.transaction_fee_percentage,
            revenue_breakdown.enterprise_revenue,
            revenue_breakdown.enterprise_percentage,
            revenue_breakdown.total_revenue,
            
            ic_cdk::api::time()
        )
    }
    
    // =============================================================================
    // UTILITY FUNCTIONS
    // =============================================================================
    
    fn get_total_asset_amount(&self, pool_state: &PoolState, asset: &Asset) -> u64 {
        pool_state.reserves.values()
            .flat_map(|chain_reserves| chain_reserves.get(asset))
            .map(|reserve| reserve.total_amount)
            .sum()
    }
    
    fn calculate_average_utilization(&self, pool_state: &PoolState) -> f64 {
        let mut total_utilization = 0.0;
        let mut count = 0;
        
        for (_chain_id, chain_reserves) in &pool_state.reserves {
            for (_asset, reserve) in chain_reserves {
                total_utilization += reserve.utilization_rate;
                count += 1;
            }
        }
        
        if count > 0 {
            total_utilization / count as f64
        } else {
            0.0
        }
    }
    
    fn estimate_usd_conversion_rate(&self, asset: &Asset) -> f64 {
        // Simplified USD conversion - in production would use price oracles
        let (price_usd, decimals) = match asset {
            Asset::BTC => (45000.0, 8),
            Asset::ETH => (2500.0, 18),
            Asset::USDC => (1.0, 6),
            Asset::USDT => (1.0, 6),
            Asset::DAI => (1.0, 18),
            Asset::SOL => (100.0, 9),
            Asset::MATIC => (0.8, 18),
            Asset::AVAX => (25.0, 18),
            Asset::BNB => (300.0, 18),
        };
        
        price_usd / 10_u64.pow(decimals) as f64
    }
}

// =============================================================================
// SUPPORTING TYPES
// =============================================================================

#[derive(Clone, Debug)]
pub struct HealthThresholds {
    pub excellent_liquidity_threshold: f64,
    pub good_liquidity_threshold: f64,
    pub fair_liquidity_threshold: f64,
    pub min_bootstrap_progress: f64,
    pub max_utilization_ratio: f64,
}

#[derive(Clone, Debug)]
pub struct RevenueBreakdown {
    pub subscription_revenue: f64,
    pub subscription_percentage: f64,
    pub transaction_fee_revenue: f64,
    pub transaction_fee_percentage: f64,
    pub enterprise_revenue: f64,
    pub enterprise_percentage: f64,
    pub total_revenue: f64,
}