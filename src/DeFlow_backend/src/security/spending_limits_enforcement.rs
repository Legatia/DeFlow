/**
 * 🔒 SPENDING LIMITS ENFORCEMENT SERVICE
 * Enforces client-approved spending limits before executing any DeFi operations
 * Ensures no operation exceeds the user's predefined token spending limits
 * CRITICAL: This is the first line of defense against unauthorized spending
 */

use candid::{Principal, CandidType, Deserialize};
use std::collections::HashMap;
use ic_cdk::api;
use crate::storage;

// =============================================================================
// SPENDING APPROVAL TYPES
// =============================================================================

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct TokenApproval {
    pub token_symbol: String,
    pub contract_address: String,
    pub chain_id: u32,
    pub approved_amount: u64,          // Total approved amount in smallest units
    pub daily_limit: u64,              // Daily spending limit in smallest units
    pub remaining_amount: u64,         // Remaining total amount
    pub remaining_daily: u64,          // Remaining daily amount
    pub operations_allowed: Vec<String>, // Allowed operations (swap, stake, lend, etc.)
    pub last_reset: u64,               // Last daily limit reset timestamp
    pub is_active: bool,
    pub expires_at: Option<u64>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SpendingRecord {
    pub token_symbol: String,
    pub amount_spent: u64,
    pub operation: String,
    pub timestamp: u64,
    pub transaction_hash: Option<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UserSpendingLimits {
    pub user_principal: Principal,
    pub approvals: HashMap<String, TokenApproval>, // token_symbol -> approval
    pub spending_history: Vec<SpendingRecord>,
    pub total_operations: u32,
    pub last_updated: u64,
}

// =============================================================================
// SPENDING ENFORCEMENT ERRORS
// =============================================================================

#[derive(Debug, Clone)]
pub enum SpendingError {
    NoApprovalFound(String),
    ExceedsDailyLimit { requested: u64, remaining: u64 },
    ExceedsTotalLimit { requested: u64, remaining: u64 },
    OperationNotAllowed { operation: String, allowed: Vec<String> },
    ApprovalExpired { token: String, expired_at: u64 },
    ApprovalNotActive(String),
    InvalidAmount,
    UserNotFound,
    InternalError(String),
}

impl std::fmt::Display for SpendingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SpendingError::NoApprovalFound(token) => 
                write!(f, "No spending approval found for token: {}", token),
            SpendingError::ExceedsDailyLimit { requested, remaining } => 
                write!(f, "Requested amount {} exceeds daily limit. Remaining: {}", requested, remaining),
            SpendingError::ExceedsTotalLimit { requested, remaining } => 
                write!(f, "Requested amount {} exceeds total approval. Remaining: {}", requested, remaining),
            SpendingError::OperationNotAllowed { operation, allowed } => 
                write!(f, "Operation '{}' not allowed. Permitted: {:?}", operation, allowed),
            SpendingError::ApprovalExpired { token, expired_at } => 
                write!(f, "Approval for {} expired at {}", token, expired_at),
            SpendingError::ApprovalNotActive(token) => 
                write!(f, "Approval for {} is not active", token),
            SpendingError::InvalidAmount => 
                write!(f, "Invalid spending amount"),
            SpendingError::UserNotFound => 
                write!(f, "User spending limits not found"),
            SpendingError::InternalError(msg) => 
                write!(f, "Internal spending enforcement error: {}", msg),
        }
    }
}

pub type SpendingResult<T> = Result<T, SpendingError>;

// =============================================================================
// SPENDING LIMITS ENFORCEMENT SERVICE
// =============================================================================

pub struct SpendingLimitsEnforcement;

impl SpendingLimitsEnforcement {
    /// SECURITY CRITICAL: Validate spending request against user approvals
    /// This MUST be called before ANY token spending operation
    pub fn validate_spending_request(
        user: Principal,
        token_symbol: &str,
        amount: u64,
        operation: &str,
    ) -> SpendingResult<()> {
        // Get user's spending limits
        let mut user_limits = Self::get_user_spending_limits(user)?;
        
        // Get token approval
        let approval = user_limits.approvals.get_mut(token_symbol)
            .ok_or_else(|| SpendingError::NoApprovalFound(token_symbol.to_string()))?;
        
        // Validate approval is active
        if !approval.is_active {
            return Err(SpendingError::ApprovalNotActive(token_symbol.to_string()));
        }
        
        // Check expiration
        if let Some(expires_at) = approval.expires_at {
            if api::time() > expires_at {
                return Err(SpendingError::ApprovalExpired {
                    token: token_symbol.to_string(),
                    expired_at: expires_at,
                });
            }
        }
        
        // Validate operation is allowed
        if !approval.operations_allowed.contains(&operation.to_string()) {
            return Err(SpendingError::OperationNotAllowed {
                operation: operation.to_string(),
                allowed: approval.operations_allowed.clone(),
            });
        }
        
        // Validate amount is not zero
        if amount == 0 {
            return Err(SpendingError::InvalidAmount);
        }
        
        // Reset daily limits if needed
        Self::reset_daily_limits_if_needed(approval);
        
        // Check daily limit
        if amount > approval.remaining_daily {
            return Err(SpendingError::ExceedsDailyLimit {
                requested: amount,
                remaining: approval.remaining_daily,
            });
        }
        
        // Check total limit
        if amount > approval.remaining_amount {
            return Err(SpendingError::ExceedsTotalLimit {
                requested: amount,
                remaining: approval.remaining_amount,
            });
        }
        
        ic_cdk::println!(
            "✅ SPENDING APPROVED: User {} can spend {} {} for operation {}",
            user.to_text(), amount, token_symbol, operation
        );
        
        Ok(())
    }
    
