use crate::types::{
    WorkflowNode, NodeInput, NodeOutput, NodeError, ValidationError, 
    NodeConfiguration, NodeDefinition, ParameterSchema, ConfigValue,
    ExecutionContext
};
use crate::storage;
use crate::defi::types::*;
use ic_cdk::{api, update, query};
use std::collections::HashMap;

#[allow(dead_code)]
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
        // Bitcoin DeFi nodes
        "bitcoin_portfolio" => execute_bitcoin_portfolio_node(node, input_data).await,
        "bitcoin_send" => execute_bitcoin_send_node(node, input_data).await,
        "bitcoin_address" => execute_bitcoin_address_node(node, input_data).await,
        "bitcoin_balance" => execute_bitcoin_balance_node(node, input_data).await,
        // Ethereum & L2 DeFi nodes
        "ethereum_portfolio" => execute_ethereum_portfolio_node(node, input_data).await,
        "ethereum_send" => execute_ethereum_send_node(node, input_data).await,
        "ethereum_address" => execute_ethereum_address_node(node, input_data).await,
        "ethereum_gas_estimate" => execute_ethereum_gas_estimate_node(node, input_data).await,
        "l2_optimization" => execute_l2_optimization_node(node, input_data).await,
        "bridge_analysis" => execute_bridge_analysis_node(node, input_data).await,
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
        // Social Media Integration nodes
        create_telegram_node_definition(),
        create_discord_node_definition(),
        create_twitter_node_definition(),
        create_facebook_node_definition(),
        create_email_node_definition(),
        create_linkedin_node_definition(),
        create_instagram_node_definition(),
        create_webhook_node_definition(),
        // Bitcoin DeFi nodes
        create_bitcoin_portfolio_node_definition(),
        create_bitcoin_send_node_definition(),
        create_bitcoin_address_node_definition(),
        create_bitcoin_balance_node_definition(),
        // Ethereum & L2 DeFi nodes
        create_ethereum_portfolio_node_definition(),
        create_ethereum_send_node_definition(),
        create_ethereum_address_node_definition(),
        create_ethereum_gas_estimate_node_definition(),
        create_l2_optimization_node_definition(),
        create_bridge_analysis_node_definition(),
    ];
    
    for node_def in built_in_nodes {
        storage::insert_node_definition(node_def.node_type.clone(), node_def);
    }
}

// ================================
// Bitcoin DeFi Workflow Nodes
// ================================

// Bitcoin Portfolio Node - Get user's Bitcoin portfolio
fn create_bitcoin_portfolio_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "bitcoin_portfolio".to_string(),
        name: "Bitcoin Portfolio".to_string(),
        description: "Get user's Bitcoin portfolio with all addresses and balances".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "total_btc".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Total Bitcoin balance".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "total_value_usd".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Total portfolio value in USD".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "addresses".to_string(),
                parameter_type: "array".to_string(),
                description: Some("List of Bitcoin addresses".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![],
    }
}

pub async fn execute_bitcoin_portfolio_node(
    _node: &WorkflowNode, 
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    // Get Bitcoin portfolio using DeFi API
    match crate::defi::api::get_bitcoin_portfolio().await {
        Ok(portfolio) => {
            let mut output_data = HashMap::new();
            output_data.insert("total_btc".to_string(), ConfigValue::Number(portfolio.total_btc));
            output_data.insert("total_value_usd".to_string(), ConfigValue::Number(portfolio.total_value_usd));
            output_data.insert("total_satoshis".to_string(), ConfigValue::Number(portfolio.total_satoshis as f64));
            
            // Convert addresses to array
            let addresses_data: Vec<ConfigValue> = portfolio.addresses
                .into_iter()
                .map(|addr| {
                    let mut addr_obj = HashMap::new();
                    addr_obj.insert("address".to_string(), ConfigValue::String(addr.address));
                    addr_obj.insert("balance_satoshis".to_string(), ConfigValue::Number(addr.balance_satoshis as f64));
                    addr_obj.insert("address_type".to_string(), ConfigValue::String(format!("{:?}", addr.address_type)));
                    ConfigValue::Object(addr_obj)
                })
                .collect();
            output_data.insert("addresses".to_string(), ConfigValue::Array(addresses_data));
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to get Bitcoin portfolio: {}", e)),
    }
}

// Bitcoin Send Node - Send Bitcoin to address
fn create_bitcoin_send_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "bitcoin_send".to_string(),
        name: "Send Bitcoin".to_string(),
        description: "Send Bitcoin to a specific address".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "to_address".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Destination Bitcoin address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "amount_satoshis".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Amount to send in satoshis".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "success".to_string(),
                parameter_type: "boolean".to_string(),
                description: Some("Whether the transaction was successful".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "transaction_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Transaction ID if successful".to_string()),
                required: false,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "fee_satoshis".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Transaction fee in satoshis (optional)".to_string()),
                required: false,
                default_value: Some(ConfigValue::Number(1000.0)),
            },
        ],
    }
}

