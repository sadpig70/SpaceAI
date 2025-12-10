//! FailsafeManager - 장애 대응 관리자
//!
//! PPR 매핑: AI_make_FailsafeManager

use std::collections::HashMap;

/// Failsafe 관리자 설정
#[derive(Debug, Clone)]
pub struct FailsafeConfig {
    pub heartbeat_timeout_ms: u64,
    pub max_retries: u32,
    pub degraded_speed_factor: f32,
    pub emergency_stop_distance: f32,
}

impl Default for FailsafeConfig {
    fn default() -> Self {
        Self {
            heartbeat_timeout_ms: 100,
            max_retries: 3,
            degraded_speed_factor: 0.5,
            emergency_stop_distance: 0.2,
        }
    }
}

/// Edge 서버 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStatus {
    Healthy,
    Degraded,
    Unresponsive,
    Failed,
}

/// 장애 대응 액션
#[derive(Debug, Clone)]
pub enum FailsafeAction {
    None,
    EnableDegradedMode { speed_factor: f32 },
    EmergencyDeceleration { target_speed: f32 },
    EmergencyStop,
    EdgeHandover { from_edge: u32, to_edge: u32 },
    ZoneIsolation { zone_id: u32 },
}

/// Failsafe 관리자
pub struct FailsafeManager {
    config: FailsafeConfig,
    zone_id: u32,
    edge_status: HashMap<u32, EdgeStatusInfo>,
    current_mode: OperationMode,
}

#[derive(Debug, Clone)]
struct EdgeStatusInfo {
    status: EdgeStatus,
    last_heartbeat_ns: u64,
    consecutive_failures: u32,
}

/// 운영 모드
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    Normal,
    Degraded,
    Emergency,
}

impl FailsafeManager {
    pub fn new(zone_id: u32, config: FailsafeConfig) -> Self {
        Self {
            config,
            zone_id,
            edge_status: HashMap::new(),
            current_mode: OperationMode::Normal,
        }
    }

    pub fn with_default_config(zone_id: u32) -> Self {
        Self::new(zone_id, FailsafeConfig::default())
    }

    pub fn register_edge(&mut self, edge_id: u32) {
        self.edge_status.insert(
            edge_id,
            EdgeStatusInfo {
                status: EdgeStatus::Healthy,
                last_heartbeat_ns: 0,
                consecutive_failures: 0,
            },
        );
    }

    pub fn receive_heartbeat(&mut self, edge_id: u32, timestamp_ns: u64) {
        if let Some(info) = self.edge_status.get_mut(&edge_id) {
            info.last_heartbeat_ns = timestamp_ns;
            info.consecutive_failures = 0;
            info.status = EdgeStatus::Healthy;
        }
    }

    pub fn check_and_decide(&mut self, current_time_ns: u64) -> FailsafeAction {
        let timeout_ns = self.config.heartbeat_timeout_ms * 1_000_000;
        let mut unhealthy_count = 0;
        let mut failed_edges = Vec::new();

        for (&edge_id, info) in self.edge_status.iter_mut() {
            let elapsed_ns = current_time_ns.saturating_sub(info.last_heartbeat_ns);

            if elapsed_ns > timeout_ns * 3 {
                info.status = EdgeStatus::Failed;
                failed_edges.push(edge_id);
                unhealthy_count += 1;
            } else if elapsed_ns > timeout_ns * 2 {
                info.status = EdgeStatus::Unresponsive;
                unhealthy_count += 1;
            } else if elapsed_ns > timeout_ns {
                info.status = EdgeStatus::Degraded;
            } else {
                info.status = EdgeStatus::Healthy;
            }
        }

        if failed_edges.len() > 1 {
            self.current_mode = OperationMode::Emergency;
            FailsafeAction::EmergencyStop
        } else if unhealthy_count > 0 {
            self.current_mode = OperationMode::Degraded;
            FailsafeAction::EnableDegradedMode {
                speed_factor: self.config.degraded_speed_factor,
            }
        } else {
            self.current_mode = OperationMode::Normal;
            FailsafeAction::None
        }
    }

    #[allow(dead_code)]
    pub fn report_edge_failure(&mut self, edge_id: u32, _current_time_ns: u64) {
        if let Some(info) = self.edge_status.get_mut(&edge_id) {
            info.consecutive_failures += 1;
            if info.consecutive_failures >= self.config.max_retries {
                info.status = EdgeStatus::Failed;
            }
        }
    }

    pub fn current_mode(&self) -> OperationMode {
        self.current_mode
    }

    pub fn get_edge_status(&self, edge_id: u32) -> Option<EdgeStatus> {
        self.edge_status.get(&edge_id).map(|i| i.status)
    }

    pub fn healthy_edge_count(&self) -> usize {
        self.edge_status
            .values()
            .filter(|i| i.status == EdgeStatus::Healthy)
            .count()
    }

    #[allow(dead_code)]
    pub fn total_edge_count(&self) -> usize {
        self.edge_status.len()
    }

    pub fn zone_id(&self) -> u32 {
        self.zone_id
    }

    pub fn emergency_stop(&mut self, _current_time_ns: u64) -> FailsafeAction {
        self.current_mode = OperationMode::Emergency;
        FailsafeAction::EmergencyStop
    }

    pub fn recover_to_normal(&mut self) {
        self.current_mode = OperationMode::Normal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failsafe_manager_new() {
        let manager = FailsafeManager::with_default_config(1);
        assert_eq!(manager.zone_id(), 1);
        assert_eq!(manager.current_mode(), OperationMode::Normal);
    }

    #[test]
    fn test_register_and_heartbeat() {
        let mut manager = FailsafeManager::with_default_config(1);
        manager.register_edge(1);
        manager.receive_heartbeat(1, 1_000_000_000);
        assert_eq!(manager.get_edge_status(1), Some(EdgeStatus::Healthy));
        assert_eq!(manager.healthy_edge_count(), 1);
    }

    #[test]
    fn test_edge_timeout() {
        let mut manager = FailsafeManager::with_default_config(1);
        manager.register_edge(1);
        manager.register_edge(2);
        manager.receive_heartbeat(1, 0);
        manager.receive_heartbeat(2, 0);
        let action = manager.check_and_decide(250_000_000);
        assert_eq!(manager.current_mode(), OperationMode::Degraded);
        assert!(matches!(action, FailsafeAction::EnableDegradedMode { .. }));
    }

    #[test]
    fn test_multiple_edge_failure() {
        let mut manager = FailsafeManager::with_default_config(1);
        manager.register_edge(1);
        manager.register_edge(2);
        manager.receive_heartbeat(1, 0);
        manager.receive_heartbeat(2, 0);
        let action = manager.check_and_decide(500_000_000);
        assert_eq!(manager.current_mode(), OperationMode::Emergency);
        assert!(matches!(action, FailsafeAction::EmergencyStop));
    }

    #[test]
    fn test_manual_emergency_stop() {
        let mut manager = FailsafeManager::with_default_config(1);
        let action = manager.emergency_stop(1_000_000_000);
        assert_eq!(manager.current_mode(), OperationMode::Emergency);
        assert!(matches!(action, FailsafeAction::EmergencyStop));
    }

    #[test]
    fn test_recover_to_normal() {
        let mut manager = FailsafeManager::with_default_config(1);
        manager.emergency_stop(1_000_000_000);
        assert_eq!(manager.current_mode(), OperationMode::Emergency);
        manager.recover_to_normal();
        assert_eq!(manager.current_mode(), OperationMode::Normal);
    }
}
