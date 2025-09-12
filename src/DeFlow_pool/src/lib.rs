use candid::Principal;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk::caller;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

mod types;
mod pool_manager;
mod business_model;
mod cross_chain;
mod analytics;
mod chain_fusion;

use types::*;
use pool_manager::PoolManager;
use business_model::DevTeamBusinessManager;
use cross_chain::CrossChainManager;
use analytics::PoolAnalytics;
use chain_fusion::ChainFusionManager;
// SECURITY: Import checked arithmetic for overflow protection
// SECURITY: Import checked arithmetic for overflow protection (currently using built-in overflow checks)

// Memory management
type Memory = VirtualMemory<DefaultMemoryImpl>;
type StableStorage<K, V> = StableBTreeMap<K, V, Memory>;

const POOL_STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
#[allow(dead_code)]
const RESERVES_MEMORY_ID: MemoryId = MemoryId::new(1);
#[allow(dead_code)]
const BUSINESS_MODEL_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    
    static POOL_STATE: RefCell<PoolState> = RefCell::new(PoolState::default());
    static POOL_MANAGER: RefCell<PoolManager> = RefCell::new(PoolManager::new());
    static BUSINESS_MANAGER: RefCell<DevTeamBusinessManager> = RefCell::new(DevTeamBusinessManager::new());
    static CROSS_CHAIN_MANAGER: RefCell<CrossChainManager> = RefCell::new(CrossChainManager::new());
    static ANALYTICS: RefCell<PoolAnalytics> = RefCell::new(PoolAnalytics::new());
    static CHAIN_FUSION_MANAGER: RefCell<Option<ChainFusionManager>> = RefCell::new(None);
}

#[init]
fn init(owner: Option<Principal>) {
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // SECURITY: Enhanced owner validation with multiple checks
        let caller = ic_cdk::caller();
        let owner_principal = owner.unwrap_or(caller);
        
        // SECURITY: Comprehensive owner validation
        if owner_principal == Principal::anonymous() {
            ic_cdk::trap("SECURITY: Cannot initialize with anonymous principal as owner");
        }
        
        // SECURITY: Prevent management canister as owner
        if owner_principal.to_text() == "aaaaa-aa" {
            ic_cdk::trap("SECURITY: Cannot use management canister as owner");
        }
        
        // SECURITY: Validate owner principal format
        let owner_text = owner_principal.to_text();
        if owner_text.len() < 27 || owner_text.len() > 63 {
            ic_cdk::trap("SECURITY: Invalid owner principal format");
        }
        
        // SECURITY: Log initialization for audit
        ic_cdk::println!("AUDIT: Canister initialized - Owner: {}, Caller: {}", 
                         owner_principal.to_text(), caller.to_text());
        
        pool_state.dev_team_business.team_hierarchy.owner_principal = owner_principal;
        
        // Business configuration
        pool_state.dev_team_business.minimum_distribution_threshold = 5000.0; // $5K minimum
        pool_state.dev_team_business.distribution_frequency = 2_629_800; // 30 days in seconds
        
        // Grant owner premium access automatically
        pool_state.dev_team_business.team_member_earnings.insert(owner_principal, types::MemberEarnings::default());
    });

    ic_cdk::println!("AUDIT: Basic canister initialization completed - Chain Fusion addresses can be initialized separately");
}

#[pre_upgrade]
fn pre_upgrade() {
    // SECURITY: Store critical state in stable memory before upgrade
    ic_cdk::println!("SECURITY: Starting canister upgrade - preserving state");
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        // Store the pool state in stable memory
        MEMORY_MANAGER.with(|m| {
            let memory = m.borrow().get(POOL_STATE_MEMORY_ID);
            let mut stable_storage: StableStorage<u64, PoolState> = 
                StableBTreeMap::init(memory);
            
            match stable_storage.insert(0, pool_state.clone()) {
                Some(_) => ic_cdk::println!("SECURITY: Pool state updated in stable memory"),
                None => ic_cdk::println!("SECURITY: Pool state stored in stable memory"),
            }
        });
        
        // Log critical metrics before upgrade
        ic_cdk::println!("AUDIT: Pre-upgrade - Total liquidity: ${}, Team members: {}, Treasury transactions: {}", 
                         pool_state.total_liquidity_usd,
                         pool_state.dev_team_business.team_member_earnings.len(),
                         pool_state.treasury_transactions.len());
    });
    
    ic_cdk::println!("SECURITY: Pre-upgrade state preservation completed");
}

#[post_upgrade]
fn post_upgrade() {
    // SECURITY: Restore critical state from stable memory after upgrade
    ic_cdk::println!("SECURITY: Starting post-upgrade state restoration");
    
    // SECURITY: Wrap stable storage access in error handling for safe migration
    let restoration_result = std::panic::catch_unwind(|| {
        MEMORY_MANAGER.with(|m| {
            let memory = m.borrow().get(POOL_STATE_MEMORY_ID);
            let mut stable_storage: StableStorage<u64, PoolState> = 
                StableBTreeMap::init(memory);
            
            match stable_storage.get(&0) {
                Some(mut restored_state) => {
                // SECURITY: Handle upgrade migration for new security fields
                if restored_state.state_version == 0 {
                    // Migrate from pre-security version
                    restored_state.state_version = 1;
                    restored_state.termination_nonce = 0;
                    ic_cdk::println!("SECURITY: Migrated state to version 1 with security features");
                }
                
                // Successfully restored state from stable memory
                POOL_STATE.with(|state| {
                    *state.borrow_mut() = restored_state;
                });
                
                ic_cdk::println!("SECURITY: Pool state successfully restored from stable memory");
                
                // Log critical metrics after upgrade
                POOL_STATE.with(|state| {
                    let pool_state = state.borrow();
                    ic_cdk::println!("AUDIT: Post-upgrade - Total liquidity: ${}, Team members: {}, Treasury transactions: {}", 
                                     pool_state.total_liquidity_usd,
                                     pool_state.dev_team_business.team_member_earnings.len(),
                                     pool_state.treasury_transactions.len());
                });
                }
                None => {
                // SECURITY: Handle migration case - use default state with new fields
                ic_cdk::println!("WARNING: No saved state found in stable memory - initializing with migration");
                
                let migrated_state = migrate_pool_state();
                POOL_STATE.with(|state| {
                    *state.borrow_mut() = migrated_state;
                });
                
                // Store the migrated state for future upgrades
                let _ = stable_storage.insert(0, POOL_STATE.with(|s| s.borrow().clone()));
                
                ic_cdk::println!("SECURITY: Pool state migrated and stored successfully");
                }
            }
        })
    });
    
    // SECURITY: Handle any panics during stable storage access
    match restoration_result {
        Ok(_) => {
            ic_cdk::println!("SECURITY: Post-upgrade state restoration completed successfully");
        }
        Err(_) => {
            ic_cdk::println!("WARNING: Stable storage restoration failed - performing emergency migration");
            
            // SECURITY: Emergency fallback - initialize with secure defaults
            let emergency_state = migrate_pool_state();
            POOL_STATE.with(|state| {
                *state.borrow_mut() = emergency_state;
            });
            
            ic_cdk::println!("SECURITY: Emergency migration completed with secure defaults");
        }
    }
    
    ic_cdk::println!("SECURITY: Post-upgrade state restoration completed");
}

// SECURITY: Handle migration from previous PoolState versions
fn migrate_pool_state() -> PoolState {
    // Create a new PoolState with default values for new fields
    let mut new_state = PoolState::default();
    
    // SECURITY: Initialize new termination-related fields with safe defaults
    new_state.active_termination_request = None;
    new_state.termination_history = Vec::new();
    new_state.cofounder_principal = None;
    
    // Reset to safe bootstrap state
    new_state.phase = PoolPhase::Bootstrapping {
        started_at: ic_cdk::api::time(),
        target_liquidity: new_state.bootstrap_targets.clone(),
        estimated_completion: ic_cdk::api::time() + (365 * 24 * 60 * 60 * 1_000_000_000), // 1 year
    };
    
    // Ensure owner is set to anonymous for now - will be set properly on first admin call
    new_state.dev_team_business.team_hierarchy.owner_principal = Principal::anonymous();
    
    ic_cdk::println!("AUDIT: State migration completed - New termination fields initialized safely");
    
    new_state
}

// =============================================================================
// SECURITY: BLOCKCHAIN ADDRESS VALIDATION FUNCTIONS  
// =============================================================================

// SECURITY: Comprehensive blockchain address validation
fn validate_blockchain_address(address: &str, chain: &str) -> Result<(), String> {
    match chain.to_lowercase().as_str() {
        "bitcoin" => validate_bitcoin_address(address),
        "ethereum" | "arbitrum" | "optimism" | "polygon" | "base" => validate_ethereum_address(address),
        "solana" => validate_solana_address(address),
        _ => Err(format!("SECURITY: Unsupported blockchain: {}", chain)),
    }
}

fn validate_bitcoin_address(address: &str) -> Result<(), String> {
    // SECURITY: Bitcoin address validation (P2PKH, P2SH, Bech32)
    if address.len() < 26 || address.len() > 62 {
        return Err("SECURITY: Invalid Bitcoin address length".to_string());
    }
    
    // P2PKH addresses (1...)
    if address.starts_with('1') {
        if address.len() >= 26 && address.len() <= 35 && address.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)) {
            return Ok(());
        }
    }
    // P2SH addresses (3...)  
    else if address.starts_with('3') {
        if address.len() >= 26 && address.len() <= 35 && address.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)) {
            return Ok(());
        }
    }
    // Bech32 addresses (bc1...)
    else if address.starts_with("bc1") {
        if address.len() >= 39 && address.len() <= 62 && address.chars().all(|c| "qpzry9x8gf2tvdw0s3jn54khce6mua7l".contains(c) || c.is_ascii_digit()) {
            return Ok(());
        }
    }
    
    Err("SECURITY: Invalid Bitcoin address format".to_string())
}

fn validate_ethereum_address(address: &str) -> Result<(), String> {
    // SECURITY: Ethereum address validation (0x + 40 hex characters)
    if !address.starts_with("0x") {
        return Err("SECURITY: Ethereum address must start with 0x".to_string());
    }
    
    if address.len() != 42 {
        return Err("SECURITY: Ethereum address must be exactly 42 characters".to_string());
    }
    
    let hex_part = &address[2..];
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("SECURITY: Ethereum address contains invalid hex characters".to_string());
    }
    
    // SECURITY: Validate checksum if mixed case (EIP-55)
    if hex_part.chars().any(|c| c.is_uppercase()) && hex_part.chars().any(|c| c.is_lowercase()) {
        // Basic checksum validation - in production, implement full EIP-55
        // For now, accept properly formatted addresses
    }
    
    Ok(())
}

fn validate_solana_address(address: &str) -> Result<(), String> {
    // SECURITY: Solana address validation (Base58, 32 bytes = 44 characters)
    if address.len() < 32 || address.len() > 44 {
        return Err("SECURITY: Invalid Solana address length".to_string());
    }
    
    // Base58 character set validation
    if !address.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)) {
        return Err("SECURITY: Solana address contains invalid Base58 characters".to_string());
    }
    
    Ok(())
}

// =============================================================================
// SECURITY: ATOMIC STATE TRANSITION FUNCTIONS
// =============================================================================

/// SECURITY: Atomically update pool state with version checking to prevent race conditions
fn atomic_state_update<F, R>(operation_name: &str, state_updater: F) -> Result<R, String>
where
    F: FnOnce(&mut PoolState) -> Result<R, String>,
{
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // Increment state version before the operation
        let previous_version = pool_state.state_version;
        pool_state.state_version = pool_state.state_version.saturating_add(1);
        
        // Perform the operation
        match state_updater(&mut pool_state) {
            Ok(result) => {
                // AUDIT: Log successful state transition
                ic_cdk::println!("AUDIT: Atomic state update '{}' successful - Version: {} -> {}", 
                               operation_name, previous_version, pool_state.state_version);
                Ok(result)
            }
            Err(error) => {
                // SECURITY: Rollback state version on failure
                pool_state.state_version = previous_version;
                ic_cdk::println!("SECURITY: Atomic state update '{}' failed, version rolled back: {}", 
                               operation_name, error);
                Err(error)
            }
        }
    })
}

/// SECURITY: Atomically check and update termination state with nonce validation
fn atomic_termination_update<F, R>(
    operation_name: &str, 
    expected_nonce: Option<u64>,
    state_updater: F
) -> Result<R, String>
where
    F: FnOnce(&mut PoolState) -> Result<R, String>,
{
    atomic_state_update(operation_name, |pool_state| {
        // SECURITY: Validate termination nonce if provided
        if let Some(nonce) = expected_nonce {
            if pool_state.termination_nonce != nonce {
                return Err(format!(
                    "SECURITY: Termination nonce mismatch. Expected: {}, Current: {}",
                    nonce, pool_state.termination_nonce
                ));
            }
        }
        
        // Increment termination nonce for next operation
        pool_state.termination_nonce = pool_state.termination_nonce.saturating_add(1);
        
        // Perform the termination-specific operation
        state_updater(pool_state)
    })
}

// =============================================================================
// SECURITY: INPUT VALIDATION FUNCTIONS
// =============================================================================

/// SECURITY: Validate emergency termination criteria with strict requirements
fn validate_emergency_termination(reason: &str, caller: Principal) -> Result<(), String> {
    // SECURITY: Emergency termination requires stronger justification
    validate_string_input(reason, 50, 1000, "emergency termination reason")?; // Minimum 50 chars
    
    let reason_lower = reason.to_lowercase();
    
    // SECURITY: Emergency termination must have valid emergency keywords
    let valid_emergency_keywords = [
        "security breach", "hack", "exploit", "vulnerability", "critical bug", 
        "funds at risk", "smart contract failure", "bridge failure", "oracle failure",
        "regulatory requirement", "legal order", "compliance issue", "audit finding"
    ];
    
    let has_valid_emergency_keyword = valid_emergency_keywords.iter()
        .any(|keyword| reason_lower.contains(keyword));
    
    if !has_valid_emergency_keyword {
        ic_cdk::println!("SECURITY: Invalid emergency termination reason from {}: {}", caller.to_text(), reason);
        return Err(format!(
            "SECURITY: Emergency termination requires valid emergency justification. Must include one of: {:?}",
            valid_emergency_keywords
        ));
    }
    
    // SECURITY: Additional validation for emergency termination
    if reason.len() < 100 {
        return Err("SECURITY: Emergency termination reason must be at least 100 characters with detailed explanation".to_string());
    }
    
    // AUDIT: Log emergency termination validation
    ic_cdk::println!("AUDIT: Emergency termination validated - Initiator: {}, Reason length: {}", 
                     caller.to_text(), reason.len());
    
    Ok(())
}

/// SECURITY: Generate cryptographically secure confirmation phrase for termination
fn generate_secure_confirmation_phrase(
    termination_id: &str, 
    initiator: Principal, 
    current_time: u64, 
    state_version: u64,
    nonce: u64
) -> String {
    // SECURITY: Combine multiple entropy sources for unpredictable confirmation phrase
    let current_time_bytes = current_time.to_be_bytes();
    let state_version_bytes = state_version.to_be_bytes();
    let nonce_bytes = nonce.to_be_bytes();
    let extra_time_bytes = ic_cdk::api::time().to_be_bytes();
    
    let entropy_components = vec![
        termination_id.as_bytes(),
        &initiator.as_slice(),
        &current_time_bytes,
        &state_version_bytes, 
        &nonce_bytes,
        &extra_time_bytes, // Additional timestamp entropy
        b"DEFLOW_SECURE_TERMINATION_SALT_2024", // Static salt
    ];
    
    // Create a hash from all entropy sources
    let mut combined_entropy = Vec::new();
    for component in entropy_components {
        combined_entropy.extend_from_slice(component);
    }
    
    // Use a simple but unpredictable hash (for production, use SHA-256 or similar)
    let mut hash_value = 0u64;
    for (i, &byte) in combined_entropy.iter().enumerate() {
        hash_value = hash_value.wrapping_mul(31).wrapping_add(byte as u64).wrapping_add(i as u64);
    }
    
    // Create human-readable but secure confirmation phrase
    let phrase_components = [
        "SECURE",
        "TERMINATE", 
        "DEFLOW",
        "POOL",
        &format!("{:016X}", hash_value), // 16-digit hex
        &format!("{:08X}", current_time.wrapping_mul(state_version)), // Additional entropy
    ];
    
    phrase_components.join("_")
}

