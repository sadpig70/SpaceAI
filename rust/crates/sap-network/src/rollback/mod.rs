//! rollback 모듈 - 롤백 관리

mod manager;

pub use manager::{
    RollbackConfig, RollbackEvent, RollbackManager, RollbackReason, SnapshotStrategy,
};
