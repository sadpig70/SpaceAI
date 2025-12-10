//! Rollback Frame 패킷
//!
//! PPR 매핑: AI_make_RollbackFrame

use crate::types::Position;
use serde::{Deserialize, Serialize};

/// 롤백 프레임 - 롤백 이벤트 발생 시 전송
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackFrame {
    /// Zone ID
    pub zone_id: u32,

    /// 대상 로봇 ID
    pub robot_id: u64,

    /// 롤백 기준 틱
    pub rollback_tick: u64,

    /// 해당 틱의 월드 상태 해시 (32바이트)
    pub world_state_hash: [u8; 32],

    /// 안전 궤적 (롤백 후 따라야 할 경로)
    pub safe_trajectory: Vec<PredictedState>,

    /// TrustOS 서명 (64바이트)
    /// TrustOS 서명
    pub tos_sig: Vec<u8>,

    /// 롤백 이유
    pub reason: RollbackReason,

    /// 생성 타임스탬프 (나노초)
    pub created_at_ns: u64,
}

/// 예측 상태 (안전 궤적의 각 포인트)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedState {
    /// 틱 오프셋 (롤백 틱 기준)
    pub tick_offset: u32,

    /// 예상 위치
    pub position: Position,

    /// 예상 방향
    pub theta: f32,

    /// 예상 속력
    pub speed: f32,
}

/// 롤백 이유
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackReason {
    /// 예측 오차 초과
    PredictionError { delta_magnitude: f32 },

    /// 충돌 예측
    CollisionPredicted {
        obstacle_id: u64,
        time_to_collision: f32,
    },

    /// 티켓 위반
    TicketViolation { ticket_id: u128 },

    /// 지오펜스 위반
    GeofenceViolation { geofence_id: u32 },

    /// 동역학 위반
    KinematicsViolation { constraint: String },

    /// Edge 다운 복구
    EdgeRecovery,
}

impl RollbackFrame {
    /// 새 RollbackFrame 생성
    pub fn new(zone_id: u32, robot_id: u64, rollback_tick: u64, reason: RollbackReason) -> Self {
        Self {
            zone_id,
            robot_id,
            rollback_tick,
            world_state_hash: [0u8; 32],
            safe_trajectory: Vec::new(),
            tos_sig: vec![0u8; 64],
            reason,
            created_at_ns: 0,
        }
    }

    /// 월드 상태 해시 설정
    pub fn with_state_hash(mut self, hash: [u8; 32]) -> Self {
        self.world_state_hash = hash;
        self
    }

    /// 안전 궤적 추가
    pub fn with_trajectory(mut self, trajectory: Vec<PredictedState>) -> Self {
        self.safe_trajectory = trajectory;
        self
    }

    /// 서명 설정
    pub fn with_signature(mut self, sig: &[u8]) -> Self {
        self.tos_sig = sig.to_vec();
        self
    }

    /// 타임스탬프 설정
    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.created_at_ns = timestamp_ns;
        self
    }

    /// 안전 궤적 길이
    pub fn trajectory_len(&self) -> usize {
        self.safe_trajectory.len()
    }

    /// 서명 여부
    pub fn is_signed(&self) -> bool {
        self.tos_sig.iter().any(|&b| b != 0)
    }
}

impl PredictedState {
    /// 새 PredictedState 생성
    pub fn new(tick_offset: u32, position: Position, theta: f32, speed: f32) -> Self {
        Self {
            tick_offset,
            position,
            theta,
            speed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollback_frame_new() {
        let frame = RollbackFrame::new(
            1,
            42,
            100,
            RollbackReason::PredictionError {
                delta_magnitude: 0.15,
            },
        );
        assert_eq!(frame.zone_id, 1);
        assert_eq!(frame.robot_id, 42);
        assert_eq!(frame.rollback_tick, 100);
    }

    #[test]
    fn test_rollback_frame_with_trajectory() {
        let trajectory = vec![
            PredictedState::new(0, Position::new(0.0, 0.0, 0.0), 0.0, 1.0),
            PredictedState::new(1, Position::new(0.05, 0.0, 0.0), 0.0, 1.0),
            PredictedState::new(2, Position::new(0.10, 0.0, 0.0), 0.0, 1.0),
        ];

        let frame =
            RollbackFrame::new(1, 1, 100, RollbackReason::EdgeRecovery).with_trajectory(trajectory);

        assert_eq!(frame.trajectory_len(), 3);
    }

    #[test]
    fn test_rollback_frame_serialization() {
        let frame = RollbackFrame::new(
            1,
            42,
            100,
            RollbackReason::CollisionPredicted {
                obstacle_id: 99,
                time_to_collision: 0.5,
            },
        );

        let encoded = bincode::serialize(&frame).unwrap();
        let decoded: RollbackFrame = bincode::deserialize(&encoded).unwrap();

        assert_eq!(frame.robot_id, decoded.robot_id);
        assert_eq!(frame.rollback_tick, decoded.rollback_tick);
    }
}
