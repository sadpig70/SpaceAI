//! 로봇 운동학 모듈
//!
//! 다양한 로봇 유형별 운동학적 특성을 정의합니다.
//!
//! PPR 매핑: AI_make_VehicleProfile

mod profile;

pub use profile::{KinematicsParams, VehicleProfile, VehicleType};
