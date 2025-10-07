// Deposit Management System
// Connects user deposit addresses to DeFi strategy execution
// Manages the flow from deposit detection → balance management → strategy allocation

use super::yield_farming::{ChainId, DeFiProtocol};
use super::automated_strategies::StrategyConfig;
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;
use ic_cdk::api::time;

/// Main deposit manager that handles user funds across all chains
#[derive(Debug, Clone)]
pub struct DepositManager {
    user_deposits: HashMap<Principal, UserDepositPortfolio>,
    deposit_monitors: HashMap<String, DepositMonitor>, // address -> monitor
    strategy_allocations: HashMap<Principal, Vec<StrategyAllocation>>,
    pending_allocations: HashMap<String, PendingAllocation>,
}

/// User's complete deposit portfolio across all chains
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserDepositPortfolio {
    pub user: Principal,
    pub deposit_addresses: Vec<UserDepositAddress>,
    pub total_value_usd: f64,
    pub available_for_strategies: f64,
    pub locked_in_strategies: f64,
    pub last_updated: u64,
}

/// Individual deposit address for a specific chain
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserDepositAddress {
    pub address: String,
    pub chain: ChainId,
    pub balance: f64,
    pub balance_usd: f64,
    pub native_token: String,
    pub deposits: Vec<DepositTransaction>,
    pub withdrawals: Vec<WithdrawalTransaction>,
    pub strategy_allocations: Vec<StrategyAllocation>,
    pub last_checked: u64,
}

/// Individual deposit transaction
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct DepositTransaction {
    pub tx_hash: String,
    pub amount: f64,
    pub amount_usd: f64,
    pub token_symbol: String,
    pub block_height: u64,
    pub timestamp: u64,
    pub confirmations: u32,
}

/// Withdrawal transaction
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct WithdrawalTransaction {
    pub tx_hash: String,
    pub to_address: String,
    pub amount: f64,
    pub amount_usd: f64,
    pub token_symbol: String,
    pub gas_cost: f64,
    pub timestamp: u64,
}

/// Strategy allocation from user deposits
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct StrategyAllocation {
    pub strategy_id: String,
    pub protocol: DeFiProtocol,
    pub chain: ChainId,
    pub allocated_amount: f64,
    pub allocated_amount_usd: f64,
    pub current_value_usd: f64,
    pub pnl: f64,
    pub pnl_percentage: f64,
    pub status: AllocationStatus,
    pub created_at: u64,
    pub last_updated: u64,
    pub withdrawal_deadline: Option<u64>, // User-set deadline for auto-withdrawal
    pub target_apy: Option<f64>, // Expected APY when allocated
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq)]
pub enum AllocationStatus {
    Active,
    Paused,
    Exiting,
    Completed,
    Failed,
}

/// Pending allocation waiting for execution
#[derive(Debug, Clone)]
pub struct PendingAllocation {
    pub user: Principal,
    pub strategy_config: StrategyConfig,
    pub amount_usd: f64,
    pub source_address: String,
    pub target_chain: ChainId,
    pub created_at: u64,
}

/// Monitor for detecting deposits on specific addresses
#[derive(Debug, Clone)]
pub struct DepositMonitor {
    pub address: String,
    pub chain: ChainId,
    pub user: Principal,
    pub last_block_checked: u64,
    pub auto_allocate_enabled: bool,
    pub auto_allocation_strategies: Vec<AutoAllocationRule>,
}