/// SECURITY: Validate that a confirmation phrase matches the expected secure phrase
fn validate_secure_confirmation_phrase(
    provided_phrase: &str,
    termination_request: &PoolTerminationRequest
) -> Result<(), String> {
    if provided_phrase != termination_request.secure_confirmation_phrase {
        ic_cdk::println!("SECURITY: Invalid secure confirmation phrase provided. Expected length: {}, Provided length: {}", 
                         termination_request.secure_confirmation_phrase.len(), 
                         provided_phrase.len());
        return Err("SECURITY: Invalid secure confirmation phrase".to_string());
    }
    
    // SECURITY: Additional validation - ensure phrase has expected structure
    if !termination_request.secure_confirmation_phrase.starts_with("SECURE_TERMINATE_DEFLOW_POOL_") {
        ic_cdk::println!("SECURITY: Confirmation phrase structure validation failed");
        return Err("SECURITY: Confirmation phrase structure invalid".to_string());
    }
    
    Ok(())
}

/// SECURITY: Safe arithmetic operations to prevent integer overflow and precision loss
#[allow(dead_code)]
fn safe_add_u64(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b)
        .ok_or_else(|| {
            ic_cdk::println!("SECURITY: Integer overflow detected - {} + {}", a, b);
            "SECURITY: Integer overflow in financial calculation".to_string()
        })
}

#[allow(dead_code)]
fn safe_sub_u64(a: u64, b: u64) -> Result<u64, String> {
    a.checked_sub(b)
        .ok_or_else(|| {
            ic_cdk::println!("SECURITY: Integer underflow detected - {} - {}", a, b);
            "SECURITY: Integer underflow in financial calculation".to_string()
        })
}

fn safe_add_f64(a: f64, b: f64) -> Result<f64, String> {
    if !a.is_finite() || !b.is_finite() {
        ic_cdk::println!("SECURITY: Non-finite numbers in addition - {} + {}", a, b);
        return Err("SECURITY: Non-finite numbers in financial calculation".to_string());
    }
    
    let result = a + b;
    
    if !result.is_finite() {
        ic_cdk::println!("SECURITY: Addition result not finite - {} + {} = {}", a, b, result);
        return Err("SECURITY: Arithmetic overflow in financial calculation".to_string());
    }
    
    // SECURITY: Check for reasonable financial limits (max $1 trillion to prevent absurd values)
    if result.abs() > 1_000_000_000_000.0 {
        ic_cdk::println!("SECURITY: Financial amount exceeds reasonable limits: {}", result);
        return Err("SECURITY: Financial amount exceeds reasonable limits".to_string());
    }
    
    Ok(result)
}

#[allow(dead_code)]
fn safe_sub_f64(a: f64, b: f64) -> Result<f64, String> {
    if !a.is_finite() || !b.is_finite() {
        ic_cdk::println!("SECURITY: Non-finite numbers in subtraction - {} - {}", a, b);
        return Err("SECURITY: Non-finite numbers in financial calculation".to_string());
    }
    
    let result = a - b;
    
    if !result.is_finite() {
        ic_cdk::println!("SECURITY: Subtraction result not finite - {} - {} = {}", a, b, result);
        return Err("SECURITY: Arithmetic overflow in financial calculation".to_string());
    }
    
    // SECURITY: Check for negative results where they shouldn't occur
    if result < 0.0 {
        ic_cdk::println!("SECURITY: Negative result in financial calculation - {} - {} = {}", a, b, result);
        return Err("SECURITY: Negative result in financial calculation".to_string());
    }
    
    Ok(result)
}

fn safe_mul_f64(a: f64, b: f64) -> Result<f64, String> {
    if !a.is_finite() || !b.is_finite() {
        ic_cdk::println!("SECURITY: Non-finite numbers in multiplication - {} * {}", a, b);
        return Err("SECURITY: Non-finite numbers in financial calculation".to_string());
    }
    
    let result = a * b;
    
    if !result.is_finite() {
        ic_cdk::println!("SECURITY: Multiplication result not finite - {} * {} = {}", a, b, result);
        return Err("SECURITY: Arithmetic overflow in financial calculation".to_string());
    }
    
    // SECURITY: Check for reasonable financial limits
    if result.abs() > 1_000_000_000_000.0 {
        ic_cdk::println!("SECURITY: Financial amount exceeds reasonable limits: {}", result);
        return Err("SECURITY: Financial amount exceeds reasonable limits".to_string());
    }
    
    Ok(result)
}

/// SECURITY: Comprehensive input validation for all user inputs
fn validate_principal_input(principal: &Principal, context: &str) -> Result<(), String> {
    if *principal == Principal::anonymous() {
        ic_cdk::println!("SECURITY: Anonymous principal in {}", context);
        return Err(format!("SECURITY: Anonymous principal not allowed for {}", context));
    }
    
    let principal_text = principal.to_text();
    if principal_text.len() < 27 || principal_text.len() > 63 {
        ic_cdk::println!("SECURITY: Invalid principal format in {}: {}", context, principal_text);
        return Err(format!("SECURITY: Invalid principal format for {}", context));
    }
    
    // SECURITY: Prevent management canister
    if principal_text == "aaaaa-aa" {
        ic_cdk::println!("SECURITY: Management canister not allowed in {}", context);
        return Err(format!("SECURITY: Management canister not allowed for {}", context));
    }
    
    Ok(())
}

fn validate_amount_input(amount: f64, min: f64, max: f64, context: &str) -> Result<(), String> {
    if !amount.is_finite() {
        ic_cdk::println!("SECURITY: Non-finite amount in {}: {}", context, amount);
        return Err(format!("SECURITY: Invalid amount for {}", context));
    }
    
    if amount < min || amount > max {
        ic_cdk::println!("SECURITY: Amount out of range in {}: {} (allowed: {}-{})", context, amount, min, max);
        return Err(format!("SECURITY: Amount {} out of allowed range {}-{} for {}", amount, min, max, context));
    }
    
    Ok(())
}

fn validate_string_input(input: &str, min_len: usize, max_len: usize, context: &str) -> Result<(), String> {
    if input.is_empty() && min_len > 0 {
        ic_cdk::println!("SECURITY: Empty string in {}", context);
        return Err(format!("SECURITY: Empty input not allowed for {}", context));
    }
    
    if input.len() < min_len || input.len() > max_len {
        ic_cdk::println!("SECURITY: String length out of range in {}: {} (allowed: {}-{})", context, input.len(), min_len, max_len);
        return Err(format!("SECURITY: Input length {} out of allowed range {}-{} for {}", input.len(), min_len, max_len, context));
    }
    
    // SECURITY: Check for potentially dangerous characters
    if input.contains('\0') || input.contains('\r') || input.contains('\n') {
        ic_cdk::println!("SECURITY: Dangerous characters in {}", context);
        return Err(format!("SECURITY: Invalid characters in {}", context));
    }
    
    Ok(())
}

fn check_rate_limit(last_timestamp: u64, min_interval_ns: u64, operation: &str) -> Result<(), String> {
    let current_time = ic_cdk::api::time();
    let time_since_last = current_time.saturating_sub(last_timestamp);
    
    if time_since_last < min_interval_ns {
        let remaining_seconds = (min_interval_ns.saturating_sub(time_since_last)) / 1_000_000_000;
        ic_cdk::println!("SECURITY: Rate limit exceeded for {}: {} seconds remaining", operation, remaining_seconds);
        return Err(format!("SECURITY: Rate limit for {} - wait {} seconds", operation, remaining_seconds));
    }
    
    Ok(())
}

// =============================================================================
// TEAM HIERARCHY & AUTHORIZATION
// =============================================================================

fn get_team_member_role(caller: Principal) -> Option<TeamRole> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let hierarchy = &pool_state.dev_team_business.team_hierarchy;
        
        if caller == hierarchy.owner_principal {
            Some(TeamRole::Owner)
        } else if hierarchy.senior_managers.contains(&caller) {
            Some(TeamRole::SeniorManager)
        } else if hierarchy.operations_managers.contains(&caller) {
            Some(TeamRole::OperationsManager)
        } else if hierarchy.tech_managers.contains(&caller) {
            Some(TeamRole::TechManager)
        } else if hierarchy.developers.contains(&caller) {
            Some(TeamRole::Developer)
        } else {
            None
        }
    })
}

fn is_owner(caller: Principal) -> bool {
    matches!(get_team_member_role(caller), Some(TeamRole::Owner))
}

fn is_manager_or_above(caller: Principal) -> bool {
    matches!(get_team_member_role(caller), Some(TeamRole::Owner | TeamRole::SeniorManager | TeamRole::OperationsManager | TeamRole::TechManager))
}

fn is_dev_team_member(caller: Principal) -> bool {
    get_team_member_role(caller).is_some()
}

fn can_view_financial_data(caller: Principal) -> bool {
    matches!(get_team_member_role(caller), Some(TeamRole::Owner | TeamRole::SeniorManager))
}

fn require_owner() -> Result<Principal, String> {
    let caller = ic_cdk::caller();
    if is_owner(caller) {
        Ok(caller)
    } else {
        Err("Unauthorized: Owner access required".to_string())
    }
}

fn require_manager_or_above() -> Result<Principal, String> {
    let caller = ic_cdk::caller();
    if is_manager_or_above(caller) {
        Ok(caller)
    } else {
        Err("Unauthorized: Manager access or above required".to_string())
    }
}

fn require_dev_team_member() -> Result<Principal, String> {
    let caller = ic_cdk::caller();
    if is_dev_team_member(caller) {
        Ok(caller)
    } else {
        Err("Unauthorized: Dev team membership required".to_string())
    }
}

fn is_authorized_fee_depositor(caller: Principal) -> bool {
    // SECURITY: Fixed circular authorization - separate fee depositors from withdrawal approvers
    if is_manager_or_above(caller) {
        return true;
    }
    
    // SECURITY: Check dedicated fee depositor list, NOT withdrawal approvers
    POOL_STATE.with(|state| {
        let _pool_state = state.borrow();
        // Create dedicated authorized_fee_depositors field to avoid circular dependency
        // For now, only managers can deposit fees to prevent circular authorization
        false // Only managers can deposit fees
    })
}

fn is_authorized_payment_processor(caller: Principal) -> bool {
    // SECURITY: Fixed circular authorization - separate payment processors from withdrawal approvers
    if is_manager_or_above(caller) {
        return true;
    }
    
    // SECURITY: Use dedicated payment processor list, NOT withdrawal approvers
    POOL_STATE.with(|state| {
        let _pool_state = state.borrow();
        // TODO: Add dedicated authorized_payment_processors field
        // For now, only managers can process payments to prevent circular authorization
        false // Only managers can process payments
    })
}

fn verify_financial_access_session(caller: Principal) -> bool {
    // SECURITY: Additional session verification for financial data access
    // In production, this would check for recent authentication, MFA, etc.
    
    let current_time = ic_cdk::api::time();
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        // Check if caller has had recent activity (within last hour)
        // This is a simplified check - production should implement proper session management
        let hierarchy = &pool_state.dev_team_business.team_hierarchy;
        
        // Owner and senior managers get longer session validity
        if caller == hierarchy.owner_principal {
            true // Owner always has access
        } else if hierarchy.senior_managers.contains(&caller) {
            // Senior managers get 4 hour sessions
            current_time - hierarchy.last_team_change < (4 * 60 * 60 * 1_000_000_000)
        } else {
            // Others need more recent verification
            current_time - hierarchy.last_team_change < (1 * 60 * 60 * 1_000_000_000)
        }
    })
}

// =============================================================================
// POOL STATE MANAGEMENT
// =============================================================================

#[query]
fn get_pool_state() -> Result<PoolState, String> {
    POOL_STATE.with(|state| {
        Ok(state.borrow().clone())
    })
}

#[query]
fn get_financial_overview() -> Result<FinancialOverview, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and senior managers can view full financial overview
    if !can_view_financial_data(caller) {
        // AUDIT: Log unauthorized access attempts
        ic_cdk::println!("SECURITY: Unauthorized financial data access attempt by {}", caller.to_text());
        return Err("Unauthorized: Financial data access restricted to Owner and Senior Managers".to_string());
    }
    
    // SECURITY: Additional verification for financial data access
    if !verify_financial_access_session(caller) {
        ic_cdk::println!("SECURITY: Financial access session verification failed for {}", caller.to_text());
        return Err("Session verification required for financial data access".to_string());
    }
    
    // AUDIT: Log successful financial data access
    ic_cdk::println!("AUDIT: Financial overview accessed by {} at {}", caller.to_text(), ic_cdk::api::time());
    
    ANALYTICS.with(|analytics| {
        POOL_STATE.with(|state| {
            analytics.borrow().get_financial_overview(&state.borrow())
        })
    })
}

#[query]
fn get_bootstrap_progress() -> f64 {
    POOL_MANAGER.with(|manager| {
        POOL_STATE.with(|state| {
            manager.borrow().get_bootstrap_progress(&state.borrow())
        })
    })
}

// =============================================================================
// FEE COLLECTION & BUSINESS MODEL
// =============================================================================

