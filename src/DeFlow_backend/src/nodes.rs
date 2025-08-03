use crate::types::{
    WorkflowNode, NodeInput, NodeOutput, NodeError, ValidationError, 
    NodeConfiguration, NodeDefinition, ParameterSchema, ConfigValue,
    ExecutionContext
};
use crate::storage;
use ic_cdk::{api, update, query};
use std::collections::HashMap;

pub trait Node {
    fn execute(&self, input: NodeInput) -> Result<NodeOutput, NodeError>;
    fn validate_config(&self, config: &NodeConfiguration) -> Result<(), ValidationError>;
    fn get_definition(&self) -> NodeDefinition;
}

#[update]
pub async fn register_node(definition: NodeDefinition) -> Result<(), String> {
    if definition.node_type.is_empty() {
        return Err("Node type cannot be empty".to_string());
    }
    
    storage::insert_node_definition(definition.node_type.clone(), definition);
    
    Ok(())
}

#[query]
pub fn get_node_definition(node_type: String) -> Result<NodeDefinition, String> {
    storage::get_node_definition(&node_type)
        .ok_or_else(|| format!("Node type '{}' not found", node_type))
}

#[query]
pub fn list_node_types() -> Vec<String> {
    storage::NODE_REGISTRY.with(|registry| {
        registry.borrow().iter()
            .map(|(key, _)| key.clone())
            .collect()
    })
}

#[query]
pub fn list_nodes_by_category(category: String) -> Vec<NodeDefinition> {
    storage::NODE_REGISTRY.with(|registry| {
        registry.borrow().iter()
            .map(|(_, storable)| storable.0.clone())
            .filter(|def| def.category == category)
            .collect()
    })
}

pub async fn execute_node_internal(
    node: &WorkflowNode,
    input_data: &HashMap<String, ConfigValue>,
    context: &ExecutionContext
) -> Result<NodeOutput, String> {
    let timeout_ms = node.configuration.parameters
        .get("timeout")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n as u64),
            _ => None,
        })
        .unwrap_or(30000);
    
    let start_time = api::time();
    
    let result = match node.node_type.as_str() {
        "delay" => execute_delay_node(node, input_data).await,
        "condition" => execute_condition_node(node, input_data).await,
        "transform" => execute_transform_node(node, input_data).await,
        "http_request" => execute_http_request_node(node, input_data).await,
        "timer" => execute_timer_node(node, input_data).await,
        _ => execute_custom_node(node, input_data, context).await,
    };
    
    let elapsed_ms = (api::time() - start_time) / 1_000_000;
    if elapsed_ms > timeout_ms {
        return Err("TimeoutError".to_string());
    }
    
    result
}

// Built-in node implementations
pub async fn execute_delay_node(node: &WorkflowNode, _input: &HashMap<String, ConfigValue>) -> Result<NodeOutput, String> {
    let delay_ms = node.configuration.parameters
        .get("delay")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n as u64),
            _ => None,
        })
        .unwrap_or(1000);
    
    Ok(NodeOutput {
        data: HashMap::from([
            ("delayed".to_string(), ConfigValue::Boolean(true)),
            ("delay_ms".to_string(), ConfigValue::Number(delay_ms as f64)),
        ]),
        next_nodes: Vec::new(),
    })
}

pub async fn execute_condition_node(node: &WorkflowNode, input: &HashMap<String, ConfigValue>) -> Result<NodeOutput, String> {
    let condition = node.configuration.parameters
        .get("condition")
        .ok_or("Missing condition parameter")?;
    
    let result = evaluate_condition(condition, input)?;
    
    Ok(NodeOutput {
        data: HashMap::from([
            ("result".to_string(), ConfigValue::Boolean(result)),
        ]),
        next_nodes: if result { 
            vec!["true".to_string()]
        } else { 
            vec!["false".to_string()]
        },
    })
}

pub async fn execute_transform_node(node: &WorkflowNode, input: &HashMap<String, ConfigValue>) -> Result<NodeOutput, String> {
    let transform_type = node.configuration.parameters
        .get("type")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("passthrough");
    
    let output_data = match transform_type {
        "passthrough" => input.clone(),
        "uppercase" => transform_strings_uppercase(input),
        "lowercase" => transform_strings_lowercase(input),
        "json_parse" => transform_json_parse(input)?,
        _ => return Err(format!("Unknown transform type: {}", transform_type)),
    };
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: Vec::new(),
    })
}

