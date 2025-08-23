// Ethereum Address Generation with Threshold ECDSA
// Generates Ethereum addresses using ICP's threshold ECDSA

use super::{EvmChain, EthereumAddress, EthereumError};
use candid::Principal;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaKeyId, EcdsaCurve, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument, EcdsaPublicKeyResponse, SignWithEcdsaResponse,
};
use sha3::{Digest, Keccak256};

/// Ethereum address manager using threshold ECDSA
#[derive(Debug, Clone)]
pub struct EthereumAddressManager {
    key_name: String,
    chain: EvmChain,
}

impl EthereumAddressManager {
    /// Create a new Ethereum address manager
    pub fn new(key_name: String, chain: EvmChain) -> Self {
        Self { key_name, chain }
    }
    
    /// Generate Ethereum address for a user using threshold ECDSA
    pub async fn get_ethereum_address(&self, user: Principal) -> Result<EthereumAddress, EthereumError> {
        let derivation_path = Self::get_derivation_path(user);
        
        // Get public key from threshold ECDSA
        let public_key = get_ethereum_public_key(
            self.key_name.clone(),
            derivation_path.clone(),
        ).await.map_err(|e| EthereumError::ThresholdEcdsaError(e))?;
        
        // Convert public key to Ethereum address
        let address = self.public_key_to_ethereum_address(&public_key)?;
        
        Ok(EthereumAddress {
            address,
            chain: self.chain.clone(),
            derivation_path,
            balance_wei: "0".to_string(),
            balance_eth: 0.0,
            nonce: 0,
            last_updated: ic_cdk::api::time(),
        })
    }
    
    /// Sign transaction hash for Ethereum
    pub async fn sign_transaction_hash(
        &self,
        user: Principal,
        transaction_hash: &[u8],
    ) -> Result<Vec<u8>, EthereumError> {
        let derivation_path = Self::get_derivation_path(user);
        
        sign_ethereum_transaction(
            self.key_name.clone(),
            derivation_path,
            transaction_hash.to_vec(),
        ).await.map_err(|e| EthereumError::ThresholdEcdsaError(e))
    }
    
    /// Get derivation path for a user
    fn get_derivation_path(user: Principal) -> Vec<Vec<u8>> {
        vec![
            b"ethereum".to_vec(),
            user.as_slice().to_vec(),
        ]
    }
    
