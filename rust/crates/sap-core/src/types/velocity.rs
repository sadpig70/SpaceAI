//! 3D 속도 타입
//!
//! PPR 매핑: AI_perceive_RobotState, AI_process_KinematicsCheck

use serde::{Deserialize, Serialize};

/// 3D 속도 벡터 (m/s)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[repr(C)]
pub struct Velocity {
    /// X 방향 속도 (m/s)
    pub vx: f32,
    /// Y 방향 속도 (m/s)
    pub vy: f32,
    /// Z 방향 속도 (m/s)
    pub vz: f32,
}

impl Velocity {
    /// 새 Velocity 생성
    #[inline]
    pub const fn new(vx: f32, vy: f32, vz: f32) -> Self {
        Self { vx, vy, vz }
    }

    /// 정지 상태 (0, 0, 0)
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// 속도 크기 (speed)
    #[inline]
    pub fn magnitude(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }

    /// 2D 속도 크기 (XY 평면)
    #[inline]
    pub fn magnitude_2d(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy).sqrt()
    }

    /// 최대 속도 제한 검사
    ///
    /// PPR: AI_process_MaxVelocity
    #[inline]
    pub fn within_limit(&self, max_speed: f32) -> bool {
        self.magnitude() <= max_speed
    }

    /// 속도 제한 적용 (클램핑)
    #[inline]
    pub fn clamp(&self, max_speed: f32) -> Self {
        let mag = self.magnitude();
        if mag <= max_speed || mag == 0.0 {
            *self
        } else {
            let scale = max_speed / mag;
            Self {
                vx: self.vx * scale,
                vy: self.vy * scale,
                vz: self.vz * scale,
            }
        }
    }

    /// 스칼라 곱
    #[inline]
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            vx: self.vx * factor,
            vy: self.vy * factor,
            vz: self.vz * factor,
        }
    }
}

impl std::ops::Add for Velocity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            vx: self.vx + rhs.vx,
            vy: self.vy + rhs.vy,
            vz: self.vz + rhs.vz,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_magnitude() {
        let vel = Velocity::new(3.0, 4.0, 0.0);
        assert!((vel.magnitude() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_velocity_within_limit() {
        let vel = Velocity::new(3.0, 4.0, 0.0); // magnitude = 5.0
        assert!(vel.within_limit(5.0));
        assert!(vel.within_limit(6.0));
        assert!(!vel.within_limit(4.0));
    }

    #[test]
    fn test_velocity_clamp() {
        let vel = Velocity::new(6.0, 8.0, 0.0); // magnitude = 10.0
        let clamped = vel.clamp(5.0);
        assert!((clamped.magnitude() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_velocity_scale() {
        let vel = Velocity::new(2.0, 3.0, 4.0);
        let scaled = vel.scale(0.5);
        assert_eq!(scaled, Velocity::new(1.0, 1.5, 2.0));
    }
}