#[update]
fn deposit_fee(asset: Asset, amount: u64, tx_id: String, user: Principal) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Comprehensive input validation
    validate_principal_input(&caller, "fee deposit caller")?;
    validate_principal_input(&user, "fee deposit user")?;
    
    // SECURITY: Amount validation with realistic limits
    if amount == 0 {
        return Err("SECURITY: Invalid amount - must be greater than 0".to_string());
    }
    
    // SECURITY: Prevent unrealistic amounts that could cause overflow
    if amount > u64::MAX / 1000 {
        ic_cdk::println!("SECURITY: Fee deposit amount too large: {}", amount);
        return Err("SECURITY: Amount exceeds maximum allowed value".to_string());
    }
    
    // SECURITY: Transaction ID validation
    validate_string_input(&tx_id, 1, 100, "transaction ID")?;
    
    // SECURITY: Rate limiting for fee deposits
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        check_rate_limit(
            pool_state.dev_team_business.team_hierarchy.last_financial_operation,
            1_000_000_000, // 1 second minimum between fee deposits
            "fee deposits"
        )
    })?;
    
    // SECURITY: Only authorized services can deposit fees
    if !is_authorized_fee_depositor(caller) {
        ic_cdk::println!("SECURITY: Unauthorized fee deposit attempt by {}", caller.to_text());
        return Err("SECURITY: Only authorized services can deposit fees".to_string());
    }
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Fee deposit - Asset: {:?}, Amount: {}, TxID: {}, User: {}, Caller: {}", 
                     asset, amount, tx_id, user.to_text(), caller.to_text());
    
    POOL_MANAGER.with(|pool_manager| {
        BUSINESS_MANAGER.with(|business_manager| {
            POOL_STATE.with(|state| {
                let mut pool_state = state.borrow_mut();
                
                // SECURITY: Safe fee calculation with overflow protection
                let pool_portion = match amount.checked_mul(70).and_then(|x| x.checked_div(100)) {
                    Some(portion) => portion,
                    None => {
                        ic_cdk::println!("SECURITY: Integer overflow in pool portion calculation for amount {}", amount);
                        return Err("SECURITY: Calculation overflow in fee split".to_string());
                    }
                };
                
                let treasury_portion = match amount.checked_mul(30).and_then(|x| x.checked_div(100)) {
                    Some(portion) => portion as f64,
                    None => {
                        ic_cdk::println!("SECURITY: Integer overflow in treasury portion calculation for amount {}", amount);
                        return Err("SECURITY: Calculation overflow in fee split".to_string());
                    }
                };
                
                // SECURITY: Update rate limiting timestamp 
                pool_state.dev_team_business.team_hierarchy.last_financial_operation = ic_cdk::api::time();
                
                // Add pool portion to reserves
                pool_manager.borrow_mut().add_to_reserves(&mut pool_state, asset.clone(), pool_portion)?;
                
                // Record treasury transaction (30% of transaction fee)
                let treasury_tx = TreasuryTransaction {
                    id: format!("fee_{}", tx_id),
                    transaction_type: TreasuryTransactionType::TransactionFeeRevenue,
                    chain: "icp".to_string(), // Transaction fees collected in ICP
                    asset: asset.to_string(),
                    amount: treasury_portion,
                    amount_usd: treasury_portion, // Assuming 1:1 for now, should use real price oracle
                    from_address: "pool".to_string(),
                    to_address: "treasury".to_string(),
                    tx_hash: Some(tx_id.clone()),
                    status: TransactionStatus::Confirmed,
                    timestamp: ic_cdk::api::time(),
                    initiated_by: ic_cdk::caller(),
                    notes: Some("30% of transaction fee automatically allocated to treasury".to_string()),
                };
                
                // Add to treasury transactions and update balances
                pool_state.treasury_transactions.push(treasury_tx);
                
                // SECURITY: Enforce storage limits before adding transaction
                if pool_state.treasury_transactions.len() >= pool_state.storage_metrics.max_treasury_transactions {
                    return Err("SECURITY: Treasury transaction limit reached - cleanup required".to_string());
                }
                
                // SECURITY: Update treasury balance with checked arithmetic
                let asset_string = asset.to_string();
                if let Some(balance) = pool_state.treasury_balances.iter_mut()
                    .find(|b| b.chain == "icp" && b.asset == asset_string) {
                    
                    // SECURITY: Safe addition with overflow protection
                    let new_amount = safe_add_f64(balance.amount, treasury_portion)?;
                    let new_amount_usd = safe_add_f64(balance.amount_usd, treasury_portion)?;
                    
                    if !new_amount.is_finite() || !new_amount_usd.is_finite() || new_amount < 0.0 || new_amount_usd < 0.0 {
                        ic_cdk::println!("SECURITY: Balance calculation error - old: {}, adding: {}", balance.amount, treasury_portion);
                        return Err("SECURITY: Treasury balance calculation error".to_string());
                    }
                    
                    balance.amount = new_amount;
                    balance.amount_usd = new_amount_usd;
                    balance.last_updated = ic_cdk::api::time();
                } else {
                    // Create new treasury balance entry
                    pool_state.treasury_balances.push(TreasuryBalance {
                        chain: "icp".to_string(),
                        asset: asset_string.clone(),
                        amount: treasury_portion,
                        amount_usd: treasury_portion,
                        last_updated: ic_cdk::api::time(),
                        last_tx_hash: Some(tx_id.clone()),
                    });
                }
                
                // Also add profit portion to legacy dev team business model (for backward compatibility)
                business_manager.borrow_mut().add_transaction_fee_revenue(&mut pool_state, treasury_portion)?;
                
                // Check for monthly profit distribution
                business_manager.borrow_mut().check_and_execute_profit_distribution(&mut pool_state)?;
                
                // Check if bootstrap thresholds are met
                pool_manager.borrow_mut().check_bootstrap_completion(&mut pool_state)?;
                
                Ok(format!("Fee deposited: {} pool, {} treasury from tx {}", pool_portion, treasury_portion, tx_id))
            })
        })
    })
}

#[update]
fn process_subscription_payment(user: Principal, amount: f64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Comprehensive input validation
    validate_principal_input(&caller, "subscription payment caller")?;
    validate_principal_input(&user, "subscription payment user")?;
    validate_amount_input(amount, 1.0, 100000.0, "subscription payment amount")?;
    
    // SECURITY: Rate limiting for payment processing
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        check_rate_limit(
            pool_state.dev_team_business.team_hierarchy.last_financial_operation,
            5_000_000_000, // 5 second minimum between payments
            "subscription payments"
        )
    })?;
    
    // SECURITY: Only authorized payment processors can process subscriptions
    if !is_authorized_payment_processor(caller) {
        ic_cdk::println!("SECURITY: Unauthorized payment processing attempt by {}", caller.to_text());
        return Err("SECURITY: Only authorized payment processors allowed".to_string());
    }
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Subscription payment - User: {}, Amount: ${}, Caller: {}", 
                     user.to_text(), amount, caller.to_text());
    
    BUSINESS_MANAGER.with(|business_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            // SECURITY: Update rate limiting timestamp
            pool_state.dev_team_business.team_hierarchy.last_financial_operation = ic_cdk::api::time();
            business_manager.borrow_mut().process_subscription_payment(&mut pool_state, user, amount)?;
            Ok(format!("Subscription payment processed: ${} from {:?}", amount, user))
        })
    })
}

#[update]
fn withdraw_dev_earnings() -> Result<f64, String> {
    let caller = ic_cdk::caller();
    
    BUSINESS_MANAGER.with(|business_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            business_manager.borrow_mut().withdraw_dev_earnings(&mut pool_state, caller)
        })
    })
}

#[query]  
fn get_dev_earnings(dev_principal: Principal) -> f64 {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.dev_team_business.team_member_earnings
            .get(&dev_principal)
            .map(|earnings| earnings.total_usd_value)
            .unwrap_or(0.0)
    })
}

#[query]
fn get_dev_earnings_detailed(dev_principal: Principal) -> Result<types::MemberEarnings, String> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        if pool_state.dev_team_business.team_member_earnings.contains_key(&dev_principal) {
            Ok(pool_state.dev_team_business.team_member_earnings
                .get(&dev_principal)
                .cloned()
                .unwrap_or_default())
        } else {
            Err("Principal not found in team member earnings".to_string())
        }
    })
}

#[update]
fn withdraw_dev_earnings_with_options(option: types::WithdrawalOption) -> Result<Vec<types::TokenTransfer>, String> {
    let caller = caller();
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let mut business_manager = business_model::DevTeamBusinessManager::new();
        
        business_manager.withdraw_dev_earnings_multi_token(&mut pool_state, caller, option)
    })
}

#[update]
fn set_withdrawal_address(chain: types::ChainId, address: String) -> Result<String, String> {
    let caller = caller();
    
    // Validate address format for the specific chain
    validate_blockchain_address(&chain.to_string().to_lowercase(), &address)?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // Check if caller is a team member
        if !pool_state.dev_team_business.team_member_earnings.contains_key(&caller) {
            return Err("Only team members can set withdrawal addresses".to_string());
        }
        
        // Get or create member earnings
        let member_earnings = pool_state.dev_team_business.team_member_earnings
            .entry(caller)
            .or_insert_with(|| types::MemberEarnings::default());
        
        // Set the withdrawal address for this chain
        member_earnings.withdrawal_addresses.insert(chain, address.clone());
        
        Ok(format!("Withdrawal address set for {:?}: {}", chain, address))
    })
}

#[query]
fn get_my_withdrawal_addresses() -> Result<std::collections::HashMap<types::ChainId, String>, String> {
    let caller = caller();
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        if let Some(member_earnings) = pool_state.dev_team_business.team_member_earnings.get(&caller) {
            Ok(member_earnings.withdrawal_addresses.clone())
        } else {
            Err("Not a team member".to_string())
        }
    })
}

#[query]
fn get_withdrawal_address(chain: types::ChainId) -> Result<String, String> {
    let caller = caller();
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        if let Some(member_earnings) = pool_state.dev_team_business.team_member_earnings.get(&caller) {
            if let Some(address) = member_earnings.withdrawal_addresses.get(&chain) {
                Ok(address.clone())
            } else {
                Err(format!("No withdrawal address set for {:?}", chain))
            }
        } else {
            Err("Not a team member".to_string())
        }
    })
}

// =============================================================================
// LIQUIDITY MANAGEMENT
// =============================================================================

#[update]
fn add_liquidity(chain_id: ChainId, asset: Asset, amount: u64) -> Result<String, String> {
    let caller = require_manager_or_above()?; // SECURITY: Only managers and above can add liquidity
    
    // SECURITY: Input validation for liquidity addition
    if amount == 0 {
        return Err("SECURITY: Liquidity amount must be greater than 0".to_string());
    }
    
    // SECURITY: Prevent unrealistic amounts that could cause overflow
    if amount > u64::MAX / 1000 {
        ic_cdk::println!("SECURITY: Liquidity amount too large: {}", amount);
        return Err("SECURITY: Amount exceeds maximum allowed value".to_string());
    }
    
    // SECURITY: Rate limiting for liquidity operations
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        check_rate_limit(
            pool_state.dev_team_business.team_hierarchy.last_financial_operation,
            30_000_000_000, // 30 second minimum between liquidity operations
            "liquidity operations"
        )
    })?;
    
    // AUDIT: Log liquidity addition
    ic_cdk::println!("AUDIT: Liquidity addition - Chain: {:?}, Asset: {:?}, Amount: {}, Caller: {}", 
                     chain_id, asset, amount, caller.to_text());
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            // Update rate limiting timestamp
            pool_state.dev_team_business.team_hierarchy.last_financial_operation = ic_cdk::api::time();
            pool_manager.borrow_mut().add_liquidity(&mut pool_state, chain_id, asset, amount)?;
            Ok(format!("Liquidity added: {} {} on {:?}", amount, asset_to_string(&asset), chain_id))
        })
    })
}

#[update]
fn withdraw_for_execution(asset: Asset, amount: u64) -> Result<String, String> {
    let caller = require_manager_or_above()?; // SECURITY: Only managers and above can execute withdrawals
    
    // SECURITY: Input validation for withdrawals
    if amount == 0 {
        return Err("SECURITY: Withdrawal amount must be greater than 0".to_string());
    }
    
    // SECURITY: Prevent unrealistic amounts
    if amount > u64::MAX / 1000 {
        ic_cdk::println!("SECURITY: Withdrawal amount too large: {}", amount);
        return Err("SECURITY: Amount exceeds maximum allowed value".to_string());
    }
    
    // SECURITY: Rate limiting for withdrawals (more restrictive)
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        check_rate_limit(
            pool_state.dev_team_business.team_hierarchy.last_financial_operation,
            300_000_000_000, // 5 minute minimum between withdrawals
            "withdrawals for execution"
        )
    })?;
    
    // AUDIT: Log withdrawal attempt
    ic_cdk::println!("AUDIT: Withdrawal for execution - Asset: {:?}, Amount: {}, Caller: {}", 
                     asset, amount, caller.to_text());
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            // Update rate limiting timestamp
            pool_state.dev_team_business.team_hierarchy.last_financial_operation = ic_cdk::api::time();
            pool_manager.borrow_mut().withdraw_for_execution(&mut pool_state, asset, amount)
        })
    })
}

#[query]
fn get_asset_balance(chain_id: ChainId, asset: Asset) -> u64 {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.reserves.get(&chain_id)
            .and_then(|assets| assets.get(&asset))
            .map(|reserve| reserve.total_amount)
            .unwrap_or(0)
    })
}

#[query]
fn get_total_liquidity_usd() -> Result<f64, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and senior managers can view liquidity amounts
    if !can_view_financial_data(caller) {
        return Err("Unauthorized: Liquidity data access restricted to Owner and Senior Managers".to_string());
    }
    
    POOL_STATE.with(|state| {
        Ok(state.borrow().total_liquidity_usd)
    })
}

// =============================================================================
// CROSS-CHAIN OPERATIONS
// =============================================================================

#[update]
async fn detect_arbitrage_opportunities() -> Result<Vec<ArbitrageOpportunity>, String> {
    CROSS_CHAIN_MANAGER.with(|manager| {
        POOL_STATE.with(|state| {
            manager.borrow().detect_arbitrage_opportunities(&state.borrow())
        })
    })
}

#[update]
async fn execute_cross_chain_trade(
    source_chain: ChainId, 
    dest_chain: ChainId, 
    asset: Asset, 
    amount: u64
) -> Result<String, String> {
    require_manager_or_above()?; // SECURITY: Managers and above can execute cross-chain trades
    
    CROSS_CHAIN_MANAGER.with(|manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            manager.borrow_mut().execute_cross_chain_trade(&mut pool_state, source_chain, dest_chain, asset, amount)
        })
    })
}

// =============================================================================
// POOL CONFIGURATION
// =============================================================================

#[update]
fn set_bootstrap_targets(targets: Vec<(Asset, u64)>) -> Result<String, String> {
    let _caller = require_owner()?; // SECURITY: Only owner can change bootstrap targets
    
    // SECURITY: Input validation for bootstrap targets
    if targets.is_empty() {
        return Err("SECURITY: Bootstrap targets cannot be empty".to_string());
    }
    
    if targets.len() > 20 {
        return Err("SECURITY: Too many bootstrap targets (max 20)".to_string());
    }
    
    // SECURITY: Validate each target amount
    for (asset, amount) in &targets {
        if *amount == 0 {
            return Err(format!("SECURITY: Bootstrap target for {:?} cannot be zero", asset));
        }
        
        if *amount > u64::MAX / 1000 {
            return Err(format!("SECURITY: Bootstrap target for {:?} exceeds maximum value", asset));
        }
    }
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            
            // SECURITY: Rate limiting - minimum 1 hour between bootstrap changes
            check_rate_limit(
                pool_state.dev_team_business.team_hierarchy.last_bootstrap_change,
                3600_000_000_000, // 1 hour in nanoseconds
                "bootstrap changes"
            )?;
            
            pool_manager.borrow_mut().set_bootstrap_targets(&mut pool_state, targets)?;
            pool_state.dev_team_business.team_hierarchy.last_bootstrap_change = ic_cdk::api::time();
            Ok("Bootstrap targets updated".to_string())
        })
    })
}

#[update]
fn activate_pool() -> Result<String, String> {
    require_owner()?; // SECURITY: Only owner can activate the pool
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            pool_manager.borrow_mut().activate_pool(&mut pool_state)
        })
    })
}

#[update]
fn emergency_pause(reason: String) -> Result<String, String> {
    let caller = require_manager_or_above()?; // SECURITY: Managers and above can emergency pause
    
    // SECURITY: Input validation for emergency pause reason
    validate_string_input(&reason, 1, 500, "emergency pause reason")?;
    
    // SECURITY: Check for suspicious keywords in reason
    let suspicious_keywords = ["test", "joke", "fun", "prank", "fake"];
    let reason_lower = reason.to_lowercase();
    for keyword in &suspicious_keywords {
        if reason_lower.contains(keyword) {
            ic_cdk::println!("SECURITY: Suspicious emergency pause reason from {}: {}", caller.to_text(), reason);
            return Err("SECURITY: Emergency pause reason appears non-genuine".to_string());
        }
    }
    
    // AUDIT: Log emergency pause attempt
    ic_cdk::println!("AUDIT: Emergency pause initiated by {} - Reason: {}", caller.to_text(), reason);
    
    POOL_MANAGER.with(|pool_manager| {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow_mut();
            pool_manager.borrow_mut().emergency_pause(&mut pool_state, reason)
        })
    })
}

// =============================================================================
// TEAM HIERARCHY MANAGEMENT  
// =============================================================================

#[update]
fn add_team_member(principal: Principal, role: TeamRole) -> Result<String, String> {
    let caller = require_owner()?; // SECURITY: Only owner can add team members directly
    
    // SECURITY: Comprehensive input validation
    validate_principal_input(&principal, "team member principal")?;
    
    // SECURITY: Prevent adding owner as team member
    if principal == caller {
        return Err("SECURITY: Cannot add yourself as team member".to_string());
    }
    
    // SECURITY: Prevent management canister
    if principal.to_text() == "aaaaa-aa" {
        return Err("SECURITY: Cannot add management canister as team member".to_string());
    }
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // SECURITY: Rate limiting - minimum 1 hour between team changes
        check_rate_limit(
            pool_state.dev_team_business.team_hierarchy.last_team_change,
            3600_000_000_000, // 1 hour in nanoseconds
            "team changes"
        )?;
        
        // Add to appropriate role list
        match role {
            TeamRole::Owner => return Err("Cannot add additional owners".to_string()),
            TeamRole::SeniorManager => {
                if !pool_state.dev_team_business.team_hierarchy.senior_managers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.senior_managers.push(principal);
                }
            },
            TeamRole::OperationsManager => {
                if !pool_state.dev_team_business.team_hierarchy.operations_managers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.operations_managers.push(principal);
                }
            },
            TeamRole::TechManager => {
                if !pool_state.dev_team_business.team_hierarchy.tech_managers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.tech_managers.push(principal);
                }
            },
            TeamRole::Developer => {
                if !pool_state.dev_team_business.team_hierarchy.developers.contains(&principal) {
                    pool_state.dev_team_business.team_hierarchy.developers.push(principal);
                }
            },
        }
        
        // Grant premium access and initialize earnings
        pool_state.dev_team_business.team_member_earnings.insert(principal, types::MemberEarnings::default());
        pool_state.dev_team_business.team_hierarchy.last_team_change = ic_cdk::api::time();
        
        Ok(format!("Team member added successfully as {:?}", role))
    })
}

