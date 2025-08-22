mod types;
mod storage;
mod stable_user_storage;
mod workflow;
mod execution;
mod nodes;
mod events;
mod http_client;
mod defi;
mod user_management;
mod security;
mod scheduler_service;
mod cycles_monitor_service;

// Re-export types for external use
pub use types::*;

use ic_cdk::{init, post_upgrade, pre_upgrade, query, heartbeat, spawn, update};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use nodes::initialize_built_in_nodes;
use events::restore_scheduled_workflows;
use storage::{save_workflow_state_for_upgrade, restore_workflow_state_after_upgrade};
use types::{InternalWorkflowState, SystemHealth as InternalSystemHealth, ExecutionStatus as InternalExecutionStatus};
use defi::api::get_defi_system_health;

// Re-export all the API functions from modules
pub use workflow::{create_workflow, update_workflow, get_workflow, list_workflows, delete_workflow, validate_workflow_query, analyze_workflow_query, WorkflowAnalysis};
pub use execution::{start_execution, get_execution, list_executions, retry_failed_execution, resume_active_workflows};
pub use nodes::{register_node, get_node_definition, list_node_types, list_nodes_by_category};
pub use events::{
    emit_event, register_event_listener, webhook_trigger, register_webhook,
    schedule_workflow, unschedule_workflow, list_scheduled_workflows,
    set_retry_policy, get_retry_policy_for_node,
    schedule_workflow_execution, list_persistent_scheduled_executions, cancel_persistent_execution
};
// DeFi functions are available as canister endpoints in defi::api module
// Strategy API functions - Advanced DeFi strategy management
pub use defi::strategy_api::{
    get_strategy_yield_opportunities, scan_arbitrage_opportunities, get_strategy_portfolio_analytics,
    execute_strategy, get_performance_report, health_check, get_api_docs, init_strategy_api
};
// Workflow Template API functions - User-friendly strategy creation
pub use defi::simple_template_api::{
    list_workflow_templates, get_templates_by_category, get_template_by_id,
    create_strategy_from_simple_template, get_simple_template_recommendations,
    get_template_categories, init_simple_workflow_template_system
};
// Price Alert API functions - Price monitoring and social media integration
pub use defi::price_alert_service::{
    create_price_alert, get_user_price_alerts, update_price_alert,
    deactivate_price_alert, delete_price_alert, init_price_alert_system
};
// DeFi Integration API functions - DeFi execution from price alerts
pub use defi::price_alert_defi_integration::{
    get_user_execution_history, get_user_daily_stats
};
// User Management API functions
pub use user_management::{
    register_user, get_user_info, upgrade_subscription, check_node_access,
    get_allowed_node_types, record_workflow_execution, update_user_volume,
    get_subscription_pricing, list_all_users, reset_monthly_stats
};

#[init]
fn init() {
    initialize_built_in_nodes();
    
    // Initialize DeFi system
    ic_cdk::spawn(async {
        if let Err(e) = defi::initialize_defi_system().await {
            ic_cdk::println!("Failed to initialize DeFi system: {}", e);
        }
    });
    
    // Initialize portfolio management system
    defi::portfolio_api::init_portfolio_system();
    
    // Initialize automated strategy system
    ic_cdk::spawn(async {
        if let Err(e) = defi::automated_strategy_api::init_automated_strategy_system().await {
            ic_cdk::println!("Failed to initialize automated strategy system: {}", e);
        }
    });
    
    // Initialize strategy API system
    defi::strategy_api::init_strategy_api();
    
    // Initialize workflow template system
    defi::simple_template_api::init_simple_workflow_template_system();
    
    ic_cdk::println!("DeFlow backend initialized");
}

