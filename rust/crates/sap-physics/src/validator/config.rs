//! PhysicsValidator 설정

use sap_core::types::Acceleration;

/// PhysicsValidator 설정
#[derive(Debug, Clone)]
pub struct PhysicsValidatorConfig {
    /// 최대 속도 (m/s)
    pub max_velocity: f32,

    /// 최대 가속도 (m/s²)
    pub max_acceleration: f32,

    /// 최대 저크 (m/s³)
    pub max_jerk: f32,

    /// 충돌 안전 거리 (m)
    pub collision_safety_distance: f32,

    /// 충돌 예측 시간 범위 (초)
    pub collision_horizon_secs: f32,

    /// 롤백 델타 임계값 (m)
    pub rollback_delta_threshold: f32,
}

impl Default for PhysicsValidatorConfig {
    fn default() -> Self {
        Self {
            max_velocity: 5.0,                       // 5 m/s
            max_acceleration: Acceleration::GRAVITY, // 9.8 m/s²
            max_jerk: 50.0,                          // 50 m/s³
            collision_safety_distance: 1.0,          // 1m
            collision_horizon_secs: 1.0,             // 1초
            rollback_delta_threshold: 0.1,           // 10cm
        }
    }
}

impl PhysicsValidatorConfig {
    /// 공장/물류센터용 설정 (보수적)
    pub fn warehouse() -> Self {
        Self {
            max_velocity: 3.0,
            max_acceleration: 5.0,
            max_jerk: 30.0,
            collision_safety_distance: 1.5,
            collision_horizon_secs: 1.5,
            rollback_delta_threshold: 0.05,
        }
    }

    /// 고속 물류용 설정
    pub fn high_speed_logistics() -> Self {
        Self {
            max_velocity: 8.0,
            max_acceleration: 8.0,
            max_jerk: 60.0,
            collision_safety_distance: 2.0,
            collision_horizon_secs: 2.0,
            rollback_delta_threshold: 0.15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PhysicsValidatorConfig::default();
        assert_eq!(config.max_velocity, 5.0);
        assert!((config.max_acceleration - 9.8).abs() < 0.01);
    }

    #[test]
    fn test_warehouse_config() {
        let config = PhysicsValidatorConfig::warehouse();
        assert_eq!(config.max_velocity, 3.0);
        assert_eq!(config.collision_safety_distance, 1.5);
    }
}
