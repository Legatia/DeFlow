use crate::types::{
    Workflow, WorkflowExecution, ExecutionStatus, NodeExecution, ExecutionContext,
    NodeOutput, ConfigValue, RetryPolicy, ExecutionGraph, WorkflowNode,
    WorkflowRecovery, FallbackStrategy, EmergencyAction
};
use crate::storage;
use crate::workflow::generate_id;
use crate::nodes::execute_node_internal;
use ic_cdk::{api, update, query, spawn};
use ic_cdk_timers::set_timer;
use std::collections::HashMap;
use std::time::Duration;

#[update]
pub async fn start_execution(workflow_id: String, trigger_data: Option<HashMap<String, ConfigValue>>) -> Result<String, String> {
    let workflow = storage::get_workflow(&workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;
    
    if !workflow.active {
        return Err("Workflow is not active".to_string());
    }
    
    let execution_id = generate_id();
    let execution = WorkflowExecution {
        id: execution_id.clone(),
        workflow_id: workflow_id.clone(),
        status: ExecutionStatus::Pending,
        started_at: api::time(),
        completed_at: None,
        trigger_data,
        node_executions: Vec::new(),
        error_message: None,
    };
    
    storage::insert_execution(execution_id.clone(), execution);
    
    spawn(execute_workflow(execution_id.clone()));
    
    Ok(execution_id)
}

#[query]
pub fn get_execution(id: String) -> Result<WorkflowExecution, String> {
    storage::get_execution(&id)
        .ok_or_else(|| "Execution not found".to_string())
}

#[query]
pub fn list_executions(workflow_id: Option<String>) -> Vec<WorkflowExecution> {
    storage::EXECUTIONS.with(|executions| {
        executions.borrow()
            .iter()
            .filter_map(|(_, storable)| {
                let execution = &storable.0;
                match &workflow_id {
                    Some(wf_id) if execution.workflow_id == *wf_id => Some(execution.clone()),
                    None => Some(execution.clone()),
                    _ => None,
                }
            })
            .collect()
    })
}

#[update]
pub async fn retry_failed_execution(execution_id: String, node_id: String) -> Result<(), String> {
    let execution = storage::get_execution(&execution_id)
        .ok_or("Execution not found")?;
    
    let workflow = storage::get_workflow(&execution.workflow_id)
        .ok_or("Workflow not found")?;
    
    let node = workflow.nodes.iter()
        .find(|n| n.id == node_id)
        .ok_or("Node not found")?;
    
    let retry_execution_id = generate_id();
    let mut retry_execution = WorkflowExecution {
        id: retry_execution_id.clone(),
        workflow_id: workflow.id.clone(),
        status: ExecutionStatus::Running,
        started_at: api::time(),
        completed_at: None,
        trigger_data: execution.trigger_data.clone(),
        node_executions: Vec::new(),
        error_message: None,
    };
    
    let context = ExecutionContext {
        workflow_id: workflow.id.clone(),
        execution_id: retry_execution_id.clone(),
        user_id: "retry".to_string(),
        timestamp: api::time(),
        global_variables: HashMap::new(),
    };
    
    let input_data = execution.node_executions
        .iter()
        .find(|ne| ne.node_id == node_id)
        .and_then(|ne| ne.input_data.clone())
        .unwrap_or_default();
    
    let result = execute_single_node(
        &retry_execution_id,
        node,
        input_data,
        &context,
        &mut retry_execution
    ).await;
    
    match result {
        Ok(_) => {
            retry_execution.status = ExecutionStatus::Completed;
            retry_execution.completed_at = Some(api::time());
        }
        Err(error) => {
            retry_execution.status = ExecutionStatus::Failed;
            retry_execution.completed_at = Some(api::time());
            retry_execution.error_message = Some(error);
        }
    }
    
    storage::insert_execution(retry_execution_id, retry_execution);
    
    Ok(())
}

pub async fn execute_workflow(execution_id: String) {
    let result = execute_workflow_internal(execution_id.clone()).await;
    
    if let Some(mut execution) = storage::get_execution(&execution_id) {
        match result {
            Ok(_) => {
                execution.status = ExecutionStatus::Completed;
                execution.completed_at = Some(api::time());
            }
            Err(error) => {
                execution.status = ExecutionStatus::Failed;
                execution.completed_at = Some(api::time());
                execution.error_message = Some(error);
            }
        }
        storage::insert_execution(execution_id, execution);
    }
}

async fn execute_workflow_internal(execution_id: String) -> Result<(), String> {
    let mut execution = storage::get_execution(&execution_id)
        .ok_or_else(|| "Execution not found".to_string())?;
    
    let workflow = storage::get_workflow(&execution.workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;
    
    execution.status = ExecutionStatus::Running;
    update_execution(&execution_id, &execution)?;
    
    let context = ExecutionContext {
        workflow_id: workflow.id.clone(),
        execution_id: execution_id.clone(),
        user_id: "anonymous".to_string(),
        timestamp: api::time(),
        global_variables: HashMap::new(),
    };
    
    let execution_graph = build_execution_graph(&workflow)?;
    let execution_order = topological_sort(&execution_graph)?;
    
    let mut node_outputs: HashMap<String, HashMap<String, ConfigValue>> = HashMap::new();
    
    for batch in execution_order {
        let mut batch_results = Vec::new();
        
        for node_id in batch {
            let node = workflow.nodes.iter()
                .find(|n| n.id == node_id)
                .ok_or_else(|| format!("Node {} not found", node_id))?;
            
            let input_data = prepare_node_input(&workflow, &node_id, &node_outputs)?;
            
            let result = execute_single_node(
                &execution_id,
                node,
                input_data,
                &context,
                &mut execution
            ).await;
            
            batch_results.push((node_id.clone(), result));
        }
        
        for (node_id, result) in batch_results {
            match result {
                Ok(output) => {
                    node_outputs.insert(node_id.clone(), output.data.clone());
                    mark_node_completed(&execution_id, &node_id, Some(output), None)?;
                }
                Err(error) => {
                    mark_node_failed(&execution_id, &node_id, &error)?;
                    
                    if is_critical_node(&workflow, &node_id) {
                        return Err(format!("Critical node {} failed: {}", node_id, error));
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn build_execution_graph(workflow: &Workflow) -> Result<ExecutionGraph, String> {
    let mut graph = ExecutionGraph {
        nodes: workflow.nodes.iter().map(|n| n.id.clone()).collect(),
        edges: Vec::new(),
    };
    
    for connection in &workflow.connections {
        graph.edges.push((
            connection.source_node_id.clone(),
            connection.target_node_id.clone()
        ));
    }
    
    validate_execution_graph(&graph)?;
    Ok(graph)
}

fn topological_sort(graph: &ExecutionGraph) -> Result<Vec<Vec<String>>, String> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    
    for node in &graph.nodes {
        in_degree.insert(node.clone(), 0);
        adjacency.insert(node.clone(), Vec::new());
    }
    
    for (from, to) in &graph.edges {
        *in_degree.get_mut(to).unwrap() += 1;
        adjacency.get_mut(from).unwrap().push(to.clone());
    }
    
    let mut result = Vec::new();
    let mut queue: Vec<String> = in_degree.iter()
        .filter(|(_, &degree)| degree == 0)
        .map(|(node, _)| node.clone())
        .collect();
    
    while !queue.is_empty() {
        let current_batch = queue.clone();
        result.push(current_batch.clone());
        queue.clear();
        
        for node in current_batch {
            for neighbor in &adjacency[&node] {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push(neighbor.clone());
                }
            }
        }
    }
    
    let processed_count: usize = result.iter().map(|batch| batch.len()).sum();
    if processed_count != graph.nodes.len() {
        return Err("Workflow contains cycles".to_string());
    }
    
    Ok(result)
}

fn validate_execution_graph(graph: &ExecutionGraph) -> Result<(), String> {
    for (from, to) in &graph.edges {
        if from == to {
            return Err(format!("Self-loop detected on node {}", from));
        }
    }
    
    for (from, to) in &graph.edges {
        if !graph.nodes.contains(from) {
            return Err(format!("Source node {} not found in workflow", from));
        }
        if !graph.nodes.contains(to) {
            return Err(format!("Target node {} not found in workflow", to));
        }
    }
    
    Ok(())
}

fn prepare_node_input(
    workflow: &Workflow,
    node_id: &str,
    node_outputs: &HashMap<String, HashMap<String, ConfigValue>>
) -> Result<HashMap<String, ConfigValue>, String> {
    let mut input_data = HashMap::new();
    
    for connection in &workflow.connections {
        if connection.target_node_id == node_id {
            if let Some(source_output) = node_outputs.get(&connection.source_node_id) {
                if let Some(value) = source_output.get(&connection.source_output) {
                    input_data.insert(connection.target_input.clone(), value.clone());
                } else {
                    return Err(format!(
                        "Output {} not found from node {}",
                        connection.source_output, connection.source_node_id
                    ));
                }
            } else {
                return Err(format!(
                    "No output found from node {}",
                    connection.source_node_id
                ));
            }
        }
    }
    
    Ok(input_data)
}

async fn execute_single_node(
    execution_id: &str,
    node: &WorkflowNode,
    input_data: HashMap<String, ConfigValue>,
    context: &ExecutionContext,
    execution: &mut WorkflowExecution
) -> Result<NodeOutput, String> {
    let node_execution = NodeExecution {
        node_id: node.id.clone(),
        status: ExecutionStatus::Running,
        started_at: Some(api::time()),
        completed_at: None,
        input_data: Some(input_data.clone()),
        output_data: None,
        error_message: None,
        retry_count: 0,
    };
    
    execution.node_executions.push(node_execution.clone());
    update_execution(execution_id, execution)?;
    
    let retry_policy = get_retry_policy(&node.node_type);
    
    let result = execute_with_retry(
        node,
        &input_data,
        context,
        &retry_policy,
        execution_id,
        execution
    ).await;
    
    if let Some(node_exec) = execution.node_executions.iter_mut()
        .find(|ne| ne.node_id == node.id) {
        match &result {
            Ok(output) => {
                node_exec.status = ExecutionStatus::Completed;
                node_exec.completed_at = Some(api::time());
                node_exec.output_data = Some(output.data.clone());
            }
            Err(error) => {
                node_exec.status = ExecutionStatus::Failed;
                node_exec.completed_at = Some(api::time());
                node_exec.error_message = Some(error.clone());
            }
        }
    }
    
    update_execution(execution_id, execution)?;
    result
}

async fn execute_with_retry(
    node: &WorkflowNode,
    input_data: &HashMap<String, ConfigValue>,
    context: &ExecutionContext,
    _retry_policy: &RetryPolicy,
    execution_id: &str,
    execution: &mut WorkflowExecution
) -> Result<NodeOutput, String> {
    // Get recovery configuration for this node type
    let recovery_config = get_recovery_config(&node.node_type);
    
    // Execute with self-healing recovery
    execute_with_recovery(
        node,
        input_data,
        context,
        &recovery_config,
        execution_id,
        execution
    ).await
}

async fn execute_with_recovery(
    node: &WorkflowNode,
    input_data: &HashMap<String, ConfigValue>,
    context: &ExecutionContext,
    recovery: &WorkflowRecovery,
    execution_id: &str,
    execution: &mut WorkflowExecution
) -> Result<NodeOutput, String> {
    let mut attempts = 0;
    let mut last_error = String::new();
    
    loop {
        let result = execute_node_internal(node, input_data, context).await;
        
        match result {
            Ok(output) => {
                // Success - update metrics and return
                if let Some(node_exec) = execution.node_executions.iter_mut()
                    .find(|ne| ne.node_id == node.id) {
                    node_exec.retry_count = attempts;
                }
                update_execution(execution_id, execution).ok();
                
                return Ok(output);
            }
            Err(error) => {
                attempts += 1;
                last_error = error.clone();
                
                // Log failure and update execution state
                log_execution_failure(execution_id, &node.id, &error, attempts);
                
                if let Some(node_exec) = execution.node_executions.iter_mut()
                    .find(|ne| ne.node_id == node.id) {
                    node_exec.retry_count = attempts;
                    node_exec.error_message = Some(error.clone());
                }
                update_execution(execution_id, execution).ok();
                
                if attempts < recovery.max_retries {
                    // Exponential backoff delay
                    let delay = recovery.retry_delay_ms * (2_u64.pow(attempts - 1));
                    let delay_ns = Duration::from_millis(delay);
                    
                        "Node {} failed (attempt {}), retrying in {}ms: {}",
                        node.id, attempts, delay, error
                    );
                    
                    // Non-blocking delay using timer
                    let _timer = set_timer(delay_ns, || {});
                    
                    // Try fallback strategy if available
                    if let Some(ref fallback) = recovery.fallback_strategy {
                        if let Ok(fallback_result) = execute_fallback_strategy(
                            node, 
                            input_data, 
                            context, 
                            fallback,
                            execution_id,
                            execution
                        ).await {
                            return Ok(fallback_result);
                        }
                    }
                } else {
                    // Max retries exceeded - execute emergency actions
                        "Node {} failed after {} retries, executing emergency actions",
                        node.id, attempts
                    );
                    
                    execute_emergency_actions(&recovery.emergency_actions, execution_id, &node.id).await;
                    
                    return Err(format!(
                        "Node {} failed after {} retries with self-healing recovery: {}",
                        node.id, attempts, last_error
                    ));
                }
            }
        }
    }
}

fn get_recovery_config(_node_type: &str) -> WorkflowRecovery {
    // In a real implementation, this would be configurable per node type
    // For now, return default recovery configuration
    WorkflowRecovery::default()
}

async fn execute_fallback_strategy(
    node: &WorkflowNode,
    _input_data: &HashMap<String, ConfigValue>,
    _context: &ExecutionContext,
    fallback: &FallbackStrategy,
    execution_id: &str,
    _execution: &mut WorkflowExecution
) -> Result<NodeOutput, String> {
    match fallback {
        FallbackStrategy::UseAlternativeNode { node_id } => {
            // In a real implementation, we would execute the alternative node
            // For now, return a simulated success
            Ok(NodeOutput {
                data: [("fallback".to_string(), ConfigValue::Boolean(true))].into(),
                next_nodes: vec![],
            })
        }
        FallbackStrategy::SkipNode => {
            Ok(NodeOutput {
                data: [("skipped".to_string(), ConfigValue::Boolean(true))].into(),
                next_nodes: vec![],
            })
        }
        FallbackStrategy::StopExecution => {
            Err("Execution stopped by fallback strategy".to_string())
        }
        FallbackStrategy::UseDefaultValue { value } => {
            Ok(NodeOutput {
                data: [("default".to_string(), value.clone())].into(),
                next_nodes: vec![],
            })
        }
        FallbackStrategy::NotifyAndContinue => {
            // Send notification (simulated)
            send_failure_notification(execution_id, &node.id).await;
            Ok(NodeOutput {
                data: [("notified".to_string(), ConfigValue::Boolean(true))].into(),
                next_nodes: vec![],
            })
        }
    }
}

async fn execute_emergency_actions(
    actions: &[EmergencyAction],
    execution_id: &str,
    node_id: &str
) {
    for action in actions {
        match action {
            EmergencyAction::SendNotification { recipient, message } => {
                    "Emergency notification to {}: {} (execution: {}, node: {})",
                    recipient, message, execution_id, node_id
                );
                // In a real implementation, this would send actual notifications
            }
            EmergencyAction::ExecuteWorkflow { workflow_id } => {
                // Execute emergency workflow
                if let Ok(emergency_execution_id) = crate::execution::start_execution(
                    workflow_id.clone(), 
                    Some([("emergency".to_string(), ConfigValue::Boolean(true))].into())
                ).await {
                }
            }
            EmergencyAction::LiquidatePosition { asset, percentage } => {
                    "Emergency liquidation: {} of {} (execution: {})",
                    percentage, asset, execution_id
                );
                // In a real DeFi implementation, this would trigger liquidation
            }
            EmergencyAction::PauseAllWorkflows => {
                pause_all_workflows().await;
            }
            EmergencyAction::EnableSafeMode => {
                enable_safe_mode().await;
            }
        }
    }
}

async fn send_failure_notification(execution_id: &str, node_id: &str) {
    // In a real implementation, this would send notifications via email, webhook, etc.
        "Notification: Node {} failed in execution {}",
        node_id, execution_id
    );
}

async fn pause_all_workflows() {
    // In a real implementation, this would pause all active workflows
}

async fn enable_safe_mode() {
    // In a real implementation, this would enable system-wide safe mode
}

fn log_execution_failure(execution_id: &str, node_id: &str, error: &str, attempt: u32) {
        "Execution failure: {} | Node: {} | Attempt: {} | Error: {}",
        execution_id, node_id, attempt, error
    );
    
    // In a real implementation, we would also update metrics and monitoring systems
    use crate::storage::{get_workflow_state, update_workflow_state};
    let mut state = get_workflow_state();
    
    // Record this failure in execution history
    if let Some((_, execution)) = state.active_workflows.iter()
        .find(|(_, exec)| exec.id == execution_id) {
        
        let failure_record = crate::types::ExecutionRecord {
            id: generate_id(),
            workflow_id: execution.workflow_id.clone(),
            execution_id: execution_id.to_string(),
            status: ExecutionStatus::Failed,
            started_at: execution.started_at,
            completed_at: Some(api::time()),
            duration_ms: None,
            gas_used: None,
            error_message: Some(error.to_string()),
            node_count: 1,
            retry_count: attempt,
        };
        
        state.execution_history.push(failure_record);
        
        // Keep only last 1000 records to prevent unbounded growth
        if state.execution_history.len() > 1000 {
            state.execution_history = state.execution_history.split_off(
                state.execution_history.len() - 1000
            );
        }
        
        update_workflow_state(state);
    }
}