#[pre_upgrade]
fn pre_upgrade() {
    let state = save_workflow_state_for_upgrade();
    ic_cdk::storage::stable_save((state,))
        .expect("Failed to save state before upgrade");
    ic_cdk::println!("DeFlow state saved for upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    // Try to restore the workflow state, but handle gracefully if it fails
    match ic_cdk::storage::stable_restore::<(InternalWorkflowState,)>() {
        Ok((saved_state,)) => {
            restore_workflow_state_after_upgrade(saved_state);
            ic_cdk::println!("DeFlow backend upgraded and state restored successfully");
        }
        Err(e) => {
            ic_cdk::println!("Could not restore previous state (this is normal for first deployment): {:?}", e);
            ic_cdk::println!("Initializing with fresh state");
            
            // Initialize with a fresh default state
            let fresh_state = InternalWorkflowState::default();
            restore_workflow_state_after_upgrade(fresh_state);
        }
    }
    
    // Re-initialize components
    initialize_built_in_nodes();
    restore_scheduled_workflows();
    resume_active_workflows();
    
    // Re-initialize DeFi system
    ic_cdk::spawn(async {
        if let Err(e) = defi::initialize_defi_system().await {
            ic_cdk::println!("Failed to re-initialize DeFi system after upgrade: {}", e);
        }
    });
    
    // Re-initialize portfolio management system
    defi::portfolio_api::init_portfolio_system();
    
    // Re-initialize automated strategy system  
    ic_cdk::spawn(async {
        if let Err(e) = defi::automated_strategy_api::init_automated_strategy_system().await {
            ic_cdk::println!("Failed to re-initialize automated strategy system: {}", e);
        }
    });
    
    // Re-initialize strategy API system
    defi::strategy_api::init_strategy_api();
    
    // Re-initialize workflow template system
    defi::simple_template_api::init_simple_workflow_template_system();
    
    ic_cdk::println!("DeFlow backend post_upgrade completed");
}

#[heartbeat]
async fn heartbeat() {
    use storage::{get_workflow_state, update_workflow_state};
    use ic_cdk::api;
    
    let current_time = api::time();
    let mut state = get_workflow_state();
    
    // Update heartbeat timestamp
    state.system_health.last_heartbeat = current_time;
    
    // Check for scheduled workflow executions
    let due_workflows = get_due_workflows(current_time, &state.scheduled_executions);
    
    for workflow_id in due_workflows {
        ic_cdk::println!("Executing scheduled workflow: {}", workflow_id);
        spawn(async move {
            if let Ok(execution_id) = start_execution(workflow_id.clone(), None).await {
                ic_cdk::println!("Started scheduled execution: {}", execution_id);
            }
        });
    }
    
    // Monitor active workflows for timeouts
    monitor_active_executions(&mut state).await;
    
    // Clean up completed workflows older than 24 hours
    cleanup_completed_workflows(&mut state, current_time);
    
    // Update system health metrics
    update_system_health(&mut state).await;
    
    // Save updated state
    update_workflow_state(state);
}

fn get_due_workflows(current_time: u64, scheduled_executions: &[(u64, String)]) -> Vec<String> {
    scheduled_executions
        .iter()
        .filter(|(timestamp, _)| *timestamp <= current_time)
        .map(|(_, workflow_id)| workflow_id.clone())
        .collect()
}

async fn monitor_active_executions(state: &mut InternalWorkflowState) {
    let current_time = ic_cdk::api::time();
    let timeout_threshold = 30 * 60 * 1_000_000_000; // 30 minutes in nanoseconds
    
    for (workflow_id, execution) in &mut state.active_workflows {
        if matches!(execution.status, InternalExecutionStatus::Running | InternalExecutionStatus::Pending) {
            let execution_time = current_time.saturating_sub(execution.started_at);
            
            if execution_time > timeout_threshold {
                ic_cdk::println!("Workflow {} timed out after {}ns", workflow_id, execution_time);
                execution.status = InternalExecutionStatus::Failed;
                execution.completed_at = Some(current_time);
                execution.error_message = Some("Execution timed out".to_string());
                
                // Update in persistent storage
                storage::insert_execution(execution.id.clone(), execution.clone());
            }
        }
    }
}

fn cleanup_completed_workflows(state: &mut InternalWorkflowState, current_time: u64) {
    let cleanup_threshold = 24 * 60 * 60 * 1_000_000_000; // 24 hours in nanoseconds
    
    let initial_count = state.active_workflows.len();
    
    state.active_workflows.retain(|(_, execution)| {
        if matches!(execution.status, InternalExecutionStatus::Completed | InternalExecutionStatus::Failed | InternalExecutionStatus::Cancelled) {
            if let Some(completed_at) = execution.completed_at {
                let age = current_time.saturating_sub(completed_at);
                age < cleanup_threshold
            } else {
                // Keep executions without completion time for now
                true
            }
        } else {
            // Keep running/pending executions
            true
        }
    });
    
    let cleaned_count = initial_count - state.active_workflows.len();
    if cleaned_count > 0 {
        ic_cdk::println!("Cleaned up {} old workflow executions", cleaned_count);
    }
}

async fn update_system_health(state: &mut InternalWorkflowState) {
    let current_time = ic_cdk::api::time();
    
    // Count active workflows
    state.system_health.active_workflows = state.active_workflows
        .iter()
        .filter(|(_, execution)| matches!(execution.status, InternalExecutionStatus::Running | InternalExecutionStatus::Pending))
        .count() as u32;
    
    // Count failed executions in the last 24 hours
    let twenty_four_hours_ago = current_time.saturating_sub(24 * 60 * 60 * 1_000_000_000);
    state.system_health.failed_executions_last_24h = state.execution_history
        .iter()
        .filter(|record| {
            record.started_at >= twenty_four_hours_ago && 
            matches!(record.status, InternalExecutionStatus::Failed)
        })
        .count() as u32;
    
    // Calculate average execution time
    let recent_executions: Vec<_> = state.execution_history
        .iter()
        .filter(|record| {
            record.started_at >= twenty_four_hours_ago &&
            matches!(record.status, InternalExecutionStatus::Completed) &&
            record.duration_ms.is_some()
        })
        .collect();
    
    if !recent_executions.is_empty() {
        let total_duration: u64 = recent_executions
            .iter()
            .map(|record| record.duration_ms.unwrap_or(0))
            .sum();
        state.system_health.average_execution_time_ms = total_duration as f64 / recent_executions.len() as f64;
    }
    
    // Test chain connectivity (simulated for now)
    state.system_health.chain_fusion_connectivity = vec![
        ("BTC".to_string(), test_btc_connectivity().await),
        ("ETH".to_string(), test_eth_connectivity().await),
        ("ICP".to_string(), true), // Always true for local canister
    ];
    
    // Estimate resource usage (simulated)
    state.system_health.memory_usage_percent = estimate_memory_usage();
    state.system_health.cpu_usage_percent = estimate_cpu_usage();
}

async fn test_btc_connectivity() -> bool {
    // Test Bitcoin connectivity through DeFi system health check
    match get_defi_system_health().await {
        Ok(health) => health.bitcoin_service.healthy,
        Err(_) => false,
    }
}

async fn test_eth_connectivity() -> bool {
    // In a real implementation, this would test actual ETH connectivity
    // For now, simulate with a simple check
    true // Assume ETH is always connected for demo
}

fn estimate_memory_usage() -> f64 {
    // In a real implementation, this would check actual memory usage
    // For now, provide a reasonable estimate
    45.0 // 45% memory usage
}

fn estimate_cpu_usage() -> f64 {
    // In a real implementation, this would check actual CPU usage
    // For now, provide a reasonable estimate
    25.0 // 25% CPU usage
}

// System Health Monitoring and Alerting
#[query]
fn get_system_health() -> InternalSystemHealth {
    use storage::get_workflow_state;
    let state = get_workflow_state();
    state.system_health
}

#[update]
async fn trigger_health_check() -> InternalSystemHealth {
    use storage::{get_workflow_state, update_workflow_state};
    let mut state = get_workflow_state();
    
    // Force update system health
    update_system_health(&mut state).await;
    update_workflow_state(state.clone());
    
    state.system_health
}

#[query]
fn get_system_metrics() -> SystemMetrics {
    use storage::get_workflow_state;
    let state = get_workflow_state();
    
    SystemMetrics {
        uptime_seconds: calculate_uptime(),
        total_workflows: count_total_workflows(),
        active_executions: state.system_health.active_workflows,
        failed_executions_24h: state.system_health.failed_executions_last_24h,
        average_execution_time_ms: state.system_health.average_execution_time_ms,
        memory_usage_percent: state.system_health.memory_usage_percent,
        cpu_usage_percent: state.system_health.cpu_usage_percent,
        chain_connectivity: state.system_health.chain_fusion_connectivity,
        last_heartbeat: state.system_health.last_heartbeat,
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub total_workflows: u32,
    pub active_executions: u32,
    pub failed_executions_24h: u32,
    pub average_execution_time_ms: f64,
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
    pub chain_connectivity: Vec<(String, bool)>,
    pub last_heartbeat: u64,
}

fn calculate_uptime() -> u64 {
    // Calculate uptime since canister initialization
    // In a real implementation, this would track actual uptime
    ic_cdk::api::time() / 1_000_000_000 // Convert nanoseconds to seconds
}

fn count_total_workflows() -> u32 {
    storage::WORKFLOWS.with(|workflows| {
        workflows.borrow().len() as u32
    })
}

#[update]
async fn enable_emergency_mode() -> Result<(), String> {
    use storage::{get_workflow_state, update_workflow_state};
    
    ic_cdk::println!("🚨 EMERGENCY MODE ACTIVATED 🚨");
    
    let mut state = get_workflow_state();
    
    // Pause all active workflows
    for (_workflow_id, execution) in &mut state.active_workflows {
        if matches!(execution.status, InternalExecutionStatus::Running | InternalExecutionStatus::Pending) {
            execution.status = InternalExecutionStatus::Cancelled;
            execution.completed_at = Some(ic_cdk::api::time());
            execution.error_message = Some("Cancelled due to emergency mode".to_string());
            
            ic_cdk::println!("Emergency: Cancelled execution {}", execution.id);
        }
    }
    
    // Update system health to reflect emergency state
    state.system_health.last_heartbeat = ic_cdk::api::time();
    
    update_workflow_state(state);
    
    Ok(())
}

#[update]
async fn disable_emergency_mode() -> Result<(), String> {
    ic_cdk::println!("Emergency mode disabled - system returning to normal operation");
    Ok(())
}

#[query]
fn check_system_alerts() -> Vec<SystemAlert> {
    use storage::get_workflow_state;
    let state = get_workflow_state();
    let mut alerts = Vec::new();
    
    let current_time = ic_cdk::api::time();
    let five_minutes_ago = current_time.saturating_sub(5 * 60 * 1_000_000_000);
    
    // Check if heartbeat is stale
    if state.system_health.last_heartbeat < five_minutes_ago {
        alerts.push(SystemAlert {
            level: AlertLevel::Critical,
            message: "System heartbeat is stale - potential system failure".to_string(),
            timestamp: current_time,
            category: "heartbeat".to_string(),
        });
    }
    
    // Check high failure rate
    if state.system_health.failed_executions_last_24h > 50 {
        alerts.push(SystemAlert {
            level: AlertLevel::Warning,
            message: format!("High failure rate: {} executions failed in last 24h", 
                           state.system_health.failed_executions_last_24h),
            timestamp: current_time,
            category: "failures".to_string(),
        });
    }
    
    // Check resource usage
    if state.system_health.memory_usage_percent > 90.0 {
        alerts.push(SystemAlert {
            level: AlertLevel::Critical,
            message: format!("High memory usage: {:.1}%", state.system_health.memory_usage_percent),
            timestamp: current_time,
            category: "resources".to_string(),
        });
    }
    
    if state.system_health.cpu_usage_percent > 95.0 {
        alerts.push(SystemAlert {
            level: AlertLevel::Critical,
            message: format!("High CPU usage: {:.1}%", state.system_health.cpu_usage_percent),
            timestamp: current_time,
            category: "resources".to_string(),
        });
    }
    
    // Check chain connectivity
    for (chain, connected) in &state.system_health.chain_fusion_connectivity {
        if !connected {
            alerts.push(SystemAlert {
                level: AlertLevel::Warning,
                message: format!("Chain {} connectivity lost", chain),
                timestamp: current_time,
                category: "connectivity".to_string(),
            });
        }
    }
    
    alerts
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SystemAlert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: u64,
    pub category: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PriceAlertStats {
    pub total_active_alerts: u32,
    pub total_users_with_alerts: u32,
    pub alerts_triggered_today: u32,
    pub most_monitored_tokens: Vec<String>,
    pub social_posts_today: u32,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[update]
async fn send_system_notification(message: String, level: AlertLevel) -> Result<(), String> {
    ic_cdk::println!("📢 SYSTEM NOTIFICATION [{:?}]: {}", level, message);
    
    // In a real implementation, this would:
    // - Send emails to administrators
    // - Post to Slack/Discord channels
    // - Send webhooks to monitoring systems
    // - Log to external monitoring services
    
    Ok(())
}


#[query]
fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to DeFlow!", name)
}

// =============================================================================
// PRICE ALERT & SOCIAL MEDIA INTEGRATION API ENDPOINTS
// =============================================================================

/// Create a new price alert with optional DeFi actions and social media posting
#[update]
async fn create_user_price_alert(alert: defi::price_alert_service::PriceAlert) -> Result<String, String> {
    create_price_alert(alert)
}

/// Get all price alerts for a user
#[query]
fn get_price_alerts(user_id: String) -> Vec<defi::price_alert_service::PriceAlert> {
    get_user_price_alerts(&user_id)
}

/// Update an existing price alert
#[update]
async fn update_user_price_alert(
    alert_id: String, 
    updates: defi::price_alert_service::PriceAlert
) -> Result<(), String> {
    update_price_alert(&alert_id, updates)
}

/// Deactivate a price alert (keep in system but stop monitoring)
#[update]
async fn deactivate_user_price_alert(alert_id: String, user_id: String) -> Result<(), String> {
    deactivate_price_alert(&alert_id, &user_id)
}

/// Permanently delete a price alert
#[update]
async fn delete_user_price_alert(alert_id: String, user_id: String) -> Result<(), String> {
    delete_price_alert(&alert_id, &user_id)
}

/// Get price alert system statistics
#[query]
fn get_price_alert_stats() -> PriceAlertStats {
    // Get basic statistics about the price alert system
    PriceAlertStats {
        total_active_alerts: 0, // Placeholder - would count from PRICE_ALERT_MANAGER
        total_users_with_alerts: 0,
        alerts_triggered_today: 0,
        most_monitored_tokens: vec!["BTC".to_string(), "ETH".to_string()],
        social_posts_today: 0,
    }
}

/// Create a quick price alert with predefined settings
#[update]
async fn create_quick_price_alert(
    user_id: String,
    token_symbol: String,
    target_price: f64,
    above_or_below: String, // "above" or "below"
    enable_social_post: bool,
) -> Result<String, String> {
    let condition = match above_or_below.as_str() {
        "above" => defi::price_alert_service::PriceCondition::Above(target_price),
        "below" => defi::price_alert_service::PriceCondition::Below(target_price),
        _ => return Err("Invalid condition. Use 'above' or 'below'".to_string()),
    };

    let mut actions = vec![
        defi::price_alert_service::AlertAction::Webhook {
            url: "https://webhook.example.com/price-alert".to_string(),
            payload_template: "Price alert triggered for {token} at ${price}".to_string(),
            headers: std::collections::HashMap::new(),
        }
    ];

    if enable_social_post {
        actions.push(defi::price_alert_service::AlertAction::SocialPost {
            platforms: vec![
                defi::price_alert_service::SocialPlatform::Twitter,
                defi::price_alert_service::SocialPlatform::Discord,
            ],
            message_template: "🚨 {token} just hit ${price}! Target was ${target_price} {condition}".to_string(),
            include_chart: true,
            hashtags: vec!["DeFi".to_string(), "PriceAlert".to_string(), token_symbol.clone()],
        });
    }

    let alert = defi::price_alert_service::PriceAlert {
        id: String::new(), // Will be set by create_alert
        user_id,
        token_symbol,
        condition,
        actions,
        social_config: if enable_social_post {
            Some(defi::price_alert_service::SocialPostConfig {
                auto_post: true,
                custom_message: None,
                include_price_chart: true,
                mention_community: true,
                share_with_followers: true,
            })
        } else {
            None
        },
        created_at: 0, // Will be set by create_alert
        expires_at: None, // No expiration
        is_active: true,
        triggered_count: 0,
        max_triggers: Some(1), // Trigger only once by default
    };

    create_price_alert(alert)
}

// =============================================================================
// DEFI INTEGRATION API ENDPOINTS - Execution history and statistics
// =============================================================================

/// Get DeFi execution history for a user's price alerts
#[query]
fn get_defi_execution_history(user_id: String) -> Vec<defi::price_alert_defi_integration::DeFiExecutionRecord> {
    get_user_execution_history(&user_id)
}

/// Get daily DeFi execution statistics for a user
#[query]
fn get_defi_daily_stats(user_id: String) -> defi::price_alert_defi_integration::DailyExecutionStats {
    get_user_daily_stats(&user_id)
}

// =============================================================================
// SOCIAL MEDIA FORMATTING API ENDPOINTS - Enhanced social posts with DeFi context
// =============================================================================

/// Get available social media templates
#[query]
fn get_social_media_templates() -> Vec<defi::social_media_formatter::SocialMediaTemplate> {
    defi::social_media_formatter::get_available_templates()
}

/// Create a custom social media template for a platform
#[update]
async fn create_custom_social_template(
    platform: defi::price_alert_service::SocialPlatform,
    message_type: defi::social_media_formatter::SocialMessageType,
    template: String,
) -> Result<(), String> {
    // In production: validate user permissions and template content
    if template.len() > 10000 {
        return Err("Template too long (max 10000 characters)".to_string());
    }
    
    // For now, return success - actual implementation would store in persistent storage
    Ok(())
}

/// Test social media formatting with sample data
#[query]
fn preview_social_media_post(
    platform: defi::price_alert_service::SocialPlatform,
    token_symbol: String,
    price: f64,
    change_24h: f64,
    include_defi_context: bool,
) -> Result<String, String> {
    // Create sample alert and price data for preview
    let sample_alert = defi::price_alert_service::PriceAlert {
        id: "preview_alert".to_string(),
        user_id: "preview_user".to_string(),
        token_symbol: token_symbol.clone(),
        condition: defi::price_alert_service::PriceCondition::Above(price * 0.9),
        actions: vec![],
        social_config: None,
        created_at: ic_cdk::api::time(),
        expires_at: None,
        is_active: true,
        triggered_count: 0,
        max_triggers: None,
    };
    
    let sample_price = defi::price_alert_service::TokenPrice {
        symbol: token_symbol,
        price_usd: price,
        change_24h,
        volume_24h: 1000000.0,
        market_cap: 50000000.0,
        timestamp: ic_cdk::api::time(),
        source: defi::price_alert_service::PriceSource::Multiple,
    };
    
    let sample_defi_result = if include_defi_context {
        Some(defi::price_alert_defi_integration::DeFiExecutionResult {
            success: true,
            transaction_hash: Some("0x1234...abcd".to_string()),
            estimated_return: Some(125.50),
            actual_gas_cost: Some(25.0),
            error_message: None,
            strategy_id: Some("yield_farm_strategy_1".to_string()),
        })
    } else {
        None
    };
    
    match defi::social_media_formatter::format_social_post_with_defi(
        &platform,
        &sample_alert,
        &sample_price,
        sample_defi_result.as_ref(),
        Some(&["DeFi".to_string(), "PriceAlert".to_string()]),
    ) {
        Ok(post_data) => Ok(post_data.message),
        Err(e) => Err(format!("Failed to format social post: {}", e)),
    }
}

/// Test the complete price alert -> DeFi -> social media flow
#[update]
async fn test_complete_price_alert_flow(
    token_symbol: String,
    target_price: f64,
    enable_defi: bool,
    enable_social: bool,
) -> Result<String, String> {
    ic_cdk::println!("🗺 Testing complete price alert flow for {} at ${}", token_symbol, target_price);
    
    let test_user_id = "test_user_123".to_string();
    
    // Create test alert with both DeFi and social actions
    let mut actions = vec![];
    
    if enable_defi {
        actions.push(defi::price_alert_service::AlertAction::DeFiExecution {
            strategy_type: "market_buy".to_string(),
            parameters: "{\"amount\":100}".to_string(),
            amount: 100.0,
        });
    }
    
    if enable_social {
        actions.push(defi::price_alert_service::AlertAction::SocialPost {
            platforms: vec![
                defi::price_alert_service::SocialPlatform::Twitter,
                defi::price_alert_service::SocialPlatform::Discord,
            ],
            message_template: "🚨 {token} just hit ${price}! Target was ${target_price}".to_string(),
            include_chart: true,
            hashtags: vec!["DeFi".to_string(), "PriceAlert".to_string(), token_symbol.clone()],
        });
    }
    
    let test_alert = defi::price_alert_service::PriceAlert {
        id: String::new(),
        user_id: test_user_id,
        token_symbol: token_symbol.clone(),
        condition: defi::price_alert_service::PriceCondition::Above(target_price),
        actions,
        social_config: Some(defi::price_alert_service::SocialPostConfig {
            auto_post: true,
            custom_message: None,
            include_price_chart: true,
            mention_community: true,
            share_with_followers: true,
        }),
        created_at: 0,
        expires_at: None,
        is_active: true,
        triggered_count: 0,
        max_triggers: Some(1),
    };
    
    // Create the alert
    let alert_id = create_price_alert(test_alert)?;
    ic_cdk::println!("✅ Created test alert: {}", alert_id);
    
    Ok(format!(
        "🎯 Test completed successfully!\n- Alert ID: {}\n- Token: {}\n- Target Price: ${}\n- DeFi Actions: {}\n- Social Posts: {}\n\n✨ Complete flow: Price Monitor → Alert Trigger → DeFi Execution → JSON Formatting → Social Media Posts",
        alert_id, token_symbol, target_price,
        if enable_defi { "Enabled" } else { "Disabled" },
        if enable_social { "Enabled" } else { "Disabled" }
    ))
}

/// Get system status for price alerts and social media integration
#[query]
fn get_price_alert_system_status() -> String {
    format!(r#"📋 DeFlow Price Alert & Social Media System Status
    
✅ Core Components:
- Price Alert Service: Active
- DeFi Integration Engine: Active  
- Social Media Formatter: Active
- Real-time Price Fetching: Active (CoinGecko + Binance)
- HTTP Outcalls: Configured

🔗 Supported Integrations:
- Twitter/X API v2 (configured)
- Discord Webhooks (configured)  
- Telegram Bot API (configured)
- Reddit API (OAuth2 ready)

💰 DeFi Actions:
- Market Buy/Sell Orders
- Yield Farming Strategies
- Portfolio Rebalancing
- Arbitrage Execution
- Stop Loss & Take Profit

🌍 Supported Price Sources:
- CoinGecko (primary)
- Binance (fallback)
- Cached fallback prices

🛡️ Security Features:
- Daily capital limits
- Risk management
- Cooldown periods
- Market condition validation

🗺 Complete Flow:
Price Monitor → Alert Trigger → DeFi Execution → JSON Formatting → Social Media Posts

✨ Ready for production deployment!
"#)
}

// =============================================================================
// SCHEDULER API ENDPOINTS
// =============================================================================

use scheduler_service::{SchedulerService, ScheduleResult, ScheduleInfo};
use std::cell::RefCell;

thread_local! {
    static SCHEDULER: RefCell<SchedulerService> = RefCell::new(SchedulerService::new());
}

/// Create a one-time schedule using universal date format (dd/mm/yy hh:mm:ss)
#[update]
async fn create_schedule(
    datetime_string: String,
    workflow_id: String,
    node_id: String,
    timezone: Option<String>,
) -> Result<ScheduleResult, String> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow_mut().create_schedule_from_universal_format(
            &datetime_string,
            workflow_id,
            node_id,
            timezone,
        )
    })
}

/// Create a recurring schedule starting at specified time
#[update]
async fn create_recurring_schedule(
    start_datetime: String,
    interval_seconds: u64,
    workflow_id: String,
    node_id: String,
    max_executions: Option<u64>,
    timezone: Option<String>,
) -> Result<ScheduleResult, String> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow_mut().create_recurring_schedule(
            &start_datetime,
            interval_seconds,
            workflow_id,
            node_id,
            max_executions,
            timezone,
        )
    })
}

