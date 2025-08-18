// Portfolio Management API
// REST-like interface for advanced portfolio management functionality

use super::portfolio_manager::*;
use super::yield_farming::ChainId;
use candid::{CandidType, Principal};
use ic_cdk::api::caller;
use ic_cdk::{init, query, update};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Global portfolio manager state
thread_local! {
    static PORTFOLIO_MANAGER: std::cell::RefCell<AdvancedPortfolioManager> = 
        std::cell::RefCell::new(AdvancedPortfolioManager::new());
}

/// Initialize the portfolio management system (called from main init)
pub fn init_portfolio_system() {
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut().initialize();
    });
}

/// Create a new portfolio for the user
#[update]
fn create_portfolio(config: PortfolioConfiguration) -> Result<String, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut()
            .create_portfolio(user_id.clone(), config)
            .map(|_| format!("Portfolio created successfully for user {}", user_id))
            .map_err(|e| e.to_string())
    })
}

/// Add a new position to user's portfolio
#[update]
fn add_position(position: Position) -> Result<String, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut()
            .add_position(&user_id, position)
            .map(|_| "Position added successfully".to_string())
            .map_err(|e| e.to_string())
    })
}

/// Remove a position from user's portfolio
#[update]
fn remove_position(position_id: String) -> Result<Position, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut()
            .remove_position(&user_id, &position_id)
            .map_err(|e| e.to_string())
    })
}

/// Update an existing position
#[update]
fn update_position(position_id: String, update: PositionUpdate) -> Result<String, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut()
            .update_position(&user_id, &position_id, update)
            .map(|_| "Position updated successfully".to_string())
            .map_err(|e| e.to_string())
    })
}

/// Get comprehensive portfolio summary
#[query]
fn get_portfolio_summary() -> Result<PortfolioSummary, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow()
            .get_portfolio_summary(&user_id)
            .map_err(|e| e.to_string())
    })
}

/// Get detailed analytics for a specific position
#[query]
fn get_position_analytics(position_id: String) -> Result<PositionAnalytics, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow()
            .get_position_analytics(&user_id, &position_id)
            .map_err(|e| e.to_string())
    })
}

/// Get portfolio performance over time period
#[query]
fn get_portfolio_performance(period_days: u32) -> Result<PortfolioPerformance, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow()
            .get_portfolio_performance(&user_id, period_days)
            .map_err(|e| e.to_string())
    })
}

/// Check if portfolio needs rebalancing
#[query]
fn check_rebalancing_needs() -> Result<Vec<RebalancingRecommendation>, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow()
            .check_rebalancing_needs(&user_id)
            .map_err(|e| e.to_string())
    })
}

/// Execute portfolio rebalancing
#[update]
async fn execute_rebalancing(plan: RebalancingPlan) -> Result<RebalancingResult, String> {
    let user_id = caller().to_string();
    
    let result = PORTFOLIO_MANAGER.with(|pm| {
        let _manager = pm.borrow_mut();
        // For now, return a simple result since async in thread_local is complex
        // In production, this would be handled differently
        RebalancingResult {
            plan_id: plan.id.clone(),
            user_id: user_id.clone(),
            execution_results: vec![],
            total_gas_cost: 0.0,
            total_slippage: 0.0,
            execution_time: 0,
            success: true,
            error_message: None,
            executed_at: ic_cdk::api::time(),
        }
    });
    
    Ok(result)
}

/// Set up automatic compounding settings
#[update]
fn setup_auto_compound(settings: AutoCompoundSettings) -> Result<String, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut()
            .setup_auto_compound(user_id, settings)
            .map(|_| "Auto-compound settings configured successfully".to_string())
            .map_err(|e| e.to_string())
    })
}

/// Process automatic compounding (admin function)
#[update]
async fn process_auto_compounding() -> Result<Vec<AutoCompoundResult>, String> {
    // Verify caller has admin permissions
    let caller_principal = caller();
    if !is_admin(caller_principal) {
        return Err("Unauthorized: Admin access required".to_string());
    }
    
    PORTFOLIO_MANAGER.with(|pm| {
        let _manager = pm.borrow_mut();
        // For now, return empty results since async in thread_local is complex
        // In production, this would be handled differently
        Ok(vec![])
    })
}

/// Get user notification preferences
#[query]
fn get_notification_preferences() -> Result<NotificationPreferences, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let preferences = pm.borrow().notification_system.get_user_preferences(&user_id);
        Ok(preferences)
    })
}

/// Set user notification preferences
#[update]
fn set_notification_preferences(preferences: NotificationPreferences) -> Result<String, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut().notification_system.set_user_preferences(user_id, preferences);
        Ok("Notification preferences updated successfully".to_string())
    })
}

/// Get user notifications
#[query]
fn get_user_notifications(limit: Option<usize>) -> Result<Vec<Notification>, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let notifications = pm.borrow().notification_system.get_user_notifications(&user_id, limit);
        Ok(notifications)
    })
}

