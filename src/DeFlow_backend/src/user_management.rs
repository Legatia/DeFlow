use crate::types::{
    User, SubscriptionTier, UserSubscriptionInfo, PaymentRecord, UsageStats, PaymentStatus, 
    UserSettings, UserPreferences, NotificationSettings, UISettings,
    OAuthToken, IntegrationCredentials, EncryptedCredentials, APIConnection, WorkflowTemplate
};
use crate::stable_user_storage::{
    get_user_profile, insert_user_profile,
    get_user_subscription_info, insert_user_subscription_info,
    get_user_settings, insert_user_settings,
    get_oauth_token, insert_oauth_token,
    get_user_integration_credentials, insert_user_integration_credentials,
    get_api_connections, insert_api_connection,
    get_user_templates, get_public_templates, insert_template
};
use ic_cdk::{query, update, caller, api};
use candid::{CandidType, Deserialize, Principal};

#[update]
pub fn register_user() -> Result<User, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Check if user already exists in stable storage
    if let Some(_existing_user) = get_user_profile(&principal_id) {
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

    // Create default user settings
    let user_settings = UserSettings {
        user_principal: principal_id.clone(),
        preferences: UserPreferences::default(),
        notification_settings: NotificationSettings::default(),
        ui_settings: UISettings::default(),
        created_at: current_time,
        updated_at: current_time,
    };
    
    // Store in stable memory
    insert_user_profile(principal_id.clone(), user.clone());
    insert_user_subscription_info(principal_id.clone(), user_info);
    insert_user_settings(principal_id, user_settings);
    
    Ok(user)
}

#[query]
pub fn get_user_info() -> Result<UserSubscriptionInfo, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    get_user_subscription_info(&principal_id)
        .ok_or_else(|| "User not found. Please register first.".to_string())
}

#[update]
pub fn upgrade_subscription(new_tier: SubscriptionTier) -> Result<User, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Get current user info from stable storage
    let mut user_info = get_user_subscription_info(&principal_id)
        .ok_or_else(|| "User not found. Please register first.".to_string())?;
    
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
    
    // Update both user profile and subscription info in stable storage
    insert_user_profile(principal_id.clone(), user_info.user.clone());
    insert_user_subscription_info(principal_id, user_info.clone());
    
    Ok(user_info.user)
}

#[query]
pub fn check_node_access(node_type: String) -> Result<bool, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    if let Some(user_info) = get_user_subscription_info(&principal_id) {
        let allowed_nodes = user_info.user.subscription_tier.allowed_node_types();
        Ok(allowed_nodes.contains(&node_type))
    } else {
        Err("User not found. Please register first.".to_string())
    }
}

#[query]
pub fn get_allowed_node_types() -> Result<Vec<String>, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    if let Some(user_info) = get_user_subscription_info(&principal_id) {
        Ok(user_info.user.subscription_tier.allowed_node_types())
    } else {
        Err("User not found. Please register first.".to_string())
    }
}

#[update]
pub fn record_workflow_execution(workflow_id: String, node_types: Vec<String>) -> Result<(), String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    if let Some(mut user_info) = get_user_subscription_info(&principal_id) {
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
        
        // Update in stable storage
        insert_user_profile(principal_id.clone(), user_info.user.clone());
        insert_user_subscription_info(principal_id, user_info);
        
        Ok(())
    } else {
        Err("User not found. Please register first.".to_string())
    }
}

#[update]
pub fn update_user_volume(volume_change: f64) -> Result<User, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    if let Some(mut user_info) = get_user_subscription_info(&principal_id) {
        let current_time = api::time();
        
        user_info.user.monthly_volume += volume_change;
        user_info.user.total_volume += volume_change;
        user_info.user.updated_at = current_time;
        
        // Update in stable storage
        insert_user_profile(principal_id.clone(), user_info.user.clone());
        insert_user_subscription_info(principal_id, user_info.clone());
        
        Ok(user_info.user)
    } else {
        Err("User not found. Please register first.".to_string())
    }
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
    // Note: This function would need to iterate through all users in stable storage
    // For now, this is a placeholder that returns empty vec
    // In production, you'd implement proper iteration through stable memory
    Vec::new()
}

#[update]
pub fn reset_monthly_stats() -> Result<(), String> {
    // Note: This would need to iterate through all users in stable storage
    // For now, this is a placeholder - in production, you'd implement
    // a background job to reset monthly stats for all users
    Ok(())
}

// ===== USER SETTINGS API =====

