// DeFlow Simple Workflow Templates - Basic user-friendly strategy creation
// Simplified version that compiles cleanly

use super::automated_strategies::{StrategyConfig, StrategyType, YieldFarmingConfig, ArbitrageConfig, RebalancingConfig, DCAConfig};
use super::yield_farming::{ChainId, DeFiProtocol, UniswapVersion};
use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

/// Simple workflow template for easy strategy creation
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct SimpleWorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub difficulty: String,
    pub estimated_apy: f64,
    pub risk_score: u8,
    pub min_capital_usd: f64,
}

/// Simple Template Manager
pub struct SimpleWorkflowTemplateManager {
    templates: HashMap<String, SimpleWorkflowTemplate>,
}

impl SimpleWorkflowTemplateManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };
        manager.initialize_basic_templates();
        manager
    }

    fn initialize_basic_templates(&mut self) {
        // Basic Yield Farming Template
        self.templates.insert("conservative_yield".to_string(), SimpleWorkflowTemplate {
            id: "conservative_yield".to_string(),
            name: "Conservative Yield Farming".to_string(),
            description: "Low-risk yield farming on established protocols".to_string(),
            category: "YieldFarming".to_string(),
            difficulty: "Beginner".to_string(),
            estimated_apy: 4.5,
            risk_score: 3,
            min_capital_usd: 100.0,
        });

        // Basic Arbitrage Template
        self.templates.insert("basic_arbitrage".to_string(), SimpleWorkflowTemplate {
            id: "basic_arbitrage".to_string(),
            name: "Cross-Chain Arbitrage".to_string(),
            description: "Automated arbitrage opportunities across chains".to_string(),
            category: "Arbitrage".to_string(),
            difficulty: "Advanced".to_string(),
            estimated_apy: 12.0,
            risk_score: 7,
            min_capital_usd: 1000.0,
        });

        // Basic Portfolio Rebalancing Template
        self.templates.insert("portfolio_rebalancing".to_string(), SimpleWorkflowTemplate {
            id: "portfolio_rebalancing".to_string(),
            name: "Portfolio Rebalancing".to_string(),
            description: "Maintain optimal asset allocation".to_string(),
            category: "Rebalancing".to_string(),
            difficulty: "Intermediate".to_string(),
            estimated_apy: 6.0,
            risk_score: 5,
            min_capital_usd: 500.0,
        });

        // Basic DCA Template
        self.templates.insert("dollar_cost_averaging".to_string(), SimpleWorkflowTemplate {
            id: "dollar_cost_averaging".to_string(),
            name: "Dollar Cost Averaging".to_string(),
            description: "Systematic investment strategy".to_string(),
            category: "DCA".to_string(),
            difficulty: "Beginner".to_string(),
            estimated_apy: 8.0,
            risk_score: 4,
            min_capital_usd: 50.0,
        });
    }

    pub fn get_all_templates(&self) -> Vec<&SimpleWorkflowTemplate> {
        self.templates.values().collect()
    }

    pub fn get_template(&self, template_id: &str) -> Option<&SimpleWorkflowTemplate> {
        self.templates.get(template_id)
    }

    pub fn get_templates_by_category(&self, category: &str) -> Vec<&SimpleWorkflowTemplate> {
        self.templates
            .values()
            .filter(|template| template.category == category)
            .collect()
    }

    /// Generate a basic strategy config from template
    pub fn generate_strategy_config(&self, template_id: &str, capital_amount: f64) -> Result<StrategyConfig, String> {
        let template = self.get_template(template_id)
            .ok_or_else(|| format!("Template {} not found", template_id))?;

        let strategy_type = match template.id.as_str() {
            "conservative_yield" => StrategyType::YieldFarming(YieldFarmingConfig {
                min_apy_threshold: 3.0,
                preferred_tokens: vec!["USDC".to_string(), "USDT".to_string()],
                max_impermanent_loss_percentage: 2.0,
                auto_harvest_rewards: true,
            }),
            "basic_arbitrage" => StrategyType::Arbitrage(ArbitrageConfig {
                min_profit_percentage: 1.5,
                max_execution_time_seconds: 300,
                max_slippage_percentage: 0.5,
                preferred_dex_pairs: vec![("Uniswap".to_string(), "Curve".to_string())],
            }),
            "portfolio_rebalancing" => {
                let mut target_allocation = HashMap::new();
                target_allocation.insert("ETH".to_string(), 40.0);
                target_allocation.insert("BTC".to_string(), 30.0);
                target_allocation.insert("USDC".to_string(), 20.0);
                target_allocation.insert("LINK".to_string(), 10.0);
                
                StrategyType::Rebalancing(RebalancingConfig {
                    target_allocation,
                    rebalance_threshold_percentage: 5.0,
                    rebalance_frequency_hours: 168, // Weekly
                })
            },
            "dollar_cost_averaging" => StrategyType::DCA(DCAConfig {
                target_token: "ETH".to_string(),
                amount_per_execution: 100.0,
                price_threshold_percentage: Some(10.0),
            }),
            _ => return Err("Unknown template".to_string()),
        };

        Ok(StrategyConfig {
            name: format!("{} - {}", template.name, ic_cdk::api::time()),
            description: template.description.clone(),
            strategy_type,
            target_chains: vec![ChainId::Ethereum],
            target_protocols: vec![DeFiProtocol::Aave, DeFiProtocol::Uniswap(UniswapVersion::V3)],
            required_wallet_addresses: vec![ChainId::Ethereum], // Required wallet for Ethereum
            risk_level: template.risk_score,
            max_allocation_usd: capital_amount,
            min_return_threshold: 1.0,
            execution_interval_minutes: 60,
            gas_limit_usd: 100.0,
            auto_compound: true,
            stop_loss_percentage: Some(10.0),
            take_profit_percentage: Some(20.0),
        })
    }
}

impl Default for SimpleWorkflowTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}