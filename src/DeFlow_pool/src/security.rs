// Security Access Control Module for Pool Canister
// Implements strict access controls to prevent unauthorized access

use crate::types::*;
use candid::Principal;
use ic_cdk::api::time;
use std::collections::HashMap;

// Rate limiting constants
const MAX_CALLS_PER_MINUTE: u32 = 60;
const MAX_CALLS_PER_HOUR: u32 = 1000;
const RATE_LIMIT_WINDOW_NS: u64 = 60_000_000_000; // 1 minute in nanoseconds
const TEMPORARY_BAN_DURATION_NS: u64 = 3600_000_000_000; // 1 hour in nanoseconds
const MAX_AUDIT_LOG_ENTRIES: usize = 10000;

pub struct AccessControlManager;

impl AccessControlManager {
    /// Check if a principal has access to perform a specific security action
    pub fn check_access(
        pool_state: &mut PoolState,
        caller: Principal,
        action: SecurityAction,
        method_name: &str,
    ) -> Result<(), String> {
        // Always allow owner
        if caller == pool_state.dev_team_business.team_hierarchy.owner_principal {
            Self::log_security_event(pool_state, caller, action, method_name, true, None);
            return Ok(());
        }

        // Check if caller is temporarily banned
        Self::check_rate_limiting(pool_state, caller)?;

        // Check access based on action type
        let access_granted = match action {
            SecurityAction::DepositFee => {
                Self::check_backend_access(&pool_state.access_control, caller)
            }
            SecurityAction::AdminOperation => {
                Self::check_admin_access(&pool_state.access_control, caller)
            }
            SecurityAction::EmergencyStop => {
                Self::check_emergency_access(&pool_state.access_control, caller)
            }
            SecurityAction::ReadonlyQuery => {
                Self::check_readonly_access(&pool_state.access_control, caller)
            }
            SecurityAction::OwnerAction => {
                // Only owner can perform owner actions
                false // Already handled above
            }
        };

        if access_granted {
            // Update rate limiting
            Self::update_rate_limit(pool_state, caller);
            Self::log_security_event(pool_state, caller, action, method_name, true, None);
            Ok(())
        } else {
            let error_msg = format!(
                "Access denied for caller {} to perform {:?} action in method {}",
                caller.to_text(),
                action,
                method_name
            );
            Self::log_security_event(pool_state, caller, action, method_name, false, Some(error_msg.clone()));
            Err(error_msg)
        }
    }

    /// Owner-only function to set backend canister principal
    pub fn set_backend_canister(
        pool_state: &mut PoolState,
        caller: Principal,
        backend_principal: Principal,
    ) -> Result<(), String> {
        Self::ensure_owner(pool_state, caller)?;

        if backend_principal == Principal::anonymous() {
            return Err("Cannot set anonymous principal as backend canister".to_string());
        }

        pool_state.access_control.backend_canister = Some(backend_principal);
        pool_state.state_version += 1;

        Self::log_security_event(
            pool_state,
            caller,
            SecurityAction::OwnerAction,
            "set_backend_canister",
            true,
            Some(format!("Set backend canister to {}", backend_principal.to_text())),
        );

        ic_cdk::println!(
            "SECURITY: Backend canister set to {} by owner {}",
            backend_principal.to_text(),
            caller.to_text()
        );

        Ok(())
    }

    /// Owner-only function to set admin canister principal
    pub fn set_admin_canister(
        pool_state: &mut PoolState,
        caller: Principal,
        admin_principal: Principal,
    ) -> Result<(), String> {
        Self::ensure_owner(pool_state, caller)?;

        if admin_principal == Principal::anonymous() {
            return Err("Cannot set anonymous principal as admin canister".to_string());
        }

        pool_state.access_control.admin_canister = Some(admin_principal);
        pool_state.state_version += 1;

        Self::log_security_event(
            pool_state,
            caller,
            SecurityAction::OwnerAction,
            "set_admin_canister",
            true,
            Some(format!("Set admin canister to {}", admin_principal.to_text())),
        );

        ic_cdk::println!(
            "SECURITY: Admin canister set to {} by owner {}",
            admin_principal.to_text(),
            caller.to_text()
        );

        Ok(())
    }

    /// Add emergency stop principal (owner-only)
    pub fn add_emergency_stop_principal(
        pool_state: &mut PoolState,
        caller: Principal,
        emergency_principal: Principal,
    ) -> Result<(), String> {
        Self::ensure_owner(pool_state, caller)?;

        if emergency_principal == Principal::anonymous() {
            return Err("Cannot add anonymous principal as emergency stop principal".to_string());
        }

        if !pool_state.access_control.emergency_stop_principals.contains(&emergency_principal) {
            pool_state.access_control.emergency_stop_principals.push(emergency_principal);
            pool_state.state_version += 1;

            Self::log_security_event(
                pool_state,
                caller,
                SecurityAction::OwnerAction,
                "add_emergency_stop_principal",
                true,
                Some(format!("Added emergency principal {}", emergency_principal.to_text())),
            );
        }

        Ok(())
    }

