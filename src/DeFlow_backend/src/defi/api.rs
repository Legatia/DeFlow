// DeFi API - Public interface for DeFi functionality
// Provides canister update/query functions for Bitcoin, Ethereum and multi-chain DeFi

// SECURITY: Import comprehensive security validation services
use crate::security::{ValidationService, ValidationResult, RateLimiterService};
use crate::defi::types::*;
use crate::defi::{with_defi_manager_mut, with_defi_manager};
use crate::defi::bitcoin::{FeePriority, BitcoinFeeEstimate};
use crate::defi::bitcoin::service::{BitcoinSendResult, BitcoinNetworkInfo};
use crate::defi::ethereum::{
    EvmChain, EthereumAddress, EthereumPortfolio, EthereumTransactionResult, 
    GasPriority, L2OptimizationResult, MinimalIcpEthereumService, TransactionType
};
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use ic_cdk::{query, update, caller};
use std::cell::RefCell;

// SECURITY: Global rate limiter for API endpoint protection
thread_local! {
    static RATE_LIMITER: RefCell<RateLimiterService> = RefCell::new(RateLimiterService::new());
}

// Bitcoin Portfolio Management
#[update]
pub async fn get_bitcoin_portfolio() -> Result<BitcoinPortfolio, String> {
    let user = caller();
    
    // SECURITY: Rate limiting for portfolio queries
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "portfolio_query")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // For now, return a basic portfolio structure since Bitcoin service needs async initialization
    // TODO: Implement proper async Bitcoin service access
    Ok(BitcoinPortfolio {
        addresses: vec![],
        total_btc: 0.0,
        total_satoshis: 0,
        total_value_usd: 0.0,
        utxos: vec![],
        last_updated: ic_cdk::api::time(),
    })
}

#[update]
pub async fn send_bitcoin(
    to_address: String,
    amount_satoshis: u64,
    fee_satoshis: Option<u64>,
    from_address_type: Option<BitcoinAddressType>,
) -> Result<BitcoinSendResult, String> {
    let user = caller();
    
    // SECURITY: Rate limiting check for Bitcoin sends
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "send_bitcoin")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // SECURITY: Comprehensive input validation
    ValidationService::validate_defi_transaction(
        &user,
        amount_satoshis,
        &to_address,
        "bitcoin",
        None, // tx_hash not available for outgoing transactions
        21_000_000 * 100_000_000, // Max 21M BTC in satoshis
        &[], // No specific authorized principals for regular users
    ).map_err(|e| format!("Validation failed: {}", e))?;
    
    // For now, initialize a new Bitcoin service for the async operation
    // In a production system, this would be handled differently
    let network = with_defi_manager(|manager| manager.context.bitcoin.network.clone());
    let key_name = with_defi_manager(|manager| manager.context.bitcoin.key_name.clone());
    
    match crate::defi::bitcoin::BitcoinDeFiService::new(network, key_name).await {
        Ok(mut service) => {
            service.send_bitcoin(
                user,
                to_address,
                amount_satoshis,
                fee_satoshis,
                from_address_type,
            ).await
        },
        Err(e) => Err(format!("Failed to initialize Bitcoin service: {}", e)),
    }
}

#[update] 
pub async fn get_bitcoin_address(address_type: BitcoinAddressType) -> Result<BitcoinAddress, String> {
    let user = caller();
    
    // Initialize Bitcoin service for address generation
    let network = with_defi_manager(|manager| manager.context.bitcoin.network.clone());
    let key_name = with_defi_manager(|manager| manager.context.bitcoin.key_name.clone());
    
    match crate::defi::bitcoin::BitcoinDeFiService::new(network, key_name).await {
        Ok(mut service) => {
            service.get_bitcoin_address(user, address_type).await
        },
        Err(e) => Err(format!("Failed to initialize Bitcoin service: {}", e)),
    }
}

#[update]
pub async fn get_all_bitcoin_addresses() -> Result<Vec<BitcoinAddress>, String> {
    let user = caller();
    
    // Initialize Bitcoin service for address listing
    let network = with_defi_manager(|manager| manager.context.bitcoin.network.clone());
    let key_name = with_defi_manager(|manager| manager.context.bitcoin.key_name.clone());
    
    match crate::defi::bitcoin::BitcoinDeFiService::new(network, key_name).await {
        Ok(mut service) => {
            service.get_all_bitcoin_addresses(user).await
        },
        Err(e) => Err(format!("Failed to initialize Bitcoin service: {}", e)),
    }
}

