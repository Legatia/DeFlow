use crate::types::{User, SubscriptionTier, UserSubscriptionInfo, PaymentRecord, UsageStats, PaymentStatus};
use ic_cdk::{query, update, caller, api};
use std::collections::HashMap;
use candid::{CandidType, Deserialize, Principal};

// Storage for user data
thread_local! {
    static USERS: std::cell::RefCell<HashMap<String, UserSubscriptionInfo>> = std::cell::RefCell::new(HashMap::new());
}

#[update]
pub fn register_user() -> Result<User, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Check if user already exists
    let existing_user = USERS.with(|users| {
        users.borrow().get(&principal_id).cloned()
    });
    
    if existing_user.is_some() {
        return Err("User already registered".to_string());
    }
    
    let current_time = api::time();
    
    let user = User {
        principal_id: principal_id.clone(),
        subscription_tier: SubscriptionTier::Standard, // Start with Standard (free) tier
        created_at: current_time,
        updated_at: current_time,
        monthly_volume: 0.0,
        total_volume: 0.0,
        active: true,
    };
    
    let user_info = UserSubscriptionInfo {
        user: user.clone(),
        payment_history: Vec::new(),
        usage_stats: UsageStats {
            total_workflows_created: 0,
            total_executions: 0,
            monthly_executions: 0,
            last_activity: current_time,
            preferred_node_types: Vec::new(),
        },
    };
    
    USERS.with(|users| {
        users.borrow_mut().insert(principal_id, user_info);
    });
    
    Ok(user)
}

#[query]
pub fn get_user_info() -> Result<UserSubscriptionInfo, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    USERS.with(|users| {
        users.borrow().get(&principal_id).cloned()
            .ok_or_else(|| "User not found. Please register first.".to_string())
    })
}

#[update]
pub fn upgrade_subscription(new_tier: SubscriptionTier) -> Result<User, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    USERS.with(|users| {
        let mut users_map = users.borrow_mut();
        if let Some(user_info) = users_map.get_mut(&principal_id) {
            let current_time = api::time();
            
            // Update user's subscription tier
            user_info.user.subscription_tier = new_tier.clone();
            user_info.user.updated_at = current_time;
            
            // Add payment record (in a real implementation, this would integrate with payment processing)
            let payment_record = PaymentRecord {
                id: format!("payment_{}", current_time),
                amount: new_tier.monthly_fee(),
                currency: "USD".to_string(),
                payment_date: current_time,
                subscription_tier: new_tier,
                status: PaymentStatus::Completed, // Simplified for demo
            };
            
            user_info.payment_history.push(payment_record);
            
            Ok(user_info.user.clone())
        } else {
            Err("User not found. Please register first.".to_string())
        }
    })
}

#[query]
pub fn check_node_access(node_type: String) -> Result<bool, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    USERS.with(|users| {
        if let Some(user_info) = users.borrow().get(&principal_id) {
            let allowed_nodes = user_info.user.subscription_tier.allowed_node_types();
            Ok(allowed_nodes.contains(&node_type))
        } else {
            Err("User not found. Please register first.".to_string())
        }
    })
}

#[query]
pub fn get_allowed_node_types() -> Result<Vec<String>, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    USERS.with(|users| {
        if let Some(user_info) = users.borrow().get(&principal_id) {
            Ok(user_info.user.subscription_tier.allowed_node_types())
        } else {
            Err("User not found. Please register first.".to_string())
        }
    })
}

#[update]
pub fn record_workflow_execution(workflow_id: String, node_types: Vec<String>) -> Result<(), String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    USERS.with(|users| {
        let mut users_map = users.borrow_mut();
        if let Some(user_info) = users_map.get_mut(&principal_id) {
            let current_time = api::time();
            
            // Update usage stats
            user_info.usage_stats.total_executions += 1;
            user_info.usage_stats.monthly_executions += 1;
            user_info.usage_stats.last_activity = current_time;
            
            // Track preferred node types
            for node_type in node_types {
                if !user_info.usage_stats.preferred_node_types.contains(&node_type) {
                    user_info.usage_stats.preferred_node_types.push(node_type);
                }
            }
            
            user_info.user.updated_at = current_time;
            
            Ok(())
        } else {
            Err("User not found. Please register first.".to_string())
        }
    })
}

#[update]
pub fn update_user_volume(volume_change: f64) -> Result<User, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    USERS.with(|users| {
        let mut users_map = users.borrow_mut();
        if let Some(user_info) = users_map.get_mut(&principal_id) {
            let current_time = api::time();
            
            user_info.user.monthly_volume += volume_change;
            user_info.user.total_volume += volume_change;
            user_info.user.updated_at = current_time;
            
            Ok(user_info.user.clone())
        } else {
            Err("User not found. Please register first.".to_string())
        }
    })
}

#[query]
pub fn get_subscription_pricing() -> Vec<(SubscriptionTier, f64, f64, Vec<String>)> {
    vec![
        (
            SubscriptionTier::Standard,
            SubscriptionTier::Standard.monthly_fee(),
            SubscriptionTier::Standard.transaction_fee_rate(),
            SubscriptionTier::Standard.allowed_node_types(),
        ),
        (
            SubscriptionTier::Premium,
            SubscriptionTier::Premium.monthly_fee(),
            SubscriptionTier::Premium.transaction_fee_rate(),
            SubscriptionTier::Premium.allowed_node_types(),
        ),
        (
            SubscriptionTier::Pro,
            SubscriptionTier::Pro.monthly_fee(),
            SubscriptionTier::Pro.transaction_fee_rate(),
            SubscriptionTier::Pro.allowed_node_types(),
        ),
    ]
}

// Admin functions (in a real implementation, these would have proper access control)
#[query]
pub fn list_all_users() -> Vec<UserSubscriptionInfo> {
    USERS.with(|users| {
        users.borrow().values().cloned().collect()
    })
}

#[update]
pub fn reset_monthly_stats() -> Result<(), String> {
    USERS.with(|users| {
        let mut users_map = users.borrow_mut();
        for (_, user_info) in users_map.iter_mut() {
            user_info.user.monthly_volume = 0.0;
            user_info.usage_stats.monthly_executions = 0;
        }
    });
    Ok(())
}