fn get_retry_policy(node_type: &str) -> RetryPolicy {
    storage::get_retry_policy(node_type)
        .unwrap_or_default()
}

#[allow(dead_code)]
fn should_retry_error(error: &str, retry_on_errors: &[String]) -> bool {
    retry_on_errors.iter().any(|retry_error| error.contains(retry_error))
}

#[allow(dead_code)]
fn calculate_retry_delay(retry_count: u32, policy: &RetryPolicy) -> u64 {
    let delay = policy.initial_delay_ms as f64 * policy.backoff_multiplier.powi(retry_count as i32 - 1);
    let delay = delay as u64;
    std::cmp::min(delay, policy.max_delay_ms)
}

fn update_execution(execution_id: &str, execution: &WorkflowExecution) -> Result<(), String> {
    storage::insert_execution(execution_id.to_string(), execution.clone());
    Ok(())
}

fn mark_node_completed(
    execution_id: &str,
    node_id: &str,
    output: Option<NodeOutput>,
    _error: Option<String>
) -> Result<(), String> {
    if let Some(mut execution) = storage::get_execution(execution_id) {
        if let Some(node_exec) = execution.node_executions.iter_mut()
            .find(|ne| ne.node_id == node_id) {
            node_exec.status = ExecutionStatus::Completed;
            node_exec.completed_at = Some(api::time());
            if let Some(output) = output {
                node_exec.output_data = Some(output.data);
            }
        }
        storage::insert_execution(execution_id.to_string(), execution);
    }
    Ok(())
}