pub async fn execute_bitcoin_send_node(
    node: &WorkflowNode, 
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    // Extract parameters
    let to_address = input.get("to_address")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing to_address parameter")?;
    
    let amount_satoshis = input.get("amount_satoshis")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n as u64),
            _ => None,
        })
        .ok_or("Missing amount_satoshis parameter")?;
    
    let fee_satoshis = node.configuration.parameters
        .get("fee_satoshis")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n as u64),
            _ => None,
        });
    
    // Send Bitcoin using DeFi API
    match crate::defi::api::send_bitcoin(to_address, amount_satoshis, fee_satoshis, None).await {
        Ok(result) => {
            let mut output_data = HashMap::new();
            output_data.insert("success".to_string(), ConfigValue::Boolean(result.success));
            
            if let Some(tx_id) = result.transaction_id {
                output_data.insert("transaction_id".to_string(), ConfigValue::String(tx_id));
            }
            
            output_data.insert("fee_satoshis".to_string(), ConfigValue::Number(result.fee_satoshis as f64));
            
            if let Some(error) = result.error_message {
                output_data.insert("error_message".to_string(), ConfigValue::String(error));
            }
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to send Bitcoin: {}", e)),
    }
}

// Bitcoin Address Node - Generate Bitcoin address
fn create_bitcoin_address_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "bitcoin_address".to_string(),
        name: "Bitcoin Address".to_string(),
        description: "Generate or get Bitcoin address for the user".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "address".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Bitcoin address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "address_type".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Type of Bitcoin address".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "address_type".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Type of address to generate (P2PKH, P2WPKH, P2TR)".to_string()),
                required: false,
                default_value: Some(ConfigValue::String("P2WPKH".to_string())),
            },
        ],
    }
}

pub async fn execute_bitcoin_address_node(
    node: &WorkflowNode, 
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let address_type_str = node.configuration.parameters
        .get("address_type")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("P2WPKH".to_string());
    
    let address_type = match address_type_str.as_str() {
        "P2PKH" => BitcoinAddressType::P2PKH,
        "P2WPKH" => BitcoinAddressType::P2WPKH,
        "P2TR" => BitcoinAddressType::P2TR,
        _ => BitcoinAddressType::P2WPKH,
    };
    
    // Get Bitcoin address using DeFi API
    match crate::defi::api::get_bitcoin_address(address_type).await {
        Ok(bitcoin_address) => {
            let mut output_data = HashMap::new();
            output_data.insert("address".to_string(), ConfigValue::String(bitcoin_address.address));
            output_data.insert("address_type".to_string(), ConfigValue::String(format!("{:?}", bitcoin_address.address_type)));
            output_data.insert("balance_satoshis".to_string(), ConfigValue::Number(bitcoin_address.balance_satoshis as f64));
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to get Bitcoin address: {}", e)),
    }
}

// Bitcoin Balance Node - Check balance of specific address
fn create_bitcoin_balance_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "bitcoin_balance".to_string(),
        name: "Bitcoin Balance".to_string(),
        description: "Check balance of a Bitcoin address".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "address".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Bitcoin address to check".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "balance_satoshis".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Balance in satoshis".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "balance_btc".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Balance in BTC".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![],
    }
}

