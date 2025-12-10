//! failsafe 모듈 - 장애 대응 관리

mod manager;

pub use manager::{EdgeStatus, FailsafeAction, FailsafeConfig, FailsafeManager};
