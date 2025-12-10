//! 검증 결과 타입
//!
//! PPR 매핑: AI_response_ValidationResult

use serde::{Deserialize, Serialize};

/// 물리 검증 결과
///
/// PPR: AI_response_ValidationResult(OK|ADJUST|REJECT)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ValidationResult {
    /// 명령 통과 - 그대로 실행 가능
    #[default]
    OK = 0,

    /// 조정 필요 - 속도/가속도 제한 후 실행
    ADJUST = 1,

    /// 거부 - 명령 실행 불가 (충돌 위험 등)
    REJECT = 2,
}

impl ValidationResult {
    /// 통과 여부 (OK 또는 ADJUST)
    #[inline]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::OK | Self::ADJUST)
    }

    /// 완전 통과 여부 (OK만)
    #[inline]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::OK)
    }

    /// 거부 여부
    #[inline]
    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::REJECT)
    }

    /// u8로 변환
    #[inline]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// u8에서 변환
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::OK),
            1 => Some(Self::ADJUST),
            2 => Some(Self::REJECT),
            _ => None,
        }
    }
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OK => write!(f, "OK"),
            Self::ADJUST => write!(f, "ADJUST"),
            Self::REJECT => write!(f, "REJECT"),
        }
    }
}

/// 상세한 검증 결과 (이유 포함)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResultDetail {
    /// 기본 결과
    pub result: ValidationResult,

    /// 거부/조정 이유 (있으면)
    pub reason: Option<ValidationReason>,

    /// 조정된 명령 (ADJUST 시 사용)
    pub adjusted_command: Option<AdjustedCommand>,

    /// 동역학 검사 통과 여부
    pub kinematics_ok: bool,

    /// 충돌 검사 통과 여부
    pub collision_ok: bool,

    /// 제약조건 검사 통과 여부
    pub constraint_ok: bool,
}

/// 조정된 명령
///
/// ADJUST 결과 시 원래 명령 대신 사용해야 하는 안전한 값
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustedCommand {
    /// 조정된 선속도 (m/s)
    pub adjusted_velocity: f32,
    /// 조정된 각속도 (rad/s)  
    pub adjusted_angular_velocity: f32,
    /// 조정된 가속도 (m/s²)
    pub adjusted_acceleration: f32,
    /// 원래 값 대비 조정 비율 (0.0~1.0)
    pub scale_factor: f32,
    /// 조정 이유 설명
    pub adjustment_note: Option<String>,
}

impl AdjustedCommand {
    /// 속도 스케일링으로 조정된 명령 생성
    pub fn scaled(original_velocity: f32, scale_factor: f32) -> Self {
        Self {
            adjusted_velocity: original_velocity * scale_factor,
            adjusted_angular_velocity: 0.0,
            adjusted_acceleration: 0.0,
            scale_factor,
            adjustment_note: Some(format!("Velocity scaled to {:.0}%", scale_factor * 100.0)),
        }
    }

    /// 최대값으로 클램핑된 명령 생성
    pub fn clamped(max_velocity: f32, max_acceleration: f32) -> Self {
        Self {
            adjusted_velocity: max_velocity,
            adjusted_angular_velocity: 0.0,
            adjusted_acceleration: max_acceleration,
            scale_factor: 1.0,
            adjustment_note: Some("Clamped to physical limits".to_string()),
        }
    }
}

/// 검증 실패 이유
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationReason {
    /// 최대 속도 초과
    VelocityExceeded { actual: f32, limit: f32 },

    /// 최대 가속도 초과
    AccelerationExceeded { actual: f32, limit: f32 },

    /// 최대 저크 초과
    JerkExceeded { actual: f32, limit: f32 },

    /// 충돌 예측
    CollisionPredicted {
        time_to_collision: f32,
        obstacle_id: u64,
    },

    /// 지오펜스 위반
    GeofenceViolation { geofence_id: u32 },

    /// 티켓 없음/만료
    InvalidTicket { ticket_id: u128 },

    /// VoxelTimeSlot 위반
    VTSViolation { vts_id: u64 },
}

#[allow(dead_code)]
impl ValidationResultDetail {
    /// 모두 통과한 결과 생성
    pub fn ok() -> Self {
        Self {
            result: ValidationResult::OK,
            reason: None,
            adjusted_command: None,
            kinematics_ok: true,
            collision_ok: true,
            constraint_ok: true,
        }
    }

    /// 거부 결과 생성
    pub fn reject(reason: ValidationReason) -> Self {
        Self {
            result: ValidationResult::REJECT,
            reason: Some(reason),
            adjusted_command: None,
            kinematics_ok: false,
            collision_ok: false,
            constraint_ok: false,
        }
    }

    /// 조정 필요 결과 생성 (조정된 명령 포함)
    pub fn adjust(reason: ValidationReason, adjusted: AdjustedCommand) -> Self {
        Self {
            result: ValidationResult::ADJUST,
            reason: Some(reason),
            adjusted_command: Some(adjusted),
            kinematics_ok: false,
            collision_ok: true,
            constraint_ok: true,
        }
    }

    /// 조정 필요 결과 생성 (조정된 명령 없이 - 레거시 호환)
    pub fn adjust_without_command(reason: ValidationReason) -> Self {
        Self {
            result: ValidationResult::ADJUST,
            reason: Some(reason),
            adjusted_command: None,
            kinematics_ok: false,
            collision_ok: true,
            constraint_ok: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_is_allowed() {
        assert!(ValidationResult::OK.is_allowed());
        assert!(ValidationResult::ADJUST.is_allowed());
        assert!(!ValidationResult::REJECT.is_allowed());
    }

    #[test]
    fn test_validation_result_from_u8() {
        assert_eq!(ValidationResult::from_u8(0), Some(ValidationResult::OK));
        assert_eq!(ValidationResult::from_u8(1), Some(ValidationResult::ADJUST));
        assert_eq!(ValidationResult::from_u8(2), Some(ValidationResult::REJECT));
        assert_eq!(ValidationResult::from_u8(3), None);
    }

    #[test]
    fn test_validation_result_detail_ok() {
        let detail = ValidationResultDetail::ok();
        assert!(detail.result.is_ok());
        assert!(detail.kinematics_ok);
        assert!(detail.collision_ok);
    }

    #[test]
    fn test_validation_result_detail_reject() {
        let detail = ValidationResultDetail::reject(ValidationReason::CollisionPredicted {
            time_to_collision: 0.5,
            obstacle_id: 42,
        });
        assert!(detail.result.is_rejected());
        assert!(detail.reason.is_some());
    }
}