pub async fn execute_bitcoin_balance_node(
    _node: &WorkflowNode, 
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let address = input.get("address")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing address parameter")?;
    
    // Validate address format first
    match crate::defi::api::validate_bitcoin_address(address.clone()) {
        Ok(_) => {
            // For now, simulate balance check since we need actual Bitcoin integration
            let balance_satoshis = 0u64; // Placeholder
            let balance_btc = (balance_satoshis as f64) / 100_000_000.0;
            
            let mut output_data = HashMap::new();
            output_data.insert("address".to_string(), ConfigValue::String(address));
            output_data.insert("balance_satoshis".to_string(), ConfigValue::Number(balance_satoshis as f64));
            output_data.insert("balance_btc".to_string(), ConfigValue::Number(balance_btc));
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Invalid Bitcoin address: {}", e)),
    }
}

// ================================
// Ethereum & L2 DeFi Workflow Nodes
// Day 9: Ethereum & L2 Integration
// ================================

// Ethereum Portfolio Node - Get user's Ethereum portfolio
fn create_ethereum_portfolio_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "ethereum_portfolio".to_string(),
        name: "Ethereum Portfolio".to_string(),
        description: "Get user's Ethereum portfolio across all supported EVM chains".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "total_eth".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Total ETH balance across all chains".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "total_value_usd".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Total portfolio value in USD".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "addresses".to_string(),
                parameter_type: "array".to_string(),
                description: Some("List of Ethereum addresses across chains".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![],
    }
}

pub async fn execute_ethereum_portfolio_node(
    _node: &WorkflowNode, 
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    // Get Ethereum portfolio using DeFi API
    match crate::defi::api::get_ethereum_portfolio().await {
        Ok(portfolio) => {
            let mut output_data = HashMap::new();
            output_data.insert("total_eth".to_string(), ConfigValue::Number(portfolio.total_eth));
            output_data.insert("total_value_usd".to_string(), ConfigValue::Number(portfolio.total_value_usd));
            
            // Convert addresses to array
            let addresses_data: Vec<ConfigValue> = portfolio.addresses
                .into_iter()
                .map(|addr| {
                    let mut addr_obj = HashMap::new();
                    addr_obj.insert("address".to_string(), ConfigValue::String(addr.address));
                    addr_obj.insert("balance_wei".to_string(), ConfigValue::String(addr.balance_wei));
                    addr_obj.insert("balance_eth".to_string(), ConfigValue::Number(addr.balance_eth));
                    addr_obj.insert("chain".to_string(), ConfigValue::String(addr.chain.name().to_string()));
                    addr_obj.insert("nonce".to_string(), ConfigValue::Number(addr.nonce as f64));
                    ConfigValue::Object(addr_obj)
                })
                .collect();
            output_data.insert("addresses".to_string(), ConfigValue::Array(addresses_data));
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to get Ethereum portfolio: {}", e)),
    }
}

// Ethereum Send Node - Send ETH to address
fn create_ethereum_send_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "ethereum_send".to_string(),
        name: "Send Ethereum".to_string(),
        description: "Send ETH to a specific address with optimal chain selection".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "to_address".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Destination Ethereum address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "amount_wei".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Amount to send in wei (as string)".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "success".to_string(),
                parameter_type: "boolean".to_string(),
                description: Some("Whether the transaction was successful".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "transaction_hash".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Transaction hash if successful".to_string()),
                required: false,
                default_value: None,
            },
            ParameterSchema {
                name: "chain_used".to_string(),
                parameter_type: "string".to_string(),
                description: Some("EVM chain used for transaction".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Specific EVM chain to use (optional)".to_string()),
                required: false,
                default_value: None,
            },
            ParameterSchema {
                name: "gas_priority".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Gas priority: Low, Medium, High, Urgent".to_string()),
                required: false,
                default_value: Some(ConfigValue::String("Medium".to_string())),
            },
            ParameterSchema {
                name: "optimize_for_cost".to_string(),
                parameter_type: "boolean".to_string(),
                description: Some("Optimize for lowest cost vs fastest execution".to_string()),
                required: false,
                default_value: Some(ConfigValue::Boolean(true)),
            },
        ],
    }
}

