// Strategy Execution Engine - Core execution logic with safety controls
// Handles the actual execution of different strategy types

use super::*;
use crate::defi::yield_farming::{ChainId, DeFiProtocol};
use crate::defi::arbitrage::{ArbitrageOpportunity, ArbitrageExecutionResult};

/// Core strategy execution engine with safety controls
#[derive(Debug, Clone)]
pub struct StrategyExecutionEngine {
    pub safety_controller: SafetyController,
    pub gas_estimator: GasEstimator,
    pub transaction_builder: TransactionBuilder,
    pub slippage_protector: SlippageProtector,
    pub execution_metrics: ExecutionMetrics,
    pub retry_manager: RetryManager,
}

impl StrategyExecutionEngine {
    pub fn new() -> Self {
        Self {
            safety_controller: SafetyController::new(),
            gas_estimator: GasEstimator::new(),
            transaction_builder: TransactionBuilder::new(),
            slippage_protector: SlippageProtector::new(),
            execution_metrics: ExecutionMetrics::new(),
            retry_manager: RetryManager::new(),
        }
    }

    /// Execute yield farming strategy
    pub async fn execute_yield_farming_strategy(
        &mut self,
        strategy: &ActiveStrategy,
        opportunity: StrategyOpportunity,
        config: &YieldFarmingConfig,
    ) -> Result<StrategyExecutionResult, StrategyError> {
        let execution_id = self.generate_execution_id();
        let start_time = self.get_current_time();

        // Safety checks
        self.safety_controller.pre_execution_safety_check(strategy, &opportunity)?;

        // Extract opportunity details
        let (apy, tokens, pool_address) = match &opportunity.opportunity_type {
            OpportunityType::YieldFarming { apy, tokens, pool_address } => {
                (apy, tokens, pool_address)
            },
            _ => return Err(StrategyError::ExecutionFailed("Invalid opportunity type for yield farming".to_string())),
        };

        // Validate opportunity meets strategy requirements
        if *apy < config.min_apy_threshold {
            return Err(StrategyError::ExecutionFailed(format!("APY {} below threshold {}", apy, config.min_apy_threshold)));
        }

        // Estimate gas costs
        let estimated_gas = self.gas_estimator.estimate_yield_farming_gas(&opportunity.chain, strategy.allocated_capital)?;
        
        if estimated_gas > strategy.config.gas_limit_usd {
            return Err(StrategyError::ExecutionFailed("Gas cost exceeds limit".to_string()));
        }

        // Get wallet address for this chain
        let wallet_address = strategy.wallet_addresses.get(&opportunity.chain)
            .ok_or_else(|| StrategyError::ExecutionFailed(
                format!("No wallet address found for chain: {:?}", opportunity.chain)
            ))?;

        // Build and execute transactions
        // Execute with retry logic - move variables into closure
        let execution_result = self.retry_manager.execute_with_retry(|| async {
            let mut transactions = Vec::new();
            let mut actual_return = 0.0;
            let mut actual_gas_cost = 0.0;
            // Step 1: Approve tokens if needed
            for token in tokens {
                let approve_tx = self.transaction_builder.build_approval_transaction(
                    &opportunity.chain,
                    token,
                    pool_address,
                    strategy.allocated_capital,
                    wallet_address,
                )?;
                
                let approve_result = self.execute_transaction(approve_tx).await?;
                transactions.push(approve_result.transaction_hash.clone());
                actual_gas_cost += approve_result.gas_cost;
            }

            // Step 2: Enter yield farming position
            let farm_tx = self.transaction_builder.build_yield_farming_transaction(
                &opportunity.chain,
                pool_address,
                tokens,
                strategy.allocated_capital,
                wallet_address,
            )?;

            let farm_result = self.execute_transaction(farm_tx).await?;
            transactions.push(farm_result.transaction_hash.clone());
            actual_gas_cost += farm_result.gas_cost;

            // Calculate actual return (simplified - assume daily return)
            let daily_return_rate = apy / 365.0 / 100.0;
            actual_return = strategy.allocated_capital * daily_return_rate;

            Ok((transactions, actual_return, actual_gas_cost))
        }).await;

        let (success, error_message, transactions, actual_return, actual_gas_cost) = match execution_result {
            Ok((transactions, actual_return, actual_gas_cost)) => (true, None, transactions, actual_return, actual_gas_cost),
            Err(e) => (false, Some(e.to_string()), Vec::new(), 0.0, 0.0)
        };

        let execution_time = self.get_current_time() - start_time;

        // Update execution metrics
        self.execution_metrics.record_execution(&strategy.config.strategy_type, success, execution_time, actual_gas_cost);

        Ok(StrategyExecutionResult {
            execution_id,
            strategy_id: strategy.id.clone(),
            user_id: strategy.user_id.clone(),
            opportunity_id: opportunity.id,
            action_type: "yield_farming_enter".to_string(),
            amount_usd: strategy.allocated_capital,
            expected_return: strategy.allocated_capital * (apy / 100.0),
            actual_return,
            gas_cost_usd: actual_gas_cost,
            execution_time_seconds: execution_time / 1_000_000_000,
            success,
            error_message,
            transaction_hashes: transactions,
            executed_at: start_time,
        })
    }

