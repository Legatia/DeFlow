mod types;
mod storage;
mod workflow;
mod execution;
mod nodes;
mod events;
mod http_client;
mod defi;
mod user_management;

// Re-export types for external use
pub use types::*;

use ic_cdk::{init, post_upgrade, pre_upgrade, query, heartbeat, spawn, update};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use nodes::initialize_built_in_nodes;
use events::restore_scheduled_workflows;
use storage::{save_workflow_state_for_upgrade, restore_workflow_state_after_upgrade};
use types::{WorkflowState as InternalWorkflowState, SystemHealth as InternalSystemHealth, ExecutionStatus as InternalExecutionStatus};
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
fn get_system_health() -> SystemHealth {
    use storage::get_workflow_state;
    let state = get_workflow_state();
    state.system_health
}

#[update]
async fn trigger_health_check() -> SystemHealth {
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