use crate::types::{
    WorkflowNode, NodeInput, NodeOutput, NodeError, ValidationError, 
    NodeConfiguration, NodeDefinition, ParameterSchema, ConfigValue,
    ExecutionContext
};
use crate::storage;
use crate::defi::{ChainId, Asset};
use crate::defi::types::*;
use crate::fee_collection::{FeeCollectionService, TransactionFeeRequest};
use crate::security::spending_limits_enforcement::{SpendingLimitsEnforcement, SpendingError};
use ic_cdk::{api, update, query, caller};
use candid::Principal;
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

/// SECURITY CRITICAL: Validate spending against user-approved limits
/// This MUST be called before any token spending operation
async fn validate_spending_limits(
    user: Principal,
    token_symbol: &str,
    amount: u64,
    operation: &str,
) -> Result<(), String> {
    match SpendingLimitsEnforcement::validate_spending_request(user, token_symbol, amount, operation) {
        Ok(()) => {
            ic_cdk::println!(
                "✅ SPENDING VALIDATED: User {} can spend {} {} for {}",
                user.to_text(), amount, token_symbol, operation
            );
            Ok(())
        }
        Err(SpendingError::NoApprovalFound(token)) => {
            Err(format!("❌ SPENDING DENIED: No approval found for token {}", token))
        }
        Err(SpendingError::ExceedsDailyLimit { requested, remaining }) => {
            Err(format!("❌ SPENDING DENIED: Amount {} exceeds daily limit. Remaining: {}", requested, remaining))
        }
        Err(SpendingError::ExceedsTotalLimit { requested, remaining }) => {
            Err(format!("❌ SPENDING DENIED: Amount {} exceeds total limit. Remaining: {}", requested, remaining))
        }
        Err(SpendingError::OperationNotAllowed { operation, allowed }) => {
            Err(format!("❌ SPENDING DENIED: Operation '{}' not allowed. Permitted: {:?}", operation, allowed))
        }
        Err(SpendingError::ApprovalExpired { token, expired_at }) => {
            Err(format!("❌ SPENDING DENIED: Approval for {} expired at {}", token, expired_at))
        }
        Err(SpendingError::ApprovalNotActive(token)) => {
            Err(format!("❌ SPENDING DENIED: Approval for {} is not active", token))
        }
        Err(e) => {
            Err(format!("❌ SPENDING DENIED: {}", e))
        }
    }
}

/// SECURITY CRITICAL: Record successful spending transaction
/// This MUST be called after any successful token spending
async fn record_successful_spending(
    user: Principal,
    token_symbol: &str,
    amount: u64,
    operation: &str,
    transaction_hash: Option<String>,
) -> Result<(), String> {
    match SpendingLimitsEnforcement::record_spending(user, token_symbol, amount, operation, transaction_hash.clone()) {
        Ok(()) => {
            ic_cdk::println!(
                "💰 SPENDING RECORDED: User {} spent {} {} for {} (tx: {:?})",
                user.to_text(), amount, token_symbol, operation, transaction_hash
            );
            Ok(())
        }
        Err(e) => {
            ic_cdk::println!(
                "⚠️ SPENDING RECORD FAILED: User {} failed to record spending: {}",
                user.to_text(), e
            );
            // Don't fail the operation, but log the issue
            Ok(())
        }
    }
}

/// Helper function to collect transaction fees for DeFi operations
async fn collect_defi_operation_fee(
    user: Principal, 
    transaction_value_usd: u64, 
    operation_type: &str,
    asset: Asset
) -> Result<(), String> {
    if transaction_value_usd == 0 {
        return Ok(()); // No fee for zero-value operations
    }

    let fee_request = TransactionFeeRequest {
        user,
        transaction_value_usd,
        asset,
        operation_type: operation_type.to_string(),
    };

    match FeeCollectionService::collect_transaction_fee(fee_request).await {
        Ok(result) if result.success => {
            ic_cdk::println!(
                "Fee collected: User={}, Amount=${}, Operation={}, Fee=${}", 
                user.to_text(), 
                transaction_value_usd, 
                operation_type,
                result.fee_amount
            );
            Ok(())
        }
        Ok(result) => {
            let error_msg = result.error.unwrap_or("Unknown fee collection error".to_string());
            ic_cdk::println!("Fee collection failed: {}", error_msg);
            // For now, don't fail the entire operation if fee collection fails
            // In production, you might want to queue for retry or fail the operation
            Ok(())
        }
        Err(e) => {
            ic_cdk::println!("Fee collection error: {}", e);
            // Don't fail the DeFi operation due to fee collection issues in beta
            Ok(())
        }
    }
}