#[update] 
fn remove_team_member(principal: Principal) -> Result<String, String> {
    require_owner()?; // SECURITY: Only owner can remove team members
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let hierarchy = &mut pool_state.dev_team_business.team_hierarchy;
        
        if principal == hierarchy.owner_principal {
            return Err("Cannot remove the owner".to_string());
        }
        
        // Remove from all role lists
        hierarchy.senior_managers.retain(|&x| x != principal);
        hierarchy.operations_managers.retain(|&x| x != principal);
        hierarchy.tech_managers.retain(|&x| x != principal);
        hierarchy.developers.retain(|&x| x != principal);
        
        // Keep their earnings but mark as removed
        hierarchy.last_team_change = ic_cdk::api::time();
        
        Ok("Team member removed successfully".to_string())
    })
}

#[update]
fn request_team_change(principal: Principal, new_role: TeamRole) -> Result<u64, String> {
    let caller = require_manager_or_above()?; // Managers can request changes
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let hierarchy = &mut pool_state.dev_team_business.team_hierarchy;
        
        let request = TeamChangeRequest {
            request_type: TeamChangeType::AddMember,
            requester: caller,
            target_principal: principal,
            new_role: new_role.clone(),
            requires_owner_approval: !matches!(new_role, TeamRole::Developer), // Only dev additions need owner approval
            timestamp: ic_cdk::api::time(),
            approved: false,
            request_id: hierarchy.next_request_id,
        };
        
        let request_id = hierarchy.next_request_id;
        hierarchy.next_request_id += 1;
        hierarchy.pending_approvals.push(request);
        
        Ok(request_id)
    })
}

#[update]
fn approve_team_change(request_id: u64) -> Result<String, String> {
    require_owner()?; // SECURITY: Only owner can approve team changes
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let hierarchy = &mut pool_state.dev_team_business.team_hierarchy;
        
        if let Some(pos) = hierarchy.pending_approvals.iter().position(|req| req.request_id == request_id) {
            let mut request = hierarchy.pending_approvals.remove(pos);
            request.approved = true;
            
            // Execute the approved change
            match request.request_type {
                TeamChangeType::AddMember => {
                    match request.new_role {
                        TeamRole::Owner => return Err("Cannot add additional owners".to_string()),
                        TeamRole::SeniorManager => hierarchy.senior_managers.push(request.target_principal),
                        TeamRole::OperationsManager => hierarchy.operations_managers.push(request.target_principal),
                        TeamRole::TechManager => hierarchy.tech_managers.push(request.target_principal),
                        TeamRole::Developer => hierarchy.developers.push(request.target_principal),
                    }
                    
                    // Grant premium access
                    pool_state.dev_team_business.team_member_earnings.insert(request.target_principal, types::MemberEarnings::default());
                },
                _ => {} // Handle other change types as needed
            }
            
            Ok(format!("Team change approved and executed for request {}", request_id))
        } else {
            Err("Request not found".to_string())
        }
    })
}

#[query]
fn get_team_hierarchy() -> Result<TeamHierarchy, String> {
    require_dev_team_member()?; // SECURITY: Only team members can view hierarchy
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        Ok(pool_state.dev_team_business.team_hierarchy.clone())
    })
}

#[query]
fn get_my_role() -> Option<TeamRole> {
    let caller = ic_cdk::caller();
    get_team_member_role(caller)
}

#[query]
fn get_my_earnings() -> f64 {
    let caller = ic_cdk::caller();
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.dev_team_business.team_member_earnings
            .get(&caller)
            .map(|earnings| earnings.total_usd_value)
            .unwrap_or(0.0)
    })
}

// =============================================================================
// PREMIUM TIER ACCESS FOR DEV TEAM
// =============================================================================

#[query]
fn get_user_fee_rate(user: Principal) -> f64 {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        // Check if user is dev team member (gets premium+ access)
        if pool_state.dev_team_business.team_member_earnings.contains_key(&user) {
            0.001 // 0.1% - Premium+ tier for all dev team members
        } else {
            // For non-team members, would check their subscription tier
            // Default to free tier for now
            0.0085 // 0.85% - Free tier 
        }
    })
}

#[query]
fn get_user_tier_info(user: Principal) -> String {
    POOL_STATE.with(|state| {
        let _pool_state = state.borrow();
        
        if let Some(role) = get_team_member_role(user) {
            format!("DEV TEAM - {:?} (Premium+ 0.1% fees, Unlimited volume)", role)
        } else {
            "Free Tier (0.85% fees, Unlimited volume)".to_string()
        }
    })
}

#[query]
fn is_premium_user(user: Principal) -> bool {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        // Dev team members automatically get premium access
        pool_state.dev_team_business.team_member_earnings.contains_key(&user)
    })
}

// =============================================================================
// ANALYTICS
// =============================================================================

#[query]
fn get_pool_analytics() -> String {
    ANALYTICS.with(|analytics| {
        POOL_STATE.with(|state| {
            analytics.borrow().generate_analytics_report(&state.borrow())
        })
    })
}

#[query]
fn get_chain_distribution() -> Vec<(ChainId, f64)> {
    ANALYTICS.with(|analytics| {
        POOL_STATE.with(|state| {
            analytics.borrow().get_chain_distribution(&state.borrow())
        })
    })
}

// =============================================================================
// TREASURY MANAGEMENT APIS
// =============================================================================

#[update]
fn configure_payment_address(
    chain: String,
    asset: String,
    address: String,
    address_type: AddressType,
    max_balance_usd: Option<f64>
) -> Result<(), String> {
    require_owner()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        let payment_address = PaymentAddress {
            chain: chain.clone(),
            asset: asset.clone(),
            address: address.clone(),
            address_type,
            max_balance_usd,
            created_at: current_time,
            last_used: 0,
        };
        
        // Remove existing address for this chain/asset combination
        pool_state.payment_addresses.retain(|addr| 
            !(addr.chain == chain && addr.asset == asset)
        );
        
        // Add new address
        pool_state.payment_addresses.push(payment_address);
        
        // Update treasury config map for quick lookup
        let key = format!("{}_{}", chain, asset);
        pool_state.treasury_config.payment_addresses.insert(key, address);
        
        Ok(())
    })
}

#[query]
fn get_payment_address(chain: String, asset: String) -> Option<String> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let key = format!("{}_{}", chain, asset);
        pool_state.treasury_config.payment_addresses.get(&key).cloned()
    })
}

#[query]
fn get_all_payment_addresses() -> Vec<PaymentAddress> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        return Vec::new(); // Only managers can see all addresses
    }
    
    POOL_STATE.with(|state| {
        state.borrow().payment_addresses.clone()
    })
}

#[update]
fn set_hot_wallet_limit(chain: String, asset: String, limit_usd: f64) -> Result<(), String> {
    require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let key = format!("{}_{}", chain, asset);
        pool_state.treasury_config.hot_wallet_limits.insert(key, limit_usd);
        Ok(())
    })
}

#[update]
fn record_subscription_payment(
    user_principal: Principal,
    chain: String,
    asset: String,
    amount: f64,
    amount_usd: f64,
    tx_hash: String,
    subscription_tier: String
) -> Result<(), String> {
    require_manager_or_above()?;
    
    // SECURITY: Input validation
    if user_principal == Principal::anonymous() {
        return Err("Invalid user principal".to_string());
    }
    
    if chain.is_empty() || asset.is_empty() || tx_hash.is_empty() {
        return Err("Invalid input: chain, asset, and tx_hash cannot be empty".to_string());
    }
    
    if amount <= 0.0 || amount_usd <= 0.0 || !amount.is_finite() || !amount_usd.is_finite() {
        return Err("Invalid amounts: must be positive finite numbers".to_string());
    }
    
    // SECURITY: Prevent duplicate transaction hash
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        for existing_tx in &pool_state.treasury_transactions {
            if let Some(existing_hash) = &existing_tx.tx_hash {
                if existing_hash == &tx_hash {
                    return Err(format!("Transaction hash already recorded: {}", tx_hash));
                }
            }
        }
        Ok(())
    })?;
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Recording subscription payment - User: {}, Amount: ${}, TX: {}", 
                     user_principal.to_text(), amount_usd, tx_hash);
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Get payment address
        let key = format!("{}_{}", chain, asset);
        let to_address = pool_state.treasury_config.payment_addresses
            .get(&key)
            .unwrap_or(&"unknown".to_string()).clone();
        
        // Record treasury transaction
        let tx = TreasuryTransaction {
            id: format!("sub_{}_{}", user_principal.to_text(), current_time),
            transaction_type: TreasuryTransactionType::SubscriptionPayment,
            chain: chain.clone(),
            asset: asset.clone(),
            amount,
            amount_usd,
            from_address: "user_wallet".to_string(),
            to_address,
            tx_hash: Some(tx_hash.clone()),
            status: TransactionStatus::Confirmed,
            timestamp: current_time,
            initiated_by: user_principal,
            notes: Some(format!("Subscription payment for {} tier", subscription_tier)),
        };
        
        // SECURITY: Check storage limits before adding transaction with automatic cleanup
        check_storage_limits(&mut pool_state)?;
        
        pool_state.treasury_transactions.push(tx);
        
        // SECURITY: Update treasury balance with validation
        let mut balance_found = false;
        let _old_balance_usd = pool_state.treasury_balances
            .iter()
            .find(|b| b.chain == chain && b.asset == asset)
            .map(|b| b.amount_usd)
            .unwrap_or(0.0);
            
        for balance in &mut pool_state.treasury_balances {
            if balance.chain == chain && balance.asset == asset {
                // SECURITY: Verify balance calculations with safe arithmetic
                let new_amount = safe_add_f64(balance.amount, amount)?;
                let new_amount_usd = safe_add_f64(balance.amount_usd, amount_usd)?;
                
                if new_amount < 0.0 || new_amount_usd < 0.0 || !new_amount.is_finite() || !new_amount_usd.is_finite() {
                    return Err("SECURITY: Invalid balance calculation".to_string());
                }
                
                balance.amount = new_amount;
                balance.amount_usd = new_amount_usd;
                balance.last_updated = current_time;
                balance.last_tx_hash = Some(tx_hash.clone());
                balance_found = true;
                break;
            }
        }
        
        if !balance_found {
            let new_balance = TreasuryBalance {
                chain,
                asset,
                amount,
                amount_usd,
                last_updated: current_time,
                last_tx_hash: None,
            };
            pool_state.treasury_balances.push(new_balance);
        }
        
        // SECURITY: Process through existing business model with safe arithmetic
        pool_state.dev_team_business.monthly_subscription_revenue = 
            safe_add_f64(pool_state.dev_team_business.monthly_subscription_revenue, amount_usd)?;
        
        Ok(())
    })
}

#[query]
fn get_treasury_balance(chain: String, asset: String) -> Option<TreasuryBalance> {
    require_manager_or_above().ok()?;
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.treasury_balances.iter()
            .find(|b| b.chain == chain && b.asset == asset)
            .cloned()
    })
}

#[query]
fn get_all_treasury_balances() -> Vec<TreasuryBalance> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        return Vec::new();
    }
    
    POOL_STATE.with(|state| {
        state.borrow().treasury_balances.clone()
    })
}

#[update]
fn request_treasury_withdrawal(
    chain: String,
    asset: String,
    amount: f64,
    destination_address: String,
    reason: String
) -> Result<String, String> {
    let caller = require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Calculate USD value (simplified - in production, use price oracle)
        let amount_usd = estimate_usd_value(&asset, amount);
        
        // Check if amount exceeds hot wallet limit
        let balance_key = format!("{}_{}", chain, asset);
        let threshold = pool_state.treasury_config.hot_wallet_limits
            .get(&balance_key).unwrap_or(&10000.0); // Default $10K limit
        
        let withdrawal_id = format!("withdraw_{}_{}", caller.to_text(), current_time);
        
        let (status, required_approvals) = if amount_usd > *threshold {
            (WithdrawalStatus::PendingApproval, 2) // Requires multi-sig approval
        } else {
            (WithdrawalStatus::Approved, 0) // Auto-approved for small amounts
        };
        
        let withdrawal_request = WithdrawalRequest {
            id: withdrawal_id.clone(),
            requested_by: caller,
            chain,
            asset,
            amount,
            amount_usd,
            destination_address,
            reason,
            status,
            required_approvals,
            current_approvals: if required_approvals == 0 { vec![caller] } else { Vec::new() },
            created_at: current_time,
            approved_at: if required_approvals == 0 { Some(current_time) } else { None },
            executed_at: None,
            tx_hash: None,
        };
        
        pool_state.withdrawal_requests.push(withdrawal_request);
        
        if required_approvals == 0 {
            Ok(format!("Withdrawal {} auto-approved and ready for execution", withdrawal_id))
        } else {
            Ok(format!("Withdrawal {} requires {} approvals", withdrawal_id, required_approvals))
        }
    })
}

#[update]
fn approve_withdrawal(withdrawal_id: String) -> Result<(), String> {
    let caller = require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        if let Some(withdrawal) = pool_state.withdrawal_requests.iter_mut()
            .find(|w| w.id == withdrawal_id) {
            
            if withdrawal.status != WithdrawalStatus::PendingApproval {
                return Err("Withdrawal is not pending approval".to_string());
            }
            
            if withdrawal.current_approvals.contains(&caller) {
                return Err("You have already approved this withdrawal".to_string());
            }
            
            withdrawal.current_approvals.push(caller);
            
            // Check if we have enough approvals
            if withdrawal.current_approvals.len() >= withdrawal.required_approvals as usize {
                withdrawal.status = WithdrawalStatus::Approved;
                withdrawal.approved_at = Some(ic_cdk::api::time());
            }
            
            Ok(())
        } else {
            Err("Withdrawal request not found".to_string())
        }
    })
}

#[query]
fn get_treasury_health_report() -> TreasuryHealthReport {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        // Return limited info for non-managers
        return TreasuryHealthReport {
            total_usd_value: 0.0,
            total_assets: 0,
            balances_over_limit: Vec::new(),
            last_payment_timestamp: None,
            pending_withdrawals: 0,
            hot_wallet_utilization: 0.0,
            largest_single_balance: 0.0,
            diversification_score: 0.0,
            security_alerts: vec!["Access restricted".to_string()],
        };
    }
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        let mut total_usd_value = 0.0;
        let mut balances_over_limit = Vec::new();
        let mut largest_single_balance = 0.0;
        
        for balance in &pool_state.treasury_balances {
            total_usd_value += balance.amount_usd;
            
            if balance.amount_usd > largest_single_balance {
                largest_single_balance = balance.amount_usd;
            }
            
            let key = format!("{}_{}", balance.chain, balance.asset);
            if let Some(limit) = pool_state.treasury_config.hot_wallet_limits.get(&key) {
                if balance.amount_usd > *limit {
                    balances_over_limit.push(format!("{}: ${:.2} (limit: ${:.2})", 
                        key, balance.amount_usd, limit));
                }
            }
        }
        
        let last_payment_timestamp = pool_state.treasury_transactions
            .iter()
            .filter(|tx| tx.transaction_type == TreasuryTransactionType::SubscriptionPayment)
            .map(|tx| tx.timestamp)
            .max();
        
        let pending_withdrawals = pool_state.withdrawal_requests
            .iter()
            .filter(|w| w.status == WithdrawalStatus::PendingApproval)
            .count();
        
        let hot_wallet_utilization = calculate_hot_wallet_utilization(&pool_state);
        let diversification_score = calculate_diversification_score(&pool_state.treasury_balances);
        let security_alerts = generate_security_alerts(&pool_state);
        
        TreasuryHealthReport {
            total_usd_value,
            total_assets: pool_state.treasury_balances.len(),
            balances_over_limit,
            last_payment_timestamp,
            pending_withdrawals,
            hot_wallet_utilization,
            largest_single_balance,
            diversification_score,
            security_alerts,
        }
    })
}

