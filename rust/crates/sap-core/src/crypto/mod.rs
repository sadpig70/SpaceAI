//! 암호화 및 보안 모듈
//!
//! SAP 2.0 보안 프리미티브를 정의합니다.
//!
//! # 서명 알고리즘
//!
//! SAP는 **Ed25519**를 기본 서명 알고리즘으로 사용합니다:
//! - 공개키: 32 bytes
//! - 개인키: 64 bytes (seed + public key)
//! - 서명: 64 bytes
//!
//! # 리플레이 방어
//!
//! 각 메시지에는 다음이 포함됩니다:
//! - `nonce`: 64-bit 일회성 랜덤값
//! - `sequence`: 64-bit 단조 증가 시퀀스
//! - `timestamp_ns`: 64-bit 타임스탬프
//!
//! # Sybil 방어
//!
//! 로봇 ID는 공개키 해시로 파생됩니다:
//! `robot_id = hash(public_key)[0..8]` (상위 64비트)
//!
//! PPR 매핑: AI_make_CryptoSpec

mod replay;
mod signature;

pub use replay::{NonceGenerator, ReplayGuard, SequenceTracker};
pub use signature::{PublicKey, SecretKey, Signature, SignatureError, SignedMessage};