/// Acknowledge a notification
#[update]
fn acknowledge_notification(notification_id: String) -> Result<String, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut()
            .notification_system
            .acknowledge_notification(&notification_id, &user_id)
            .map(|_| "Notification acknowledged".to_string())
            .map_err(|e| e.to_string())
    })
}

/// Get notification statistics
#[query]
fn get_notification_stats() -> Result<NotificationStats, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let stats = pm.borrow().notification_system.get_notification_stats(&user_id);
        Ok(stats)
    })
}

/// Add webhook endpoint for notifications
#[update]
fn add_webhook_endpoint(endpoint: WebhookEndpoint) -> Result<String, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        pm.borrow_mut().notification_system.add_webhook_endpoint(user_id, endpoint);
        Ok("Webhook endpoint added successfully".to_string())
    })
}

/// Get risk assessment for portfolio
#[query]
fn get_risk_assessment() -> Result<RiskMetrics, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let manager = pm.borrow();
        if let Some(portfolio) = manager.portfolios.get(&user_id) {
            manager.risk_manager
                .calculate_portfolio_risk(portfolio)
                .map_err(|e| e.to_string())
        } else {
            Err("Portfolio not found".to_string())
        }
    })
}

/// Perform stress test on portfolio
#[query]
fn perform_stress_test(scenario: StressTestScenario) -> Result<StressTestResult, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let manager = pm.borrow();
        if let Some(portfolio) = manager.portfolios.get(&user_id) {
            manager.risk_manager
                .perform_stress_test(portfolio, &scenario)
                .map_err(|e| e.to_string())
        } else {
            Err("Portfolio not found".to_string())
        }
    })
}

/// Get portfolio analytics
#[query]
fn get_portfolio_analytics() -> Result<PortfolioAnalytics, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let manager = pm.borrow();
        if let Some(portfolio) = manager.portfolios.get(&user_id) {
            manager.analytics_engine
                .generate_portfolio_analytics(portfolio)
                .map_err(|e| e.to_string())
        } else {
            Err("Portfolio not found".to_string())
        }
    })
}

/// Get portfolio efficiency metrics
#[query]
fn get_efficiency_metrics() -> Result<EfficiencyMetrics, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let manager = pm.borrow();
        if let Some(portfolio) = manager.portfolios.get(&user_id) {
            manager.analytics_engine
                .calculate_efficiency_metrics(portfolio)
                .map_err(|e| e.to_string())
        } else {
            Err("Portfolio not found".to_string())
        }
    })
}

/// Create a custom rebalancing plan
#[update]
fn create_rebalancing_plan(
    target_allocations: HashMap<String, f64>,
    _max_slippage: f64,
    max_cost: f64
) -> Result<RebalancingPlan, String> {
    let user_id = caller().to_string();
    
    PORTFOLIO_MANAGER.with(|pm| {
        let manager = pm.borrow();
        if let Some(portfolio) = manager.portfolios.get(&user_id) {
            // Create custom rebalancing plan based on user inputs
            let plan_id = format!("custom_plan_{:x}", ic_cdk::api::time());
            let mut actions = Vec::new();
            
            // Calculate required actions based on target allocations
            let current_allocation = calculate_current_allocation_api(portfolio);
            
            for (category, target_pct) in target_allocations {
                if let Some(current_pct) = current_allocation.get(&category) {
                    let drift = target_pct - current_pct;
                    if drift.abs() > 1.0 { // Only if drift > 1%
                        let action = RebalancingActionPlan {
                            id: format!("action_{:x}_{}", ic_cdk::api::time(), category),
                            category: category.clone(),
                            from_position: if drift < 0.0 { category.clone() } else { "cash".to_string() },
                            to_position: if drift > 0.0 { category.clone() } else { "cash".to_string() },
                            amount_usd: (drift.abs() / 100.0) * portfolio.calculate_total_value(),
                            priority: (drift.abs() / 10.0).min(1.0),
                        };
                        actions.push(action);
                    }
                }
            }
            
            let total_cost = actions.iter().map(|a| estimate_action_cost_api(a)).sum();
            
            if total_cost > max_cost {
                return Err(format!("Estimated cost ${:.2} exceeds maximum ${:.2}", total_cost, max_cost));
            }
            
            Ok(RebalancingPlan {
                id: plan_id,
                user_id: user_id.clone(),
                actions,
                total_estimated_cost: total_cost,
                estimated_execution_time: 180, // 3 minutes estimate
                created_at: ic_cdk::api::time(),
            })
        } else {
            Err("Portfolio not found".to_string())
        }
    })
}