pub async fn execute_http_request_node(node: &WorkflowNode, input: &HashMap<String, ConfigValue>) -> Result<NodeOutput, String> {
    use crate::http_client::{HttpClient, parse_json_response, json_to_config_value};
    use std::collections::HashMap as StdHashMap;
    
    let url = node.configuration.parameters
        .get("url")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing URL parameter")?;
    
    let method = node.configuration.parameters
        .get("method")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("GET".to_string());
    
    // Extract headers from configuration
    let headers = node.configuration.parameters
        .get("headers")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => {
                let mut headers = StdHashMap::new();
                for (key, value) in obj {
                    if let ConfigValue::String(val) = value {
                        headers.insert(key.clone(), val.clone());
                    }
                }
                Some(headers)
            }
            _ => None,
        });
    
    // Extract body from configuration or input
    let body = node.configuration.parameters
        .get("body")
        .or_else(|| input.get("body"))
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Object(_) | ConfigValue::Array(_) => {
                // Convert to JSON string
                let json_value = crate::http_client::config_value_to_json(v);
                serde_json::to_string(&json_value).ok()
            }
            _ => None,
        });
    
    // Make the HTTP request
    let response = match method.to_uppercase().as_str() {
        "GET" => HttpClient::get(&url, headers).await?,
        "POST" => HttpClient::post(&url, body, headers).await?,
        "PUT" => HttpClient::put(&url, body, headers).await?,
        "DELETE" => HttpClient::delete(&url, headers).await?,
        _ => return Err(format!("Unsupported HTTP method: {}", method)),
    };
    
    // Build response data
    let mut response_data = HashMap::new();
    response_data.insert("status".to_string(), ConfigValue::Number(response.status as f64));
    response_data.insert("url".to_string(), ConfigValue::String(url));
    response_data.insert("method".to_string(), ConfigValue::String(method));
    
    // Try to parse response as JSON, fall back to string
    let body_value = if let Ok(json_value) = parse_json_response(&response) {
        json_to_config_value(&json_value)
    } else {
        ConfigValue::String(response.body.clone())
    };
    response_data.insert("body".to_string(), body_value);
    
    // Add headers to response
    let headers_obj: HashMap<String, ConfigValue> = response.headers
        .into_iter()
        .map(|(k, v)| (k, ConfigValue::String(v)))
        .collect();
    response_data.insert("headers".to_string(), ConfigValue::Object(headers_obj));
    
    Ok(NodeOutput {
        data: response_data,
        next_nodes: Vec::new(),
    })
}

pub async fn execute_timer_node(node: &WorkflowNode, _input: &HashMap<String, ConfigValue>) -> Result<NodeOutput, String> {
    let interval = node.configuration.parameters
        .get("interval")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n as u64),
            _ => None,
        })
        .unwrap_or(60000);
    
    Ok(NodeOutput {
        data: HashMap::from([
            ("triggered_at".to_string(), ConfigValue::Number(api::time() as f64)),
            ("interval_ms".to_string(), ConfigValue::Number(interval as f64)),
        ]),
        next_nodes: Vec::new(),
    })
}

pub async fn execute_custom_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>,
    _context: &ExecutionContext
) -> Result<NodeOutput, String> {
    Err(format!("Unknown node type: {}", node.node_type))
}

// Helper functions
fn evaluate_condition(condition: &ConfigValue, input: &HashMap<String, ConfigValue>) -> Result<bool, String> {
    match condition {
        ConfigValue::Boolean(b) => Ok(*b),
        ConfigValue::String(expr) => {
            if expr.contains("==") {
                let parts: Vec<&str> = expr.split("==").collect();
                if parts.len() == 2 {
                    let left = parts[0].trim();
                    let right = parts[1].trim();
                    
                    if let Some(value) = input.get(left) {
                        match value {
                            ConfigValue::String(s) => Ok(s == right),
                            ConfigValue::Number(n) => {
                                if let Ok(num) = right.parse::<f64>() {
                                    Ok((n - num).abs() < f64::EPSILON)
                                } else {
                                    Ok(false)
                                }
                            }
                            ConfigValue::Boolean(b) => {
                                if let Ok(bool_val) = right.parse::<bool>() {
                                    Ok(*b == bool_val)
                                } else {
                                    Ok(false)
                                }
                            }
                            _ => Ok(false),
                        }
                    } else {
                        Ok(false)
                    }
                } else {
                    Err("Invalid condition format".to_string())
                }
            } else {
                Err("Unsupported condition operator".to_string())
            }
        }
        _ => Err("Invalid condition type".to_string()),
    }
}

