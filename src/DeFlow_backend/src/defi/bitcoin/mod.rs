// Bitcoin DeFi Integration - Chain Fusion Implementation
// Day 8: Bitcoin integration with threshold ECDSA and Schnorr

pub mod service;
pub mod addresses;
pub mod utxo;
pub mod transactions;

use crate::defi::types::*;
use candid::{CandidType, Deserialize};
use serde::Serialize;
use ic_cdk::api::management_canister::bitcoin::{
    bitcoin_get_balance, bitcoin_get_utxos, bitcoin_send_transaction,
    BitcoinNetwork as ICPBitcoinNetwork, GetBalanceRequest, GetUtxosRequest, 
    SendTransactionRequest, Utxo
};
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument
};

// Re-export sub-modules
pub use service::BitcoinDeFiService;
pub use addresses::BitcoinAddressManager;
pub use utxo::UTXOManager;
// Transaction types available but not re-exported to avoid unused warnings

// Fee types are defined in this module, no need to re-export

// Bitcoin context for Chain Fusion
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BitcoinContext {
    pub network: ICPBitcoinNetwork,
    pub key_name: String,
}

#[allow(dead_code)]
impl BitcoinContext {
    pub fn new(network: BitcoinNetwork, key_name: String) -> Self {
        let icp_network = match network {
            BitcoinNetwork::Mainnet => ICPBitcoinNetwork::Mainnet,
            BitcoinNetwork::Testnet => ICPBitcoinNetwork::Testnet,
            BitcoinNetwork::Regtest => ICPBitcoinNetwork::Regtest,
        };
        
        Self {
            network: icp_network,
            key_name,
        }
    }
    
    pub fn ecdsa_key_id(&self) -> EcdsaKeyId {
        EcdsaKeyId {
            curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
            name: self.key_name.clone(),
        }
    }
}

// Bitcoin operation results
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinOperationResult {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub error_message: Option<String>,
    pub gas_fee_satoshis: Option<u64>,
    pub execution_time_ms: u64,
}

// Bitcoin fee estimation
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct BitcoinFeeEstimate {
    pub sat_per_byte: u64,
    pub priority: FeePriority,
    pub confirmation_blocks: u32,
    pub total_fee_satoshis: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum FeePriority {
    Low,     // ~60 minutes
    Medium,  // ~30 minutes  
    High,    // ~10 minutes
    Urgent,  // ~5 minutes
}

// Convert between our types and ICP Bitcoin API types
impl From<Utxo> for BitcoinUTXO {
    fn from(utxo: Utxo) -> Self {
        BitcoinUTXO {
            txid: hex::encode(&utxo.outpoint.txid),
            vout: utxo.outpoint.vout,
            value_satoshis: utxo.value,
            script_pubkey: hex::encode(&utxo.outpoint.txid), // Simplified
            confirmations: utxo.height,
        }
    }
}

// Bitcoin API wrapper functions
#[allow(dead_code)]
pub async fn get_bitcoin_balance(network: ICPBitcoinNetwork, address: String) -> Result<u64, String> {
    let request = GetBalanceRequest {
        address,
        network,
        min_confirmations: Some(1),
    };
    
    match bitcoin_get_balance(request).await {
        Ok((balance,)) => Ok(balance),
        Err((code, msg)) => Err(format!("Bitcoin balance error {}: {}", code as u8, msg)),
    }
}

#[allow(dead_code)]
pub async fn get_bitcoin_utxos(network: ICPBitcoinNetwork, address: String) -> Result<Vec<BitcoinUTXO>, String> {
    let request = GetUtxosRequest {
        address,
        network,
        filter: None,
    };
    
    match bitcoin_get_utxos(request).await {
        Ok((utxos_response,)) => {
            let utxos = utxos_response.utxos
                .into_iter()
                .map(BitcoinUTXO::from)
                .collect();
            Ok(utxos)
        },
        Err((code, msg)) => Err(format!("Bitcoin UTXOs error {}: {}", code as u8, msg)),
    }
}

#[allow(dead_code)]
pub async fn send_bitcoin_transaction(network: ICPBitcoinNetwork, transaction: Vec<u8>) -> Result<String, String> {
    let request = SendTransactionRequest {
        transaction,
        network,
    };
    
    match bitcoin_send_transaction(request).await {
        Ok(()) => Ok("Transaction sent successfully".to_string()),
        Err((code, msg)) => Err(format!("Bitcoin send error {}: {}", code as u8, msg)),
    }
}

// ECDSA signing for Bitcoin transactions
#[allow(dead_code)]
pub async fn get_bitcoin_public_key(key_name: String, derivation_path: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    let request = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
            name: key_name,
        },
    };
    
    match ecdsa_public_key(request).await {
        Ok((response,)) => Ok(response.public_key),
        Err((code, msg)) => Err(format!("ECDSA public key error {}: {}", code as u8, msg)),
    }
}

#[allow(dead_code)]
pub async fn sign_bitcoin_transaction(
    key_name: String, 
    derivation_path: Vec<Vec<u8>>, 
    message_hash: Vec<u8>
) -> Result<Vec<u8>, String> {
    let request = SignWithEcdsaArgument {
        message_hash,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
            name: key_name,
        },
    };
    
    match sign_with_ecdsa(request).await {
        Ok((response,)) => Ok(response.signature),
        Err((code, msg)) => Err(format!("ECDSA signing error {}: {}", code as u8, msg)),
    }
}