    /// Convert secp256k1 public key to Ethereum address
    fn public_key_to_ethereum_address(&self, public_key: &[u8]) -> Result<String, EthereumError> {
        if public_key.len() != 65 {
            return Err(EthereumError::ThresholdEcdsaError(
                ic_cdk::println!("Invalid public key length".to_string()
            ));
        }
        
        // Remove the 0x04 prefix for uncompressed public key
        let public_key_bytes = &public_key[1..];
        
        // Hash the public key with Keccak-256
        let mut hasher = Keccak256::new();
        hasher.update(public_key_bytes);
        let hash = hasher.finalize();
        
        // Take the last 20 bytes and format as hex address
        let address_bytes = &hash[12..];
        let address = format!("0x{}", hex::encode(address_bytes));
        
        // Validate the generated address
        if !super::utils::validate_ethereum_address(&address) {
            return Err(EthereumError::InvalidAddress(
                ic_cdk::println!("Generated invalid Ethereum address".to_string()
            ));
        }
        
        Ok(address)
    }
    
    /// Get multiple addresses for a user (for different purposes)
    pub async fn get_user_addresses(&self, user: Principal, count: u32) -> Result<Vec<EthereumAddress>, EthereumError> {
        let mut addresses = Vec::new();
        
        for i in 0..count {
            let mut derivation_path = Self::get_derivation_path(user);
            derivation_path.push(i.to_be_bytes().to_vec());
            
            let public_key = get_ethereum_public_key(
                self.key_name.clone(),
                derivation_path.clone(),
            ).await.map_err(|e| EthereumError::ThresholdEcdsaError(e))?;
            
            let address = self.public_key_to_ethereum_address(&public_key)?;
            
            addresses.push(EthereumAddress {
                address,
                chain: self.chain.clone(),
                derivation_path: format!("ethereum/{}/{}", hex::encode(user.as_slice()), i),
                balance_wei: "0".to_string(),
                balance_eth: 0.0,
                nonce: 0,
                last_updated: ic_cdk::api::time(),
            });
        }
        
        Ok(addresses)
    }
    
    /// Validate if an address belongs to this user
    pub async fn validate_user_address(&self, user: Principal, address: &str) -> Result<bool, EthereumError> {
        let user_address = self.get_ethereum_address(user).await?;
        Ok(user_address.address.to_lowercase() == address.to_lowercase())
    }
    
    /// Get address for contract deployment
    pub async fn get_contract_deployment_address(&self, user: Principal, nonce: u64) -> Result<String, EthereumError> {
        let user_address = self.get_ethereum_address(user).await?;
        
        // Calculate contract address using CREATE opcode formula
        // address = keccak256(rlp([sender, nonce]))[12:]
        let contract_address = self.calculate_contract_address(&user_address.address, nonce)?;
        
        Ok(contract_address)
    }
    
    /// Calculate contract address for CREATE opcode
    fn calculate_contract_address(&self, sender: &str, nonce: u64) -> Result<String, EthereumError> {
        use rlp::RlpStream;
        
        // Remove 0x prefix and convert to bytes
        let sender_bytes = hex::decode(&sender[2..])
            .map_err(|_| EthereumError::InvalidAddress(sender.to_string()))?;
        
        // RLP encode [sender, nonce]
        let mut stream = RlpStream::new_list(2);
        stream.append(&sender_bytes);
        stream.append(&nonce);
        
        let encoded = stream.out();
        
        // Hash with Keccak-256
        let mut hasher = Keccak256::new();
        hasher.update(&encoded);
        let hash = hasher.finalize();
        
        // Take last 20 bytes as address
        let address_bytes = &hash[12..];
        Ok(format!("0x{}", hex::encode(address_bytes)))
    }
}

/// Get Ethereum public key from threshold ECDSA
async fn get_ethereum_public_key(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let request = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    };
    
    let (response,): (EcdsaPublicKeyResponse,) = ecdsa_public_key(request)
        .await
        .map_err(|e| format!("Failed to get ECDSA public key: {:?}", e))?;
    
    Ok(response.public_key)
}

/// Sign Ethereum transaction with threshold ECDSA
async fn sign_ethereum_transaction(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    message_hash: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let request = SignWithEcdsaArgument {
        message_hash,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    };
    
    let (response,): (SignWithEcdsaResponse,) = sign_with_ecdsa(request)
        .await
        .map_err(|e| format!("Failed to sign with ECDSA: {:?}", e))?;
    
    Ok(response.signature)
}

/// Ethereum signature utilities
pub mod signature_utils {
    use super::*;
    use k256::{ecdsa::{RecoveryId, Signature, VerifyingKey}, elliptic_curve::sec1::ToEncodedPoint};
    
    /// Convert threshold ECDSA signature to Ethereum format (with recovery ID)
    pub fn format_ethereum_signature(signature: &[u8], message_hash: &[u8], address: &str) -> Result<Vec<u8>, EthereumError> {
        if signature.len() != 64 {
            return Err(EthereumError::SerializationError(
                ic_cdk::println!("Invalid signature length".to_string()
            ));
        }
        
        // Parse r and s from signature
        let r = &signature[0..32];
        let s = &signature[32..64];
        
        // Try recovery IDs 0 and 1 to find the correct one
        for recovery_id in 0..4 {
            if let Ok(recovered_address) = recover_address(message_hash, r, s, recovery_id) {
                if recovered_address.to_lowercase() == address.to_lowercase() {
                    // Return signature with recovery ID (v = 27 + recovery_id for Ethereum)
                    let mut eth_signature = Vec::with_capacity(65);
                    eth_signature.extend_from_slice(r);
                    eth_signature.extend_from_slice(s);
                    eth_signature.push(27 + recovery_id);
                    return Ok(eth_signature);
                }
            }
        }
        
        Err(EthereumError::SerializationError(
            ic_cdk::println!("Could not determine recovery ID".to_string()
        ))
    }
    
    /// Recover Ethereum address from signature
    fn recover_address(message_hash: &[u8], r: &[u8], s: &[u8], recovery_id: u8) -> Result<String, EthereumError> {
        use k256::ecdsa::{signature::Signature as _, Signature};
        
        // Create signature from r and s
        let mut signature_bytes = [0u8; 64];
        signature_bytes[..32].copy_from_slice(r);
        signature_bytes[32..].copy_from_slice(s);
        
        let signature = Signature::from_bytes(&signature_bytes.into())
            .map_err(|e| EthereumError::SerializationError(format!("Invalid signature: {}", e)))?;
        
        let recovery_id = RecoveryId::try_from(recovery_id)
            .map_err(|e| EthereumError::SerializationError(format!("Invalid recovery ID: {}", e)))?;
        
        // Recover the public key
        let recovered_key = VerifyingKey::recover_from_prehash(message_hash, &signature, recovery_id)
            .map_err(|e| EthereumError::SerializationError(format!("Key recovery failed: {}", e)))?;
        
        // Convert to uncompressed format
        let public_key_point = recovered_key.to_encoded_point(false);
        let public_key_bytes = public_key_point.as_bytes();
        
        // Convert to Ethereum address
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]); // Skip the 0x04 prefix
        let hash = hasher.finalize();
        
        let address_bytes = &hash[12..];
        Ok(format!("0x{}", hex::encode(address_bytes)))
    }
    