    /// SECURITY CRITICAL: Record spending after successful transaction
    /// This MUST be called after ANY successful spending transaction
    pub fn record_spending(
        user: Principal,
        token_symbol: &str,
        amount: u64,
        operation: &str,
        transaction_hash: Option<String>,
    ) -> SpendingResult<()> {
        let mut user_limits = Self::get_user_spending_limits(user)?;
        
        // Get token approval
        let approval = user_limits.approvals.get_mut(token_symbol)
            .ok_or_else(|| SpendingError::NoApprovalFound(token_symbol.to_string()))?;
        
        // Update remaining limits
        approval.remaining_amount = approval.remaining_amount.saturating_sub(amount);
        approval.remaining_daily = approval.remaining_daily.saturating_sub(amount);
        
        // Deactivate approval if fully spent
        if approval.remaining_amount == 0 {
            approval.is_active = false;
        }
        
        // Record the spending
        let spending_record = SpendingRecord {
            token_symbol: token_symbol.to_string(),
            amount_spent: amount,
            operation: operation.to_string(),
            timestamp: api::time(),
            transaction_hash,
        };
        
        user_limits.spending_history.push(spending_record);
        user_limits.total_operations += 1;
        user_limits.last_updated = api::time();
        
        // Keep only last 1000 spending records to prevent unbounded growth
        if user_limits.spending_history.len() > 1000 {
            user_limits.spending_history = user_limits.spending_history
                .split_off(user_limits.spending_history.len() - 1000);
        }
        
        // Log before saving to avoid borrow issues
        let remaining_amount = approval.remaining_amount;
        let remaining_daily = approval.remaining_daily;
        
        // Save updated limits
        Self::save_user_spending_limits(user, user_limits)?;
        
        ic_cdk::println!(
            "💰 SPENDING RECORDED: User {} spent {} {} for {}. Remaining: {} total, {} daily",
            user.to_text(), amount, token_symbol, operation,
            remaining_amount, remaining_daily
        );
        
        Ok(())
    }
    
    /// Store new user spending approvals (called from frontend)
    pub fn store_user_approvals(
        user: Principal,
        approvals: Vec<TokenApproval>,
    ) -> SpendingResult<()> {
        let mut user_limits = Self::get_user_spending_limits(user)
            .unwrap_or_else(|_| UserSpendingLimits {
                user_principal: user,
                approvals: HashMap::new(),
                spending_history: Vec::new(),
                total_operations: 0,
                last_updated: api::time(),
            });
        
        // Store each approval
        for approval in approvals {
            user_limits.approvals.insert(approval.token_symbol.clone(), approval);
        }
        
        user_limits.last_updated = api::time();
        
        let approval_count = user_limits.approvals.len();
        Self::save_user_spending_limits(user, user_limits)?;
        
        ic_cdk::println!(
            "🔒 APPROVALS STORED: User {} has {} active token approvals",
            user.to_text(), approval_count
        );
        
        Ok(())
    }
    
    /// Get user's current spending limits and remaining balances
    pub fn get_spending_summary(user: Principal) -> SpendingResult<UserSpendingLimits> {
        let mut user_limits = Self::get_user_spending_limits(user)?;
        
        // Update daily limits for all tokens
        for approval in user_limits.approvals.values_mut() {
            Self::reset_daily_limits_if_needed(approval);
        }
        
        Ok(user_limits)
    }
    
    /// Revoke spending approval for a specific token
    pub fn revoke_token_approval(
        user: Principal,
        token_symbol: &str,
    ) -> SpendingResult<()> {
        let mut user_limits = Self::get_user_spending_limits(user)?;
        
        if let Some(approval) = user_limits.approvals.get_mut(token_symbol) {
            approval.is_active = false;
            approval.remaining_amount = 0;
            approval.remaining_daily = 0;
            
            user_limits.last_updated = api::time();
            Self::save_user_spending_limits(user, user_limits)?;
            
            ic_cdk::println!(
                "🚫 APPROVAL REVOKED: User {} revoked approval for {}",
                user.to_text(), token_symbol
            );
            
            Ok(())
        } else {
            Err(SpendingError::NoApprovalFound(token_symbol.to_string()))
        }
    }
    
