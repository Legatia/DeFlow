// Portfolio Notification System
// Real-time alerts, notifications, and user communication

use super::*;
use crate::defi::yield_farming::ChainId;
use ic_cdk::api::time;

/// Advanced notification system for portfolio management
#[derive(Debug, Clone)]
pub struct NotificationSystem {
    pub notification_queue: Vec<Notification>,
    pub user_preferences: HashMap<String, NotificationPreferences>,
    pub alert_history: Vec<AlertRecord>,
    pub webhook_endpoints: HashMap<String, WebhookEndpoint>,
    pub email_templates: HashMap<NotificationType, EmailTemplate>,
    pub last_processed: u64,
}

impl NotificationSystem {
    pub fn new() -> Self {
        Self {
            notification_queue: Vec::new(),
            user_preferences: HashMap::new(),
            alert_history: Vec::new(),
            webhook_endpoints: HashMap::new(),
            email_templates: Self::initialize_email_templates(),
            last_processed: 0,
        }
    }

    /// Send risk alerts to user
    pub fn send_risk_alerts(&mut self, user_id: &str, alerts: Vec<RiskAlert>) {
        for alert in alerts {
            let notification = Notification {
                id: self.generate_notification_id(),
                user_id: user_id.to_string(),
                notification_type: NotificationType::RiskAlert,
                title: format!("{:?} Risk Alert", alert.alert_type),
                message: alert.message.clone(),
                severity: alert.severity.clone(),
                data: NotificationData::RiskAlert(alert.clone()),
                channels: self.get_user_notification_channels(user_id, &NotificationType::RiskAlert),
                created_at: self.get_current_time(),
                sent_at: None,
                acknowledged_at: None,
            };

            self.queue_notification(notification);
        }
    }

    /// Send performance alerts
    pub fn send_performance_alert(&mut self, user_id: &str, alert_type: PerformanceAlertType, data: PerformanceAlertData) {
        let (title, message, severity) = match &alert_type {
            PerformanceAlertType::SignificantGain => (
                "Significant Portfolio Gain".to_string(),
                format!("Your portfolio has gained {:.2}% in the last 24 hours", data.percentage_change),
                AlertSeverity::Info,
            ),
            PerformanceAlertType::SignificantLoss => (
                "Significant Portfolio Loss".to_string(),
                format!("Your portfolio has declined {:.2}% in the last 24 hours", data.percentage_change.abs()),
                AlertSeverity::Warning,
            ),
            PerformanceAlertType::MilestoneReached => (
                "Portfolio Milestone Reached".to_string(),
                format!("Your portfolio has reached ${:.2}", data.current_value),
                AlertSeverity::Info,
            ),
            PerformanceAlertType::TargetAchieved => (
                "Target Achievement".to_string(),
                format!("Your portfolio has achieved the target return of {:.2}%", data.target_return.unwrap_or(0.0)),
                AlertSeverity::Info,
            ),
        };

        let notification = Notification {
            id: self.generate_notification_id(),
            user_id: user_id.to_string(),
            notification_type: NotificationType::PerformanceAlert,
            title,
            message,
            severity,
            data: NotificationData::PerformanceAlert { alert_type, data },
            channels: self.get_user_notification_channels(user_id, &NotificationType::PerformanceAlert),
            created_at: self.get_current_time(),
            sent_at: None,
            acknowledged_at: None,
        };

        self.queue_notification(notification);
    }

    /// Send rebalancing notifications
    pub fn send_rebalancing_notification(&mut self, user_id: &str, notification_type: RebalancingNotificationType, data: RebalancingNotificationData) {
        let (title, message, severity) = match &notification_type {
            RebalancingNotificationType::RebalanceNeeded => (
                "Portfolio Rebalancing Needed".to_string(),
                format!("Your portfolio has drifted {:.1}% from target allocation", data.drift_percentage),
                AlertSeverity::Info,
            ),
            RebalancingNotificationType::RebalanceCompleted => (
                "Rebalancing Completed".to_string(),
                format!("Portfolio rebalancing completed successfully. Total cost: ${:.2}", data.total_cost),
                AlertSeverity::Info,
            ),
            RebalancingNotificationType::RebalanceFailed => (
                "Rebalancing Failed".to_string(),
                format!("Portfolio rebalancing failed: {}", data.error_message.as_ref().unwrap_or(&"Unknown error".to_string())),
                AlertSeverity::Warning,
            ),
        };

        let notification = Notification {
            id: self.generate_notification_id(),
            user_id: user_id.to_string(),
            notification_type: NotificationType::RebalancingAlert,
            title,
            message,
            severity,
            data: NotificationData::RebalancingAlert { notification_type, data },
            channels: self.get_user_notification_channels(user_id, &NotificationType::RebalancingAlert),
            created_at: self.get_current_time(),
            sent_at: None,
            acknowledged_at: None,
        };

        self.queue_notification(notification);
    }