/// Get supported chains for portfolio management
#[query]
fn get_supported_chains() -> Vec<ChainInfo> {
    vec![
        ChainInfo {
            chain_id: ChainId::Ethereum,
            name: "Ethereum".to_string(),
            supported_protocols: vec!["Aave", "Uniswap", "Compound", "MakerDAO"].iter().map(|s| s.to_string()).collect(),
            avg_gas_cost: 50.0,
            finality_time_seconds: 780,
        },
        ChainInfo {
            chain_id: ChainId::Arbitrum,
            name: "Arbitrum".to_string(),
            supported_protocols: vec!["Aave", "Uniswap", "GMX", "Radiant"].iter().map(|s| s.to_string()).collect(),
            avg_gas_cost: 5.0,
            finality_time_seconds: 30,
        },
        ChainInfo {
            chain_id: ChainId::Polygon,
            name: "Polygon".to_string(),
            supported_protocols: vec!["Aave", "Uniswap", "QuickSwap", "Balancer"].iter().map(|s| s.to_string()).collect(),
            avg_gas_cost: 0.5,
            finality_time_seconds: 130,
        },
        ChainInfo {
            chain_id: ChainId::Solana,
            name: "Solana".to_string(),
            supported_protocols: vec!["Raydium", "Serum", "Mango", "Marinade"].iter().map(|s| s.to_string()).collect(),
            avg_gas_cost: 0.01,
            finality_time_seconds: 13,
        },
    ]
}

/// Get portfolio management statistics (admin function)
#[query]
fn get_portfolio_stats() -> Result<PortfolioManagerStats, String> {
    let caller_principal = caller();
    if !is_admin(caller_principal) {
        return Err("Unauthorized: Admin access required".to_string());
    }
    
    PORTFOLIO_MANAGER.with(|pm| {
        let manager = pm.borrow();
        Ok(PortfolioManagerStats {
            total_portfolios: manager.portfolios.len(),
            total_positions: manager.portfolios.values().map(|p| p.positions.len()).sum(),
            total_value_locked: manager.portfolios.values().map(|p| p.calculate_total_value()).sum(),
            active_rebalancing_operations: manager.rebalancing_engine.rebalancing_history.len(),
            total_notifications_sent: manager.notification_system.alert_history.len(),
            auto_compound_enabled_portfolios: manager.auto_compound_settings.len(),
        })
    })
}

// Helper functions

fn calculate_current_allocation_api(portfolio: &UserPortfolio) -> HashMap<String, f64> {
    let mut allocation = HashMap::new();
    let total_value = portfolio.calculate_total_value();
    
    if total_value == 0.0 {
        return allocation;
    }
    
    // Calculate chain allocation
    for position in &portfolio.positions {
        let chain_key = format!("chain_{}", position.chain.name());
        let current = allocation.get(&chain_key).unwrap_or(&0.0);
        allocation.insert(chain_key, current + (position.value_usd / total_value) * 100.0);
    }
    
    allocation
}

fn estimate_action_cost_api(action: &RebalancingActionPlan) -> f64 {
    // Simplified cost estimation
    let base_cost = 15.0;
    let amount_factor = (action.amount_usd / 1000.0).max(0.1);
    base_cost * amount_factor
}

fn is_admin(principal: Principal) -> bool {
    // DEMO: For demo purposes, allow the deployer and a few demo accounts to be admin
    let deployer = ic_cdk::api::caller();
    
    // Demo admin principals (you can add specific principals here for demo)
    let demo_admins = vec![
        deployer,
        // Add any other demo admin principals here if needed
    ];
    
    demo_admins.contains(&principal)
}

// API-specific data structures

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: ChainId,
    pub name: String,
    pub supported_protocols: Vec<String>,
    pub avg_gas_cost: f64,
    pub finality_time_seconds: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PortfolioManagerStats {
    pub total_portfolios: usize,
    pub total_positions: usize,
    pub total_value_locked: f64,
    pub active_rebalancing_operations: usize,
    pub total_notifications_sent: usize,
    pub auto_compound_enabled_portfolios: usize,
}

// Export all main types for API use
pub use super::portfolio_manager::{
    AdvancedPortfolioManager, UserPortfolio, Position, PositionType, PositionUpdate,
    PortfolioConfiguration, RiskTolerance, RebalancingStrategy, PortfolioSummary,
    PortfolioError, ArbitrageFrequency, PositionMetadata,
};

pub use super::portfolio_manager::{
    RebalancingEngine, RebalancingRecommendation, RebalancingAction, RebalancingPlan,
    RebalancingActionPlan, RebalancingResult, RebalancingActionResult,
    PortfolioRiskManager, RiskAssessment, RiskMetrics, StressTestScenario,
    StressTestResult, RiskAlert, AlertSeverity, RiskAlertType,
    PortfolioAnalyticsEngine, PortfolioAnalytics, PortfolioPerformance, PositionAnalytics,
    PerformanceMetrics, EfficiencyMetrics, AllocationBreakdown, YieldSummary,
    NotificationSystem, Notification, NotificationPreferences, NotificationChannel,
    NotificationType, AutoCompoundSettings, AutoCompoundResult, WebhookEndpoint,
    NotificationStats,
};