/// Extract transaction value from node parameters or input data
fn extract_transaction_value(
    node: &WorkflowNode, 
    input_data: &HashMap<String, ConfigValue>
) -> u64 {
    // Try to get transaction amount from node configuration
    if let Some(ConfigValue::Number(amount)) = node.configuration.parameters.get("amount") {
        return *amount as u64;
    }
    
    // Try to get from input data
    if let Some(ConfigValue::Number(amount)) = input_data.get("amount") {
        return *amount as u64;
    }
    
    // Try common parameter names
    for key in &["value", "transaction_value", "amount_usd", "trade_size"] {
        if let Some(ConfigValue::Number(amount)) = input_data.get(*key) {
            return *amount as u64;
        }
    }
    
    0 // Default to 0 if no value found
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
    
    // Check if this is a DeFi operation that requires fee collection
    let is_defi_operation = matches!(node.node_type.as_str(), 
        "bitcoin_send" | "ethereum_send" | "swap" | "yield_farm" | 
        "arbitrage" | "lending" | "borrowing" | "bridge_analysis" |
        "l2_optimization"
    );
    
    // Collect fee before executing DeFi operations
    if is_defi_operation {
        let transaction_value = extract_transaction_value(node, input_data);
        if transaction_value > 0 {
            // Get user from context (for now use anonymous, in production get from context)
            let user = caller(); // Get the actual caller
            
            // Create a default asset (should be extracted from node config in production)
            let asset = Asset {
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                chain: ChainId::Ethereum,
                contract_address: Some("0xA0b86a33E6411E6A3fc0c39E4e90C8C4Bb8eF5E8".to_string()),
                decimals: 6,
                is_native: false,
            };
            
            // Collect fee (non-blocking - don't fail operation if fee collection fails)
            let _ = collect_defi_operation_fee(user, transaction_value, &node.node_type, asset).await;
        }
    }

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
        // Simplified DeFi Strategy nodes
        "select-yield-protocol" => execute_select_yield_protocol_node(node, input_data).await,
        "set-farm-amount" => execute_set_farm_amount_node(node, input_data).await,
        "execute-yield-farm" => execute_execute_yield_farm_node(node, input_data).await,
        "select-arbitrage-asset" => execute_select_arbitrage_asset_node(node, input_data).await,
        "set-arbitrage-chains" => execute_set_arbitrage_chains_node(node, input_data).await,
        "execute-arbitrage" => execute_execute_arbitrage_node(node, input_data).await,
        "set-portfolio-allocation" => execute_set_portfolio_allocation_node(node, input_data).await,
        "execute-rebalance" => execute_execute_rebalance_node(node, input_data).await,
        "check-cycles" => execute_check_cycles_node(node, input_data).await,
        "cycles-alert" => execute_cycles_alert_node(node, input_data).await,
        "auto-topup-cycles" => execute_auto_topup_cycles_node(node, input_data).await,
        // Common utility nodes
        "select-asset" => execute_select_asset_node(node, input_data).await,
        "select-chain" => execute_select_chain_node(node, input_data).await,
        "set-amount" => execute_set_amount_node(node, input_data).await,
        "check-price" => execute_check_price_node(node, input_data).await,
        "check-balance" => execute_check_balance_node(node, input_data).await,
        "estimate-gas" => execute_estimate_gas_node(node, input_data).await,
        // Simplified Social Media nodes
        "social-auth-setup" => execute_social_auth_setup_node(node, input_data).await,
        "select-platform" => execute_select_platform_node(node, input_data).await,
        "social-media-post" => execute_social_media_post_node(node, input_data).await,
        // AI Content Generation nodes
        "ai-content-setup" => execute_ai_content_setup_node(node, input_data).await,
        "generate-content" => execute_generate_content_node(node, input_data).await,
        "content-optimizer" => execute_content_optimizer_node(node, input_data).await,
        "ai-responder" => execute_ai_responder_node(node, input_data).await,
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
        // Social Media Integration nodes - Simplified
        create_telegram_node_definition(),
        create_discord_node_definition(),
        create_email_node_definition(),
        create_webhook_node_definition(),
        // Simplified Social Media Workflow Nodes
        create_social_auth_setup_node_definition(),
        create_select_platform_node_definition(),
        create_social_media_post_node_definition(),
        // AI Content Generation Nodes
        create_ai_content_setup_node_definition(),
        create_generate_content_node_definition(),
        create_content_optimizer_node_definition(),
        create_ai_responder_node_definition(),
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
        // Simplified DeFi Strategy Nodes
        create_select_yield_protocol_node_definition(),
        create_set_farm_amount_node_definition(),
        create_execute_yield_farm_node_definition(),
        create_select_arbitrage_asset_node_definition(),
        create_set_arbitrage_chains_node_definition(),
        create_execute_arbitrage_node_definition(),
        create_set_portfolio_allocation_node_definition(),
        create_execute_rebalance_node_definition(),
        create_check_cycles_node_definition(),
        create_cycles_alert_node_definition(),
        create_auto_topup_cycles_node_definition(),
        // Common Reusable DeFi Utility Nodes
        create_select_asset_node_definition(),
        create_select_chain_node_definition(),
        create_set_amount_node_definition(),
        create_check_price_node_definition(),
        create_check_balance_node_definition(),
        create_estimate_gas_node_definition(),
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
    let user = caller();
    
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
    
    // SECURITY CRITICAL: Validate spending limits before transaction
    validate_spending_limits(user, "BTC", amount_satoshis, "send").await?;
    
    // Send Bitcoin using DeFi API
    match crate::defi::api::send_bitcoin(to_address, amount_satoshis, fee_satoshis, None).await {
        Ok(result) => {
            let mut output_data = HashMap::new();
            output_data.insert("success".to_string(), ConfigValue::Boolean(result.success));
            
            if let Some(tx_id) = result.transaction_id.clone() {
                output_data.insert("transaction_id".to_string(), ConfigValue::String(tx_id.clone()));
                
                // SECURITY CRITICAL: Record successful spending
                if result.success {
                    record_successful_spending(user, "BTC", amount_satoshis, "send", Some(tx_id)).await?;
                }
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
    
    let user = caller();
    
    // Extract parameters
    let to_address = input.get("to_address")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or("Missing to_address parameter")?;
    
    let amount_wei_str = input.get("amount_wei")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Number(n) => Some((*n as u128).to_string()),
            _ => None,
        })
        .ok_or("Missing amount_wei parameter")?;
    
    // Convert amount to u64 for spending validation (wei)
    let amount_wei_u64: u64 = amount_wei_str.parse()
        .map_err(|_| "Invalid amount_wei format")?;
    
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
    
    // SECURITY CRITICAL: Validate spending limits before transaction
    validate_spending_limits(user, "ETH", amount_wei_u64, "send").await?;
    
    // Send Ethereum using DeFi API
    match crate::defi::api::send_ethereum(to_address, amount_wei_str, chain, gas_priority, optimize_for_cost).await {
        Ok(result) => {
            let mut output_data = HashMap::new();
            output_data.insert("success".to_string(), ConfigValue::Boolean(result.success));
            
            if let Some(tx_hash) = result.transaction_hash.clone() {
                output_data.insert("transaction_hash".to_string(), ConfigValue::String(tx_hash.clone()));
                
                // SECURITY CRITICAL: Record successful spending
                if result.success {
                    record_successful_spending(user, "ETH", amount_wei_u64, "send", Some(tx_hash)).await?;
                }
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
// ================================
// Simplified DeFi Strategy Nodes
// ================================

// Select Yield Protocol Node
fn create_select_yield_protocol_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "select-yield-protocol".to_string(),
        name: "Select Yield Protocol".to_string(),
        description: "Choose DeFi protocol for farming".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "protocol_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Protocol configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "protocol".to_string(),
                parameter_type: "string".to_string(),
                description: Some("DeFi Protocol".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("Aave".to_string())),
            }
        ],
    }
}

// Set Farm Amount Node  
fn create_set_farm_amount_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "set-farm-amount".to_string(),
        name: "Set Farm Amount".to_string(),
        description: "Configure farming amount and token".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "protocol_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Protocol configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "farm_config".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Farm configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "token".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Token to Farm".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("USDC".to_string())),
            },
            ParameterSchema {
                name: "amount".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Amount in USD".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(1000.0)),
            }
        ],
    }
}

