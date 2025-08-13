// ICP Threshold ECDSA Integration for Ethereum
// Uses ICP's management canister for secure, decentralized key management

use candid::Principal;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument, EcdsaPublicKeyResponse, SignWithEcdsaResponse,
};
use super::{EvmChain, EthereumError};
use sha3::{Digest, Keccak256};

/// ICP Threshold ECDSA service for Ethereum integration
#[derive(Debug, Clone)]
pub struct ThresholdEcdsaService {
    /// ECDSA key identifier for signing
    pub key_id: EcdsaKeyId,
    /// Derivation path prefix for this canister
    pub derivation_path_prefix: Vec<Vec<u8>>,
}

impl ThresholdEcdsaService {
    /// Create a new threshold ECDSA service
    pub fn new(key_name: String, canister_id: Principal) -> Self {
        let key_id = EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        };

        // Use canister ID as part of derivation path for isolation
        let derivation_path_prefix = vec![canister_id.as_slice().to_vec()];

        Self {
            key_id,
            derivation_path_prefix,
        }
    }

    /// Generate Ethereum address for a user on a specific chain
    pub async fn get_ethereum_address(
        &self,
        user: Principal,
        chain: &EvmChain,
    ) -> Result<String, EthereumError> {
        let derivation_path = self.build_derivation_path(user, chain);
        let public_key = self.get_public_key(derivation_path).await?;
        self.public_key_to_ethereum_address(&public_key.public_key)
    }

    /// Sign an Ethereum transaction hash
    pub async fn sign_transaction_hash(
        &self,
        user: Principal,
        chain: &EvmChain,
        message_hash: &[u8],
    ) -> Result<Vec<u8>, EthereumError> {
        let derivation_path = self.build_derivation_path(user, chain);
        let signature_response = self.sign_message(derivation_path, message_hash).await?;
        
        // Convert ICP signature to Ethereum format
        self.format_ethereum_signature(&signature_response.signature, message_hash)
    }

    /// Get public key for a specific derivation path
    async fn get_public_key(&self, derivation_path: Vec<Vec<u8>>) -> Result<EcdsaPublicKeyResponse, EthereumError> {
        let request = EcdsaPublicKeyArgument {
            canister_id: None, // Use current canister
            derivation_path,
            key_id: self.key_id.clone(),
        };

        ecdsa_public_key(request)
            .await
            .map_err(|(code, msg)| {
                EthereumError::ThresholdEcdsaError(format!("Public key generation failed: {} - {}", code as u8, msg))
            })
            .map(|(response,)| response)
    }

    /// Sign a message with threshold ECDSA
    async fn sign_message(&self, derivation_path: Vec<Vec<u8>>, message_hash: &[u8]) -> Result<SignWithEcdsaResponse, EthereumError> {
        let request = SignWithEcdsaArgument {
            message_hash: message_hash.to_vec(),
            derivation_path,
            key_id: self.key_id.clone(),
        };

        sign_with_ecdsa(request)
            .await
            .map_err(|(code, msg)| {
                EthereumError::ThresholdEcdsaError(format!("Message signing failed: {} - {}", code as u8, msg))
            })
            .map(|(response,)| response)
    }

    /// Build derivation path for a user and chain
    fn build_derivation_path(&self, user: Principal, chain: &EvmChain) -> Vec<Vec<u8>> {
        let mut path = self.derivation_path_prefix.clone();
        
        // Add chain-specific component
        path.push(format!("ethereum-{}", chain.chain_id()).into_bytes());
        
        // Add user-specific component
        path.push(user.as_slice().to_vec());
        
        path
    }

    /// Convert secp256k1 public key to Ethereum address
    fn public_key_to_ethereum_address(&self, public_key_bytes: &[u8]) -> Result<String, EthereumError> {
        if public_key_bytes.len() != 65 {
            return Err(EthereumError::InvalidAddress("Invalid public key length".to_string()));
        }

        // Remove the 0x04 prefix for uncompressed public key
        let public_key = &public_key_bytes[1..];
        
        // Hash the public key with Keccak-256
        let mut hasher = Keccak256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        
        // Take the last 20 bytes and format as hex address
        let address_bytes = &hash[12..];
        let address = format!("0x{}", hex::encode(address_bytes));
        
        // Apply EIP-55 checksum
        self.to_checksum_address(&address)
    }

    /// Apply EIP-55 checksum to Ethereum address
    fn to_checksum_address(&self, address: &str) -> Result<String, EthereumError> {
        let address_lower = address.to_lowercase();
        let address_hash = {
            let mut hasher = Keccak256::new();
            hasher.update(&address_lower[2..].as_bytes());
            hasher.finalize()
        };

        let mut checksum_address = String::from("0x");
        
        for (i, ch) in address_lower[2..].chars().enumerate() {
            if ch.is_ascii_hexdigit() && ch.is_alphabetic() {
                let hash_byte = address_hash[i / 2];
                let nibble = if i % 2 == 0 { hash_byte >> 4 } else { hash_byte & 0xf };
                
                if nibble >= 8 {
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

    /// Convert ICP signature to Ethereum format
    fn format_ethereum_signature(&self, signature: &[u8], message_hash: &[u8]) -> Result<Vec<u8>, EthereumError> {
        if signature.len() != 64 {
            return Err(EthereumError::SerializationError("Invalid signature length".to_string()));
        }

        let r = &signature[0..32];
        let s = &signature[32..64];

        // Calculate recovery ID for Ethereum
        let recovery_id = self.calculate_recovery_id(r, s, message_hash)?;
        
        // Format as Ethereum signature: r + s + v
        let mut ethereum_signature = Vec::with_capacity(65);
        ethereum_signature.extend_from_slice(r);
        ethereum_signature.extend_from_slice(s);
        ethereum_signature.push(recovery_id);

        Ok(ethereum_signature)
    }

    /// Calculate recovery ID for Ethereum signature
    fn calculate_recovery_id(&self, r: &[u8], _s: &[u8], _message_hash: &[u8]) -> Result<u8, EthereumError> {
        // This is a simplified recovery ID calculation
        // In production, you would need to test both recovery IDs (0 and 1)
        // and see which one recovers to the correct public key
        
        // For now, we'll use a basic calculation
        // This should be replaced with proper recovery ID computation
        let r_value = u256_from_bytes(r);
        let recovery_id = if r_value % 2 == 0 { 0 } else { 1 };
        
        Ok(recovery_id + 27) // Ethereum uses 27/28 for legacy transactions
    }

    /// Verify Ethereum address format
    pub fn validate_ethereum_address(&self, address: &str) -> Result<(), EthereumError> {
        if !address.starts_with("0x") || address.len() != 42 {
            return Err(EthereumError::InvalidAddress("Invalid address format".to_string()));
        }

        // Check if all characters after 0x are valid hex
        for ch in address[2..].chars() {
            if !ch.is_ascii_hexdigit() {
                return Err(EthereumError::InvalidAddress("Invalid hex characters in address".to_string()));
            }
        }

        Ok(())
    }

    /// Get the canonical Ethereum address for a user (using Ethereum mainnet derivation)
    pub async fn get_canonical_ethereum_address(&self, user: Principal) -> Result<String, EthereumError> {
        self.get_ethereum_address(user, &EvmChain::Ethereum).await
    }

    /// Check if two addresses are the same (case-insensitive)
    pub fn addresses_equal(&self, addr1: &str, addr2: &str) -> bool {
        addr1.to_lowercase() == addr2.to_lowercase()
    }
}

/// Helper function to convert bytes to u256-like value for recovery ID calculation
fn u256_from_bytes(bytes: &[u8]) -> u64 {
    let mut result = 0u64;
    for (i, &byte) in bytes.iter().enumerate().take(8) {
        result |= (byte as u64) << (8 * (7 - i));
    }
    result
}

/// Ethereum-specific derivation paths and key management
pub mod derivation {
    use super::*;

    /// Standard derivation path components for Ethereum
    pub struct EthereumDerivation;

    impl EthereumDerivation {
        /// Create derivation path for Ethereum mainnet
        pub fn mainnet(user: Principal) -> Vec<Vec<u8>> {
            vec![
                b"ethereum".to_vec(),
                b"mainnet".to_vec(),
                user.as_slice().to_vec(),
            ]
        }

        /// Create derivation path for specific EVM chain
        pub fn chain(chain: &EvmChain, user: Principal) -> Vec<Vec<u8>> {
            vec![
                b"ethereum".to_vec(),
                format!("chain-{}", chain.chain_id()).into_bytes(),
                user.as_slice().to_vec(),
            ]
        }

        /// Create derivation path for specific application context
        pub fn application(app_name: &str, user: Principal) -> Vec<Vec<u8>> {
            vec![
                b"ethereum".to_vec(),
                app_name.as_bytes().to_vec(),
                user.as_slice().to_vec(),
            ]
        }
    }
}

/// Ethereum message signing utilities
pub mod signing {
    use super::*;

    /// Sign an Ethereum message with the "\x19Ethereum Signed Message:\n" prefix
    pub async fn sign_ethereum_message(
        ecdsa_service: &ThresholdEcdsaService,
        user: Principal,
        chain: &EvmChain,
        message: &str,
    ) -> Result<Vec<u8>, EthereumError> {
        let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
        let message_hash = {
            let mut hasher = Keccak256::new();
            hasher.update(prefixed_message.as_bytes());
            hasher.finalize()
        };

        ecdsa_service.sign_transaction_hash(user, chain, &message_hash).await
    }

    /// Create EIP-712 structured data hash
    pub fn create_eip712_hash(
        domain_separator: &[u8],
        struct_hash: &[u8],
    ) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(b"\x19\x01");
        hasher.update(domain_separator);
        hasher.update(struct_hash);
        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_ecdsa_service_creation() {
        let canister_id = Principal::anonymous();
        let service = ThresholdEcdsaService::new("test_key".to_string(), canister_id);
        
        assert_eq!(service.key_id.curve, EcdsaCurve::Secp256k1);
        assert_eq!(service.key_id.name, "test_key");
        assert!(!service.derivation_path_prefix.is_empty());
    }

    #[test]
    fn test_address_validation() {
        let canister_id = Principal::anonymous();
        let service = ThresholdEcdsaService::new("test_key".to_string(), canister_id);
        
        // Valid address
        assert!(service.validate_ethereum_address("0x742d35Cc6436C0532925a3b00A4d98f8A2b71D86").is_ok());
        
        // Invalid addresses
        assert!(service.validate_ethereum_address("0x742d35Cc6436C0532925a3b00A4d98f8A2b71D8").is_err()); // Too short
        assert!(service.validate_ethereum_address("742d35Cc6436C0532925a3b00A4d98f8A2b71D86").is_err()); // No 0x prefix
        assert!(service.validate_ethereum_address("0xZZZd35Cc6436C0532925a3b00A4d98f8A2b71D86").is_err()); // Invalid hex
    }

    #[test]
    fn test_addresses_equal() {
        let canister_id = Principal::anonymous();
        let service = ThresholdEcdsaService::new("test_key".to_string(), canister_id);
        
        let addr1 = "0x742d35Cc6436C0532925a3b00A4d98f8A2b71D86";
        let addr2 = "0x742d35cc6436c0532925a3b00a4d98f8a2b71d86";
        
        assert!(service.addresses_equal(addr1, addr2));
    }

    #[test]
    fn test_derivation_paths() {
        let user = Principal::anonymous();
        
        let mainnet_path = derivation::EthereumDerivation::mainnet(user);
        assert_eq!(mainnet_path.len(), 3);
        assert_eq!(mainnet_path[0], b"ethereum");
        assert_eq!(mainnet_path[1], b"mainnet");
        
        let chain_path = derivation::EthereumDerivation::chain(&EvmChain::Arbitrum, user);
        assert_eq!(chain_path.len(), 3);
        assert_eq!(chain_path[0], b"ethereum");
        assert_eq!(chain_path[1], b"chain-42161");
    }
}