pub async fn execute_ethereum_send_node(
    node: &WorkflowNode, 
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    use crate::defi::ethereum::{EvmChain, GasPriority};
    
    // Extract parameters
    let to_address = input.get("to_address")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing to_address parameter")?;
    
    let amount_wei = input.get("amount_wei")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Number(n) => Some((*n as u128).to_string()),
            _ => None,
        })
        .ok_or("Missing amount_wei parameter")?;
    
    // Extract configuration
    let chain = node.configuration.parameters
        .get("chain")
        .and_then(|v| match v {
            ConfigValue::String(s) => match s.as_str() {
                "Ethereum" => Some(EvmChain::Ethereum),
                "Arbitrum" => Some(EvmChain::Arbitrum),
                "Optimism" => Some(EvmChain::Optimism),
                "Polygon" => Some(EvmChain::Polygon),
                "Base" => Some(EvmChain::Base),
                "Avalanche" => Some(EvmChain::Avalanche),
                _ => None,
            },
            _ => None,
        });
    
    let gas_priority = node.configuration.parameters
        .get("gas_priority")
        .and_then(|v| match v {
            ConfigValue::String(s) => match s.as_str() {
                "Low" => Some(GasPriority::Low),
                "Medium" => Some(GasPriority::Medium),
                "High" => Some(GasPriority::High),
                "Urgent" => Some(GasPriority::Urgent),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or(GasPriority::Medium);
    
    let optimize_for_cost = node.configuration.parameters
        .get("optimize_for_cost")
        .and_then(|v| match v {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        });
    
    // Send Ethereum using DeFi API
    match crate::defi::api::send_ethereum(to_address, amount_wei, chain, gas_priority, optimize_for_cost).await {
        Ok(result) => {
            let mut output_data = HashMap::new();
            output_data.insert("success".to_string(), ConfigValue::Boolean(result.success));
            
            if let Some(tx_hash) = result.transaction_hash {
                output_data.insert("transaction_hash".to_string(), ConfigValue::String(tx_hash));
            }
            
            output_data.insert("from_address".to_string(), ConfigValue::String(result.from_address));
            output_data.insert("to_address".to_string(), ConfigValue::String(result.to_address));
            output_data.insert("value_wei".to_string(), ConfigValue::String(result.value_wei));
            output_data.insert("total_fee_wei".to_string(), ConfigValue::String(result.total_fee_wei));
            
            if let Some(error) = result.error_message {
                output_data.insert("error_message".to_string(), ConfigValue::String(error));
            }
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to send Ethereum: {}", e)),
    }
}

// Ethereum Address Node - Generate Ethereum address
fn create_ethereum_address_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "ethereum_address".to_string(),
        name: "Ethereum Address".to_string(),
        description: "Generate or get Ethereum address for a specific chain".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "address".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Ethereum address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("EVM chain for the address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "balance_wei".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Current balance in wei".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("EVM chain: Ethereum, Arbitrum, Optimism, Polygon, Base, Avalanche".to_string()),
                required: false,
                default_value: Some(ConfigValue::String("Ethereum".to_string())),
            },
        ],
    }
}

