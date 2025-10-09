// HTTP Outcall Monitoring and Logging Service
// Tracks success/failure rates, latency, and errors for social media API calls

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct HTTPOutcallMonitor {
    logs: Vec<OutcallLog>,
    stats: HashMap<String, OutcallStats>, // key: platform (twitter, discord, etc.)
    max_logs: usize,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct OutcallLog {
    pub timestamp: u64,
    pub user: Principal,
    pub platform: String,
    pub endpoint: String,
    pub method: String, // GET, POST, etc.
    pub status: OutcallStatus,
    pub latency_ms: u64,
    pub error_message: Option<String>,
    pub response_code: Option<u16>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum OutcallStatus {
    Success,
    Failure,
    RateLimited,
    Timeout,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct OutcallStats {
    pub platform: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub rate_limited_calls: u64,
    pub timeout_calls: u64,
    pub average_latency_ms: f64,
    pub last_24h_calls: u64,
    pub last_error: Option<String>,
    pub last_error_timestamp: Option<u64>,
}

impl Default for HTTPOutcallMonitor {
    fn default() -> Self {
        Self::new(1000) // Default: keep last 1000 logs
    }
}

impl HTTPOutcallMonitor {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Vec::new(),
            stats: HashMap::new(),
            max_logs,
        }
    }

    /// Log an HTTP outcall
    pub fn log_outcall(
        &mut self,
        user: Principal,
        platform: &str,
        endpoint: &str,
        method: &str,
        status: OutcallStatus,
        latency_ms: u64,
        error_message: Option<String>,
        response_code: Option<u16>,
    ) {
        let timestamp = ic_cdk::api::time();

        // Add log entry
        let log = OutcallLog {
            timestamp,
            user,
            platform: platform.to_string(),
            endpoint: endpoint.to_string(),
            method: method.to_string(),
            status: status.clone(),
            latency_ms,
            error_message: error_message.clone(),
            response_code,
        };

        self.logs.push(log);

        // Trim old logs if exceeded max
        if self.logs.len() > self.max_logs {
            self.logs.drain(0..self.logs.len() - self.max_logs);
        }

        // Update stats
        let stats = self.stats.entry(platform.to_string()).or_insert(OutcallStats {
            platform: platform.to_string(),
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            rate_limited_calls: 0,
            timeout_calls: 0,
            average_latency_ms: 0.0,
            last_24h_calls: 0,
            last_error: None,
            last_error_timestamp: None,
        });

        stats.total_calls += 1;

        match status {
            OutcallStatus::Success => stats.successful_calls += 1,
            OutcallStatus::Failure => {
                stats.failed_calls += 1;
                stats.last_error = error_message.clone();
                stats.last_error_timestamp = Some(timestamp);
            }
            OutcallStatus::RateLimited => stats.rate_limited_calls += 1,
            OutcallStatus::Timeout => stats.timeout_calls += 1,
        }

        // Update average latency (rolling average)
        stats.average_latency_ms = (stats.average_latency_ms * (stats.total_calls - 1) as f64
            + latency_ms as f64)
            / stats.total_calls as f64;

        // Count last 24h calls
        let twenty_four_hours_ago = timestamp.saturating_sub(24 * 60 * 60 * 1_000_000_000);
        stats.last_24h_calls = self
            .logs
            .iter()
            .filter(|log| log.platform == platform && log.timestamp >= twenty_four_hours_ago)
            .count() as u64;
    }

    /// Get stats for a specific platform
    pub fn get_platform_stats(&self, platform: &str) -> Option<OutcallStats> {
        self.stats.get(platform).cloned()
    }

    /// Get all platform stats
    pub fn get_all_stats(&self) -> Vec<OutcallStats> {
        self.stats.values().cloned().collect()
    }

    /// Get recent logs for a platform
    pub fn get_recent_logs(&self, platform: &str, limit: usize) -> Vec<OutcallLog> {
        self.logs
            .iter()
            .filter(|log| log.platform == platform)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get recent logs for a user
    pub fn get_user_logs(&self, user: Principal, limit: usize) -> Vec<OutcallLog> {
        self.logs
            .iter()
            .filter(|log| log.user == user)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get error rate for a platform (percentage)
    pub fn get_error_rate(&self, platform: &str) -> f64 {
        if let Some(stats) = self.stats.get(platform) {
            if stats.total_calls == 0 {
                return 0.0;
            }
            (stats.failed_calls as f64 / stats.total_calls as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if platform is experiencing issues (>20% error rate in last 24h)
    pub fn is_platform_degraded(&self, platform: &str) -> bool {
        let error_rate = self.get_error_rate(platform);
        error_rate > 20.0
    }

    /// Get health summary for all platforms
    pub fn get_health_summary(&self) -> Vec<PlatformHealth> {
        self.stats
            .iter()
            .map(|(platform, stats)| {
                let error_rate = (stats.failed_calls as f64 / stats.total_calls.max(1) as f64) * 100.0;
                let health_status = if error_rate > 20.0 {
                    HealthStatus::Degraded
                } else if error_rate > 50.0 {
                    HealthStatus::Down
                } else {
                    HealthStatus::Healthy
                };

                PlatformHealth {
                    platform: platform.clone(),
                    status: health_status,
                    error_rate,
                    avg_latency_ms: stats.average_latency_ms,
                    calls_24h: stats.last_24h_calls,
                    last_error: stats.last_error.clone(),
                }
            })
            .collect()
    }

    /// Clear old logs (older than 7 days)
    pub fn cleanup_old_logs(&mut self) -> usize {
        let seven_days_ago = ic_cdk::api::time().saturating_sub(7 * 24 * 60 * 60 * 1_000_000_000);
        let before_count = self.logs.len();

        self.logs.retain(|log| log.timestamp >= seven_days_ago);

        before_count - self.logs.len()
    }

    /// Reset stats for a platform
    pub fn reset_platform_stats(&mut self, platform: &str) {
        self.stats.remove(platform);
        self.logs.retain(|log| log.platform != platform);
    }
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PlatformHealth {
    pub platform: String,
    pub status: HealthStatus,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub calls_24h: u64,
    pub last_error: Option<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require ic_cdk::api::time() which only works inside canisters

    #[test]
    #[ignore] // Requires canister environment
    fn test_outcall_logging() {
        let mut monitor = HTTPOutcallMonitor::new(100);
        let user = Principal::from_text("aaaaa-aa").unwrap();

        monitor.log_outcall(
            user,
            "twitter",
            "/tweets",
            "POST",
            OutcallStatus::Success,
            250,
            None,
            Some(200),
        );

        let stats = monitor.get_platform_stats("twitter").unwrap();
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.successful_calls, 1);
        assert_eq!(stats.average_latency_ms, 250.0);
    }

    #[test]
    #[ignore] // Requires canister environment
    fn test_error_rate_calculation() {
        let mut monitor = HTTPOutcallMonitor::new(100);
        let user = Principal::from_text("aaaaa-aa").unwrap();

        // 3 successful, 1 failed = 25% error rate
        monitor.log_outcall(user, "discord", "/webhook", "POST", OutcallStatus::Success, 100, None, Some(200));
        monitor.log_outcall(user, "discord", "/webhook", "POST", OutcallStatus::Success, 100, None, Some(200));
        monitor.log_outcall(user, "discord", "/webhook", "POST", OutcallStatus::Success, 100, None, Some(200));
        monitor.log_outcall(user, "discord", "/webhook", "POST", OutcallStatus::Failure, 100, Some("Error".to_string()), Some(500));

        let error_rate = monitor.get_error_rate("discord");
        assert_eq!(error_rate, 25.0);
    }

    #[test]
    #[ignore] // Requires canister environment
    fn test_platform_health_degraded() {
        let mut monitor = HTTPOutcallMonitor::new(100);
        let user = Principal::from_text("aaaaa-aa").unwrap();

        // 30% error rate (3 errors out of 10)
        for _ in 0..7 {
            monitor.log_outcall(user, "telegram", "/sendMessage", "POST", OutcallStatus::Success, 150, None, Some(200));
        }
        for _ in 0..3 {
            monitor.log_outcall(user, "telegram", "/sendMessage", "POST", OutcallStatus::Failure, 150, Some("Error".to_string()), Some(429));
        }

        assert!(monitor.is_platform_degraded("telegram"));
    }
}
