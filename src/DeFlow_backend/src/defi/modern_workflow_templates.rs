// Modern Workflow Templates - Updated to match reorganized functional block inputs
// Properly segregates inputs, outputs, and configuration parameters

use crate::types::{ParameterSchema, ConfigValue, NodeDefinition};
use super::yield_farming::{ChainId, DeFiProtocol, UniswapVersion};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

/// Modern workflow template with segregated parameter schemas
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ModernWorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: WorkflowCategory,
    pub difficulty: DifficultyLevel,
    pub estimated_apy: f64,
    pub risk_score: u8,
    pub min_capital_usd: f64,
    pub nodes: Vec<WorkflowNodeTemplate>,
    pub connections: Vec<NodeConnectionTemplate>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum WorkflowCategory {
    YieldFarming,
    Arbitrage,
    Rebalancing,
    DCA,
    RiskManagement,
    Social,
    Portfolio,
    CrossChain,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct WorkflowNodeTemplate {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub description: String,
    pub position: NodePosition,
    // Segregated parameter schemas
    pub input_parameters: Vec<ParameterTemplate>,
    pub output_parameters: Vec<ParameterTemplate>,
    pub configuration_parameters: Vec<ParameterTemplate>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ParameterTemplate {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub description: Option<String>,
    pub default_value: Option<ConfigValue>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: String,
    pub value: Option<ConfigValue>,
    pub message: String,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct NodeConnectionTemplate {
    pub id: String,
    pub source_node_id: String,
    pub source_output: String,
    pub target_node_id: String,
    pub target_input: String,
}

/// Modern Workflow Template Manager
pub struct ModernWorkflowTemplateManager {
    templates: HashMap<String, ModernWorkflowTemplate>,
}

impl ModernWorkflowTemplateManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        manager.initialize_modern_templates();
        manager
    }

    fn initialize_modern_templates(&mut self) {
        // 1. Conservative Yield Farming Template
        self.add_template(self.create_conservative_yield_farming_template());

        // 2. Cross-Chain Arbitrage Template
        self.add_template(self.create_cross_chain_arbitrage_template());

        // 3. Social Media Integration Template
        self.add_template(self.create_social_media_template());

        // 4. Portfolio Management Template
        self.add_template(self.create_portfolio_management_template());

        // 5. Multi-Chain DCA Template
        self.add_template(self.create_multi_chain_dca_template());
    }

    fn add_template(&mut self, template: ModernWorkflowTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn get_all_templates(&self) -> Vec<&ModernWorkflowTemplate> {
        self.templates.values().collect()
    }

    pub fn get_template(&self, template_id: &str) -> Option<&ModernWorkflowTemplate> {
        self.templates.get(template_id)
    }

    pub fn get_templates_by_category(&self, category: &WorkflowCategory) -> Vec<&ModernWorkflowTemplate> {
        self.templates
            .values()
            .filter(|template| std::mem::discriminant(&template.category) == std::mem::discriminant(category))
            .collect()
    }

    /// Conservative Yield Farming Template
    fn create_conservative_yield_farming_template(&self) -> ModernWorkflowTemplate {
        ModernWorkflowTemplate {
            id: "conservative_yield_farming_v2".to_string(),
            name: "Conservative Yield Farming (Modern)".to_string(),
            description: "Low-risk yield farming with proper input/output segregation".to_string(),
            category: WorkflowCategory::YieldFarming,
            difficulty: DifficultyLevel::Beginner,
            estimated_apy: 4.5,
            risk_score: 3,
            min_capital_usd: 100.0,
            nodes: vec![
                // 1. Portfolio Check Node
                WorkflowNodeTemplate {
                    id: "portfolio_check".to_string(),
                    node_type: "ethereum_portfolio".to_string(),
                    name: "Check Portfolio".to_string(),
                    description: "Get current portfolio status".to_string(),
                    position: NodePosition { x: 100.0, y: 100.0 },
                    input_parameters: vec![], // No inputs required for portfolio check
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "total_value_usd".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Total portfolio value in USD".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "available_balance".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Available balance for investment".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "chain".to_string(),
                            parameter_type: "string".to_string(),
                            required: false,
                            description: Some("Blockchain to check (default: Ethereum)".to_string()),
                            default_value: Some(ConfigValue::String("ethereum".to_string())),
                            validation_rules: vec![],
                        },
                    ],
                },
                // 2. Yield Strategy Node
                WorkflowNodeTemplate {
                    id: "yield_strategy".to_string(),
                    node_type: "yield_farming".to_string(),
                    name: "Execute Yield Strategy".to_string(),
                    description: "Deploy funds to yield farming protocols".to_string(),
                    position: NodePosition { x: 300.0, y: 100.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "investment_amount".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Amount to invest in yield farming".to_string()),
                            default_value: None,
                            validation_rules: vec![
                                ValidationRule {
                                    rule_type: "min_value".to_string(),
                                    value: Some(ConfigValue::Number(100.0)),
                                    message: "Minimum investment is $100".to_string(),
                                },
                            ],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "transaction_hash".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Transaction hash of yield deployment".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "apy_rate".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Current APY rate achieved".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "protocol".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("DeFi protocol to use".to_string()),
                            default_value: Some(ConfigValue::String("aave".to_string())),
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "asset".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Asset to farm with".to_string()),
                            default_value: Some(ConfigValue::String("USDC".to_string())),
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "auto_compound".to_string(),
                            parameter_type: "boolean".to_string(),
                            required: false,
                            description: Some("Enable auto-compounding".to_string()),
                            default_value: Some(ConfigValue::Boolean(true)),
                            validation_rules: vec![],
                        },
                    ],
                },
                // 3. Discord Notification Node
                WorkflowNodeTemplate {
                    id: "discord_notification".to_string(),
                    node_type: "discord".to_string(),
                    name: "Discord Notification".to_string(),
                    description: "Send success notification to Discord".to_string(),
                    position: NodePosition { x: 500.0, y: 100.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "transaction_hash".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Transaction hash to include in notification".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "apy_rate".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("APY rate to include in notification".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "notification_sent".to_string(),
                            parameter_type: "boolean".to_string(),
                            required: true,
                            description: Some("Whether notification was sent successfully".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "message_template".to_string(),
                            parameter_type: "string".to_string(),
                            required: false,
                            description: Some("Discord message template".to_string()),
                            default_value: Some(ConfigValue::String("🎯 Yield farming deployed! TX: ${transaction_hash}, APY: ${apy_rate}%".to_string())),
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "webhook_url".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Discord webhook URL".to_string()),
                            default_value: None,
                            validation_rules: vec![
                                ValidationRule {
                                    rule_type: "pattern".to_string(),
                                    value: Some(ConfigValue::String("^https://discord(app)?\\.com/api/webhooks/".to_string())),
                                    message: "Must be a valid Discord webhook URL".to_string(),
                                },
                            ],
                        },
                    ],
                },
            ],
            connections: vec![
                NodeConnectionTemplate {
                    id: "conn_1".to_string(),
                    source_node_id: "portfolio_check".to_string(),
                    source_output: "available_balance".to_string(),
                    target_node_id: "yield_strategy".to_string(),
                    target_input: "investment_amount".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_2".to_string(),
                    source_node_id: "yield_strategy".to_string(),
                    source_output: "transaction_hash".to_string(),
                    target_node_id: "discord_notification".to_string(),
                    target_input: "transaction_hash".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_3".to_string(),
                    source_node_id: "yield_strategy".to_string(),
                    source_output: "apy_rate".to_string(),
                    target_node_id: "discord_notification".to_string(),
                    target_input: "apy_rate".to_string(),
                },
            ],
        }
    }

    /// Cross-Chain Arbitrage Template
    fn create_cross_chain_arbitrage_template(&self) -> ModernWorkflowTemplate {
        ModernWorkflowTemplate {
            id: "cross_chain_arbitrage_v2".to_string(),
            name: "Cross-Chain Arbitrage (Modern)".to_string(),
            description: "Automated arbitrage with proper parameter segregation".to_string(),
            category: WorkflowCategory::Arbitrage,
            difficulty: DifficultyLevel::Advanced,
            estimated_apy: 12.0,
            risk_score: 7,
            min_capital_usd: 1000.0,
            nodes: vec![
                // 1. L2 Optimization Node
                WorkflowNodeTemplate {
                    id: "l2_optimization".to_string(),
                    node_type: "l2_optimization".to_string(),
                    name: "Find Optimal L2".to_string(),
                    description: "Find the best L2 chain for arbitrage".to_string(),
                    position: NodePosition { x: 100.0, y: 150.0 },
                    input_parameters: vec![], // No inputs required
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "optimal_chain".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Optimal L2 chain for arbitrage".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "estimated_gas_cost".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Estimated gas cost in USD".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "target_chains".to_string(),
                            parameter_type: "array".to_string(),
                            required: false,
                            description: Some("L2 chains to consider".to_string()),
                            default_value: Some(ConfigValue::Array(vec![
                                ConfigValue::String("arbitrum".to_string()),
                                ConfigValue::String("polygon".to_string()),
                                ConfigValue::String("optimism".to_string()),
                            ])),
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "prioritize_speed".to_string(),
                            parameter_type: "boolean".to_string(),
                            required: false,
                            description: Some("Prioritize speed over cost".to_string()),
                            default_value: Some(ConfigValue::Boolean(false)),
                            validation_rules: vec![],
                        },
                    ],
                },
                // 2. Arbitrage Execution Node
                WorkflowNodeTemplate {
                    id: "arbitrage_execution".to_string(),
                    node_type: "arbitrage_bot".to_string(),
                    name: "Execute Arbitrage".to_string(),
                    description: "Execute arbitrage opportunity".to_string(),
                    position: NodePosition { x: 300.0, y: 150.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "target_chain".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Chain to execute arbitrage on".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "max_gas_cost".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Maximum acceptable gas cost".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "profit_usd".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Profit achieved in USD".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "execution_time".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Execution time in seconds".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "min_profit_threshold".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Minimum profit threshold in USD".to_string()),
                            default_value: Some(ConfigValue::Number(50.0)),
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "slippage_tolerance".to_string(),
                            parameter_type: "number".to_string(),
                            required: false,
                            description: Some("Maximum slippage tolerance (%)".to_string()),
                            default_value: Some(ConfigValue::Number(0.5)),
                            validation_rules: vec![],
                        },
                    ],
                },
                // 3. Telegram Success Notification
                WorkflowNodeTemplate {
                    id: "telegram_notification".to_string(),
                    node_type: "telegram".to_string(),
                    name: "Telegram Alert".to_string(),
                    description: "Send arbitrage results to Telegram".to_string(),
                    position: NodePosition { x: 500.0, y: 150.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "profit_amount".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Profit amount to report".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "execution_time".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Time taken for execution".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "message_sent".to_string(),
                            parameter_type: "boolean".to_string(),
                            required: true,
                            description: Some("Whether message was sent successfully".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "bot_token".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Telegram bot token".to_string()),
                            default_value: None,
                            validation_rules: vec![
                                ValidationRule {
                                    rule_type: "pattern".to_string(),
                                    value: Some(ConfigValue::String("^[0-9]+:".to_string())),
                                    message: "Must be a valid Telegram bot token".to_string(),
                                },
                            ],
                        },
                        ParameterTemplate {
                            name: "chat_id".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Telegram chat ID".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "message_template".to_string(),
                            parameter_type: "string".to_string(),
                            required: false,
                            description: Some("Message template for arbitrage alerts".to_string()),
                            default_value: Some(ConfigValue::String("🔄 Arbitrage completed! Profit: $${profit_amount} in ${execution_time}s".to_string())),
                            validation_rules: vec![],
                        },
                    ],
                },
            ],
            connections: vec![
                NodeConnectionTemplate {
                    id: "conn_1".to_string(),
                    source_node_id: "l2_optimization".to_string(),
                    source_output: "optimal_chain".to_string(),
                    target_node_id: "arbitrage_execution".to_string(),
                    target_input: "target_chain".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_2".to_string(),
                    source_node_id: "l2_optimization".to_string(),
                    source_output: "estimated_gas_cost".to_string(),
                    target_node_id: "arbitrage_execution".to_string(),
                    target_input: "max_gas_cost".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_3".to_string(),
                    source_node_id: "arbitrage_execution".to_string(),
                    source_output: "profit_usd".to_string(),
                    target_node_id: "telegram_notification".to_string(),
                    target_input: "profit_amount".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_4".to_string(),
                    source_node_id: "arbitrage_execution".to_string(),
                    source_output: "execution_time".to_string(),
                    target_node_id: "telegram_notification".to_string(),
                    target_input: "execution_time".to_string(),
                },
            ],
        }
    }

    /// Social Media Integration Template
    fn create_social_media_template(&self) -> ModernWorkflowTemplate {
        ModernWorkflowTemplate {
            id: "social_media_integration".to_string(),
            name: "Social Media Integration".to_string(),
            description: "Multi-platform social media posting workflow".to_string(),
            category: WorkflowCategory::Social,
            difficulty: DifficultyLevel::Intermediate,
            estimated_apy: 0.0,
            risk_score: 1,
            min_capital_usd: 0.0,
            nodes: vec![
                // 1. Timer Trigger
                WorkflowNodeTemplate {
                    id: "daily_timer".to_string(),
                    node_type: "timer".to_string(),
                    name: "Daily Timer".to_string(),
                    description: "Trigger daily portfolio updates".to_string(),
                    position: NodePosition { x: 100.0, y: 200.0 },
                    input_parameters: vec![], // Timer is a trigger node
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "triggered_at".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Timestamp when triggered".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "interval".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Interval in milliseconds (24h = 86400000)".to_string()),
                            default_value: Some(ConfigValue::Number(86400000.0)),
                            validation_rules: vec![],
                        },
                    ],
                },
                // 2. Portfolio Summary
                WorkflowNodeTemplate {
                    id: "portfolio_summary".to_string(),
                    node_type: "bitcoin_portfolio".to_string(),
                    name: "Get Portfolio Summary".to_string(),
                    description: "Get current portfolio status for social posting".to_string(),
                    position: NodePosition { x: 300.0, y: 200.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "trigger_time".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Time when summary was triggered".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "total_value_usd".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Total portfolio value".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "daily_change_percent".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("24h change percentage".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![], // No configuration needed
                },
                // 3. Multi-Platform Social Post
                WorkflowNodeTemplate {
                    id: "multi_social_post".to_string(),
                    node_type: "social_media_post".to_string(),
                    name: "Post to All Platforms".to_string(),
                    description: "Post portfolio update to multiple social platforms".to_string(),
                    position: NodePosition { x: 500.0, y: 200.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "portfolio_value".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Portfolio value to include in post".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "change_percent".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Change percentage to include in post".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "posts_sent".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Number of successful posts".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "platforms".to_string(),
                            parameter_type: "array".to_string(),
                            required: true,
                            description: Some("Social platforms to post to".to_string()),
                            default_value: Some(ConfigValue::Array(vec![
                                ConfigValue::String("telegram".to_string()),
                                ConfigValue::String("discord".to_string()),
                            ])),
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "message_template".to_string(),
                            parameter_type: "string".to_string(),
                            required: false,
                            description: Some("Message template for social posts".to_string()),
                            default_value: Some(ConfigValue::String("📊 Daily Portfolio Update: $${portfolio_value} (${change_percent}% 24h change)".to_string())),
                            validation_rules: vec![],
                        },
                    ],
                },
            ],
            connections: vec![
                NodeConnectionTemplate {
                    id: "conn_1".to_string(),
                    source_node_id: "daily_timer".to_string(),
                    source_output: "triggered_at".to_string(),
                    target_node_id: "portfolio_summary".to_string(),
                    target_input: "trigger_time".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_2".to_string(),
                    source_node_id: "portfolio_summary".to_string(),
                    source_output: "total_value_usd".to_string(),
                    target_node_id: "multi_social_post".to_string(),
                    target_input: "portfolio_value".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_3".to_string(),
                    source_node_id: "portfolio_summary".to_string(),
                    source_output: "daily_change_percent".to_string(),
                    target_node_id: "multi_social_post".to_string(),
                    target_input: "change_percent".to_string(),
                },
            ],
        }
    }

    /// Portfolio Management Template
    fn create_portfolio_management_template(&self) -> ModernWorkflowTemplate {
        ModernWorkflowTemplate {
            id: "portfolio_management_v2".to_string(),
            name: "Advanced Portfolio Management".to_string(),
            description: "Comprehensive portfolio management with rebalancing".to_string(),
            category: WorkflowCategory::Portfolio,
            difficulty: DifficultyLevel::Advanced,
            estimated_apy: 8.0,
            risk_score: 5,
            min_capital_usd: 5000.0,
            nodes: vec![
                // 1. Multi-Chain Portfolio Check
                WorkflowNodeTemplate {
                    id: "multi_portfolio_check".to_string(),
                    node_type: "ethereum_portfolio".to_string(),
                    name: "Multi-Chain Portfolio Check".to_string(),
                    description: "Check portfolio across all chains".to_string(),
                    position: NodePosition { x: 100.0, y: 250.0 },
                    input_parameters: vec![], // No inputs required
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "total_value_usd".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Total portfolio value across all chains".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "allocation_breakdown".to_string(),
                            parameter_type: "object".to_string(),
                            required: true,
                            description: Some("Asset allocation breakdown".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "include_chains".to_string(),
                            parameter_type: "array".to_string(),
                            required: false,
                            description: Some("Chains to include in portfolio check".to_string()),
                            default_value: Some(ConfigValue::Array(vec![
                                ConfigValue::String("ethereum".to_string()),
                                ConfigValue::String("arbitrum".to_string()),
                                ConfigValue::String("polygon".to_string()),
                            ])),
                            validation_rules: vec![],
                        },
                    ],
                },
                // 2. Rebalancing Logic
                WorkflowNodeTemplate {
                    id: "rebalancing_logic".to_string(),
                    node_type: "portfolio_rebalancer".to_string(),
                    name: "Portfolio Rebalancing".to_string(),
                    description: "Analyze and execute portfolio rebalancing".to_string(),
                    position: NodePosition { x: 300.0, y: 250.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "current_allocation".to_string(),
                            parameter_type: "object".to_string(),
                            required: true,
                            description: Some("Current asset allocation".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "total_value".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Total portfolio value".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "rebalance_actions".to_string(),
                            parameter_type: "array".to_string(),
                            required: true,
                            description: Some("List of rebalancing actions taken".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "gas_costs_total".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Total gas costs for rebalancing".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "target_allocation".to_string(),
                            parameter_type: "object".to_string(),
                            required: true,
                            description: Some("Target asset allocation percentages".to_string()),
                            default_value: Some(ConfigValue::Object(HashMap::new())), // Would be populated with default allocations
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "rebalance_threshold".to_string(),
                            parameter_type: "number".to_string(),
                            required: false,
                            description: Some("Threshold for triggering rebalance (%)".to_string()),
                            default_value: Some(ConfigValue::Number(5.0)),
                            validation_rules: vec![],
                        },
                    ],
                },
            ],
            connections: vec![
                NodeConnectionTemplate {
                    id: "conn_1".to_string(),
                    source_node_id: "multi_portfolio_check".to_string(),
                    source_output: "allocation_breakdown".to_string(),
                    target_node_id: "rebalancing_logic".to_string(),
                    target_input: "current_allocation".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_2".to_string(),
                    source_node_id: "multi_portfolio_check".to_string(),
                    source_output: "total_value_usd".to_string(),
                    target_node_id: "rebalancing_logic".to_string(),
                    target_input: "total_value".to_string(),
                },
            ],
        }
    }

    /// Multi-Chain DCA Template
    fn create_multi_chain_dca_template(&self) -> ModernWorkflowTemplate {
        ModernWorkflowTemplate {
            id: "multi_chain_dca_v2".to_string(),
            name: "Multi-Chain Dollar Cost Averaging".to_string(),
            description: "DCA across multiple chains with cost optimization".to_string(),
            category: WorkflowCategory::DCA,
            difficulty: DifficultyLevel::Intermediate,
            estimated_apy: 6.0,
            risk_score: 3,
            min_capital_usd: 200.0,
            nodes: vec![
                // 1. Gas Estimation across chains
                WorkflowNodeTemplate {
                    id: "gas_estimation".to_string(),
                    node_type: "ethereum_gas_estimate".to_string(),
                    name: "Multi-Chain Gas Estimation".to_string(),
                    description: "Estimate gas costs across multiple chains".to_string(),
                    position: NodePosition { x: 100.0, y: 300.0 },
                    input_parameters: vec![], // No inputs required
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "cheapest_chain".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Chain with lowest transaction cost".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "estimated_cost_usd".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Estimated transaction cost in USD".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "chains_to_check".to_string(),
                            parameter_type: "array".to_string(),
                            required: false,
                            description: Some("Chains to check for gas costs".to_string()),
                            default_value: Some(ConfigValue::Array(vec![
                                ConfigValue::String("ethereum".to_string()),
                                ConfigValue::String("arbitrum".to_string()),
                                ConfigValue::String("polygon".to_string()),
                            ])),
                            validation_rules: vec![],
                        },
                    ],
                },
                // 2. DCA Execution
                WorkflowNodeTemplate {
                    id: "dca_execution".to_string(),
                    node_type: "dca_bot".to_string(),
                    name: "Execute DCA Purchase".to_string(),
                    description: "Execute DCA purchase on optimal chain".to_string(),
                    position: NodePosition { x: 300.0, y: 300.0 },
                    input_parameters: vec![
                        ParameterTemplate {
                            name: "target_chain".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Chain to execute DCA on".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "max_gas_cost".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Maximum acceptable gas cost".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    output_parameters: vec![
                        ParameterTemplate {
                            name: "tokens_purchased".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Amount of tokens purchased".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "average_price".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Average purchase price".to_string()),
                            default_value: None,
                            validation_rules: vec![],
                        },
                    ],
                    configuration_parameters: vec![
                        ParameterTemplate {
                            name: "target_asset".to_string(),
                            parameter_type: "string".to_string(),
                            required: true,
                            description: Some("Asset to purchase with DCA".to_string()),
                            default_value: Some(ConfigValue::String("ETH".to_string())),
                            validation_rules: vec![],
                        },
                        ParameterTemplate {
                            name: "purchase_amount_usd".to_string(),
                            parameter_type: "number".to_string(),
                            required: true,
                            description: Some("Amount to purchase in USD".to_string()),
                            default_value: Some(ConfigValue::Number(100.0)),
                            validation_rules: vec![
                                ValidationRule {
                                    rule_type: "min_value".to_string(),
                                    value: Some(ConfigValue::Number(10.0)),
                                    message: "Minimum DCA amount is $10".to_string(),
                                },
                            ],
                        },
                    ],
                },
            ],
            connections: vec![
                NodeConnectionTemplate {
                    id: "conn_1".to_string(),
                    source_node_id: "gas_estimation".to_string(),
                    source_output: "cheapest_chain".to_string(),
                    target_node_id: "dca_execution".to_string(),
                    target_input: "target_chain".to_string(),
                },
                NodeConnectionTemplate {
                    id: "conn_2".to_string(),
                    source_node_id: "gas_estimation".to_string(),
                    source_output: "estimated_cost_usd".to_string(),
                    target_node_id: "dca_execution".to_string(),
                    target_input: "max_gas_cost".to_string(),
                },
            ],
        }
    }
}

impl Default for ModernWorkflowTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}