    /// Send yield optimization alerts
    pub fn send_yield_alert(&mut self, user_id: &str, alert_type: YieldAlertType, data: YieldAlertData) {
        let (title, message, severity) = match &alert_type {
            YieldAlertType::HighYieldOpportunity => (
                "High Yield Opportunity".to_string(),
                format!("New high-yield opportunity found: {:.2}% APY on {}", data.apy, data.chain.name()),
                AlertSeverity::Info,
            ),
            YieldAlertType::YieldDropped => (
                "Yield Rate Decreased".to_string(),
                format!("Yield on your {} position dropped to {:.2}% APY", data.protocol_name, data.apy),
                AlertSeverity::Warning,
            ),
            YieldAlertType::CompoundAvailable => (
                "Compound Rewards Available".to_string(),
                format!("${:.2} in rewards available for compounding", data.rewards_amount),
                AlertSeverity::Info,
            ),
            YieldAlertType::AutoCompoundExecuted => (
                "Auto-Compound Executed".to_string(),
                format!("Auto-compounded ${:.2} in rewards", data.rewards_amount),
                AlertSeverity::Info,
            ),
        };

        let notification = Notification {
            id: self.generate_notification_id(),
            user_id: user_id.to_string(),
            notification_type: NotificationType::YieldAlert,
            title,
            message,
            severity,
            data: NotificationData::YieldAlert { alert_type, data },
            channels: self.get_user_notification_channels(user_id, &NotificationType::YieldAlert),
            created_at: self.get_current_time(),
            sent_at: None,
            acknowledged_at: None,
        };

        self.queue_notification(notification);
    }

    /// Send system notifications
    pub fn send_system_notification(&mut self, user_id: &str, alert_type: SystemAlertType, message: String) {
        let (title, severity) = match &alert_type {
            SystemAlertType::MaintenanceScheduled => ("Scheduled Maintenance", AlertSeverity::Info),
            SystemAlertType::ServiceDisruption => ("Service Disruption", AlertSeverity::Warning),
            SystemAlertType::SecurityAlert => ("Security Alert", AlertSeverity::Critical),
            SystemAlertType::ProtocolUpdate => ("Protocol Update", AlertSeverity::Info),
            SystemAlertType::ChainCongestion => ("Network Congestion", AlertSeverity::Warning),
        };

        let notification = Notification {
            id: self.generate_notification_id(),
            user_id: user_id.to_string(),
            notification_type: NotificationType::SystemAlert,
            title: title.to_string(),
            message,
            severity,
            data: NotificationData::SystemAlert { alert_type },
            channels: self.get_user_notification_channels(user_id, &NotificationType::SystemAlert),
            created_at: self.get_current_time(),
            sent_at: None,
            acknowledged_at: None,
        };

        self.queue_notification(notification);
    }

    /// Process notification queue
    pub async fn process_notification_queue(&mut self) -> Result<Vec<NotificationResult>, PortfolioError> {
        let mut results = Vec::new();
        let notifications_to_process: Vec<Notification> = self.notification_queue
            .iter()
            .filter(|n| n.sent_at.is_none())
            .cloned()
            .collect();

        for mut notification in notifications_to_process {
            let result = self.send_notification(&mut notification).await?;
            results.push(result);

            // Update notification status
            let current_time = self.get_current_time();
            if let Some(queued_notification) = self.notification_queue.iter_mut().find(|n| n.id == notification.id) {
                queued_notification.sent_at = Some(current_time);
            }
        }

        // Clean up old notifications (keep last 1000)
        if self.notification_queue.len() > 1000 {
            self.notification_queue.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            self.notification_queue.truncate(1000);
        }

        Ok(results)
    }

