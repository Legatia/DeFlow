// Strategy Authorization System
// Provides secure authorization for strategy execution using signatures and permissions

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;

use crate::defi::yield_farming::ChainId;
use super::automated_strategies::{StrategyError, ActiveStrategy};

/// Authorization system for strategy execution
#[derive(Debug, Clone)]
pub struct StrategyAuthorizationService {
    pub permissions: HashMap<String, UserPermissions>,
    pub execution_signatures: HashMap<String, ExecutionAuthorization>,
    pub session_tokens: HashMap<String, AuthSession>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct UserPermissions {
    pub user_id: Principal,
    pub max_daily_execution_amount: f64,
    pub allowed_chains: Vec<ChainId>,
    pub allowed_strategy_types: Vec<String>,
    pub requires_signature: bool,
    pub auto_approve_under_amount: Option<f64>,
    pub permissions_granted_at: u64,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ExecutionAuthorization {
    pub authorization_id: String,
    pub strategy_id: String,
    pub user_id: Principal,
    pub execution_amount: f64,
    pub authorized_chains: Vec<ChainId>,
    pub signature: Option<String>,
    pub authorization_type: AuthorizationType,
    pub created_at: u64,
    pub expires_at: u64,
    pub used: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AuthSession {
    pub session_id: String,
    pub user_id: Principal,
    pub created_at: u64,
    pub expires_at: u64,
    pub permissions: UserPermissions,
    pub active: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum AuthorizationType {
    SignatureRequired,
    AutoApproved,
    SessionBased,
    ThresholdECDSA,
}

impl StrategyAuthorizationService {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            execution_signatures: HashMap::new(),
            session_tokens: HashMap::new(),
        }
    }

    /// Set user permissions for strategy execution
    pub fn set_user_permissions(
        &mut self,
        user_id: Principal,
        permissions: UserPermissions,
    ) -> Result<(), StrategyError> {
        let key = user_id.to_string();
        self.permissions.insert(key, permissions);
        Ok(())
    }

    /// Check if user is authorized to execute a strategy
    pub fn is_authorized_to_execute(
        &self,
        user_id: Principal,
        strategy: &ActiveStrategy,
        execution_amount: f64,
    ) -> Result<AuthorizationCheck, StrategyError> {
        let key = user_id.to_string();
        
        // Get user permissions
        let permissions = self.permissions.get(&key)
            .ok_or_else(|| StrategyError::ValidationFailed("No permissions found for user".to_string()))?;

        // Check if permissions are expired
        if let Some(expires_at) = permissions.expires_at {
            if ic_cdk::api::time() > expires_at {
                return Ok(AuthorizationCheck {
                    authorized: false,
                    reason: Some("User permissions expired".to_string()),
                    authorization_required: AuthorizationType::SignatureRequired,
                });
            }
        }

        // Check daily execution limits
        let daily_spent = self.calculate_daily_spending(user_id);
        if daily_spent + execution_amount > permissions.max_daily_execution_amount {
            return Ok(AuthorizationCheck {
                authorized: false,
                reason: Some(format!("Daily limit exceeded: ${:.2} + ${:.2} > ${:.2}", 
                    daily_spent, execution_amount, permissions.max_daily_execution_amount)),
                authorization_required: AuthorizationType::SignatureRequired,
            });
        }

        // Check allowed chains
        for chain in &strategy.config.target_chains {
            if !permissions.allowed_chains.contains(chain) {
                return Ok(AuthorizationCheck {
                    authorized: false,
                    reason: Some(format!("Chain not authorized: {:?}", chain)),
                    authorization_required: AuthorizationType::SignatureRequired,
                });
            }
        }

        // Check strategy type authorization
        let strategy_type_str = format!("{:?}", strategy.config.strategy_type);
        if !permissions.allowed_strategy_types.is_empty() && 
           !permissions.allowed_strategy_types.contains(&strategy_type_str) {
            return Ok(AuthorizationCheck {
                authorized: false,
                reason: Some(format!("Strategy type not authorized: {}", strategy_type_str)),
                authorization_required: AuthorizationType::SignatureRequired,
            });
        }

        // Check if auto-approval applies
        if let Some(auto_approve_amount) = permissions.auto_approve_under_amount {
            if execution_amount <= auto_approve_amount {
                return Ok(AuthorizationCheck {
                    authorized: true,
                    reason: None,
                    authorization_required: AuthorizationType::AutoApproved,
                });
            }
        }

        // Check if signature is required
        if permissions.requires_signature {
            return Ok(AuthorizationCheck {
                authorized: false,
                reason: Some("Signature authorization required".to_string()),
                authorization_required: AuthorizationType::SignatureRequired,
            });
        }

        // Default approval
        Ok(AuthorizationCheck {
            authorized: true,
            reason: None,
            authorization_required: AuthorizationType::SessionBased,
        })
    }

    /// Create authorization challenge for strategy execution
    pub fn create_execution_authorization(
        &mut self,
        user_id: Principal,
        strategy: &ActiveStrategy,
        execution_amount: f64,
    ) -> Result<ExecutionAuthorization, StrategyError> {
        let authorization_id = format!("auth_{}_{:x}", user_id, ic_cdk::api::time());
        
        let authorization = ExecutionAuthorization {
            authorization_id: authorization_id.clone(),
            strategy_id: strategy.id.clone(),
            user_id,
            execution_amount,
            authorized_chains: strategy.config.target_chains.clone(),
            signature: None,
            authorization_type: AuthorizationType::SignatureRequired,
            created_at: ic_cdk::api::time(),
            expires_at: ic_cdk::api::time() + 300 * 1_000_000_000, // 5 minutes
            used: false,
        };

        self.execution_signatures.insert(authorization_id, authorization.clone());
        Ok(authorization)
    }

    /// Authorize strategy execution with signature
    pub fn authorize_with_signature(
        &mut self,
        authorization_id: &str,
        signature: String,
        message_hash: &str,
    ) -> Result<(), StrategyError> {
        // Get user_id first to avoid borrowing issues
        let user_id = {
            let authorization = self.execution_signatures.get(authorization_id)
                .ok_or_else(|| StrategyError::ValidationFailed("Authorization not found".to_string()))?;

            // Check if authorization has expired
            if ic_cdk::api::time() > authorization.expires_at {
                return Err(StrategyError::ValidationFailed("Authorization expired".to_string()));
            }

            // Check if authorization is already used
            if authorization.used {
                return Err(StrategyError::ValidationFailed("Authorization already used".to_string()));
            }

            authorization.user_id
        };
        
        // Verify signature (simplified - in production would verify against user's wallet)
        if self.verify_authorization_signature(&signature, message_hash, user_id)? {
            let authorization = self.execution_signatures.get_mut(authorization_id).unwrap();
            authorization.signature = Some(signature);
            authorization.authorization_type = AuthorizationType::SignatureRequired;
        } else {
            return Err(StrategyError::ValidationFailed("Invalid signature".to_string()));
        }

        Ok(())
    }

    /// Consume authorization for execution
    pub fn consume_authorization(
        &mut self,
        authorization_id: &str,
    ) -> Result<ExecutionAuthorization, StrategyError> {
        let mut authorization = self.execution_signatures.get(authorization_id)
            .ok_or_else(|| StrategyError::ValidationFailed("Authorization not found".to_string()))?
            .clone();

        // Check if authorization has expired
        if ic_cdk::api::time() > authorization.expires_at {
            self.execution_signatures.remove(authorization_id);
            return Err(StrategyError::ValidationFailed("Authorization expired".to_string()));
        }

        // Check if authorization is already used
        if authorization.used {
            return Err(StrategyError::ValidationFailed("Authorization already used".to_string()));
        }

        // Check if signature is provided when required
        if matches!(authorization.authorization_type, AuthorizationType::SignatureRequired) && authorization.signature.is_none() {
            return Err(StrategyError::ValidationFailed("Signature required but not provided".to_string()));
        }

        // Mark as used
        authorization.used = true;
        self.execution_signatures.insert(authorization_id.to_string(), authorization.clone());

        Ok(authorization)
    }

    /// Create auth session for multiple executions
    pub fn create_auth_session(
        &mut self,
        user_id: Principal,
        duration_seconds: u64,
    ) -> Result<AuthSession, StrategyError> {
        let session_id = format!("session_{}_{:x}", user_id, ic_cdk::api::time());
        
        let permissions = self.permissions.get(&user_id.to_string())
            .ok_or_else(|| StrategyError::ValidationFailed("No permissions found for user".to_string()))?
            .clone();

        let session = AuthSession {
            session_id: session_id.clone(),
            user_id,
            created_at: ic_cdk::api::time(),
            expires_at: ic_cdk::api::time() + duration_seconds * 1_000_000_000,
            permissions,
            active: true,
        };

        self.session_tokens.insert(session_id, session.clone());
        Ok(session)
    }

    /// Validate auth session
    pub fn validate_session(&self, session_id: &str) -> Result<&AuthSession, StrategyError> {
        let session = self.session_tokens.get(session_id)
            .ok_or_else(|| StrategyError::ValidationFailed("Session not found".to_string()))?;

        if !session.active {
            return Err(StrategyError::ValidationFailed("Session is inactive".to_string()));
        }

        if ic_cdk::api::time() > session.expires_at {
            return Err(StrategyError::ValidationFailed("Session expired".to_string()));
        }

        Ok(session)
    }

    /// Revoke auth session
    pub fn revoke_session(&mut self, session_id: &str) -> Result<(), StrategyError> {
        if let Some(session) = self.session_tokens.get_mut(session_id) {
            session.active = false;
        }
        Ok(())
    }

    /// Generate authorization message for signing
    pub fn generate_authorization_message(
        &self,
        strategy_id: &str,
        execution_amount: f64,
        chains: &[ChainId],
    ) -> String {
        format!(
            "DeFlow Strategy Execution Authorization\n\
            Strategy ID: {}\n\
            Execution Amount: ${:.2}\n\
            Authorized Chains: {:?}\n\
            Timestamp: {}\n\
            \n\
            By signing this message, you authorize the execution of the above strategy with the specified parameters.",
            strategy_id,
            execution_amount,
            chains,
            ic_cdk::api::time()
        )
    }

    /// Get pending authorizations for a user
    pub fn get_pending_authorizations(&self, user_id: Principal) -> Vec<&ExecutionAuthorization> {
        self.execution_signatures
            .values()
            .filter(|auth| auth.user_id == user_id && !auth.used && ic_cdk::api::time() <= auth.expires_at)
            .collect()
    }

    /// Cleanup expired authorizations and sessions
    pub fn cleanup_expired(&mut self) {
        let current_time = ic_cdk::api::time();

        // Remove expired authorizations
        self.execution_signatures.retain(|_, auth| auth.expires_at > current_time);

        // Remove expired sessions
        self.session_tokens.retain(|_, session| session.expires_at > current_time);
    }

    // Helper methods

    fn calculate_daily_spending(&self, user_id: Principal) -> f64 {
        let current_time = ic_cdk::api::time();
        let day_start = current_time - (current_time % (24 * 3600 * 1_000_000_000));

        self.execution_signatures
            .values()
            .filter(|auth| {
                auth.user_id == user_id && 
                auth.used && 
                auth.created_at >= day_start
            })
            .map(|auth| auth.execution_amount)
            .sum()
    }

    fn verify_authorization_signature(
        &self,
        signature: &str,
        message_hash: &str,
        _user_id: Principal,
    ) -> Result<bool, StrategyError> {
        // In a full implementation, this would:
        // 1. Recover the public key/address from the signature
        // 2. Verify it matches one of the user's validated wallet addresses
        // 3. Check that the message hash is correct
        
        // For now, basic validation
        if signature.len() >= 130 && message_hash.len() >= 64 {
            Ok(true) // Simplified validation
        } else {
            Ok(false)
        }
    }
}

/// Result of authorization check
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AuthorizationCheck {
    pub authorized: bool,
    pub reason: Option<String>,
    pub authorization_required: AuthorizationType,
}

impl Default for UserPermissions {
    fn default() -> Self {
        Self {
            user_id: Principal::anonymous(),
            max_daily_execution_amount: 10000.0, // $10,000 default
            allowed_chains: vec![
                ChainId::Ethereum,
                ChainId::Arbitrum,
                ChainId::Polygon,
            ],
            allowed_strategy_types: vec![], // Empty means all types allowed
            requires_signature: true,
            auto_approve_under_amount: Some(100.0), // Auto-approve under $100
            permissions_granted_at: 0,
            expires_at: None,
        }
    }
}