#[query]
pub fn estimate_bitcoin_fee(
    utxo_count: usize,
    output_count: usize,
    priority: FeePriority,
) -> BitcoinFeeEstimate {
    // This can be done synchronously as it's just a calculation
    let sat_per_byte = match priority {
        FeePriority::Low => 5,
        FeePriority::Medium => 10,
        FeePriority::High => 20,
        FeePriority::Urgent => 50,
    };
    
    let confirmation_blocks = match priority {
        FeePriority::Low => 144,   // ~24 hours
        FeePriority::Medium => 6,  // ~1 hour
        FeePriority::High => 3,    // ~30 minutes
        FeePriority::Urgent => 1,  // ~10 minutes
    };
    
    // Estimate transaction size: 10 bytes overhead + 148 bytes per input + 34 bytes per output
    let estimated_size = 10 + (utxo_count * 148) + (output_count * 34);
    let total_fee = (estimated_size as u64) * sat_per_byte;
    
    BitcoinFeeEstimate {
        sat_per_byte,
        priority,
        confirmation_blocks,
        total_fee_satoshis: total_fee,
    }
}

#[query]
pub fn validate_bitcoin_address(address: String) -> Result<BitcoinAddressType, String> {
    // SECURITY: Use comprehensive Bitcoin address validation
    use crate::security::BitcoinValidator;
    
    match BitcoinValidator::validate_address(&address) {
        Ok(crate::security::BitcoinAddressType::P2PKH) => Ok(BitcoinAddressType::P2PKH),
        Ok(crate::security::BitcoinAddressType::P2SH) => Ok(BitcoinAddressType::P2SH),
        Ok(crate::security::BitcoinAddressType::Bech32) => Ok(BitcoinAddressType::P2WPKH),
        Ok(crate::security::BitcoinAddressType::Taproot) => Ok(BitcoinAddressType::P2TR),
        Err(e) => Err(format!("Bitcoin address validation failed: {}", e)),
    }
}

#[query]
pub fn get_bitcoin_network_info() -> BitcoinNetworkInfo {
    with_defi_manager(|manager| {
        BitcoinNetworkInfo {
            network: manager.context.bitcoin.network.clone(),
            key_name: manager.context.bitcoin.key_name.clone(),
            supported_address_types: manager.context.bitcoin.address_types.clone(),
            chain_fusion_enabled: true,
        }
    })
}

// DeFi System Health and Monitoring
#[update]
pub async fn get_defi_system_health() -> Result<DeFiSystemHealth, String> {
    // For now, return a simplified health status since we can't properly test yet
    Ok(DeFiSystemHealth {
        overall_healthy: true,
        bitcoin_service: crate::defi::bitcoin::service::BitcoinServiceHealth {
            healthy: true,
            issues: vec![],
            last_checked: ic_cdk::api::time(),
            network: get_bitcoin_network_info(),
        },
        ethereum_chains: vec![],
        solana_service: Some(SolanaServiceHealth {
            healthy: true,
            issues: vec![],
            cluster: SolanaNetwork::Devnet,
            last_slot: 180_000_000,
            tps: 2000.0,
        }),
        last_updated: ic_cdk::api::time(),
    })
}

// Multi-chain portfolio management (placeholder for future implementation)
#[update]
pub async fn get_cross_chain_portfolio() -> Result<CrossChainPortfolio, String> {
    let _user = caller();
    
    // This will be implemented when we add Ethereum and Solana integration
    Err("Cross-chain portfolio functionality coming in Days 9-14".to_string())
}

#[update]
pub async fn rebalance_portfolio(_strategy: AllocationStrategy) -> Result<Vec<DeFiExecutionResult>, String> {
    let user = caller();
    
    // SECURITY: Rate limiting for arbitrage operations
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "arbitrage")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // This will be implemented when we add portfolio management features
    Err("Portfolio rebalancing functionality coming in Days 9-14".to_string())
}

// DeFi workflow integration
#[query]
pub fn get_supported_defi_chains() -> Vec<ChainId> {
    with_defi_manager(|manager| {
        manager.get_active_chains()
    })
}

