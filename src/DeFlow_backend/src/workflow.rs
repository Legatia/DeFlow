use crate::types::{Workflow, ConfigValue, ValidationError};
use crate::storage::WORKFLOWS;
use ic_cdk::{api, update, query};
use std::collections::HashMap;

pub fn validate_node_configuration(node_type: &str, config: &crate::types::NodeConfiguration) -> Result<(), ValidationError> {
    use crate::storage::NODE_REGISTRY;
    
    let definition = NODE_REGISTRY.with(|registry| {
        registry.borrow()
            .get(node_type)
            .cloned()
            .ok_or_else(|| ValidationError::SchemaValidationFailed(format!("Unknown node type: {}", node_type)))
    })?;
    
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

#[update]
pub async fn create_workflow(mut workflow: Workflow) -> Result<String, String> {
    if workflow.id.is_empty() {
        workflow.id = generate_id();
    }
    
    let current_time = api::time();
    workflow.created_at = current_time;
    workflow.updated_at = current_time;
    
    for node in &workflow.nodes {
        validate_node_configuration(&node.node_type, &node.configuration)
            .map_err(|e| format!("Node validation failed: {:?}", e))?;
    }
    
    WORKFLOWS.with(|workflows| {
        workflows.borrow_mut().insert(workflow.id.clone(), workflow.clone());
    });
    
    Ok(workflow.id)
}

#[update]
pub async fn update_workflow(workflow: Workflow) -> Result<(), String> {
    WORKFLOWS.with(|workflows| {
        let mut workflows = workflows.borrow_mut();
        
        if !workflows.contains_key(&workflow.id) {
            return Err("Workflow not found".to_string());
        }
        
        for node in &workflow.nodes {
            validate_node_configuration(&node.node_type, &node.configuration)
                .map_err(|e| format!("Node validation failed: {:?}", e))?;
        }
        
        let mut updated_workflow = workflow;
        updated_workflow.updated_at = api::time();
        
        workflows.insert(updated_workflow.id.clone(), updated_workflow);
        Ok(())
    })
}

#[query]
pub fn get_workflow(id: String) -> Result<Workflow, String> {
    WORKFLOWS.with(|workflows| {
        workflows.borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| "Workflow not found".to_string())
    })
}

#[query]
pub fn list_workflows() -> Vec<Workflow> {
    WORKFLOWS.with(|workflows| {
        workflows.borrow()
            .iter()
            .map(|(_, workflow)| workflow.clone())
            .collect()
    })
}

#[update]
pub async fn delete_workflow(id: String) -> Result<(), String> {
    WORKFLOWS.with(|workflows| {
        workflows.borrow_mut()
            .remove(&id)
            .ok_or_else(|| "Workflow not found".to_string())
            .map(|_| ())
    })
}