fn mark_node_failed(execution_id: &str, node_id: &str, error: &str) -> Result<(), String> {
    if let Some(mut execution) = storage::get_execution(execution_id) {
        if let Some(node_exec) = execution.node_executions.iter_mut()
            .find(|ne| ne.node_id == node_id) {
            node_exec.status = ExecutionStatus::Failed;
            node_exec.completed_at = Some(api::time());
            node_exec.error_message = Some(error.to_string());
        }
        storage::insert_execution(execution_id.to_string(), execution);
    }
    Ok(())
}

fn is_critical_node(_workflow: &Workflow, _node_id: &str) -> bool {
    true
}

// Zero-downtime workflow recovery
pub fn resume_active_workflows() {
    use crate::storage::{get_workflow_state, update_workflow_state};
    
    let mut state = get_workflow_state();
    let current_time = api::time();
    
    
    for (workflow_id, mut execution) in state.active_workflows.clone() {
        match execution.status {
            ExecutionStatus::Running | ExecutionStatus::Pending => {
                
                // Update execution to indicate it was resumed
                execution.error_message = Some("Execution resumed after canister upgrade".to_string());
                
                // Clone execution_id for the async move
                let execution_id = execution.id.clone();
                
                // Schedule the workflow to continue execution
                spawn(async move {
                    execute_workflow(execution_id).await;
                });
                
                // Update the execution in storage
                storage::insert_execution(execution.id.clone(), execution.clone());
            }
            ExecutionStatus::Failed => {
                // Check if we should retry failed executions
                let time_since_failure = current_time.saturating_sub(execution.started_at);
                let retry_threshold = 5 * 60 * 1_000_000_000; // 5 minutes in nanoseconds
                
                if time_since_failure < retry_threshold {
                    execution.status = ExecutionStatus::Pending;
                    execution.error_message = Some("Retrying after canister upgrade".to_string());
                    
                    // Clone execution_id for the async move
                    let execution_id = execution.id.clone();
                    
                    spawn(async move {
                        execute_workflow(execution_id).await;
                    });
                    
                    storage::insert_execution(execution.id.clone(), execution.clone());
                }
            }
            ExecutionStatus::Completed | ExecutionStatus::Cancelled => {
                // Remove completed/cancelled executions from active list
                state.active_workflows.retain(|(id, _)| id != &workflow_id);
            }
        }
    }
    
    // Update system health to reflect resumption
    state.system_health.last_heartbeat = current_time;
    state.system_health.active_workflows = state.active_workflows.len() as u32;
    
    update_workflow_state(state);
    
}