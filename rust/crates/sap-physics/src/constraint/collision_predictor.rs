//! CollisionPredictor - 충돌 예측기
//!
//! PPR 매핑: AI_process_CollisionPrediction, AI_process_SpatialIndex

use crate::validator::physics_validator::CollisionResult;
use sap_core::types::{Position, Velocity};

/// 동적 예측 지평 설정
#[derive(Debug, Clone)]
pub struct DynamicHorizonConfig {
    /// 최소 예측 지평 (초)
    pub min_horizon_secs: f32,
    /// 최대 예측 지평 (초)
    pub max_horizon_secs: f32,
    /// 정지 거리 배율 (정지 거리의 몇 배까지 예측할지)
    pub stopping_distance_multiplier: f32,
    /// 최대 감속도 (m/s²) - 정지 거리 계산용
    pub max_deceleration: f32,
    /// 반응 시간 (초) - 정지 거리에 추가
    pub reaction_time_secs: f32,
}

impl Default for DynamicHorizonConfig {
    fn default() -> Self {
        Self {
            min_horizon_secs: 0.5,
            max_horizon_secs: 5.0,
            stopping_distance_multiplier: 2.0,
            max_deceleration: 3.0,
            reaction_time_secs: 0.2,
        }
    }
}

impl DynamicHorizonConfig {
    /// 현재 속력 기반 동적 horizon 계산
    ///
    /// horizon = (정지 거리 × 배율 + 반응 거리) / 속력
    pub fn compute_horizon(&self, current_speed: f32) -> f32 {
        if current_speed < 0.01 {
            return self.min_horizon_secs;
        }

        // 정지 거리: d = v² / (2a)
        let stopping_distance = (current_speed * current_speed) / (2.0 * self.max_deceleration);
        // 반응 거리: d = v × t
        let reaction_distance = current_speed * self.reaction_time_secs;
        // 총 필요 거리
        let required_distance =
            (stopping_distance + reaction_distance) * self.stopping_distance_multiplier;
        // horizon = 거리 / 속력
        let horizon = required_distance / current_speed;

        horizon.clamp(self.min_horizon_secs, self.max_horizon_secs)
    }
}

/// 충돌 예측기
///
/// 간소화된 충돌 예측 (AABB 기반)
#[derive(Debug, Clone)]
pub struct CollisionPredictor {
    /// 안전 거리 (m)
    safety_distance: f32,

    /// 예측 시간 범위 (초) - 고정 값 (동적 미사용 시)
    horizon_secs: f32,

    /// 동적 horizon 설정 (Some이면 동적 계산 사용)
    dynamic_horizon: Option<DynamicHorizonConfig>,
}

impl CollisionPredictor {
    /// 새 CollisionPredictor 생성
    pub fn new(safety_distance: f32, horizon_secs: f32) -> Self {
        Self {
            safety_distance,
            horizon_secs,
            dynamic_horizon: None,
        }
    }

    /// 동적 horizon 사용 설정
    pub fn with_dynamic_horizon(mut self, config: DynamicHorizonConfig) -> Self {
        self.dynamic_horizon = Some(config);
        self
    }

    /// 현재 속력에 맞는 horizon 계산
    pub fn effective_horizon(&self, current_speed: f32) -> f32 {
        if let Some(ref config) = self.dynamic_horizon {
            config.compute_horizon(current_speed)
        } else {
            self.horizon_secs
        }
    }

    /// 충돌 예측 (PPR: AI_process_CollisionPrediction)
    ///
    /// # Arguments
    /// * `position` - 현재 위치
    /// * `velocity` - 현재 속도
    /// * `obstacles` - 장애물 목록
    ///
    /// # Returns
    /// * `CollisionResult` - 충돌 여부 및 시간
    pub fn predict(
        &self,
        position: &Position,
        velocity: &Velocity,
        obstacles: &[Position],
    ) -> CollisionResult {
        if obstacles.is_empty() {
            return CollisionResult {
                will_collide: false,
                time_to_collision: None,
                nearest_obstacle_distance: f32::MAX,
            };
        }

        let mut nearest_distance = f32::MAX;
        let mut will_collide = false;
        let mut ttc: Option<f32> = None;

        for obstacle in obstacles {
            // 현재 거리 계산
            let current_distance = position.distance(obstacle);

            if current_distance < nearest_distance {
                nearest_distance = current_distance;
            }

            // 안전 거리 내에 있으면 충돌
            if current_distance < self.safety_distance {
                will_collide = true;
                ttc = Some(0.0);
                break;
            }

            // 속도 기반 충돌 시간 예측 (간소화)
            let speed = velocity.magnitude();
            if speed > 0.001 {
                // 장애물 방향으로 이동 중인지 확인
                let to_obstacle = Position::new(
                    obstacle.x - position.x,
                    obstacle.y - position.y,
                    obstacle.z - position.z,
                );

                // 방향 내적 (이동 방향과 장애물 방향)
                let dot = velocity.vx * to_obstacle.x
                    + velocity.vy * to_obstacle.y
                    + velocity.vz * to_obstacle.z;

                if dot > 0.0 {
                    // 장애물 방향으로 이동 중
                    let time_to_reach = (current_distance - self.safety_distance) / speed;
                    // 동적 horizon 사용
                    let effective_horizon = self.effective_horizon(speed);
                    if time_to_reach < effective_horizon {
                        will_collide = true;
                        ttc = Some(time_to_reach.max(0.0));
                    }
                }
            }
        }

        CollisionResult {
            will_collide,
            time_to_collision: ttc,
            nearest_obstacle_distance: nearest_distance,
        }
    }