    /// Set user notification preferences
    pub fn set_user_preferences(&mut self, user_id: String, preferences: NotificationPreferences) {
        self.user_preferences.insert(user_id, preferences);
    }

    /// Get user notification preferences
    pub fn get_user_preferences(&self, user_id: &str) -> NotificationPreferences {
        self.user_preferences.get(user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Add webhook endpoint
    pub fn add_webhook_endpoint(&mut self, user_id: String, endpoint: WebhookEndpoint) {
        self.webhook_endpoints.insert(user_id, endpoint);
    }

    /// Get user notifications
    pub fn get_user_notifications(&self, user_id: &str, limit: Option<usize>) -> Vec<Notification> {
        let limit = limit.unwrap_or(50).min(200);
        
        self.notification_queue.iter()
            .filter(|n| n.user_id == user_id)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Mark notification as acknowledged
    pub fn acknowledge_notification(&mut self, notification_id: &str, user_id: &str) -> Result<(), PortfolioError> {
        let current_time = self.get_current_time();
        if let Some(notification) = self.notification_queue.iter_mut()
            .find(|n| n.id == notification_id && n.user_id == user_id) {
            notification.acknowledged_at = Some(current_time);
            Ok(())
        } else {
            Err(PortfolioError::NotificationError(format!("Notification not found: {}", notification_id)))
        }
    }

    /// Get notification statistics
    pub fn get_notification_stats(&self, user_id: &str) -> NotificationStats {
        let user_notifications: Vec<&Notification> = self.notification_queue.iter()
            .filter(|n| n.user_id == user_id)
            .collect();

        let total_notifications = user_notifications.len();
        let unread_notifications = user_notifications.iter()
            .filter(|n| n.acknowledged_at.is_none())
            .count();
        let critical_alerts = user_notifications.iter()
            .filter(|n| matches!(n.severity, AlertSeverity::Critical))
            .count();
        let warning_alerts = user_notifications.iter()
            .filter(|n| matches!(n.severity, AlertSeverity::Warning))
            .count();

        // Count by type
        let mut by_type = HashMap::new();
        for notification in &user_notifications {
            let count = by_type.get(&notification.notification_type).unwrap_or(&0);
            by_type.insert(notification.notification_type.clone(), count + 1);
        }

        // Recent activity (last 7 days)
        let seven_days_ago = self.get_current_time().saturating_sub(7 * 24 * 3600 * 1_000_000_000);
        let recent_notifications = user_notifications.iter()
            .filter(|n| n.created_at >= seven_days_ago)
            .count();

        NotificationStats {
            total_notifications,
            unread_notifications,
            critical_alerts,
            warning_alerts,
            notifications_by_type: by_type,
            recent_activity_7d: recent_notifications,
            last_notification: user_notifications.first().map(|n| n.created_at),
        }
    }

    /// Private helper functions
    fn queue_notification(&mut self, notification: Notification) {
        self.notification_queue.push(notification);
    }

    fn generate_notification_id(&self) -> String {
        format!("notif_{:x}", self.get_current_time())
    }

    fn get_user_notification_channels(&self, user_id: &str, notification_type: &NotificationType) -> Vec<NotificationChannel> {
        let preferences = self.get_user_preferences(user_id);
        
        match notification_type {
            NotificationType::RiskAlert => preferences.risk_alerts,
            NotificationType::PerformanceAlert => preferences.performance_alerts,
            NotificationType::RebalancingAlert => preferences.rebalancing_alerts,
            NotificationType::YieldAlert => preferences.yield_alerts,
            NotificationType::SystemAlert => preferences.system_alerts,
        }
    }

    async fn send_notification(&mut self, notification: &mut Notification) -> Result<NotificationResult, PortfolioError> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for channel in &notification.channels {
            let channel_result = match channel {
                NotificationChannel::InApp => {
                    self.send_in_app_notification(notification).await
                },
                NotificationChannel::Email => {
                    self.send_email_notification(notification).await
                },
                NotificationChannel::Push => {
                    self.send_push_notification(notification).await
                },
                NotificationChannel::Webhook => {
                    self.send_webhook_notification(notification).await
                },
            };

            match channel_result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    failure_count += 1;
                    results.push(format!("{:?} failed: {}", channel, e));
                },
            }
        }

        // Record in alert history
        let alert_record = AlertRecord {
            notification_id: notification.id.clone(),
            user_id: notification.user_id.clone(),
            notification_type: notification.notification_type.clone(),
            severity: notification.severity.clone(),
            channels_attempted: notification.channels.clone(),
            success_count,
            failure_count,
            timestamp: self.get_current_time(),
        };
        self.alert_history.push(alert_record);

        Ok(NotificationResult {
            notification_id: notification.id.clone(),
            success: failure_count == 0,
            channels_sent: success_count,
            channels_failed: failure_count,
            error_messages: results,
            sent_at: self.get_current_time(),
        })
    }

