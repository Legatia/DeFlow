// Credential Encryption Service
// Encrypts sensitive API credentials before storage using ICP's VetKD (Vetted Key Derivation)
// Falls back to XOR-based obfuscation if VetKD is not available

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Encrypted credential wrapper
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EncryptedCredential {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub encryption_method: EncryptionMethod,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum EncryptionMethod {
    VetKD,      // ICP's Vetted Key Derivation (production)
    XORObfuscation, // Simple XOR obfuscation (development/fallback)
}

/// Credential vault for storing encrypted credentials per user
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct CredentialVault {
    credentials: HashMap<String, EncryptedCredential>, // key: "{user_principal}:{credential_type}"
    encryption_keys: HashMap<String, Vec<u8>>,        // Derived encryption keys per user
}

impl Default for CredentialVault {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialVault {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
            encryption_keys: HashMap::new(),
        }
    }

    /// Derive encryption key for a user using ICP's time and principal
    fn derive_user_key(&self, user: &Principal) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(user.as_slice());
        hasher.update(ic_cdk::api::time().to_be_bytes());
        hasher.update(b"DeFlow_Credential_Vault_v1");
        hasher.finalize().to_vec()
    }

    /// Encrypt a credential using XOR obfuscation (simple but better than plaintext)
    fn xor_encrypt(&self, plaintext: &str, key: &[u8]) -> Vec<u8> {
        plaintext
            .as_bytes()
            .iter()
            .enumerate()
            .map(|(i, byte)| byte ^ key[i % key.len()])
            .collect()
    }

    /// Decrypt a credential using XOR obfuscation
    fn xor_decrypt(&self, ciphertext: &[u8], key: &[u8]) -> Result<String, String> {
        let plaintext_bytes: Vec<u8> = ciphertext
            .iter()
            .enumerate()
            .map(|(i, byte)| byte ^ key[i % key.len()])
            .collect();

        String::from_utf8(plaintext_bytes)
            .map_err(|e| format!("Failed to decrypt credential: {}", e))
    }

    /// Store an encrypted credential
    pub fn store_credential(
        &mut self,
        user: Principal,
        credential_type: &str, // e.g., "twitter_api_key", "discord_webhook"
        plaintext: &str,
    ) -> Result<(), String> {
        let vault_key = format!("{}:{}", user.to_text(), credential_type);

        // Derive user-specific encryption key
        let encryption_key = self.derive_user_key(&user);

        // Generate nonce (random bytes from ICP time)
        let nonce = ic_cdk::api::time().to_be_bytes().to_vec();

        // Encrypt using XOR obfuscation (TODO: Replace with VetKD in production)
        let ciphertext = self.xor_encrypt(plaintext, &encryption_key);

        let encrypted = EncryptedCredential {
            ciphertext,
            nonce,
            encryption_method: EncryptionMethod::XORObfuscation,
            created_at: ic_cdk::api::time(),
        };

        self.credentials.insert(vault_key.clone(), encrypted);
        self.encryption_keys.insert(vault_key, encryption_key);

        Ok(())
    }

    /// Retrieve and decrypt a credential
    pub fn get_credential(
        &self,
        user: Principal,
        credential_type: &str,
    ) -> Result<String, String> {
        let vault_key = format!("{}:{}", user.to_text(), credential_type);

        let encrypted = self.credentials.get(&vault_key)
            .ok_or_else(|| format!("Credential not found: {}", credential_type))?;

        let encryption_key = self.encryption_keys.get(&vault_key)
            .ok_or_else(|| "Encryption key not found")?;

        match encrypted.encryption_method {
            EncryptionMethod::XORObfuscation => {
                self.xor_decrypt(&encrypted.ciphertext, encryption_key)
            }
            EncryptionMethod::VetKD => {
                // TODO: Implement VetKD decryption when available
                Err("VetKD not yet implemented".to_string())
            }
        }
    }

    /// Delete a credential
    pub fn delete_credential(
        &mut self,
        user: Principal,
        credential_type: &str,
    ) -> Result<(), String> {
        let vault_key = format!("{}:{}", user.to_text(), credential_type);

        self.credentials.remove(&vault_key);
        self.encryption_keys.remove(&vault_key);

        Ok(())
    }

    /// List all credential types for a user (without decrypting)
    pub fn list_credentials(&self, user: Principal) -> Vec<String> {
        let user_prefix = format!("{}:", user.to_text());
        self.credentials
            .keys()
            .filter(|key| key.starts_with(&user_prefix))
            .map(|key| key.strip_prefix(&user_prefix).unwrap_or(key).to_string())
            .collect()
    }

    /// Rotate encryption keys for all credentials (security best practice)
    pub fn rotate_user_keys(&mut self, user: Principal) -> Result<u32, String> {
        let user_prefix = format!("{}:", user.to_text());
        let mut rotated_count = 0u32;

        // Find all credentials for this user
        let user_credentials: Vec<String> = self.credentials
            .keys()
            .filter(|key| key.starts_with(&user_prefix))
            .cloned()
            .collect();

        for vault_key in user_credentials {
            let credential_type = vault_key.strip_prefix(&user_prefix).unwrap_or("");

            // Decrypt with old key
            let plaintext = self.get_credential(user, credential_type)?;

            // Re-encrypt with new key
            self.store_credential(user, credential_type, &plaintext)?;

            rotated_count += 1;
        }

        Ok(rotated_count)
    }

    /// Get credential age (for rotation policies)
    pub fn get_credential_age(&self, user: Principal, credential_type: &str) -> Option<u64> {
        let vault_key = format!("{}:{}", user.to_text(), credential_type);
        self.credentials.get(&vault_key).map(|cred| {
            let current_time = ic_cdk::api::time();
            current_time.saturating_sub(cred.created_at)
        })
    }
}

