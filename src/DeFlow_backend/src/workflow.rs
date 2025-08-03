use crate::types::{Workflow, ConfigValue, ValidationError};
use crate::storage;
use ic_cdk::{api, update, query};
use candid::{CandidType, Deserialize};
use serde::Serialize;

pub fn validate_node_configuration(node_type: &str, config: &crate::types::NodeConfiguration) -> Result<(), ValidationError> {
    let definition = storage::get_node_definition(node_type)
        .ok_or_else(|| ValidationError::SchemaValidationFailed(format!("Unknown node type: {}", node_type)))?;
    
    for param_schema in definition.configuration_schema {
        if param_schema.required && !config.parameters.contains_key(&param_schema.name) {
            return Err(ValidationError::MissingRequiredParameter(param_schema.name));
        }
        
        if let Some(value) = config.parameters.get(&param_schema.name) {
            if !validate_parameter_type(value, &param_schema.parameter_type) {
                return Err(ValidationError::InvalidParameterType {
                    parameter: param_schema.name,
                    expected: param_schema.parameter_type,
                    got: get_config_value_type(value),
                });
            }
        }
    }
    
    Ok(())
}

fn validate_parameter_type(value: &ConfigValue, expected_type: &str) -> bool {
    match (value, expected_type) {
        (ConfigValue::String(_), "string") => true,
        (ConfigValue::Number(_), "number") => true,
        (ConfigValue::Boolean(_), "boolean") => true,
        (ConfigValue::Array(_), "array") => true,
        (ConfigValue::Object(_), "object") => true,
        _ => false,
    }
}

fn get_config_value_type(value: &ConfigValue) -> String {
    match value {
        ConfigValue::String(_) => "string".to_string(),
        ConfigValue::Number(_) => "number".to_string(),
        ConfigValue::Boolean(_) => "boolean".to_string(),
        ConfigValue::Array(_) => "array".to_string(),
        ConfigValue::Object(_) => "object".to_string(),
    }
}

pub fn generate_id() -> String {
    let time = api::time();
    format!("{:x}", time)
}

// Workflow validation functions
pub fn validate_workflow(workflow: &Workflow) -> Result<(), ValidationError> {
    // Check for duplicate node IDs
    let mut node_ids = std::collections::HashSet::new();
    for node in &workflow.nodes {
        if !node_ids.insert(&node.id) {
            return Err(ValidationError::DuplicateNodeId(node.id.clone()));
        }
    }

    // Validate node configurations
    for node in &workflow.nodes {
        validate_node_configuration(&node.node_type, &node.configuration)
            .map_err(|_| ValidationError::InvalidNodeConfiguration(node.id.clone()))?;
    }

    // Validate connections
    for connection in &workflow.connections {
        // Check that source node exists
        if !workflow.nodes.iter().any(|n| n.id == connection.source_node_id) {
            return Err(ValidationError::InvalidConnection {
                connection_id: format!("{}->{}",connection.source_node_id, connection.target_node_id),
                reason: format!("Source node '{}' not found", connection.source_node_id),
            });
        }

        // Check that target node exists
        if !workflow.nodes.iter().any(|n| n.id == connection.target_node_id) {
            return Err(ValidationError::InvalidConnection {
                connection_id: format!("{}->{}",connection.source_node_id, connection.target_node_id),
                reason: format!("Target node '{}' not found", connection.target_node_id),
            });
        }
    }

    // Check for cycles
    detect_cycles(workflow)?;

    // Validate triggers
    for trigger in &workflow.triggers {
        validate_trigger(trigger)?;
    }

    Ok(())
}

