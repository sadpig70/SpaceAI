//! 로봇 프로파일 정의
//!
//! 다양한 로봇 유형 (Ackermann, Differential, Mecanum, Omnidirectional)의
//! 운동학적 제약을 정의합니다.
//!
//! PPR 매핑: AI_make_VehicleProfile

use serde::{Deserialize, Serialize};

/// 로봇 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum VehicleType {
    /// Ackermann 조향 (자동차형)
    /// - 전륜 조향, 후륜 구동
    /// - 최소 회전 반경 존재
    Ackermann,

    /// Differential Drive (차동 구동)
    /// - 양쪽 바퀴 독립 제어
    /// - 제자리 회전 가능
    #[default]
    Differential,

    /// Mecanum Wheel (메카넘 휠)
    /// - 전방향 이동 가능
    /// - 횡이동(스트래이프) 지원
    Mecanum,

    /// Omnidirectional (홀로노믹)
    /// - 완전한 전방향 이동
    /// - 회전과 이동 독립
    Omnidirectional,

    /// Tracked (무한궤도)
    /// - 험지 주행 가능
    /// - 회전 시 슬립 발생
    Tracked,
}

impl VehicleType {
    /// 제자리 회전 가능 여부
    pub fn can_pivot(&self) -> bool {
        !matches!(self, Self::Ackermann)
    }

    /// 횡이동 가능 여부
    pub fn can_strafe(&self) -> bool {
        matches!(self, Self::Mecanum | Self::Omnidirectional)
    }

    /// 홀로노믹 여부 (모든 방향 독립 제어)
    pub fn is_holonomic(&self) -> bool {
        matches!(self, Self::Omnidirectional)
    }
}

/// 운동학 파라미터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinematicsParams {
    /// 최대 선속도 (m/s)
    pub max_velocity: f32,
    /// 최대 가속도 (m/s²)
    pub max_acceleration: f32,
    /// 최대 감속도 (m/s²) - 일반적으로 가속도보다 큼
    pub max_deceleration: f32,
    /// 최대 각속도 (rad/s)
    pub max_angular_velocity: f32,
    /// 최대 각가속도 (rad/s²)
    pub max_angular_acceleration: f32,
    /// 최대 저크 (m/s³)
    pub max_jerk: f32,
    /// 최소 회전 반경 (m) - Ackermann에서 중요
    pub min_turning_radius: f32,
}

impl Default for KinematicsParams {
    fn default() -> Self {
        Self {
            max_velocity: 2.0,                              // 2 m/s
            max_acceleration: 1.5,                          // 1.5 m/s²
            max_deceleration: 3.0,                          // 3 m/s² (비상 시 더 빠르게)
            max_angular_velocity: 1.57,                     // 90°/s ≈ 1.57 rad/s
            max_angular_acceleration: std::f32::consts::PI, // 180°/s²
            max_jerk: 10.0,                                 // 10 m/s³
            min_turning_radius: 0.0,                        // 제자리 회전 가능
        }
    }
}

impl KinematicsParams {
    /// 정지 거리 계산 (현재 속력 기준)
    pub fn stopping_distance(&self, current_speed: f32) -> f32 {
        // d = v² / (2a)
        if self.max_deceleration <= 0.0 {
            return f32::MAX;
        }
        (current_speed * current_speed) / (2.0 * self.max_deceleration)
    }

    /// 정지 시간 계산 (현재 속력 기준)
    pub fn stopping_time(&self, current_speed: f32) -> f32 {
        // t = v / a
        if self.max_deceleration <= 0.0 {
            return f32::MAX;
        }
        current_speed / self.max_deceleration
    }
}

/// 로봇 프로파일 (유형 + 파라미터)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleProfile {
    /// 로봇 유형
    pub vehicle_type: VehicleType,
    /// 운동학 파라미터
    pub kinematics: KinematicsParams,
    /// 로봇 폭 (m)
    pub width: f32,
    /// 로봇 길이 (m)
    pub length: f32,
    /// 로봇 높이 (m)
    pub height: f32,
    /// 안전 마진 (m) - 충돌 검사 시 추가
    pub safety_margin: f32,
}

