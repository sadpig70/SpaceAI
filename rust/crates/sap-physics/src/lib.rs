//! # SAP Physics
//!
//! SAP v2.0 L2 TrustOS - 물리 검증 커널
//!
//! ## 모듈 구조
//!
//! - `validator`: 물리 검증기 (PhysicsValidator)
//! - `constraint`: 제약조건 엔진
//! - `command`: 명령 게이트
//! - `recovery`: 물리적 복구 명령
//! - `kinematics`: 로봇 운동학 프로파일
//!
//! ## PPR 매핑
//!
//! - `AI_make_PhysicsValidator` → `PhysicsValidator::validate()`
//! - `AI_process_KinematicsCheck` → `KinematicsChecker::check()`
//! - `AI_process_CollisionPrediction` → `CollisionPredictor::predict()`
//! - `AI_response_PhysicalRecovery` → `RecoveryCommand`
//! - `AI_make_VehicleProfile` → `VehicleProfile`

pub mod command;
pub mod constraint;
pub mod kinematics;
pub mod recovery;
pub mod validator;

// 주요 타입 re-export
pub use command::{CommandGate, MotionCommand};
pub use constraint::{CollisionPredictor, KinematicsChecker};
pub use kinematics::{KinematicsParams, VehicleProfile, VehicleType};
pub use recovery::{RecoveryCommand, RecoveryLevel, RecoveryResult};
pub use validator::{PhysicsValidator, PhysicsValidatorConfig};