    /// Execute arbitrage strategy
    pub async fn execute_arbitrage_strategy(
        &mut self,
        strategy: &ActiveStrategy,
        opportunity: StrategyOpportunity,
        config: &ArbitrageConfig,
    ) -> Result<StrategyExecutionResult, StrategyError> {
        let execution_id = self.generate_execution_id();
        let start_time = self.get_current_time();

        // Safety checks
        self.safety_controller.pre_execution_safety_check(strategy, &opportunity)?;

        // Extract opportunity details
        let (profit_percentage, token_pair, dex_pair) = match &opportunity.opportunity_type {
            OpportunityType::Arbitrage { profit_percentage, token_pair, dex_pair } => {
                (profit_percentage, token_pair, dex_pair)
            },
            _ => return Err(StrategyError::ExecutionFailed("Invalid opportunity type for arbitrage".to_string())),
        };

        // Validate opportunity meets strategy requirements
        if *profit_percentage < config.min_profit_percentage {
            return Err(StrategyError::ExecutionFailed(format!("Profit {} below threshold {}", profit_percentage, config.min_profit_percentage)));
        }

        // Estimate gas costs
        let estimated_gas = self.gas_estimator.estimate_arbitrage_gas(&opportunity.chain, strategy.allocated_capital)?;
        
        if estimated_gas > strategy.config.gas_limit_usd {
            return Err(StrategyError::ExecutionFailed("Gas cost exceeds limit".to_string()));
        }

        // Get wallet address for this chain
        let wallet_address = strategy.wallet_addresses.get(&opportunity.chain)
            .ok_or_else(|| StrategyError::ExecutionFailed(
                format!("No wallet address found for chain: {:?}", opportunity.chain)
            ))?;

        // Build arbitrage transaction sequence
        let mut transactions = Vec::new();
        let mut actual_return = 0.0;
        let mut actual_gas_cost = 0.0;

        // Execute arbitrage sequence (simplified without timeout for ICP compatibility)
        let execution_result = self.execute_arbitrage_sequence(strategy, &opportunity, token_pair, dex_pair, config).await;

        match execution_result {
            Ok((tx_hashes, return_amount, gas_cost)) => {
                transactions = tx_hashes;
                actual_return = return_amount;
                actual_gas_cost = gas_cost;
            },
            Err(e) => return Err(e),
        }

        let execution_time = self.get_current_time() - start_time;
        self.execution_metrics.record_execution(&strategy.config.strategy_type, true, execution_time, actual_gas_cost);

        Ok(StrategyExecutionResult {
            execution_id,
            strategy_id: strategy.id.clone(),
            user_id: strategy.user_id.clone(),
            opportunity_id: opportunity.id,
            action_type: "arbitrage_execute".to_string(),
            amount_usd: strategy.allocated_capital,
            expected_return: strategy.allocated_capital * (profit_percentage / 100.0),
            actual_return,
            gas_cost_usd: actual_gas_cost,
            execution_time_seconds: execution_time / 1_000_000_000,
            success: true,
            error_message: None,
            transaction_hashes: transactions,
            executed_at: start_time,
        })
    }