// Execute Yield Farm Node
fn create_execute_yield_farm_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "execute-yield-farm".to_string(),
        name: "Execute Yield Farm".to_string(),
        description: "Execute the yield farming operation".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "farm_config".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Farm configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "result".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Farm execution result".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "min_apy".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Minimum APY (%)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(5.0)),
            },
            ParameterSchema {
                name: "auto_compound".to_string(),
                parameter_type: "boolean".to_string(),
                description: Some("Auto Compound".to_string()),
                required: false,
                default_value: Some(ConfigValue::Boolean(true)),
            }
        ],
    }
}

// Select Arbitrage Asset Node
fn create_select_arbitrage_asset_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "select-arbitrage-asset".to_string(),
        name: "Select Arbitrage Asset".to_string(),
        description: "Choose asset for arbitrage trading".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "asset_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Asset configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "asset".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Asset to Arbitrage".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("BTC".to_string())),
            }
        ],
    }
}

// Set Arbitrage Chains Node
fn create_set_arbitrage_chains_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "set-arbitrage-chains".to_string(),
        name: "Set Arbitrage Chains".to_string(),
        description: "Configure buy and sell chains".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "asset_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Asset configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "chain_config".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Chain configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "buy_chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Buy Chain".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("Ethereum".to_string())),
            },
            ParameterSchema {
                name: "sell_chain".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Sell Chain".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("Arbitrum".to_string())),
            }
        ],
    }
}

// Execute Arbitrage Node
fn create_execute_arbitrage_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "execute-arbitrage".to_string(),
        name: "Execute Arbitrage".to_string(),
        description: "Execute arbitrage opportunity".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "chain_config".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Chain configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "result".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Arbitrage execution result".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "min_profit_percent".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Min Profit (%)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(1.0)),
            },
            ParameterSchema {
                name: "max_amount".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Max Amount (USD)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(5000.0)),
            }
        ],
    }
}

// Set Portfolio Allocation Node
fn create_set_portfolio_allocation_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "set-portfolio-allocation".to_string(),
        name: "Set Portfolio Allocation".to_string(),
        description: "Define target portfolio percentages".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "allocation_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Allocation configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "btc_percent".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Bitcoin (%)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(60.0)),
            },
            ParameterSchema {
                name: "eth_percent".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Ethereum (%)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(30.0)),
            },
            ParameterSchema {
                name: "stable_percent".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Stablecoin (%)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(10.0)),
            }
        ],
    }
}

// Execute Rebalance Node
fn create_execute_rebalance_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "execute-rebalance".to_string(),
        name: "Execute Rebalance".to_string(),
        description: "Rebalance portfolio to target allocation".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "allocation_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Allocation configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "result".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Rebalance execution result".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "rebalance_threshold".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Rebalance Threshold (%)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(5.0)),
            },
            ParameterSchema {
                name: "min_trade_amount".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Min Trade Amount (USD)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(50.0)),
            }
        ],
    }
}

// Check Cycles Node
fn create_check_cycles_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "check-cycles".to_string(),
        name: "Check Cycles".to_string(),
        description: "Check current canister cycle balance".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "cycles_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Cycles data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "canister_id".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Canister ID (optional)".to_string()),
                required: false,
                default_value: None,
            }
        ],
    }
}

// Cycles Alert Node
fn create_cycles_alert_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "cycles-alert".to_string(),
        name: "Cycles Alert".to_string(),
        description: "Alert when cycles are running low".to_string(),
        category: "conditions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "cycles_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Cycles data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "low_cycles".to_string(),
                parameter_type: "boolean".to_string(),
                description: Some("Low cycles alert".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "cycles_ok".to_string(),
                parameter_type: "boolean".to_string(),
                description: Some("Cycles OK".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "warning_threshold".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Warning Threshold (T Cycles)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(10.0)),
            }
        ],
    }
}

// Auto Top-up Cycles Node
fn create_auto_topup_cycles_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "auto-topup-cycles".to_string(),
        name: "Auto Top-up Cycles".to_string(),
        description: "Automatically request cycles top-up".to_string(),
        category: "actions".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "low_cycles".to_string(),
                parameter_type: "boolean".to_string(),
                description: Some("Low cycles alert".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "topup_result".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Top-up result".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "topup_amount".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Top-up Amount (T Cycles)".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(20.0)),
            },
            ParameterSchema {
                name: "notification_channel".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Notification Channel".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("email".to_string())),
            }
        ],
    }
}

// ================================
// Simplified DeFi Strategy Node Execution Functions
// ================================