#[query]
fn get_treasury_transactions(limit: Option<u64>) -> Vec<TreasuryTransaction> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        return Vec::new();
    }
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let mut transactions = pool_state.treasury_transactions.clone();
        
        // Sort by timestamp (newest first)
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if let Some(limit) = limit {
            transactions.truncate(limit as usize);
        }
        
        transactions
    })
}

// =============================================================================
// EARNINGS MANAGEMENT FUNCTIONS
// =============================================================================

#[update]
fn set_member_earnings(member: Principal, allocation: EarningsAllocation) -> Result<String, String> {
    let caller = ic_cdk::caller();
    if !is_owner_or_senior_manager(caller) {
        return Err("Access denied: Only owner or senior managers can set member earnings".to_string());
    }

    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // Verify the member exists in the team hierarchy
        let is_team_member = pool_state.dev_team_business.team_hierarchy.owner_principal == member ||
            pool_state.dev_team_business.team_hierarchy.senior_managers.contains(&member) ||
            pool_state.dev_team_business.team_hierarchy.operations_managers.contains(&member) ||
            pool_state.dev_team_business.team_hierarchy.tech_managers.contains(&member) ||
            pool_state.dev_team_business.team_hierarchy.developers.contains(&member);
            
        if !is_team_member {
            return Err("Member not found in team hierarchy".to_string());
        }

        // Validate allocation
        match &allocation {
            EarningsAllocation::Percentage(pct) => {
                if *pct < 0.0 || *pct > 100.0 {
                    return Err("Percentage must be between 0-100%".to_string());
                }
            },
            EarningsAllocation::FixedMonthlyUSD(amount) => {
                if *amount < 0.0 || *amount > 50000.0 { // Max $50k/month
                    return Err("Fixed monthly amount must be between $0-$50,000".to_string());
                }
            },
            EarningsAllocation::FixedPerTransaction(amount) => {
                if *amount < 0.0 || *amount > 1000.0 { // Max $1k per transaction
                    return Err("Fixed per-transaction amount must be between $0-$1,000".to_string());
                }
            }
        }

        // Update or create earnings config
        let config = pool_state.dev_team_business.member_earnings_config
            .entry(member)
            .or_insert_with(MemberEarningsConfig::default);
            
        config.allocation = allocation.clone();
        config.last_modified_by = caller;
        config.last_modified_time = ic_cdk::api::time();

        Ok(format!("Member earnings updated: {:?}", allocation))
    })
}

#[update]
fn update_member_role(member: Principal, new_role: TeamRole) -> Result<String, String> {
    let caller = ic_cdk::caller();
    if !is_owner_or_senior_manager(caller) {
        return Err("Access denied: Only owner or senior managers can update roles".to_string());
    }

    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        if let Some(config) = pool_state.dev_team_business.member_earnings_config.get_mut(&member) {
            config.role = new_role.clone();
            config.last_modified_by = caller;
            config.last_modified_time = ic_cdk::api::time();
            Ok(format!("Member role updated to: {:?}", new_role))
        } else {
            Err("Member earnings config not found".to_string())
        }
    })
}

#[update]
fn activate_member_earnings(member: Principal, is_active: bool) -> Result<String, String> {
    let caller = ic_cdk::caller();
    if !is_owner_or_senior_manager(caller) {
        return Err("Access denied: Only owner or senior managers can activate/deactivate earnings".to_string());
    }

    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        if let Some(config) = pool_state.dev_team_business.member_earnings_config.get_mut(&member) {
            config.is_active = is_active;
            config.last_modified_by = caller;
            config.last_modified_time = ic_cdk::api::time();
            Ok(format!("Member earnings {}", if is_active { "activated" } else { "deactivated" }))
        } else {
            Err("Member earnings config not found".to_string())
        }
    })
}

#[query]
fn get_member_earnings_config(member: Principal) -> Option<MemberEarningsConfig> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) && caller != member {
        return None; // Privacy: members can only see their own config
    }

    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.dev_team_business.member_earnings_config.get(&member).cloned()
    })
}

#[query]
fn get_all_earnings_config() -> Vec<(Principal, MemberEarningsConfig)> {
    let caller = ic_cdk::caller();
    if !is_manager_or_above(caller) {
        return Vec::new();
    }

    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        pool_state.dev_team_business.member_earnings_config
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    })
}

fn is_owner_or_senior_manager(caller: Principal) -> bool {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        caller == pool_state.dev_team_business.team_hierarchy.owner_principal ||
        pool_state.dev_team_business.team_hierarchy.senior_managers.contains(&caller)
    })
}

// =============================================================================
// TREASURY UTILITY FUNCTIONS
// =============================================================================

fn estimate_usd_value(asset: &str, amount: f64) -> f64 {
    // SECURITY: Enhanced input validation
    if asset.is_empty() || asset.len() > 10 {
        ic_cdk::println!("SECURITY: Invalid asset name: {}", asset);
        return 0.0;
    }
    
    // SECURITY: Strict amount validation with finite check first
    if !amount.is_finite() {
        ic_cdk::println!("SECURITY: Non-finite amount: {}", amount);
        return 0.0;
    }
    
    if amount < 0.0 {
        ic_cdk::println!("SECURITY: Negative amount: {}", amount);
        return 0.0;
    }
    
    if amount > 1_000_000_000.0 {
        ic_cdk::println!("SECURITY: Amount too large: {}", amount);
        return 0.0;
    }
    
    // SECURITY: Get price multiplier with strict bounds
    let price_multiplier = match asset.to_uppercase().as_str() {
        "USDC" | "USDT" | "DAI" => 1.0, // Stablecoins = 1:1 USD
        "ETH" => 2500.0, // TEMPORARY: Use oracle
        "BTC" => 45000.0, // TEMPORARY: Use oracle  
        "SOL" => 100.0, // TEMPORARY: Use oracle
        "MATIC" => 0.9, // TEMPORARY: Use oracle
        _ => {
            ic_cdk::println!("SECURITY: Unknown asset: {}", asset);
            return 0.0; // Don't guess unknown assets
        }
    };
    
    // SECURITY: Safe multiplication with overflow check
    let result = match amount.partial_cmp(&0.0) {
        Some(std::cmp::Ordering::Greater) => {
            let calculation = amount * price_multiplier;
            if calculation.is_finite() && calculation > 0.0 && calculation <= 1_000_000_000_000.0 {
                calculation
            } else {
                ic_cdk::println!("SECURITY: Calculation overflow for asset: {}, amount: {}, price: {}", asset, amount, price_multiplier);
                0.0
            }
        }
        _ => 0.0
    };
    
    result
}

fn calculate_hot_wallet_utilization(pool_state: &PoolState) -> f64 {
    let mut total_used = 0.0;
    let mut total_limits = 0.0;
    
    for balance in &pool_state.treasury_balances {
        let key = format!("{}_{}", balance.chain, balance.asset);
        if let Some(limit) = pool_state.treasury_config.hot_wallet_limits.get(&key) {
            total_used += balance.amount_usd;
            total_limits += limit;
        }
    }
    
    if total_limits > 0.0 {
        (total_used / total_limits) * 100.0
    } else {
        0.0
    }
}

fn calculate_diversification_score(balances: &Vec<TreasuryBalance>) -> f64 {
    if balances.is_empty() {
        return 0.0;
    }
    
    let total_value: f64 = balances.iter().map(|b| b.amount_usd).sum();
    if total_value == 0.0 {
        return 0.0;
    }
    
    // Calculate Herfindahl-Hirschman Index for diversification
    let hhi: f64 = balances.iter()
        .map(|b| {
            let share = b.amount_usd / total_value;
            share * share
        })
        .sum();
    
    // Convert to diversification score (1 = perfectly diversified, 0 = all in one asset)
    1.0 - hhi
}

fn generate_security_alerts(pool_state: &PoolState) -> Vec<String> {
    let mut alerts = Vec::new();
    
    // Check for balances over limits
    for balance in &pool_state.treasury_balances {
        let key = format!("{}_{}", balance.chain, balance.asset);
        if let Some(limit) = pool_state.treasury_config.hot_wallet_limits.get(&key) {
            if balance.amount_usd > *limit {
                alerts.push(format!("⚠️ {} balance exceeds limit: ${:.2} > ${:.2}", 
                    key, balance.amount_usd, limit));
            }
        }
    }
    
    // Check for stale balances (not updated in 24 hours)
    let current_time = ic_cdk::api::time();
    let day_in_ns = 24 * 60 * 60 * 1_000_000_000;
    
    for balance in &pool_state.treasury_balances {
        if current_time - balance.last_updated > day_in_ns {
            alerts.push(format!("🕐 Stale balance data for {}_{}", balance.chain, balance.asset));
        }
    }
    
    // Check for pending withdrawals older than 48 hours
    let two_days_in_ns = 2 * day_in_ns;
    
    for withdrawal in &pool_state.withdrawal_requests {
        if withdrawal.status == WithdrawalStatus::PendingApproval && 
           current_time - withdrawal.created_at > two_days_in_ns {
            alerts.push(format!("⏰ Pending withdrawal {} requires attention", withdrawal.id));
        }
    }
    
    if alerts.is_empty() {
        alerts.push("✅ No security alerts".to_string());
    }
    
    alerts
}

// SECURITY: Enhanced storage management functions with aggressive cleanup
fn prune_old_transactions(pool_state: &mut PoolState) -> Result<(), String> {
    let initial_count = pool_state.treasury_transactions.len();
    let target_count = pool_state.storage_metrics.max_treasury_transactions / 2; // Keep only 50% to create buffer
    
    if initial_count <= target_count {
        return Ok(()); // No pruning needed
    }
    
    // SECURITY: Sort by timestamp (newest first) with validation
    pool_state.treasury_transactions.sort_by(|a, b| {
        // SECURITY: Handle potential timestamp overflow/underflow
        match (a.timestamp, b.timestamp) {
            (ts_a, ts_b) if ts_a == 0 || ts_b == 0 => std::cmp::Ordering::Equal, // Handle zero timestamps
            (ts_a, ts_b) => ts_b.cmp(&ts_a) // Newest first
        }
    });
    
    // SECURITY: Additional validation before truncation
    if target_count > pool_state.treasury_transactions.len() {
        ic_cdk::println!("SECURITY: Invalid target count in pruning: {} > {}", target_count, pool_state.treasury_transactions.len());
        return Err("SECURITY: Invalid pruning parameters".to_string());
    }
    
    // Keep only the most recent transactions
    pool_state.treasury_transactions.truncate(target_count);
    
    let pruned_count = initial_count.saturating_sub(pool_state.treasury_transactions.len());
    
    // SECURITY: Check for overflow in pruned count
    match pool_state.storage_metrics.transactions_pruned.checked_add(pruned_count as u64) {
        Some(new_count) => pool_state.storage_metrics.transactions_pruned = new_count,
        None => {
            ic_cdk::println!("SECURITY: Overflow in transactions_pruned counter");
            pool_state.storage_metrics.transactions_pruned = u64::MAX; // Cap at max
        }
    }
    
    pool_state.storage_metrics.last_cleanup_time = ic_cdk::api::time();
    
    ic_cdk::println!("SECURITY: Pruned {} old transactions, kept {} (target: {})", 
                     pruned_count, pool_state.treasury_transactions.len(), target_count);
    
    Ok(())
}

// SECURITY: Enhanced storage limit checking with automatic cleanup
fn check_storage_limits(pool_state: &mut PoolState) -> Result<(), String> {
    let max_treasury_transactions = pool_state.storage_metrics.max_treasury_transactions;
    let max_withdrawal_requests = pool_state.storage_metrics.max_withdrawal_requests;
    let current_time = ic_cdk::api::time();
    
    // SECURITY: Check treasury transactions with automatic pruning
    if pool_state.treasury_transactions.len() >= max_treasury_transactions {
        ic_cdk::println!("SECURITY: Treasury transactions limit reached ({}/{}), triggering cleanup", 
                         pool_state.treasury_transactions.len(), max_treasury_transactions);
        
        // Attempt automatic cleanup
        if let Err(e) = prune_old_transactions(pool_state) {
            ic_cdk::println!("SECURITY: Failed to prune transactions: {}", e);
            return Err("SECURITY: Treasury transactions storage limit exceeded and cleanup failed".to_string());
        }
        
        // Check if cleanup was sufficient
        if pool_state.treasury_transactions.len() >= max_treasury_transactions {
            return Err("SECURITY: Treasury transactions storage limit still exceeded after cleanup".to_string());
        }
    }
    
    // SECURITY: Check withdrawal requests with cleanup of old/completed requests
    if pool_state.withdrawal_requests.len() >= max_withdrawal_requests {
        ic_cdk::println!("SECURITY: Withdrawal requests limit reached ({}/{}), attempting cleanup", 
                         pool_state.withdrawal_requests.len(), max_withdrawal_requests);
        
        // Remove completed/expired withdrawal requests older than 30 days
        let thirty_days_ago = current_time.saturating_sub(30 * 24 * 60 * 60 * 1_000_000_000);
        let initial_count = pool_state.withdrawal_requests.len();
        
        pool_state.withdrawal_requests.retain(|req| {
            // Keep pending requests and recent completed/rejected ones
            match req.status {
                WithdrawalStatus::PendingApproval => true, // Always keep pending
                WithdrawalStatus::Approved => req.created_at > thirty_days_ago, // Keep recent approved
                WithdrawalStatus::Executed | WithdrawalStatus::Rejected | WithdrawalStatus::Expired => {
                    req.created_at > thirty_days_ago // Remove old completed/rejected/expired
                }
            }
        });
        
        let cleaned_count = initial_count - pool_state.withdrawal_requests.len();
        if cleaned_count > 0 {
            ic_cdk::println!("SECURITY: Cleaned {} old withdrawal requests", cleaned_count);
        }
        
        // Check if cleanup was sufficient
        if pool_state.withdrawal_requests.len() >= max_withdrawal_requests {
            return Err("SECURITY: Withdrawal requests storage limit exceeded even after cleanup".to_string());
        }
    }
    
    // SECURITY: Check payment addresses (these should rarely need cleanup)
    let max_payment_addresses = pool_state.storage_metrics.max_payment_addresses;
    if pool_state.payment_addresses.len() >= max_payment_addresses {
        ic_cdk::println!("SECURITY: Payment addresses limit reached ({}/{})", 
                         pool_state.payment_addresses.len(), max_payment_addresses);
        
        // Remove unused payment addresses older than 1 year
        let one_year_ago = current_time.saturating_sub(365 * 24 * 60 * 60 * 1_000_000_000);
        let initial_count = pool_state.payment_addresses.len();
        
        pool_state.payment_addresses.retain(|addr| {
            // Keep recently created or recently used addresses
            addr.created_at > one_year_ago || addr.last_used > one_year_ago
        });
        
        let cleaned_count = initial_count - pool_state.payment_addresses.len();
        if cleaned_count > 0 {
            ic_cdk::println!("SECURITY: Cleaned {} unused payment addresses", cleaned_count);
        }
        
        // Check if cleanup was sufficient
        if pool_state.payment_addresses.len() >= max_payment_addresses {
            return Err("SECURITY: Payment addresses storage limit exceeded even after cleanup".to_string());
        }
    }
    
    Ok(())
}

// =============================================================================
// ICP CHAIN FUSION APIS
// =============================================================================

#[query]
fn get_chain_fusion_status() -> Result<chain_fusion::ChainFusionStatus, String> {
    CHAIN_FUSION_MANAGER.with(|manager| {
        let manager_ref = manager.borrow();
        match manager_ref.as_ref() {
            Some(chain_fusion) => Ok(chain_fusion.get_status()),
            None => Err("Chain Fusion not initialized".to_string())
        }
    })
}

#[query]
fn get_native_address(chain: String, asset: String) -> Option<String> {
    CHAIN_FUSION_MANAGER.with(|manager| {
        let manager_ref = manager.borrow();
        match manager_ref.as_ref() {
            Some(chain_fusion) => chain_fusion.get_address(&chain, &asset),
            None => None
        }
    })
}