    /// Execute rebalancing strategy
    pub async fn execute_rebalancing_strategy(
        &mut self,
        strategy: &ActiveStrategy,
        opportunity: StrategyOpportunity,
        config: &RebalancingConfig,
    ) -> Result<StrategyExecutionResult, StrategyError> {
        let execution_id = self.generate_execution_id();
        let start_time = self.get_current_time();

        // Safety checks
        self.safety_controller.pre_execution_safety_check(strategy, &opportunity)?;

        // Extract rebalancing details
        let (current_allocation, target_allocation) = match &opportunity.opportunity_type {
            OpportunityType::Rebalancing { current_allocation, target_allocation } => {
                (current_allocation, target_allocation)
            },
            _ => return Err(StrategyError::ExecutionFailed("Invalid opportunity type for rebalancing".to_string())),
        };

        // Calculate required trades
        let trades = self.calculate_rebalancing_trades(current_allocation, target_allocation, strategy.allocated_capital)?;

        // Estimate total gas cost
        let estimated_gas = self.gas_estimator.estimate_rebalancing_gas(&opportunity.chain, &trades)?;
        
        if estimated_gas > strategy.config.gas_limit_usd {
            return Err(StrategyError::ExecutionFailed("Total gas cost exceeds limit".to_string()));
        }

        // Get wallet address for this chain
        let wallet_address = strategy.wallet_addresses.get(&opportunity.chain)
            .ok_or_else(|| StrategyError::ExecutionFailed(
                format!("No wallet address found for chain: {:?}", opportunity.chain)
            ))?;

        // Execute rebalancing trades
        let mut transactions = Vec::new();
        let mut actual_gas_cost = 0.0;

        for trade in trades {
            let trade_tx = self.transaction_builder.build_swap_transaction(
                &opportunity.chain,
                &trade.from_token,
                &trade.to_token,
                trade.amount,
                wallet_address,
            )?;

            let trade_result = self.execute_transaction(trade_tx).await?;
            transactions.push(trade_result.transaction_hash.clone());
            actual_gas_cost += trade_result.gas_cost;
        }

        let execution_time = self.get_current_time() - start_time;
        self.execution_metrics.record_execution(&strategy.config.strategy_type, true, execution_time, actual_gas_cost);

        Ok(StrategyExecutionResult {
            execution_id,
            strategy_id: strategy.id.clone(),
            user_id: strategy.user_id.clone(),
            opportunity_id: opportunity.id,
            action_type: "portfolio_rebalance".to_string(),
            amount_usd: strategy.allocated_capital,
            expected_return: 0.0, // Rebalancing doesn't generate direct returns
            actual_return: 0.0,
            gas_cost_usd: actual_gas_cost,
            execution_time_seconds: execution_time / 1_000_000_000,
            success: true,
            error_message: None,
            transaction_hashes: transactions,
            executed_at: start_time,
        })
    }

    /// Execute liquidity mining strategy
    pub async fn execute_liquidity_mining_strategy(
        &mut self,
        strategy: &ActiveStrategy,
        opportunity: StrategyOpportunity,
        config: &LiquidityMiningConfig,
    ) -> Result<StrategyExecutionResult, StrategyError> {
        let execution_id = self.generate_execution_id();
        let start_time = self.get_current_time();

        // Safety checks
        self.safety_controller.pre_execution_safety_check(strategy, &opportunity)?;

        // Extract liquidity mining details
        let (apr, reward_tokens, pool_info) = match &opportunity.opportunity_type {
            OpportunityType::LiquidityMining { apr, reward_tokens, pool_info } => {
                (apr, reward_tokens, pool_info)
            },
            _ => return Err(StrategyError::ExecutionFailed("Invalid opportunity type for liquidity mining".to_string())),
        };

        // Validate opportunity meets strategy requirements
        if *apr < config.min_apr_threshold {
            return Err(StrategyError::ExecutionFailed(format!("APR {} below threshold {}", apr, config.min_apr_threshold)));
        }

        // Get wallet address for this chain
        let wallet_address = strategy.wallet_addresses.get(&opportunity.chain)
            .ok_or_else(|| StrategyError::ExecutionFailed(
                format!("No wallet address found for chain: {:?}", opportunity.chain)
            ))?;

        // Calculate liquidity provision amounts (50/50 split for simplicity)
        let amount_per_token = strategy.allocated_capital / 2.0;
        
        // Execute liquidity provision
        let mut transactions = Vec::new();
        let mut actual_gas_cost = 0.0;

        // Add liquidity transaction
        let lp_tx = self.transaction_builder.build_liquidity_provision_transaction(
            &opportunity.chain,
            pool_info,
            amount_per_token,
            wallet_address,
        )?;

        let lp_result = self.execute_transaction(lp_tx).await?;
        transactions.push(lp_result.transaction_hash.clone());
        actual_gas_cost += lp_result.gas_cost;

        // Calculate expected return
        let expected_return = strategy.allocated_capital * (apr / 100.0) / 12.0; // Monthly return

        let execution_time = self.get_current_time() - start_time;
        self.execution_metrics.record_execution(&strategy.config.strategy_type, true, execution_time, actual_gas_cost);

        Ok(StrategyExecutionResult {
            execution_id,
            strategy_id: strategy.id.clone(),
            user_id: strategy.user_id.clone(),
            opportunity_id: opportunity.id,
            action_type: "liquidity_provision".to_string(),
            amount_usd: strategy.allocated_capital,
            expected_return,
            actual_return: 0.0, // Will be updated when rewards are claimed
            gas_cost_usd: actual_gas_cost,
            execution_time_seconds: execution_time / 1_000_000_000,
            success: true,
            error_message: None,
            transaction_hashes: transactions,
            executed_at: start_time,
        })
    }