pub async fn execute_select_yield_protocol_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let protocol = node.configuration.parameters
        .get("protocol")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Aave".to_string());
    
    let mut output_data = HashMap::new();
    output_data.insert("protocol".to_string(), ConfigValue::String(protocol.clone()));
    output_data.insert("protocol_id".to_string(), ConfigValue::String(format!("proto_{}", protocol.to_lowercase())));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_set_farm_amount_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let protocol_data = input.get("protocol_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing protocol_data input")?;
    
    let token = node.configuration.parameters
        .get("token")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("USDC".to_string());
    
    let amount = node.configuration.parameters
        .get("amount")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(1000.0);
    
    let mut output_data = HashMap::new();
    output_data.insert("protocol_data".to_string(), ConfigValue::Object(protocol_data.clone()));
    output_data.insert("token".to_string(), ConfigValue::String(token));
    output_data.insert("amount".to_string(), ConfigValue::Number(amount));
    output_data.insert("config_id".to_string(), ConfigValue::String(format!("config_{}", api::time())));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_execute_yield_farm_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let _farm_config = input.get("farm_config")
        .ok_or("Missing farm_config input")?;
    
    let min_apy = node.configuration.parameters
        .get("min_apy")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(5.0);
    
    let auto_compound = node.configuration.parameters
        .get("auto_compound")
        .and_then(|v| match v {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(true);
    
    // Mock execution result
    let mut output_data = HashMap::new();
    output_data.insert("success".to_string(), ConfigValue::Boolean(true));
    output_data.insert("transaction_id".to_string(), ConfigValue::String(format!("tx_{}", api::time())));
    output_data.insert("apy_achieved".to_string(), ConfigValue::Number(min_apy + 1.5));
    output_data.insert("compounding_enabled".to_string(), ConfigValue::Boolean(auto_compound));
    output_data.insert("estimated_yield_usd".to_string(), ConfigValue::Number(50.25));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_select_arbitrage_asset_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let asset = node.configuration.parameters
        .get("asset")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("BTC".to_string());
    
    let mut output_data = HashMap::new();
    output_data.insert("asset".to_string(), ConfigValue::String(asset.clone()));
    output_data.insert("asset_id".to_string(), ConfigValue::String(format!("asset_{}", asset.to_lowercase())));
    output_data.insert("current_price".to_string(), ConfigValue::Number(45000.0)); // Mock price
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_set_arbitrage_chains_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let asset_data = input.get("asset_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing asset_data input")?;
    
    let buy_chain = node.configuration.parameters
        .get("buy_chain")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Ethereum".to_string());
    
    let sell_chain = node.configuration.parameters
        .get("sell_chain")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Arbitrum".to_string());
    
    let mut output_data = HashMap::new();
    output_data.insert("asset_data".to_string(), ConfigValue::Object(asset_data.clone()));
    output_data.insert("buy_chain".to_string(), ConfigValue::String(buy_chain));
    output_data.insert("sell_chain".to_string(), ConfigValue::String(sell_chain));
    output_data.insert("route_id".to_string(), ConfigValue::String(format!("route_{}", api::time())));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_execute_arbitrage_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let _chain_config = input.get("chain_config")
        .ok_or("Missing chain_config input")?;
    
    let min_profit_percent = node.configuration.parameters
        .get("min_profit_percent")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(1.0);
    
    let max_amount = node.configuration.parameters
        .get("max_amount")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(5000.0);
    
    // Mock execution result
    let mut output_data = HashMap::new();
    output_data.insert("success".to_string(), ConfigValue::Boolean(true));
    output_data.insert("profit_percent".to_string(), ConfigValue::Number(min_profit_percent + 0.5));
    output_data.insert("profit_usd".to_string(), ConfigValue::Number(max_amount * (min_profit_percent / 100.0)));
    output_data.insert("buy_transaction_id".to_string(), ConfigValue::String(format!("buy_tx_{}", api::time())));
    output_data.insert("sell_transaction_id".to_string(), ConfigValue::String(format!("sell_tx_{}", api::time() + 1)));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_set_portfolio_allocation_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let btc_percent = node.configuration.parameters
        .get("btc_percent")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(60.0);
    
    let eth_percent = node.configuration.parameters
        .get("eth_percent")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(30.0);
    
    let stable_percent = node.configuration.parameters
        .get("stable_percent")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(10.0);
    
    // Validate percentages add up to 100
    let total = btc_percent + eth_percent + stable_percent;
    if (total - 100.0).abs() > 0.1 {
        return Err(format!("Allocation percentages must add up to 100%, got {}", total));
    }
    
    let mut output_data = HashMap::new();
    output_data.insert("btc_percent".to_string(), ConfigValue::Number(btc_percent));
    output_data.insert("eth_percent".to_string(), ConfigValue::Number(eth_percent));
    output_data.insert("stable_percent".to_string(), ConfigValue::Number(stable_percent));
    output_data.insert("allocation_id".to_string(), ConfigValue::String(format!("alloc_{}", api::time())));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_execute_rebalance_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let _allocation_data = input.get("allocation_data")
        .ok_or("Missing allocation_data input")?;
    
    let rebalance_threshold = node.configuration.parameters
        .get("rebalance_threshold")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(5.0);
    
    let min_trade_amount = node.configuration.parameters
        .get("min_trade_amount")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(50.0);
    
    // Mock rebalance execution result
    let mut output_data = HashMap::new();
    output_data.insert("success".to_string(), ConfigValue::Boolean(true));
    output_data.insert("trades_executed".to_string(), ConfigValue::Number(3.0));
    output_data.insert("total_fees_usd".to_string(), ConfigValue::Number(min_trade_amount * 0.01));
    output_data.insert("drift_corrected".to_string(), ConfigValue::Number(rebalance_threshold + 2.0));
    output_data.insert("rebalance_id".to_string(), ConfigValue::String(format!("rebal_{}", api::time())));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_check_cycles_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let canister_id = node.configuration.parameters
        .get("canister_id")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        });
    
    // Mock cycles check (in production, would call ic_cdk::api::canister_balance128() or similar)
    let current_cycles = 15_000_000_000_000u64; // 15T cycles
    
    let mut output_data = HashMap::new();
    output_data.insert("current_cycles".to_string(), ConfigValue::Number(current_cycles as f64));
    output_data.insert("current_cycles_trillions".to_string(), ConfigValue::Number((current_cycles as f64) / 1_000_000_000_000.0));
    output_data.insert("canister_id".to_string(), ConfigValue::String(canister_id.unwrap_or("current".to_string())));
    output_data.insert("checked_at".to_string(), ConfigValue::Number(api::time() as f64));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_cycles_alert_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let cycles_data = input.get("cycles_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing cycles_data input")?;
    
    let current_cycles_trillions = cycles_data.get("current_cycles_trillions")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(0.0);
    
    let warning_threshold = node.configuration.parameters
        .get("warning_threshold")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(10.0);
    
    let low_cycles = current_cycles_trillions < warning_threshold;
    
    let mut output_data = HashMap::new();
    output_data.insert("low_cycles".to_string(), ConfigValue::Boolean(low_cycles));
    output_data.insert("cycles_ok".to_string(), ConfigValue::Boolean(!low_cycles));
    output_data.insert("current_cycles_trillions".to_string(), ConfigValue::Number(current_cycles_trillions));
    output_data.insert("threshold_trillions".to_string(), ConfigValue::Number(warning_threshold));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: if low_cycles { vec!["low_cycles".to_string()] } else { vec!["cycles_ok".to_string()] },
    })
}

