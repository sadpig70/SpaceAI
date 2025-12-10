//! Ed25519 서명 타입 정의
//!
//! 실제 암호화 구현은 외부 크레이트(ed25519-dalek 등)에 위임합니다.
//! 이 모듈은 타입 정의와 인터페이스만 제공합니다.

use serde::{Deserialize, Serialize};

/// Ed25519 공개키 (32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(pub [u8; 32]);

impl PublicKey {
    /// 빈 공개키 (테스트용)
    pub const ZERO: Self = Self([0u8; 32]);

    /// 바이트 배열에서 생성
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// 바이트 슬라이스에서 생성
    pub fn try_from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Some(Self(bytes))
    }

    /// 바이트 배열 반환
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// robot_id 파생 (상위 64비트)
    /// Sybil 방어: robot_id는 공개키에서 결정론적으로 파생
    pub fn derive_robot_id(&self) -> u64 {
        // FNV-1a 해시의 상위 64비트 사용
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        for byte in &self.0 {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3); // FNV prime
        }
        hash
    }
}

impl Default for PublicKey {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Ed25519 비밀키 (64 bytes: 32 seed + 32 public)
#[derive(Clone)]
pub struct SecretKey(pub [u8; 64]);

impl SecretKey {
    /// 바이트 배열에서 생성
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// 바이트 배열 반환
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 비밀키는 출력하지 않음
        write!(f, "SecretKey([REDACTED])")
    }
}

/// Ed25519 서명 (64 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(pub [u8; 64]);

impl Signature {
    /// 빈 서명 (미서명 상태)
    pub const ZERO: Self = Self([0u8; 64]);

    /// 바이트 배열에서 생성
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// 바이트 슬라이스에서 생성
    pub fn try_from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 64 {
            return None;
        }
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(slice);
        Some(Self(bytes))
    }

    /// 바이트 배열 반환
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// 서명 여부 확인
    pub fn is_signed(&self) -> bool {
        self.0.iter().any(|&b| b != 0)
    }

    /// Vec<u8>로 변환 (기존 코드 호환)
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Vec<u8>> for Signature {
    fn from(vec: Vec<u8>) -> Self {
        Self::try_from_slice(&vec).unwrap_or(Self::ZERO)
    }
}

// Signature 수동 Serialize 구현
impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Vec<u8>로 직렬화
        serializer.serialize_bytes(&self.0)
    }
}

// Signature 수동 Deserialize 구현
impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        Self::try_from_slice(&bytes)
            .ok_or_else(|| serde::de::Error::custom("Invalid signature length, expected 64 bytes"))
    }
}

/// 서명 에러
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureError {
    /// 잘못된 서명 길이
    InvalidLength { expected: usize, actual: usize },
    /// 서명 검증 실패
    VerificationFailed,
    /// 잘못된 공개키
    InvalidPublicKey,
    /// 메시지 변조 감지
    MessageTampered,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "Invalid signature length: expected {}, got {}",
                    expected, actual
                )
            }
            Self::VerificationFailed => write!(f, "Signature verification failed"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::MessageTampered => write!(f, "Message has been tampered"),
        }
    }
}

impl std::error::Error for SignatureError {}

/// 서명된 메시지 래퍼
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage<T> {
    /// 원본 메시지
    pub message: T,
    /// 서명자 공개키
    pub signer: PublicKey,
    /// 서명
    pub signature: Signature,
    /// Nonce (리플레이 방어)
    pub nonce: u64,
    /// 시퀀스 번호 (리플레이 방어)
    pub sequence: u64,
    /// 타임스탬프 (나노초)
    pub timestamp_ns: u64,
}

impl<T> SignedMessage<T> {
    /// 새 서명 메시지 생성 (서명 전)
    pub fn new(
        message: T,
        signer: PublicKey,
        nonce: u64,
        sequence: u64,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            message,
            signer,
            signature: Signature::ZERO,
            nonce,
            sequence,
            timestamp_ns,
        }
    }

    /// 서명 여부 확인
    pub fn is_signed(&self) -> bool {
        self.signature.is_signed()
    }

    /// signer의 robot_id 반환
    pub fn signer_robot_id(&self) -> u64 {
        self.signer.derive_robot_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_derive_robot_id() {
        let pk = PublicKey::from_bytes([1u8; 32]);
        let robot_id = pk.derive_robot_id();
        assert_ne!(robot_id, 0);

        // 동일 입력 → 동일 출력
        let robot_id2 = pk.derive_robot_id();
        assert_eq!(robot_id, robot_id2);

        // 다른 입력 → 다른 출력
        let pk2 = PublicKey::from_bytes([2u8; 32]);
        let robot_id3 = pk2.derive_robot_id();
        assert_ne!(robot_id, robot_id3);
    }

    #[test]
    fn test_signature_is_signed() {
        assert!(!Signature::ZERO.is_signed());

        let mut sig_bytes = [0u8; 64];
        sig_bytes[0] = 1;
        let sig = Signature::from_bytes(sig_bytes);
        assert!(sig.is_signed());
    }

    #[test]
    fn test_secret_key_debug_redacted() {
        let sk = SecretKey::from_bytes([42u8; 64]);
        let debug = format!("{:?}", sk);
        assert!(debug.contains("REDACTED"));
        assert!(!debug.contains("42"));
    }

    #[test]
    fn test_signed_message() {
        let pk = PublicKey::from_bytes([1u8; 32]);
        let msg = SignedMessage::new("test", pk, 12345, 1, 1_000_000_000);

        assert!(!msg.is_signed());
        assert_eq!(msg.nonce, 12345);
        assert_eq!(msg.sequence, 1);
    }
}
