//! # SAP Network
//!
//! SAP v1.2 L3 PredictiveSync - 예측/롤백 네트워크 엔진
//!
//! ## 모듈 구조
//!
//! - `sync`: 상태 동기화
//! - `rollback`: 롤백 관리자
//! - `failsafe`: 장애 대응 관리자
//!
//! ## PPR 매핑
//!
//! - `AI_make_RollbackManager` → `RollbackManager`
//! - `AI_make_FailsafeManager` → `FailsafeManager`
//! - `AI_process_StateComparison` → `StateComparator`

pub mod failsafe;
pub mod rollback;
pub mod sync;

// 주요 타입 re-export
pub use failsafe::{FailsafeAction, FailsafeManager};
pub use rollback::{RollbackEvent, RollbackManager};
pub use sync::{StateComparator, SyncResult};