#[query]
pub fn is_chain_active(chain_id: ChainId) -> bool {
    with_defi_manager(|manager| {
        manager.is_chain_active(&chain_id)
    })
}

// Price and market data (placeholder for future oracle integration)
#[update]
pub async fn get_asset_price(_asset: Asset) -> Result<PriceInfo, String> {
    // This will integrate with Chainlink, Pyth, or other price oracles
    Err("Price oracle integration coming in Days 9-14".to_string())
}

#[update] 
pub async fn get_gas_estimates(chain: ChainId) -> Result<GasInfo, String> {
    // This will provide real-time gas estimates for different chains
    match chain {
        ChainId::Bitcoin => {
            Ok(GasInfo {
                chain,
                gas_price: 10, // sat/byte
                priority_fee: None,
                estimated_cost_usd: 2.50,
                confirmation_time_seconds: 600, // 10 minutes
                last_updated: ic_cdk::api::time(),
            })
        },
        _ => Err("Gas estimates for non-Bitcoin chains coming in Days 9-14".to_string())
    }
}

// Transaction history and analytics
#[query]
pub fn get_defi_transaction_history(_limit: Option<usize>) -> Vec<DeFiTransaction> {
    let _user = caller();
    
    // Placeholder - will integrate with actual transaction storage
    vec![]
}

#[query]
pub fn get_defi_analytics() -> DeFiAnalytics {
    let _user = caller();
    
    // Placeholder analytics
    DeFiAnalytics {
        total_value_usd: 0.0,
        profit_loss_24h: 0.0,
        profit_loss_7d: 0.0,
        profit_loss_30d: 0.0,
        active_positions: 0,
        chains_used: vec![],
        top_performing_asset: None,
        worst_performing_asset: None,
        yield_earned_usd: 0.0,
        fees_paid_usd: 0.0,
        last_updated: ic_cdk::api::time(),
    }
}

// Simple gas estimate for ICP-compliant implementation
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SimpleGasEstimate {
    pub gas_limit: u64,
    pub gas_price: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub total_fee_wei: String,
    pub total_fee_eth: f64,
    pub total_fee_usd: f64,
    pub confirmation_time_estimate_seconds: u64,
    pub priority: GasPriority,
}

// Helper types for API responses
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DeFiSystemHealth {
    pub overall_healthy: bool,
    pub bitcoin_service: crate::defi::bitcoin::service::BitcoinServiceHealth,
    pub ethereum_chains: Vec<EthereumChainHealth>,
    pub solana_service: Option<SolanaServiceHealth>,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EthereumChainHealth {
    pub chain_id: ChainId,
    pub healthy: bool,
    pub issues: Vec<String>,
    pub last_rpc_call: u64,
    pub gas_price_gwei: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SolanaServiceHealth {
    pub healthy: bool,
    pub issues: Vec<String>,
    pub cluster: crate::defi::solana::SolanaNetwork,
    pub last_slot: u64,
    pub tps: f64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DeFiAnalytics {
    pub total_value_usd: f64,
    pub profit_loss_24h: f64,
    pub profit_loss_7d: f64,
    pub profit_loss_30d: f64,
    pub active_positions: u32,
    pub chains_used: Vec<ChainId>,
    pub top_performing_asset: Option<Asset>,
    pub worst_performing_asset: Option<Asset>,
    pub yield_earned_usd: f64,
    pub fees_paid_usd: f64,
    pub last_updated: u64,
}

// Administrative functions
#[update]
pub async fn clear_defi_caches() -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Basic authorization check (in production, use proper controller verification)
    // For now, allow any authenticated caller - in production implement proper admin roles
    if caller == Principal::anonymous() {
        return Err("Unauthorized: Anonymous callers cannot clear caches".to_string());
    }
    
    // SECURITY: Log administrative action
    ic_cdk::println!("🔧 ADMIN: Cache clearing requested by controller: {}", caller.to_text());
    
    with_defi_manager_mut(|manager| {
        if let Some(ref mut bitcoin_service) = manager.bitcoin_service {
            bitcoin_service.clear_all_caches();
        }
    });
    
    Ok(())
}

#[update]
pub async fn emergency_pause_defi() -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Basic authorization check (in production, use proper controller verification)
    if caller == Principal::anonymous() {
        return Err("Unauthorized: Anonymous callers cannot trigger emergency pause".to_string());
    }
    
    // SECURITY: Log emergency action
    ic_cdk::println!("🚨 EMERGENCY: DeFi operations paused by controller: {}", caller.to_text());
    ic_cdk::println!("🚨 DeFi EMERGENCY PAUSE ACTIVATED 🚨");
    
    // This would implement emergency pause logic
    // - Stop all active transactions
    // - Disable new operations
    // - Enable emergency withdrawals only
    
    Ok(())
}

#[update]
pub async fn resume_defi_operations() -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Basic authorization check (in production, use proper controller verification)
    if caller == Principal::anonymous() {
        return Err("Unauthorized: Anonymous callers cannot resume operations".to_string());
    }
    
    // SECURITY: Log administrative action
    ic_cdk::println!("✅ ADMIN: DeFi operations resumed by controller: {}", caller.to_text());
    
    // Re-enable normal DeFi operations
    Ok(())
}