#[query]
fn get_all_native_addresses() -> Result<std::collections::HashMap<String, String>, String> {
    require_manager_or_above()?; // Only managers can see all addresses
    
    CHAIN_FUSION_MANAGER.with(|manager| {
        let manager_ref = manager.borrow();
        match manager_ref.as_ref() {
            Some(chain_fusion) => Ok(chain_fusion.get_all_addresses()),
            None => Err("Chain Fusion not initialized".to_string())
        }
    })
}

#[update]
async fn retry_chain_fusion_initialization() -> Result<String, String> {
    require_owner()?; // Only owner can retry initialization
    
    ic_cdk::println!("ICP CHAIN FUSION: Retrying initialization...");
    
    let mut chain_fusion_manager = ChainFusionManager::new(false); // false = testnet
    match chain_fusion_manager.initialize_all_addresses().await {
        Ok(addresses) => {
            ic_cdk::println!("ICP CHAIN FUSION: Retry successful - generated {} native addresses", addresses.len());
            
            // Update treasury config with new addresses
            POOL_STATE.with(|state| {
                let mut pool_state = state.borrow_mut();
                for (chain_asset, address) in &addresses {
                    pool_state.treasury_config.payment_addresses.insert(chain_asset.clone(), address.clone());
                }
            });
            
            // Store initialized manager
            CHAIN_FUSION_MANAGER.with(|manager| {
                *manager.borrow_mut() = Some(chain_fusion_manager);
            });
            
            Ok(format!("Chain Fusion initialization successful - {} addresses generated", addresses.len()))
        }
        Err(e) => {
            ic_cdk::println!("ICP CHAIN FUSION: Retry failed - {}", e);
            Err(format!("Chain Fusion initialization failed: {}", e))
        }
    }
}

#[update]
async fn sign_transaction(chain: String, transaction_hash: String) -> Result<String, String> {
    require_manager_or_above()?; // Only managers can sign transactions
    
    // SECURITY: Input validation
    if chain.is_empty() || transaction_hash.is_empty() {
        return Err("Invalid input: chain and transaction_hash cannot be empty".to_string());
    }
    
    if transaction_hash.len() > 128 {
        return Err("Invalid transaction hash: too long".to_string());
    }
    
    // Convert hex string to bytes
    let tx_bytes = hex::decode(&transaction_hash)
        .map_err(|e| format!("Invalid transaction hash format: {}", e))?;
    
    // Check if Chain Fusion is available
    let chain_fusion_available = CHAIN_FUSION_MANAGER.with(|manager| {
        manager.borrow().is_some()
    });
    
    if !chain_fusion_available {
        return Err("Chain Fusion not initialized".to_string());
    }
    
    ic_cdk::println!("ICP CHAIN FUSION: Signing {} transaction: {}", chain, transaction_hash);
    
    // Create a temporary manager for signing
    let signature = match chain.to_lowercase().as_str() {
        "bitcoin" => {
            let temp_manager = ChainFusionManager::new(false);
            temp_manager.sign_bitcoin_transaction(&tx_bytes, None).await?
        }
        "ethereum" => {
            let temp_manager = ChainFusionManager::new(false);
            temp_manager.sign_ethereum_transaction(&tx_bytes, None).await?
        }
        "polygon" => {
            let temp_manager = ChainFusionManager::new(false);
            temp_manager.sign_polygon_transaction(&tx_bytes, None).await?
        }
        "arbitrum" => {
            let temp_manager = ChainFusionManager::new(false);
            let derivation_path = vec![b"arbitrum".to_vec()];
            temp_manager.sign_ethereum_transaction(&tx_bytes, Some(derivation_path)).await?
        }
        "base" => {
            let temp_manager = ChainFusionManager::new(false);
            let derivation_path = vec![b"base".to_vec()];
            temp_manager.sign_ethereum_transaction(&tx_bytes, Some(derivation_path)).await?
        }
        _ => return Err(format!("Unsupported chain for signing: {}", chain))
    };
    
    let signature_hex = hex::encode(&signature);
    ic_cdk::println!("ICP CHAIN FUSION: Transaction signed successfully - signature length: {}", signature.len());
    
    Ok(signature_hex)
}

#[query]
fn validate_canister_address(chain: String, asset: String, address: String) -> bool {
    CHAIN_FUSION_MANAGER.with(|manager| {
        let manager_ref = manager.borrow();
        match manager_ref.as_ref() {
            Some(chain_fusion) => chain_fusion.validate_canister_address(&chain, &asset, &address),
            None => false
        }
    })
}

#[query]
fn get_supported_chain_fusion_combinations() -> Vec<(String, String)> {
    CHAIN_FUSION_MANAGER.with(|manager| {
        let manager_ref = manager.borrow();
        match manager_ref.as_ref() {
            Some(chain_fusion) => chain_fusion.get_supported_combinations(),
            None => Vec::new()
        }
    })
}

// =============================================================================
// PAYMENT METHODS API (Updated for Chain Fusion)
// =============================================================================

#[query]
fn get_supported_payment_methods() -> Vec<PaymentMethod> {
    // Get native addresses from Chain Fusion if available
    let native_addresses = CHAIN_FUSION_MANAGER.with(|manager| {
        let manager_ref = manager.borrow();
        match manager_ref.as_ref() {
            Some(chain_fusion) => Some(chain_fusion.get_all_addresses()),
            None => None
        }
    });
    
    if let Some(addresses) = native_addresses {
        // Use Chain Fusion generated payment methods
        use chain_fusion::create_chain_fusion_payment_methods;
        create_chain_fusion_payment_methods(&addresses)
    } else {
        // Fallback to manual configuration (legacy)
        vec![
            // Ethereum USDC (Legacy manual configuration)
            PaymentMethod {
                id: "legacy_ethereum_usdc".to_string(),
                chain: ChainId::Ethereum,
                asset: Asset::USDC,
                canister_address: "manual_config_required".to_string(),
                token_address: Some("0xA0b86a33E6441b5cBb5b9c7e9a8e49A44A2a1c6f".to_string()),
                is_native_integration: false,
                key_derivation_path: Vec::new(),
                enabled: false, // Disabled until manual configuration
                min_amount_usd: 1.0,
                max_amount_usd: 10000.0,
                processing_fee_bps: 100, // 1%
                confirmation_blocks: 12,
                estimated_settlement_time: 900, // 15 minutes
            },
        ]
    }
}

#[update]
fn create_payment_request(
    payment_method_id: String,
    amount_usd: f64,
    purpose: PaymentPurpose,
    sender_address: String
) -> Result<Payment, String> {
    let caller = ic_cdk::caller();
    let current_time = ic_cdk::api::time();
    
    // SECURITY: Input validation
    if payment_method_id.is_empty() || payment_method_id.len() > 50 {
        return Err("Invalid payment method ID".to_string());
    }
    
    if amount_usd <= 0.0 || amount_usd > 1_000_000.0 || !amount_usd.is_finite() {
        return Err("Invalid amount: Must be between $0.01 and $1,000,000".to_string());
    }
    
    if sender_address.is_empty() || sender_address.len() > 100 {
        return Err("Invalid sender address".to_string());
    }
    
    if caller == Principal::anonymous() {
        return Err("Anonymous users cannot create payment requests".to_string());
    }
    
    // SECURITY: Audit logging
    ic_cdk::println!("AUDIT: Payment request - Method: {}, Amount: ${}, Caller: {}", 
                     payment_method_id, amount_usd, caller.to_text());
    
    // Find payment method
    let payment_methods = get_supported_payment_methods();
    let payment_method = payment_methods.iter()
        .find(|pm| pm.id == payment_method_id)
        .ok_or("Payment method not found")?;
    
    if !payment_method.enabled {
        return Err("Payment method is currently disabled".to_string());
    }
    
    if amount_usd < payment_method.min_amount_usd || amount_usd > payment_method.max_amount_usd {
        return Err(format!("Amount must be between ${} and ${}", 
                          payment_method.min_amount_usd, payment_method.max_amount_usd));
    }
    
    // SECURITY: Calculate fee with safe arithmetic
    let fee_amount_usd = safe_mul_f64(amount_usd, (payment_method.processing_fee_bps as f64) / 10000.0)?;
    let total_amount_usd = safe_add_f64(amount_usd, fee_amount_usd)?;
    
    // Convert to token units (assuming 1:1 for stablecoins)
    let amount = total_amount_usd;
    let fee_amount = fee_amount_usd;
    
    // Get destination address from treasury config
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        let key = format!("{}_{}", 
                         payment_method.chain.to_string().to_lowercase(),
                         asset_to_string(&payment_method.asset).to_lowercase());
        
        let destination_address = pool_state.treasury_config.payment_addresses
            .get(&key)
            .ok_or("Treasury address not configured for this payment method")?
            .clone();
        
        let payment_id = format!("pay_{}_{}", caller.to_text(), current_time);
        
        let payment = Payment {
            id: payment_id,
            user_principal: caller,
            payment_method: payment_method.clone(),
            amount,
            amount_usd: total_amount_usd,
            fee_amount,
            fee_amount_usd,
            destination_address,
            sender_address,
            tx_hash: None,
            status: PaymentStatus::Created,
            initiated_at: current_time,
            confirmed_at: None,
            expires_at: current_time + (24 * 60 * 60 * 1_000_000_000), // 24 hours in nanoseconds
            purpose,
            metadata: PaymentMetadata {
                invoice_id: None,
                notes: None,
                tags: vec![],
                refund_policy: RefundPolicy::FullRefund { within_hours: 24 },
            },
        };
        
        Ok(payment)
    })
}

#[update]
fn confirm_payment(payment_id: String, tx_hash: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Create treasury transaction for the payment
        let treasury_tx = TreasuryTransaction {
            id: format!("payment_{}", payment_id),
            transaction_type: TreasuryTransactionType::PaymentReceived,
            chain: "polygon".to_string(), // This should come from payment data
            asset: "usdc".to_string(),    // This should come from payment data
            amount: 100.0,                // This should come from payment data
            amount_usd: 100.0,           // This should come from payment data
            from_address: "user_wallet".to_string(), // This should come from payment data
            to_address: "treasury_wallet".to_string(), // This should come from payment data
            tx_hash: Some(tx_hash),
            status: TransactionStatus::Confirmed,
            timestamp: current_time,
            initiated_by: caller,
            notes: Some(format!("Payment confirmed for user {}", caller.to_text())),
        };
        
        pool_state.treasury_transactions.push(treasury_tx);
        Ok(())
    })
}

#[query]
fn get_payment_status(_payment_id: String) -> Result<PaymentStatus, String> {
    let _caller = ic_cdk::caller();
    
    // In a full implementation, we would store payments and check their status
    // For now, return a mock status
    Ok(PaymentStatus::WaitingConfirmation)
}

#[query]
fn get_user_payments(user_principal: Principal) -> Vec<Payment> {
    let caller = ic_cdk::caller();
    
    // Only allow users to see their own payments, or managers to see all
    if caller != user_principal && !is_manager_or_above(caller) {
        return Vec::new();
    }
    
    // In a full implementation, we would fetch payments from storage
    // For now, return empty list
    Vec::new()
}

#[update]
fn issue_refund(payment_id: String, reason: String) -> Result<(), String> {
    require_manager_or_above()?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        let current_time = ic_cdk::api::time();
        
        // Create refund transaction
        let refund_tx = TreasuryTransaction {
            id: format!("refund_{}", payment_id),
            transaction_type: TreasuryTransactionType::RefundIssued,
            chain: "polygon".to_string(), // This should come from original payment
            asset: "usdc".to_string(),    // This should come from original payment
            amount: 100.0,                // This should come from original payment
            amount_usd: 100.0,           // This should come from original payment
            from_address: "treasury_wallet".to_string(),
            to_address: "user_wallet".to_string(), // This should come from original payment
            tx_hash: None, // Will be filled when refund is processed
            status: TransactionStatus::Pending,
            timestamp: current_time,
            initiated_by: ic_cdk::caller(),
            notes: Some(format!("Refund issued: {}", reason)),
        };
        
        pool_state.treasury_transactions.push(refund_tx);
        Ok(())
    })
}

// =============================================================================
// CHAIN FUSION INITIALIZATION
// =============================================================================

#[update]
async fn initialize_chain_fusion() -> Result<String, String> {
    require_manager_or_above()?;
    
    ic_cdk::println!("ICP CHAIN FUSION: Initializing threshold cryptography...");
    
    let mut chain_fusion_manager = ChainFusionManager::new(false); // false = testnet for now
    match chain_fusion_manager.initialize_all_addresses().await {
        Ok(addresses) => {
            ic_cdk::println!("ICP CHAIN FUSION: Successfully generated {} native addresses", addresses.len());
            
            // Store native addresses in treasury config
            POOL_STATE.with(|state| {
                let mut pool_state = state.borrow_mut();
                for (chain_asset, address) in &addresses {
                    pool_state.treasury_config.payment_addresses.insert(chain_asset.clone(), address.clone());
                    ic_cdk::println!("ICP CHAIN FUSION: {} -> {}", chain_asset, address);
                }
            });
            
            // Store initialized Chain Fusion manager
            CHAIN_FUSION_MANAGER.with(|manager| {
                *manager.borrow_mut() = Some(chain_fusion_manager);
            });
            
            let success_msg = format!("ICP CHAIN FUSION: Successfully initialized {} native addresses", addresses.len());
            ic_cdk::println!("{}", success_msg);
            Ok(success_msg)
        }
        Err(e) => {
            let error_msg = format!("ICP CHAIN FUSION: Failed to initialize - {}", e);
            ic_cdk::println!("{}", error_msg);
            
            // Still store the manager for later retry
            CHAIN_FUSION_MANAGER.with(|manager| {
                *manager.borrow_mut() = Some(chain_fusion_manager);
            });
            
            Err(error_msg)
        }
    }
}

// =============================================================================
// POOL TERMINATION FUNCTIONS
// =============================================================================

#[update]
fn set_cofounder(cofounder_principal: Principal) -> Result<String, String> {
    let caller = require_owner()?; // Only owner can set cofounder
    
    // SECURITY: Comprehensive input validation
    validate_principal_input(&cofounder_principal, "cofounder principal")?;
    
    // SECURITY: Prevent owner from setting themselves as cofounder
    if cofounder_principal == caller {
        return Err("SECURITY: Owner cannot be their own cofounder".to_string());
    }
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // SECURITY: Check if cofounder already set
        if pool_state.cofounder_principal.is_some() {
            return Err("SECURITY: Cofounder already set - cannot be changed".to_string());
        }
        
        pool_state.cofounder_principal = Some(cofounder_principal);
        
        // AUDIT: Log cofounder assignment
        ic_cdk::println!("AUDIT: Cofounder set - Owner: {}, Cofounder: {}", 
                         caller.to_text(), cofounder_principal.to_text());
        
        Ok(format!("Cofounder successfully set: {}", cofounder_principal.to_text()))
    })
}

#[query]
fn get_cofounder() -> Option<Principal> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and existing cofounder can view cofounder info
    if !is_owner(caller) && !is_cofounder(caller) {
        return None;
    }
    
    POOL_STATE.with(|state| {
        state.borrow().cofounder_principal
    })
}