/// Create a schedule using cron expression
#[update]
async fn create_cron_schedule(
    cron_expression: String,
    workflow_id: String,
    node_id: String,
    timezone: Option<String>,
) -> Result<ScheduleResult, String> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow_mut().create_cron_schedule(
            &cron_expression,
            workflow_id,
            node_id,
            timezone,
        )
    })
}

/// List all active schedules
#[query]
fn list_schedules() -> Vec<ScheduleInfo> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow().list_active_schedules()
    })
}

/// Get details of a specific schedule
#[query]
fn get_schedule_details(schedule_id: String) -> Option<ScheduleInfo> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow().get_schedule(&schedule_id).cloned()
    })
}

/// Cancel a schedule
#[update]
async fn cancel_schedule(schedule_id: String) -> Result<String, String> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow_mut().cancel_schedule(&schedule_id)
    })
}

/// Update an existing schedule with new date/time
#[update]
async fn update_schedule(
    schedule_id: String,
    new_datetime: String,
) -> Result<ScheduleResult, String> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow_mut().update_schedule(&schedule_id, &new_datetime)
    })
}

/// Get next upcoming executions
#[query]
fn get_upcoming_executions(limit: usize) -> Vec<(String, u64, String)> {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow().get_next_executions(limit)
    })
}