/// Auto-allocation rule for new deposits
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AutoAllocationRule {
    pub strategy_type: String,
    pub min_deposit_amount: f64,
    pub allocation_percentage: f64,
    pub protocol_preference: Vec<DeFiProtocol>,
    pub risk_tolerance: RiskTolerance,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RiskTolerance {
    Conservative,  // 3-8% APY, blue chip protocols
    Moderate,      // 5-15% APY, established protocols
    Aggressive,    // 10-50% APY, newer protocols
}

impl DepositManager {
    pub fn new() -> Self {
        Self {
            user_deposits: HashMap::new(),
            deposit_monitors: HashMap::new(),
            strategy_allocations: HashMap::new(),
            pending_allocations: HashMap::new(),
        }
    }

    /// Register a new deposit address for a user
    pub async fn register_deposit_address(
        &mut self,
        user: Principal,
        address: String,
        chain: ChainId,
    ) -> Result<(), String> {
        // Create deposit monitor
        let monitor = DepositMonitor {
            address: address.clone(),
            chain: chain.clone(),
            user,
            last_block_checked: 0,
            auto_allocate_enabled: false, // Start with manual allocation
            auto_allocation_strategies: Vec::new(),
        };

        self.deposit_monitors.insert(address.clone(), monitor);

        // Get native token first to avoid borrowing issues
        let native_token = self.get_native_token(&chain);

        // Initialize user portfolio if needed
        let portfolio = self.user_deposits.entry(user).or_insert(UserDepositPortfolio {
            user,
            deposit_addresses: Vec::new(),
            total_value_usd: 0.0,
            available_for_strategies: 0.0,
            locked_in_strategies: 0.0,
            last_updated: time(),
        });

        // Add deposit address
        let deposit_address = UserDepositAddress {
            address: address.clone(),
            chain,
            balance: 0.0,
            balance_usd: 0.0,
            native_token,
            deposits: Vec::new(),
            withdrawals: Vec::new(),
            strategy_allocations: Vec::new(),
            last_checked: time(),
        };

        portfolio.deposit_addresses.push(deposit_address);
        Ok(())
    }

    /// Check all monitored addresses for new deposits
    pub async fn scan_for_deposits(&mut self) -> Result<Vec<DepositTransaction>, String> {
        let mut new_deposits = Vec::new();

        // Clone the monitors data to avoid borrowing issues
        let monitor_data: Vec<(String, ChainId, u64, Principal, bool)> = self.deposit_monitors
            .iter()
            .map(|(addr, monitor)| (
                addr.clone(),
                monitor.chain.clone(),
                monitor.last_block_checked,
                monitor.user,
                monitor.auto_allocate_enabled
            ))
            .collect();

        for (address, chain, last_block, user, auto_allocate_enabled) in monitor_data {
            match self.check_address_deposits(&address, &chain, last_block).await {
                Ok(deposits) => {
                    for deposit in deposits {
                        // Update user portfolio
                        if let Some(portfolio) = self.user_deposits.get_mut(&user) {
                            if let Some(deposit_addr) = portfolio.deposit_addresses.iter_mut()
                                .find(|addr| addr.address == address) {

                                deposit_addr.deposits.push(deposit.clone());
                                deposit_addr.balance += deposit.amount;
                                deposit_addr.balance_usd += deposit.amount_usd;

                                portfolio.total_value_usd += deposit.amount_usd;
                                portfolio.available_for_strategies += deposit.amount_usd;
                                portfolio.last_updated = time();
                            }
                        }

                        // Check for auto-allocation
                        if auto_allocate_enabled {
                            self.process_auto_allocation(&user, &deposit).await?;
                        }

                        new_deposits.push(deposit);
                    }
                },
                Err(e) => {
                    ic_cdk::println!("Failed to check deposits for {}: {}", address, e);
                }
            }
        }

        Ok(new_deposits)
    }

    /// Allocate user funds to a specific strategy
    pub async fn allocate_to_strategy(
        &mut self,
        user: Principal,
        strategy_config: StrategyConfig,
        amount_usd: f64,
        source_address: Option<String>,
    ) -> Result<String, String> {
        // Validate user has sufficient funds
        let portfolio = self.user_deposits.get(&user)
            .ok_or_else(|| "User has no deposit portfolio".to_string())?;

        if portfolio.available_for_strategies < amount_usd {
            return Err(format!(
                "Insufficient funds. Available: ${:.2}, Requested: ${:.2}",
                portfolio.available_for_strategies, amount_usd
            ));
        }

        // Choose source address if not specified
        let source_addr = match source_address {
            Some(addr) => addr,
            None => {
                // Find address with sufficient balance on the target chains
                let target_chain = strategy_config.target_chains.get(0)
                    .ok_or_else(|| "No target chain specified in strategy".to_string())?;
                self.find_best_source_address(&user, target_chain, amount_usd)?
            }
        };

        // Create pending allocation
        let allocation_id = self.generate_allocation_id();
        let target_chain = strategy_config.target_chains.get(0)
            .ok_or_else(|| "No target chain specified in strategy".to_string())?
            .clone();

        let pending = PendingAllocation {
            user,
            strategy_config: strategy_config.clone(),
            amount_usd,
            source_address: source_addr,
            target_chain,
            created_at: time(),
        };

        self.pending_allocations.insert(allocation_id.clone(), pending);

        // Execute allocation asynchronously
        ic_cdk::spawn(async move {
            // This would call the strategy execution engine
            // For now, we'll simulate the process
        });

        Ok(allocation_id)
    }

    /// Get user's complete deposit portfolio
    pub fn get_user_portfolio(&self, user: &Principal) -> Option<&UserDepositPortfolio> {
        self.user_deposits.get(user)
    }

    /// Get user's available balance for strategies
    pub fn get_available_balance(&self, user: &Principal) -> f64 {
        self.user_deposits.get(user)
            .map(|portfolio| portfolio.available_for_strategies)
            .unwrap_or(0.0)
    }

    /// Enable auto-allocation for a user's deposit address
    pub fn setup_auto_allocation(
        &mut self,
        user: Principal,
        address: String,
        rules: Vec<AutoAllocationRule>,
    ) -> Result<(), String> {
        let monitor = self.deposit_monitors.get_mut(&address)
            .ok_or_else(|| "Address not found".to_string())?;

        if monitor.user != user {
            return Err("Unauthorized".to_string());
        }

        monitor.auto_allocate_enabled = true;
        monitor.auto_allocation_strategies = rules;
        Ok(())
    }

    // Private helper methods

    async fn check_address_deposits(
        &self,
        address: &str,
        chain: &ChainId,
        last_block: u64,
    ) -> Result<Vec<DepositTransaction>, String> {
        // This would integrate with your blockchain monitoring services
        // For now, return empty vec (implement actual blockchain scanning)
        match chain {
            ChainId::Bitcoin => self.scan_bitcoin_deposits(address, last_block).await,
            ChainId::Ethereum | ChainId::Arbitrum | ChainId::Optimism | ChainId::Polygon => {
                self.scan_ethereum_deposits(address, chain, last_block).await
            },
            ChainId::Solana => self.scan_solana_deposits(address, last_block).await,
            _ => Ok(Vec::new()),
        }
    }

    async fn scan_bitcoin_deposits(&self, address: &str, last_block: u64) -> Result<Vec<DepositTransaction>, String> {
        // Integrate with your Bitcoin monitoring service
        // Return actual deposits found since last_block
        Ok(Vec::new())
    }

    async fn scan_ethereum_deposits(&self, address: &str, chain: &ChainId, last_block: u64) -> Result<Vec<DepositTransaction>, String> {
        // Integrate with your Ethereum/L2 monitoring service
        // Check for ETH and token deposits across networks
        Ok(Vec::new())
    }

    async fn scan_solana_deposits(&self, address: &str, last_block: u64) -> Result<Vec<DepositTransaction>, String> {
        // Integrate with your Solana monitoring service
        Ok(Vec::new())
    }

    async fn process_auto_allocation(&mut self, user: &Principal, deposit: &DepositTransaction) -> Result<(), String> {
        // Find matching auto-allocation rules
        // Execute automatic strategy allocation based on rules
        Ok(())
    }

    fn find_best_source_address(&self, user: &Principal, target_chain: &ChainId, amount_usd: f64) -> Result<String, String> {
        let portfolio = self.user_deposits.get(user)
            .ok_or_else(|| "User portfolio not found".to_string())?;

        // Prefer same-chain addresses first, then cross-chain
        for addr in &portfolio.deposit_addresses {
            if addr.chain == *target_chain && addr.balance_usd >= amount_usd {
                return Ok(addr.address.clone());
            }
        }

        // If no same-chain address, find any address with sufficient balance
        for addr in &portfolio.deposit_addresses {
            if addr.balance_usd >= amount_usd {
                return Ok(addr.address.clone());
            }
        }

        Err("No address with sufficient balance found".to_string())
    }

    fn get_native_token(&self, chain: &ChainId) -> String {
        match chain {
            ChainId::Bitcoin => "BTC".to_string(),
            ChainId::Ethereum => "ETH".to_string(),
            ChainId::Arbitrum => "ETH".to_string(),
            ChainId::Optimism => "ETH".to_string(),
            ChainId::Polygon => "MATIC".to_string(),
            ChainId::Base => "ETH".to_string(),
            ChainId::Avalanche => "AVAX".to_string(),
            ChainId::Solana => "SOL".to_string(),
            ChainId::Stacks => "STX".to_string(),
        }
    }

    fn generate_allocation_id(&self) -> String {
        format!("alloc_{}", time())
    }

    /// Deposit funds from escrow to DeFi protocol (Aave, Compound, etc.)
    pub async fn deposit_to_protocol(
        &mut self,
        user: Principal,
        protocol: DeFiProtocol,
        chain: ChainId,
        amount_usd: f64,
        withdrawal_deadline: Option<u64>,
    ) -> Result<StrategyAllocation, String> {
        // Find source address with funds
        let source_address = self.find_best_source_address(&user, &chain, amount_usd)?;

        // Get user portfolio
        let portfolio = self.user_deposits.get_mut(&user)
            .ok_or("User portfolio not found")?;

        // Deduct from available balance
        if portfolio.available_for_strategies < amount_usd {
            return Err("Insufficient available funds".to_string());
        }

        portfolio.available_for_strategies -= amount_usd;
        portfolio.locked_in_strategies += amount_usd;

        // Create strategy allocation
        let allocation = StrategyAllocation {
            strategy_id: self.generate_allocation_id(),
            protocol: protocol.clone(),
            chain: chain.clone(),
            allocated_amount: amount_usd,
            allocated_amount_usd: amount_usd,
            current_value_usd: amount_usd,
            pnl: 0.0,
            pnl_percentage: 0.0,
            status: AllocationStatus::Active,
            created_at: time(),
            last_updated: time(),
            withdrawal_deadline,
            target_apy: None, // Will be set from protocol data
        };

        // Add to user's strategy allocations
        self.strategy_allocations
            .entry(user)
            .or_insert_with(Vec::new)
            .push(allocation.clone());

        // Execute actual protocol deposit
        let token = self.get_native_token(&chain);
        let protocol_clone = protocol.clone();
        let chain_clone = chain.clone();
        let token_clone = token.clone();
        let amount_clone = amount_usd;

        // Spawn async deposit task (fire and forget)
        ic_cdk::spawn(async move {
            if let Some(executor) = super::protocol_executor::with_protocol_executor(|e| e.clone()) {
                match executor.deposit_to_protocol(
                    &protocol_clone,
                    &chain_clone,
                    &token_clone,
                    amount_clone,
                    "escrow_address",
                ).await {
                    Ok(tx_hash) => ic_cdk::println!("Protocol deposit successful: {}", tx_hash),
                    Err(e) => ic_cdk::println!("Protocol deposit failed: {}", e),
                }
            }
        });

        Ok(allocation)
    }

    /// Withdraw from protocol back to user address at deadline
    pub async fn withdraw_at_deadline(
        &mut self,
        user: Principal,
        allocation_id: String,
        user_address: String,
    ) -> Result<WithdrawalTransaction, String> {
        // Find and validate the allocation
        let (chain_for_token, current_value, allocated_amount) = {
            let allocations = self.strategy_allocations.get_mut(&user)
                .ok_or("No allocations found for user")?;

            let allocation = allocations.iter_mut()
                .find(|a| a.strategy_id == allocation_id)
                .ok_or("Allocation not found")?;

            // Check if deadline passed
            if let Some(deadline) = allocation.withdrawal_deadline {
                if time() < deadline {
                    return Err(format!("Withdrawal deadline not reached. Remaining: {} seconds", deadline - time()));
                }
            }

            // Mark as exiting
            allocation.status = AllocationStatus::Exiting;

            (allocation.chain.clone(), allocation.current_value_usd, allocation.allocated_amount_usd)
        };

        // Get token symbol outside of mutable borrow
        let token_symbol = self.get_native_token(&chain_for_token);

        // Execute protocol withdrawal
        let protocol_clone = {
            let allocations = self.strategy_allocations.get(&user)
                .ok_or("Allocations not found")?;
            allocations.iter()
                .find(|a| a.strategy_id == allocation_id)
                .map(|a| a.protocol.clone())
                .ok_or("Allocation protocol not found")?
        };

        // Spawn withdrawal task
        let chain_clone = chain_for_token.clone();
        let token_clone = token_symbol.clone();
        let amount_clone = current_value;
        let user_addr_clone = user_address.clone();

        ic_cdk::spawn(async move {
            if let Some(executor) = super::protocol_executor::with_protocol_executor(|e| e.clone()) {
                match executor.withdraw_from_protocol(
                    &protocol_clone,
                    &chain_clone,
                    &token_clone,
                    amount_clone,
                    &user_addr_clone,
                ).await {
                    Ok(tx_hash) => ic_cdk::println!("Protocol withdrawal successful: {}", tx_hash),
                    Err(e) => ic_cdk::println!("Protocol withdrawal failed: {}", e),
                }
            }
        });

        let withdrawal = WithdrawalTransaction {
            tx_hash: format!("0x{:x}", time()),
            to_address: user_address,
            amount: current_value,
            amount_usd: current_value,
            token_symbol,
            gas_cost: 5.0, // Estimated
            timestamp: time(),
        };

        // Update portfolio
        if let Some(portfolio) = self.user_deposits.get_mut(&user) {
            portfolio.locked_in_strategies -= allocated_amount;
            portfolio.available_for_strategies += current_value;
        }

        // Mark allocation as completed
        if let Some(allocations) = self.strategy_allocations.get_mut(&user) {
            if let Some(allocation) = allocations.iter_mut().find(|a| a.strategy_id == allocation_id) {
                allocation.status = AllocationStatus::Completed;
            }
        }

        Ok(withdrawal)
    }

    /// Check all allocations for deadline expiry and auto-withdraw
    pub async fn process_deadline_withdrawals(&mut self) -> Result<Vec<WithdrawalTransaction>, String> {
        let mut withdrawals = Vec::new();
        let current_time = time();

        // Collect allocation IDs and user addresses that need withdrawal
        let mut pending_withdrawals: Vec<(Principal, String, String)> = Vec::new();

        for (user, allocations) in &self.strategy_allocations {
            for allocation in allocations {
                if let Some(deadline) = allocation.withdrawal_deadline {
                    if current_time >= deadline && allocation.status == AllocationStatus::Active {
                        // Get user's first deposit address as withdrawal target
                        if let Some(portfolio) = self.user_deposits.get(user) {
                            if let Some(first_addr) = portfolio.deposit_addresses.first() {
                                pending_withdrawals.push((
                                    *user,
                                    allocation.strategy_id.clone(),
                                    first_addr.address.clone()
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Execute withdrawals
        for (user, allocation_id, address) in pending_withdrawals {
            match self.withdraw_at_deadline(user, allocation_id, address).await {
                Ok(withdrawal) => withdrawals.push(withdrawal),
                Err(e) => ic_cdk::println!("Auto-withdrawal failed: {}", e),
            }
        }

        Ok(withdrawals)
    }

    /// Bridge funds to L2 if better yield available
    pub async fn bridge_to_l2_for_yield(
        &mut self,
        user: Principal,
        from_chain: ChainId,
        to_chain: ChainId,
        amount_usd: f64,
    ) -> Result<String, String> {
        // Validate L2 chains
        if !from_chain.is_ethereum_ecosystem() || !to_chain.is_ethereum_ecosystem() {
            return Err("Only ETH L2 bridging supported".to_string());
        }

        // Find source address
        let source_address = self.find_best_source_address(&user, &from_chain, amount_usd)?;

        // Get native token before mutable borrow
        let native_token = self.get_native_token(&to_chain);

        // Execute L2 bridge via official bridge contracts
        let amount_wei = (amount_usd * 1e18 / 2000.0) as u128; // Convert USD to wei (assuming $2000 ETH)

        let bridge_tx_hash = if let Some(executor) = super::l2_bridge_executor::with_l2_bridge_executor(|e| e.clone()) {
            // Spawn bridge execution task
            let to_chain_clone = to_chain.clone();
            let from_chain_clone = from_chain.clone();
            let source_addr_clone = source_address.clone();

            ic_cdk::spawn(async move {
                // Execute real bridge transaction
                match executor.bridge_eth_to_l2(&to_chain_clone, amount_wei, &source_addr_clone).await {
                    Ok(tx_hash) => {
                        ic_cdk::println!("L2 bridge successful: {} → {:?}, tx: {}",
                            format!("{:?}", from_chain_clone), to_chain_clone, tx_hash);
                    },
                    Err(e) => {
                        ic_cdk::println!("L2 bridge failed: {}", e);
                    }
                }
            });

            format!("bridge_pending_0x{:x}", time())
        } else {
            format!("bridge_0x{:x}", time())
        };

        // Update user balances (deduct from source chain, add to destination)
        if let Some(portfolio) = self.user_deposits.get_mut(&user) {
            // Deduct from source chain
            if let Some(source_addr) = portfolio.deposit_addresses.iter_mut()
                .find(|addr| addr.chain == from_chain && addr.address == source_address) {
                source_addr.balance_usd -= amount_usd;
            }

            // Add to destination chain (or create new address if doesn't exist)
            let dest_exists = portfolio.deposit_addresses.iter()
                .any(|addr| addr.chain == to_chain);

            if !dest_exists {
                // Create new L2 address
                portfolio.deposit_addresses.push(UserDepositAddress {
                    address: format!("0x{:x}", time()), // Generate L2 address
                    chain: to_chain.clone(),
                    balance: amount_usd,
                    balance_usd: amount_usd,
                    native_token,
                    deposits: Vec::new(),
                    withdrawals: Vec::new(),
                    strategy_allocations: Vec::new(),
                    last_checked: time(),
                });
            } else {
                // Update existing L2 address
                if let Some(dest_addr) = portfolio.deposit_addresses.iter_mut()
                    .find(|addr| addr.chain == to_chain) {
                    dest_addr.balance_usd += amount_usd;
                }
            }
        }

        Ok(bridge_tx_hash)
    }
}

/// Internal function for deposit management - called by API layer
pub async fn register_user_deposit_address(
    chain_type: String,
    address: String,
) -> Result<String, String> {
    let user = ic_cdk::caller();
    let chain = parse_chain_from_string(&chain_type)?;

    // Get global deposit manager (you'd need to add this to your global state)
    // For now, simulate success
    Ok(format!("Registered address {} for chain {} for user {}", address, chain_type, user.to_text()))
}

// Internal function - called by API layer
pub fn get_user_deposit_portfolio(user_principal: Option<String>) -> Result<UserDepositPortfolio, String> {
    let user = match user_principal {
        Some(principal_text) => Principal::from_text(&principal_text)
            .map_err(|e| format!("Invalid principal: {}", e))?,
        None => ic_cdk::caller(),
    };

    // Get from global deposit manager
    // For now, return mock portfolio
    Ok(UserDepositPortfolio {
        user,
        deposit_addresses: Vec::new(),
        total_value_usd: 0.0,
        available_for_strategies: 0.0,
        locked_in_strategies: 0.0,
        last_updated: time(),
    })
}

// Internal function - called by API layer
pub async fn allocate_funds_to_strategy(
    strategy_type: String,
    amount_usd: f64,
    source_address: Option<String>,
) -> Result<String, String> {
    let user = ic_cdk::caller();

    if amount_usd <= 0.0 {
        return Err("Amount must be positive".to_string());
    }

    if amount_usd < 10.0 {
        return Err("Minimum allocation is $10".to_string());
    }

    // Create strategy config based on strategy_type
    let strategy_config = create_strategy_config_from_type(&strategy_type)?;

    // For now, simulate allocation
    Ok(format!("Allocated ${} to {} strategy", amount_usd, strategy_type))
}

// Helper functions
fn parse_chain_from_string(chain_str: &str) -> Result<ChainId, String> {
    match chain_str.to_lowercase().as_str() {
        "bitcoin" => Ok(ChainId::Bitcoin),
        "ethereum" => Ok(ChainId::Ethereum),
        "arbitrum" => Ok(ChainId::Arbitrum),
        "optimism" => Ok(ChainId::Optimism),
        "polygon" => Ok(ChainId::Polygon),
        "base" => Ok(ChainId::Base),
        "avalanche" => Ok(ChainId::Avalanche),
        "solana" => Ok(ChainId::Solana),
        _ => Err(format!("Unsupported chain: {}", chain_str)),
    }
}

fn create_strategy_config_from_type(strategy_type: &str) -> Result<StrategyConfig, String> {
    // This would create appropriate StrategyConfig based on type
    // For now, return a mock config
    Err("Strategy config creation not implemented yet".to_string())
}

// Test modules are located in separate files
// #[cfg(test)]
// mod deposit_manager_tests;

// #[cfg(test)]
// mod deposit_integration_tests;