impl Default for VehicleProfile {
    fn default() -> Self {
        Self {
            vehicle_type: VehicleType::default(),
            kinematics: KinematicsParams::default(),
            width: 0.6,
            length: 0.8,
            height: 1.2,
            safety_margin: 0.1,
        }
    }
}

impl VehicleProfile {
    /// AMR (자율 이동 로봇) 프로파일
    pub fn amr() -> Self {
        Self {
            vehicle_type: VehicleType::Differential,
            kinematics: KinematicsParams {
                max_velocity: 2.0,
                max_acceleration: 1.5,
                max_deceleration: 3.0,
                max_angular_velocity: 1.57,
                max_angular_acceleration: std::f32::consts::PI,
                max_jerk: 10.0,
                min_turning_radius: 0.0,
            },
            width: 0.6,
            length: 0.8,
            height: 1.2,
            safety_margin: 0.1,
        }
    }

    /// AGV (자동 유도 차량) 프로파일 - Ackermann
    pub fn agv() -> Self {
        Self {
            vehicle_type: VehicleType::Ackermann,
            kinematics: KinematicsParams {
                max_velocity: 1.5,
                max_acceleration: 1.0,
                max_deceleration: 2.5,
                max_angular_velocity: 0.5,
                max_angular_acceleration: 1.0,
                max_jerk: 5.0,
                min_turning_radius: 2.0, // 최소 2m 회전 반경
            },
            width: 1.0,
            length: 2.0,
            height: 1.5,
            safety_margin: 0.2,
        }
    }

    /// 메카넘 휠 로봇 프로파일
    pub fn mecanum() -> Self {
        Self {
            vehicle_type: VehicleType::Mecanum,
            kinematics: KinematicsParams {
                max_velocity: 1.2,
                max_acceleration: 1.0,
                max_deceleration: 2.0,
                max_angular_velocity: 1.0,
                max_angular_acceleration: 2.0,
                max_jerk: 8.0,
                min_turning_radius: 0.0,
            },
            width: 0.5,
            length: 0.5,
            height: 0.3,
            safety_margin: 0.05,
        }
    }

    /// 바운딩 반경 (대각선 길이의 절반 + 안전 마진)
    pub fn bounding_radius(&self) -> f32 {
        let diag = (self.width.powi(2) + self.length.powi(2)).sqrt();
        diag / 2.0 + self.safety_margin
    }

    /// 정지 거리 (현재 속력 기준)
    pub fn stopping_distance(&self, current_speed: f32) -> f32 {
        self.kinematics.stopping_distance(current_speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_type_capabilities() {
        assert!(!VehicleType::Ackermann.can_pivot());
        assert!(VehicleType::Differential.can_pivot());
        assert!(VehicleType::Mecanum.can_strafe());
        assert!(!VehicleType::Differential.can_strafe());
        assert!(VehicleType::Omnidirectional.is_holonomic());
    }

    #[test]
    fn test_kinematics_stopping() {
        let params = KinematicsParams::default();
        // v=3, a=3 -> d = 9/6 = 1.5m
        let dist = params.stopping_distance(3.0);
        assert!((dist - 1.5).abs() < 0.01);

        // v=3, a=3 -> t = 1s
        let time = params.stopping_time(3.0);
        assert!((time - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_vehicle_profile_presets() {
        let amr = VehicleProfile::amr();
        assert!(amr.vehicle_type.can_pivot());

        let agv = VehicleProfile::agv();
        assert!(!agv.vehicle_type.can_pivot());
        assert!(agv.kinematics.min_turning_radius > 0.0);

        let mecanum = VehicleProfile::mecanum();
        assert!(mecanum.vehicle_type.can_strafe());
    }

    #[test]
    fn test_bounding_radius() {
        let profile = VehicleProfile::default();
        let radius = profile.bounding_radius();
        // sqrt(0.6² + 0.8²) / 2 + 0.1 = 0.5 + 0.1 = 0.6
        assert!((radius - 0.6).abs() < 0.01);
    }
}
