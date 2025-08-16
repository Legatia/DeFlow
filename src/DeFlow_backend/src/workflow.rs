use crate::types::{Workflow, ConfigValue, ValidationError, WorkflowState};
use crate::storage;
use ic_cdk::{api, update, query, caller};
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

    // Note: Node access validation is handled by frontend for better UX
    // Backend allows all nodes but execution will check user tier

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

// fn validate_node_access(workflow: &Workflow) -> Result<(), ValidationError> {
//     // Get user's allowed node types
//     let allowed_node_types = match user_management::get_allowed_node_types() {
//         Ok(nodes) => nodes,
//         Err(e) => {
//             // If user is not registered, they get Standard tier access
//             if e.contains("User not found") {
//                 crate::types::SubscriptionTier::Standard.allowed_node_types()
//             } else {
//                 return Err(ValidationError::SchemaValidationFailed(format!("Failed to check user access: {}", e)));
//             }
//         }
//     };

//     // Check each node in the workflow
//     for node in &workflow.nodes {
//         if !allowed_node_types.contains(&node.node_type) {
//             return Err(ValidationError::SchemaValidationFailed(
//                 format!(
//                     "Access denied: Node type '{}' requires Premium or Pro subscription. Your current tier only allows: {}",
//                     node.node_type,
//                     allowed_node_types.join(", ")
//                 )
//             ));
//         }
//     }

//     Ok(())
// }

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
    let principal = caller();
    let principal_id = principal.to_text();
    
    if workflow.id.is_empty() {
        workflow.id = generate_id();
    }
    
    let current_time = api::time();
    workflow.created_at = current_time;
    workflow.updated_at = current_time;
    
    // Set owner to current user
    workflow.owner = Some(principal_id);
    
    // Workflow state is already set from frontend or defaults to Draft
    
    // Initialize metadata if not set
    if workflow.metadata.is_none() {
        workflow.metadata = Some(crate::types::WorkflowMetadata::default());
    }
    
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

// ===== WORKFLOW STATE MANAGEMENT API =====

#[query]
pub fn get_user_workflows_by_state(state: WorkflowState) -> Vec<Workflow> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    storage::WORKFLOWS.with(|workflows| {
        workflows.borrow().iter()
            .filter(|(_, workflow)| {
                workflow.0.owner.as_ref() == Some(&principal_id) &&
                workflow.0.state == state
            })
            .map(|(_, workflow)| workflow.0)
            .collect()
    })
}

#[query] 
pub fn get_user_drafts() -> Vec<Workflow> {
    get_user_workflows_by_state(WorkflowState::Draft)
}

#[query]
pub fn get_user_published_workflows() -> Vec<Workflow> {
    get_user_workflows_by_state(WorkflowState::Published)
}

#[query]
pub fn get_user_templates() -> Vec<Workflow> {
    get_user_workflows_by_state(WorkflowState::Template)
}

#[update]
pub async fn publish_workflow(workflow_id: String) -> Result<(), String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    let mut workflow = storage::get_workflow(&workflow_id)
        .ok_or_else(|| "Workflow not found".to_string())?;
    
    // Verify ownership
    if workflow.owner.as_ref() != Some(&principal_id) {
        return Err("Access denied. You can only publish your own workflows.".to_string());
    }
    
    // Change state to published
    workflow.state = WorkflowState::Published;
    workflow.active = true;
    workflow.updated_at = api::time();
    
    // Validate before publishing
    validate_workflow(&workflow)
        .map_err(|e| format!("Cannot publish invalid workflow: {:?}", e))?;
    
    storage::insert_workflow(workflow_id, workflow);
    Ok(())
}

#[update]
pub async fn save_as_template(workflow_id: String, template_name: String, category: String, description: String, is_public: bool) -> Result<String, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    let source_workflow = storage::get_workflow(&workflow_id)
        .ok_or_else(|| "Source workflow not found".to_string())?;
    
    // Verify ownership
    if source_workflow.owner.as_ref() != Some(&principal_id) {
        return Err("Access denied. You can only create templates from your own workflows.".to_string());
    }
    
    let current_time = api::time();
    let template_id = format!("template_{}_{}", principal_id, current_time);
    
    // Create template workflow
    let mut template_workflow = source_workflow.clone();
    template_workflow.id = template_id.clone();
    template_workflow.name = template_name;
    template_workflow.description = Some(description.clone());
    template_workflow.state = WorkflowState::Template;
    template_workflow.active = false; // Templates are not executable
    template_workflow.created_at = current_time;
    template_workflow.updated_at = current_time;
    
    // Set template metadata
    template_workflow.metadata = Some(crate::types::WorkflowMetadata {
        template_category: Some(category),
        template_description: Some(description),
        usage_count: Some(0),
        is_public: Some(is_public),
        original_workflow_id: Some(workflow_id),
    });
    
    storage::insert_workflow(template_id.clone(), template_workflow);
    Ok(template_id)
}

#[update]
pub async fn create_from_template(template_id: String, workflow_name: String) -> Result<String, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    let template = storage::get_workflow(&template_id)
        .ok_or_else(|| "Template not found".to_string())?;
    
    // Verify it's a template
    if !matches!(template.state, WorkflowState::Template) {
        return Err("Specified workflow is not a template".to_string());
    }
    
    // Check if template is public or owned by user
    let can_access = template.metadata
        .as_ref()
        .and_then(|m| m.is_public)
        .unwrap_or(false) || 
        template.owner.as_ref() == Some(&principal_id);
        
    if !can_access {
        return Err("Access denied. Template is private.".to_string());
    }
    
    let current_time = api::time();
    let new_workflow_id = generate_id();
    
    // Create new workflow from template
    let mut new_workflow = template.clone();
    new_workflow.id = new_workflow_id.clone();
    new_workflow.name = workflow_name;
    new_workflow.owner = Some(principal_id);
    new_workflow.state = WorkflowState::Draft; // New workflows start as drafts
    new_workflow.active = false;
    new_workflow.created_at = current_time;
    new_workflow.updated_at = current_time;
    new_workflow.metadata = Some(crate::types::WorkflowMetadata::default());
    
    // Increment template usage count
    if let Some(mut template_meta) = template.metadata.clone() {
        template_meta.usage_count = Some(template_meta.usage_count.unwrap_or(0) + 1);
        let mut updated_template = template;
        updated_template.metadata = Some(template_meta);
        storage::insert_workflow(template_id, updated_template);
    }
    
    storage::insert_workflow(new_workflow_id.clone(), new_workflow);
    Ok(new_workflow_id)
}

#[query]
pub fn get_public_templates() -> Vec<Workflow> {
    storage::WORKFLOWS.with(|workflows| {
        workflows.borrow().iter()
            .filter(|(_, workflow)| {
                matches!(workflow.0.state, WorkflowState::Template) &&
                workflow.0.metadata
                    .as_ref()
                    .and_then(|m| m.is_public)
                    .unwrap_or(false)
            })
            .map(|(_, workflow)| workflow.0)
            .collect()
    })
}