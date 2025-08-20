// Bitcoin DeFi Service - Main service layer for Bitcoin Chain Fusion
// Integrates address management, UTXO handling, and transaction creation

use crate::defi::types::*;
use crate::defi::bitcoin::{
    BitcoinContext, BitcoinAddressManager, UTXOManager, 
    FeePriority, BitcoinFeeEstimate
};
use crate::defi::bitcoin::transactions::{BitcoinTransactionBuilder, TransactionParams};
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;

// Main Bitcoin DeFi service
#[allow(dead_code)]
pub struct BitcoinDeFiService {
    context: BitcoinContext,
    address_manager: BitcoinAddressManager,
    utxo_manager: UTXOManager,
    transaction_builder: BitcoinTransactionBuilder,
    user_portfolios: HashMap<Principal, BitcoinPortfolio>,
}

#[allow(dead_code)]
impl BitcoinDeFiService {
    // Initialize Bitcoin DeFi service
    pub async fn new(network: BitcoinNetwork, key_name: String) -> Result<Self, String> {
        let context = BitcoinContext::new(network, key_name);
        let address_manager = BitcoinAddressManager::new(context.clone());
        let utxo_manager = UTXOManager::new(context.clone());
        let transaction_builder = BitcoinTransactionBuilder::new(context.clone());
        
        ic_cdk::println!("Bitcoin DeFi service initialized successfully");
        
        Ok(Self {
            context,
            address_manager,
            utxo_manager,
            transaction_builder,
            user_portfolios: HashMap::new(),
        })
    }
    
    // Get or create user's Bitcoin portfolio
    pub async fn get_user_portfolio(&mut self, user: Principal) -> Result<BitcoinPortfolio, String> {
        // Check if we have cached portfolio
        if let Some(portfolio) = self.user_portfolios.get(&user) {
            let now = ic_cdk::api::time();
            let cache_duration = 300_000_000_000; // 5 minutes in nanoseconds
            
            if now - portfolio.last_updated < cache_duration {
                return Ok(portfolio.clone());
            }
        }
        
        // Generate fresh portfolio
        let portfolio = self.build_user_portfolio(user).await?;
        self.user_portfolios.insert(user, portfolio.clone());
        
        Ok(portfolio)
    }
    
    // Build user's Bitcoin portfolio from scratch
    async fn build_user_portfolio(&mut self, user: Principal) -> Result<BitcoinPortfolio, String> {
        // Generate all address types for user
        let addresses = self.address_manager.get_all_addresses(user).await?;
        
        if addresses.is_empty() {
            return Err("Failed to generate Bitcoin addresses for user".to_string());
        }
        
        // Get UTXOs and balances for all addresses
        let mut updated_addresses = Vec::new();
        let mut all_utxos = Vec::new();
        let mut total_satoshis = 0u64;
        
        for mut address in addresses {
            // Get balance for this address
            match self.utxo_manager.get_balance(address.address.clone()).await {
                Ok(balance) => {
                    address.balance_satoshis = balance;
                    total_satoshis = total_satoshis.saturating_add(balance);
                },
                Err(e) => {
                    ic_cdk::println!("Failed to get balance for address {}: {}", address.address, e);
                }
            }
            
            // Get UTXOs for this address
            match self.utxo_manager.get_utxos(address.address.clone()).await {
                Ok(utxos) => {
                    address.utxo_count = utxos.len() as u32;
                    all_utxos.extend(utxos);
                },
                Err(e) => {
                    ic_cdk::println!("Failed to get UTXOs for address {}: {}", address.address, e);
                    address.utxo_count = 0;
                }
            }
            
            updated_addresses.push(address);
        }
        
        // Calculate BTC amount (1 BTC = 100,000,000 satoshis)
        let total_btc = (total_satoshis as f64) / 100_000_000.0;
        
        // Get current BTC price (placeholder - would integrate with price oracle)
        let btc_price_usd = 45000.0; // Placeholder price
        let total_value_usd = total_btc * btc_price_usd;
        
        Ok(BitcoinPortfolio {
            addresses: updated_addresses,
            total_btc,
            total_satoshis,
            total_value_usd,
            utxos: all_utxos,
            last_updated: ic_cdk::api::time(),
        })
    }
    
