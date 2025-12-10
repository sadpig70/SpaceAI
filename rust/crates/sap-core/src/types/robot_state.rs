//! 로봇 상태 타입
//!
//! PPR 매핑: AI_perceive_RobotState, AI_perceive_CurrentState

use super::{Acceleration, Position, Velocity};
use serde::{Deserialize, Serialize};

/// 로봇의 전체 상태
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotState {
    /// 로봇 고유 ID
    pub robot_id: u64,

    /// 현재 위치
    pub position: Position,

    /// 현재 속도
    pub velocity: Velocity,

    /// 현재 가속도
    pub acceleration: Acceleration,

    /// 방향각 (라디안, 0 = +X 방향)
    pub theta: f32,

    /// 각속도 (rad/s)
    pub omega: f32,

    /// PTP 동기화된 타임스탬프 (나노초)
    pub timestamp_ns: u64,

    /// 현재 Zone ID
    pub zone_id: u32,

    /// 현재 사용 중인 티켓 ID (없으면 0)
    pub ticket_id: u128,

    /// 현재 VoxelTimeSlot 진행률 (0.0 ~ 1.0)
    pub ticket_phase: f32,

    /// 배터리 잔량 (밀리 퍼센트, 0~100000)
    pub battery_soc_milli: u32,

    /// 컨트롤러 온도 (섭씨)
    pub controller_temp_c: i16,
}

impl RobotState {
    /// 새 RobotState 생성 (기본값)
    pub fn new(robot_id: u64) -> Self {
        Self {
            robot_id,
            position: Position::ORIGIN,
            velocity: Velocity::ZERO,
            acceleration: Acceleration::ZERO,
            theta: 0.0,
            omega: 0.0,
            timestamp_ns: 0,
            zone_id: 0,
            ticket_id: 0,
            ticket_phase: 0.0,
            battery_soc_milli: 100_000, // 100%
            controller_temp_c: 25,
        }
    }

    /// 위치와 속도만 업데이트
    pub fn with_motion(mut self, position: Position, velocity: Velocity) -> Self {
        self.position = position;
        self.velocity = velocity;
        self
    }

    /// 타임스탬프 업데이트
    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }

    /// Zone 정보 업데이트
    pub fn with_zone(mut self, zone_id: u32, ticket_id: u128) -> Self {
        self.zone_id = zone_id;
        self.ticket_id = ticket_id;
        self
    }

    /// 속력(speed) 반환
    #[inline]
    pub fn speed(&self) -> f32 {
        self.velocity.magnitude()
    }

    /// 최대 속도 제한 내인지 검사
    #[inline]
    pub fn velocity_within_limit(&self, max_speed: f32) -> bool {
        self.velocity.within_limit(max_speed)
    }

    /// 최대 가속도 제한 내인지 검사
    #[inline]
    pub fn acceleration_within_limit(&self, max_accel: f32) -> bool {
        self.acceleration.within_limit(max_accel)
    }
}

impl Default for RobotState {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_state_new() {
        let state = RobotState::new(42);
        assert_eq!(state.robot_id, 42);
        assert_eq!(state.position, Position::ORIGIN);
        assert_eq!(state.battery_soc_milli, 100_000);
    }

    #[test]
    fn test_robot_state_with_motion() {
        let state = RobotState::new(1)
            .with_motion(Position::new(10.0, 20.0, 0.0), Velocity::new(1.0, 2.0, 0.0));
        assert_eq!(state.position.x, 10.0);
        assert_eq!(state.velocity.vx, 1.0);
    }

    #[test]
    fn test_robot_state_speed() {
        let mut state = RobotState::new(1);
        state.velocity = Velocity::new(3.0, 4.0, 0.0);
        assert!((state.speed() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_robot_state_serialization() {
        let state = RobotState::new(123)
            .with_motion(Position::new(1.0, 2.0, 3.0), Velocity::new(0.1, 0.2, 0.3));

        let encoded = bincode::serialize(&state).unwrap();
        let decoded: RobotState = bincode::deserialize(&encoded).unwrap();

        assert_eq!(state.robot_id, decoded.robot_id);
        assert_eq!(state.position, decoded.position);
    }
}
