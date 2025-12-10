//! 검증 타입 정의 모듈

mod frame;
mod proof;
mod result;

pub use frame::{constraint_ids, ValidationFrame};
pub use proof::ProofDigest;
pub use result::{AdjustedCommand, ValidationResult, ValidationResultDetail};