    // Send Bitcoin transaction
    pub async fn send_bitcoin(
        &mut self,
        user: Principal,
        to_address: String,
        amount_satoshis: u64,
        fee_satoshis: Option<u64>,
        from_address_type: Option<BitcoinAddressType>,
    ) -> Result<BitcoinSendResult, String> {
        // Get user's portfolio to find addresses with sufficient balance
        let portfolio = self.get_user_portfolio(user).await?;
        
        // Select source address based on type preference or highest balance
        let source_address = match from_address_type {
            Some(addr_type) => {
                portfolio.addresses
                    .iter()
                    .filter(|addr| addr.address_type == addr_type)
                    .max_by_key(|addr| addr.balance_satoshis)
                    .ok_or("No address of specified type found")?
                    .clone()
            },
            None => {
                portfolio.addresses
                    .iter()
                    .max_by_key(|addr| addr.balance_satoshis)
                    .ok_or("No addresses available")?
                    .clone()
            }
        };
        
        // Calculate fees
        let estimated_fee = fee_satoshis.unwrap_or_else(|| {
            self.transaction_builder.estimate_fee(2, 2, 10) // Conservative estimate
        });
        
        let total_needed = amount_satoshis.saturating_add(estimated_fee);
        
        if source_address.balance_satoshis < total_needed {
            return Err(format!(
                "Insufficient balance: need {} satoshis, have {} satoshis",
                total_needed,
                source_address.balance_satoshis
            ));
        }
        
        // Get UTXOs for the source address
        let utxos = self.utxo_manager.select_utxos_for_amount(
            source_address.address.clone(),
            total_needed,
        ).await?;
        
        // Create transaction parameters
        let tx_params = TransactionParams {
            from_address: source_address.address.clone(),
            to_address: to_address.clone(),
            amount_satoshis,
            fee_satoshis: Some(estimated_fee),
            change_address: Some(source_address.address.clone()),
            utxo_selection_strategy: None,
        };
        
        // Create and sign transaction
        let transaction = self.transaction_builder.create_transaction(
            tx_params,
            utxos,
            user,
        ).await?;
        
        // Broadcast transaction
        let broadcast_result = self.transaction_builder.broadcast_transaction(&transaction).await;
        
        // Clear cache to refresh balances
        self.user_portfolios.remove(&user);
        self.utxo_manager.clear_cache(&source_address.address);
        
        match broadcast_result {
            Ok(_) => Ok(BitcoinSendResult {
                success: true,
                transaction_id: transaction.signatures.first().cloned(),
                from_address: source_address.address,
                to_address,
                amount_satoshis,
                fee_satoshis: estimated_fee,
                change_amount_satoshis: transaction.outputs
                    .iter()
                    .skip(1)
                    .map(|output| output.value)
                    .sum(),
                confirmation_time_estimate_minutes: 30,
                error_message: None,
            }),
            Err(e) => Ok(BitcoinSendResult {
                success: false,
                transaction_id: None,
                from_address: source_address.address,
                to_address,
                amount_satoshis,
                fee_satoshis: estimated_fee,
                change_amount_satoshis: 0,
                confirmation_time_estimate_minutes: 0,
                error_message: Some(e),
            })
        }
    }
    
    // Get Bitcoin address for user with specific type
    pub async fn get_bitcoin_address(
        &mut self,
        user: Principal,
        address_type: BitcoinAddressType,
    ) -> Result<BitcoinAddress, String> {
        match address_type {
            BitcoinAddressType::P2PKH => {
                self.address_manager.get_p2pkh_address(user).await
            },
            BitcoinAddressType::P2SH => {
                // For now, return P2PKH address for P2SH requests
                // In production, implement proper P2SH address generation
                self.address_manager.get_p2pkh_address(user).await
            },
            BitcoinAddressType::P2WPKH => {
                self.address_manager.get_p2wpkh_address(user).await
            },
            BitcoinAddressType::P2TR => {
                self.address_manager.get_p2tr_address(user).await
            },
        }
    }
    
    // Get all Bitcoin addresses for user
    pub async fn get_all_bitcoin_addresses(&mut self, user: Principal) -> Result<Vec<BitcoinAddress>, String> {
        self.address_manager.get_all_addresses(user).await
    }
    
    // Get UTXO statistics for user
    pub async fn get_utxo_statistics(&mut self, user: Principal) -> Result<Vec<UTXOStatsWithAddress>, String> {
        let portfolio = self.get_user_portfolio(user).await?;
        let mut stats = Vec::new();
        
        for address in portfolio.addresses {
            match self.utxo_manager.get_utxo_stats(address.address.clone()).await {
                Ok(utxo_stats) => {
                    stats.push(UTXOStatsWithAddress {
                        address: address.address,
                        address_type: address.address_type,
                        stats: utxo_stats,
                    });
                },
                Err(e) => {
                    ic_cdk::println!("Failed to get UTXO stats for {}: {}", address.address, e);
                }
            }
        }
        
        Ok(stats)
    }
    
    // Estimate transaction fee
    pub fn estimate_transaction_fee(
        &self,
        utxo_count: usize,
        output_count: usize,
        priority: FeePriority,
    ) -> BitcoinFeeEstimate {
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
        
        let total_fee = self.transaction_builder.estimate_fee(utxo_count, output_count, sat_per_byte);
        
        BitcoinFeeEstimate {
            sat_per_byte,
            priority,
            confirmation_blocks,
            total_fee_satoshis: total_fee,
        }
    }
    