    /// Execute dollar cost averaging strategy
    pub async fn execute_dca_strategy(
        &mut self,
        strategy: &ActiveStrategy,
        opportunity: StrategyOpportunity,
        config: &DCAConfig,
    ) -> Result<StrategyExecutionResult, StrategyError> {
        let execution_id = self.generate_execution_id();
        let start_time = self.get_current_time();

        // Safety checks
        self.safety_controller.pre_execution_safety_check(strategy, &opportunity)?;

        // Get wallet address for this chain
        let wallet_address = strategy.wallet_addresses.get(&opportunity.chain)
            .ok_or_else(|| StrategyError::ExecutionFailed(
                format!("No wallet address found for chain: {:?}", opportunity.chain)
            ))?;

        // Calculate purchase amount
        let purchase_amount = config.amount_per_execution.min(strategy.allocated_capital);

        // Build DCA purchase transaction
        let dca_tx = self.transaction_builder.build_token_purchase_transaction(
            &opportunity.chain,
            &config.target_token,
            purchase_amount,
            wallet_address,
        )?;

        // Execute transaction
        let dca_result = self.execute_transaction(dca_tx).await?;

        let execution_time = self.get_current_time() - start_time;
        self.execution_metrics.record_execution(&strategy.config.strategy_type, true, execution_time, dca_result.gas_cost);

        Ok(StrategyExecutionResult {
            execution_id,
            strategy_id: strategy.id.clone(),
            user_id: strategy.user_id.clone(),
            opportunity_id: opportunity.id,
            action_type: "dca_purchase".to_string(),
            amount_usd: purchase_amount,
            expected_return: 0.0, // DCA doesn't have immediate returns
            actual_return: 0.0,
            gas_cost_usd: dca_result.gas_cost,
            execution_time_seconds: execution_time / 1_000_000_000,
            success: true,
            error_message: None,
            transaction_hashes: vec![dca_result.transaction_hash],
            executed_at: start_time,
        })
    }