fn transform_strings_uppercase(input: &HashMap<String, ConfigValue>) -> HashMap<String, ConfigValue> {
    input.iter().map(|(k, v)| {
        let new_value = match v {
            ConfigValue::String(s) => ConfigValue::String(s.to_uppercase()),
            _ => v.clone(),
        };
        (k.clone(), new_value)
    }).collect()
}

fn transform_strings_lowercase(input: &HashMap<String, ConfigValue>) -> HashMap<String, ConfigValue> {
    input.iter().map(|(k, v)| {
        let new_value = match v {
            ConfigValue::String(s) => ConfigValue::String(s.to_lowercase()),
            _ => v.clone(),
        };
        (k.clone(), new_value)
    }).collect()
}

fn transform_json_parse(input: &HashMap<String, ConfigValue>) -> Result<HashMap<String, ConfigValue>, String> {
    Ok(input.clone())
}

// Built-in node definitions
pub fn create_delay_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "delay".to_string(),
        name: "Delay".to_string(),
        description: "Delays execution for specified time".to_string(),
        category: "utility".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "delayed".to_string(),
                parameter_type: "boolean".to_string(),
                required: true,
                description: Some("Whether delay was applied".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "delay".to_string(),
                parameter_type: "number".to_string(),
                required: true,
                description: Some("Delay in milliseconds".to_string()),
                default_value: Some(ConfigValue::Number(1000.0)),
            }
        ],
    }
}

pub fn create_condition_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "condition".to_string(),
        name: "Condition".to_string(),
        description: "Evaluates conditions and controls flow".to_string(),
        category: "logic".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "result".to_string(),
                parameter_type: "boolean".to_string(),
                required: true,
                description: Some("Condition evaluation result".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "condition".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Condition expression to evaluate".to_string()),
                default_value: None,
            }
        ],
    }
}

pub fn create_transform_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "transform".to_string(),
        name: "Transform".to_string(),
        description: "Transforms input data".to_string(),
        category: "data".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![],
        configuration_schema: vec![
            ParameterSchema {
                name: "type".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Type of transformation".to_string()),
                default_value: Some(ConfigValue::String("passthrough".to_string())),
            }
        ],
    }
}

pub fn create_http_request_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "http_request".to_string(),
        name: "HTTP Request".to_string(),
        description: "Makes HTTP requests".to_string(),
        category: "network".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "status".to_string(),
                parameter_type: "number".to_string(),
                required: true,
                description: Some("HTTP status code".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "url".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("URL to request".to_string()),
                default_value: None,
            },
            ParameterSchema {
                name: "method".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                description: Some("HTTP method".to_string()),
                default_value: Some(ConfigValue::String("GET".to_string())),
            }
        ],
    }
}

pub fn create_timer_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "timer".to_string(),
        name: "Timer".to_string(),
        description: "Triggers on timer intervals".to_string(),
        category: "trigger".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "triggered_at".to_string(),
                parameter_type: "number".to_string(),
                required: true,
                description: Some("Timestamp when triggered".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "interval".to_string(),
                parameter_type: "number".to_string(),
                required: true,
                description: Some("Interval in milliseconds".to_string()),
                default_value: Some(ConfigValue::Number(60000.0)),
            }
        ],
    }
}

pub fn initialize_built_in_nodes() {
    let built_in_nodes = vec![
        create_delay_node_definition(),
        create_condition_node_definition(),
        create_transform_node_definition(),
        create_http_request_node_definition(),
        create_timer_node_definition(),
    ];
    
    for node_def in built_in_nodes {
        storage::insert_node_definition(node_def.node_type.clone(), node_def);
    }
}