#[query]
pub fn get_user_settings_api() -> Result<UserSettings, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    get_user_settings(&principal_id)
        .ok_or_else(|| "User settings not found. Please register first.".to_string())
}

#[update]
pub fn update_user_settings(settings: UserSettings) -> Result<UserSettings, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Verify user exists
    if get_user_profile(&principal_id).is_none() {
        return Err("User not found. Please register first.".to_string());
    }
    
    let current_time = api::time();
    let mut updated_settings = settings;
    updated_settings.user_principal = principal_id.clone();
    updated_settings.updated_at = current_time;
    
    insert_user_settings(principal_id, updated_settings.clone());
    Ok(updated_settings)
}

// ===== INTEGRATION CREDENTIALS API =====

#[update]
pub fn save_oauth_token(platform: String, access_token: String, refresh_token: Option<String>, expires_at: u64, scopes: Vec<String>) -> Result<(), String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Verify user exists
    if get_user_profile(&principal_id).is_none() {
        return Err("User not found. Please register first.".to_string());
    }
    
    let token = OAuthToken {
        user_principal: principal_id.clone(),
        platform: platform.clone(),
        access_token, // In production, this should be encrypted
        refresh_token, // In production, this should be encrypted
        expires_at,
        scopes,
    };
    
    insert_oauth_token(principal_id, platform, token);
    Ok(())
}

#[query]
pub fn get_oauth_token_api(platform: String) -> Result<OAuthToken, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    get_oauth_token(&principal_id, &platform)
        .ok_or_else(|| format!("OAuth token for {} not found", platform))
}

#[update]
pub fn save_integration_credentials(integration_type: String, encrypted_data: Vec<u8>, key_id: String) -> Result<(), String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Verify user exists
    if get_user_profile(&principal_id).is_none() {
        return Err("User not found. Please register first.".to_string());
    }
    
    let current_time = api::time();
    let credentials = IntegrationCredentials {
        user_principal: principal_id.clone(),
        integration_type: integration_type.clone(),
        credentials: EncryptedCredentials {
            encrypted_data,
            key_id,
        },
        created_at: current_time,
        last_used: current_time,
        active: true,
    };
    
    insert_user_integration_credentials(principal_id, integration_type, credentials);
    Ok(())
}

#[query]
pub fn get_integration_credentials_api(integration_type: String) -> Result<IntegrationCredentials, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    get_user_integration_credentials(&principal_id, &integration_type)
        .ok_or_else(|| format!("Integration credentials for {} not found", integration_type))
}

// ===== API CONNECTIONS API =====

#[update]
pub fn save_api_connection(connection_name: String, api_type: String, configuration: String) -> Result<String, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Verify user exists
    if get_user_profile(&principal_id).is_none() {
        return Err("User not found. Please register first.".to_string());
    }
    
    let current_time = api::time();
    let connection_id = format!("conn_{}_{}", principal_id, current_time);
    
    let connection = APIConnection {
        user_principal: principal_id.clone(),
        connection_id: connection_id.clone(),
        connection_name,
        api_type,
        configuration, // In production, this should be encrypted
        created_at: current_time,
        last_tested: current_time,
        status: "active".to_string(),
    };
    
    insert_api_connection(connection_id.clone(), connection);
    Ok(connection_id)
}

#[query]
pub fn get_user_api_connections() -> Result<Vec<APIConnection>, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    Ok(get_api_connections(&principal_id))
}

// ===== TEMPLATE API =====

#[update]
pub fn create_workflow_template(name: String, description: String, category: String, workflow_data: String, is_public: bool, tags: Vec<String>) -> Result<String, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    // Verify user exists
    if get_user_profile(&principal_id).is_none() {
        return Err("User not found. Please register first.".to_string());
    }
    
    let current_time = api::time();
    let template_id = format!("template_{}_{}", principal_id, current_time);
    
    let template = WorkflowTemplate {
        template_id: template_id.clone(),
        creator_principal: principal_id,
        name,
        description,
        category,
        workflow_data,
        usage_count: 0,
        rating: 0.0,
        created_at: current_time,
        updated_at: current_time,
        is_public,
        tags,
    };
    
    insert_template(template_id.clone(), template);
    Ok(template_id)
}

#[query]
pub fn get_user_templates_api() -> Result<Vec<WorkflowTemplate>, String> {
    let principal = caller();
    let principal_id = principal.to_text();
    
    Ok(get_user_templates(&principal_id))
}

#[query]
pub fn get_public_templates_api() -> Result<Vec<WorkflowTemplate>, String> {
    Ok(get_public_templates())
}