    /// Execute composite strategy
    pub async fn execute_composite_strategy(
        &mut self,
        strategy: &ActiveStrategy,
        opportunity: StrategyOpportunity,
        configs: &[CompositeStrategyConfig],
    ) -> Result<StrategyExecutionResult, StrategyError> {
        let execution_id = self.generate_execution_id();
        let start_time = self.get_current_time();

        let mut all_transactions = Vec::new();
        let mut total_gas_cost = 0.0;
        let mut total_expected_return = 0.0;
        let mut total_actual_return = 0.0;

        // Execute each sub-strategy based on allocation percentage
        for config in configs {
            let sub_allocation = strategy.allocated_capital * (config.allocation_percentage / 100.0);
            
            // Create temporary strategy for sub-execution
            let mut sub_strategy = strategy.clone();
            sub_strategy.allocated_capital = sub_allocation;

            // Execute based on sub-strategy type
            let sub_result = match &config.sub_strategy {
                StrategyType::YieldFarming(yf_config) => {
                    self.execute_yield_farming_strategy(&sub_strategy, opportunity.clone(), yf_config).await?
                },
                StrategyType::Arbitrage(arb_config) => {
                    self.execute_arbitrage_strategy(&sub_strategy, opportunity.clone(), arb_config).await?
                },
                StrategyType::Rebalancing(reb_config) => {
                    self.execute_rebalancing_strategy(&sub_strategy, opportunity.clone(), reb_config).await?
                },
                StrategyType::LiquidityMining(lm_config) => {
                    self.execute_liquidity_mining_strategy(&sub_strategy, opportunity.clone(), lm_config).await?
                },
                StrategyType::DCA(dca_config) => {
                    self.execute_dca_strategy(&sub_strategy, opportunity.clone(), dca_config).await?
                },
                StrategyType::Composite(_) => {
                    return Err(StrategyError::ExecutionFailed("Nested composite strategies not supported".to_string()));
                },
            };

            // Aggregate results
            all_transactions.extend(sub_result.transaction_hashes);
            total_gas_cost += sub_result.gas_cost_usd;
            total_expected_return += sub_result.expected_return;
            total_actual_return += sub_result.actual_return;
        }

        let execution_time = self.get_current_time() - start_time;
        self.execution_metrics.record_execution(&strategy.config.strategy_type, true, execution_time, total_gas_cost);

        Ok(StrategyExecutionResult {
            execution_id,
            strategy_id: strategy.id.clone(),
            user_id: strategy.user_id.clone(),
            opportunity_id: opportunity.id,
            action_type: "composite_execution".to_string(),
            amount_usd: strategy.allocated_capital,
            expected_return: total_expected_return,
            actual_return: total_actual_return,
            gas_cost_usd: total_gas_cost,
            execution_time_seconds: execution_time / 1_000_000_000,
            success: true,
            error_message: None,
            transaction_hashes: all_transactions,
            executed_at: start_time,
        })
    }

    // Helper methods
    async fn execute_arbitrage_sequence(
        &mut self,
        strategy: &ActiveStrategy,
        opportunity: &StrategyOpportunity,
        token_pair: &(String, String),
        dex_pair: &(String, String),
        config: &ArbitrageConfig,
    ) -> Result<(Vec<String>, f64, f64), StrategyError> {
        let mut transactions = Vec::new();
        let mut gas_cost = 0.0;

        // Get wallet address for this chain
        let wallet_address = strategy.wallet_addresses.get(&opportunity.chain)
            .ok_or_else(|| StrategyError::ExecutionFailed(
                format!("No wallet address found for chain: {:?}", opportunity.chain)
            ))?;

        // Step 1: Buy on first DEX
        let buy_tx = self.transaction_builder.build_dex_trade_transaction(
            &opportunity.chain,
            &dex_pair.0,
            &token_pair.0,
            &token_pair.1,
            strategy.allocated_capital,
            true, // buy
            wallet_address,
        )?;

        let buy_result = self.execute_transaction(buy_tx).await?;
        transactions.push(buy_result.transaction_hash);
        gas_cost += buy_result.gas_cost;

        // Step 2: Sell on second DEX
        let sell_tx = self.transaction_builder.build_dex_trade_transaction(
            &opportunity.chain,
            &dex_pair.1,
            &token_pair.1,
            &token_pair.0,
            strategy.allocated_capital, // This would be the amount of tokens bought
            false, // sell
            wallet_address,
        )?;

        let sell_result = self.execute_transaction(sell_tx).await?;
        transactions.push(sell_result.transaction_hash);
        gas_cost += sell_result.gas_cost;

        // Calculate profit (simplified)
        let profit_percentage = match &opportunity.opportunity_type {
            OpportunityType::Arbitrage { profit_percentage, .. } => *profit_percentage,
            _ => 0.0,
        };
        let actual_return = strategy.allocated_capital * (profit_percentage / 100.0);

        Ok((transactions, actual_return, gas_cost))
    }

    async fn execute_transaction(&self, transaction: Transaction) -> Result<TransactionResult, StrategyError> {
        // Mock transaction execution - in production this would interact with actual blockchains
        let tx_hash = format!("0x{:x}", self.get_current_time());
        let gas_cost = self.gas_estimator.estimate_transaction_gas(&transaction)?;

        // Simulate execution delay
        // In production, this would wait for actual transaction confirmation
        
        Ok(TransactionResult {
            transaction_hash: tx_hash,
            gas_cost,
            success: true,
            block_number: 12345678,
            confirmations: 1,
        })
    }

