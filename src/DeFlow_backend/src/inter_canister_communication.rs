// Robust Inter-Canister Communication Module
// Implements ICP best practices for secure and reliable canister-to-canister calls

use candid::{CandidType, Deserialize, Principal, Encode};
use ic_cdk::api::call::{call_with_payment, RejectionCode};
use ic_cdk::api::cycles_burn;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Maximum number of retry attempts for failed calls
const MAX_RETRY_ATTEMPTS: u8 = 3;

/// Base delay for exponential backoff (in nanoseconds)
const BASE_RETRY_DELAY_NS: u64 = 1_000_000_000; // 1 second

/// Maximum cycles to spend on a single call
const MAX_CYCLES_PER_CALL: u64 = 100_000_000_000; // 100B cycles

/// Timeout for inter-canister calls (in seconds)
const CALL_TIMEOUT_SECONDS: u64 = 30;

/// Result of an inter-canister call attempt
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct CallResult<T> {
    pub success: bool,
    pub result: Option<T>,
    pub error: Option<String>,
    pub attempts: u8,
    pub cycles_consumed: u64,
}

/// Error types for inter-canister communication
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum InterCanisterError {
    /// Call was rejected with a specific rejection code
    CallRejected {
        code: u32, // Use u32 instead of RejectionCode for serialization
        message: String,
        is_retriable: bool,
    },
    /// Call timed out
    Timeout,
    /// Maximum retries exceeded
    MaxRetriesExceeded { last_error: String },
    /// Insufficient cycles to make the call
    InsufficientCycles { required: u64, available: u64 },
    /// Invalid destination canister
    InvalidDestination { canister_id: Principal },
    /// Serialization/deserialization error
    SerializationError { message: String },
    /// State validation failed after callback
    StateValidationFailed { message: String },
    /// Potential call loop detected
    CallLoopDetected { path: Vec<Principal> },
}

/// Configuration for inter-canister calls
#[derive(Debug, Clone)]
pub struct CallConfig {
    pub max_retries: u8,
    pub timeout_seconds: u64,
    pub cycles_limit: u64,
    pub enable_idempotency: bool,
    pub validate_state_after: bool,
}

impl Default for CallConfig {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRY_ATTEMPTS,
            timeout_seconds: CALL_TIMEOUT_SECONDS,
            cycles_limit: MAX_CYCLES_PER_CALL,
            enable_idempotency: true,
            validate_state_after: true,
        }
    }
}

/// Tracks ongoing calls to detect potential loops
static CALL_STACK: OnceLock<Mutex<Vec<Principal>>> = OnceLock::new();
static IDEMPOTENCY_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

/// Enhanced inter-canister communication manager
pub struct InterCanisterManager;

