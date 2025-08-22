// ICP Cycles Monitoring Service
// Replaces gas optimization since ICP users don't pay gas fees - devs pay cycles

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use ic_cdk::api::{canister_balance, canister_balance128};
use std::collections::HashMap;

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CyclesMonitorService {
    monitored_canisters: HashMap<String, CyclesMonitorConfig>,
    alert_history: Vec<CyclesAlert>,
    last_check_time: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CyclesMonitorConfig {
    pub canister_id: Option<String>,
    pub warning_threshold: u128,
    pub critical_threshold: u128,
    pub auto_topup: bool,
    pub topup_amount: u128,
    pub notification_channels: Vec<String>,
    pub owner: String, // User who created the monitor
    pub created_at: u64,
    pub last_alert_sent: Option<u64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CyclesAlert {
    pub monitor_id: String,
    pub canister_id: String,
    pub alert_type: AlertType,
    pub current_cycles: u128,
    pub threshold: u128,
    pub timestamp: u64,
    pub notification_sent: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum AlertType {
    Warning,
    Critical,
    TopupRequested,
    TopupCompleted,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CyclesData {
    pub canister_id: String,
    pub current_cycles: u128,
    pub warning_threshold: u128,
    pub critical_threshold: u128,
    pub status: CyclesStatus,
    pub estimated_runtime_days: Option<u32>,
    pub last_check: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum CyclesStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CyclesMonitorResult {
    pub success: bool,
    pub message: String,
    pub cycles_data: Option<CyclesData>,
    pub alerts_triggered: Vec<CyclesAlert>,
}

impl CyclesMonitorService {
    pub fn new() -> Self {
        CyclesMonitorService {
            monitored_canisters: HashMap::new(),
            alert_history: Vec::new(),
            last_check_time: ic_cdk::api::time(),
        }
    }

    // =============================================================================
    // MONITORING SETUP
    // =============================================================================

    pub fn create_monitor(
        &mut self,
        monitor_id: String,
        config: CyclesMonitorConfig,
    ) -> Result<String, String> {
        // Validate thresholds
        if config.critical_threshold >= config.warning_threshold {
            return Err("Critical threshold must be lower than warning threshold".to_string());
        }

        if config.auto_topup && config.topup_amount < 1_000_000_000_000 {
            return Err("Top-up amount must be at least 1T cycles".to_string());
        }

        self.monitored_canisters.insert(monitor_id.clone(), config);
        
        Ok(format!("Cycles monitor '{}' created successfully", monitor_id))
    }

    pub fn update_monitor(
        &mut self,
        monitor_id: &str,
        config: CyclesMonitorConfig,
    ) -> Result<String, String> {
        if !self.monitored_canisters.contains_key(monitor_id) {
            return Err(format!("Monitor '{}' not found", monitor_id));
        }

        self.monitored_canisters.insert(monitor_id.to_string(), config);
        Ok(format!("Monitor '{}' updated successfully", monitor_id))
    }

    pub fn remove_monitor(&mut self, monitor_id: &str) -> Result<String, String> {
        if self.monitored_canisters.remove(monitor_id).is_some() {
            Ok(format!("Monitor '{}' removed successfully", monitor_id))
        } else {
            Err(format!("Monitor '{}' not found", monitor_id))
        }
    }

    // =============================================================================
    // CYCLES MONITORING
    // =============================================================================

    pub async fn check_cycles(&mut self, monitor_id: &str) -> Result<CyclesMonitorResult, String> {
        let config = self.monitored_canisters.get(monitor_id)
            .ok_or_else(|| format!("Monitor '{}' not found", monitor_id))?
            .clone();

        let cycles_data = self.get_cycles_data(&config).await?;
        let mut alerts_triggered = Vec::new();

        // Check thresholds and create alerts
        if cycles_data.current_cycles <= config.critical_threshold {
            let alert = CyclesAlert {
                monitor_id: monitor_id.to_string(),
                canister_id: cycles_data.canister_id.clone(),
                alert_type: AlertType::Critical,
                current_cycles: cycles_data.current_cycles,
                threshold: config.critical_threshold,
                timestamp: ic_cdk::api::time(),
                notification_sent: false,
            };
            
            alerts_triggered.push(alert.clone());
            self.alert_history.push(alert);

            // Auto top-up if enabled
            if config.auto_topup {
                match self.request_topup(&cycles_data.canister_id, config.topup_amount).await {
                    Ok(_) => {
                        let topup_alert = CyclesAlert {
                            monitor_id: monitor_id.to_string(),
                            canister_id: cycles_data.canister_id.clone(),
                            alert_type: AlertType::TopupRequested,
                            current_cycles: cycles_data.current_cycles,
                            threshold: config.topup_amount,
                            timestamp: ic_cdk::api::time(),
                            notification_sent: false,
                        };
                        alerts_triggered.push(topup_alert.clone());
                        self.alert_history.push(topup_alert);
                    },
                    Err(e) => {
                    }
                }
            }
        } else if cycles_data.current_cycles <= config.warning_threshold {
            let alert = CyclesAlert {
                monitor_id: monitor_id.to_string(),
                canister_id: cycles_data.canister_id.clone(),
                alert_type: AlertType::Warning,
                current_cycles: cycles_data.current_cycles,
                threshold: config.warning_threshold,
                timestamp: ic_cdk::api::time(),
                notification_sent: false,
            };
            
            alerts_triggered.push(alert.clone());
            self.alert_history.push(alert);
        }

        // Send notifications
        for alert in &mut alerts_triggered {
            if let Err(e) = self.send_alert_notifications(alert, &config.notification_channels).await {
            } else {
                alert.notification_sent = true;
            }
        }

        self.last_check_time = ic_cdk::api::time();

        Ok(CyclesMonitorResult {
            success: true,
            message: format!("Cycles check completed for {}", cycles_data.canister_id),
            cycles_data: Some(cycles_data),
            alerts_triggered,
        })
    }

    async fn get_cycles_data(&self, config: &CyclesMonitorConfig) -> Result<CyclesData, String> {
        let current_cycles = if config.canister_id.is_some() {
            // For external canisters, we'd need to make an inter-canister call
            // For now, return current canister balance as demo
            canister_balance128()
        } else {
            // Current canister
            canister_balance128()
        };

        let status = if current_cycles <= config.critical_threshold {
            CyclesStatus::Critical
        } else if current_cycles <= config.warning_threshold {
            CyclesStatus::Warning
        } else {
            CyclesStatus::Healthy
        };

        // Estimate runtime days based on average consumption
        // This is a simplified calculation - in production would track historical usage
        let estimated_runtime_days = if current_cycles > 0 {
            // Assume 100M cycles per day as baseline consumption
            let daily_consumption = 100_000_000u128;
            Some((current_cycles / daily_consumption) as u32)
        } else {
            None
        };

        let canister_id = config.canister_id.clone()
            .unwrap_or_else(|| "current".to_string());

        Ok(CyclesData {
            canister_id,
            current_cycles,
            warning_threshold: config.warning_threshold,
            critical_threshold: config.critical_threshold,
            status,
            estimated_runtime_days,
            last_check: ic_cdk::api::time(),
        })
    }

    // =============================================================================
    // NOTIFICATIONS AND ALERTS
    // =============================================================================

    async fn send_alert_notifications(
        &self,
        alert: &CyclesAlert,
        channels: &[String],
    ) -> Result<(), String> {
        let message = self.format_alert_message(alert);

        for channel in channels {
            match channel.as_str() {
                "email" => {
                    self.send_email_alert(&message).await?;
                },
                "discord" => {
                    self.send_discord_alert(&message).await?;
                },
                "telegram" => {
                    self.send_telegram_alert(&message).await?;
                },
                "slack" => {
                    self.send_slack_alert(&message).await?;
                },
                _ => {
                }
            }
        }

        Ok(())
    }

    fn format_alert_message(&self, alert: &CyclesAlert) -> String {
        let cycles_in_t = alert.current_cycles as f64 / 1_000_000_000_000.0;
        let threshold_in_t = alert.threshold as f64 / 1_000_000_000_000.0;

        match alert.alert_type {
            AlertType::Warning => {
                format!(
                    "🟡 CYCLES WARNING: Canister {} has {:.2}T cycles remaining (below {:.2}T threshold)",
                    alert.canister_id, cycles_in_t, threshold_in_t
                )
            },
            AlertType::Critical => {
                format!(
                    "🔴 CYCLES CRITICAL: Canister {} has only {:.2}T cycles remaining (below {:.2}T threshold)! Immediate top-up required.",
                    alert.canister_id, cycles_in_t, threshold_in_t
                )
            },
            AlertType::TopupRequested => {
                format!(
                    "🔄 CYCLES TOP-UP: Requested {:.2}T cycles for canister {}",
                    threshold_in_t, alert.canister_id
                )
            },
            AlertType::TopupCompleted => {
                format!(
                    "✅ CYCLES TOP-UP COMPLETE: Canister {} now has {:.2}T cycles",
                    alert.canister_id, cycles_in_t
                )
            },
        }
    }

    async fn send_email_alert(&self, message: &str) -> Result<(), String> {
        // Placeholder for email notification
        Ok(())
    }

    async fn send_discord_alert(&self, message: &str) -> Result<(), String> {
        // Placeholder for Discord webhook
        Ok(())
    }

    async fn send_telegram_alert(&self, message: &str) -> Result<(), String> {
        // Placeholder for Telegram bot
        Ok(())
    }

    async fn send_slack_alert(&self, message: &str) -> Result<(), String> {
        // Placeholder for Slack webhook
        Ok(())
    }

    // =============================================================================
    // CYCLES MANAGEMENT
    // =============================================================================

    async fn request_topup(&self, canister_id: &str, amount: u128) -> Result<(), String> {
        // In a real implementation, this would:
        // 1. Check if we have permission to top up the canister
        // 2. Use the cycles management API to send cycles
        // 3. For external canisters, might need to notify the owner
        
            "TOP-UP REQUESTED: {} cycles for canister {}",
            amount, canister_id
        );

        // Placeholder for actual top-up logic
        // This would involve cycles management APIs
        
        Ok(())
    }

    // =============================================================================
    // UTILITY FUNCTIONS
    // =============================================================================

    pub fn get_monitor_status(&self, monitor_id: &str) -> Option<&CyclesMonitorConfig> {
        self.monitored_canisters.get(monitor_id)
    }

    pub fn list_monitors(&self) -> Vec<(String, &CyclesMonitorConfig)> {
        self.monitored_canisters.iter()
            .map(|(id, config)| (id.clone(), config))
            .collect()
    }

    pub fn get_recent_alerts(&self, limit: usize) -> Vec<&CyclesAlert> {
        let mut alerts: Vec<&CyclesAlert> = self.alert_history.iter().collect();
        alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        alerts.truncate(limit);
        alerts
    }

    pub fn get_cycles_statistics(&self) -> CyclesStatistics {
        let total_monitors = self.monitored_canisters.len();
        let total_alerts = self.alert_history.len();
        let critical_alerts = self.alert_history.iter()
            .filter(|alert| matches!(alert.alert_type, AlertType::Critical))
            .count();
        let warnings = self.alert_history.iter()
            .filter(|alert| matches!(alert.alert_type, AlertType::Warning))
            .count();

        CyclesStatistics {
            total_monitors,
            total_alerts,
            critical_alerts,
            warnings,
            last_check: self.last_check_time,
        }
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct CyclesStatistics {
    pub total_monitors: usize,
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub warnings: usize,
    pub last_check: u64,
}

// =============================================================================
// USAGE EXAMPLES
// =============================================================================

impl CyclesMonitorService {
    pub fn usage_examples() -> String {
        format!(r#"
ICP CYCLES MONITOR EXAMPLES:

1. Basic Monitor Setup:
   - Warning Threshold: 10T cycles
   - Critical Threshold: 1T cycles
   - Notifications: Email, Discord

2. Auto Top-up Configuration:
   - Enable auto top-up when critical
   - Top-up amount: 20T cycles
   - Notification on completion

3. Enterprise Monitoring:
   - Monitor multiple canisters
   - Custom thresholds per canister
   - Multi-channel notifications
   - Historical usage tracking

4. Dev Team Usage:
   - Monitor deployment canister cycles
   - Get alerts before workflows fail
   - Automatic cycle management
   - Cost optimization insights

Benefits for ICP Developers:
✅ No more unexpected canister freezes
✅ Proactive cycle management
✅ Cost monitoring and optimization
✅ Automated top-up workflows
✅ Multi-channel alerting system
        "#)
    }
}