pub async fn execute_ethereum_address_node(
    node: &WorkflowNode, 
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    use crate::defi::ethereum::EvmChain;
    
    let chain_str = node.configuration.parameters
        .get("chain")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Ethereum".to_string());
    
    let chain = match chain_str.as_str() {
        "Ethereum" => EvmChain::Ethereum,
        "Arbitrum" => EvmChain::Arbitrum,
        "Optimism" => EvmChain::Optimism,
        "Polygon" => EvmChain::Polygon,
        "Base" => EvmChain::Base,
        "Avalanche" => EvmChain::Avalanche,
        _ => EvmChain::Ethereum,
    };
    
    // Get Ethereum address using DeFi API
    match crate::defi::api::get_ethereum_address(chain.clone()).await {
        Ok(ethereum_address) => {
            let mut output_data = HashMap::new();
            output_data.insert("address".to_string(), ConfigValue::String(ethereum_address.address));
            output_data.insert("chain".to_string(), ConfigValue::String(ethereum_address.chain.name().to_string()));
            output_data.insert("balance_wei".to_string(), ConfigValue::String(ethereum_address.balance_wei));
            output_data.insert("balance_eth".to_string(), ConfigValue::Number(ethereum_address.balance_eth));
            output_data.insert("nonce".to_string(), ConfigValue::Number(ethereum_address.nonce as f64));
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to get Ethereum address: {}", e)),
    }
}

// Gas Estimation Node - Estimate gas for Ethereum transactions
fn create_ethereum_gas_estimate_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "ethereum_gas_estimate".to_string(),
        name: "Ethereum Gas Estimate".to_string(),
        description: "Estimate gas costs for Ethereum transactions".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "to_address".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Destination address (optional)".to_string()),
                required: false,
                default_value: None,
            },
            ParameterSchema {
                name: "value_wei".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Transaction value in wei (optional)".to_string()),
                required: false,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "gas_limit".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Estimated gas limit".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "gas_price".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Gas price in wei".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "total_fee_usd".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Total estimated fee in USD".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("EVM chain for gas estimation".to_string()),
                required: false,
                default_value: Some(ConfigValue::String("Ethereum".to_string())),
            },
            ParameterSchema {
                name: "priority".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Gas priority: Low, Medium, High, Urgent".to_string()),
                required: false,
                default_value: Some(ConfigValue::String("Medium".to_string())),
            },
        ],
    }
}

pub async fn execute_ethereum_gas_estimate_node(
    node: &WorkflowNode, 
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    use crate::defi::ethereum::{EvmChain, GasPriority};
    
    // Extract parameters
    let to_address = input.get("to_address")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        });
    
    let value_wei = input.get("value_wei")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Number(n) => Some((*n as u128).to_string()),
            _ => None,
        });
    
    // Extract configuration
    let chain_str = node.configuration.parameters
        .get("chain")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Ethereum".to_string());
    
    let chain = match chain_str.as_str() {
        "Ethereum" => EvmChain::Ethereum,
        "Arbitrum" => EvmChain::Arbitrum,
        "Optimism" => EvmChain::Optimism,
        "Polygon" => EvmChain::Polygon,
        "Base" => EvmChain::Base,
        "Avalanche" => EvmChain::Avalanche,
        _ => EvmChain::Ethereum,
    };
    
    let priority = node.configuration.parameters
        .get("priority")
        .and_then(|v| match v {
            ConfigValue::String(s) => match s.as_str() {
                "Low" => Some(GasPriority::Low),
                "Medium" => Some(GasPriority::Medium),
                "High" => Some(GasPriority::High),
                "Urgent" => Some(GasPriority::Urgent),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or(GasPriority::Medium);
    
    // Estimate gas using DeFi API
    match crate::defi::api::estimate_ethereum_gas(chain, to_address, None, value_wei, priority).await {
        Ok(estimate) => {
            let mut output_data = HashMap::new();
            output_data.insert("gas_limit".to_string(), ConfigValue::Number(estimate.gas_limit as f64));
            output_data.insert("gas_price".to_string(), ConfigValue::String(estimate.gas_price));
            output_data.insert("max_fee_per_gas".to_string(), ConfigValue::String(estimate.max_fee_per_gas));
            output_data.insert("max_priority_fee_per_gas".to_string(), ConfigValue::String(estimate.max_priority_fee_per_gas));
            output_data.insert("total_fee_wei".to_string(), ConfigValue::String(estimate.total_fee_wei));
            output_data.insert("total_fee_eth".to_string(), ConfigValue::Number(estimate.total_fee_eth));
            output_data.insert("total_fee_usd".to_string(), ConfigValue::Number(estimate.total_fee_usd));
            output_data.insert("confirmation_time_seconds".to_string(), ConfigValue::Number(estimate.confirmation_time_estimate_seconds as f64));
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to estimate gas: {}", e)),
    }
}

// L2 Optimization Node - Find optimal L2 for transaction
fn create_l2_optimization_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "l2_optimization".to_string(),
        name: "L2 Optimization".to_string(),
        description: "Find the optimal L2 chain for a transaction based on cost and speed".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "amount_wei".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Transaction amount in wei".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "transaction_type".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Type of transaction".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "recommended_chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Recommended EVM chain".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "estimated_fee_usd".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Estimated fee in USD".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "savings_vs_ethereum".to_string(),
                parameter_type: "number".to_string(),
                description: Some("USD savings vs Ethereum mainnet".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "gas_priority".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Gas priority: Low, Medium, High, Urgent".to_string()),
                required: false,
                default_value: Some(ConfigValue::String("Medium".to_string())),
            },
        ],
    }
}