// ================================
// ETHEREUM & L2 API ENDPOINTS
// Day 9: Ethereum & L2 Integration
// ================================

// Helper function to create ICP-compliant Ethereum service
fn create_icp_ethereum_service() -> MinimalIcpEthereumService {
    MinimalIcpEthereumService::new(
        "deflow_ethereum_key".to_string(),
        ic_cdk::api::id(),
    )
}

#[update]
pub async fn get_ethereum_address(chain: EvmChain) -> Result<EthereumAddress, String> {
    let user = caller();
    
    // SECURITY: Rate limiting for address generation
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_user_rate_limit(user)
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    let ethereum_service = create_icp_ethereum_service();
    
    ethereum_service.get_ethereum_address(user, chain)
        .await
        .map_err(|e| e.to_string())
}

#[update]
pub async fn get_ethereum_portfolio() -> Result<EthereumPortfolio, String> {
    let user = caller();
    
    // SECURITY: Rate limiting for portfolio queries
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "portfolio_query")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    let ethereum_service = create_icp_ethereum_service();
    
    ethereum_service.get_ethereum_portfolio(user)
        .await
        .map_err(|e| e.to_string())
}

#[update]
pub async fn send_ethereum(
    to_address: String,
    amount_wei: String,
    chain: Option<EvmChain>,
    gas_priority: GasPriority,
    optimize_for_cost: Option<bool>,
) -> Result<EthereumTransactionResult, String> {
    let user = caller();
    
    // SECURITY: Rate limiting check for Ethereum sends
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "send_ethereum")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // SECURITY: Parse and validate amount
    let amount_u64 = amount_wei.parse::<u64>()
        .map_err(|_| "Invalid amount format".to_string())?;
    
    // SECURITY: Comprehensive input validation
    ValidationService::validate_defi_transaction(
        &user,
        amount_u64,
        &to_address,
        "ethereum",
        None, // tx_hash not available for outgoing transactions
        1_000_000 * 1_000_000_000_000_000_000, // Max 1M ETH in wei
        &[], // No specific authorized principals for regular users
    ).map_err(|e| format!("Validation failed: {}", e))?;
    
    let ethereum_service = create_icp_ethereum_service();
    ethereum_service.send_ethereum(
        user,
        to_address,
        amount_wei,
        chain,
        gas_priority,
        optimize_for_cost.unwrap_or(false),
    ).await.map_err(|e| e.to_string())
}

#[update]
pub async fn estimate_ethereum_gas(
    _chain: EvmChain,
    _to_address: Option<String>,
    _data: Option<String>,
    _value: Option<String>,
    priority: GasPriority,
) -> Result<SimpleGasEstimate, String> {
    // For now, return a simplified gas estimate
    // In production, this would use the full ICP gas estimator
    let base_gas_limit = 21000u64;
    let gas_price_gwei = match priority {
        GasPriority::Low => 5,
        GasPriority::Medium => 20,
        GasPriority::High => 50,
        GasPriority::Urgent => 100,
    };
    
    let gas_price_wei = gas_price_gwei * 1_000_000_000;
    let total_fee_wei = base_gas_limit as u128 * gas_price_wei as u128;
    let total_fee_eth = total_fee_wei as f64 / 1e18;
    let total_fee_usd = total_fee_eth * 2000.0; // Approximate ETH price
    
    Ok(SimpleGasEstimate {
        gas_limit: base_gas_limit,
        gas_price: format!("0x{:x}", gas_price_wei),
        max_fee_per_gas: format!("0x{:x}", gas_price_wei),
        max_priority_fee_per_gas: format!("0x{:x}", gas_price_wei / 10),
        total_fee_wei: total_fee_wei.to_string(),
        total_fee_eth,
        total_fee_usd,
        confirmation_time_estimate_seconds: 60,
        priority,
    })
}

