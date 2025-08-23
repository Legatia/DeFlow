// DeFlow Workflow Templates - User-friendly strategy creation
// Pre-configured templates for common DeFi strategies

use super::automated_strategies::{StrategyConfig, StrategyType, YieldFarmingConfig, ArbitrageConfig, RebalancingConfig, LiquidityMiningConfig, DCAConfig, CompositeStrategyConfig};
use super::yield_farming::{ChainId, DeFiProtocol, UniswapVersion};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

/// Workflow template for easy strategy creation
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: WorkflowCategory,
    pub difficulty: DifficultyLevel,
    pub estimated_apy: f64,
    pub risk_score: u8,
    pub min_capital_usd: f64,
    pub template_config: TemplateConfig,
    pub user_inputs: Vec<UserInput>,
    pub workflow_steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum WorkflowCategory {
    YieldFarming,
    Arbitrage,
    Rebalancing,
    DCA,
    RiskManagement,
    Composite,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub strategy_type: StrategyType,
    pub default_chains: Vec<ChainId>,
    pub recommended_protocols: Vec<DeFiProtocol>,
    pub auto_rebalance: bool,
    pub stop_loss_enabled: bool,
    pub compound_rewards: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserInput {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_type: InputType,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum InputType {
    Number { min: Option<f64>, max: Option<f64> },
    Text { max_length: Option<u32> },
    Selection { options: Vec<String> },
    Boolean,
    ChainSelection,
    ProtocolSelection,
    TokenSelection,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub message: String,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum ValidationType {
    MinValue(f64),
    MaxValue(f64),
    Required,
    Pattern(String),
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_number: u32,
    pub name: String,
    pub description: String,
    pub action_type: ActionType,
    pub estimated_time_minutes: u32,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum ActionType {
    ValidateInputs,
    CheckBalances,
    EstimateGas,
    ExecuteTransaction,
    MonitorPosition,
    Rebalance,
    HarvestRewards,
    UpdateMetrics,
}

/// Workflow Template Manager
pub struct WorkflowTemplateManager {
    templates: HashMap<String, WorkflowTemplate>,
}

impl WorkflowTemplateManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        manager.initialize_default_templates();
        manager
    }

    /// Initialize pre-configured templates
    fn initialize_default_templates(&mut self) {
        // Template 1: Conservative Yield Farming
        self.add_template(self.create_conservative_yield_farming_template());
        
        // Template 2: Cross-Chain Arbitrage
        self.add_template(self.create_cross_chain_arbitrage_template());
        
        // Template 3: Balanced Portfolio Rebalancing
        self.add_template(self.create_portfolio_rebalancing_template());
        
        // Template 4: Dollar Cost Averaging
        self.add_template(self.create_dca_template());
        
        // Template 5: Risk Management
        self.add_template(self.create_risk_management_template());
        
        // Template 6: Advanced Multi-Strategy
        self.add_template(self.create_advanced_multi_strategy_template());
    }

    fn add_template(&mut self, template: WorkflowTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Get all available templates
    pub fn get_all_templates(&self) -> Vec<&WorkflowTemplate> {
        self.templates.values().collect()
    }

    /// Get template by ID
    pub fn get_template(&self, template_id: &str) -> Option<&WorkflowTemplate> {
        self.templates.get(template_id)
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &WorkflowCategory) -> Vec<&WorkflowTemplate> {
        self.templates
            .values()
            .filter(|template| std::mem::discriminant(&template.category) == std::mem::discriminant(category))
            .collect()
    }

    /// Generate strategy config from template and user inputs
    pub fn generate_strategy_config(&self, template_id: &str, user_inputs: HashMap<String, String>) -> Result<StrategyConfig, String> {
        let template = self.get_template(template_id)
            .ok_or_else(|| format!("Template {} not found", template_id))?;

        // Validate user inputs
        self.validate_inputs(&template.user_inputs, &user_inputs)?;

        // Generate strategy config based on template and inputs
        let strategy_config = StrategyConfig {
            name: format!("{} - {}", template.name, ic_cdk::api::time()),
            strategy_type: template.template_config.strategy_type.clone(),
            risk_level: self.calculate_risk_level(&user_inputs, template.risk_score),
            max_allocation_percentage: user_inputs.get("max_allocation")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(10.0),
            rebalance_threshold: user_inputs.get("rebalance_threshold")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(5.0),
            stop_loss_percentage: if template.template_config.stop_loss_enabled {
                user_inputs.get("stop_loss")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(10.0)
            } else { 0.0 },
            take_profit_percentage: user_inputs.get("take_profit")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(20.0),
            auto_compound: template.template_config.compound_rewards,
            gas_limit: 500_000,
        };

        Ok(strategy_config)
    }

    fn validate_inputs(&self, input_definitions: &[UserInput], user_inputs: &HashMap<String, String>) -> Result<(), String> {
        for input_def in input_definitions {
            if input_def.required {
                if !user_inputs.contains_key(&input_def.id) {
                    return Err(format!("Required input '{}' is missing", input_def.name));
                }
            }

            if let Some(value) = user_inputs.get(&input_def.id) {
                // Validate based on input type and rules
                match &input_def.input_type {
                    InputType::Number { min, max } => {
                        let num_value: f64 = value.parse()
                            .map_err(|_| format!("'{}' must be a valid number", input_def.name))?;
                        
                        if let Some(min_val) = min {
                            if num_value < *min_val {
                                return Err(format!("'{}' must be at least {}", input_def.name, min_val));
                            }
                        }
                        if let Some(max_val) = max {
                            if num_value > *max_val {
                                return Err(format!("'{}' cannot exceed {}", input_def.name, max_val));
                            }
                        }
                    },
                    InputType::Selection { options } => {
                        if !options.contains(value) {
                            return Err(format!("'{}' must be one of: {}", input_def.name, options.join(", ")));
                        }
                    },
                    _ => {} // Other validations can be added here
                }
            }
        }

        Ok(())
    }

    fn calculate_risk_level(&self, user_inputs: &HashMap<String, String>, base_risk: u8) -> u8 {
        let risk_tolerance = user_inputs.get("risk_tolerance")
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(5);

        // Adjust risk level based on user tolerance
        ((base_risk as f64 * risk_tolerance as f64 / 5.0) as u8).min(10)
    }

    // Template Creation Methods

    /// Conservative Yield Farming Template
    fn create_conservative_yield_farming_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: "conservative_yield_farming".to_string(),
            name: "Conservative Yield Farming".to_string(),
            description: "Low-risk yield farming on established protocols with stable assets".to_string(),
            category: WorkflowCategory::YieldFarming,
            difficulty: DifficultyLevel::Beginner,
            estimated_apy: 4.5,
            risk_score: 3,
            min_capital_usd: 100.0,
            template_config: TemplateConfig {
                strategy_type: StrategyType::YieldFarming(super::automated_strategies::YieldFarmingConfig {
                    target_protocols: vec![DeFiProtocol::Aave, DeFiProtocol::Compound],
                    preferred_assets: vec!["USDC".to_string(), "USDT".to_string(), "DAI".to_string()],
                    min_apy_threshold: 3.0,
                    max_risk_score: 4,
                    auto_compound: true,
                    harvest_threshold: 0.01,
                }),
                default_chains: vec![ChainId::Ethereum],
                recommended_protocols: vec![DeFiProtocol::Aave, DeFiProtocol::Compound],
                auto_rebalance: true,
                stop_loss_enabled: true,
                compound_rewards: true,
            },
            user_inputs: vec![
                UserInput {
                    id: "capital_amount".to_string(),
                    name: "Investment Amount (USD)".to_string(),
                    description: "How much capital do you want to allocate to this strategy?".to_string(),
                    input_type: InputType::Number { min: Some(100.0), max: Some(100_000.0) },
                    required: true,
                    default_value: Some("1000".to_string()),
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: ValidationType::MinValue(100.0),
                            message: "Minimum investment is $100".to_string(),
                        }
                    ],
                },
                UserInput {
                    id: "preferred_asset".to_string(),
                    name: "Preferred Stable Asset".to_string(),
                    description: "Which stable asset do you prefer to earn yield on?".to_string(),
                    input_type: InputType::Selection { options: vec!["USDC".to_string(), "USDT".to_string(), "DAI".to_string()] },
                    required: true,
                    default_value: Some("USDC".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "auto_compound".to_string(),
                    name: "Auto-Compound Rewards".to_string(),
                    description: "Automatically reinvest earned rewards?".to_string(),
                    input_type: InputType::Boolean,
                    required: false,
                    default_value: Some("true".to_string()),
                    validation_rules: vec![],
                },
            ],
            workflow_steps: vec![
                WorkflowStep {
                    step_number: 1,
                    name: "Validate Inputs".to_string(),
                    description: "Check user inputs and available capital".to_string(),
                    action_type: ActionType::ValidateInputs,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 2,
                    name: "Check Balances".to_string(),
                    description: "Verify sufficient balance in connected wallets".to_string(),
                    action_type: ActionType::CheckBalances,
                    estimated_time_minutes: 2,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 3,
                    name: "Execute Yield Strategy".to_string(),
                    description: "Deploy capital to selected yield farming protocols".to_string(),
                    action_type: ActionType::ExecuteTransaction,
                    estimated_time_minutes: 5,
                    requires_approval: true,
                },
                WorkflowStep {
                    step_number: 4,
                    name: "Monitor Performance".to_string(),
                    description: "Track APY, rewards, and position health".to_string(),
                    action_type: ActionType::MonitorPosition,
                    estimated_time_minutes: 0, // Continuous
                    requires_approval: false,
                },
            ],
        }
    }

    /// Cross-Chain Arbitrage Template
    fn create_cross_chain_arbitrage_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: "cross_chain_arbitrage".to_string(),
            name: "Cross-Chain Arbitrage".to_string(),
            description: "Automated arbitrage opportunities across different chains and DEXes".to_string(),
            category: WorkflowCategory::Arbitrage,
            difficulty: DifficultyLevel::Advanced,
            estimated_apy: 12.0,
            risk_score: 7,
            min_capital_usd: 1000.0,
            template_config: TemplateConfig {
                strategy_type: StrategyType::Arbitrage(super::automated_strategies::ArbitrageConfig {
                    target_chains: vec![ChainId::Ethereum, ChainId::Arbitrum, ChainId::Polygon],
                    min_profit_threshold: 1.5,
                    max_slippage: 0.5,
                    gas_price_limit: 100.0,
                    execution_timeout: 300,
                }),
                default_chains: vec![ChainId::Ethereum, ChainId::Arbitrum, ChainId::Polygon],
                recommended_protocols: vec![DeFiProtocol::Uniswap(UniswapVersion::V3), DeFiProtocol::Curve],
                auto_rebalance: false,
                stop_loss_enabled: true,
                compound_rewards: false,
            },
            user_inputs: vec![
                UserInput {
                    id: "capital_amount".to_string(),
                    name: "Trading Capital (USD)".to_string(),
                    description: "Capital allocated for arbitrage trading".to_string(),
                    input_type: InputType::Number { min: Some(1000.0), max: Some(1_000_000.0) },
                    required: true,
                    default_value: Some("5000".to_string()),
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: ValidationType::MinValue(1000.0),
                            message: "Minimum capital for arbitrage is $1,000".to_string(),
                        }
                    ],
                },
                UserInput {
                    id: "min_profit_threshold".to_string(),
                    name: "Minimum Profit Threshold (%)".to_string(),
                    description: "Only execute arbitrage if profit exceeds this threshold".to_string(),
                    input_type: InputType::Number { min: Some(0.5), max: Some(10.0) },
                    required: true,
                    default_value: Some("1.5".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "max_slippage".to_string(),
                    name: "Maximum Slippage (%)".to_string(),
                    description: "Maximum acceptable slippage for trades".to_string(),
                    input_type: InputType::Number { min: Some(0.1), max: Some(5.0) },
                    required: true,
                    default_value: Some("0.5".to_string()),
                    validation_rules: vec![],
                },
            ],
            workflow_steps: vec![
                WorkflowStep {
                    step_number: 1,
                    name: "Scan Opportunities".to_string(),
                    description: "Scan for profitable arbitrage opportunities across chains".to_string(),
                    action_type: ActionType::ValidateInputs,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 2,
                    name: "Validate Opportunity".to_string(),
                    description: "Confirm opportunity is still profitable and executable".to_string(),
                    action_type: ActionType::EstimateGas,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 3,
                    name: "Execute Arbitrage".to_string(),
                    description: "Execute buy/sell transactions across chains".to_string(),
                    action_type: ActionType::ExecuteTransaction,
                    estimated_time_minutes: 3,
                    requires_approval: true,
                },
                WorkflowStep {
                    step_number: 4,
                    name: "Track Performance".to_string(),
                    description: "Monitor arbitrage performance and profits".to_string(),
                    action_type: ActionType::UpdateMetrics,
                    estimated_time_minutes: 0, // Continuous
                    requires_approval: false,
                },
            ],
        }
    }

    /// Portfolio Rebalancing Template  
    fn create_portfolio_rebalancing_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: "portfolio_rebalancing".to_string(),
            name: "Automated Portfolio Rebalancing".to_string(),
            description: "Maintain optimal asset allocation across your DeFi portfolio".to_string(),
            category: WorkflowCategory::Rebalancing,
            difficulty: DifficultyLevel::Intermediate,
            estimated_apy: 6.0,
            risk_score: 5,
            min_capital_usd: 500.0,
            template_config: TemplateConfig {
                strategy_type: StrategyType::Rebalancing(super::automated_strategies::RebalancingConfig {
                    target_allocations: vec![
                        ("ETH".to_string(), 40.0),
                        ("BTC".to_string(), 30.0),
                        ("USDC".to_string(), 20.0),
                        ("LINK".to_string(), 10.0),
                    ],
                    rebalance_threshold: 5.0,
                    rebalance_frequency: super::automated_strategies::RebalanceFrequency::Weekly,
                    slippage_tolerance: 1.0,
                }),
                default_chains: vec![ChainId::Ethereum],
                recommended_protocols: vec![DeFiProtocol::Uniswap(UniswapVersion::V3)],
                auto_rebalance: true,
                stop_loss_enabled: false,
                compound_rewards: false,
            },
            user_inputs: vec![
                UserInput {
                    id: "total_portfolio_value".to_string(),
                    name: "Total Portfolio Value (USD)".to_string(),
                    description: "Total value of assets to be rebalanced".to_string(),
                    input_type: InputType::Number { min: Some(500.0), max: Some(10_000_000.0) },
                    required: true,
                    default_value: Some("10000".to_string()),
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: ValidationType::MinValue(500.0),
                            message: "Minimum portfolio value is $500".to_string(),
                        }
                    ],
                },
                UserInput {
                    id: "rebalance_threshold".to_string(),
                    name: "Rebalance Threshold (%)".to_string(),
                    description: "Rebalance when allocation drifts by this percentage".to_string(),
                    input_type: InputType::Number { min: Some(1.0), max: Some(20.0) },
                    required: true,
                    default_value: Some("5.0".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "rebalance_frequency".to_string(),
                    name: "Rebalancing Frequency".to_string(),
                    description: "How often should the portfolio be checked for rebalancing?".to_string(),
                    input_type: InputType::Selection { 
                        options: vec!["Daily".to_string(), "Weekly".to_string(), "Monthly".to_string()] 
                    },
                    required: true,
                    default_value: Some("Weekly".to_string()),
                    validation_rules: vec![],
                },
            ],
            workflow_steps: vec![
                WorkflowStep {
                    step_number: 1,
                    name: "Analyze Current Allocation".to_string(),
                    description: "Review current portfolio allocation vs targets".to_string(),
                    action_type: ActionType::ValidateInputs,
                    estimated_time_minutes: 2,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 2,
                    name: "Calculate Rebalancing Needs".to_string(),
                    description: "Determine which assets need buying/selling".to_string(),
                    action_type: ActionType::EstimateGas,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 3,
                    name: "Execute Rebalancing Trades".to_string(),
                    description: "Execute necessary trades to restore target allocation".to_string(),
                    action_type: ActionType::Rebalance,
                    estimated_time_minutes: 5,
                    requires_approval: true,
                },
                WorkflowStep {
                    step_number: 4,
                    name: "Monitor and Schedule Next Rebalance".to_string(),
                    description: "Track allocation drift and schedule next rebalancing check".to_string(),
                    action_type: ActionType::MonitorPosition,
                    estimated_time_minutes: 0, // Continuous
                    requires_approval: false,
                },
            ],
        }
    }

    /// Dollar Cost Averaging Template
    fn create_dca_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: "dollar_cost_averaging".to_string(),
            name: "Dollar Cost Averaging (DCA)".to_string(),
            description: "Systematic investment strategy to reduce timing risk".to_string(),
            category: WorkflowCategory::DCA,
            difficulty: DifficultyLevel::Beginner,
            estimated_apy: 8.0,
            risk_score: 4,
            min_capital_usd: 50.0,
            template_config: TemplateConfig {
                strategy_type: StrategyType::DCA(super::automated_strategies::DCAConfig {
                    target_asset: "ETH".to_string(),
                    purchase_amount: 100.0,
                    frequency: super::automated_strategies::DCAFrequency::Weekly,
                    price_deviation_threshold: Some(10.0),
                    stop_conditions: vec![
                        super::automated_strategies::DCAStopCondition::MaxInvestment(5000.0),
                        super::automated_strategies::DCAStopCondition::TargetPrice(3000.0),
                    ],
                }),
                default_chains: vec![ChainId::Ethereum],
                recommended_protocols: vec![DeFiProtocol::Uniswap(UniswapVersion::V3)],
                auto_rebalance: false,
                stop_loss_enabled: false,
                compound_rewards: false,
            },
            user_inputs: vec![
                UserInput {
                    id: "target_asset".to_string(),
                    name: "Target Asset".to_string(),
                    description: "Which asset do you want to dollar-cost average into?".to_string(),
                    input_type: InputType::Selection { 
                        options: vec!["ETH".to_string(), "BTC".to_string(), "LINK".to_string(), "UNI".to_string()] 
                    },
                    required: true,
                    default_value: Some("ETH".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "purchase_amount".to_string(),
                    name: "Purchase Amount per Interval (USD)".to_string(),
                    description: "How much to invest in each DCA purchase".to_string(),
                    input_type: InputType::Number { min: Some(10.0), max: Some(10_000.0) },
                    required: true,
                    default_value: Some("100".to_string()),
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: ValidationType::MinValue(10.0),
                            message: "Minimum purchase amount is $10".to_string(),
                        }
                    ],
                },
                UserInput {
                    id: "frequency".to_string(),
                    name: "Purchase Frequency".to_string(),
                    description: "How often should purchases be made?".to_string(),
                    input_type: InputType::Selection { 
                        options: vec!["Daily".to_string(), "Weekly".to_string(), "BiWeekly".to_string(), "Monthly".to_string()] 
                    },
                    required: true,
                    default_value: Some("Weekly".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "max_investment".to_string(),
                    name: "Maximum Total Investment (USD)".to_string(),
                    description: "Stop DCA when this total amount has been invested".to_string(),
                    input_type: InputType::Number { min: Some(100.0), max: Some(1_000_000.0) },
                    required: true,
                    default_value: Some("5000".to_string()),
                    validation_rules: vec![],
                },
            ],
            workflow_steps: vec![
                WorkflowStep {
                    step_number: 1,
                    name: "Check DCA Schedule".to_string(),
                    description: "Verify if it's time for the next DCA purchase".to_string(),
                    action_type: ActionType::ValidateInputs,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 2,
                    name: "Check Available Balance".to_string(),
                    description: "Ensure sufficient balance for the purchase".to_string(),
                    action_type: ActionType::CheckBalances,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 3,
                    name: "Execute DCA Purchase".to_string(),
                    description: "Buy the target asset with the scheduled amount".to_string(),
                    action_type: ActionType::ExecuteTransaction,
                    estimated_time_minutes: 3,
                    requires_approval: false, // Auto-execute DCA
                },
                WorkflowStep {
                    step_number: 4,
                    name: "Update DCA Metrics".to_string(),
                    description: "Track average cost basis and total invested".to_string(),
                    action_type: ActionType::UpdateMetrics,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
            ],
        }
    }

    /// Risk Management Template
    fn create_risk_management_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: "risk_management".to_string(),
            name: "Advanced Risk Management".to_string(),
            description: "Automated risk monitoring and portfolio protection".to_string(),
            category: WorkflowCategory::RiskManagement,
            difficulty: DifficultyLevel::Expert,
            estimated_apy: 0.0, // Risk management, not yield generation
            risk_score: 2,
            min_capital_usd: 1000.0,
            template_config: TemplateConfig {
                strategy_type: StrategyType::Composite(super::automated_strategies::CompositeConfig {
                    sub_strategies: vec![], // Risk management doesn't have sub-strategies
                    coordination_rules: vec![],
                    rebalancing_frequency: super::automated_strategies::RebalanceFrequency::Daily,
                }),
                default_chains: vec![ChainId::Ethereum, ChainId::Arbitrum, ChainId::Polygon],
                recommended_protocols: vec![],
                auto_rebalance: true,
                stop_loss_enabled: true,
                compound_rewards: false,
            },
            user_inputs: vec![
                UserInput {
                    id: "portfolio_value".to_string(),
                    name: "Total Portfolio Value (USD)".to_string(),
                    description: "Total value of portfolio to monitor".to_string(),
                    input_type: InputType::Number { min: Some(1000.0), max: Some(100_000_000.0) },
                    required: true,
                    default_value: Some("50000".to_string()),
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: ValidationType::MinValue(1000.0),
                            message: "Minimum portfolio value is $1,000".to_string(),
                        }
                    ],
                },
                UserInput {
                    id: "max_drawdown".to_string(),
                    name: "Maximum Acceptable Drawdown (%)".to_string(),
                    description: "Trigger emergency actions if drawdown exceeds this".to_string(),
                    input_type: InputType::Number { min: Some(5.0), max: Some(50.0) },
                    required: true,
                    default_value: Some("15.0".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "var_threshold".to_string(),
                    name: "Value at Risk Threshold (%)".to_string(),
                    description: "Alert when 1-day VaR exceeds this threshold".to_string(),
                    input_type: InputType::Number { min: Some(1.0), max: Some(25.0) },
                    required: true,
                    default_value: Some("5.0".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "correlation_limit".to_string(),
                    name: "Maximum Asset Correlation".to_string(),
                    description: "Alert when asset correlations exceed this limit".to_string(),
                    input_type: InputType::Number { min: Some(0.5), max: Some(0.95) },
                    required: true,
                    default_value: Some("0.8".to_string()),
                    validation_rules: vec![],
                },
            ],
            workflow_steps: vec![
                WorkflowStep {
                    step_number: 1,
                    name: "Calculate Risk Metrics".to_string(),
                    description: "Compute VaR, drawdown, correlation, and volatility".to_string(),
                    action_type: ActionType::UpdateMetrics,
                    estimated_time_minutes: 2,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 2,
                    name: "Check Risk Thresholds".to_string(),
                    description: "Compare metrics against configured thresholds".to_string(),
                    action_type: ActionType::ValidateInputs,
                    estimated_time_minutes: 1,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 3,
                    name: "Execute Risk Actions".to_string(),
                    description: "Take protective actions if thresholds are breached".to_string(),
                    action_type: ActionType::ExecuteTransaction,
                    estimated_time_minutes: 5,
                    requires_approval: true,
                },
                WorkflowStep {
                    step_number: 4,
                    name: "Monitor and Alert".to_string(),
                    description: "Continuous monitoring and alert notifications".to_string(),
                    action_type: ActionType::MonitorPosition,
                    estimated_time_minutes: 0, // Continuous
                    requires_approval: false,
                },
            ],
        }
    }

    /// Advanced Multi-Strategy Template
    fn create_advanced_multi_strategy_template(&self) -> WorkflowTemplate {
        WorkflowTemplate {
            id: "advanced_multi_strategy".to_string(),
            name: "Advanced Multi-Strategy Portfolio".to_string(),
            description: "Sophisticated combination of yield farming, arbitrage, and rebalancing".to_string(),
            category: WorkflowCategory::Composite,
            difficulty: DifficultyLevel::Expert,
            estimated_apy: 15.0,
            risk_score: 8,
            min_capital_usd: 10000.0,
            template_config: TemplateConfig {
                strategy_type: StrategyType::Composite(super::automated_strategies::CompositeConfig {
                    sub_strategies: vec![
                        "conservative_yield_farming".to_string(),
                        "cross_chain_arbitrage".to_string(),
                        "portfolio_rebalancing".to_string(),
                    ]);
                    coordination_rules: vec![
                        "max_risk_allocation_50_percent".to_string(),
                        "yield_farming_priority_high".to_string(),
                        "arbitrage_only_high_profit".to_string(),
                    ],
                    rebalancing_frequency: super::automated_strategies::RebalanceFrequency::Daily,
                }),
                default_chains: vec![ChainId::Ethereum, ChainId::Arbitrum, ChainId::Polygon]);
                recommended_protocols: vec![
                    DeFiProtocol::Aave, 
                    DeFiProtocol::Uniswap(UniswapVersion::V3), 
                    DeFiProtocol::Curve
                ],
                auto_rebalance: true,
                stop_loss_enabled: true,
                compound_rewards: true,
            },
            user_inputs: vec![
                UserInput {
                    id: "total_capital".to_string(),
                    name: "Total Investment Capital (USD)".to_string(),
                    description: "Total capital to be managed across all strategies".to_string(),
                    input_type: InputType::Number { min: Some(10000.0), max: Some(10_000_000.0) },
                    required: true,
                    default_value: Some("100000".to_string()),
                    validation_rules: vec![
                        ValidationRule {
                            rule_type: ValidationType::MinValue(10000.0),
                            message: "Minimum capital for multi-strategy is $10,000".to_string(),
                        }
                    ],
                },
                UserInput {
                    id: "risk_tolerance".to_string(),
                    name: "Risk Tolerance (1-10)".to_string(),
                    description: "Higher values enable more aggressive strategies".to_string(),
                    input_type: InputType::Number { min: Some(1.0), max: Some(10.0) },
                    required: true,
                    default_value: Some("7.0".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "yield_allocation".to_string(),
                    name: "Yield Farming Allocation (%)".to_string(),
                    description: "Percentage allocated to yield farming strategies".to_string(),
                    input_type: InputType::Number { min: Some(20.0), max: Some(80.0) },
                    required: true,
                    default_value: Some("50.0".to_string()),
                    validation_rules: vec![],
                },
                UserInput {
                    id: "arbitrage_allocation".to_string(),
                    name: "Arbitrage Allocation (%)".to_string(),
                    description: "Percentage allocated to arbitrage opportunities".to_string(),
                    input_type: InputType::Number { min: Some(10.0), max: Some(50.0) },
                    required: true,
                    default_value: Some("30.0".to_string()),
                    validation_rules: vec![],
                },
            ],
            workflow_steps: vec![
                WorkflowStep {
                    step_number: 1,
                    name: "Strategy Coordination".to_string(),
                    description: "Coordinate allocation across multiple strategies".to_string(),
                    action_type: ActionType::ValidateInputs,
                    estimated_time_minutes: 3,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 2,
                    name: "Multi-Chain Execution".to_string(),
                    description: "Execute strategies across multiple chains simultaneously".to_string(),
                    action_type: ActionType::ExecuteTransaction,
                    estimated_time_minutes: 10,
                    requires_approval: true,
                },
                WorkflowStep {
                    step_number: 3,
                    name: "Dynamic Rebalancing".to_string(),
                    description: "Continuously rebalance between strategies based on performance".to_string(),
                    action_type: ActionType::Rebalance,
                    estimated_time_minutes: 5,
                    requires_approval: false,
                },
                WorkflowStep {
                    step_number: 4,
                    name: "Performance Optimization".to_string(),
                    description: "Monitor and optimize strategy performance".to_string(),
                    action_type: ActionType::UpdateMetrics,
                    estimated_time_minutes: 0, // Continuous
                    requires_approval: false,
                },
            ],
        }
    }
}

impl Default for WorkflowTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}