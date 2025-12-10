//! KinematicsChecker - 동역학 제한 검사기
//!
//! PPR 매핑: AI_process_KinematicsCheck, AI_process_MaxVelocity, AI_process_MaxAcceleration

use crate::validator::physics_validator::KinematicsResult;
use sap_core::types::{Acceleration, Velocity};

/// 동역학 제한 검사기
///
/// 속도, 가속도, 저크 제한을 검사합니다.
#[derive(Debug, Clone)]
pub struct KinematicsChecker {
    /// 최대 속도 (m/s)
    max_velocity: f32,

    /// 최대 가속도 (m/s²)
    max_acceleration: f32,

    /// 최대 저크 (m/s³)
    max_jerk: f32,

    /// 이전 가속도 (저크 계산용)
    prev_acceleration: Option<Acceleration>,

    /// 이전 시간 (저크 계산용)
    prev_time_ns: u64,
}

impl KinematicsChecker {
    /// 새 KinematicsChecker 생성
    pub fn new(max_velocity: f32, max_acceleration: f32, max_jerk: f32) -> Self {
        Self {
            max_velocity,
            max_acceleration,
            max_jerk,
            prev_acceleration: None,
            prev_time_ns: 0,
        }
    }

    /// 동역학 제한 검사 (PPR: AI_process_KinematicsCheck)
    ///
    /// # Arguments
    /// * `velocity` - 목표 속도
    /// * `acceleration` - 목표 가속도
    ///
    /// # Returns
    /// * `KinematicsResult` - 각 제약조건 통과 여부
    pub fn check(&self, velocity: &Velocity, acceleration: &Acceleration) -> KinematicsResult {
        let actual_velocity = velocity.magnitude();
        let actual_acceleration = acceleration.magnitude();

        let velocity_ok = actual_velocity <= self.max_velocity;
        let acceleration_ok = actual_acceleration <= self.max_acceleration;

        // 저크는 이전 값이 필요하므로, 없으면 OK로 처리
        let jerk_ok = true; // 간소화: 실제 저크 계산은 update_and_check에서

        KinematicsResult {
            velocity_ok,
            acceleration_ok,
            jerk_ok,
            actual_velocity,
            actual_acceleration,
        }
    }

    /// 시간 기반 저크 검사 포함 업데이트
    pub fn update_and_check(
        &mut self,
        velocity: &Velocity,
        acceleration: &Acceleration,
        current_time_ns: u64,
    ) -> KinematicsResult {
        let actual_velocity = velocity.magnitude();
        let actual_acceleration = acceleration.magnitude();

        let velocity_ok = actual_velocity <= self.max_velocity;
        let acceleration_ok = actual_acceleration <= self.max_acceleration;

        // 저크 계산
        let jerk_ok = if let Some(prev_accel) = &self.prev_acceleration {
            let dt_ns = current_time_ns.saturating_sub(self.prev_time_ns);
            if dt_ns > 0 {
                let dt_secs = dt_ns as f32 / 1_000_000_000.0;
                let jerk = acceleration.jerk(prev_accel, dt_secs);
                jerk <= self.max_jerk
            } else {
                true
            }
        } else {
            true
        };

        // 상태 업데이트
        self.prev_acceleration = Some(*acceleration);
        self.prev_time_ns = current_time_ns;

        KinematicsResult {
            velocity_ok,
            acceleration_ok,
            jerk_ok,
            actual_velocity,
            actual_acceleration,
        }
    }

    /// 속도만 검사
    #[inline]
    pub fn check_velocity(&self, velocity: &Velocity) -> bool {
        velocity.within_limit(self.max_velocity)
    }

    /// 가속도만 검사
    #[inline]
    pub fn check_acceleration(&self, acceleration: &Acceleration) -> bool {
        acceleration.within_limit(self.max_acceleration)
    }

    /// 조정된 속도 반환 (제한 내로 클램핑)
    pub fn clamp_velocity(&self, velocity: &Velocity) -> Velocity {
        velocity.clamp(self.max_velocity)
    }

    /// 상태 리셋
    pub fn reset(&mut self) {
        self.prev_acceleration = None;
        self.prev_time_ns = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_velocity_ok() {
        let checker = KinematicsChecker::new(5.0, 10.0, 50.0);
        let vel = Velocity::new(3.0, 4.0, 0.0); // magnitude = 5.0

        assert!(checker.check_velocity(&vel));
    }

    #[test]
    fn test_check_velocity_exceeded() {
        let checker = KinematicsChecker::new(5.0, 10.0, 50.0);
        let vel = Velocity::new(6.0, 8.0, 0.0); // magnitude = 10.0

        assert!(!checker.check_velocity(&vel));
    }

    #[test]
    fn test_check_full() {
        let checker = KinematicsChecker::new(5.0, 10.0, 50.0);
        let vel = Velocity::new(2.0, 1.0, 0.0);
        let accel = Acceleration::new(1.0, 0.0, 0.0);

        let result = checker.check(&vel, &accel);

        assert!(result.velocity_ok);
        assert!(result.acceleration_ok);
    }

    #[test]
    fn test_clamp_velocity() {
        let checker = KinematicsChecker::new(5.0, 10.0, 50.0);
        let vel = Velocity::new(6.0, 8.0, 0.0); // magnitude = 10.0

        let clamped = checker.clamp_velocity(&vel);
        assert!((clamped.magnitude() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_update_and_check_jerk() {
        let mut checker = KinematicsChecker::new(10.0, 10.0, 50.0);
        let vel = Velocity::new(1.0, 0.0, 0.0);

        // 첫 번째 호출
        let accel1 = Acceleration::new(0.0, 0.0, 0.0);
        let result1 = checker.update_and_check(&vel, &accel1, 0);
        assert!(result1.jerk_ok);

        // 두 번째 호출 (100ms 후, 가속도 변화 50 m/s³ = 5.0 / 0.1)
        let accel2 = Acceleration::new(5.0, 0.0, 0.0);
        let result2 = checker.update_and_check(&vel, &accel2, 100_000_000);
        assert!(result2.jerk_ok); // 50 m/s³ <= 50 limit
    }
}
