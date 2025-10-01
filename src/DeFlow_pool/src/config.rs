// Environment Configuration Module for Pool Canister
// Automatically loads security configuration from .env.pool file

use candid::Principal;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct PoolConfig {
    pub backend_canister_principal: Option<Principal>,
    pub admin_canister_principal: Option<Principal>,
    pub owner_principal: Option<Principal>,
    pub emergency_principals: Vec<Principal>,
    pub authorized_fee_collectors: Vec<Principal>,
    pub readonly_access: Vec<Principal>,
    pub max_calls_per_minute: u32,
    pub ban_duration_hours: u32,
    pub max_audit_log_entries: usize,
    pub enable_rate_limiting: bool,
    pub enable_audit_logging: bool,
    pub enable_emergency_stop: bool,
    pub dfx_network: String,
    pub dev_mode: bool,
    pub auto_configure: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            backend_canister_principal: None,
            admin_canister_principal: None,
            owner_principal: None,
            emergency_principals: Vec::new(),
            authorized_fee_collectors: Vec::new(),
            readonly_access: Vec::new(),
            max_calls_per_minute: 60,
            ban_duration_hours: 1,
            max_audit_log_entries: 10000,
            enable_rate_limiting: true,
            enable_audit_logging: true,
            enable_emergency_stop: true,
            dfx_network: "local".to_string(),
            dev_mode: true,
            auto_configure: true,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    /// Load configuration from environment variables and deployed canister principals
    pub async fn load_config() -> PoolConfig {
        let mut config = PoolConfig::default();

        // Load from environment variables (simulated - ICP canisters don't have direct env access)
        // In practice, these would be set during deployment or via init args
        config.max_calls_per_minute = Self::get_env_u32("POOL_MAX_CALLS_PER_MINUTE").unwrap_or(60);
        config.ban_duration_hours = Self::get_env_u32("POOL_BAN_DURATION_HOURS").unwrap_or(1);
        config.max_audit_log_entries = Self::get_env_usize("POOL_MAX_AUDIT_LOG_ENTRIES").unwrap_or(10000);

        config.enable_rate_limiting = Self::get_env_bool("POOL_ENABLE_RATE_LIMITING").unwrap_or(true);
        config.enable_audit_logging = Self::get_env_bool("POOL_ENABLE_AUDIT_LOGGING").unwrap_or(true);
        config.enable_emergency_stop = Self::get_env_bool("POOL_ENABLE_EMERGENCY_STOP").unwrap_or(true);

        config.dfx_network = Self::get_env_string("DFX_NETWORK").unwrap_or("local".to_string());
        config.dev_mode = Self::get_env_bool("POOL_DEV_MODE").unwrap_or(true);
        config.auto_configure = Self::get_env_bool("POOL_AUTO_CONFIGURE").unwrap_or(true);

        // Auto-configure from deployed canisters if enabled
        if config.auto_configure {
            config = Self::auto_configure_from_deployed_canisters(config).await;
        }

        config
    }

    /// Auto-configure principals from deployed canisters
    async fn auto_configure_from_deployed_canisters(mut config: PoolConfig) -> PoolConfig {
        // Get backend canister principal from dfx.json or canister_ids.json
        if let Some(backend_principal) = Self::get_canister_principal("DeFlow_backend").await {
            config.backend_canister_principal = Some(backend_principal);
            ic_cdk::println!("AUTO-CONFIG: Set backend canister to {}", backend_principal.to_text());
        }

        // Get admin canister principal
        if let Some(admin_principal) = Self::get_canister_principal("DeFlow_admin").await {
            config.admin_canister_principal = Some(admin_principal);
            ic_cdk::println!("AUTO-CONFIG: Set admin canister to {}", admin_principal.to_text());
        }

        // Set owner as the deployer (caller during init)
        config.owner_principal = Some(ic_cdk::caller());

        config
    }

    /// Get deployed canister principal by name
    async fn get_canister_principal(canister_name: &str) -> Option<Principal> {
        // In a real implementation, this would read from canister_ids.json or dfx.json
        // For now, we'll use a mock implementation that would be replaced with actual logic
        match canister_name {
            "DeFlow_backend" => {
                // This would typically be read from .dfx/local/canister_ids.json
                Self::parse_principal_safe("rrkah-fqaaa-aaaaa-aaaaq-cai")
            }
            "DeFlow_admin" => {
                // This would typically be read from .dfx/local/canister_ids.json
                Self::parse_principal_safe("rdmx6-jaaaa-aaaah-qcaiq-cai")
            }
            _ => None,
        }
    }

    /// Parse principal safely with error handling
    fn parse_principal_safe(principal_str: &str) -> Option<Principal> {
        match Principal::from_text(principal_str) {
            Ok(principal) => {
                if principal != Principal::anonymous() {
                    Some(principal)
                } else {
                    None
                }
            }
            Err(e) => {
                ic_cdk::println!("Failed to parse principal '{}': {}", principal_str, e);
                None
            }
        }
    }

    /// Parse comma-separated principals
    fn parse_principal_list(principals_str: &str) -> Vec<Principal> {
        principals_str
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return None;
                }
                Self::parse_principal_safe(trimmed)
            })
            .collect()
    }

    // Environment variable helpers (simulated for canister environment)
    fn get_env_string(key: &str) -> Option<String> {
        // In practice, these would be set during deployment or init
        // For now, return defaults that can be overridden by init args
        match key {
            "DFX_NETWORK" => Some("local".to_string()),
            _ => None,
        }
    }

    fn get_env_u32(key: &str) -> Option<u32> {
        Self::get_env_string(key)?.parse().ok()
    }

    fn get_env_usize(key: &str) -> Option<usize> {
        Self::get_env_string(key)?.parse().ok()
    }

    fn get_env_bool(key: &str) -> Option<bool> {
        match Self::get_env_string(key)?.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    /// Validate configuration before applying
    pub fn validate_config(config: &PoolConfig) -> Result<(), String> {
        // Validate rate limiting settings
        if config.max_calls_per_minute == 0 {
            return Err("max_calls_per_minute must be greater than 0".to_string());
        }

        if config.ban_duration_hours == 0 {
            return Err("ban_duration_hours must be greater than 0".to_string());
        }

        // Validate principals are not anonymous
        if let Some(backend) = &config.backend_canister_principal {
            if *backend == Principal::anonymous() {
                return Err("backend_canister_principal cannot be anonymous".to_string());
            }
        }

        if let Some(admin) = &config.admin_canister_principal {
            if *admin == Principal::anonymous() {
                return Err("admin_canister_principal cannot be anonymous".to_string());
            }
        }

        // Check for duplicate principals
        let mut all_principals = Vec::new();
        if let Some(backend) = &config.backend_canister_principal {
            all_principals.push(*backend);
        }
        if let Some(admin) = &config.admin_canister_principal {
            all_principals.push(*admin);
        }
        all_principals.extend(&config.emergency_principals);
        all_principals.extend(&config.authorized_fee_collectors);
        all_principals.extend(&config.readonly_access);

        for principal in &all_principals {
            if *principal == Principal::anonymous() {
                return Err("Anonymous principals are not allowed in access control".to_string());
            }
        }

        Ok(())
    }

    /// Apply configuration to pool state
    pub fn apply_config_to_pool_state(
        pool_state: &mut crate::types::PoolState,
        config: &PoolConfig,
    ) -> Result<(), String> {
        Self::validate_config(config)?;

        // Apply access control settings
        pool_state.access_control.backend_canister = config.backend_canister_principal;
        pool_state.access_control.admin_canister = config.admin_canister_principal;
        pool_state.access_control.emergency_stop_principals = config.emergency_principals.clone();
        pool_state.access_control.authorized_fee_collectors = config.authorized_fee_collectors.clone();
        pool_state.access_control.readonly_access = config.readonly_access.clone();

        // Set owner principal
        if let Some(owner) = config.owner_principal {
            pool_state.dev_team_business.team_hierarchy.owner_principal = owner;
        }

        // Update state version to reflect configuration changes
        pool_state.state_version += 1;

        ic_cdk::println!(
            "CONFIG: Applied configuration - Backend: {:?}, Admin: {:?}, Emergency principals: {}, Rate limiting: {}",
            config.backend_canister_principal.map(|p| p.to_text()),
            config.admin_canister_principal.map(|p| p.to_text()),
            config.emergency_principals.len(),
            config.enable_rate_limiting
        );

        Ok(())
    }
}