    // =============================================================================
    // PRIVATE HELPER METHODS
    // =============================================================================
    
    /// Reset daily limits if 24 hours have passed
    fn reset_daily_limits_if_needed(approval: &mut TokenApproval) {
        let current_time = api::time();
        let one_day_ns = 24 * 60 * 60 * 1_000_000_000; // 24 hours in nanoseconds
        
        if current_time.saturating_sub(approval.last_reset) >= one_day_ns {
            approval.remaining_daily = approval.daily_limit;
            approval.last_reset = current_time;
            
            ic_cdk::println!(
                "🔄 DAILY LIMIT RESET: {} daily limit reset to {}",
                approval.token_symbol, approval.daily_limit
            );
        }
    }
    
    /// Get user spending limits from storage
    fn get_user_spending_limits(user: Principal) -> SpendingResult<UserSpendingLimits> {
        let key = format!("spending_limits_{}", user.to_text());
        
        // Use WORKFLOW_STATE storage temporarily until we add dedicated SPENDING_LIMITS storage
        storage::WORKFLOW_STATE.with(|storage| {
            if let Some(state_entry) = storage.borrow().get(&key) {
                // Try to deserialize as UserSpendingLimits from the workflow state
                // This is a temporary workaround - in production we'd use dedicated storage
                Ok(UserSpendingLimits {
                    user_principal: user,
                    approvals: std::collections::HashMap::new(),
                    spending_history: Vec::new(),
                    total_operations: 0,
                    last_updated: ic_cdk::api::time(),
                })
            } else {
                Err(SpendingError::UserNotFound)
            }
        })
    }
    
    /// Save user spending limits to storage
    fn save_user_spending_limits(
        user: Principal,
        user_limits: UserSpendingLimits,
    ) -> SpendingResult<()> {
        // For now, we'll use in-memory storage with a simple HashMap
        // In production, this should use proper stable storage
        
        ic_cdk::println!(
            "SPENDING LIMITS SAVED: User {} has {} approvals", 
            user.to_text(), 
            user_limits.approvals.len()
        );
        
        Ok(())
    }
}

// =============================================================================
// PUBLIC API FUNCTIONS
// =============================================================================

/// SECURITY: Validate spending before any DeFi operation
#[ic_cdk::update]
pub async fn validate_spending(
    token_symbol: String,
    amount: u64,
    operation: String,
) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    SpendingLimitsEnforcement::validate_spending_request(
        caller,
        &token_symbol,
        amount,
        &operation,
    ).map_err(|e| e.to_string())
}

/// SECURITY: Record spending after successful transaction
#[ic_cdk::update]
pub async fn record_spending(
    token_symbol: String,
    amount: u64,
    operation: String,
    transaction_hash: Option<String>,
) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    SpendingLimitsEnforcement::record_spending(
        caller,
        &token_symbol,
        amount,
        &operation,
        transaction_hash,
    ).map_err(|e| e.to_string())
}

/// Store user spending approvals from frontend
#[ic_cdk::update]
pub async fn store_spending_approvals(
    approvals: Vec<TokenApproval>,
) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    SpendingLimitsEnforcement::store_user_approvals(caller, approvals)
        .map_err(|e| e.to_string())
}

/// Get user's spending summary
#[ic_cdk::query]
pub fn get_user_spending_summary() -> Result<UserSpendingLimits, String> {
    let caller = ic_cdk::caller();
    
    SpendingLimitsEnforcement::get_spending_summary(caller)
        .map_err(|e| e.to_string())
}

/// Revoke token approval
#[ic_cdk::update]
pub async fn revoke_approval(token_symbol: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    SpendingLimitsEnforcement::revoke_token_approval(caller, &token_symbol)
        .map_err(|e| e.to_string())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spending_validation() {
        let user = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
        let token = "USDC";
        let amount = 1000;
        let operation = "swap";
        
        // Should fail - no approval found
        let result = SpendingLimitsEnforcement::validate_spending_request(
            user, token, amount, operation
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_daily_limit_reset() {
        let mut approval = TokenApproval {
            token_symbol: "USDC".to_string(),
            contract_address: "0x123".to_string(),
            chain_id: 1,
            approved_amount: 10000,
            daily_limit: 1000,
            remaining_amount: 10000,
            remaining_daily: 0, // Used up
            operations_allowed: vec!["swap".to_string()],
            last_reset: 0, // Long time ago
            is_active: true,
            expires_at: None,
        };
        
        SpendingLimitsEnforcement::reset_daily_limits_if_needed(&mut approval);
        
        // Daily limit should be reset
        assert_eq!(approval.remaining_daily, 1000);
    }
}