fn detect_cycles(workflow: &Workflow) -> Result<(), ValidationError> {
    let mut graph = std::collections::HashMap::new();
    let mut in_degree = std::collections::HashMap::new();

    // Build adjacency list and calculate in-degrees
    for node in &workflow.nodes {
        graph.insert(node.id.clone(), Vec::new());
        in_degree.insert(node.id.clone(), 0);
    }

    for connection in &workflow.connections {
        graph.get_mut(&connection.source_node_id)
            .unwrap()
            .push(connection.target_node_id.clone());
        *in_degree.get_mut(&connection.target_node_id).unwrap() += 1;
    }

    // Kahn's algorithm for topological sorting and cycle detection
    let mut queue = std::collections::VecDeque::new();
    for (node_id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node_id.clone());
        }
    }

    let mut processed_count = 0;
    while let Some(node_id) = queue.pop_front() {
        processed_count += 1;

        if let Some(neighbors) = graph.get(&node_id) {
            for neighbor in neighbors {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    if processed_count != workflow.nodes.len() {
        return Err(ValidationError::CycleDetected);
    }

    Ok(())
}

fn validate_trigger(trigger: &crate::types::WorkflowTrigger) -> Result<(), ValidationError> {
    match trigger {
        crate::types::WorkflowTrigger::Manual => Ok(()),
        crate::types::WorkflowTrigger::Schedule { cron } => {
            validate_cron_expression(cron)
        }
        crate::types::WorkflowTrigger::Webhook { path } => {
            if path.is_empty() {
                Err(ValidationError::InvalidTrigger("Webhook path cannot be empty".to_string()))
            } else {
                Ok(())
            }
        }
        crate::types::WorkflowTrigger::Event { event_type, conditions: _ } => {
            if event_type.is_empty() {
                Err(ValidationError::InvalidTrigger("Event type cannot be empty".to_string()))
            } else {
                Ok(())
            }
        }
    }
}

fn validate_cron_expression(cron: &str) -> Result<(), ValidationError> {
    if cron.is_empty() {
        return Err(ValidationError::InvalidTrigger("Cron expression cannot be empty".to_string()));
    }

    // Basic cron validation - supports simple patterns like "*/5" for every 5 minutes
    if cron.starts_with("*/") {
        let parts: Vec<&str> = cron.splitn(2, '/').collect();
        if parts.len() == 2 {
            if let Ok(interval) = parts[1].parse::<u32>() {
                if interval > 0 && interval <= 60 {
                    return Ok(());
                }
            }
        }
    }

    // For now, accept any non-empty cron expression
    // TODO: Implement full cron validation
    Ok(())
}

// Workflow analysis functions
pub fn analyze_workflow(workflow: &Workflow) -> WorkflowAnalysis {
    let mut analysis = WorkflowAnalysis {
        node_count: workflow.nodes.len(),
        connection_count: workflow.connections.len(),
        trigger_count: workflow.triggers.len(),
        has_cycles: false,
        entry_points: Vec::new(),
        orphaned_nodes: Vec::new(),
        complexity_score: 0,
    };

    // Find entry points (nodes with no incoming connections)
    let nodes_with_inputs: std::collections::HashSet<_> = workflow.connections
        .iter()
        .map(|c| &c.target_node_id)
        .collect();

    for node in &workflow.nodes {
        if !nodes_with_inputs.contains(&node.id) {
            analysis.entry_points.push(node.id.clone());
        }
    }

    // Find orphaned nodes (nodes with no connections at all)
    let connected_nodes: std::collections::HashSet<_> = workflow.connections
        .iter()
        .flat_map(|c| [&c.source_node_id, &c.target_node_id])
        .collect();

    for node in &workflow.nodes {
        if !connected_nodes.contains(&node.id) {
            analysis.orphaned_nodes.push(node.id.clone());
        }
    }

    // Check for cycles
    analysis.has_cycles = detect_cycles(workflow).is_err();

    // Calculate complexity score
    analysis.complexity_score = calculate_complexity_score(workflow);

    analysis
}

fn calculate_complexity_score(workflow: &Workflow) -> u32 {
    let base_score = workflow.nodes.len() as u32;
    let connection_bonus = (workflow.connections.len() as f32 * 0.5) as u32;
    let trigger_bonus = workflow.triggers.len() as u32 * 2;
    
    base_score + connection_bonus + trigger_bonus
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub struct WorkflowAnalysis {
    pub node_count: usize,
    pub connection_count: usize,
    pub trigger_count: usize,
    pub has_cycles: bool,
    pub entry_points: Vec<String>,
    pub orphaned_nodes: Vec<String>,
    pub complexity_score: u32,
}

#[update]
pub async fn create_workflow(mut workflow: Workflow) -> Result<String, String> {
    if workflow.id.is_empty() {
        workflow.id = generate_id();
    }
    
    let current_time = api::time();
    workflow.created_at = current_time;
    workflow.updated_at = current_time;
    
    // Comprehensive workflow validation
    validate_workflow(&workflow)
        .map_err(|e| format!("Workflow validation failed: {:?}", e))?;
    
    storage::insert_workflow(workflow.id.clone(), workflow.clone());
    
    Ok(workflow.id)
}

#[update]
pub async fn update_workflow(workflow: Workflow) -> Result<(), String> {
    if storage::get_workflow(&workflow.id).is_none() {
        return Err("Workflow not found".to_string());
    }
    
    // Comprehensive workflow validation
    validate_workflow(&workflow)
        .map_err(|e| format!("Workflow validation failed: {:?}", e))?;
    
    let mut updated_workflow = workflow;
    updated_workflow.updated_at = api::time();
    
    storage::insert_workflow(updated_workflow.id.clone(), updated_workflow);
    Ok(())
}

#[query]
pub fn get_workflow(id: String) -> Result<Workflow, String> {
    storage::get_workflow(&id)
        .ok_or_else(|| "Workflow not found".to_string())
}

#[query]
pub fn list_workflows() -> Vec<Workflow> {
    storage::WORKFLOWS.with(|workflows| {
        workflows.borrow().iter()
            .map(|(_, storable)| storable.0.clone())
            .collect()
    })
}

#[update]
pub async fn delete_workflow(id: String) -> Result<(), String> {
    storage::remove_workflow(&id)
        .ok_or_else(|| "Workflow not found".to_string())
        .map(|_| ())
}

#[query]
pub fn validate_workflow_query(workflow: Workflow) -> Result<(), String> {
    validate_workflow(&workflow)
        .map_err(|e| format!("Validation failed: {:?}", e))
}

#[query]
pub fn analyze_workflow_query(workflow: Workflow) -> WorkflowAnalysis {
    analyze_workflow(&workflow)
}