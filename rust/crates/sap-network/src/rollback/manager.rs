//! RollbackManager - 롤백(상태 재조정) 관리자
//!
//! # 용어 정의
//!
//! "Rollback"은 **논리적 상태 재조정(Logical State Reconciliation)**을 의미합니다.
//! - **논리적 롤백**: 월드 상태(WorldState)를 이전 스냅샷으로 되돌림
//! - **물리적 복구**: 로봇에게 안전 정지/감속/재경로 명령 전송 (별도 처리 필요)
//!
//! SAP에서 "Rollback"은 시뮬레이션/게임 넷코드의 롤백 개념을 차용했으며,
//! 실제 물리적 로봇은 시간을 되돌릴 수 없으므로, 이 모듈의 역할은:
//! 1. 예측 오차 감지 시 논리 상태를 이전으로 되돌림
//! 2. 물리적 로봇에게는 RollbackFrame을 통해 복구 궤적(safe_trajectory) 전달
//! 3. 로봇은 safe_trajectory를 따라 물리적으로 복구 동작 수행
//!
//! PPR 매핑: AI_make_RollbackManager

use sap_core::packet::{RollbackFrame, RollbackReason as CoreRollbackReason};
use sap_core::types::WorldState;
use std::collections::HashMap;

/// 스냅샷 저장 전략
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapshotStrategy {
    /// 고정 틱 간격으로 저장 (기본)
    /// - 장점: 예측 가능, 구현 간단
    /// - 단점: 메모리 사용량 고정
    TickBased {
        /// 스냅샷 간격 (틱 수)
        interval: u64,
    },

    /// 메모리 예산 기반 저장
    /// - 장점: 메모리 사용량 제한
    /// - 단점: 오래된 스냅샷 자동 삭제
    MemoryBudget {
        /// 최대 메모리 (바이트)
        max_bytes: usize,
        /// 스냅샷당 예상 크기 (바이트)
        estimated_size_per_snapshot: usize,
    },

    /// 적응형 저장 (롤백 빈도 기반)
    /// - 장점: 롤백 많은 로봇에 더 자주 저장
    /// - 단점: 예측 어려움
    Adaptive {
        /// 기본 간격 (틱)
        base_interval: u64,
        /// 롤백 시 간격 감소 배율 (0.5 = 절반)
        reduction_factor: f32,
        /// 최소 간격 (틱)
        min_interval: u64,
    },
}

impl Default for SnapshotStrategy {
    fn default() -> Self {
        Self::TickBased { interval: 10 }
    }
}

impl SnapshotStrategy {
    /// 다음 스냅샷까지의 간격 계산
    pub fn compute_interval(&self, consecutive_rollbacks: u32) -> u64 {
        match self {
            Self::TickBased { interval } => *interval,
            Self::MemoryBudget {
                max_bytes,
                estimated_size_per_snapshot,
            } => {
                // 예상 최대 스냅샷 수에서 간격 역산
                let size = (*estimated_size_per_snapshot).max(1);
                let max_snapshots = *max_bytes / size;
                (100 / max_snapshots.max(1)) as u64 // 100틱당 max_snapshots개
            }
            Self::Adaptive {
                base_interval,
                reduction_factor,
                min_interval,
            } => {
                // 롤백 횟수에 따라 간격 감소
                let factor = reduction_factor.powf(consecutive_rollbacks as f32);
                let interval = (*base_interval as f32 * factor) as u64;
                interval.max(*min_interval)
            }
        }
    }

    /// 전략 이름 반환
    pub fn name(&self) -> &'static str {
        match self {
            Self::TickBased { .. } => "TickBased",
            Self::MemoryBudget { .. } => "MemoryBudget",
            Self::Adaptive { .. } => "Adaptive",
        }
    }
}

/// 롤백 관리자 설정
#[derive(Debug, Clone)]
pub struct RollbackConfig {
    /// 최대 스냅샷 보관 개수
    pub max_snapshots: usize,

    /// 스냅샷 간격 (틱) - 기본 전략용 (deprecated, use strategy)
    pub snapshot_interval: u64,

    /// 스냅샷 저장 전략
    pub strategy: SnapshotStrategy,

    /// 연속 롤백 최대 허용 횟수
    pub max_consecutive_rollbacks: u32,

    /// 롤백 쿨다운 (밀리초)
    pub rollback_cooldown_ms: u64,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            max_snapshots: 100,
            snapshot_interval: 10,
            strategy: SnapshotStrategy::default(),
            max_consecutive_rollbacks: 3,
            rollback_cooldown_ms: 500,
        }
    }
}