    /// Verify Ethereum signature
    pub fn verify_ethereum_signature(signature: &[u8], message_hash: &[u8], address: &str) -> Result<bool, EthereumError> {
        if signature.len() != 65 {
            return Ok(false);
        }
        
        let r = &signature[0..32];
        let s = &signature[32..64];
        let recovery_id = signature[64];
        
        // Adjust recovery ID (Ethereum uses 27/28, but we need 0/1)
        let recovery_id = if recovery_id >= 27 {
            recovery_id - 27
        } else {
            recovery_id
        };
        
        match recover_address(message_hash, r, s, recovery_id) {
            Ok(recovered_address) => Ok(recovered_address.to_lowercase() == address.to_lowercase()),
            Err(_) => Ok(false),
        }
    }
}

/// Address validation utilities
pub mod validation {
    use super::*;
    
    /// Validate Ethereum address checksum (EIP-55)
    pub fn validate_checksum_address(address: &str) -> bool {
        if !super::super::utils::validate_ethereum_address(address) {
            return false;
        }
        
        let address_lower = &address[2..].to_lowercase();
        let address_chars: Vec<char> = address[2..].chars().collect();
        
        // Hash the lowercase address
        let mut hasher = Keccak256::new();
        hasher.update(address_lower.as_bytes());
        let hash = hasher.finalize();
        
        // Check each character
        for (i, &ch) in address_chars.iter().enumerate() {
            if ch.is_ascii_alphabetic() {
                let hash_byte = hash[i / 2];
                let hash_nibble = if i % 2 == 0 {
                    hash_byte >> 4
                } else {
                    hash_byte & 0xf
                };
                
                let should_be_uppercase = hash_nibble >= 8;
                let is_uppercase = ch.is_ascii_uppercase();
                
                if should_be_uppercase != is_uppercase {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Convert address to checksum format (EIP-55)
    pub fn to_checksum_address(address: &str) -> Result<String, EthereumError> {
        if !super::super::utils::validate_ethereum_address(address) {
            return Err(EthereumError::InvalidAddress(address.to_string()));
        }
        
        let address_lower = &address[2..].to_lowercase();
        let mut hasher = Keccak256::new();
        hasher.update(address_lower.as_bytes());
        let hash = hasher.finalize();
        
        let mut checksum_address = String::from("0x");
        
        for (i, ch) in address_lower.chars().enumerate() {
            if ch.is_ascii_alphabetic() {
                let hash_byte = hash[i / 2];
                let hash_nibble = if i % 2 == 0 {
                    hash_byte >> 4
                } else {
                    hash_byte & 0xf
                };
                
                if hash_nibble >= 8 {
                    checksum_address.push(ch.to_ascii_uppercase());
                } else {
                    checksum_address.push(ch);
                }
            } else {
                checksum_address.push(ch);
            }
        }
        
        Ok(checksum_address)
    }
    
    /// Check if address is a contract (requires RPC call)
    pub async fn is_contract_address(address: &str, rpc_url: &str) -> Result<bool, EthereumError> {
        // This would require an RPC call to get code at address
        // For now, return false (assume all addresses are EOAs)
        // In a full implementation, this would call eth_getCode
        let _ = (address, rpc_url);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ethereum_address_validation() {
        assert!(super::super::utils::validate_ethereum_address("0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e"));
        assert!(!super::super::utils::validate_ethereum_address("0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3")); // Too short
        assert!(!super::super::utils::validate_ethereum_address("742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e")); // No 0x prefix
        assert!(!super::super::utils::validate_ethereum_address("0xGGGd35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e")); // Invalid hex
    }
    
    #[test]
    fn test_checksum_address() {
        let address = "0x742d35cc6634c0532925a3b8d0ff6e5fd8fe9a3e";
        let checksum = validation::to_checksum_address(address).unwrap();
        assert!(validation::validate_checksum_address(&checksum));
    }
}