pub async fn execute_l2_optimization_node(
    node: &WorkflowNode, 
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    use crate::defi::ethereum::GasPriority;
    
    // Extract parameters
    let amount_wei = input.get("amount_wei")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Number(n) => Some((*n as u128).to_string()),
            _ => None,
        })
        .ok_or("Missing amount_wei parameter")?;
    
    let transaction_type = input.get("transaction_type")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing transaction_type parameter")?;
    
    let gas_priority = node.configuration.parameters
        .get("gas_priority")
        .and_then(|v| match v {
            ConfigValue::String(s) => match s.as_str() {
                "Low" => Some(GasPriority::Low),
                "Medium" => Some(GasPriority::Medium),
                "High" => Some(GasPriority::High),
                "Urgent" => Some(GasPriority::Urgent),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or(GasPriority::Medium);
    
    // Get L2 optimization using DeFi API
    match crate::defi::api::get_l2_optimization(amount_wei, transaction_type, gas_priority).await {
        Ok(optimization) => {
            let mut output_data = HashMap::new();
            output_data.insert("recommended_chain".to_string(), ConfigValue::String(optimization.recommended_chain.name().to_string()));
            output_data.insert("estimated_fee_usd".to_string(), ConfigValue::Number(optimization.estimated_fee_usd));
            output_data.insert("estimated_time_seconds".to_string(), ConfigValue::Number(optimization.estimated_time_seconds as f64));
            output_data.insert("savings_vs_ethereum".to_string(), ConfigValue::Number(optimization.savings_vs_ethereum));
            output_data.insert("total_cost_usd".to_string(), ConfigValue::Number(optimization.total_cost_usd));
            
            if let Some(bridge_cost) = optimization.bridge_cost_usd {
                output_data.insert("bridge_cost_usd".to_string(), ConfigValue::Number(bridge_cost));
            }
            
            // Convert alternatives to array
            let alternatives_data: Vec<ConfigValue> = optimization.alternatives
                .into_iter()
                .map(|alt| {
                    let mut alt_obj = HashMap::new();
                    alt_obj.insert("chain".to_string(), ConfigValue::String(alt.chain.name().to_string()));
                    alt_obj.insert("fee_usd".to_string(), ConfigValue::Number(alt.fee_usd));
                    alt_obj.insert("total_cost_usd".to_string(), ConfigValue::Number(alt.total_cost_usd));
                    alt_obj.insert("time_seconds".to_string(), ConfigValue::Number(alt.time_seconds as f64));
                    alt_obj.insert("confidence_score".to_string(), ConfigValue::Number(alt.confidence_score));
                    ConfigValue::Object(alt_obj)
                })
                .collect();
            output_data.insert("alternatives".to_string(), ConfigValue::Array(alternatives_data));
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to get L2 optimization: {}", e)),
    }
}

// Bridge Analysis Node - Analyze bridge options between chains
fn create_bridge_analysis_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "bridge_analysis".to_string(),
        name: "Bridge Analysis".to_string(),
        description: "Analyze bridge options between EVM chains".to_string(),
        category: "DeFi".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "from_chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Source EVM chain".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "to_chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Destination EVM chain".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "bridge_routes".to_string(),
                parameter_type: "array".to_string(),
                description: Some("Available bridge routes".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "recommended_route".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Recommended bridge route".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![],
    }
}

pub async fn execute_bridge_analysis_node(
    _node: &WorkflowNode, 
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    use crate::defi::ethereum::EvmChain;
    
    // Extract parameters
    let from_chain_str = input.get("from_chain")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing from_chain parameter")?;
    
    let to_chain_str = input.get("to_chain")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing to_chain parameter")?;
    
    let from_chain = match from_chain_str.as_str() {
        "Ethereum" => EvmChain::Ethereum,
        "Arbitrum" => EvmChain::Arbitrum,
        "Optimism" => EvmChain::Optimism,
        "Polygon" => EvmChain::Polygon,
        "Base" => EvmChain::Base,
        "Avalanche" => EvmChain::Avalanche,
        _ => return Err(format!("Unsupported from_chain: {}", from_chain_str)),
    };
    
    let to_chain = match to_chain_str.as_str() {
        "Ethereum" => EvmChain::Ethereum,
        "Arbitrum" => EvmChain::Arbitrum,
        "Optimism" => EvmChain::Optimism,
        "Polygon" => EvmChain::Polygon,
        "Base" => EvmChain::Base,
        "Avalanche" => EvmChain::Avalanche,
        _ => return Err(format!("Unsupported to_chain: {}", to_chain_str)),
    };
    
    // Get bridge options using simplified DeFi API
    match crate::defi::api::get_bridge_options(from_chain.clone(), to_chain.clone()).await {
        Ok(routes) => {
            let mut output_data = HashMap::new();
            
            // Convert string routes to array
            let routes_data: Vec<ConfigValue> = routes.iter()
                .enumerate()
                .map(|(i, route)| {
                    let mut route_obj = HashMap::new();
                    route_obj.insert("from_chain".to_string(), ConfigValue::String(from_chain.name().to_string()));
                    route_obj.insert("to_chain".to_string(), ConfigValue::String(to_chain.name().to_string()));
                    route_obj.insert("bridge_type".to_string(), ConfigValue::String(route.clone()));
                    route_obj.insert("estimated_cost_usd".to_string(), ConfigValue::Number(5.0 + i as f64));
                    route_obj.insert("estimated_time_minutes".to_string(), ConfigValue::Number(30.0 + (i * 15) as f64));
                    route_obj.insert("total_hops".to_string(), ConfigValue::Number(1.0));
                    ConfigValue::Object(route_obj)
                })
                .collect();
            output_data.insert("bridge_routes".to_string(), ConfigValue::Array(routes_data));
            
            // Set recommended route (first one as default)
            if let Some(first_route) = routes.first() {
                let mut recommended_obj = HashMap::new();
                recommended_obj.insert("from_chain".to_string(), ConfigValue::String(from_chain.name().to_string()));
                recommended_obj.insert("to_chain".to_string(), ConfigValue::String(to_chain.name().to_string()));
                recommended_obj.insert("bridge_type".to_string(), ConfigValue::String(first_route.clone()));
                recommended_obj.insert("estimated_cost_usd".to_string(), ConfigValue::Number(5.0));
                recommended_obj.insert("estimated_time_minutes".to_string(), ConfigValue::Number(30.0));
                output_data.insert("recommended_route".to_string(), ConfigValue::Object(recommended_obj));
            }
            
            Ok(NodeOutput {
                data: output_data,
                next_nodes: vec![],
            })
        },
        Err(e) => Err(format!("Failed to get bridge options: {}", e)),
    }
}

// ================================
// Social Media Integration Nodes
// ================================

// Telegram Node - Send messages to Telegram
fn create_telegram_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "telegram".to_string(),
        name: "Telegram".to_string(),
        description: "Send messages to Telegram channels or users".to_string(),
        category: "Social".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "message".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Message to send".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "message_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Telegram message ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "bot_token".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Telegram bot token".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "chat_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Target chat ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
    }
}