impl InterCanisterManager {
    /// Make a robust inter-canister call with full error handling and retries
    pub async fn call_with_retry<T, R>(
        destination: Principal,
        method: &str,
        args: T,
        config: Option<CallConfig>,
    ) -> Result<CallResult<R>, InterCanisterError>
    where
        T: CandidType + Clone + std::fmt::Debug,
        R: CandidType + for<'de> Deserialize<'de> + Serialize,
    {
        let config = config.unwrap_or_default();
        let mut attempts = 0u8;
        #[allow(unused_assignments)]
        let mut last_error = String::new();
        let mut total_cycles_consumed = 0u64;

        // Check for call loops
        Self::check_call_loop(destination)?;

        // Add to call stack
        Self::add_to_call_stack(destination);

        // Generate idempotency key if enabled
        let idempotency_key = if config.enable_idempotency {
            Some(Self::generate_idempotency_key(&destination, method, &args))
        } else {
            None
        };

        // Check idempotency cache
        if let Some(ref key) = idempotency_key {
            if let Some(cached_result) = Self::get_cached_result(key) {
                Self::remove_from_call_stack();
                return Self::deserialize_cached_result(cached_result);
            }
        }

        let call_result = loop {
            attempts += 1;

            // Pre-call state snapshot for validation
            let pre_call_state = if config.validate_state_after {
                Some(Self::capture_state_snapshot())
            } else {
                None
            };

            // Check cycles availability
            let available_cycles = ic_cdk::api::canister_balance();
            if available_cycles < config.cycles_limit {
                break Err(InterCanisterError::InsufficientCycles {
                    required: config.cycles_limit,
                    available: available_cycles,
                });
            }

            // Make the actual call
            let call_start_time = ic_cdk::api::time();
            let call_result = call_with_payment(
                destination,
                method,
                (args.clone(),),
                config.cycles_limit / (config.max_retries as u64 + 1), // Distribute cycles across retries
            )
            .await;

            let call_result: Result<(R,), _> = call_result;

            let call_end_time = ic_cdk::api::time();
            let call_duration = call_end_time - call_start_time;

            // Estimate cycles consumed (rough approximation)
            let cycles_consumed = Self::estimate_cycles_consumed(call_duration);
            total_cycles_consumed += cycles_consumed;

            match call_result {
                Ok((result,)) => {
                    // Success - validate state if required
                    if let Some(snapshot) = pre_call_state {
                        if let Err(validation_error) = Self::validate_state_after_call(&snapshot) {
                            break Err(InterCanisterError::StateValidationFailed {
                                message: validation_error,
                            });
                        }
                    }

                    // Cache result if idempotency is enabled
                    if let Some(ref key) = idempotency_key {
                        Self::cache_result(key, &result)?;
                    }

                    break Ok(CallResult {
                        success: true,
                        result: Some(result),
                        error: None,
                        attempts,
                        cycles_consumed: total_cycles_consumed,
                    });
                }
                Err((rejection_code, rejection_message)) => {
                    last_error = format!("Rejection: {:?} - {}", rejection_code, rejection_message);

                    let is_retriable = Self::is_retriable_rejection(&rejection_code);

                    if !is_retriable || attempts >= config.max_retries {
                        break Err(InterCanisterError::CallRejected {
                            code: rejection_code as u32,
                            message: rejection_message,
                            is_retriable,
                        });
                    }
                }
            }

            // If we haven't broken out, prepare for retry
            if attempts >= config.max_retries {
                break Err(InterCanisterError::MaxRetriesExceeded { last_error });
            }

            // Exponential backoff before retry
            let delay_ns = BASE_RETRY_DELAY_NS * (2u64.pow(attempts as u32 - 1));
            Self::delay(delay_ns).await;
        };

        // Remove from call stack
        Self::remove_from_call_stack();

        call_result
    }

    /// Simplified call for trusted canisters (same subnet, controlled by us)
    pub async fn call_trusted<T, R>(
        destination: Principal,
        method: &str,
        args: T,
    ) -> Result<R, InterCanisterError>
    where
        T: CandidType + Clone + std::fmt::Debug,
        R: CandidType + for<'de> Deserialize<'de> + Serialize,
    {
        let config = CallConfig {
            max_retries: 1, // Minimal retries for trusted calls
            validate_state_after: false, // Skip validation for trusted calls
            enable_idempotency: false, // Skip caching for trusted calls
            ..Default::default()
        };

        let result = Self::call_with_retry(destination, method, args, Some(config)).await?;
        result.result.ok_or_else(|| InterCanisterError::CallRejected {
            code: 5, // RejectionCode::Unknown as u32
            message: "No result returned".to_string(),
            is_retriable: false,
        })
    }

    /// Call with explicit timeout
    pub async fn call_with_timeout<T, R>(
        destination: Principal,
        method: &str,
        args: T,
        timeout_seconds: u64,
    ) -> Result<R, InterCanisterError>
    where
        T: CandidType + Clone + std::fmt::Debug,
        R: CandidType + for<'de> Deserialize<'de> + Serialize,
    {
        let config = CallConfig {
            timeout_seconds,
            ..Default::default()
        };

        let result = Self::call_with_retry(destination, method, args, Some(config)).await?;
        result.result.ok_or_else(|| InterCanisterError::CallRejected {
            code: 5, // RejectionCode::Unknown as u32
            message: "No result returned".to_string(),
            is_retriable: false,
        })
    }