    fn calculate_rebalancing_trades(
        &self,
        current: &HashMap<String, f64>,
        target: &HashMap<String, f64>,
        total_value: f64,
    ) -> Result<Vec<RebalancingTrade>, StrategyError> {
        let mut trades = Vec::new();

        for (asset, target_pct) in target {
            let current_pct = current.get(asset).unwrap_or(&0.0);
            let difference_pct = target_pct - current_pct;
            
            if difference_pct.abs() > 1.0 { // Only trade if difference > 1%
                let trade_amount = total_value * (difference_pct / 100.0);
                
                if trade_amount > 0.0 {
                    // Need to buy this asset
                    trades.push(RebalancingTrade {
                        from_token: "USDC".to_string(), // Assume we're buying with USDC
                        to_token: asset.clone(),
                        amount: trade_amount,
                        trade_type: TradeType::Buy,
                    });
                } else {
                    // Need to sell this asset
                    trades.push(RebalancingTrade {
                        from_token: asset.clone(),
                        to_token: "USDC".to_string(),
                        amount: trade_amount.abs(),
                        trade_type: TradeType::Sell,
                    });
                }
            }
        }

        Ok(trades)
    }

    fn generate_execution_id(&self) -> String {
        format!("exec_{:x}", self.get_current_time())
    }

    fn get_current_time(&self) -> u64 {
        #[cfg(test)]
        {
            1234567890_u64
        }
        #[cfg(not(test))]
        {
            ic_cdk::api::time()
        }
    }
}

/// Safety controller for pre and post execution checks
#[derive(Debug, Clone)]
pub struct SafetyController {
    pub max_single_execution_usd: f64,
    pub max_daily_volume_usd: f64,
    pub daily_volume_tracker: HashMap<String, f64>, // user_id -> daily volume
    pub last_reset: u64,
}

impl SafetyController {
    pub fn new() -> Self {
        Self {
            max_single_execution_usd: 100000.0,
            max_daily_volume_usd: 500000.0,
            daily_volume_tracker: HashMap::new(),
            last_reset: 0,
        }
    }

    pub fn pre_execution_safety_check(&mut self, strategy: &ActiveStrategy, opportunity: &StrategyOpportunity) -> Result<(), StrategyError> {
        // Check single execution limit
        if strategy.allocated_capital > self.max_single_execution_usd {
            return Err(StrategyError::RiskLimitExceeded(format!("Single execution limit exceeded: ${:.2}", strategy.allocated_capital)));
        }

        // Update and check daily volume
        self.update_daily_volume_tracker();
        let daily_volume = self.daily_volume_tracker.entry(strategy.user_id.clone()).or_insert(0.0);
        
        if *daily_volume + strategy.allocated_capital > self.max_daily_volume_usd {
            return Err(StrategyError::RiskLimitExceeded(format!("Daily volume limit exceeded for user: {}", strategy.user_id)));
        }

        // Check opportunity expiration
        if opportunity.expires_at <= ic_cdk::api::time() {
            return Err(StrategyError::ExecutionFailed("Opportunity expired".to_string()));
        }

        // Update daily volume
        *daily_volume += strategy.allocated_capital;

        Ok(())
    }

    fn update_daily_volume_tracker(&mut self) {
        let current_time = ic_cdk::api::time();
        let one_day_nanos = 24 * 3600 * 1_000_000_000;
        
        if current_time - self.last_reset > one_day_nanos {
            self.daily_volume_tracker.clear();
            self.last_reset = current_time;
        }
    }
}

/// Gas estimator for different transaction types
#[derive(Debug, Clone)]
pub struct GasEstimator {
    pub base_gas_costs: HashMap<String, f64>,
}

impl GasEstimator {
    pub fn new() -> Self {
        let mut base_costs = HashMap::new();
        base_costs.insert("ethereum_swap".to_string(), 150000.0);
        base_costs.insert("ethereum_approval".to_string(), 46000.0);
        base_costs.insert("ethereum_lp_add".to_string(), 200000.0);
        base_costs.insert("arbitrum_swap".to_string(), 150000.0);
        base_costs.insert("polygon_swap".to_string(), 150000.0);
        
        Self {
            base_gas_costs: base_costs,
        }
    }

    pub fn estimate_yield_farming_gas(&self, chain: &ChainId, amount: f64) -> Result<f64, StrategyError> {
        let base_key = format!("{}_lp_add", format!("{:?}", chain).to_lowercase());
        let base_gas = self.base_gas_costs.get(&base_key).unwrap_or(&200000.0);
        
        // Scale with amount (larger amounts may require more gas)
        let amount_multiplier = (amount / 10000.0).max(1.0);
        
        Ok(base_gas * amount_multiplier * self.get_gas_price(chain))
    }