// Discord Node - Send messages to Discord
fn create_discord_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "discord".to_string(),
        name: "Discord".to_string(),
        description: "Send messages to Discord channels".to_string(),
        category: "Social".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "message".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Message to send".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "message_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Discord message ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "webhook_url".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Discord webhook URL".to_string()),
                required: true,
                default_value: None,
            },
        ],
    }
}

// Twitter Node - Post tweets
fn create_twitter_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "twitter".to_string(),
        name: "Twitter".to_string(),
        description: "Post tweets to Twitter".to_string(),
        category: "Social".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "tweet_text".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Tweet content".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "tweet_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Posted tweet ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "api_key".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Twitter API key".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "api_secret".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Twitter API secret".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "access_token".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Twitter access token".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "access_token_secret".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Twitter access token secret".to_string()),
                required: true,
                default_value: None,
            },
        ],
    }
}

// Facebook Node - Post to Facebook
fn create_facebook_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "facebook".to_string(),
        name: "Facebook".to_string(),
        description: "Post to Facebook pages or profiles".to_string(),
        category: "Social".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "message".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Post content".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "post_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Facebook post ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "access_token".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Facebook access token".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "page_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Facebook page ID".to_string()),
                required: false,
                default_value: None,
            },
        ],
    }
}

// Email Node - Send emails
fn create_email_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "email".to_string(),
        name: "Email".to_string(),
        description: "Send emails via SMTP".to_string(),
        category: "Communication".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "subject".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Email subject".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "body".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Email body".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "message_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Email message ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "to_email".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Recipient email address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "from_email".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Sender email address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "smtp_server".to_string(),
                parameter_type: "string".to_string(),
                description: Some("SMTP server address".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "smtp_port".to_string(),
                parameter_type: "number".to_string(),
                description: Some("SMTP server port".to_string()),
                required: false,
                default_value: Some(ConfigValue::Number(587.0)),
            },
            ParameterSchema {
                name: "username".to_string(),
                parameter_type: "string".to_string(),
                description: Some("SMTP username".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "password".to_string(),
                parameter_type: "string".to_string(),
                description: Some("SMTP password".to_string()),
                required: true,
                default_value: None,
            },
        ],
    }
}