    /// 즉각적 충돌 검사 (속도 무시, 거리만)
    pub fn check_immediate(&self, position: &Position, obstacles: &[Position]) -> bool {
        for obstacle in obstacles {
            if position.distance(obstacle) < self.safety_distance {
                return true;
            }
        }
        false
    }

    /// 안전 경로 검사 (경로상의 모든 점 검사)
    pub fn check_path(
        &self,
        start: &Position,
        end: &Position,
        obstacles: &[Position],
        num_samples: usize,
    ) -> bool {
        for i in 0..=num_samples {
            let t = i as f32 / num_samples as f32;
            let point = Position::new(
                start.x + t * (end.x - start.x),
                start.y + t * (end.y - start.y),
                start.z + t * (end.z - start.z),
            );

            if self.check_immediate(&point, obstacles) {
                return false; // 안전하지 않음
            }
        }
        true // 모든 점이 안전
    }

    /// 안전 거리 조회
    pub fn safety_distance(&self) -> f32 {
        self.safety_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_obstacles() {
        let predictor = CollisionPredictor::new(1.0, 1.0);
        let pos = Position::ORIGIN;
        let vel = Velocity::new(1.0, 0.0, 0.0);

        let result = predictor.predict(&pos, &vel, &[]);

        assert!(!result.will_collide);
        assert!(result.nearest_obstacle_distance == f32::MAX);
    }

    #[test]
    fn test_immediate_collision() {
        let predictor = CollisionPredictor::new(1.0, 1.0);
        let pos = Position::ORIGIN;
        let vel = Velocity::new(1.0, 0.0, 0.0);
        let obstacles = vec![Position::new(0.5, 0.0, 0.0)]; // 0.5m away

        let result = predictor.predict(&pos, &vel, &obstacles);

        assert!(result.will_collide);
        assert_eq!(result.time_to_collision, Some(0.0));
    }

    #[test]
    fn test_future_collision() {
        let predictor = CollisionPredictor::new(1.0, 2.0);
        let pos = Position::ORIGIN;
        let vel = Velocity::new(2.0, 0.0, 0.0); // 2 m/s toward obstacle
        let obstacles = vec![Position::new(3.0, 0.0, 0.0)]; // 3m away

        let result = predictor.predict(&pos, &vel, &obstacles);

        // 3m - 1m safety = 2m, at 2 m/s = 1 sec
        assert!(result.will_collide);
        assert!(result.time_to_collision.unwrap() < 2.0);
    }

    #[test]
    fn test_moving_away() {
        let predictor = CollisionPredictor::new(1.0, 2.0);
        let pos = Position::ORIGIN;
        let vel = Velocity::new(-2.0, 0.0, 0.0); // Moving AWAY from obstacle
        let obstacles = vec![Position::new(3.0, 0.0, 0.0)];

        let result = predictor.predict(&pos, &vel, &obstacles);

        assert!(!result.will_collide); // Moving away
    }

    #[test]
    fn test_check_path_safe() {
        let predictor = CollisionPredictor::new(1.0, 1.0);
        let start = Position::ORIGIN;
        let end = Position::new(5.0, 0.0, 0.0);
        let obstacles = vec![Position::new(0.0, 5.0, 0.0)]; // Far away

        assert!(predictor.check_path(&start, &end, &obstacles, 10));
    }

    #[test]
    fn test_check_path_blocked() {
        let predictor = CollisionPredictor::new(1.0, 1.0);
        let start = Position::ORIGIN;
        let end = Position::new(5.0, 0.0, 0.0);
        let obstacles = vec![Position::new(2.5, 0.0, 0.0)]; // On the path

        assert!(!predictor.check_path(&start, &end, &obstacles, 10));
    }

    #[test]
    fn test_dynamic_horizon_low_speed() {
        let config = DynamicHorizonConfig::default();
        // 저속: 0.5 m/s
        let horizon = config.compute_horizon(0.5);
        // 저속이므로 최소 horizon(0.5초)에 가까워야 함
        assert!(horizon >= config.min_horizon_secs);
        assert!(horizon <= 1.0);
    }

    #[test]
    fn test_dynamic_horizon_medium_speed() {
        let config = DynamicHorizonConfig::default();
        // 중속: 3.0 m/s
        let horizon = config.compute_horizon(3.0);
        // 정지 거리 = 9/(2*3) = 1.5m
        // 반응 거리 = 3*0.2 = 0.6m
        // 총 거리 = (1.5+0.6)*2 = 4.2m
        // horizon = 4.2/3 = 1.4초
        assert!(horizon >= 1.0);
        assert!(horizon <= 2.0);
    }

    #[test]
    fn test_dynamic_horizon_high_speed() {
        let config = DynamicHorizonConfig::default();
        // 고속: 10.0 m/s
        let horizon = config.compute_horizon(10.0);
        // 고속이므로 최상 horizon(5.0초)에 가까워야 함
        assert!(horizon >= 3.0);
        assert!(horizon <= config.max_horizon_secs);
    }

    #[test]
    fn test_dynamic_horizon_bounds() {
        let config = DynamicHorizonConfig::default();

        // 거의 정지 상태 → 최소값
        assert_eq!(config.compute_horizon(0.001), config.min_horizon_secs);

        // 매우 고속 → 최대값 제한
        let very_high = config.compute_horizon(100.0);
        assert_eq!(very_high, config.max_horizon_secs);
    }

    #[test]
    fn test_predictor_with_dynamic_horizon() {
        let config = DynamicHorizonConfig::default();
        let predictor = CollisionPredictor::new(1.0, 1.0).with_dynamic_horizon(config);

        // 저속 (1 m/s) - 짧은 horizon
        let short_horizon = predictor.effective_horizon(1.0);

        // 고속 (5 m/s) - 긴 horizon
        let long_horizon = predictor.effective_horizon(5.0);

        assert!(long_horizon > short_horizon);
    }
}