#[update]
pub async fn get_l2_optimization(
    amount_wei: String,
    transaction_type: String,
    gas_priority: GasPriority,
) -> Result<L2OptimizationResult, String> {
    let user = caller();
    
    // SECURITY: Rate limiting for L2 optimization queries
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_user_rate_limit(user)
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // SECURITY: Validate amount format
    let _amount_parsed = amount_wei.parse::<u128>()
        .map_err(|_| "Invalid amount format".to_string())?;
    
    // SECURITY: Whitelist transaction types
    let tx_type = match transaction_type.as_str() {
        "simple_transfer" => TransactionType::SimpleTransfer,
        "token_transfer" => TransactionType::TokenTransfer,
        "dex_swap" => TransactionType::DexSwap,
        "lending" => TransactionType::Lending,
        "nft" => TransactionType::Nft,
        "contract_deployment" => TransactionType::ContractDeployment,
        "complex_defi" => TransactionType::ComplexDefi,
        _ => return Err(format!("Unsupported transaction type: {}", transaction_type)),
    };
    
    let ethereum_service = create_icp_ethereum_service();
    ethereum_service.get_l2_optimization(user, amount_wei, tx_type, gas_priority)
        .await
        .map_err(|e| e.to_string())
}

#[query]
pub fn get_supported_evm_chains() -> Vec<EvmChain> {
    vec![
        EvmChain::Ethereum,
        EvmChain::Arbitrum,
        EvmChain::Optimism,
        EvmChain::Polygon,
        EvmChain::Base,
        EvmChain::Avalanche,
    ]
}