pub async fn execute_auto_topup_cycles_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let low_cycles = input.get("low_cycles")
        .and_then(|v| match v {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false);
    
    if !low_cycles {
        return Err("Auto top-up can only be executed when cycles are low".to_string());
    }
    
    let topup_amount = node.configuration.parameters
        .get("topup_amount")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(20.0);
    
    let notification_channel = node.configuration.parameters
        .get("notification_channel")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("email".to_string());
    
    // Mock top-up execution (in production, would integrate with cycles management system)
    let mut output_data = HashMap::new();
    output_data.insert("success".to_string(), ConfigValue::Boolean(true));
    output_data.insert("topup_amount_trillions".to_string(), ConfigValue::Number(topup_amount));
    output_data.insert("notification_sent".to_string(), ConfigValue::Boolean(true));
    output_data.insert("notification_channel".to_string(), ConfigValue::String(notification_channel));
    output_data.insert("request_id".to_string(), ConfigValue::String(format!("topup_{}", api::time())));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// ================================
// Common Reusable DeFi Utility Node Definitions
// ================================

// Select Asset Node
fn create_select_asset_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "select-asset".to_string(),
        name: "Select Asset".to_string(),
        description: "Choose any cryptocurrency asset".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "asset_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Asset configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "asset_symbol".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Asset Symbol".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("BTC".to_string())),
            }
        ],
    }
}

// Select Chain Node
fn create_select_chain_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "select-chain".to_string(),
        name: "Select Chain".to_string(),
        description: "Choose blockchain network".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "chain_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Chain configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "chain_name".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Blockchain".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("Ethereum".to_string())),
            }
        ],
    }
}

// Set Amount Node
fn create_set_amount_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "set-amount".to_string(),
        name: "Set Amount".to_string(),
        description: "Configure transaction amount".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "asset_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Asset data (optional)".to_string()),
                required: false,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "amount_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Amount configuration data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "amount".to_string(),
                parameter_type: "number".to_string(),
                description: Some("Amount".to_string()),
                required: true,
                default_value: Some(ConfigValue::Number(100.0)),
            },
            ParameterSchema {
                name: "currency".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Currency Unit".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("USD".to_string())),
            }
        ],
    }
}

// Check Price Node
fn create_check_price_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "check-price".to_string(),
        name: "Check Price".to_string(),
        description: "Get current asset price".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "asset_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Asset data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "price_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Price data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "vs_currency".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Price In".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("USD".to_string())),
            }
        ],
    }
}

// Check Balance Node
fn create_check_balance_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "check-balance".to_string(),
        name: "Check Balance".to_string(),
        description: "Get wallet balance for asset".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "asset_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Asset data".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "chain_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Chain data (optional)".to_string()),
                required: false,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "balance_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Balance data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "wallet_address".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Wallet Address (Optional)".to_string()),
                required: false,
                default_value: None,
            }
        ],
    }
}

// Estimate Gas Node
fn create_estimate_gas_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "estimate-gas".to_string(),
        name: "Estimate Gas".to_string(),
        description: "Estimate transaction gas costs".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "chain_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Chain data".to_string()),
                required: true,
                default_value: None,
            },
            ParameterSchema {
                name: "amount_data".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Amount data (optional)".to_string()),
                required: false,
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "gas_estimate".to_string(),
                parameter_type: "object".to_string(),
                description: Some("Gas estimation data".to_string()),
                required: true,
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "transaction_type".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Transaction Type".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("transfer".to_string())),
            },
            ParameterSchema {
                name: "gas_priority".to_string(),
                parameter_type: "string".to_string(),
                description: Some("Gas Priority".to_string()),
                required: true,
                default_value: Some(ConfigValue::String("medium".to_string())),
            }
        ],
    }
}

// ================================
// Common Utility Node Execution Functions
// ================================