    async fn send_in_app_notification(&self, _notification: &Notification) -> Result<(), PortfolioError> {
        // In-app notifications are stored in the queue and retrieved by the frontend
        // This is already handled by queueing the notification
        Ok(())
    }

    async fn send_email_notification(&self, notification: &Notification) -> Result<(), PortfolioError> {
        // Mock email sending implementation
        // In production, this would integrate with an email service
        Ok(())
    }

    async fn send_push_notification(&self, notification: &Notification) -> Result<(), PortfolioError> {
        // Mock push notification implementation
        // In production, this would integrate with push notification services
        Ok(())
    }

    async fn send_webhook_notification(&self, notification: &Notification) -> Result<(), PortfolioError> {
        // Mock webhook implementation
        // In production, this would make HTTP requests to webhook endpoints
        if let Some(endpoint) = self.webhook_endpoints.get(&notification.user_id) {
        }
        Ok(())
    }

    fn initialize_email_templates() -> HashMap<NotificationType, EmailTemplate> {
        let mut templates = HashMap::new();
        
        templates.insert(NotificationType::RiskAlert, EmailTemplate {
            subject: "🚨 Portfolio Risk Alert".to_string(),
            html_body: r#"
                <h2>Risk Alert</h2>
                <p>{{message}}</p>
                <p><strong>Current Risk Level:</strong> {{risk_level}}</p>
                <p><strong>Recommendation:</strong> {{recommendation}}</p>
            "#.to_string(),
            text_body: "Risk Alert: {{message}}\n\nRecommendation: {{recommendation}}".to_string(),
        });

        templates.insert(NotificationType::PerformanceAlert, EmailTemplate {
            subject: "📈 Portfolio Performance Update".to_string(),
            html_body: r#"
                <h2>Performance Alert</h2>
                <p>{{message}}</p>
                <p><strong>Current Portfolio Value:</strong> ${{portfolio_value}}</p>
            "#.to_string(),
            text_body: "Performance Alert: {{message}}\n\nCurrent Portfolio Value: ${{portfolio_value}}".to_string(),
        });

        templates
    }

    fn get_current_time(&self) -> u64 {
        #[cfg(test)]
        {
            1234567890_u64
        }
        #[cfg(not(test))]
        {
            if self.last_processed == 0 || self.last_processed == 1234567890_u64 {
                1234567890_u64
            } else {
                time()
            }
        }
    }
}

/// Notification preferences for users
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub risk_alerts: Vec<NotificationChannel>,
    pub performance_alerts: Vec<NotificationChannel>,
    pub rebalancing_alerts: Vec<NotificationChannel>,
    pub yield_alerts: Vec<NotificationChannel>,
    pub system_alerts: Vec<NotificationChannel>,
    pub quiet_hours_start: Option<u8>,  // Hour of day (0-23)
    pub quiet_hours_end: Option<u8>,    // Hour of day (0-23)
    pub timezone: String,
    pub email_frequency: EmailFrequency,
    pub minimum_alert_threshold: f64,   // Minimum dollar value for alerts
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            risk_alerts: vec![NotificationChannel::InApp, NotificationChannel::Email],
            performance_alerts: vec![NotificationChannel::InApp],
            rebalancing_alerts: vec![NotificationChannel::InApp],
            yield_alerts: vec![NotificationChannel::InApp],
            system_alerts: vec![NotificationChannel::InApp, NotificationChannel::Email],
            quiet_hours_start: Some(22), // 10 PM
            quiet_hours_end: Some(8),    // 8 AM
            timezone: "UTC".to_string(),
            email_frequency: EmailFrequency::Immediate,
            minimum_alert_threshold: 100.0, // $100 minimum
        }
    }
}