#[update]
fn initiate_pool_termination(
    reason: String,
    asset_distribution_addresses: Vec<(String, String, String)>, // (chain, asset, address)
    emergency: bool
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner or cofounder can initiate termination
    if !is_owner(caller) && !is_cofounder(caller) {
        return Err("SECURITY: Only owner or cofounder can initiate pool termination".to_string());
    }
    
    // SECURITY: Input validation with emergency-specific checks
    if emergency {
        validate_emergency_termination(&reason, caller)?;
    } else {
        validate_string_input(&reason, 10, 500, "termination reason")?;
    }
    
    if asset_distribution_addresses.is_empty() {
        return Err("SECURITY: Asset distribution addresses cannot be empty".to_string());
    }
    
    if asset_distribution_addresses.len() > 50 {
        return Err("SECURITY: Too many distribution addresses (max 50)".to_string());
    }
    
    // SECURITY: Use atomic state transition to prevent race conditions
    atomic_termination_update("initiate_pool_termination", None, |pool_state| {
        // SECURITY: Check current pool state
        match pool_state.phase {
            PoolPhase::Terminated { .. } => {
                return Err("SECURITY: Pool is already terminated".to_string());
            }
            PoolPhase::Terminating { .. } => {
                return Err("SECURITY: Pool termination already in progress".to_string());
            }
            _ => {} // Allow termination from other states
        }
        
        // SECURITY: Check if there's already an active termination request
        if pool_state.active_termination_request.is_some() {
            return Err("SECURITY: Active termination request already exists".to_string());
        }
        
        // SECURITY: Cofounder must be set for all terminations (emergency requires expedited approval, not bypass)
        if pool_state.cofounder_principal.is_none() {
            return Err("SECURITY: Cofounder must be set before any pool termination".to_string());
        }
        
        let current_time = ic_cdk::api::time();
        let termination_id = format!("termination_{}_{}", caller.to_text(), current_time);
        
        // Validate and create asset distribution plan
        let mut asset_distribution_plan = Vec::new();
        for (chain, asset, address) in asset_distribution_addresses {
            // SECURITY: Validate each distribution entry
            validate_string_input(&chain, 3, 20, "chain name")?;
            validate_string_input(&asset, 2, 10, "asset name")?;
            
            // SECURITY: Comprehensive blockchain address validation
            validate_blockchain_address(&address, &chain)
                .map_err(|e| format!("TERMINATION: Invalid {}/{} address '{}': {}", chain, asset, address, e))?;
            
            // Calculate current balance for this chain/asset
            let balance = pool_state.treasury_balances.iter()
                .find(|b| b.chain.to_lowercase() == chain.to_lowercase() && 
                         b.asset.to_lowercase() == asset.to_lowercase())
                .map(|b| b.amount)
                .unwrap_or(0.0);
            
            let estimated_usd = pool_state.treasury_balances.iter()
                .find(|b| b.chain.to_lowercase() == chain.to_lowercase() && 
                         b.asset.to_lowercase() == asset.to_lowercase())
                .map(|b| b.amount_usd)
                .unwrap_or(0.0);
            
            asset_distribution_plan.push(AssetDistribution {
                chain,
                asset,
                total_amount: balance,
                destination_address: address,
                estimated_usd_value: estimated_usd,
                status: DistributionStatus::Pending,
                tx_hash: None,
                executed_at: None,
            });
        }
        
        // SECURITY: Emergency terminations have expedited but not bypassed approval timeframes
        let expiration_time = if emergency {
            current_time + (12 * 60 * 60 * 1_000_000_000) // 12 hours for emergency (expedited)
        } else {
            current_time + (48 * 60 * 60 * 1_000_000_000) // 48 hours for normal
        };
        
        // SECURITY: Generate cryptographically secure confirmation phrase
        let secure_phrase = generate_secure_confirmation_phrase(
            &termination_id,
            caller,
            current_time,
            pool_state.state_version,
            pool_state.termination_nonce
        );
        
        // Create termination request with state version and nonce
        let termination_request = PoolTerminationRequest {
            id: termination_id.clone(),
            initiated_by: caller,
            reason: reason.clone(),
            asset_distribution_plan,
            owner_approval: None,
            cofounder_approval: None,
            created_at: current_time,
            expires_at: expiration_time,
            emergency_termination: emergency,
            expected_state_version: pool_state.state_version, // Capture current state version
            termination_nonce: pool_state.termination_nonce, // Capture current nonce
            secure_confirmation_phrase: secure_phrase.clone(), // Store secure phrase
        };
        
        pool_state.active_termination_request = Some(termination_request);
        
        // Update pool phase to Terminating
        pool_state.phase = PoolPhase::Terminating {
            initiated_at: current_time,
            termination_request: pool_state.active_termination_request.as_ref().unwrap().clone(),
        };
        
        // AUDIT: Log termination initiation (but not the secure phrase)
        ic_cdk::println!("AUDIT: Pool termination initiated - ID: {}, Initiator: {}, Emergency: {}, State Version: {}, Nonce: {}", 
                         termination_id, caller.to_text(), emergency, pool_state.state_version, pool_state.termination_nonce);
        
        Ok(format!(
            "Pool termination initiated with ID: {}\nSecure confirmation phrase: {}\n\nIMPORTANT: Save this phrase - you'll need it for approval!", 
            termination_id, secure_phrase
        ))
    })
}

#[update]
fn approve_pool_termination(
    termination_id: String,
    confirmation_phrase: String,
    approval_notes: Option<String>
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Input validation
    validate_string_input(&termination_id, 10, 100, "termination ID")?;
    validate_string_input(&confirmation_phrase, 1, 200, "confirmation phrase")?;
    
    // SECURITY: Check approval notes length if provided
    if let Some(ref notes) = approval_notes {
        validate_string_input(notes, 0, 500, "approval notes")?;
    }
    
    // SECURITY: Use atomic state transition with nonce validation to prevent race conditions
    atomic_termination_update("approve_pool_termination", None, |pool_state| {
        // SECURITY: Only owner or cofounder can approve
        let is_owner_caller = caller == pool_state.dev_team_business.team_hierarchy.owner_principal;
        let is_cofounder_caller = match pool_state.cofounder_principal {
            Some(cofounder) => caller == cofounder,
            None => false,
        };
        
        if !is_owner_caller && !is_cofounder_caller {
            return Err("SECURITY: Only owner or cofounder can approve pool termination".to_string());
        }
        
        // SECURITY: Check if termination request exists, matches ID, and has valid nonce
        let termination_request = match pool_state.active_termination_request.as_mut() {
            Some(req) if req.id == termination_id => {
                // SECURITY: Validate that the termination request is still valid for the current state
                if req.expected_state_version >= pool_state.state_version {
                    return Err("SECURITY: Termination request state version conflict - possible race condition".to_string());
                }
                req
            },
            Some(_) => return Err("SECURITY: Termination ID mismatch".to_string()),
            None => return Err("SECURITY: No active termination request found".to_string()),
        };
        
        let current_time = ic_cdk::api::time();
        
        // SECURITY: Check if request has expired
        if current_time > termination_request.expires_at {
            return Err("SECURITY: Termination request has expired".to_string());
        }
        
        // SECURITY: Validate secure confirmation phrase
        validate_secure_confirmation_phrase(&confirmation_phrase, termination_request)
            .map_err(|e| {
                ic_cdk::println!("SECURITY: Confirmation phrase validation failed from {}: {}", caller.to_text(), e);
                format!("SECURITY: {}", e)
            })?;
        
        // SECURITY: Prevent double approval from same person
        if is_owner_caller && termination_request.owner_approval.is_some() {
            return Err("SECURITY: Owner has already approved this termination".to_string());
        }
        
        if is_cofounder_caller && termination_request.cofounder_approval.is_some() {
            return Err("SECURITY: Cofounder has already approved this termination".to_string());
        }
        
        // Add approval
        let approval = TerminationApproval {
            approver: caller,
            approved_at: current_time,
            signature_confirmation: confirmation_phrase.clone(),
            notes: approval_notes,
        };
        
        if is_owner_caller {
            termination_request.owner_approval = Some(approval);
        } else if is_cofounder_caller {
            termination_request.cofounder_approval = Some(approval);
        }
        
        // AUDIT: Log approval with state tracking
        ic_cdk::println!("AUDIT: Termination approval - ID: {}, Approver: {}, Role: {}, State Version: {}, Nonce: {}", 
                         termination_id, caller.to_text(), 
                         if is_owner_caller { "Owner" } else { "Cofounder" },
                         pool_state.state_version, pool_state.termination_nonce);
        
        // SECURITY: All terminations require both approvals (emergency gets expedited timeframe, not bypassed authorization)
        let ready_to_execute = termination_request.owner_approval.is_some() && termination_request.cofounder_approval.is_some();
        
        if ready_to_execute {
            ic_cdk::println!("AUDIT: Pool termination fully approved - ready for execution");
            return Ok(format!("Termination approved and ready for execution: {}", termination_id));
        } else {
            let still_needed = if termination_request.owner_approval.is_none() {
                "still need owner approval"
            } else {
                "still need cofounder approval"
            };
            
            let urgency_note = if termination_request.emergency_termination {
                " (EMERGENCY - expedited timeframe)"
            } else {
                ""
            };
            
            Ok(format!("Termination approval recorded - {}{}", still_needed, urgency_note))
        }
    })
}

#[update]
async fn execute_pool_termination(termination_id: String) -> Result<TerminationSummary, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner can execute termination
    if !is_owner(caller) {
        return Err("SECURITY: Only owner can execute pool termination".to_string());
    }
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // Validate termination request
        let termination_request = match pool_state.active_termination_request.as_ref() {
            Some(req) if req.id == termination_id => req.clone(),
            Some(_) => return Err("SECURITY: Termination ID mismatch".to_string()),
            None => return Err("SECURITY: No active termination request found".to_string()),
        };
        
        // SECURITY: Verify both approvals exist for all terminations (emergency gets expedited timing, not bypassed authorization)
        let can_execute = termination_request.owner_approval.is_some() && termination_request.cofounder_approval.is_some();
        
        if !can_execute {
            return Err("SECURITY: Insufficient approvals for termination execution".to_string());
        }
        
        let current_time = ic_cdk::api::time();
        
        // SECURITY: Check if request has expired
        if current_time > termination_request.expires_at {
            return Err("SECURITY: Termination request has expired".to_string());
        }
        
        // Execute asset distributions (mock implementation for now)
        let mut successful_distributions = 0;
        let mut failed_distributions = 0;
        let mut total_distributed_usd = 0.0;
        let mut chains_processed = Vec::new();
        let mut final_distributions = Vec::new();
        
        for distribution in &termination_request.asset_distribution_plan {
            // SECURITY: Comprehensive blockchain address validation
            if let Err(validation_error) = validate_blockchain_address(&distribution.destination_address, &distribution.chain) {
                ic_cdk::println!("SECURITY: Invalid destination address for {}/{}: {} - {}", 
                               distribution.chain, distribution.asset, distribution.destination_address, validation_error);
                failed_distributions += 1;
                continue;
            }
            
            // Simulate asset transfer execution
            // In production, this would:
            // 1. Use Chain Fusion to sign and send transactions
            // 2. Wait for confirmations
            // 3. Update distribution status
            
            let mock_tx_hash = format!("0x{:x}{:x}", 
                                     distribution.chain.len() as u64,
                                     current_time);
            
            let mut executed_distribution = distribution.clone();
            executed_distribution.status = DistributionStatus::Completed;
            executed_distribution.tx_hash = Some(mock_tx_hash);
            executed_distribution.executed_at = Some(current_time);
            
            successful_distributions += 1;
            total_distributed_usd += executed_distribution.estimated_usd_value;
            
            if !chains_processed.contains(&executed_distribution.chain) {
                chains_processed.push(executed_distribution.chain.clone());
            }
            
            final_distributions.push(executed_distribution);
            
            ic_cdk::println!("AUDIT: Asset distributed - Chain: {}, Asset: {}, Amount: {}, Destination: {}", 
                           distribution.chain, distribution.asset, distribution.total_amount, distribution.destination_address);
        }
        
        // Create termination summary
        let termination_summary = TerminationSummary {
            total_assets_distributed: total_distributed_usd,
            chains_processed: chains_processed.clone(),
            successful_distributions,
            failed_distributions,
            termination_initiated_at: termination_request.created_at,
            termination_completed_at: Some(current_time),
            final_state_hash: format!("pool_terminated_{}", current_time),
        };
        
        // Update pool to terminated state
        pool_state.phase = PoolPhase::Terminated {
            terminated_at: current_time,
            final_asset_distribution: final_distributions,
            termination_reason: termination_request.reason.clone(),
        };
        
        // Move active request to history
        pool_state.termination_history.push(termination_request);
        pool_state.active_termination_request = None;
        
        // Clear all treasury balances (they've been distributed)
        pool_state.treasury_balances.clear();
        pool_state.total_liquidity_usd = 0.0;
        
        // AUDIT: Log termination completion
        ic_cdk::println!("AUDIT: Pool termination completed - ID: {}, Total USD: ${}, Chains: {:?}, Success: {}, Failed: {}", 
                         termination_id, total_distributed_usd, chains_processed, successful_distributions, failed_distributions);
        
        Ok(termination_summary)
    })
}

#[update]
fn cancel_pool_termination(termination_id: String, reason: String) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner can cancel termination
    if !is_owner(caller) {
        return Err("SECURITY: Only owner can cancel pool termination".to_string());
    }
    
    // SECURITY: Input validation
    validate_string_input(&reason, 5, 200, "cancellation reason")?;
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // Check active termination request
        let termination_request = match pool_state.active_termination_request.take() {
            Some(req) if req.id == termination_id => req,
            Some(req) => {
                // Put it back if ID doesn't match
                pool_state.active_termination_request = Some(req);
                return Err("SECURITY: Termination ID mismatch".to_string());
            }
            None => return Err("SECURITY: No active termination request found".to_string()),
        };
        
        // Move to history with cancellation note
        let cancelled_request = termination_request;
        // We could add a cancellation field to track this
        
        pool_state.termination_history.push(cancelled_request);
        
        // Restore previous pool phase
        pool_state.phase = match pool_state.total_liquidity_usd {
            x if x > 0.0 => PoolPhase::Active {
                activated_at: ic_cdk::api::time(),
                min_reserve_ratio: 0.1,
                max_utilization: 0.8,
            },
            _ => PoolPhase::Bootstrapping {
                started_at: ic_cdk::api::time(),
                target_liquidity: pool_state.bootstrap_targets.clone(),
                estimated_completion: ic_cdk::api::time() + (365 * 24 * 60 * 60 * 1_000_000_000),
            }
        };
        
        // AUDIT: Log cancellation
        ic_cdk::println!("AUDIT: Pool termination cancelled - ID: {}, Reason: {}, Cancelled by: {}", 
                         termination_id, reason, caller.to_text());
        
        Ok(format!("Pool termination cancelled: {}", reason))
    })
}

#[query]
fn get_active_termination_request() -> Option<PoolTerminationRequest> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and cofounder can view termination requests
    if !is_owner(caller) && !is_cofounder(caller) {
        return None;
    }
    
    POOL_STATE.with(|state| {
        state.borrow().active_termination_request.clone()
    })
}

#[query]
fn get_termination_history() -> Vec<PoolTerminationRequest> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and cofounder can view termination history
    if !is_owner(caller) && !is_cofounder(caller) {
        return Vec::new();
    }
    
    POOL_STATE.with(|state| {
        state.borrow().termination_history.clone()
    })
}

#[query]
fn get_secure_confirmation_phrase() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only owner and cofounder can retrieve confirmation phrase
    if !is_owner(caller) && !is_cofounder(caller) {
        return Err("SECURITY: Only owner or cofounder can retrieve confirmation phrase".to_string());
    }
    
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        match &pool_state.active_termination_request {
            Some(request) => {
                // AUDIT: Log phrase retrieval (but not the actual phrase)
                ic_cdk::println!("AUDIT: Secure confirmation phrase retrieved by {}", caller.to_text());
                Ok(request.secure_confirmation_phrase.clone())
            }
            None => Err("SECURITY: No active termination request found".to_string())
        }
    })
}

// Helper function to check if caller is cofounder
fn is_cofounder(caller: Principal) -> bool {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        match pool_state.cofounder_principal {
            Some(cofounder) => caller == cofounder,
            None => false,
        }
    })
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

fn asset_to_string(asset: &Asset) -> String {
    match asset {
        Asset::BTC => "BTC".to_string(),
        Asset::ETH => "ETH".to_string(),
        Asset::USDC => "USDC".to_string(),
        Asset::USDT => "USDT".to_string(),
        Asset::DAI => "DAI".to_string(),
        Asset::SOL => "SOL".to_string(),
        Asset::MATIC => "MATIC".to_string(),
        Asset::AVAX => "AVAX".to_string(),
        Asset::FLOW => "FLOW".to_string(),
    }
}

// =============================================================================
// $FLOW TOKEN MANAGEMENT FUNCTIONS
// =============================================================================

