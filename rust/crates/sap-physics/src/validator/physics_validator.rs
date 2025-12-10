//! PhysicsValidator - 물리 검증기 메인
//!
//! PPR 매핑: AI_make_PhysicsValidator

use super::PhysicsValidatorConfig;
use crate::command::MotionCommand;
use crate::constraint::{CollisionPredictor, KinematicsChecker};
use sap_core::{
    types::Position,
    validation::{constraint_ids, ValidationFrame, ValidationResult},
};

/// PhysicsValidator - L2 TrustOS 물리 검증기
///
/// PPR: AI_make_PhysicsValidator(cmd, obstacles) -> ValidationResult
pub struct PhysicsValidator {
    /// 설정
    config: PhysicsValidatorConfig,

    /// 동역학 검사기
    kinematics_checker: KinematicsChecker,

    /// 충돌 예측기
    collision_predictor: CollisionPredictor,

    /// 검증 로그 (최근 N개)
    validation_log: Vec<ValidationLogEntry>,

    /// 로그 최대 크기
    log_capacity: usize,
}

/// 검증 로그 항목
#[derive(Debug, Clone)]
pub struct ValidationLogEntry {
    /// 로봇 ID
    pub robot_id: u64,

    /// 결과
    pub result: ValidationResult,

    /// 타임스탬프 (나노초)
    pub timestamp_ns: u64,

    /// 상세 정보
    pub details: String,
}

impl PhysicsValidator {
    /// 새 PhysicsValidator 생성
    pub fn new(config: PhysicsValidatorConfig) -> Self {
        let kinematics_checker = KinematicsChecker::new(
            config.max_velocity,
            config.max_acceleration,
            config.max_jerk,
        );

        let collision_predictor = CollisionPredictor::new(
            config.collision_safety_distance,
            config.collision_horizon_secs,
        );

        Self {
            config,
            kinematics_checker,
            collision_predictor,
            validation_log: Vec::new(),
            log_capacity: 1000,
        }
    }

    /// 기본 설정으로 생성
    pub fn with_default_config() -> Self {
        Self::new(PhysicsValidatorConfig::default())
    }

    /// 명령 검증 (PPR: AI_make_PhysicsValidator)
    pub fn validate(
        &mut self,
        cmd: &MotionCommand,
        obstacles: &[Position],
        timestamp_ns: u64,
    ) -> ValidationResult {
        let kinematics_result = self
            .kinematics_checker
            .check(&cmd.target_velocity, &cmd.target_acceleration);

        let collision_result = self.collision_predictor.predict(
            &cmd.current_position,
            &cmd.target_velocity,
            obstacles,
        );

        let result = self.determine_result(&kinematics_result, &collision_result);

        self.log_validation(
            cmd.robot_id,
            result,
            timestamp_ns,
            &kinematics_result,
            &collision_result,
        );

        result
    }

    /// 검증 프레임 생성
    pub fn create_validation_frame(
        &self,
        cmd: &MotionCommand,
        result: ValidationResult,
        tick: u64,
        zone_id: u32,
    ) -> ValidationFrame {
        let mut frame = ValidationFrame::new(tick, cmd.robot_id, zone_id);

        let cmd_hash = sap_core::util::compute_hash(cmd);
        frame = frame.with_cmd_hash(cmd_hash);

        if result.is_ok() {
            frame.set_constraint(constraint_ids::VELOCITY_LIMIT, true);
            frame.set_constraint(constraint_ids::ACCELERATION_LIMIT, true);
            frame.set_constraint(constraint_ids::JERK_LIMIT, true);
            frame.set_constraint(constraint_ids::COLLISION_PREDICTION, true);
        }

        frame
    }

    fn determine_result(
        &self,
        kinematics: &KinematicsResult,
        collision: &CollisionResult,
    ) -> ValidationResult {
        if collision.will_collide {
            return ValidationResult::REJECT;
        }

        if !kinematics.velocity_ok || !kinematics.acceleration_ok || !kinematics.jerk_ok {
            return ValidationResult::ADJUST;
        }

        ValidationResult::OK
    }

    fn log_validation(
        &mut self,
        robot_id: u64,
        result: ValidationResult,
        timestamp_ns: u64,
        kinematics: &KinematicsResult,
        collision: &CollisionResult,
    ) {
        let details = format!(
            "vel_ok={}, accel_ok={}, collision={}",
            kinematics.velocity_ok, kinematics.acceleration_ok, collision.will_collide
        );

        let entry = ValidationLogEntry {
            robot_id,
            result,
            timestamp_ns,
            details,
        };

        if self.validation_log.len() >= self.log_capacity {
            self.validation_log.remove(0);
        }
        self.validation_log.push(entry);
    }

    /// 최근 검증 로그 조회
    pub fn recent_logs(&self, count: usize) -> &[ValidationLogEntry] {
        let start = self.validation_log.len().saturating_sub(count);
        &self.validation_log[start..]
    }

    /// 설정 조회
    pub fn config(&self) -> &PhysicsValidatorConfig {
        &self.config
    }
}

/// 동역학 검사 결과
#[derive(Debug, Clone)]
pub struct KinematicsResult {
    pub velocity_ok: bool,
    pub acceleration_ok: bool,
    pub jerk_ok: bool,
    pub actual_velocity: f32,
    pub actual_acceleration: f32,
}

/// 충돌 예측 결과
#[derive(Debug, Clone)]
pub struct CollisionResult {
    pub will_collide: bool,
    pub time_to_collision: Option<f32>,
    pub nearest_obstacle_distance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sap_core::types::{Acceleration, Velocity};

    fn create_test_command(vel_magnitude: f32) -> MotionCommand {
        MotionCommand {
            robot_id: 1,
            current_position: Position::ORIGIN,
            target_velocity: Velocity::new(vel_magnitude, 0.0, 0.0),
            target_acceleration: Acceleration::new(1.0, 0.0, 0.0),
            ticket_id: 1,
        }
    }

    #[test]
    fn test_validator_ok() {
        let mut validator = PhysicsValidator::with_default_config();
        let cmd = create_test_command(2.0);

        let result = validator.validate(&cmd, &[], 0);
        assert_eq!(result, ValidationResult::OK);
    }

    #[test]
    fn test_validator_adjust_velocity() {
        let mut validator = PhysicsValidator::with_default_config();
        let cmd = create_test_command(10.0);

        let result = validator.validate(&cmd, &[], 0);
        assert_eq!(result, ValidationResult::ADJUST);
    }

    #[test]
    fn test_validator_reject_collision() {
        let mut validator = PhysicsValidator::with_default_config();
        let cmd = create_test_command(1.0);
        let obstacles = vec![Position::new(0.5, 0.0, 0.0)];

        let result = validator.validate(&cmd, &obstacles, 0);
        assert_eq!(result, ValidationResult::REJECT);
    }

    #[test]
    fn test_validation_frame_creation() {
        let validator = PhysicsValidator::with_default_config();
        let cmd = create_test_command(2.0);

        let frame = validator.create_validation_frame(&cmd, ValidationResult::OK, 100, 1);

        assert_eq!(frame.tick, 100);
        assert_eq!(frame.robot_id, 1);
        assert!(frame.check_constraint(constraint_ids::VELOCITY_LIMIT));
    }

    #[test]
    fn test_validation_logging() {
        let mut validator = PhysicsValidator::with_default_config();
        let cmd = create_test_command(2.0);

        validator.validate(&cmd, &[], 1000);
        validator.validate(&cmd, &[], 2000);
        validator.validate(&cmd, &[], 3000);

        let logs = validator.recent_logs(2);
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[1].timestamp_ns, 3000);
    }
}