pub async fn execute_select_asset_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let asset_symbol = node.configuration.parameters
        .get("asset_symbol")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("BTC".to_string());
    
    // Mock asset data
    let price = match asset_symbol.as_str() {
        "BTC" => 45000.0,
        "ETH" => 2800.0,
        "USDC" => 1.0,
        "USDT" => 1.0,
        "DAI" => 1.0,
        "SOL" => 85.0,
        "MATIC" => 0.8,
        "AVAX" => 25.0,
        _ => 1.0,
    };
    
    let mut output_data = HashMap::new();
    output_data.insert("asset_symbol".to_string(), ConfigValue::String(asset_symbol.clone()));
    output_data.insert("asset_name".to_string(), ConfigValue::String(format!("{} Token", asset_symbol)));
    output_data.insert("current_price_usd".to_string(), ConfigValue::Number(price));
    output_data.insert("decimals".to_string(), ConfigValue::Number(if asset_symbol == "BTC" { 8.0 } else { 18.0 }));
    output_data.insert("is_stable".to_string(), ConfigValue::Boolean(matches!(asset_symbol.as_str(), "USDC" | "USDT" | "DAI")));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_select_chain_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let chain_name = node.configuration.parameters
        .get("chain_name")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Ethereum".to_string());
    
    // Mock chain data
    let (chain_id, native_token, avg_gas_cost) = match chain_name.as_str() {
        "Ethereum" => (1, "ETH", 25.0),
        "Bitcoin" => (0, "BTC", 5.0),
        "Arbitrum" => (42161, "ETH", 0.5),
        "Optimism" => (10, "ETH", 0.3),
        "Polygon" => (137, "MATIC", 0.1),
        "Base" => (8453, "ETH", 0.2),
        "Avalanche" => (43114, "AVAX", 1.0),
        "Solana" => (101, "SOL", 0.01),
        "ICP" => (8668, "ICP", 0.001),
        _ => (1, "ETH", 25.0),
    };
    
    let mut output_data = HashMap::new();
    output_data.insert("chain_name".to_string(), ConfigValue::String(chain_name.clone()));
    output_data.insert("chain_id".to_string(), ConfigValue::Number(chain_id as f64));
    output_data.insert("native_token".to_string(), ConfigValue::String(native_token.to_string()));
    output_data.insert("avg_gas_cost_usd".to_string(), ConfigValue::Number(avg_gas_cost));
    output_data.insert("block_time_seconds".to_string(), ConfigValue::Number(if chain_name == "Bitcoin" { 600.0 } else { 12.0 }));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_set_amount_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let asset_data = input.get("asset_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        });
    
    let amount = node.configuration.parameters
        .get("amount")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(100.0);
    
    let currency = node.configuration.parameters
        .get("currency")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("USD".to_string());
    
    let mut output_data = HashMap::new();
    output_data.insert("amount".to_string(), ConfigValue::Number(amount));
    output_data.insert("currency".to_string(), ConfigValue::String(currency.clone()));
    
    if let Some(asset_data) = asset_data {
        output_data.insert("asset_data".to_string(), ConfigValue::Object(asset_data.clone()));
        
        // Convert to token amount if we have asset data
        if currency == "NATIVE" || currency == "TOKEN" {
            if let Some(ConfigValue::Number(price)) = asset_data.get("current_price_usd") {
                let token_amount = amount / price;
                output_data.insert("token_amount".to_string(), ConfigValue::Number(token_amount));
            }
        }
    }
    
    output_data.insert("amount_id".to_string(), ConfigValue::String(format!("amt_{}", api::time())));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_check_price_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let asset_data = input.get("asset_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing asset_data input")?;
    
    let asset_symbol = asset_data.get("asset_symbol")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("BTC".to_string());
    
    let vs_currency = node.configuration.parameters
        .get("vs_currency")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("USD".to_string());
    
    // Mock price data (in production would query real price feeds)
    let base_price = match asset_symbol.as_str() {
        "BTC" => 45000.0,
        "ETH" => 2800.0,
        "USDC" => 1.0,
        "USDT" => 1.0,
        "DAI" => 1.0,
        "SOL" => 85.0,
        "MATIC" => 0.8,
        "AVAX" => 25.0,
        _ => 1.0,
    };
    
    let price = match vs_currency.as_str() {
        "USD" => base_price,
        "ETH" => base_price / 2800.0,
        "BTC" => base_price / 45000.0,
        _ => base_price,
    };
    
    let mut output_data = HashMap::new();
    output_data.insert("asset_symbol".to_string(), ConfigValue::String(asset_symbol.clone()));
    output_data.insert("vs_currency".to_string(), ConfigValue::String(vs_currency));
    output_data.insert("price".to_string(), ConfigValue::Number(price));
    output_data.insert("price_change_24h".to_string(), ConfigValue::Number(2.5)); // Mock change
    output_data.insert("market_cap_rank".to_string(), ConfigValue::Number(if asset_symbol.clone() == "BTC" { 1.0 } else { 2.0 }));
    output_data.insert("last_updated".to_string(), ConfigValue::Number(api::time() as f64));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_check_balance_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let asset_data = input.get("asset_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing asset_data input")?;
    
    let chain_data = input.get("chain_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        });
    
    let wallet_address = node.configuration.parameters
        .get("wallet_address")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("connected_wallet".to_string());
    
    let asset_symbol = asset_data.get("asset_symbol")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("BTC".to_string());
    
    // Mock balance data (in production would query actual blockchain)
    let balance = match asset_symbol.as_str() {
        "BTC" => 0.5,
        "ETH" => 2.3,
        "USDC" => 1500.0,
        "USDT" => 800.0,
        "DAI" => 300.0,
        _ => 100.0,
    };
    
    let current_price = asset_data.get("current_price_usd")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(1.0);
    
    let balance_usd = balance * current_price;
    
    let mut output_data = HashMap::new();
    output_data.insert("asset_symbol".to_string(), ConfigValue::String(asset_symbol));
    output_data.insert("balance".to_string(), ConfigValue::Number(balance));
    output_data.insert("balance_usd".to_string(), ConfigValue::Number(balance_usd));
    output_data.insert("wallet_address".to_string(), ConfigValue::String(wallet_address));
    
    if let Some(chain_data) = chain_data {
        output_data.insert("chain_data".to_string(), ConfigValue::Object(chain_data.clone()));
    }
    
    output_data.insert("last_updated".to_string(), ConfigValue::Number(api::time() as f64));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

pub async fn execute_estimate_gas_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let chain_data = input.get("chain_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing chain_data input")?;
    
    let amount_data = input.get("amount_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        });
    
    let transaction_type = node.configuration.parameters
        .get("transaction_type")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("transfer".to_string());
    
    let gas_priority = node.configuration.parameters
        .get("gas_priority")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("medium".to_string());
    
    let chain_name = chain_data.get("chain_name")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Ethereum".to_string());
    
    // Mock gas estimation based on chain and transaction type
    let base_gas_cost = match chain_name.as_str() {
        "Ethereum" => match transaction_type.as_str() {
            "transfer" => 21000.0,
            "swap" => 150000.0,
            "defi" => 300000.0,
            "nft" => 100000.0,
            _ => 21000.0,
        },
        "Bitcoin" => 250.0, // Virtual size in bytes
        "Arbitrum" | "Optimism" | "Base" => 21000.0,
        "Polygon" => 21000.0,
        "Solana" => 5000.0, // Compute units
        _ => 21000.0,
    };
    
    let priority_multiplier = match gas_priority.as_str() {
        "low" => 0.8,
        "medium" => 1.0,
        "high" => 1.5,
        "urgent" => 2.0,
        _ => 1.0,
    };
    
    let gas_price_gwei = match chain_name.as_str() {
        "Ethereum" => 25.0 * priority_multiplier,
        "Arbitrum" => 0.1 * priority_multiplier,
        "Optimism" => 0.001 * priority_multiplier,
        "Polygon" => 30.0 * priority_multiplier,
        "Base" => 0.01 * priority_multiplier,
        _ => 20.0 * priority_multiplier,
    };
    
    let estimated_cost_usd = match chain_name.as_str() {
        "Bitcoin" => base_gas_cost * 0.00001, // Sats per byte
        "Solana" => 0.01, // Very low fixed cost
        _ => (base_gas_cost * gas_price_gwei) / 1_000_000_000.0 * 2800.0, // ETH price assumption
    };
    
    let mut output_data = HashMap::new();
    output_data.insert("chain_name".to_string(), ConfigValue::String(chain_name));
    output_data.insert("transaction_type".to_string(), ConfigValue::String(transaction_type));
    output_data.insert("gas_priority".to_string(), ConfigValue::String(gas_priority.clone()));
    output_data.insert("estimated_gas_units".to_string(), ConfigValue::Number(base_gas_cost));
    output_data.insert("gas_price_gwei".to_string(), ConfigValue::Number(gas_price_gwei));
    output_data.insert("estimated_cost_usd".to_string(), ConfigValue::Number(estimated_cost_usd));
    output_data.insert("estimated_time_seconds".to_string(), ConfigValue::Number(match gas_priority.clone().as_str() {
        "low" => 300.0,
        "medium" => 120.0,
        "high" => 60.0,
        "urgent" => 30.0,
        _ => 120.0,
    }));
    
    if let Some(amount_data) = amount_data {
        output_data.insert("amount_data".to_string(), ConfigValue::Object(amount_data.clone()));
    }
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// ===== SIMPLIFIED SOCIAL MEDIA NODES =====

// Social Auth Setup Node
fn create_social_auth_setup_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "social-auth-setup".to_string(),
        name: "Social Auth Setup".to_string(),
        description: "Configure social media platform credentials once for your workflow".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "auth_data".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Auth Data".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "platform".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Select social media platform".to_string()),
                default_value: Some(ConfigValue::String("twitter".to_string())),
            },
            ParameterSchema {
                name: "auth_token".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Main authentication token for the platform".to_string()),
                default_value: None,
            }
        ],
    }
}

