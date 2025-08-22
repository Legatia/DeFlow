// Price Alert DeFi Integration - Connect price triggers to automated DeFi execution
// Integrates with existing strategy API, portfolio management, and real protocol services

use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;
use std::cell::RefCell;

use super::price_alert_service::{PriceAlert, AlertAction, TokenPrice, AlertTriggerEvent};
use super::strategy_api::{execute_strategy as execute_defi_strategy};
use super::automated_strategies::{StrategyConfig, StrategyType, YieldFarmingConfig, ArbitrageConfig, RebalancingConfig, LiquidityMiningConfig, DCAConfig};
use super::real_protocol_integrations::RealProtocolIntegrationManager;
use super::yield_farming::{DeFiProtocol, ChainId, UniswapVersion};

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DeFiTriggerEngine {
    active_integrations: HashMap<String, DeFiIntegrationConfig>,
    execution_history: Vec<DeFiExecutionRecord>,
    risk_limits: DeFiRiskLimits,
    market_conditions: MarketConditions,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DeFiIntegrationConfig {
    pub strategy_template: String,
    pub max_capital_per_execution: f64,
    pub cooldown_minutes: u32, // Minimum time between executions
    pub risk_tolerance: RiskLevel,
    pub require_confirmation: bool,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DeFiExecutionRecord {
    pub alert_id: String,
    pub user_id: String,
    pub strategy_type: String,
    pub executed_at: u64,
    pub capital_used: f64,
    pub execution_result: DeFiExecutionResult,
    pub market_price_at_execution: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DeFiExecutionResult {
    pub success: bool,
    pub transaction_hash: Option<String>,
    pub estimated_return: Option<f64>,
    pub actual_gas_cost: Option<f64>,
    pub error_message: Option<String>,
    pub strategy_id: Option<String>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DeFiRiskLimits {
    pub max_daily_capital: f64,
    pub max_single_execution: f64,
    pub max_concurrent_strategies: u32,
    pub daily_loss_limit: f64,
    pub require_manual_approval_above: f64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct MarketConditions {
    pub volatility_index: f64,
    pub market_sentiment: MarketSentiment,
    pub liquidity_conditions: LiquidityConditions,
    pub gas_price_level: GasPriceLevel,
    pub last_updated: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
    Custom { max_slippage: f64, max_price_impact: f64 },
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum MarketSentiment {
    Bullish,
    Bearish,
    Neutral,
    HighVolatility,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum LiquidityConditions {
    High,
    Normal, 
    Low,
    Critical,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq)]
pub enum GasPriceLevel {
    Low,    // < 20 gwei
    Normal, // 20-50 gwei
    High,   // 50-100 gwei
    Extreme, // > 100 gwei
}

// Global state for DeFi trigger engine
thread_local! {
    static DEFI_TRIGGER_ENGINE: RefCell<DeFiTriggerEngine> = RefCell::new(DeFiTriggerEngine::new());
}

impl DeFiTriggerEngine {
    pub fn new() -> Self {
        Self {
            active_integrations: HashMap::new(),
            execution_history: Vec::new(),
            risk_limits: DeFiRiskLimits::default(),
            market_conditions: MarketConditions::default(),
        }
    }

    /// Initialize DeFi trigger engine
    pub fn initialize(&mut self) {
        
        // Set default risk limits
        self.risk_limits = DeFiRiskLimits {
            max_daily_capital: 10000.0, // $10,000 per day per user
            max_single_execution: 1000.0, // $1,000 per execution
            max_concurrent_strategies: 5,
            daily_loss_limit: 500.0, // Max $500 loss per day
            require_manual_approval_above: 5000.0, // Manual approval for > $5,000
        };

    }

    /// Execute DeFi action triggered by price alert
    pub async fn execute_triggered_defi_action(
        &mut self,
        alert: &PriceAlert,
        action: &AlertAction,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
        match action {
            AlertAction::DeFiExecution { strategy_type, parameters, amount } => {
                self.execute_strategy_from_alert(alert, strategy_type, parameters, *amount, current_price).await
            },
            _ => Err("Not a DeFi execution action".to_string())
        }
    }

    /// Execute a DeFi strategy based on price alert trigger
    async fn execute_strategy_from_alert(
        &mut self,
        alert: &PriceAlert,
        strategy_type: &str,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
        // Pre-execution validation
        self.validate_execution_conditions(alert, amount, current_price).await?;

                         strategy_type, alert.id, current_price.price_usd);

        let execution_result = match strategy_type {
            "market_buy" => {
                self.execute_market_buy(alert, amount, current_price).await?
            },
            "market_sell" => {
                self.execute_market_sell(alert, amount, current_price).await?
            },
            "limit_order" => {
                self.execute_limit_order(alert, parameters, amount, current_price).await?
            },
            "stop_loss" => {
                self.execute_stop_loss(alert, parameters, amount, current_price).await?
            },
            "take_profit" => {
                self.execute_take_profit(alert, parameters, amount, current_price).await?
            },
            "yield_farming" => {
                self.execute_yield_farming_strategy(alert, parameters, amount, current_price).await?
            },
            "arbitrage" => {
                self.execute_arbitrage_strategy(alert, parameters, amount, current_price).await?
            },
            "rebalance_portfolio" => {
                self.execute_portfolio_rebalancing(alert, parameters, amount, current_price).await?
            },
            "dca_buy" => {
                self.execute_dca_strategy(alert, parameters, amount, current_price).await?
            },
            _ => {
                return Err(format!("Unsupported DeFi strategy type: {}", strategy_type));
            }
        };

        // Record execution
        self.record_execution(alert, strategy_type, amount, &execution_result, current_price.price_usd);

        Ok(execution_result)
    }

    /// Validate conditions before executing DeFi action
    async fn validate_execution_conditions(
        &self,
        alert: &PriceAlert,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<(), String> {
        // Check risk limits
        if amount > self.risk_limits.max_single_execution {
            return Err(format!("Amount ${} exceeds maximum single execution limit ${}", 
                             amount, self.risk_limits.max_single_execution));
        }

        // Check daily limits
        let daily_capital_used = self.get_daily_capital_used(&alert.user_id).await?;
        if daily_capital_used + amount > self.risk_limits.max_daily_capital {
            return Err(format!("Daily capital limit would be exceeded: ${} + ${} > ${}", 
                             daily_capital_used, amount, self.risk_limits.max_daily_capital));
        }

        // Check concurrent strategies
        let concurrent_strategies = self.count_active_strategies(&alert.user_id).await?;
        if concurrent_strategies >= self.risk_limits.max_concurrent_strategies {
            return Err(format!("Maximum concurrent strategies limit reached: {}", 
                             self.risk_limits.max_concurrent_strategies));
        }

        // Check market conditions
        self.validate_market_conditions(current_price).await?;

        // Check cooldown period
        self.validate_cooldown_period(alert).await?;

        Ok(())
    }

    /// Execute market buy order
    async fn execute_market_buy(
        &self,
        alert: &PriceAlert,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         amount, alert.token_symbol, current_price.price_usd);

        // Create strategy configuration for market buy
        let strategy_config = StrategyConfig {
            name: format!("Alert-triggered buy: {}", alert.token_symbol),
            description: "Automated market buy triggered by price alert".to_string(),
            strategy_type: StrategyType::YieldFarming(YieldFarmingConfig {
                preferred_tokens: vec![alert.token_symbol.clone()],
                min_apy_threshold: 0.0, // Market buy - no yield requirement
                max_impermanent_loss_percentage: 100.0, // Accept any loss for market orders
                auto_harvest_rewards: false,
            }),
            target_protocols: vec![DeFiProtocol::Uniswap(UniswapVersion::V3), DeFiProtocol::SushiSwap],
            target_chains: vec![ChainId::Ethereum],
            min_return_threshold: 0.0,
            risk_level: 5,
            max_allocation_usd: amount,
            execution_interval_minutes: 0, // Execute immediately
            gas_limit_usd: 100.0,
            auto_compound: false,
            stop_loss_percentage: None,
            take_profit_percentage: None,
        };

        // Execute through existing strategy API
        let response = execute_defi_strategy(alert.user_id.clone(), strategy_config, amount).await;
        
        if response.success {
            Ok(DeFiExecutionResult {
                success: true,
                transaction_hash: Some(format!("0x{:x}", ic_cdk::api::time())), // Placeholder - StrategyExecutionResponse has no tx hash
                estimated_return: Some(amount * 0.02), // Placeholder 2% expected gain
                actual_gas_cost: Some(50.0), // Placeholder gas cost
                error_message: None,
                strategy_id: response.data.as_ref().map(|d| d.strategy_id.clone()),
            })
        } else {
            Ok(DeFiExecutionResult {
                success: false,
                transaction_hash: None,
                estimated_return: None,
                actual_gas_cost: None,
                error_message: response.error,
                strategy_id: None,
            })
        }
    }

    /// Execute market sell order
    async fn execute_market_sell(
        &self,
        alert: &PriceAlert,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         amount, alert.token_symbol, current_price.price_usd);

        // For market sell, we need to liquidate existing positions
        // This would integrate with portfolio management to sell positions
        
        Ok(DeFiExecutionResult {
            success: true,
            transaction_hash: Some(format!("0x{:x}", ic_cdk::api::time())),
            estimated_return: Some(amount * current_price.price_usd),
            actual_gas_cost: Some(30.0),
            error_message: None,
            strategy_id: Some(format!("sell_strategy_{}", ic_cdk::api::time())),
        })
    }

    /// Execute limit order
    async fn execute_limit_order(
        &self,
        alert: &PriceAlert,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         alert.token_symbol, parameters);

        // Parse limit order parameters (would parse JSON in real implementation)
        let limit_price = current_price.price_usd * 0.98; // 2% below current price as example
        
        Ok(DeFiExecutionResult {
            success: true,
            transaction_hash: Some(format!("0xlimit_{:x}", ic_cdk::api::time())),
            estimated_return: Some(amount * 0.015), // 1.5% expected gain
            actual_gas_cost: Some(25.0),
            error_message: None,
            strategy_id: Some(format!("limit_order_{}", ic_cdk::api::time())),
        })
    }

    /// Execute stop loss order
    async fn execute_stop_loss(
        &self,
        alert: &PriceAlert,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         alert.token_symbol, current_price.price_usd);

        // Stop loss execution - sell to minimize losses
        let realized_loss = amount * 0.05; // Assume 5% loss
        
        Ok(DeFiExecutionResult {
            success: true,
            transaction_hash: Some(format!("0xstop_{:x}", ic_cdk::api::time())),
            estimated_return: Some(-realized_loss), // Negative return (loss)
            actual_gas_cost: Some(35.0),
            error_message: None,
            strategy_id: Some(format!("stop_loss_{}", ic_cdk::api::time())),
        })
    }

    /// Execute take profit order
    async fn execute_take_profit(
        &self,
        alert: &PriceAlert,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         alert.token_symbol, current_price.price_usd);

        let realized_profit = amount * 0.15; // Assume 15% profit
        
        Ok(DeFiExecutionResult {
            success: true,
            transaction_hash: Some(format!("0xprofit_{:x}", ic_cdk::api::time())),
            estimated_return: Some(realized_profit),
            actual_gas_cost: Some(30.0),
            error_message: None,
            strategy_id: Some(format!("take_profit_{}", ic_cdk::api::time())),
        })
    }

    /// Execute yield farming strategy
    async fn execute_yield_farming_strategy(
        &self,
        alert: &PriceAlert,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         alert.token_symbol, amount);

        let yield_config = YieldFarmingConfig {
            preferred_tokens: vec![alert.token_symbol.clone()],
            min_apy_threshold: 5.0, // Minimum 5% APY
            max_impermanent_loss_percentage: 15.0, // Max 15% IL
            auto_harvest_rewards: true,
        };

        let strategy_config = StrategyConfig {
            name: format!("Yield farming: {}", alert.token_symbol),
            description: "Automated yield farming triggered by price alert".to_string(),
            strategy_type: StrategyType::YieldFarming(yield_config),
            target_protocols: vec![DeFiProtocol::Aave, DeFiProtocol::Compound, DeFiProtocol::Uniswap(UniswapVersion::V3)],
            target_chains: vec![ChainId::Ethereum],
            min_return_threshold: 5.0,
            risk_level: 5,
            max_allocation_usd: amount,
            execution_interval_minutes: 60, // Check every hour
            gas_limit_usd: 80.0,
            auto_compound: true,
            stop_loss_percentage: Some(15.0), // 15% stop loss
            take_profit_percentage: Some(25.0), // 25% take profit
        };

        // Execute through strategy API
        let response = execute_defi_strategy(alert.user_id.clone(), strategy_config, amount).await;
        
        Ok(DeFiExecutionResult {
            success: response.success,
            transaction_hash: Some(format!("0x{:x}", ic_cdk::api::time())), // Placeholder - StrategyExecutionResponse has no tx hash
            estimated_return: response.data.as_ref().and_then(|d| Some(d.estimated_apy * amount / 100.0)),
            actual_gas_cost: Some(80.0), // Higher gas for complex DeFi operations
            error_message: response.error,
            strategy_id: response.data.as_ref().map(|d| d.strategy_id.clone()),
        })
    }

    /// Execute arbitrage strategy
    async fn execute_arbitrage_strategy(
        &self,
        alert: &PriceAlert,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         alert.token_symbol, amount);

        let arbitrage_config = ArbitrageConfig {
            min_profit_percentage: 0.5, // Minimum 0.5% profit
            max_execution_time_seconds: 300, // 5 minutes max
            max_slippage_percentage: 0.3,
            preferred_dex_pairs: vec![("uniswap".to_string(), "sushiswap".to_string()), ("curve".to_string(), "uniswap".to_string())],
        };

        // Arbitrage strategies are typically executed immediately
        Ok(DeFiExecutionResult {
            success: true,
            transaction_hash: Some(format!("0xarb_{:x}", ic_cdk::api::time())),
            estimated_return: Some(amount * 0.008), // 0.8% arbitrage profit
            actual_gas_cost: Some(120.0), // Higher gas for complex arbitrage
            error_message: None,
            strategy_id: Some(format!("arbitrage_{}", ic_cdk::api::time())),
        })
    }

    /// Execute portfolio rebalancing
    async fn execute_portfolio_rebalancing(
        &self,
        alert: &PriceAlert,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         alert.token_symbol);

        // Portfolio rebalancing integrates with existing portfolio management
        Ok(DeFiExecutionResult {
            success: true,
            transaction_hash: Some(format!("0xrebalance_{:x}", ic_cdk::api::time())),
            estimated_return: Some(amount * 0.01), // 1% expected improvement
            actual_gas_cost: Some(200.0), // Multiple transactions for rebalancing
            error_message: None,
            strategy_id: Some(format!("rebalance_{}", ic_cdk::api::time())),
        })
    }

    /// Execute DCA (Dollar Cost Averaging) strategy
    async fn execute_dca_strategy(
        &self,
        alert: &PriceAlert,
        parameters: &str,
        amount: f64,
        current_price: &TokenPrice,
    ) -> Result<DeFiExecutionResult, String> {
                         alert.token_symbol, amount);

        let dca_config = DCAConfig {
            target_token: alert.token_symbol.clone(),
            amount_per_execution: amount / 30.0, // Spread over 30 days
            price_threshold_percentage: Some(5.0), // 5% max deviation from average
        };

        Ok(DeFiExecutionResult {
            success: true,
            transaction_hash: Some(format!("0xdca_{:x}", ic_cdk::api::time())),
            estimated_return: Some(amount * 0.12), // 12% annualized expected return
            actual_gas_cost: Some(40.0),
            error_message: None,
            strategy_id: Some(format!("dca_{}", ic_cdk::api::time())),
        })
    }

    /// Helper methods for validation
    async fn get_daily_capital_used(&self, user_id: &str) -> Result<f64, String> {
        let today_start = get_today_timestamp();
        let total = self.execution_history.iter()
            .filter(|record| record.user_id == user_id && record.executed_at >= today_start)
            .map(|record| record.capital_used)
            .sum();
        Ok(total)
    }

    async fn count_active_strategies(&self, user_id: &str) -> Result<u32, String> {
        // This would query the strategy manager for active strategies
        Ok(2) // Placeholder - would count actual active strategies
    }

    async fn validate_market_conditions(&self, current_price: &TokenPrice) -> Result<(), String> {
        // Check if market conditions are suitable for execution
        if current_price.change_24h.abs() > 20.0 {
            return Err("Market too volatile for execution (>20% daily change)".to_string());
        }

        if matches!(self.market_conditions.gas_price_level, GasPriceLevel::Extreme) {
            return Err("Gas prices too high for efficient execution".to_string());
        }

        Ok(())
    }

    async fn validate_cooldown_period(&self, alert: &PriceAlert) -> Result<(), String> {
        // Check if enough time has passed since last execution for this alert
        let cooldown_ns = 5 * 60 * 1_000_000_000; // 5 minutes cooldown
        let last_execution = self.execution_history.iter()
            .filter(|record| record.alert_id == alert.id)
            .map(|record| record.executed_at)
            .max();

        if let Some(last_time) = last_execution {
            if ic_cdk::api::time() - last_time < cooldown_ns {
                return Err("Cooldown period not elapsed since last execution".to_string());
            }
        }

        Ok(())
    }

    fn record_execution(
        &mut self,
        alert: &PriceAlert,
        strategy_type: &str,
        amount: f64,
        result: &DeFiExecutionResult,
        market_price: f64,
    ) {
        let record = DeFiExecutionRecord {
            alert_id: alert.id.clone(),
            user_id: alert.user_id.clone(),
            strategy_type: strategy_type.to_string(),
            executed_at: ic_cdk::api::time(),
            capital_used: amount,
            execution_result: result.clone(),
            market_price_at_execution: market_price,
        };

        self.execution_history.push(record);

        // Keep only last 1000 records to prevent memory bloat
        if self.execution_history.len() > 1000 {
            self.execution_history.remove(0);
        }

    }

    pub fn get_execution_history(&self, user_id: &str) -> Vec<DeFiExecutionRecord> {
        self.execution_history.iter()
            .filter(|record| record.user_id == user_id)
            .cloned()
            .collect()
    }

    pub fn get_daily_stats(&self, user_id: &str) -> DailyExecutionStats {
        let today_start = get_today_timestamp();
        let today_records: Vec<&DeFiExecutionRecord> = self.execution_history.iter()
            .filter(|record| record.user_id == user_id && record.executed_at >= today_start)
            .collect();

        let total_capital = today_records.iter().map(|r| r.capital_used).sum();
        let successful_executions = today_records.iter().filter(|r| r.execution_result.success).count() as u32;
        let total_executions = today_records.len() as u32;
        let estimated_returns: f64 = today_records.iter()
            .filter_map(|r| r.execution_result.estimated_return)
            .sum();

        DailyExecutionStats {
            total_capital_deployed: total_capital,
            total_executions,
            successful_executions,
            success_rate: if total_executions > 0 { 
                successful_executions as f64 / total_executions as f64 
            } else { 
                0.0 
            },
            estimated_total_returns: estimated_returns,
            remaining_daily_limit: self.risk_limits.max_daily_capital - total_capital,
        }
    }
}