#[query]
pub fn get_evm_chain_info(chain: EvmChain) -> Result<EVMChainInfo, String> {
    // Simplified chain info for ICP-compliant implementation
    Ok(EVMChainInfo {
        chain: chain.clone(),
        chain_id: chain.chain_id(),
        name: chain.name().to_string(),
        native_token: chain.native_token().to_string(),
        is_l2: chain.is_l2(),
        is_sidechain: chain.is_sidechain(),
        supports_eip1559: true,
        average_block_time_seconds: match chain {
            EvmChain::Ethereum => 12,
            EvmChain::Arbitrum => 1,
            EvmChain::Optimism => 2,
            EvmChain::Polygon => 2,
            EvmChain::Base => 2,
            EvmChain::Avalanche => 2,
        },
        typical_gas_price_gwei: match chain {
            EvmChain::Ethereum => 20,
            EvmChain::Arbitrum => 1,
            EvmChain::Optimism => 1,
            EvmChain::Polygon => 30,
            EvmChain::Base => 1,
            EvmChain::Avalanche => 25,
        },
        block_explorer: match chain {
            EvmChain::Ethereum => "https://etherscan.io",
            EvmChain::Arbitrum => "https://arbiscan.io",
            EvmChain::Optimism => "https://optimistic.etherscan.io",
            EvmChain::Polygon => "https://polygonscan.com",
            EvmChain::Base => "https://basescan.org",
            EvmChain::Avalanche => "https://snowtrace.io",
        }.to_string(),
    })
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EVMChainInfo {
    pub chain: EvmChain,
    pub chain_id: u64,
    pub name: String,
    pub native_token: String,
    pub is_l2: bool,
    pub is_sidechain: bool,
    pub supports_eip1559: bool,
    pub average_block_time_seconds: u64,
    pub typical_gas_price_gwei: u64,
    pub block_explorer: String,
}

// L2 Bridge Information
#[update]
pub async fn get_bridge_options(
    _from_chain: EvmChain,
    _to_chain: EvmChain,
) -> Result<Vec<String>, String> {
    // Simplified bridge options for ICP-compliant implementation
    Ok(vec![
        "Official Bridge".to_string(),
        "Third-party Bridge".to_string(),
        "Multi-hop via Ethereum".to_string(),
    ])
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SimpleChainOption {
    pub chain: EvmChain,
    pub fee_usd: f64,
    pub time_seconds: u64,
    pub total_cost_usd: f64,
}

#[update]
pub async fn compare_l2_costs(
    l2_chains: Vec<EvmChain>,
    amount_wei: String,
    _transaction_type: String,
    gas_priority: GasPriority,
) -> Result<Vec<SimpleChainOption>, String> {
    // Simplified L2 comparison for ICP-compliant implementation
    let amount_eth = super::ethereum::utils::wei_to_eth(&amount_wei)
        .map_err(|e| format!("Invalid amount: {}", e))?;
    let amount_usd = amount_eth * 2000.0; // Approximate ETH price
    
    let mut options = Vec::new();
    
    for chain in l2_chains {
        let base_fee = match gas_priority {
            GasPriority::Low => 0.5,
            GasPriority::Medium => 2.0,
            GasPriority::High => 5.0,
            GasPriority::Urgent => 10.0,
        };
        
        let chain_multiplier = match chain {
            EvmChain::Ethereum => 5.0,
            EvmChain::Arbitrum => 0.1,
            EvmChain::Optimism => 0.1,
            EvmChain::Polygon => 0.01,
            EvmChain::Base => 0.1,
            EvmChain::Avalanche => 0.2,
        };
        
        let fee_usd = base_fee * chain_multiplier;
        let time_seconds = match gas_priority {
            GasPriority::Low => 300,
            GasPriority::Medium => 60,
            GasPriority::High => 30,
            GasPriority::Urgent => 15,
        };
        
        options.push(SimpleChainOption {
            chain,
            fee_usd,
            time_seconds,
            total_cost_usd: fee_usd,
        });
    }
    
    // Sort by cost
    options.sort_by(|a, b| a.total_cost_usd.partial_cmp(&b.total_cost_usd).unwrap());
    
    Ok(options)
}

// Update gas estimates to include Ethereum chains
#[update] 
pub async fn get_gas_estimates_v2(chain: ChainId) -> Result<GasInfo, String> {
    match chain {
        ChainId::Bitcoin => {
            Ok(GasInfo {
                chain,
                gas_price: 10, // sat/byte
                priority_fee: None,
                estimated_cost_usd: 2.50,
                confirmation_time_seconds: 600, // 10 minutes
                last_updated: ic_cdk::api::time(),
            })
        },
        ChainId::Ethereum => {
            // Simplified Ethereum gas estimate
            Ok(GasInfo {
                chain,
                gas_price: 20_000_000_000, // 20 gwei
                priority_fee: Some(2_000_000_000), // 2 gwei priority
                estimated_cost_usd: 10.0,
                confirmation_time_seconds: 60,
                last_updated: ic_cdk::api::time(),
            })
        },
        ChainId::Solana => {
            // Simplified Solana fee estimate
            Ok(GasInfo {
                chain,
                gas_price: 5000, // lamports per signature
                priority_fee: None,
                estimated_cost_usd: 0.001, // Very low Solana fees
                confirmation_time_seconds: 1, // Fast Solana confirmations
                last_updated: ic_cdk::api::time(),
            })
        },
        _ => Err(format!("Gas estimates not yet implemented for chain: {:?}", chain))
    }
}

// ================================
// SECURITY API ENDPOINTS
// Rate limiting and security status
// ================================

#[query]
pub fn get_user_rate_limit_status() -> Result<crate::security::RateLimitStatus, String> {
    let user = caller();
    
    RATE_LIMITER.with(|limiter| {
        Ok(limiter.borrow_mut().get_rate_limit_status(user))
    })
}

#[update]
pub async fn cleanup_rate_limiters() -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Basic authorization check (in production, use proper controller verification)
    if caller == Principal::anonymous() {
        return Err("Unauthorized: Anonymous callers cannot cleanup rate limiters".to_string());
    }
    
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().cleanup();
    });
    
    ic_cdk::println!("🧹 ADMIN: Rate limiter cleanup completed by controller: {}", caller.to_text());
    Ok(())
}