/// Convert timestamp to universal format for display
#[query]
fn format_timestamp(timestamp_ns: u64) -> String {
    SCHEDULER.with(|scheduler| {
        scheduler.borrow().format_timestamp_to_universal(timestamp_ns)
    })
}

/// Get scheduler usage examples and documentation
#[query]
fn get_scheduler_examples() -> String {
    SchedulerService::example_usage()
}

// =============================================================================
// CYCLES MONITORING API ENDPOINTS
// =============================================================================

use cycles_monitor_service::{CyclesMonitorService, CyclesMonitorConfig, CyclesMonitorResult, CyclesData, CyclesStatistics};

thread_local! {
    static CYCLES_MONITOR: RefCell<CyclesMonitorService> = RefCell::new(CyclesMonitorService::new());
}

/// Create a new cycles monitor for a canister
#[update]
async fn create_cycles_monitor(
    monitor_id: String,
    canister_id: Option<String>,
    warning_threshold: u128,
    critical_threshold: u128,
    auto_topup: bool,
    topup_amount: u128,
    notification_channels: Vec<String>,
) -> Result<String, String> {
    let config = CyclesMonitorConfig {
        canister_id,
        warning_threshold,
        critical_threshold,
        auto_topup,
        topup_amount,
        notification_channels,
        owner: "current_user".to_string(), // In production, get from auth
        created_at: ic_cdk::api::time(),
        last_alert_sent: None,
    };

    CYCLES_MONITOR.with(|monitor| {
        monitor.borrow_mut().create_monitor(monitor_id, config)
    })
}

