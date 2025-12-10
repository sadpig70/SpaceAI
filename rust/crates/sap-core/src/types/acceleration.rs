//! 3D 가속도 타입
//!
//! PPR 매핑: AI_process_MaxAcceleration, AI_process_MaxJerk

use serde::{Deserialize, Serialize};

/// 3D 가속도 벡터 (m/s²)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[repr(C)]
pub struct Acceleration {
    /// X 방향 가속도 (m/s²)
    pub ax: f32,
    /// Y 방향 가속도 (m/s²)
    pub ay: f32,
    /// Z 방향 가속도 (m/s²)
    pub az: f32,
}

impl Acceleration {
    /// 새 Acceleration 생성
    #[inline]
    pub const fn new(ax: f32, ay: f32, az: f32) -> Self {
        Self { ax, ay, az }
    }

    /// 가속도 없음 (0, 0, 0)
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// 기본 중력 가속도 상수 (9.8 m/s²)
    pub const GRAVITY: f32 = 9.8;

    /// 가속도 크기
    #[inline]
    pub fn magnitude(&self) -> f32 {
        (self.ax * self.ax + self.ay * self.ay + self.az * self.az).sqrt()
    }

    /// 최대 가속도 제한 검사
    ///
    /// PPR: AI_process_MaxAcceleration
    #[inline]
    pub fn within_limit(&self, max_accel: f32) -> bool {
        self.magnitude() <= max_accel
    }

    /// 저크(Jerk) 계산 - 가속도 변화율 (m/s³)
    ///
    /// PPR: AI_process_MaxJerk
    #[inline]
    pub fn jerk(&self, prev: &Self, dt: f32) -> f32 {
        if dt == 0.0 {
            return 0.0;
        }
        let dax = self.ax - prev.ax;
        let day = self.ay - prev.ay;
        let daz = self.az - prev.az;
        (dax * dax + day * day + daz * daz).sqrt() / dt
    }

    /// 스칼라 곱
    #[inline]
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            ax: self.ax * factor,
            ay: self.ay * factor,
            az: self.az * factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceleration_magnitude() {
        let accel = Acceleration::new(3.0, 4.0, 0.0);
        assert!((accel.magnitude() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_acceleration_within_limit() {
        let accel = Acceleration::new(5.0, 0.0, 0.0);
        assert!(accel.within_limit(Acceleration::GRAVITY));
        assert!(!accel.within_limit(4.0));
    }

    #[test]
    fn test_acceleration_jerk() {
        let prev = Acceleration::new(0.0, 0.0, 0.0);
        let curr = Acceleration::new(5.0, 0.0, 0.0);
        let dt = 0.1; // 100ms
        let jerk = curr.jerk(&prev, dt);
        assert!((jerk - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_acceleration_jerk_zero_dt() {
        let prev = Acceleration::new(0.0, 0.0, 0.0);
        let curr = Acceleration::new(5.0, 0.0, 0.0);
        let jerk = curr.jerk(&prev, 0.0);
        assert_eq!(jerk, 0.0);
    }
}
