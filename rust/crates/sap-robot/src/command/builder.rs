//! CommandBuilder - 명령 생성 및 서명
//!
//! PPR 매핑: AI_make_MotionCommand

use sap_core::types::{Acceleration, Position, Velocity};

/// 명령 빌더
///
/// 로봇 이동 명령을 생성하고 서명
pub struct CommandBuilder {
    robot_id: u64,
    current_position: Position,
    target_velocity: Velocity,
    target_acceleration: Acceleration,
    ticket_id: u128,
    priority: u8,
}

/// 빌드된 명령
#[derive(Debug, Clone)]
pub struct RobotCommand {
    pub robot_id: u64,
    pub current_position: Position,
    pub target_velocity: Velocity,
    pub target_acceleration: Acceleration,
    pub ticket_id: u128,
    pub priority: u8,
    pub timestamp_ns: u64,
    pub sequence: u64,
}

impl CommandBuilder {
    /// 새 CommandBuilder 생성
    pub fn new(robot_id: u64) -> Self {
        Self {
            robot_id,
            current_position: Position::ORIGIN,
            target_velocity: Velocity::ZERO,
            target_acceleration: Acceleration::ZERO,
            ticket_id: 0,
            priority: 0,
        }
    }

    /// 현재 위치 설정
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

    /// 티켓 ID 설정
    pub fn with_ticket(mut self, ticket_id: u128) -> Self {
        self.ticket_id = ticket_id;
        self
    }

    /// 우선순위 설정
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// 이동 명령 생성 (속도 기반)
    pub fn move_to_velocity(mut self, vx: f32, vy: f32) -> Self {
        self.target_velocity = Velocity::new(vx, vy, 0.0);
        self
    }

    /// 정지 명령 생성
    pub fn stop(mut self) -> Self {
        self.target_velocity = Velocity::ZERO;
        self.target_acceleration = Acceleration::ZERO;
        self
    }

    /// 명령 빌드
    pub fn build(self, timestamp_ns: u64, sequence: u64) -> RobotCommand {
        RobotCommand {
            robot_id: self.robot_id,
            current_position: self.current_position,
            target_velocity: self.target_velocity,
            target_acceleration: self.target_acceleration,
            ticket_id: self.ticket_id,
            priority: self.priority,
            timestamp_ns,
            sequence,
        }
    }

    /// 명령 유효성 검사
    pub fn validate(&self) -> Result<(), CommandError> {
        let speed = self.target_velocity.magnitude();
        if speed > 5.0 {
            return Err(CommandError::SpeedTooHigh {
                max: 5.0,
                actual: speed,
            });
        }

        let accel = self.target_acceleration.magnitude();
        if accel > 3.0 {
            return Err(CommandError::AccelerationTooHigh {
                max: 3.0,
                actual: accel,
            });
        }

        if self.ticket_id == 0 && self.target_velocity.magnitude() > 0.0 {
            return Err(CommandError::NoTicket);
        }

        Ok(())
    }
}

impl RobotCommand {
    /// 속도 크기
    pub fn speed(&self) -> f32 {
        self.target_velocity.magnitude()
    }

    /// 정지 명령인지 확인
    pub fn is_stop(&self) -> bool {
        self.target_velocity.magnitude() < 0.001
    }
}

/// 명령 오류
#[derive(Debug, Clone)]
pub enum CommandError {
    SpeedTooHigh { max: f32, actual: f32 },
    AccelerationTooHigh { max: f32, actual: f32 },
    NoTicket,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = CommandBuilder::new(42);
        let cmd = builder.build(1_000_000_000, 1);

        assert_eq!(cmd.robot_id, 42);
        assert_eq!(cmd.current_position, Position::ORIGIN);
        assert!(cmd.is_stop());
    }

    #[test]
    fn test_builder_chain() {
        let cmd = CommandBuilder::new(42)
            .with_position(Position::new(1.0, 2.0, 0.0))
            .with_velocity(Velocity::new(1.0, 0.0, 0.0))
            .with_ticket(100)
            .build(1_000_000_000, 1);

        assert_eq!(cmd.current_position.x, 1.0);
        assert_eq!(cmd.target_velocity.vx, 1.0);
        assert_eq!(cmd.ticket_id, 100);
    }

    #[test]
    fn test_move_to_velocity() {
        let cmd = CommandBuilder::new(42)
            .move_to_velocity(2.0, 1.0)
            .with_ticket(1)
            .build(0, 0);

        assert_eq!(cmd.target_velocity.vx, 2.0);
        assert_eq!(cmd.target_velocity.vy, 1.0);
    }

    #[test]
    fn test_stop() {
        let cmd = CommandBuilder::new(42)
            .with_velocity(Velocity::new(5.0, 0.0, 0.0))
            .stop()
            .build(0, 0);

        assert!(cmd.is_stop());
    }

    #[test]
    fn test_validate_speed() {
        let builder = CommandBuilder::new(42).with_velocity(Velocity::new(10.0, 0.0, 0.0));

        assert!(matches!(
            builder.validate(),
            Err(CommandError::SpeedTooHigh { .. })
        ));
    }

    #[test]
    fn test_validate_no_ticket() {
        let builder = CommandBuilder::new(42).with_velocity(Velocity::new(1.0, 0.0, 0.0));

        assert!(matches!(builder.validate(), Err(CommandError::NoTicket)));
    }

    #[test]
    fn test_validate_ok() {
        let builder = CommandBuilder::new(42)
            .with_velocity(Velocity::new(1.0, 0.0, 0.0))
            .with_ticket(1);

        assert!(builder.validate().is_ok());
    }
}
