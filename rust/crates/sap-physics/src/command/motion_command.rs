//! MotionCommand - 이동 명령
//!
//! PPR 매핑: AI_perceive_MotionCommand

use sap_core::types::{Acceleration, Position, Velocity};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// 이동 명령
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionCommand {
    /// 로봇 ID
    pub robot_id: u64,

    /// 현재 위치
    pub current_position: Position,

    /// 목표 속도
    pub target_velocity: Velocity,

    /// 목표 가속도
    pub target_acceleration: Acceleration,

    /// 사용 중인 티켓 ID
    pub ticket_id: u128,
}

impl MotionCommand {
    /// 새 MotionCommand 생성
    pub fn new(robot_id: u64) -> Self {
        Self {
            robot_id,
            current_position: Position::ORIGIN,
            target_velocity: Velocity::ZERO,
            target_acceleration: Acceleration::ZERO,
            ticket_id: 0,
        }
    }

    /// 위치 설정
    pub fn with_position(mut self, position: Position) -> Self {
        self.current_position = position;
        self
    }

    /// 목표 속도 설정
    pub fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.target_velocity = velocity;
        self
    }

    /// 목표 가속도 설정
    pub fn with_acceleration(mut self, acceleration: Acceleration) -> Self {
        self.target_acceleration = acceleration;
        self
    }

    /// 티켓 설정
    pub fn with_ticket(mut self, ticket_id: u128) -> Self {
        self.ticket_id = ticket_id;
        self
    }

    /// 목표 속력 (속도 크기)
    #[inline]
    pub fn target_speed(&self) -> f32 {
        self.target_velocity.magnitude()
    }

    /// 정지 명령인지 확인
    #[inline]
    pub fn is_stop_command(&self) -> bool {
        self.target_velocity.magnitude() < 0.001 && self.target_acceleration.magnitude() < 0.001
    }
}

impl Hash for MotionCommand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.robot_id.hash(state);
        self.ticket_id.hash(state);
        // 위치/속도는 f32이므로 비트 변환 후 해시
        self.target_velocity.vx.to_bits().hash(state);
        self.target_velocity.vy.to_bits().hash(state);
        self.target_velocity.vz.to_bits().hash(state);
    }
}

impl Default for MotionCommand {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motion_command_new() {
        let cmd = MotionCommand::new(42);
        assert_eq!(cmd.robot_id, 42);
        assert!(cmd.is_stop_command());
    }

    #[test]
    fn test_motion_command_builder() {
        let cmd = MotionCommand::new(1)
            .with_position(Position::new(10.0, 20.0, 0.0))
            .with_velocity(Velocity::new(1.0, 0.0, 0.0))
            .with_ticket(100);

        assert_eq!(cmd.current_position.x, 10.0);
        assert_eq!(cmd.target_speed(), 1.0);
        assert_eq!(cmd.ticket_id, 100);
    }

    #[test]
    fn test_motion_command_is_stop() {
        let moving = MotionCommand::new(1).with_velocity(Velocity::new(1.0, 0.0, 0.0));
        assert!(!moving.is_stop_command());

        let stopped = MotionCommand::new(1);
        assert!(stopped.is_stop_command());
    }

    #[test]
    fn test_motion_command_hash() {
        use std::collections::hash_map::DefaultHasher;

        let cmd1 = MotionCommand::new(1).with_velocity(Velocity::new(1.0, 0.0, 0.0));
        let cmd2 = MotionCommand::new(1).with_velocity(Velocity::new(1.0, 0.0, 0.0));
        let cmd3 = MotionCommand::new(1).with_velocity(Velocity::new(2.0, 0.0, 0.0));

        let mut hasher1 = DefaultHasher::new();
        cmd1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        cmd2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        let mut hasher3 = DefaultHasher::new();
        cmd3.hash(&mut hasher3);
        let hash3 = hasher3.finish();

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