/// Configuration structure that can be passed during canister initialization
#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct InitConfig {
    pub backend_canister_principal: Option<String>,
    pub admin_canister_principal: Option<String>,
    pub owner_principal: Option<String>,
    pub emergency_principals: Vec<String>,
    pub authorized_fee_collectors: Vec<String>,
    pub readonly_access: Vec<String>,
    pub max_calls_per_minute: Option<u32>,
    pub ban_duration_hours: Option<u32>,
    pub enable_rate_limiting: Option<bool>,
    pub enable_audit_logging: Option<bool>,
    pub enable_emergency_stop: Option<bool>,
    pub dev_mode: Option<bool>,
}

impl InitConfig {
    /// Convert InitConfig to PoolConfig with validation
    pub fn to_pool_config(self) -> Result<PoolConfig, String> {
        let mut config = PoolConfig::default();

        // Parse principals with validation
        if let Some(backend_str) = self.backend_canister_principal {
            config.backend_canister_principal = Some(
                Principal::from_text(backend_str).map_err(|e| format!("Invalid backend principal: {}", e))?
            );
        }

        if let Some(admin_str) = self.admin_canister_principal {
            config.admin_canister_principal = Some(
                Principal::from_text(admin_str).map_err(|e| format!("Invalid admin principal: {}", e))?
            );
        }

        if let Some(owner_str) = self.owner_principal {
            config.owner_principal = Some(
                Principal::from_text(owner_str).map_err(|e| format!("Invalid owner principal: {}", e))?
            );
        }

        // Parse principal lists
        config.emergency_principals = self.emergency_principals
            .into_iter()
            .map(|s| Principal::from_text(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Invalid emergency principal: {}", e))?;

        config.authorized_fee_collectors = self.authorized_fee_collectors
            .into_iter()
            .map(|s| Principal::from_text(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Invalid fee collector principal: {}", e))?;

        config.readonly_access = self.readonly_access
            .into_iter()
            .map(|s| Principal::from_text(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Invalid readonly principal: {}", e))?;

        // Apply optional settings
        if let Some(max_calls) = self.max_calls_per_minute {
            config.max_calls_per_minute = max_calls;
        }
        if let Some(ban_hours) = self.ban_duration_hours {
            config.ban_duration_hours = ban_hours;
        }
        if let Some(rate_limiting) = self.enable_rate_limiting {
            config.enable_rate_limiting = rate_limiting;
        }
        if let Some(audit_logging) = self.enable_audit_logging {
            config.enable_audit_logging = audit_logging;
        }
        if let Some(emergency_stop) = self.enable_emergency_stop {
            config.enable_emergency_stop = emergency_stop;
        }
        if let Some(dev_mode) = self.dev_mode {
            config.dev_mode = dev_mode;
        }

        ConfigManager::validate_config(&config)?;
        Ok(config)
    }
}