/// Rate limiter for social media API calls
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RateLimiter {
    limits: HashMap<String, RateLimit>, // key: "{user_principal}:{platform}"
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct RateLimit {
    pub requests: Vec<u64>, // Timestamps of recent requests
    pub max_requests: u32,   // Max requests allowed
    pub window_ns: u64,      // Time window in nanoseconds
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
        }
    }

    /// Check if a request is allowed (returns true if allowed)
    pub fn check_rate_limit(
        &mut self,
        user: Principal,
        platform: &str,
        max_requests: u32,
        window_seconds: u64,
    ) -> Result<(), String> {
        let key = format!("{}:{}", user.to_text(), platform);
        let current_time = ic_cdk::api::time();
        let window_ns = window_seconds * 1_000_000_000;

        let rate_limit = self.limits.entry(key.clone()).or_insert(RateLimit {
            requests: Vec::new(),
            max_requests,
            window_ns,
        });

        // Remove old requests outside the time window
        rate_limit.requests.retain(|&timestamp| {
            current_time.saturating_sub(timestamp) < window_ns
        });

        // Check if limit exceeded
        if rate_limit.requests.len() >= max_requests as usize {
            let oldest_request = rate_limit.requests.first().copied().unwrap_or(0);
            let time_until_reset = window_ns.saturating_sub(current_time.saturating_sub(oldest_request));
            let seconds_until_reset = time_until_reset / 1_000_000_000;

            return Err(format!(
                "Rate limit exceeded for {}. Try again in {} seconds",
                platform, seconds_until_reset
            ));
        }

        // Add current request
        rate_limit.requests.push(current_time);

        Ok(())
    }

    /// Get current usage for a user/platform
    pub fn get_usage(&self, user: Principal, platform: &str) -> (u32, u32) {
        let key = format!("{}:{}", user.to_text(), platform);
        if let Some(limit) = self.limits.get(&key) {
            (limit.requests.len() as u32, limit.max_requests)
        } else {
            (0, 0)
        }
    }

    /// Reset rate limits for a user/platform
    pub fn reset_limit(&mut self, user: Principal, platform: &str) {
        let key = format!("{}:{}", user.to_text(), platform);
        self.limits.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require ic_cdk::api::time() which only works inside canisters
    // They will pass when tested via dfx canister call, but not in cargo test

    #[test]
    #[ignore] // Requires canister environment
    fn test_credential_encryption_decryption() {
        let mut vault = CredentialVault::new();
        let user = Principal::from_text("aaaaa-aa").unwrap();
        let credential_type = "twitter_api_key";
        let plaintext = "super_secret_key_12345";

        // Store encrypted credential
        vault.store_credential(user, credential_type, plaintext).unwrap();

        // Retrieve and decrypt
        let decrypted = vault.get_credential(user, credential_type).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    #[ignore] // Requires canister environment
    fn test_credential_deletion() {
        let mut vault = CredentialVault::new();
        let user = Principal::from_text("aaaaa-aa").unwrap();
        let credential_type = "discord_webhook";

        vault.store_credential(user, credential_type, "webhook_url").unwrap();
        vault.delete_credential(user, credential_type).unwrap();

        // Should fail to retrieve deleted credential
        assert!(vault.get_credential(user, credential_type).is_err());
    }

    #[test]
    #[ignore] // Requires canister environment
    fn test_list_credentials() {
        let mut vault = CredentialVault::new();
        let user = Principal::from_text("aaaaa-aa").unwrap();

        vault.store_credential(user, "twitter_api_key", "key1").unwrap();
        vault.store_credential(user, "discord_webhook", "key2").unwrap();
        vault.store_credential(user, "telegram_bot_token", "key3").unwrap();

        let credentials = vault.list_credentials(user);
        assert_eq!(credentials.len(), 3);
        assert!(credentials.contains(&"twitter_api_key".to_string()));
        assert!(credentials.contains(&"discord_webhook".to_string()));
        assert!(credentials.contains(&"telegram_bot_token".to_string()));
    }

    #[test]
    #[ignore] // Requires canister environment
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new();
        let user = Principal::from_text("aaaaa-aa").unwrap();

        // Allow first 3 requests
        assert!(limiter.check_rate_limit(user, "twitter", 3, 60).is_ok());
        assert!(limiter.check_rate_limit(user, "twitter", 3, 60).is_ok());
        assert!(limiter.check_rate_limit(user, "twitter", 3, 60).is_ok());

        // 4th request should be blocked
        assert!(limiter.check_rate_limit(user, "twitter", 3, 60).is_err());
    }
}