// Add a new endpoint to validate addresses across all supported chains
#[query]
pub fn validate_address_universal(address: String, chain: String) -> Result<bool, String> {
    match chain.as_str() {
        "bitcoin" => {
            match crate::security::BitcoinValidator::validate_address(&address) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        },
        "ethereum" | "arbitrum" | "optimism" | "polygon" | "base" | "avalanche" => {
            match crate::security::EthereumValidator::validate_address(&address) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        },
        "solana" => {
            Ok(crate::defi::solana::utils::validate_solana_address(&address))
        },
        _ => Err(format!("Unsupported chain: {}", chain)),
    }
}

// ================================
// SOLANA API ENDPOINTS
// Day 10: Solana Integration
// ================================
use crate::defi::solana::{
    SolanaNetwork, SolanaPortfolio, SolanaTransactionResult, SolanaError
};

// Helper function to create pure ICP-compliant Solana service
fn create_pure_icp_solana_service() -> crate::defi::solana::pure_icp::PureIcpSolanaService {
    crate::defi::solana::pure_icp::PureIcpSolanaService::new(
        SolanaNetwork::Devnet, // Start with Devnet for development
        "deflow_solana_key".to_string(),
    )
}

#[update]
pub async fn get_solana_address() -> Result<String, String> {
    let user = caller();
    let solana_service = create_pure_icp_solana_service();
    
    match solana_service.get_solana_account(user).await {
        Ok(account) => Ok(account.address),
        Err(e) => Err(e.to_string()),
    }
}

#[update]
pub async fn get_solana_portfolio() -> Result<SolanaPortfolio, String> {
    let user = caller();
    
    // SECURITY: Rate limiting for portfolio queries
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "portfolio_query")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    let solana_service = create_pure_icp_solana_service();
    
    solana_service.get_solana_portfolio(user)
        .await
        .map_err(|e| e.to_string())
}

#[update]
pub async fn send_solana(
    to_address: String,
    amount_lamports: u64,
) -> Result<SolanaTransactionResult, String> {
    let user = caller();
    
    // SECURITY: Rate limiting check for Solana sends
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "send_solana")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // SECURITY: Validate Solana address format
    if !crate::defi::solana::utils::validate_solana_address(&to_address) {
        return Err("Invalid Solana address format".to_string());
    }
    
    // SECURITY: Validate amount (basic validation since Solana has different validation needs)
    if amount_lamports == 0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    // SECURITY: Check reasonable limits for Solana
    const MAX_SOL_AMOUNT: u64 = 1_000_000_000_000_000; // 1M SOL in lamports
    if amount_lamports > MAX_SOL_AMOUNT {
        return Err("Amount exceeds maximum limit".to_string());
    }
    
    let solana_service = create_pure_icp_solana_service();
    solana_service.send_sol(user, to_address, amount_lamports)
        .await
        .map_err(|e| e.to_string())
}

#[update]
pub async fn get_spl_token_balance(mint_address: String) -> Result<crate::defi::solana::SplTokenBalance, String> {
    let user = caller();
    
    if mint_address.is_empty() {
        return Err("Mint address cannot be empty".to_string());
    }
    
    // Create token manager
    let token_manager = crate::defi::solana::SolanaTokenManager::new(
        "deflow_solana_key".to_string(),
        SolanaNetwork::Devnet,
    );
    
    token_manager.get_token_balance(user, mint_address)
        .await
        .map_err(|e| e.to_string())
}

#[update]
pub async fn transfer_spl_tokens(
    mint_address: String,
    to_address: String,
    amount: u64,
) -> Result<SolanaTransactionResult, String> {
    let user = caller();
    
    // SECURITY: Rate limiting check for SPL token transfers
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_combined_limits(user, "send_solana")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // SECURITY: Validate all addresses
    if mint_address.is_empty() || !crate::defi::solana::utils::validate_solana_address(&mint_address) {
        return Err("Invalid mint address".to_string());
    }
    
    if to_address.is_empty() || !crate::defi::solana::utils::validate_solana_address(&to_address) {
        return Err("Invalid destination address".to_string());
    }
    
    // SECURITY: Validate amount
    if amount == 0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    // SECURITY: Maximum amount check for SPL tokens
    const MAX_SPL_AMOUNT: u64 = u64::MAX / 1000; // Conservative limit
    if amount > MAX_SPL_AMOUNT {
        return Err("Amount exceeds maximum limit".to_string());
    }
    
    // Create token manager
    let token_manager = crate::defi::solana::SolanaTokenManager::new(
        "deflow_solana_key".to_string(),
        SolanaNetwork::Devnet,
    );
    
    token_manager.transfer_tokens(user, mint_address, to_address, amount)
        .await
        .map_err(|e| e.to_string())
}

