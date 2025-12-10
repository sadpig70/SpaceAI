//! # SAP Core
//!
//! SAP v1.2 공통 타입, 프로토콜, 유틸리티 크레이트.
//!
//! ## 모듈 구조
//!
//! - `types`: 기본 타입 (Position, Velocity, Acceleration, RobotState)
//! - `validation`: 검증 타입 (ValidationResult, ValidationFrame)
//! - `packet`: 네트워크 패킷 (DeltaTickPacket, RollbackFrame)
//! - `ticket`: 티켓/경제 타입 (VoxelTimeSlot, TransitTicket)
//! - `crypto`: 암호화/보안 (Signature, ReplayGuard)
//! - `error`: 에러 타입
//! - `util`: 유틸리티 (해시, 시간, 고정 소수점)

pub mod crypto;
pub mod error;
pub mod packet;
pub mod ticket;
pub mod types;
pub mod util;
pub mod validation;

// 자주 사용되는 타입 re-export
pub use crypto::{PublicKey, ReplayGuard, Signature, SignedMessage};
pub use error::{Result, SapError};
pub use packet::{DeltaTickPacket, PacketHeader, RollbackFrame};
pub use ticket::{Bid, TransitTicket, VoxelTimeSlot, VtsId};
pub use types::{Acceleration, Position, RobotState, Velocity, WorldState};
pub use validation::{ProofDigest, ValidationFrame, ValidationResult};
