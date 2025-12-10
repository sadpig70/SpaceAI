//! 물리적 복구 명령 타입
//!
//! PPR 매핑: AI_response_RecoveryCommand

use sap_core::types::{Position, Velocity};
use serde::{Deserialize, Serialize};

/// 복구 수준
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RecoveryLevel {
    /// L0: 비상 정지 - 최고 감속으로 즉시 정지
    EmergencyStop = 0,

    /// L1: 안전 감속 - 물리 제약 내 부드러운 감속
    SafeDeceleration = 1,

    /// L2: 위치 유지 - 현재 위치에서 정지 유지
    SafeHold = 2,

    /// L3: 경로 재계획 - 새 경로로 전환
    PathReplanning = 3,
}

impl RecoveryLevel {
    /// 복구 수준의 우선순위 반환 (낮을수록 긴급)
    #[inline]
    pub fn priority(self) -> u8 {
        self as u8
    }

    /// 비상 정지 여부
    #[inline]
    pub fn is_emergency(self) -> bool {
        matches!(self, Self::EmergencyStop)
    }
}

/// 물리적 복구 명령
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCommand {
    /// 대상 로봇 ID
    pub robot_id: u64,

    /// 복구 수준
    pub level: RecoveryLevel,

    /// 목표 위치 (L2, L3에서 사용)
    pub target_position: Option<Position>,

    /// 목표 속도 (L1에서 감속 목표, 보통 0)
    pub target_velocity: Velocity,

    /// 최대 감속도 (m/s²)
    pub max_deceleration: f32,

    /// 복구 완료 후 재개 가능 여부
    pub allow_resume: bool,

    /// 복구 사유 코드
    pub reason_code: u32,

    /// 타임스탬프 (나노초)
    pub timestamp_ns: u64,
}

impl RecoveryCommand {
    /// 비상 정지 명령 생성
    pub fn emergency_stop(robot_id: u64, max_decel: f32, timestamp_ns: u64) -> Self {
        Self {
            robot_id,
            level: RecoveryLevel::EmergencyStop,
            target_position: None,
            target_velocity: Velocity::ZERO,
            max_deceleration: max_decel,
            allow_resume: false,
            reason_code: 0,
            timestamp_ns,
        }
    }

    /// 안전 감속 명령 생성
    pub fn safe_deceleration(robot_id: u64, max_decel: f32, timestamp_ns: u64) -> Self {
        Self {
            robot_id,
            level: RecoveryLevel::SafeDeceleration,
            target_position: None,
            target_velocity: Velocity::ZERO,
            max_deceleration: max_decel,
            allow_resume: true,
            reason_code: 0,
            timestamp_ns,
        }
    }

    /// 위치 유지 명령 생성
    pub fn safe_hold(robot_id: u64, position: Position, timestamp_ns: u64) -> Self {
        Self {
            robot_id,
            level: RecoveryLevel::SafeHold,
            target_position: Some(position),
            target_velocity: Velocity::ZERO,
            max_deceleration: 2.0, // 기본 감속
            allow_resume: true,
            reason_code: 0,
            timestamp_ns,
        }
    }

    /// 경로 재계획 명령 생성
    pub fn path_replanning(robot_id: u64, new_target: Position, timestamp_ns: u64) -> Self {
        Self {
            robot_id,
            level: RecoveryLevel::PathReplanning,
            target_position: Some(new_target),
            target_velocity: Velocity::ZERO,
            max_deceleration: 1.5, // 부드러운 감속
            allow_resume: true,
            reason_code: 0,
            timestamp_ns,
        }
    }

    /// 사유 코드 설정
    pub fn with_reason(mut self, code: u32) -> Self {
        self.reason_code = code;
        self
    }

    /// 재개 허용 여부 설정
    pub fn with_resume(mut self, allow: bool) -> Self {
        self.allow_resume = allow;
        self
    }

    /// 필요 정지 거리 계산 (현재 속력 기준)
    pub fn stopping_distance(&self, current_speed: f32) -> f32 {
        if self.max_deceleration <= 0.0 {
            return f32::MAX;
        }
        // d = v² / (2a)
        (current_speed * current_speed) / (2.0 * self.max_deceleration)
    }

    /// 필요 정지 시간 계산 (현재 속력 기준)
    pub fn stopping_time(&self, current_speed: f32) -> f32 {
        if self.max_deceleration <= 0.0 {
            return f32::MAX;
        }
        // t = v / a
        current_speed / self.max_deceleration
    }
}

/// 복구 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    /// 대상 로봇 ID
    pub robot_id: u64,

    /// 복구 성공 여부
    pub success: bool,

    /// 실제 정지 위치
    pub final_position: Position,

    /// 복구 소요 시간 (나노초)
    pub duration_ns: u64,

    /// 에러 메시지 (실패 시)
    pub error_message: Option<String>,
}

impl RecoveryResult {
    /// 성공 결과 생성
    pub fn success(robot_id: u64, final_position: Position, duration_ns: u64) -> Self {
        Self {
            robot_id,
            success: true,
            final_position,
            duration_ns,
            error_message: None,
        }
    }

    /// 실패 결과 생성
    pub fn failure(robot_id: u64, position: Position, error: impl Into<String>) -> Self {
        Self {
            robot_id,
            success: false,
            final_position: position,
            duration_ns: 0,
            error_message: Some(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency_stop() {
        let cmd = RecoveryCommand::emergency_stop(42, 5.0, 1_000_000_000);
        assert_eq!(cmd.robot_id, 42);
        assert_eq!(cmd.level, RecoveryLevel::EmergencyStop);
        assert!(!cmd.allow_resume);
    }

    #[test]
    fn test_safe_deceleration() {
        let cmd = RecoveryCommand::safe_deceleration(1, 3.0, 0);
        assert_eq!(cmd.level, RecoveryLevel::SafeDeceleration);
        assert!(cmd.allow_resume);
    }

    #[test]
    fn test_stopping_distance() {
        let cmd = RecoveryCommand::emergency_stop(1, 5.0, 0);
        // v=10, a=5 → d = 100/10 = 10m
        let distance = cmd.stopping_distance(10.0);
        assert!((distance - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_stopping_time() {
        let cmd = RecoveryCommand::safe_deceleration(1, 2.0, 0);
        // v=4, a=2 → t = 2s
        let time = cmd.stopping_time(4.0);
        assert!((time - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_recovery_level_priority() {
        assert_eq!(RecoveryLevel::EmergencyStop.priority(), 0);
        assert_eq!(RecoveryLevel::PathReplanning.priority(), 3);
    }

    #[test]
    fn test_recovery_result_success() {
        let result = RecoveryResult::success(42, Position::new(1.0, 2.0, 0.0), 500_000_000);
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_recovery_result_failure() {
        let result = RecoveryResult::failure(42, Position::ORIGIN, "Hardware fault");
        assert!(!result.success);
        assert!(result.error_message.is_some());
    }
}