    /// Get security status (public query for transparency)
    pub fn get_security_status(pool_state: &PoolState) -> SecurityStatus {
        SecurityStatus {
            backend_canister_set: pool_state.access_control.backend_canister.is_some(),
            admin_canister_set: pool_state.access_control.admin_canister.is_some(),
            emergency_stop_principals_count: pool_state.access_control.emergency_stop_principals.len() as u32,
            authorized_fee_collectors_count: pool_state.access_control.authorized_fee_collectors.len() as u32,
            readonly_access_count: pool_state.access_control.readonly_access.len() as u32,
            audit_log_entries: pool_state.security_audit_log.len() as u32,
            last_security_update: pool_state.state_version,
        }
    }

    // Private helper methods

    fn ensure_owner(pool_state: &PoolState, caller: Principal) -> Result<(), String> {
        if caller != pool_state.dev_team_business.team_hierarchy.owner_principal {
            return Err(format!(
                "Only owner {} can perform this action. Caller: {}",
                pool_state.dev_team_business.team_hierarchy.owner_principal.to_text(),
                caller.to_text()
            ));
        }
        Ok(())
    }

    fn check_backend_access(acl: &AccessControlList, caller: Principal) -> bool {
        if let Some(backend_canister) = acl.backend_canister {
            return caller == backend_canister;
        }

        // Fallback: Check if caller is in authorized fee collectors
        acl.authorized_fee_collectors.contains(&caller)
    }

    fn check_admin_access(acl: &AccessControlList, caller: Principal) -> bool {
        if let Some(admin_canister) = acl.admin_canister {
            return caller == admin_canister;
        }
        false
    }

    fn check_emergency_access(acl: &AccessControlList, caller: Principal) -> bool {
        acl.emergency_stop_principals.contains(&caller)
    }

    fn check_readonly_access(acl: &AccessControlList, caller: Principal) -> bool {
        // Readonly access is more permissive - includes admin and backend
        if Self::check_admin_access(acl, caller) || Self::check_backend_access(acl, caller) {
            return true;
        }
        acl.readonly_access.contains(&caller)
    }

    fn check_rate_limiting(pool_state: &mut PoolState, caller: Principal) -> Result<(), String> {
        let current_time = time();

        let rate_limit_state = pool_state.access_control.rate_limits
            .entry(caller)
            .or_insert_with(RateLimitState::default);

        // Check if temporarily banned
        if rate_limit_state.is_temporarily_banned && current_time < rate_limit_state.ban_until {
            return Err(format!(
                "Caller {} is temporarily banned until {}",
                caller.to_text(),
                rate_limit_state.ban_until
            ));
        }

        // Reset ban if expired
        if rate_limit_state.is_temporarily_banned && current_time >= rate_limit_state.ban_until {
            rate_limit_state.is_temporarily_banned = false;
            rate_limit_state.ban_until = 0;
            rate_limit_state.call_count_in_window = 0;
            rate_limit_state.window_start = current_time;
        }

        // Check rate limits
        if current_time - rate_limit_state.window_start >= RATE_LIMIT_WINDOW_NS {
            // Reset window
            rate_limit_state.call_count_in_window = 0;
            rate_limit_state.window_start = current_time;
        }

        if rate_limit_state.call_count_in_window >= MAX_CALLS_PER_MINUTE {
            // Temporarily ban the caller
            rate_limit_state.is_temporarily_banned = true;
            rate_limit_state.ban_until = current_time + TEMPORARY_BAN_DURATION_NS;

            return Err(format!(
                "Rate limit exceeded for caller {}. Temporarily banned for {} seconds",
                caller.to_text(),
                TEMPORARY_BAN_DURATION_NS / 1_000_000_000
            ));
        }

        Ok(())
    }

    fn update_rate_limit(pool_state: &mut PoolState, caller: Principal) {
        let current_time = time();

        let rate_limit_state = pool_state.access_control.rate_limits
            .entry(caller)
            .or_insert_with(RateLimitState::default);

        rate_limit_state.last_call_time = current_time;
        rate_limit_state.call_count_in_window += 1;
    }

    fn log_security_event(
        pool_state: &mut PoolState,
        caller: Principal,
        action: SecurityAction,
        method_name: &str,
        success: bool,
        additional_context: Option<String>,
    ) {
        let log_entry = SecurityAuditLog {
            timestamp: time(),
            caller,
            action,
            method_name: method_name.to_string(),
            success,
            failure_reason: if success { None } else { additional_context.clone() },
            additional_context,
        };

        pool_state.security_audit_log.push(log_entry);

        // Keep audit log bounded
        if pool_state.security_audit_log.len() > MAX_AUDIT_LOG_ENTRIES {
            pool_state.security_audit_log.drain(0..1000); // Remove oldest 1000 entries
        }
    }
}

#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct SecurityStatus {
    pub backend_canister_set: bool,
    pub admin_canister_set: bool,
    pub emergency_stop_principals_count: u32,
    pub authorized_fee_collectors_count: u32,
    pub readonly_access_count: u32,
    pub audit_log_entries: u32,
    pub last_security_update: u64,
}

/// Decorator macro for methods that require specific access levels
pub fn require_access(action: SecurityAction) -> impl Fn(&mut PoolState, &str) -> Result<(), String> {
    move |pool_state: &mut PoolState, method_name: &str| {
        let caller = ic_cdk::caller();
        AccessControlManager::check_access(pool_state, caller, action.clone(), method_name)
    }
}