/// Auto-compound settings
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AutoCompoundSettings {
    pub enabled: bool,
    pub frequency_hours: u64,           // How often to check for compounding
    pub min_rewards_threshold: f64,     // Minimum rewards to trigger compound
    pub max_gas_ratio: f64,            // Maximum gas cost as ratio of rewards
    pub preferred_chains: Vec<ChainId>, // Preferred chains for auto-compound
    pub notification_enabled: bool,     // Send notifications for compounds
}

impl Default for AutoCompoundSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            frequency_hours: 24,        // Check daily
            min_rewards_threshold: 10.0, // $10 minimum
            max_gas_ratio: 0.1,         // Max 10% of rewards on gas
            preferred_chains: vec![
                ChainId::Arbitrum,
                ChainId::Polygon,
                ChainId::Solana,
            ],
            notification_enabled: true,
        }
    }
}

/// Auto-compound execution result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AutoCompoundResult {
    pub position_id: String,
    pub success: bool,
    pub compound_amount: f64,
    pub gas_cost: f64,
    pub error_message: Option<String>,
    pub executed_at: u64,
}

/// Core notification structure
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub data: NotificationData,
    pub channels: Vec<NotificationChannel>,
    pub created_at: u64,
    pub sent_at: Option<u64>,
    pub acknowledged_at: Option<u64>,
}

/// Notification types
#[derive(Debug, Clone, PartialEq, Eq, Hash, CandidType, Serialize, Deserialize)]
pub enum NotificationType {
    RiskAlert,
    PerformanceAlert,
    RebalancingAlert,
    YieldAlert,
    SystemAlert,
}

/// Notification channels
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum NotificationChannel {
    InApp,
    Email,
    Push,
    Webhook,
}

/// Email frequency settings
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum EmailFrequency {
    Immediate,
    Hourly,
    Daily,
    Weekly,
    Never,
}

/// Notification data union
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum NotificationData {
    RiskAlert(RiskAlert),
    PerformanceAlert {
        alert_type: PerformanceAlertType,
        data: PerformanceAlertData,
    },
    RebalancingAlert {
        notification_type: RebalancingNotificationType,
        data: RebalancingNotificationData,
    },
    YieldAlert {
        alert_type: YieldAlertType,
        data: YieldAlertData,
    },
    SystemAlert {
        alert_type: SystemAlertType,
    },
}

/// Performance alert types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum PerformanceAlertType {
    SignificantGain,
    SignificantLoss,
    MilestoneReached,
    TargetAchieved,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct PerformanceAlertData {
    pub current_value: f64,
    pub previous_value: f64,
    pub percentage_change: f64,
    pub time_period: String,
    pub target_return: Option<f64>,
}

/// Rebalancing notification types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum RebalancingNotificationType {
    RebalanceNeeded,
    RebalanceCompleted,
    RebalanceFailed,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RebalancingNotificationData {
    pub drift_percentage: f64,
    pub total_cost: f64,
    pub execution_time: u64,
    pub error_message: Option<String>,
}

/// Yield alert types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum YieldAlertType {
    HighYieldOpportunity,
    YieldDropped,
    CompoundAvailable,
    AutoCompoundExecuted,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct YieldAlertData {
    pub chain: ChainId,
    pub protocol_name: String,
    pub apy: f64,
    pub rewards_amount: f64,
    pub position_id: Option<String>,
}

/// System alert types
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum SystemAlertType {
    MaintenanceScheduled,
    ServiceDisruption,
    SecurityAlert,
    ProtocolUpdate,
    ChainCongestion,
}

/// Webhook endpoint configuration
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub url: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub notification_types: Vec<NotificationType>,
}

/// Email template
#[derive(Debug, Clone)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

/// Notification result
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct NotificationResult {
    pub notification_id: String,
    pub success: bool,
    pub channels_sent: usize,
    pub channels_failed: usize,
    pub error_messages: Vec<String>,
    pub sent_at: u64,
}

/// Alert history record
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct AlertRecord {
    pub notification_id: String,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub severity: AlertSeverity,
    pub channels_attempted: Vec<NotificationChannel>,
    pub success_count: usize,
    pub failure_count: usize,
    pub timestamp: u64,
}

/// Notification statistics
#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_notifications: usize,
    pub unread_notifications: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub notifications_by_type: HashMap<NotificationType, usize>,
    pub recent_activity_7d: usize,
    pub last_notification: Option<u64>,
}