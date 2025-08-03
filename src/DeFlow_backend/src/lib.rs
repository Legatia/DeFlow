mod types;
mod storage;
mod workflow;
mod execution;
mod nodes;
mod events;
mod http_client;

// Re-export types for external use
pub use types::*;

use ic_cdk::{init, post_upgrade, query};
use nodes::initialize_built_in_nodes;
use events::restore_scheduled_workflows;

// Re-export all the API functions from modules
pub use workflow::{create_workflow, update_workflow, get_workflow, list_workflows, delete_workflow, validate_workflow_query, analyze_workflow_query, WorkflowAnalysis};
pub use execution::{start_execution, get_execution, list_executions, retry_failed_execution};
pub use nodes::{register_node, get_node_definition, list_node_types, list_nodes_by_category};
pub use events::{
    emit_event, register_event_listener, webhook_trigger, register_webhook,
    schedule_workflow, unschedule_workflow, list_scheduled_workflows,
    set_retry_policy, get_retry_policy_for_node
};

#[init]
fn init() {
    initialize_built_in_nodes();
    ic_cdk::println!("DeFlow backend initialized");
}

#[post_upgrade]
fn post_upgrade() {
    initialize_built_in_nodes();
    restore_scheduled_workflows();
    ic_cdk::println!("DeFlow backend upgraded");
}

#[query]
fn greet(name: String) -> String {
    format!("Hello, {}! Welcome to DeFlow!", name)
}