/// 롤백 관리자
pub struct RollbackManager {
    config: RollbackConfig,
    zone_id: u32,
    snapshots: HashMap<u64, WorldState>,
    snapshot_ticks: Vec<u64>,
    rollback_history: Vec<RollbackEvent>,
    consecutive_rollbacks: HashMap<u64, u32>,
    last_rollback_time: HashMap<u64, u64>,
}

/// 롤백 이벤트
#[derive(Debug, Clone)]
pub struct RollbackEvent {
    pub robot_id: u64,
    pub rollback_tick: u64,
    pub current_tick: u64,
    pub reason: RollbackReason,
    pub timestamp_ns: u64,
    pub success: bool,
}

/// 롤백 이유
#[derive(Debug, Clone)]
pub enum RollbackReason {
    PredictionError { delta: f32 },
    CollisionPredicted,
    TicketViolation,
    EdgeRecovery,
    Manual,
}

impl RollbackManager {
    pub fn new(zone_id: u32, config: RollbackConfig) -> Self {
        Self {
            config,
            zone_id,
            snapshots: HashMap::new(),
            snapshot_ticks: Vec::new(),
            rollback_history: Vec::new(),
            consecutive_rollbacks: HashMap::new(),
            last_rollback_time: HashMap::new(),
        }
    }

    pub fn with_default_config(zone_id: u32) -> Self {
        Self::new(zone_id, RollbackConfig::default())
    }

    pub fn save_snapshot(&mut self, tick: u64, state: WorldState) {
        if !self.snapshot_ticks.is_empty() {
            let last_tick = *self.snapshot_ticks.last().unwrap();
            if tick.saturating_sub(last_tick) < self.config.snapshot_interval {
                return;
            }
        }

        while self.snapshots.len() >= self.config.max_snapshots {
            if let Some(oldest_tick) = self.snapshot_ticks.first().copied() {
                self.snapshots.remove(&oldest_tick);
                self.snapshot_ticks.remove(0);
            }
        }

        self.snapshots.insert(tick, state);
        self.snapshot_ticks.push(tick);
    }

    pub fn get_snapshot(&self, tick: u64) -> Option<&WorldState> {
        self.snapshots.get(&tick)
    }

    pub fn find_nearest_snapshot(&self, tick: u64) -> Option<(&u64, &WorldState)> {
        self.snapshot_ticks
            .iter()
            .rev()
            .find(|&&t| t <= tick)
            .and_then(|t| self.snapshots.get(t).map(|s| (t, s)))
    }

    pub fn execute_rollback(
        &mut self,
        robot_id: u64,
        current_tick: u64,
        reason: RollbackReason,
        timestamp_ns: u64,
    ) -> Result<RollbackFrame, RollbackError> {
        // 쿨다운 확인
        if let Some(&last_time) = self.last_rollback_time.get(&robot_id) {
            let elapsed_ms = timestamp_ns.saturating_sub(last_time) / 1_000_000;
            if elapsed_ms < self.config.rollback_cooldown_ms {
                return Err(RollbackError::CooldownActive {
                    remaining_ms: self.config.rollback_cooldown_ms - elapsed_ms,
                });
            }
        }

        // 연속 롤백 횟수 확인
        let consecutive = *self.consecutive_rollbacks.get(&robot_id).unwrap_or(&0);
        if consecutive >= self.config.max_consecutive_rollbacks {
            return Err(RollbackError::TooManyConsecutive { count: consecutive });
        }

        // 스냅샷 찾기
        let (rollback_tick, snapshot) = self
            .find_nearest_snapshot(current_tick)
            .ok_or(RollbackError::NoSnapshotAvailable)?;

        // RollbackFrame 생성
        let frame = RollbackFrame::new(
            self.zone_id,
            robot_id,
            *rollback_tick,
            CoreRollbackReason::PredictionError {
                delta_magnitude: 0.0,
            },
        )
        .with_state_hash(snapshot.compute_hash())
        .with_timestamp(timestamp_ns);

        // 히스토리 기록
        let event = RollbackEvent {
            robot_id,
            rollback_tick: *rollback_tick,
            current_tick,
            reason,
            timestamp_ns,
            success: true,
        };
        self.rollback_history.push(event);

        *self.consecutive_rollbacks.entry(robot_id).or_insert(0) += 1;
        self.last_rollback_time.insert(robot_id, timestamp_ns);

        Ok(frame)
    }