    pub fn estimate_arbitrage_gas(&self, chain: &ChainId, amount: f64) -> Result<f64, StrategyError> {
        let swap_key = format!("{}_swap", format!("{:?}", chain).to_lowercase());
        let swap_gas = self.base_gas_costs.get(&swap_key).unwrap_or(&150000.0);
        
        // Arbitrage typically requires 2 swaps
        let total_gas = swap_gas * 2.0;
        let amount_multiplier = (amount / 10000.0).max(1.0);
        
        Ok(total_gas * amount_multiplier * self.get_gas_price(chain))
    }

    pub fn estimate_rebalancing_gas(&self, chain: &ChainId, trades: &[RebalancingTrade]) -> Result<f64, StrategyError> {
        let swap_key = format!("{}_swap", format!("{:?}", chain).to_lowercase());
        let swap_gas = self.base_gas_costs.get(&swap_key).unwrap_or(&150000.0);
        
        let total_gas = swap_gas * trades.len() as f64;
        Ok(total_gas * self.get_gas_price(chain))
    }

    pub fn estimate_transaction_gas(&self, transaction: &Transaction) -> Result<f64, StrategyError> {
        // Simplified gas estimation based on transaction type
        Ok(match &transaction.transaction_type {
            TransactionType::Swap => 25.0,
            TransactionType::Approval => 10.0,
            TransactionType::LiquidityAdd => 35.0,
            TransactionType::LiquidityRemove => 30.0,
        })
    }

    fn get_gas_price(&self, chain: &ChainId) -> f64 {
        match chain {
            ChainId::Ethereum => 0.00015, // High gas price
            ChainId::Arbitrum => 0.00002,
            ChainId::Polygon => 0.00001,
            ChainId::Solana => 0.000005,
            _ => 0.00005,
        }
    }
}

/// Transaction builder for different operations
#[derive(Debug, Clone)]
pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build_approval_transaction(&self, chain: &ChainId, token: &str, spender: &str, amount: f64, from_address: &str) -> Result<Transaction, StrategyError> {
        Ok(Transaction {
            chain: chain.clone(),
            transaction_type: TransactionType::Approval,
            from_address: from_address.to_string(),
            to_address: token.to_string(),
            data: format!("approve({}, {})", spender, amount),
            value: 0.0,
            gas_limit: 50000,
        })
    }

    pub fn build_yield_farming_transaction(&self, chain: &ChainId, pool: &str, tokens: &[String], amount: f64, from_address: &str) -> Result<Transaction, StrategyError> {
        Ok(Transaction {
            chain: chain.clone(),
            transaction_type: TransactionType::LiquidityAdd,
            from_address: from_address.to_string(),
            to_address: pool.to_string(),
            data: format!("addLiquidity({:?}, {})", tokens, amount),
            value: 0.0,
            gas_limit: 250000,
        })
    }

    pub fn build_swap_transaction(&self, chain: &ChainId, from_token: &str, to_token: &str, amount: f64, from_address: &str) -> Result<Transaction, StrategyError> {
        Ok(Transaction {
            chain: chain.clone(),
            transaction_type: TransactionType::Swap,
            from_address: from_address.to_string(),
            to_address: "0xDEX_ROUTER".to_string(),
            data: format!("swap({}, {}, {})", from_token, to_token, amount),
            value: 0.0,
            gas_limit: 200000,
        })
    }

    pub fn build_liquidity_provision_transaction(&self, chain: &ChainId, pool_info: &str, amount: f64, from_address: &str) -> Result<Transaction, StrategyError> {
        Ok(Transaction {
            chain: chain.clone(),
            transaction_type: TransactionType::LiquidityAdd,
            from_address: from_address.to_string(),
            to_address: pool_info.to_string(),
            data: format!("provideLiquidity({})", amount),
            value: 0.0,
            gas_limit: 300000,
        })
    }

    pub fn build_token_purchase_transaction(&self, chain: &ChainId, token: &str, amount: f64, from_address: &str) -> Result<Transaction, StrategyError> {
        Ok(Transaction {
            chain: chain.clone(),
            transaction_type: TransactionType::Swap,
            from_address: from_address.to_string(),
            to_address: "0xDEX_ROUTER".to_string(),
            data: format!("buyToken({}, {})", token, amount),
            value: 0.0,
            gas_limit: 180000,
        })
    }

    pub fn build_dex_trade_transaction(&self, chain: &ChainId, dex: &str, token_a: &str, token_b: &str, amount: f64, is_buy: bool, from_address: &str) -> Result<Transaction, StrategyError> {
        let action = if is_buy { "buy" } else { "sell" };
        Ok(Transaction {
            chain: chain.clone(),
            transaction_type: TransactionType::Swap,
            from_address: from_address.to_string(),
            to_address: dex.to_string(),
            data: format!("{}({}, {}, {})", action, token_a, token_b, amount),
            value: 0.0,
            gas_limit: 200000,
        })
    }
}