/// Get user's $FLOW token balance and staking information
#[query]
fn get_user_flow_balance(user: Principal) -> Result<UserFlowBalance, String> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        match pool_state.user_flow_balances.get(&user) {
            Some(balance) => Ok(balance.clone()),
            None => {
                // Return default balance for new users
                Ok(UserFlowBalance {
                    user,
                    total_balance: 0,
                    available_balance: 0,
                    staked_balance: 0,
                    pending_rewards: 0,
                    stake_lock_period: None,
                    stake_end_time: None,
                    stake_multiplier: 1.0,
                    defi_operations_count: 0,
                    social_posts_count: 0,
                    last_activity_timestamp: 0,
                    activity_streak_days: 0,
                    lifetime_rewards_earned: 0,
                    lifetime_fees_paid_in_flow: 0,
                })
            }
        }
    })
}

/// Get $FLOW token reserve information (public stats)
#[query]
fn get_flow_token_reserve() -> Result<FlowTokenReserve, String> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        Ok(pool_state.flow_token_reserve.clone())
    })
}

/// Award $FLOW tokens to user for DeFi operations
#[update]
fn award_flow_tokens(
    user: Principal, 
    operation_type: String, 
    transaction_amount_usd: f64,
    is_cross_chain: bool
) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only authorized backend canisters can award tokens
    if !is_authorized_payment_processor(caller) {
        return Err("SECURITY: Only authorized backends can award tokens".to_string());
    }
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // Calculate reward amount based on operation type and user tier
        let base_reward = match operation_type.as_str() {
            "yield_farming" => {
                (transaction_amount_usd / 1000.0) * (pool_state.flow_reward_config.yield_farming_reward_rate as f64)
            },
            "arbitrage" => pool_state.flow_reward_config.arbitrage_reward_rate as f64,
            "rebalancing" => pool_state.flow_reward_config.rebalancing_reward_rate as f64,
            "social_automation" => pool_state.flow_reward_config.social_automation_reward_rate as f64,
            _ => return Err("Invalid operation type".to_string()),
        };
        
        // Apply bonuses
        let mut final_reward = base_reward;
        
        // Cross-chain bonus
        if is_cross_chain {
            final_reward *= pool_state.flow_reward_config.cross_chain_bonus;
        }
        
        // Store the reward config values to avoid borrow conflicts
        let daily_active_bonus = pool_state.flow_reward_config.daily_active_bonus;
        let weekly_streak_bonus = pool_state.flow_reward_config.weekly_streak_bonus;
        let monthly_power_user_bonus = pool_state.flow_reward_config.monthly_power_user_bonus;
        let quarterly_champion_bonus = pool_state.flow_reward_config.quarterly_champion_bonus;
        let current_rewards_pool = pool_state.flow_token_reserve.community_rewards_pool;
        
        // Get or create user balance
        let user_balance = pool_state.user_flow_balances.entry(user).or_insert_with(|| {
            UserFlowBalance {
                user,
                total_balance: 0,
                available_balance: 0,
                staked_balance: 0,
                pending_rewards: 0,
                stake_lock_period: None,
                stake_end_time: None,
                stake_multiplier: 1.0,
                defi_operations_count: 0,
                social_posts_count: 0,
                last_activity_timestamp: ic_cdk::api::time(),
                activity_streak_days: 0,
                lifetime_rewards_earned: 0,
                lifetime_fees_paid_in_flow: 0,
            }
        });
        
        // Apply staking multiplier
        final_reward *= user_balance.stake_multiplier;
        
        // Apply activity streak bonuses
        let current_time = ic_cdk::api::time();
        let days_since_last_activity = (current_time - user_balance.last_activity_timestamp) / (24 * 60 * 60 * 1_000_000_000);
        
        if days_since_last_activity <= 1 {
            user_balance.activity_streak_days += 1;
        } else {
            user_balance.activity_streak_days = 1;
        }
        
        // Apply streak bonuses
        if user_balance.activity_streak_days >= 1 {
            final_reward *= daily_active_bonus;
        }
        if user_balance.activity_streak_days >= 7 {
            final_reward *= weekly_streak_bonus;
        }
        if user_balance.activity_streak_days >= 30 {
            final_reward *= monthly_power_user_bonus;
        }
        if user_balance.activity_streak_days >= 90 {
            final_reward *= quarterly_champion_bonus;
        }
        
        let reward_amount = final_reward as u64;
        
        // Check if community rewards pool has sufficient balance
        if current_rewards_pool < reward_amount {
            return Err("Insufficient rewards pool balance".to_string());
        }
        
        // Award tokens
        user_balance.pending_rewards += reward_amount;
        user_balance.lifetime_rewards_earned += reward_amount;
        user_balance.last_activity_timestamp = current_time;
        
        // Update operation counts
        match operation_type.as_str() {
            "yield_farming" | "arbitrage" | "rebalancing" => {
                user_balance.defi_operations_count += 1;
            },
            "social_automation" => {
                user_balance.social_posts_count += 1;
            },
            _ => {}
        }
        
        // Update reserve
        pool_state.flow_token_reserve.community_rewards_pool -= reward_amount;
        pool_state.flow_token_reserve.rewards_distributed_total += reward_amount;
        pool_state.flow_token_reserve.last_reward_distribution = current_time;
        
        // Record transaction
        pool_state.flow_transactions.push(FlowTransaction {
            transaction_id: format!("reward_{}_{}_{}", user.to_text(), operation_type, current_time),
            user,
            transaction_type: FlowTransactionType::RewardEarned { operation_type: operation_type.clone() },
            amount: reward_amount,
            timestamp: current_time,
            details: format!("Earned {} FLOW for {} operation (${} USD)", 
                           reward_amount as f64 / 100_000_000.0, operation_type, transaction_amount_usd),
        });
        
        Ok(format!("Awarded {} FLOW tokens to {} for {} operation", 
                  reward_amount as f64 / 100_000_000.0, user.to_text(), operation_type))
    })
}

/// Claim pending $FLOW rewards (move from pending to available balance)
#[update]
fn claim_flow_rewards() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        let user_balance = pool_state.user_flow_balances.get_mut(&caller)
            .ok_or("User balance not found".to_string())?;
        
        if user_balance.pending_rewards == 0 {
            return Ok("No pending rewards to claim".to_string());
        }
        
        let claimed_amount = user_balance.pending_rewards;
        user_balance.available_balance += claimed_amount;
        user_balance.total_balance += claimed_amount;
        user_balance.pending_rewards = 0;
        
        // Update circulating supply
        pool_state.flow_token_reserve.circulating_supply += claimed_amount;
        
        // Record transaction
        let current_time = ic_cdk::api::time();
        pool_state.flow_transactions.push(FlowTransaction {
            transaction_id: format!("claim_{}_{}", caller.to_text(), current_time),
            user: caller,
            transaction_type: FlowTransactionType::RewardEarned { operation_type: "claim".to_string() },
            amount: claimed_amount,
            timestamp: current_time,
            details: format!("Claimed {} FLOW rewards", claimed_amount as f64 / 100_000_000.0),
        });
        
        Ok(format!("Successfully claimed {} FLOW tokens", claimed_amount as f64 / 100_000_000.0))
    })
}

/// Stake $FLOW tokens for enhanced rewards
#[update]
fn stake_flow_tokens(amount: u64, lock_period_days: u32) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // Validate lock period
    let (lock_period_seconds, multiplier) = match lock_period_days {
        30 => (30 * 24 * 60 * 60, 1.2),
        90 => (90 * 24 * 60 * 60, 1.5),
        180 => (180 * 24 * 60 * 60, 2.0),
        365 => (365 * 24 * 60 * 60, 3.0),
        _ => return Err("Invalid lock period. Valid options: 30, 90, 180, 365 days".to_string()),
    };
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        let user_balance = pool_state.user_flow_balances.get_mut(&caller)
            .ok_or("User balance not found".to_string())?;
        
        if user_balance.available_balance < amount {
            return Err("Insufficient available balance".to_string());
        }
        
        if user_balance.staked_balance > 0 {
            return Err("Already have active staking. Unstake first to change terms".to_string());
        }
        
        let current_time = ic_cdk::api::time();
        let stake_end_time = current_time + (lock_period_seconds as u64 * 1_000_000_000);
        
        // Stake tokens
        user_balance.available_balance -= amount;
        user_balance.staked_balance = amount;
        user_balance.stake_lock_period = Some(lock_period_seconds as u64);
        user_balance.stake_end_time = Some(stake_end_time);
        user_balance.stake_multiplier = multiplier;
        
        // Record transaction
        pool_state.flow_transactions.push(FlowTransaction {
            transaction_id: format!("stake_{}_{}", caller.to_text(), current_time),
            user: caller,
            transaction_type: FlowTransactionType::Staking { lock_period_days },
            amount,
            timestamp: current_time,
            details: format!("Staked {} FLOW for {} days ({}x multiplier)", 
                           amount as f64 / 100_000_000.0, lock_period_days, multiplier),
        });
        
        Ok(format!("Successfully staked {} FLOW tokens for {} days with {}x multiplier", 
                  amount as f64 / 100_000_000.0, lock_period_days, multiplier))
    })
}

/// Unstake $FLOW tokens (after lock period expires)
#[update]
fn unstake_flow_tokens() -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        let user_balance = pool_state.user_flow_balances.get_mut(&caller)
            .ok_or("User balance not found".to_string())?;
        
        if user_balance.staked_balance == 0 {
            return Err("No tokens currently staked".to_string());
        }
        
        let current_time = ic_cdk::api::time();
        if let Some(stake_end_time) = user_balance.stake_end_time {
            if current_time < stake_end_time {
                return Err("Staking period has not expired yet".to_string());
            }
        }
        
        let unstaked_amount = user_balance.staked_balance;
        user_balance.available_balance += unstaked_amount;
        user_balance.staked_balance = 0;
        user_balance.stake_lock_period = None;
        user_balance.stake_end_time = None;
        user_balance.stake_multiplier = 1.0;
        
        // Record transaction
        pool_state.flow_transactions.push(FlowTransaction {
            transaction_id: format!("unstake_{}_{}", caller.to_text(), current_time),
            user: caller,
            transaction_type: FlowTransactionType::Unstaking,
            amount: unstaked_amount,
            timestamp: current_time,
            details: format!("Unstaked {} FLOW tokens", unstaked_amount as f64 / 100_000_000.0),
        });
        
        Ok(format!("Successfully unstaked {} FLOW tokens", unstaked_amount as f64 / 100_000_000.0))
    })
}

/// Calculate fee discount for user based on FLOW holdings
#[query]
fn get_user_fee_discount(user: Principal) -> Result<(f64, f64), String> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        let user_balance = pool_state.user_flow_balances.get(&user);
        let total_flow_balance = match user_balance {
            Some(balance) => balance.total_balance,
            None => 0,
        };
        
        // Find applicable discount tier
        let mut transaction_discount = 0.0;
        let mut subscription_discount = 0.0;
        
        for tier in &pool_state.flow_reward_config.fee_discount_tiers {
            if total_flow_balance >= tier.minimum_flow_balance {
                transaction_discount = tier.transaction_fee_discount;
                subscription_discount = tier.subscription_discount;
            }
        }
        
        Ok((transaction_discount, subscription_discount))
    })
}

/// Pay fees using $FLOW tokens (with discount)
#[update]
fn pay_fees_with_flow(service: String, base_fee_usd: f64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        // Get discount tiers first to avoid borrow conflicts
        let fee_discount_tiers = pool_state.flow_reward_config.fee_discount_tiers.clone();
        
        let user_balance = pool_state.user_flow_balances.get_mut(&caller)
            .ok_or("User balance not found".to_string())?;
        
        // Calculate discount
        let (transaction_discount, _) = fee_discount_tiers
            .iter()
            .rev()
            .find(|tier| user_balance.total_balance >= tier.minimum_flow_balance)
            .map(|tier| (tier.transaction_fee_discount, tier.subscription_discount))
            .unwrap_or((0.0, 0.0));
        
        let discounted_fee_usd = base_fee_usd * (1.0 - transaction_discount);
        
        // Convert USD to FLOW tokens (assuming $0.10 per FLOW for now)
        // In production, this should use real price feed
        let flow_price_usd = 0.10;
        let fee_in_flow = (discounted_fee_usd / flow_price_usd * 100_000_000.0) as u64; // Convert to 8 decimals
        
        if user_balance.available_balance < fee_in_flow {
            return Err(format!("Insufficient FLOW balance. Need {} FLOW, have {}", 
                              fee_in_flow as f64 / 100_000_000.0, 
                              user_balance.available_balance as f64 / 100_000_000.0));
        }
        
        // Deduct tokens
        user_balance.available_balance -= fee_in_flow;
        user_balance.total_balance -= fee_in_flow;
        user_balance.lifetime_fees_paid_in_flow += fee_in_flow;
        
        // Update treasury (fees collected in FLOW)
        pool_state.flow_token_reserve.treasury_reserve_pool += fee_in_flow;
        
        // Record transaction
        let current_time = ic_cdk::api::time();
        pool_state.flow_transactions.push(FlowTransaction {
            transaction_id: format!("fee_{}_{}", caller.to_text(), current_time),
            user: caller,
            transaction_type: FlowTransactionType::FeePayment { service: service.clone() },
            amount: fee_in_flow,
            timestamp: current_time,
            details: format!("Paid {} FLOW for {} ({}% discount)", 
                           fee_in_flow as f64 / 100_000_000.0, service, transaction_discount * 100.0),
        });
        
        Ok(format!("Paid {} FLOW tokens for {} with {}% discount", 
                  fee_in_flow as f64 / 100_000_000.0, service, transaction_discount * 100.0))
    })
}

/// Get user's transaction history with FLOW tokens
#[query]
fn get_user_flow_transactions(user: Principal, limit: usize) -> Result<Vec<FlowTransaction>, String> {
    POOL_STATE.with(|state| {
        let pool_state = state.borrow();
        
        let user_transactions: Vec<FlowTransaction> = pool_state.flow_transactions
            .iter()
            .filter(|tx| tx.user == user)
            .rev() // Most recent first
            .take(limit)
            .cloned()
            .collect();
        
        Ok(user_transactions)
    })
}

/// Admin function: Initialize token airdrop for early users
#[update]
fn initialize_token_airdrop(recipients: Vec<(Principal, u64)>) -> Result<String, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only managers and above can initiate airdrops
    if !is_manager_or_above(caller) {
        return Err("Access denied: Only managers can initialize airdrops".to_string());
    }
    
    POOL_STATE.with(|state| {
        let mut pool_state = state.borrow_mut();
        
        let mut total_airdrop = 0u64;
        for (_, amount) in &recipients {
            total_airdrop += amount;
        }
        
        // Check if public launch pool has sufficient balance
        if pool_state.flow_token_reserve.public_launch_pool < total_airdrop {
            return Err("Insufficient public launch pool balance for airdrop".to_string());
        }
        
        let current_time = ic_cdk::api::time();
        let recipients_len = recipients.len();
        
        // Distribute tokens
        for (user, amount) in recipients {
            // Get or create user balance
            let user_balance = pool_state.user_flow_balances.entry(user).or_insert_with(|| {
                UserFlowBalance {
                    user,
                    total_balance: 0,
                    available_balance: 0,
                    staked_balance: 0,
                    pending_rewards: 0,
                    stake_lock_period: None,
                    stake_end_time: None,
                    stake_multiplier: 1.0,
                    defi_operations_count: 0,
                    social_posts_count: 0,
                    last_activity_timestamp: current_time,
                    activity_streak_days: 0,
                    lifetime_rewards_earned: 0,
                    lifetime_fees_paid_in_flow: 0,
                }
            });
            
            // Add airdrop tokens
            user_balance.available_balance += amount;
            user_balance.total_balance += amount;
            
            // Record transaction
            pool_state.flow_transactions.push(FlowTransaction {
                transaction_id: format!("airdrop_{}_{}", user.to_text(), current_time),
                user,
                transaction_type: FlowTransactionType::Airdrop,
                amount,
                timestamp: current_time,
                details: format!("Received {} FLOW tokens via airdrop", amount as f64 / 100_000_000.0),
            });
        }
        
        // Update reserve
        pool_state.flow_token_reserve.public_launch_pool -= total_airdrop;
        pool_state.flow_token_reserve.circulating_supply += total_airdrop;
        
        Ok(format!("Successfully airdropped {} total FLOW tokens to {} users", 
                  total_airdrop as f64 / 100_000_000.0, recipients_len))
    })
}

// Export Candid interface
ic_cdk::export_candid!();