    pub fn reset_consecutive(&mut self, robot_id: u64) {
        self.consecutive_rollbacks.remove(&robot_id);
    }

    pub fn rollback_stats(&self, robot_id: u64) -> RollbackStats {
        let robot_events: Vec<&RollbackEvent> = self
            .rollback_history
            .iter()
            .filter(|e| e.robot_id == robot_id)
            .collect();

        RollbackStats {
            total_rollbacks: robot_events.len(),
            successful_rollbacks: robot_events.iter().filter(|e| e.success).count(),
            consecutive_count: *self.consecutive_rollbacks.get(&robot_id).unwrap_or(&0),
        }
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn zone_id(&self) -> u32 {
        self.zone_id
    }
}

/// 롤백 통계
#[derive(Debug, Clone)]
pub struct RollbackStats {
    pub total_rollbacks: usize,
    pub successful_rollbacks: usize,
    pub consecutive_count: u32,
}

/// 롤백 에러
#[derive(Debug, Clone)]
pub enum RollbackError {
    CooldownActive { remaining_ms: u64 },
    TooManyConsecutive { count: u32 },
    NoSnapshotAvailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_world_state(zone_id: u32, tick: u64) -> WorldState {
        WorldState::new(zone_id).with_tick(tick, tick * 50_000_000)
    }

    #[test]
    fn test_save_and_get_snapshot() {
        let mut manager = RollbackManager::with_default_config(1);
        let state = create_world_state(1, 100);
        manager.save_snapshot(100, state);
        assert!(manager.get_snapshot(100).is_some());
        assert_eq!(manager.snapshot_count(), 1);
    }

    #[test]
    fn test_find_nearest_snapshot() {
        let mut manager = RollbackManager::with_default_config(1);
        manager.save_snapshot(10, create_world_state(1, 10));
        manager.save_snapshot(30, create_world_state(1, 30));
        manager.save_snapshot(50, create_world_state(1, 50));
        let (tick, _) = manager.find_nearest_snapshot(45).unwrap();
        assert_eq!(*tick, 30);
    }

    #[test]
    fn test_execute_rollback() {
        let mut manager = RollbackManager::with_default_config(1);
        manager.save_snapshot(10, create_world_state(1, 10));
        manager.save_snapshot(30, create_world_state(1, 30));
        let result = manager.execute_rollback(
            42,
            50,
            RollbackReason::PredictionError { delta: 0.15 },
            5_000_000_000,
        );
        assert!(result.is_ok());
        let frame = result.unwrap();
        assert_eq!(frame.robot_id, 42);
        assert_eq!(frame.rollback_tick, 30);
    }

    #[test]
    fn test_rollback_cooldown() {
        let mut manager = RollbackManager::with_default_config(1);
        manager.save_snapshot(10, create_world_state(1, 10));
        let _ = manager.execute_rollback(42, 50, RollbackReason::Manual, 1_000_000_000);
        let result = manager.execute_rollback(42, 51, RollbackReason::Manual, 1_100_000_000);
        assert!(matches!(result, Err(RollbackError::CooldownActive { .. })));
    }

    #[test]
    fn test_max_consecutive_rollbacks() {
        let config = RollbackConfig {
            max_consecutive_rollbacks: 2,
            rollback_cooldown_ms: 0,
            ..Default::default()
        };
        let mut manager = RollbackManager::new(1, config);
        manager.save_snapshot(10, create_world_state(1, 10));
        let _ = manager.execute_rollback(42, 50, RollbackReason::Manual, 1_000_000_000);
        let _ = manager.execute_rollback(42, 51, RollbackReason::Manual, 2_000_000_000);
        let result = manager.execute_rollback(42, 52, RollbackReason::Manual, 3_000_000_000);
        assert!(matches!(
            result,
            Err(RollbackError::TooManyConsecutive { count: 2 })
        ));
    }

    #[test]
    fn test_reset_consecutive() {
        let config = RollbackConfig {
            max_consecutive_rollbacks: 2,
            rollback_cooldown_ms: 0,
            ..Default::default()
        };
        let mut manager = RollbackManager::new(1, config);
        manager.save_snapshot(10, create_world_state(1, 10));
        let _ = manager.execute_rollback(42, 50, RollbackReason::Manual, 1_000_000_000);
        let _ = manager.execute_rollback(42, 51, RollbackReason::Manual, 2_000_000_000);
        manager.reset_consecutive(42);
        let result = manager.execute_rollback(42, 52, RollbackReason::Manual, 3_000_000_000);
        assert!(result.is_ok());
    }
}
