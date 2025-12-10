//! 벤치마크 유틸리티

use sap_core::types::{Position, Velocity};

/// 랜덤 위치 생성
pub fn random_position(rng: &mut impl rand::Rng, max: f32) -> Position {
    Position::new(rng.gen_range(-max..max), rng.gen_range(-max..max), 0.0)
}

/// 랜덤 속도 생성
pub fn random_velocity(rng: &mut impl rand::Rng, max: f32) -> Velocity {
    Velocity::new(rng.gen_range(-max..max), rng.gen_range(-max..max), 0.0)
}
