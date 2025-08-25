// Transaction fee collection service
// Handles automatic fee collection and deposit to pool canister

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use ic_cdk::api;
use crate::types::SubscriptionTier;
use crate::defi::Asset;
use crate::user_management;

// Pool canister ID - should be set via environment or init
static mut POOL_CANISTER_ID: Option<Principal> = None;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct FeeCollectionResult {
    pub success: bool,
    pub fee_amount: u64,
    pub transaction_id: String,
    pub error: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TransactionFeeRequest {
    pub user: Principal,
    pub transaction_value_usd: u64,
    pub asset: Asset,
    pub operation_type: String, // "swap", "yield_farm", "arbitrage", etc.
}

pub struct FeeCollectionService;

impl FeeCollectionService {
    /// Initialize the fee collection service with pool canister ID
    pub fn initialize(pool_canister_id: Principal) {
        unsafe {
            POOL_CANISTER_ID = Some(pool_canister_id);
        }
    }

    /// Get the pool canister ID
    fn get_pool_canister_id() -> Result<Principal, String> {
        unsafe {
            POOL_CANISTER_ID.ok_or_else(|| "Pool canister ID not initialized".to_string())
        }
    }

    /// Calculate transaction fee based on user's subscription tier
    pub async fn calculate_fee(user: Principal, transaction_value_usd: u64) -> Result<u64, String> {
        // For now, use a direct call to get user info - this would need proper inter-canister call in production
        // Since we can't make an inter-canister call from here to user_management, we'll use default rates
        // In a production system, this would be passed from the calling context
        
        // Default to Standard tier (0.85%) - in production this would be retrieved from user data
        let fee_rate = SubscriptionTier::Standard.transaction_fee_rate();
        let fee_amount = ((transaction_value_usd as f64) * fee_rate) as u64;

        // Minimum fee of $0.01 to prevent zero fees on small transactions
        Ok(fee_amount.max(1))
    }

    /// Collect transaction fee and deposit to pool
    pub async fn collect_transaction_fee(
        request: TransactionFeeRequest
    ) -> Result<FeeCollectionResult, String> {
        let pool_canister_id = Self::get_pool_canister_id()?;
        
        // Calculate fee amount
        let fee_amount = Self::calculate_fee(request.user, request.transaction_value_usd).await?;
        
        if fee_amount == 0 {
            return Ok(FeeCollectionResult {
                success: true,
                fee_amount: 0,
                transaction_id: "no_fee".to_string(),
                error: None,
            });
        }

        // Generate transaction ID
        let transaction_id = format!("fee_{}_{}", request.user.to_text(), api::time());

        // Call pool canister to deposit fee
        let call_result: Result<(Result<String, String>,), _> = ic_cdk::call(
            pool_canister_id,
            "deposit_fee",
            (request.asset, fee_amount, transaction_id.clone(), request.user),
        ).await;

        match call_result {
            Ok((Ok(receipt),)) => {
                ic_cdk::println!(
                    "Fee collected successfully: User={}, Amount=${}, TxId={}, Receipt={}", 
                    request.user.to_text(), 
                    fee_amount, 
                    transaction_id,
                    receipt
                );
                
                Ok(FeeCollectionResult {
                    success: true,
                    fee_amount,
                    transaction_id,
                    error: None,
                })
            }
            Ok((Err(pool_error),)) => {
                let error_msg = format!("Pool deposit failed: {}", pool_error);
                ic_cdk::println!("Fee collection error: {}", error_msg);
                
                Ok(FeeCollectionResult {
                    success: false,
                    fee_amount,
                    transaction_id,
                    error: Some(error_msg),
                })
            }
            Err(call_error) => {
                let error_msg = format!("Inter-canister call failed: {:?}", call_error);
                ic_cdk::println!("Fee collection call error: {}", error_msg);
                
                Err(error_msg)
            }
        }
    }

    /// Get user's current fee rate for display
    pub async fn get_user_fee_rate(user: Principal) -> Result<f64, String> {
        // Default to Standard tier rate - in production this would query user data
        Ok(SubscriptionTier::Standard.transaction_fee_rate())
    }

    /// Estimate fee for a transaction (for UI display)
    pub async fn estimate_fee(
        user: Principal, 
        transaction_value_usd: u64
    ) -> Result<(u64, f64), String> {
        let fee_amount = Self::calculate_fee(user, transaction_value_usd).await?;
        let fee_rate = Self::get_user_fee_rate(user).await?;
        
        Ok((fee_amount, fee_rate * 100.0)) // Return amount and percentage
    }

    /// Collect fee with automatic retry logic
    pub async fn collect_fee_with_retry(
        request: TransactionFeeRequest,
        max_retries: u32
    ) -> Result<FeeCollectionResult, String> {
        let mut last_error = String::new();
        
        for attempt in 1..=max_retries {
            match Self::collect_transaction_fee(request.clone()).await {
                Ok(result) if result.success => return Ok(result),
                Ok(result) => {
                    last_error = result.error.unwrap_or("Unknown error".to_string());
                    if attempt < max_retries {
                        // Wait before retry (exponential backoff)
                        let delay_ms = 1000 * (2_u64.pow(attempt - 1));
                        ic_cdk::println!("Fee collection attempt {} failed, retrying in {}ms", attempt, delay_ms);
                        // Note: In a real implementation, you'd want to use a proper delay mechanism
                    }
                }
                Err(e) => {
                    last_error = e;
                    if attempt < max_retries {
                        ic_cdk::println!("Fee collection attempt {} failed: {}", attempt, last_error);
                    }
                }
            }
        }
        
        Err(format!("Fee collection failed after {} attempts: {}", max_retries, last_error))
    }
}

// Public API functions

/// Initialize fee collection service (called during canister init)
pub fn initialize_fee_collection(pool_canister_id: Principal) {
    FeeCollectionService::initialize(pool_canister_id);
    ic_cdk::println!("Fee collection service initialized with pool canister: {}", pool_canister_id);
}

/// API function to collect transaction fee
#[ic_cdk::update]
pub async fn collect_transaction_fee(request: TransactionFeeRequest) -> Result<FeeCollectionResult, String> {
    // Validate caller permissions if needed
    let caller = ic_cdk::caller();
    
    // For now, allow any authenticated caller
    // In production, you might want to restrict this to specific canisters or users
    if caller == Principal::anonymous() {
        return Err("Anonymous calls not allowed for fee collection".to_string());
    }

    FeeCollectionService::collect_fee_with_retry(request, 3).await
}

/// API function to estimate transaction fee
#[ic_cdk::query]
pub async fn estimate_transaction_fee(
    user: Principal,
    transaction_value_usd: u64
) -> Result<(u64, f64), String> {
    FeeCollectionService::estimate_fee(user, transaction_value_usd).await
}

/// API function to get user's current fee rate
#[ic_cdk::query] 
pub async fn get_user_fee_rate(user: Principal) -> Result<f64, String> {
    FeeCollectionService::get_user_fee_rate(user).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;

    fn create_test_asset() -> Asset {
        Asset {
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            chain: crate::types::ChainId::Ethereum,
            contract_address: Some("0xA0b86a33E6411E6A3fc0c39E4e90C8C4Bb8eF5E8".to_string()),
            decimals: 6,
            is_native: false,
        }
    }

    #[tokio::test]
    async fn test_calculate_fee() {
        let user = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
        
        // Test with different transaction amounts
        // Note: This would need proper mock setup in real tests
        let result = FeeCollectionService::calculate_fee(user, 1000).await;
        // Would assert expected fee calculation based on user tier
    }

    #[test]
    fn test_fee_collection_request() {
        let user = Principal::from_text("rdmx6-jaaaa-aaaah-qcaiq-cai").unwrap();
        let request = TransactionFeeRequest {
            user,
            transaction_value_usd: 1000,
            asset: create_test_asset(),
            operation_type: "swap".to_string(),
        };
        
        assert_eq!(request.transaction_value_usd, 1000);
        assert_eq!(request.operation_type, "swap");
    }
}