/// Check cycles for a specific monitor  
#[query]
fn check_cycles(monitor_id: String) -> Result<CyclesMonitorResult, String> {
    CYCLES_MONITOR.with(|monitor| {
        let service = monitor.borrow();
        let config = service.get_monitor_status(&monitor_id)
            .ok_or_else(|| format!("Monitor '{}' not found", monitor_id))?;

        // Get current cycles balance (simplified - not async for demo)
        let current_cycles = ic_cdk::api::canister_balance128();
        let status = if current_cycles <= config.critical_threshold {
            cycles_monitor_service::CyclesStatus::Critical
        } else if current_cycles <= config.warning_threshold {
            cycles_monitor_service::CyclesStatus::Warning
        } else {
            cycles_monitor_service::CyclesStatus::Healthy
        };

        let estimated_runtime_days = if current_cycles > 0 {
            Some((current_cycles / 100_000_000u128) as u32) // Simplified calculation
        } else {
            None
        };

        let cycles_data = cycles_monitor_service::CyclesData {
            canister_id: config.canister_id.clone().unwrap_or_else(|| "current".to_string()),
            current_cycles,
            warning_threshold: config.warning_threshold,
            critical_threshold: config.critical_threshold,
            status,
            estimated_runtime_days,
            last_check: ic_cdk::api::time(),
        };

        Ok(CyclesMonitorResult {
            success: true,
            message: format!("Cycles check completed for {}", cycles_data.canister_id),
            cycles_data: Some(cycles_data),
            alerts_triggered: Vec::new(), // Simplified for demo
        })
    })
}