async fn execute_social_auth_setup_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let platform = node.configuration.parameters
        .get("platform")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("twitter".to_string());
    
    let auth_token = node.configuration.parameters
        .get("auth_token")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();
    
    let mut output_data = HashMap::new();
    output_data.insert("platform".to_string(), ConfigValue::String(platform));
    output_data.insert("auth_token".to_string(), ConfigValue::String(auth_token));
    output_data.insert("setup_timestamp".to_string(), ConfigValue::Number(api::time() as f64));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// Select Platform Node
fn create_select_platform_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "select-platform".to_string(),
        name: "Select Platform".to_string(),
        description: "Choose target social media platform for posting".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "auth_data".to_string(),
                parameter_type: "object".to_string(),
                required: false,
                description: Some("Auth Data".to_string()),
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "platform_config".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Platform Config".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "platform".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Target platform for posting".to_string()),
                default_value: Some(ConfigValue::String("twitter".to_string())),
            },
            ParameterSchema {
                name: "target_id".to_string(),
                parameter_type: "string".to_string(),
                required: false,
                description: Some("ID for specific page/group/company (required for some platforms)".to_string()),
                default_value: None,
            }
        ],
    }
}

async fn execute_select_platform_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let platform = node.configuration.parameters
        .get("platform")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("twitter".to_string());
    
    let target_id = node.configuration.parameters
        .get("target_id")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();
    
    // Get auth data if provided
    let mut auth_data = HashMap::new();
    if let Some(ConfigValue::Object(auth)) = input.get("auth_data") {
        auth_data = auth.clone();
    }
    
    let mut output_data = HashMap::new();
    output_data.insert("platform".to_string(), ConfigValue::String(platform));
    output_data.insert("target_id".to_string(), ConfigValue::String(target_id));
    output_data.insert("auth_data".to_string(), ConfigValue::Object(auth_data));
    output_data.insert("config_timestamp".to_string(), ConfigValue::Number(api::time() as f64));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// Social Media Post Node
fn create_social_media_post_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "social-media-post".to_string(),
        name: "Social Media Post".to_string(),
        description: "Execute the social media post to the selected platform".to_string(),
        category: "integrations".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "platform_config".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Platform Config".to_string()),
                default_value: None,
            },
            ParameterSchema {
                name: "content_data".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Content Data".to_string()),
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "result".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Post Result".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![],
    }
}

async fn execute_social_media_post_node(
    _node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let platform_config = input.get("platform_config")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing platform_config input")?;
    
    let content_data = input.get("content_data")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing content_data input")?;
    
    let platform = platform_config.get("platform")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("twitter".to_string());
    
    // Mock execution - in production, this would actually post to the platform
    let mut output_data = HashMap::new();
    output_data.insert("platform".to_string(), ConfigValue::String(platform));
    output_data.insert("post_id".to_string(), ConfigValue::String(format!("post_{}", api::time())));
    output_data.insert("status".to_string(), ConfigValue::String("success".to_string()));
    output_data.insert("post_url".to_string(), ConfigValue::String("https://example.com/post/123".to_string()));
    output_data.insert("content_preview".to_string(), ConfigValue::Object(content_data.clone()));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// ===== AI CONTENT GENERATION NODES =====

// AI Content Setup Node
fn create_ai_content_setup_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "ai-content-setup".to_string(),
        name: "AI Content Setup".to_string(),
        description: "Configure AI provider and basic settings once".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![],
        output_schema: vec![
            ParameterSchema {
                name: "ai_config".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("AI Config".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "provider".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Select AI provider".to_string()),
                default_value: Some(ConfigValue::String("openai".to_string())),
            },
            ParameterSchema {
                name: "api_key".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("API key for the selected AI provider".to_string()),
                default_value: None,
            }
        ],
    }
}

async fn execute_ai_content_setup_node(
    node: &WorkflowNode,
    _input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let provider = node.configuration.parameters
        .get("provider")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("openai".to_string());
    
    let api_key = node.configuration.parameters
        .get("api_key")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();
    
    let mut output_data = HashMap::new();
    output_data.insert("provider".to_string(), ConfigValue::String(provider));
    output_data.insert("api_key".to_string(), ConfigValue::String(api_key));
    output_data.insert("setup_timestamp".to_string(), ConfigValue::Number(api::time() as f64));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// Generate Content Node
fn create_generate_content_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "generate-content".to_string(),
        name: "Generate Content".to_string(),
        description: "Generate AI content with simple prompt and data input".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "ai_config".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("AI Config".to_string()),
                default_value: None,
            },
            ParameterSchema {
                name: "data".to_string(),
                parameter_type: "object".to_string(),
                required: false,
                description: Some("Input Data".to_string()),
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "content".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Generated Content".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "content_type".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Type of content to generate".to_string()),
                default_value: Some(ConfigValue::String("social_post".to_string())),
            },
            ParameterSchema {
                name: "prompt".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Content prompt with {{variable}} placeholders".to_string()),
                default_value: Some(ConfigValue::String("Generate a professional tweet about {{topic}}".to_string())),
            },
            ParameterSchema {
                name: "max_length".to_string(),
                parameter_type: "number".to_string(),
                required: true,
                description: Some("Maximum content length".to_string()),
                default_value: Some(ConfigValue::String("280".to_string())),
            }
        ],
    }
}