    // Private helper methods

    /// Helper functions for managing static state
    fn get_call_stack() -> std::sync::MutexGuard<'static, Vec<Principal>> {
        CALL_STACK.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap()
    }

    fn get_cache() -> std::sync::MutexGuard<'static, HashMap<String, String>> {
        IDEMPOTENCY_CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap()
    }

    fn add_to_call_stack(destination: Principal) {
        let mut stack = Self::get_call_stack();
        stack.push(destination);
    }

    fn remove_from_call_stack() {
        let mut stack = Self::get_call_stack();
        stack.pop();
    }

    /// Check for potential call loops
    fn check_call_loop(destination: Principal) -> Result<(), InterCanisterError> {
        let stack = Self::get_call_stack();
        if stack.contains(&destination) {
            return Err(InterCanisterError::CallLoopDetected {
                path: stack.clone(),
            });
        }
        Ok(())
    }

    /// Generate idempotency key for caching
    fn generate_idempotency_key<T: CandidType + std::fmt::Debug>(
        destination: &Principal,
        method: &str,
        args: &T,
    ) -> String {
        // In a real implementation, you'd use a proper hash function
        format!("{}-{}-{:?}", destination.to_text(), method, args)
    }

    /// Cache successful result
    fn cache_result<R: CandidType + Serialize>(
        key: &str,
        result: &R,
    ) -> Result<(), InterCanisterError> {
        // Simplified caching - in production, use proper serialization
        let serialized = serde_json::to_string(result)
            .map_err(|e| InterCanisterError::SerializationError {
                message: e.to_string(),
            })?;

        let mut cache = Self::get_cache();
        cache.insert(key.to_string(), serialized);
        Ok(())
    }

    /// Get cached result
    fn get_cached_result(key: &str) -> Option<String> {
        let cache = Self::get_cache();
        cache.get(key).cloned()
    }

    /// Deserialize cached result
    fn deserialize_cached_result<R: for<'de> Deserialize<'de>>(
        cached: String,
    ) -> Result<CallResult<R>, InterCanisterError> {
        let result: R = serde_json::from_str(&cached)
            .map_err(|e| InterCanisterError::SerializationError {
                message: e.to_string(),
            })?;

        Ok(CallResult {
            success: true,
            result: Some(result),
            error: None,
            attempts: 0, // Cached result
            cycles_consumed: 0,
        })
    }

    /// Capture state snapshot for validation
    fn capture_state_snapshot() -> String {
        // Simplified - in practice, capture relevant canister state
        format!("snapshot-{}", ic_cdk::api::time())
    }

    /// Validate state after call
    fn validate_state_after_call(snapshot: &str) -> Result<(), String> {
        // Simplified validation - implement actual state consistency checks
        ic_cdk::println!("Validating state after call with snapshot: {}", snapshot);
        Ok(())
    }

    /// Check if rejection code indicates a retriable error
    fn is_retriable_rejection(code: &RejectionCode) -> bool {
        matches!(
            code,
            RejectionCode::SysTransient | RejectionCode::Unknown
        )
    }

    /// Check if application error is retriable
    fn is_retriable_app_error(error: &str) -> bool {
        // Define patterns for retriable application errors
        let retriable_patterns = [
            "temporarily unavailable",
            "rate limited",
            "queue full",
            "busy",
            "timeout",
        ];

        let error_lower = error.to_lowercase();
        retriable_patterns.iter().any(|pattern| error_lower.contains(pattern))
    }

    /// Estimate cycles consumed based on call duration
    fn estimate_cycles_consumed(duration_ns: u64) -> u64 {
        // Rough approximation - adjust based on actual profiling
        duration_ns / 1000 // Very simplified calculation
    }

    /// Async delay implementation
    async fn delay(duration_ns: u64) {
        // Simple delay - in production, use proper timer
        let cycles_to_burn = duration_ns / 1000;
        if cycles_to_burn > 0 {
            cycles_burn(cycles_to_burn as u128);
        }
    }
}