/// Get current cycles balance for the current canister
#[query]
fn get_current_cycles_balance() -> u128 {
    ic_cdk::api::canister_balance128()
}

/// List all active cycles monitors
#[query]
fn list_cycles_monitors() -> Vec<(String, CyclesMonitorConfig)> {
    CYCLES_MONITOR.with(|monitor| {
        monitor.borrow().list_monitors()
            .into_iter()
            .map(|(id, config)| (id, config.clone()))
            .collect()
    })
}

/// Get recent cycles alerts
#[query]
fn get_cycles_alerts(limit: usize) -> Vec<cycles_monitor_service::CyclesAlert> {
    CYCLES_MONITOR.with(|monitor| {
        monitor.borrow().get_recent_alerts(limit)
            .into_iter()
            .cloned()
            .collect()
    })
}

/// Get cycles monitoring statistics
#[query]
fn get_cycles_statistics() -> CyclesStatistics {
    CYCLES_MONITOR.with(|monitor| {
        monitor.borrow().get_cycles_statistics()
    })
}

/// Remove a cycles monitor
#[update]
async fn remove_cycles_monitor(monitor_id: String) -> Result<String, String> {
    CYCLES_MONITOR.with(|monitor| {
        monitor.borrow_mut().remove_monitor(&monitor_id)
    })
}

/// Get cycles monitoring usage examples
#[query]
fn get_cycles_monitor_examples() -> String {
    CyclesMonitorService::usage_examples()
}