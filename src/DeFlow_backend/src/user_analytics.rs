// User Analytics and Admin Dashboard Module
// Provides comprehensive user statistics and activity tracking

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;
use crate::types::{User, SubscriptionTier, UserSubscriptionInfo};
use crate::stable_user_storage;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserAnalytics {
    pub total_users: u64,
    pub active_users_30d: u64,
    pub new_users_7d: u64,
    pub users_by_tier: HashMap<String, u64>,
    pub total_workflows_created: u64,
    pub total_executions: u64,
    pub executions_last_24h: u64,
    pub average_workflows_per_user: f64,
    pub top_node_types: Vec<(String, u64)>,
    pub total_revenue_usd: f64,
    pub monthly_recurring_revenue: f64,
    pub user_retention_rate: f64,
    pub last_updated: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserDetail {
    pub principal_id: String,
    pub subscription_tier: SubscriptionTier,
    pub created_at: u64,
    pub last_activity: u64,
    pub days_since_registration: u64,
    pub total_workflows: u64,
    pub total_executions: u64,
    pub monthly_executions: u64,
    pub monthly_volume_usd: f64,
    pub total_volume_usd: f64,
    pub is_active: bool,
    pub payment_history_count: u64,
    pub total_paid_usd: f64,
    pub preferred_nodes: Vec<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct UserActivity {
    pub timestamp: u64,
    pub user_principal: String,
    pub activity_type: ActivityType,
    pub details: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ActivityType {
    UserRegistered,
    WorkflowCreated,
    WorkflowExecuted,
    SubscriptionUpgraded,
    SubscriptionDowngraded,
    PaymentProcessed,
    NodeExecuted,
    CredentialStored,
    APICallMade,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PlatformMetrics {
    pub daily_active_users: Vec<(String, u64)>, // (date, count)
    pub signup_trend: Vec<(String, u64)>,       // (date, count)
    pub execution_trend: Vec<(String, u64)>,    // (date, count)
    pub revenue_trend: Vec<(String, f64)>,      // (date, usd)
    pub tier_distribution: HashMap<String, f64>, // tier -> percentage
}

/// Get comprehensive analytics overview
pub fn get_analytics_overview() -> UserAnalytics {
    let current_time = ic_cdk::api::time();
    let all_users = get_all_users();

    let total_users = all_users.len() as u64;

    // Active users (last 30 days)
    let thirty_days_ago = current_time.saturating_sub(30 * 24 * 60 * 60 * 1_000_000_000);
    let active_users_30d = all_users.iter()
        .filter(|u| u.usage_stats.last_activity >= thirty_days_ago)
        .count() as u64;

    // New users (last 7 days)
    let seven_days_ago = current_time.saturating_sub(7 * 24 * 60 * 60 * 1_000_000_000);
    let new_users_7d = all_users.iter()
        .filter(|u| u.user.created_at >= seven_days_ago)
        .count() as u64;

    // Users by tier
    let mut users_by_tier = HashMap::new();
    for user in &all_users {
        let tier_name = format!("{:?}", user.user.subscription_tier);
        *users_by_tier.entry(tier_name).or_insert(0) += 1;
    }

    // Workflows and executions
    let total_workflows_created: u64 = all_users.iter()
        .map(|u| u.usage_stats.total_workflows_created as u64)
        .sum();

    let total_executions: u64 = all_users.iter()
        .map(|u| u.usage_stats.total_executions as u64)
        .sum();

    // TODO: Track executions in last 24h (requires activity tracking)
    let executions_last_24h = 0u64;

    let average_workflows_per_user = if total_users > 0 {
        total_workflows_created as f64 / total_users as f64
    } else {
        0.0
    };

    // Top node types
    let mut node_type_counts: HashMap<String, u64> = HashMap::new();
    for user in &all_users {
        for node_type in &user.usage_stats.preferred_node_types {
            *node_type_counts.entry(node_type.clone()).or_insert(0) += 1;
        }
    }
    let mut top_node_types: Vec<(String, u64)> = node_type_counts.into_iter().collect();
    top_node_types.sort_by(|a, b| b.1.cmp(&a.1));
    top_node_types.truncate(10);

    // Revenue calculations
    let total_revenue_usd: f64 = all_users.iter()
        .flat_map(|u| &u.payment_history)
        .map(|p| p.amount)
        .sum();

    let monthly_recurring_revenue: f64 = all_users.iter()
        .map(|u| u.user.subscription_tier.monthly_fee())
        .sum();

    // Retention rate (users active in last 30 days / total users)
    let user_retention_rate = if total_users > 0 {
        (active_users_30d as f64 / total_users as f64) * 100.0
    } else {
        0.0
    };

    UserAnalytics {
        total_users,
        active_users_30d,
        new_users_7d,
        users_by_tier,
        total_workflows_created,
        total_executions,
        executions_last_24h,
        average_workflows_per_user,
        top_node_types,
        total_revenue_usd,
        monthly_recurring_revenue,
        user_retention_rate,
        last_updated: current_time,
    }
}

/// Get detailed list of all users
pub fn get_all_user_details() -> Vec<UserDetail> {
    let current_time = ic_cdk::api::time();
    let all_users = get_all_users();

    all_users.iter().map(|user_info| {
        let days_since_registration = (current_time.saturating_sub(user_info.user.created_at))
            / (24 * 60 * 60 * 1_000_000_000);

        let total_paid_usd: f64 = user_info.payment_history.iter()
            .map(|p| p.amount)
            .sum();

        UserDetail {
            principal_id: user_info.user.principal_id.clone(),
            subscription_tier: user_info.user.subscription_tier.clone(),
            created_at: user_info.user.created_at,
            last_activity: user_info.usage_stats.last_activity,
            days_since_registration,
            total_workflows: user_info.usage_stats.total_workflows_created as u64,
            total_executions: user_info.usage_stats.total_executions as u64,
            monthly_executions: user_info.usage_stats.monthly_executions as u64,
            monthly_volume_usd: user_info.user.monthly_volume,
            total_volume_usd: user_info.user.total_volume,
            is_active: user_info.user.active,
            payment_history_count: user_info.payment_history.len() as u64,
            total_paid_usd,
            preferred_nodes: user_info.usage_stats.preferred_node_types.clone(),
        }
    }).collect()
}

/// Get users filtered by criteria
pub fn get_users_filtered(
    tier: Option<SubscriptionTier>,
    active_only: bool,
    min_executions: Option<u64>,
) -> Vec<UserDetail> {
    let all_details = get_all_user_details();

    all_details.into_iter().filter(|user| {
        // Filter by tier
        if let Some(ref filter_tier) = tier {
            if &user.subscription_tier != filter_tier {
                return false;
            }
        }

        // Filter by active status
        if active_only && !user.is_active {
            return false;
        }

        // Filter by minimum executions
        if let Some(min) = min_executions {
            if user.total_executions < min {
                return false;
            }
        }

        true
    }).collect()
}

/// Get user growth trend (last 30 days)
pub fn get_user_growth_trend() -> Vec<(String, u64)> {
    let current_time = ic_cdk::api::time();
    let all_users = get_all_users();

    let mut daily_signups: HashMap<String, u64> = HashMap::new();

    for user in all_users {
        // Convert timestamp to date string (YYYY-MM-DD)
        let days_ago = (current_time.saturating_sub(user.user.created_at)) / (24 * 60 * 60 * 1_000_000_000);
        if days_ago <= 30 {
            let date = format_date_from_days_ago(days_ago);
            *daily_signups.entry(date).or_insert(0) += 1;
        }
    }

    let mut trend: Vec<(String, u64)> = daily_signups.into_iter().collect();
    trend.sort_by(|a, b| a.0.cmp(&b.0));
    trend
}

/// Get revenue breakdown
pub fn get_revenue_breakdown() -> HashMap<String, f64> {
    let all_users = get_all_users();
    let mut breakdown = HashMap::new();

    for user in all_users {
        let tier_name = format!("{:?}", user.user.subscription_tier);
        let revenue = user.user.subscription_tier.monthly_fee();
        *breakdown.entry(tier_name).or_insert(0.0) += revenue;
    }

    breakdown
}

/// Get top users by activity
pub fn get_top_users(limit: usize, sort_by: String) -> Vec<UserDetail> {
    let mut users = get_all_user_details();

    match sort_by.as_str() {
        "executions" => users.sort_by(|a, b| b.total_executions.cmp(&a.total_executions)),
        "workflows" => users.sort_by(|a, b| b.total_workflows.cmp(&a.total_workflows)),
        "volume" => users.sort_by(|a, b| b.total_volume_usd.partial_cmp(&a.total_volume_usd).unwrap()),
        "revenue" => users.sort_by(|a, b| b.total_paid_usd.partial_cmp(&a.total_paid_usd).unwrap()),
        _ => users.sort_by(|a, b| b.last_activity.cmp(&a.last_activity)),
    }

    users.truncate(limit);
    users
}

/// Search users by principal ID (partial match)
pub fn search_users(query: String) -> Vec<UserDetail> {
    let all_users = get_all_user_details();
    let query_lower = query.to_lowercase();

    all_users.into_iter()
        .filter(|user| user.principal_id.to_lowercase().contains(&query_lower))
        .collect()
}

// Helper function to get all users from stable storage
fn get_all_users() -> Vec<UserSubscriptionInfo> {
    stable_user_storage::get_all_user_subscription_infos()
}

// Helper function to format date from days ago
fn format_date_from_days_ago(days_ago: u64) -> String {
    let current_time = ic_cdk::api::time();
    let target_time = current_time.saturating_sub(days_ago * 24 * 60 * 60 * 1_000_000_000);
    let seconds = target_time / 1_000_000_000;

    // Simple date formatting (YYYY-MM-DD)
    // This is a simplified version - in production use proper date library
    let days_since_epoch = seconds / (24 * 60 * 60);
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        let date = format_date_from_days_ago(0);
        assert!(!date.is_empty());
        assert!(date.contains("-"));
    }
}