// LinkedIn Node - Post to LinkedIn
fn create_linkedin_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "linkedin".to_string(),
        name: "LinkedIn".to_string(),
        description: "Post to LinkedIn profiles or company pages".to_string(),
        category: "Social".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "message".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Post content".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "post_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("LinkedIn post ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "access_token".to_string(),
                parameter_type: "string".to_string(),
                description: Some("LinkedIn access token".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "person_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("LinkedIn person ID".to_string()),
                required: false,
                default_value: None,
            },
        ],
    }
}

// Instagram Node - Post to Instagram
fn create_instagram_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "instagram".to_string(),
        name: "Instagram".to_string(),
        description: "Post to Instagram via Facebook Graph API".to_string(),
        category: "Social".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "image_url".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Image URL to post".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "caption".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Post caption".to_string()),
                required: false,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "media_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Instagram media ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "access_token".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Instagram access token".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "instagram_account_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Instagram business account ID".to_string()),
                required: true,
                default_value: None,
            },
        ],
    }
}

// Webhook Node - Send HTTP webhooks
fn create_webhook_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "webhook".to_string(),
        name: "Webhook".to_string(),
        description: "Send HTTP webhooks to external services".to_string(),
        category: "Integration".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "payload".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Webhook payload data".to_string()),
                required: true,
                default_value: None,
            },
        ],
        output_schema: vec![
            ParameterSchema {
                name: "response_status".to_string(),
                parameter_type: "number".to_string(),
                description: Some("HTTP response status code".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "response_body".to_string(),
                parameter_type: "string".to_string(),
                description: Some("HTTP response body".to_string()),
                required: false,
                default_value: None,
            },
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "url".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Webhook URL".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "method".to_string(),
                parameter_type: "string".to_string(),
                description: Some("HTTP method (GET, POST, PUT, DELETE)".to_string()),
                required: false,
                default_value: Some(ConfigValue::String("POST".to_string())),
            },
            ParameterSchema {
                name: "headers".to_string(),
                parameter_type: "object".to_string(),
                description: Some("HTTP headers".to_string()),
                required: false,
                default_value: None,
            },
        ],
    }
}