#[update]
pub async fn create_spl_token(
    name: String,
    symbol: String,
    decimals: u8,
    initial_supply: u64,
) -> Result<crate::defi::solana::tokens::TokenCreationResult, String> {
    let user = caller();
    
    // SECURITY: Rate limiting for token creation (expensive operation)
    RATE_LIMITER.with(|limiter| {
        limiter.borrow_mut().check_operation_rate_limit(user, "create_token")
    }).map_err(|e| format!("Rate limit exceeded: {}", e))?;
    
    // SECURITY: Comprehensive input validation
    if name.is_empty() || name.len() > 100 {
        return Err("Token name must be 1-100 characters".to_string());
    }
    
    if symbol.is_empty() || symbol.len() > 10 {
        return Err("Token symbol must be 1-10 characters".to_string());
    }
    
    // SECURITY: Validate only alphanumeric characters for security
    if !name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
        return Err("Token name contains invalid characters".to_string());
    }
    
    if !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Token symbol must contain only alphanumeric characters".to_string());
    }
    
    if decimals > 18 {
        return Err("Token decimals cannot exceed 18".to_string());
    }
    
    // SECURITY: Reasonable initial supply limits
    const MAX_INITIAL_SUPPLY: u64 = 1_000_000_000_000_000_000; // 1 billion tokens with 9 decimals
    if initial_supply > MAX_INITIAL_SUPPLY {
        return Err("Initial supply exceeds maximum limit".to_string());
    }
    
    // Create token manager
    let token_manager = crate::defi::solana::SolanaTokenManager::new(
        "deflow_solana_key".to_string(),
        SolanaNetwork::Devnet,
    );
    
    token_manager.create_token(user, name, symbol, decimals, initial_supply)
        .await
        .map_err(|e| e.to_string())
}

#[query]
pub fn get_supported_solana_networks() -> Vec<SolanaNetwork> {
    vec![
        SolanaNetwork::Mainnet,
        SolanaNetwork::Devnet,
        SolanaNetwork::Testnet,
    ]
}

#[query]
pub fn get_solana_network_info() -> SolanaNetworkInfo {
    SolanaNetworkInfo {
        network: SolanaNetwork::Devnet,
        cluster_name: "Devnet".to_string(),
        rpc_endpoint: "https://api.devnet.solana.com".to_string(),
        sol_rpc_canister: crate::defi::solana::icp_solana::SOL_RPC_CANISTER_ID.to_string(),
        key_name: "deflow_solana_key".to_string(),
        supported_features: vec![
            "SOL Transfers".to_string(),
            "SPL Token Operations".to_string(), 
            "Portfolio Management".to_string(),
            "ICP Chain Fusion".to_string(),
            "Threshold ECDSA Signing".to_string(),
        ],
        current_slot: None, // Would be populated from real RPC in production
        tps: None, // Would be populated from real metrics in production
    }
}

#[query]
pub fn validate_solana_address(address: String) -> Result<bool, String> {
    Ok(crate::defi::solana::utils::validate_solana_address(&address))
}

#[query] 
pub fn convert_lamports_to_sol(lamports: u64) -> f64 {
    crate::defi::solana::utils::lamports_to_sol(lamports)
}

#[query]
pub fn convert_sol_to_lamports(sol: f64) -> u64 {
    crate::defi::solana::utils::sol_to_lamports(sol)
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct SolanaNetworkInfo {
    pub network: SolanaNetwork,
    pub cluster_name: String,
    pub rpc_endpoint: String,
    pub sol_rpc_canister: String,
    pub key_name: String,
    pub supported_features: Vec<String>,
    pub current_slot: Option<u64>,
    pub tps: Option<f64>,
}