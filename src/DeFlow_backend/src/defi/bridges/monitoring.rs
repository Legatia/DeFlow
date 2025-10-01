// Bridge Health Monitoring and Failover System
// Monitors bridge performance and automatically switches on failures

use super::*;
use std::collections::{HashMap, VecDeque};

const MAX_FAILURE_HISTORY: usize = 100;
const HEALTH_CHECK_INTERVAL: u64 = 300; // 5 minutes

/// Bridge monitoring service
#[derive(Debug, Clone)]
pub struct BridgeMonitor {
    pub bridge_health: HashMap<BridgeProtocol, BridgeHealthMetrics>,
    pub failure_history: HashMap<BridgeProtocol, VecDeque<BridgeFailure>>,
    pub last_health_check: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BridgeHealthMetrics {
    pub protocol: BridgeProtocol,
    pub is_healthy: bool,
    pub uptime_percentage: f64,
    pub avg_completion_time: u64,
    pub success_rate_1h: f64,
    pub success_rate_24h: f64,
    pub success_rate_7d: f64,
    pub total_volume_24h: f64,
    pub failed_transactions_24h: u32,
    pub last_successful_bridge: Option<u64>,
    pub last_failed_bridge: Option<u64>,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct BridgeFailure {
    pub timestamp: u64,
    pub bridge_id: String,
    pub error_type: FailureType,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub asset: String,
    pub amount: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub enum FailureType {
    Timeout,
    InsufficientLiquidity,
    NetworkError,
    ContractError,
    ValidationError,
    Unknown(String),
}

impl BridgeMonitor {
    pub fn new() -> Self {
        let mut bridge_health = HashMap::new();

        // Initialize health metrics for all bridges
        for protocol in [
            BridgeProtocol::Wormhole,
            BridgeProtocol::LayerZero,
            BridgeProtocol::Stargate,
        ] {
            bridge_health.insert(protocol.clone(), BridgeHealthMetrics {
                protocol: protocol.clone(),
                is_healthy: true,
                uptime_percentage: 100.0,
                avg_completion_time: 0,
                success_rate_1h: 1.0,
                success_rate_24h: 1.0,
                success_rate_7d: 1.0,
                total_volume_24h: 0.0,
                failed_transactions_24h: 0,
                last_successful_bridge: None,
                last_failed_bridge: None,
                consecutive_failures: 0,
            });
        }

        Self {
            bridge_health,
            failure_history: HashMap::new(),
            last_health_check: ic_cdk::api::time(),
        }
    }

    /// Record successful bridge transaction
    pub fn record_success(&mut self, protocol: BridgeProtocol, completion_time: u64) {
        if let Some(metrics) = self.bridge_health.get_mut(&protocol) {
            metrics.last_successful_bridge = Some(ic_cdk::api::time());
            metrics.consecutive_failures = 0;

            // Update average completion time
            if metrics.avg_completion_time == 0 {
                metrics.avg_completion_time = completion_time;
            } else {
                metrics.avg_completion_time = (metrics.avg_completion_time + completion_time) / 2;
            }

            // Recalculate success rates
            self.recalculate_success_rates(&protocol);
        }
    }

    /// Record failed bridge transaction
    pub fn record_failure(
        &mut self,
        protocol: BridgeProtocol,
        bridge_id: String,
        error_type: FailureType,
        from_chain: ChainId,
        to_chain: ChainId,
        asset: String,
        amount: u64,
    ) {
        // Update health metrics
        if let Some(metrics) = self.bridge_health.get_mut(&protocol) {
            metrics.last_failed_bridge = Some(ic_cdk::api::time());
            metrics.consecutive_failures += 1;
            metrics.failed_transactions_24h += 1;

            // Mark as unhealthy if too many consecutive failures
            if metrics.consecutive_failures >= 3 {
                metrics.is_healthy = false;
            }
        }

        // Record failure in history
        let failure = BridgeFailure {
            timestamp: ic_cdk::api::time(),
            bridge_id,
            error_type,
            from_chain,
            to_chain,
            asset,
            amount,
        };

        self.failure_history
            .entry(protocol.clone())
            .or_insert_with(VecDeque::new)
            .push_back(failure);

        // Limit history size
        if let Some(history) = self.failure_history.get_mut(&protocol) {
            while history.len() > MAX_FAILURE_HISTORY {
                history.pop_front();
            }
        }

        // Recalculate success rates
        self.recalculate_success_rates(&protocol);
    }

    /// Get healthiest bridge for a route
    pub fn get_healthiest_bridge(
        &self,
        available_protocols: Vec<BridgeProtocol>,
    ) -> Option<BridgeProtocol> {
        available_protocols
            .into_iter()
            .filter(|p| self.is_bridge_healthy(p))
            .max_by(|a, b| {
                let score_a = self.calculate_health_score(a);
                let score_b = self.calculate_health_score(b);
                score_a.partial_cmp(&score_b).unwrap()
            })
    }

    /// Check if bridge is healthy
    pub fn is_bridge_healthy(&self, protocol: &BridgeProtocol) -> bool {
        self.bridge_health
            .get(protocol)
            .map(|m| m.is_healthy && m.success_rate_24h > 0.95)
            .unwrap_or(false)
    }

    /// Calculate overall health score
    fn calculate_health_score(&self, protocol: &BridgeProtocol) -> f64 {
        if let Some(metrics) = self.bridge_health.get(protocol) {
            if !metrics.is_healthy {
                return 0.0;
            }

            // Weighted score
            let uptime_weight = 0.3;
            let success_weight = 0.5;
            let speed_weight = 0.2;

            let uptime_score = metrics.uptime_percentage / 100.0;
            let success_score = metrics.success_rate_24h;
            let speed_score = if metrics.avg_completion_time > 0 {
                1.0 / (1.0 + metrics.avg_completion_time as f64 / 600.0) // Normalize to 10 min
            } else {
                0.5
            };

            (uptime_score * uptime_weight)
                + (success_score * success_weight)
                + (speed_score * speed_weight)
        } else {
            0.0
        }
    }

    /// Recalculate success rates based on history
    fn recalculate_success_rates(&mut self, protocol: &BridgeProtocol) {
        // In production, query actual transaction history
        // For now, use simplified logic based on recent failures

        if let Some(metrics) = self.bridge_health.get_mut(protocol) {
            let failures = self.failure_history
                .get(protocol)
                .map(|h| h.len())
                .unwrap_or(0);

            // Simple estimation (in production, use actual success/fail counts)
            metrics.success_rate_1h = 1.0 - (failures.min(10) as f64 / 100.0);
            metrics.success_rate_24h = 1.0 - (failures.min(50) as f64 / 200.0);
            metrics.success_rate_7d = 1.0 - (failures as f64 / 1000.0);
        }
    }

    /// Perform health check on all bridges
    pub async fn health_check(&mut self) {
        let now = ic_cdk::api::time();

        // Only check if enough time has passed
        if now - self.last_health_check < HEALTH_CHECK_INTERVAL * 1_000_000_000 {
            return;
        }

        // Check each bridge
        for protocol in [
            BridgeProtocol::Wormhole,
            BridgeProtocol::LayerZero,
            BridgeProtocol::Stargate,
        ] {
            // In production, query actual bridge health endpoints
            // For now, check based on recent failures

            if let Some(metrics) = self.bridge_health.get_mut(&protocol) {
                // Auto-recover if enough time passed since last failure
                if let Some(last_failure) = metrics.last_failed_bridge {
                    let time_since_failure = now - last_failure;
                    if time_since_failure > 3600 * 1_000_000_000 {
                        // 1 hour
                        metrics.is_healthy = true;
                        metrics.consecutive_failures = 0;
                    }
                }

                // Update uptime
                metrics.uptime_percentage = if metrics.is_healthy { 99.9 } else { 95.0 };
            }
        }

        self.last_health_check = now;
    }

    /// Get monitoring report
    pub fn get_monitoring_report(&self) -> MonitoringReport {
        let total_bridges = self.bridge_health.len();
        let healthy_bridges = self.bridge_health.values().filter(|m| m.is_healthy).count();

        let total_failures_24h: u32 = self.bridge_health
            .values()
            .map(|m| m.failed_transactions_24h)
            .sum();

        MonitoringReport {
            total_bridges,
            healthy_bridges,
            unhealthy_bridges: total_bridges - healthy_bridges,
            overall_health_score: (healthy_bridges as f64 / total_bridges as f64) * 100.0,
            total_failures_24h,
            bridge_metrics: self.bridge_health.values().cloned().collect(),
            last_updated: ic_cdk::api::time(),
        }
    }
}

impl Default for BridgeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub total_bridges: usize,
    pub healthy_bridges: usize,
    pub unhealthy_bridges: usize,
    pub overall_health_score: f64,
    pub total_failures_24h: u32,
    pub bridge_metrics: Vec<BridgeHealthMetrics>,
    pub last_updated: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = BridgeMonitor::new();
        assert_eq!(monitor.bridge_health.len(), 3);
    }

    #[test]
    fn test_record_success() {
        let mut monitor = BridgeMonitor::new();
        monitor.record_success(BridgeProtocol::Stargate, 300);

        let metrics = monitor.bridge_health.get(&BridgeProtocol::Stargate).unwrap();
        assert_eq!(metrics.consecutive_failures, 0);
        assert_eq!(metrics.avg_completion_time, 300);
    }

    #[test]
    fn test_record_failure() {
        let mut monitor = BridgeMonitor::new();

        for _ in 0..3 {
            monitor.record_failure(
                BridgeProtocol::Wormhole,
                "test_id".to_string(),
                FailureType::Timeout,
                ChainId::Ethereum,
                ChainId::Polygon,
                "USDC".to_string(),
                1000000,
            );
        }

        let metrics = monitor.bridge_health.get(&BridgeProtocol::Wormhole).unwrap();
        assert!(!metrics.is_healthy); // Should be unhealthy after 3 failures
    }

    #[test]
    fn test_health_score() {
        let monitor = BridgeMonitor::new();
        let score = monitor.calculate_health_score(&BridgeProtocol::Stargate);
        assert!(score > 0.0 && score <= 1.0);
    }
}