    // Validate Bitcoin address
    pub fn validate_bitcoin_address(&self, address: &str) -> Result<BitcoinAddressType, String> {
        self.address_manager.validate_address(address)
    }
    
    // Get Bitcoin network info
    pub fn get_network_info(&self) -> BitcoinNetworkInfo {
        BitcoinNetworkInfo {
            network: match self.context.network {
                ic_cdk::api::management_canister::bitcoin::BitcoinNetwork::Mainnet => BitcoinNetwork::Mainnet,
                ic_cdk::api::management_canister::bitcoin::BitcoinNetwork::Testnet => BitcoinNetwork::Testnet,
                ic_cdk::api::management_canister::bitcoin::BitcoinNetwork::Regtest => BitcoinNetwork::Regtest,
            },
            key_name: self.context.key_name.clone(),
            supported_address_types: vec![
                BitcoinAddressType::P2PKH,
                BitcoinAddressType::P2WPKH,
                BitcoinAddressType::P2TR,
            ],
            chain_fusion_enabled: true,
        }
    }
    
    // Clear all caches
    pub fn clear_all_caches(&mut self) {
        self.user_portfolios.clear();
        self.utxo_manager.clear_all_cache();
    }
    
    // Health check
    pub async fn health_check(&mut self) -> BitcoinServiceHealth {
        let mut issues = Vec::new();
        let mut overall_healthy = true;
        
        // Test address generation
        let test_user = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
        match self.address_manager.get_p2pkh_address(test_user).await {
            Ok(_) => {},
            Err(e) => {
                issues.push(format!("Address generation failed: {}", e));
                overall_healthy = false;
            }
        }
        
        // Test network connectivity (would check actual Bitcoin network)
        // For now, assume healthy
        
        BitcoinServiceHealth {
            healthy: overall_healthy,
            issues,
            last_checked: ic_cdk::api::time(),
            network: self.get_network_info(),
        }
    }
}

// Result types
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinSendResult {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub from_address: String,
    pub to_address: String,
    pub amount_satoshis: u64,
    pub fee_satoshis: u64,
    pub change_amount_satoshis: u64,
    pub confirmation_time_estimate_minutes: u32,
    pub error_message: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UTXOStatsWithAddress {
    pub address: String,
    pub address_type: BitcoinAddressType,
    pub stats: crate::defi::bitcoin::utxo::UTXOStats,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinNetworkInfo {
    pub network: BitcoinNetwork,
    pub key_name: String,
    pub supported_address_types: Vec<BitcoinAddressType>,
    pub chain_fusion_enabled: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinServiceHealth {
    pub healthy: bool,
    pub issues: Vec<String>,
    pub last_checked: u64,
    pub network: BitcoinNetworkInfo,
}

// Public API functions
#[allow(dead_code)]
impl BitcoinDeFiService {
    // Batch operations for efficiency
    pub async fn batch_get_balances(&mut self, addresses: Vec<String>) -> Result<HashMap<String, u64>, String> {
        let mut balances = HashMap::new();
        
        for address in addresses {
            match self.utxo_manager.get_balance(address.clone()).await {
                Ok(balance) => {
                    balances.insert(address, balance);
                },
                Err(e) => {
                    ic_cdk::println!("Failed to get balance for {}: {}", address, e);
                    balances.insert(address, 0);
                }
            }
        }
        
        Ok(balances)
    }
    
    // Multi-address transaction (sweep multiple addresses)
    pub async fn sweep_to_address(
        &mut self,
        user: Principal,
        target_address: String,
        source_addresses: Vec<String>,
        fee_per_address: u64,
    ) -> Result<Vec<BitcoinSendResult>, String> {
        let mut results = Vec::new();
        
        for source_address in source_addresses {
            let balance = match self.utxo_manager.get_balance(source_address.clone()).await {
                Ok(balance) => balance,
                Err(e) => {
                    results.push(BitcoinSendResult {
                        success: false,
                        transaction_id: None,
                        from_address: source_address,
                        to_address: target_address.clone(),
                        amount_satoshis: 0,
                        fee_satoshis: fee_per_address,
                        change_amount_satoshis: 0,
                        confirmation_time_estimate_minutes: 0,
                        error_message: Some(e),
                    });
                    continue;
                }
            };
            
            if balance <= fee_per_address {
                continue; // Skip addresses with insufficient balance for fees
            }
            
            let send_amount = balance - fee_per_address;
            
            match self.send_bitcoin(
                user,
                target_address.clone(),
                send_amount,
                Some(fee_per_address),
                None, // Auto-detect address type
            ).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(BitcoinSendResult {
                        success: false,
                        transaction_id: None,
                        from_address: source_address,
                        to_address: target_address.clone(),
                        amount_satoshis: send_amount,
                        fee_satoshis: fee_per_address,
                        change_amount_satoshis: 0,
                        confirmation_time_estimate_minutes: 0,
                        error_message: Some(e),
                    });
                }
            }
        }
        
        Ok(results)
    }
}