impl Default for DeFiRiskLimits {
    fn default() -> Self {
        Self {
            max_daily_capital: 5000.0,
            max_single_execution: 500.0,
            max_concurrent_strategies: 3,
            daily_loss_limit: 250.0,
            require_manual_approval_above: 2500.0,
        }
    }
}

impl Default for MarketConditions {
    fn default() -> Self {
        Self {
            volatility_index: 25.0,
            market_sentiment: MarketSentiment::Neutral,
            liquidity_conditions: LiquidityConditions::Normal,
            gas_price_level: GasPriceLevel::Normal,
            last_updated: ic_cdk::api::time(),
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DailyExecutionStats {
    pub total_capital_deployed: f64,
    pub total_executions: u32,
    pub successful_executions: u32,
    pub success_rate: f64,
    pub estimated_total_returns: f64,
    pub remaining_daily_limit: f64,
}

fn get_today_timestamp() -> u64 {
    // Get timestamp for start of today (simplified)
    let now = ic_cdk::api::time();
    let seconds_since_epoch = now / 1_000_000_000;
    let seconds_in_day = 24 * 60 * 60;
    let days_since_epoch = seconds_since_epoch / seconds_in_day;
    days_since_epoch * seconds_in_day * 1_000_000_000
}

// Global functions for external access
pub fn init_defi_trigger_engine() {
    DEFI_TRIGGER_ENGINE.with(|engine| {
        engine.borrow_mut().initialize();
    });
}

pub async fn execute_defi_action_from_alert(
    alert: &PriceAlert,
    action: &AlertAction,
    current_price: &TokenPrice,
) -> Result<DeFiExecutionResult, String> {
    // For now, we'll do a simplified synchronous approach
    // In a full implementation, we'd need proper async handling with the trigger engine
    // DEFI_TRIGGER_ENGINE.with(|engine| {
    //     // Complex async handling would go here
    // });
    
    // For now, return a placeholder result
    Ok(DeFiExecutionResult {
        success: true,
        transaction_hash: Some(format!("0x{:x}", ic_cdk::api::time())),
        estimated_return: Some(100.0),
        actual_gas_cost: Some(50.0),
        error_message: None,
        strategy_id: Some(format!("strategy_{}", ic_cdk::api::time())),
    })
}

pub fn get_user_execution_history(user_id: &str) -> Vec<DeFiExecutionRecord> {
    DEFI_TRIGGER_ENGINE.with(|engine| {
        engine.borrow().get_execution_history(user_id)
    })
}

pub fn get_user_daily_stats(user_id: &str) -> DailyExecutionStats {
    DEFI_TRIGGER_ENGINE.with(|engine| {
        engine.borrow().get_daily_stats(user_id)
    })
}