/// Slippage protection mechanism
#[derive(Debug, Clone)]
pub struct SlippageProtector {
    pub max_slippage_tolerance: f64,
}

impl SlippageProtector {
    pub fn new() -> Self {
        Self {
            max_slippage_tolerance: 5.0, // 5% maximum slippage
        }
    }

    pub fn validate_slippage(&self, expected_amount: f64, actual_amount: f64) -> Result<(), StrategyError> {
        let slippage = ((expected_amount - actual_amount) / expected_amount).abs() * 100.0;
        
        if slippage > self.max_slippage_tolerance {
            return Err(StrategyError::ExecutionFailed(format!("Slippage too high: {:.2}%", slippage)));
        }
        
        Ok(())
    }
}

/// Execution metrics tracker
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    pub total_executions: HashMap<String, u32>, // strategy_type -> count
    pub successful_executions: HashMap<String, u32>,
    pub total_gas_spent: HashMap<String, f64>,
    pub avg_execution_time: HashMap<String, f64>,
}

impl ExecutionMetrics {
    pub fn new() -> Self {
        Self {
            total_executions: HashMap::new(),
            successful_executions: HashMap::new(),
            total_gas_spent: HashMap::new(),
            avg_execution_time: HashMap::new(),
        }
    }

    pub fn record_execution(&mut self, strategy_type: &StrategyType, success: bool, execution_time: u64, gas_cost: f64) {
        let type_key = format!("{:?}", strategy_type);
        
        *self.total_executions.entry(type_key.clone()).or_insert(0) += 1;
        
        if success {
            *self.successful_executions.entry(type_key.clone()).or_insert(0) += 1;
        }
        
        *self.total_gas_spent.entry(type_key.clone()).or_insert(0.0) += gas_cost;
        
        // Update average execution time
        let current_avg = self.avg_execution_time.entry(type_key.clone()).or_insert(0.0);
        let total_count = self.total_executions.get(&type_key).unwrap_or(&1);
        *current_avg = (*current_avg * (*total_count - 1) as f64 + execution_time as f64) / *total_count as f64;
    }
}

/// Retry management for failed executions
#[derive(Debug, Clone)]
pub struct RetryManager {
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

impl RetryManager {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            retry_delay_seconds: 5,
        }
    }

    pub async fn execute_with_retry<F, Fut, T>(&self, mut operation: F) -> Result<T, StrategyError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, StrategyError>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    
                    if attempt < self.max_retries {
                        // Wait before retry (in production, use actual async delay)
                        // tokio::time::sleep(std::time::Duration::from_secs(self.retry_delay_seconds)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or(StrategyError::ExecutionFailed("Unknown error during retry".to_string())))
    }
}

// Supporting data structures
#[derive(Debug, Clone)]
pub struct Transaction {
    pub chain: ChainId,
    pub transaction_type: TransactionType,
    pub from_address: String, // Real user wallet address
    pub to_address: String,
    pub data: String,
    pub value: f64,
    pub gas_limit: u64,
}

#[derive(Debug, Clone)]
pub enum TransactionType {
    Swap,
    Approval,
    LiquidityAdd,
    LiquidityRemove,
}

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction_hash: String,
    pub gas_cost: f64,
    pub success: bool,
    pub block_number: u64,
    pub confirmations: u32,
}

#[derive(Debug, Clone)]
pub struct RebalancingTrade {
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
    pub trade_type: TradeType,
}

#[derive(Debug, Clone)]
pub enum TradeType {
    Buy,
    Sell,
}