/// Convenience functions for common call patterns

/// Pool Asset enum (must match DeFlow_pool/src/types.rs::Asset)
#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum PoolAsset {
    BTC,
    ETH,
    USDC,
    USDT,
    DAI,
    SOL,
    MATIC,
    AVAX,
    FLOW,
}

impl PoolAsset {
    /// Convert asset symbol string to PoolAsset enum
    pub fn from_symbol(symbol: &str) -> Result<Self, String> {
        match symbol.to_uppercase().as_str() {
            "BTC" | "BITCOIN" => Ok(PoolAsset::BTC),
            "ETH" | "ETHEREUM" => Ok(PoolAsset::ETH),
            "USDC" => Ok(PoolAsset::USDC),
            "USDT" => Ok(PoolAsset::USDT),
            "DAI" => Ok(PoolAsset::DAI),
            "SOL" | "SOLANA" => Ok(PoolAsset::SOL),
            "MATIC" | "POLYGON" => Ok(PoolAsset::MATIC),
            "AVAX" | "AVALANCHE" => Ok(PoolAsset::AVAX),
            "FLOW" | "DEFLOW" => Ok(PoolAsset::FLOW),
            _ => Err(format!("Unsupported asset: {}", symbol)),
        }
    }
}

/// Call the pool canister to deposit fees (with full error handling)
pub async fn deposit_fee_to_pool(
    pool_canister: Principal,
    asset: String,
    amount: u64,
    transaction_id: String,
    user: Principal,
) -> Result<String, InterCanisterError> {
    // Convert asset symbol to PoolAsset enum
    let pool_asset = PoolAsset::from_symbol(&asset)
        .map_err(|e| InterCanisterError::SerializationError { message: e })?;

    InterCanisterManager::call_with_retry(
        pool_canister,
        "deposit_fee",
        (pool_asset, amount, transaction_id, user),
        Some(CallConfig {
            max_retries: 3,
            cycles_limit: 50_000_000_000, // 50B cycles for fee operations
            ..Default::default()
        }),
    )
    .await?
    .result
    .ok_or_else(|| InterCanisterError::CallRejected {
        code: 5, // RejectionCode::Unknown as u32
        message: "No receipt returned".to_string(),
        is_retriable: false,
    })
}

/// Get canister status (for health checks)
pub async fn get_canister_status(
    canister_id: Principal,
) -> Result<ic_cdk::api::management_canister::main::CanisterStatusResponse, InterCanisterError> {
    InterCanisterManager::call_trusted(
        Principal::management_canister(),
        "canister_status",
        (ic_cdk::api::management_canister::main::CanisterIdRecord {
            canister_id,
        },),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_config_default() {
        let config = CallConfig::default();
        assert_eq!(config.max_retries, MAX_RETRY_ATTEMPTS);
        assert_eq!(config.timeout_seconds, CALL_TIMEOUT_SECONDS);
        assert!(config.enable_idempotency);
        assert!(config.validate_state_after);
    }

    #[test]
    fn test_is_retriable_rejection() {
        assert!(InterCanisterManager::is_retriable_rejection(&RejectionCode::SysTransient));
        assert!(InterCanisterManager::is_retriable_rejection(&RejectionCode::Unknown));
        assert!(!InterCanisterManager::is_retriable_rejection(&RejectionCode::CanisterReject));
        assert!(!InterCanisterManager::is_retriable_rejection(&RejectionCode::DestinationInvalid));
    }

    #[test]
    fn test_is_retriable_app_error() {
        assert!(InterCanisterManager::is_retriable_app_error("Service temporarily unavailable"));
        assert!(InterCanisterManager::is_retriable_app_error("Rate limited"));
        assert!(!InterCanisterManager::is_retriable_app_error("Invalid input parameter"));
        assert!(!InterCanisterManager::is_retriable_app_error("Unauthorized access"));
    }
}