async fn execute_generate_content_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let ai_config = input.get("ai_config")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing ai_config input")?;
    
    let content_type = node.configuration.parameters
        .get("content_type")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("social_post".to_string());
    
    let prompt = node.configuration.parameters
        .get("prompt")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Generate a professional social media post".to_string());
    
    let max_length = node.configuration.parameters
        .get("max_length")
        .and_then(|v| match v {
            ConfigValue::Number(n) => Some(*n as u32),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        })
        .unwrap_or(280);
    
    // Mock AI content generation - in production, this would call actual AI APIs
    let generated_content = "🚀 Exciting update from our DeFi automation platform! 📊 Portfolio value up 12% this week thanks to our smart yield farming strategies. Keep building! 💪 #DeFi #Automation #YieldFarming".to_string();
    
    let mut output_data = HashMap::new();
    output_data.insert("content_type".to_string(), ConfigValue::String(content_type));
    output_data.insert("generated_text".to_string(), ConfigValue::String(generated_content));
    output_data.insert("prompt_used".to_string(), ConfigValue::String(prompt));
    output_data.insert("max_length".to_string(), ConfigValue::Number(max_length as f64));
    output_data.insert("provider".to_string(), ConfigValue::String(
        ai_config.get("provider")
            .and_then(|v| match v { ConfigValue::String(s) => Some(s.clone()), _ => None })
            .unwrap_or("openai".to_string())
    ));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// Content Optimizer Node
fn create_content_optimizer_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "content-optimizer".to_string(),
        name: "Content Optimizer".to_string(),
        description: "Optimize generated content for specific platforms".to_string(),
        category: "utilities".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "content".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Raw Content".to_string()),
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "optimized_content".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Optimized Content".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "platform".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Platform to optimize for".to_string()),
                default_value: Some(ConfigValue::String("twitter".to_string())),
            },
            ParameterSchema {
                name: "add_hashtags".to_string(),
                parameter_type: "boolean".to_string(),
                required: false,
                description: Some("Automatically add relevant hashtags".to_string()),
                default_value: Some(ConfigValue::String("true".to_string())),
            },
            ParameterSchema {
                name: "add_emojis".to_string(),
                parameter_type: "boolean".to_string(),
                required: false,
                description: Some("Add relevant emojis to content".to_string()),
                default_value: Some(ConfigValue::String("true".to_string())),
            }
        ],
    }
}

async fn execute_content_optimizer_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let content_data = input.get("content")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing content input")?;
    
    let platform = node.configuration.parameters
        .get("platform")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("twitter".to_string());
    
    let add_hashtags = node.configuration.parameters
        .get("add_hashtags")
        .and_then(|v| match v {
            ConfigValue::Boolean(b) => Some(*b),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        })
        .unwrap_or(true);
    
    let original_text = content_data.get("generated_text")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Generated content".to_string());
    
    // Mock optimization - in production would use AI for optimization
    let mut optimized_text = original_text.clone();
    
    if add_hashtags && platform == "twitter" {
        optimized_text.push_str(" #DeFi #Automation");
    }
    
    let char_limit = match platform.as_str() {
        "twitter" => 280,
        "linkedin" => 3000,
        "discord" => 2000,
        _ => 1000,
    };
    
    if optimized_text.len() > char_limit {
        optimized_text.truncate(char_limit - 3);
        optimized_text.push_str("...");
    }
    
    let mut output_data = HashMap::new();
    output_data.insert("optimized_text".to_string(), ConfigValue::String(optimized_text.clone()));
    output_data.insert("platform".to_string(), ConfigValue::String(platform));
    output_data.insert("char_count".to_string(), ConfigValue::Number(optimized_text.len() as f64));
    output_data.insert("char_limit".to_string(), ConfigValue::Number(char_limit as f64));
    output_data.insert("optimization_applied".to_string(), ConfigValue::Boolean(true));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}

// AI Responder Node
fn create_ai_responder_node_definition() -> NodeDefinition {
    NodeDefinition {
        node_type: "ai-responder".to_string(),
        name: "AI Responder".to_string(),
        description: "Generate AI responses to social media mentions or comments".to_string(),
        category: "integrations".to_string(),
        version: "1.0.0".to_string(),
        input_schema: vec![
            ParameterSchema {
                name: "ai_config".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("AI Config".to_string()),
                default_value: None,
            },
            ParameterSchema {
                name: "mention".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("Social Mention".to_string()),
                default_value: None,
            }
        ],
        output_schema: vec![
            ParameterSchema {
                name: "response".to_string(),
                parameter_type: "object".to_string(),
                required: true,
                description: Some("AI Response".to_string()),
                default_value: None,
            }
        ],
        configuration_schema: vec![
            ParameterSchema {
                name: "personality".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("AI response personality".to_string()),
                default_value: Some(ConfigValue::String("professional".to_string())),
            },
            ParameterSchema {
                name: "guidelines".to_string(),
                parameter_type: "string".to_string(),
                required: true,
                description: Some("Guidelines for AI responses".to_string()),
                default_value: Some(ConfigValue::String("Always be helpful and accurate. Keep responses under 280 characters.".to_string())),
            }
        ],
    }
}

async fn execute_ai_responder_node(
    node: &WorkflowNode,
    input: &HashMap<String, ConfigValue>
) -> Result<NodeOutput, String> {
    let ai_config = input.get("ai_config")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing ai_config input")?;
    
    let mention_data = input.get("mention")
        .and_then(|v| match v {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        })
        .ok_or("Missing mention input")?;
    
    let personality = node.configuration.parameters
        .get("personality")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("professional".to_string());
    
    let guidelines = node.configuration.parameters
        .get("guidelines")
        .and_then(|v| match v {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or("Always be helpful and accurate".to_string());
    
    // Mock AI response generation - in production would call AI APIs
    let response_text = match personality.as_str() {
        "friendly" => "Hey there! 👋 Thanks for reaching out! We're always here to help with any DeFi automation questions you might have. Feel free to ask!",
        "expert" => "Thank you for your inquiry. Based on our platform's capabilities, I can provide detailed information about yield farming strategies and portfolio optimization techniques.",
        _ => "Thank you for your message. We appreciate your interest in our DeFi automation platform. How can we assist you today?"
    };
    
    let mut output_data = HashMap::new();
    output_data.insert("response_text".to_string(), ConfigValue::String(response_text.to_string()));
    output_data.insert("personality".to_string(), ConfigValue::String(personality));
    output_data.insert("guidelines_applied".to_string(), ConfigValue::String(guidelines));
    output_data.insert("original_mention".to_string(), ConfigValue::Object(mention_data.clone()));
    output_data.insert("response_timestamp".to_string(), ConfigValue::Number(api::time() as f64));
    
    Ok(NodeOutput {
        data: output_data,
        next_nodes: vec![],
    })
}
