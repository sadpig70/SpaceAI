//! 증명 다이제스트 타입
//!
//! PPR 매핑: AI_make_ProofDigest

use serde::{Deserialize, Serialize};

/// 검증 증명 다이제스트 (Merkle Root + 서명)
///
/// 여러 ValidationFrame을 집계하여 하나의 증명으로 만듦
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDigest {
    /// 시작 틱
    pub from_tick: u64,

    /// 종료 틱
    pub to_tick: u64,

    /// Merkle Root (32바이트)
    pub merkle_root: [u8; 32],

    /// TrustOS 서명 (64바이트, Ed25519) - Vec으로 변경하여 serde 호환
    pub tos_sig: Vec<u8>,

    /// Zone ID
    pub zone_id: u32,

    /// 포함된 프레임 수
    pub frame_count: u32,

    /// 생성 시각 (PTP 나노초)
    pub created_at_ns: u64,
}

impl ProofDigest {
    /// 새 ProofDigest 생성 (서명 전)
    pub fn new(from_tick: u64, to_tick: u64, zone_id: u32) -> Self {
        Self {
            from_tick,
            to_tick,
            merkle_root: [0u8; 32],
            tos_sig: vec![0u8; 64],
            zone_id,
            frame_count: 0,
            created_at_ns: 0,
        }
    }

    /// Merkle Root 설정
    pub fn with_merkle_root(mut self, root: [u8; 32]) -> Self {
        self.merkle_root = root;
        self
    }

    /// 서명 설정 (64바이트)
    pub fn with_signature(mut self, sig: &[u8]) -> Self {
        self.tos_sig = sig.to_vec();
        self
    }

    /// 프레임 수 설정
    pub fn with_frame_count(mut self, count: u32) -> Self {
        self.frame_count = count;
        self
    }

    /// 틱 범위 확인
    pub fn tick_range(&self) -> u64 {
        self.to_tick.saturating_sub(self.from_tick)
    }

    /// 서명 여부 확인
    pub fn is_signed(&self) -> bool {
        self.tos_sig.iter().any(|&b| b != 0)
    }

    /// 빈 Merkle Root인지 확인
    pub fn has_merkle_root(&self) -> bool {
        self.merkle_root.iter().any(|&b| b != 0)
    }
}

impl Default for ProofDigest {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_digest_new() {
        let digest = ProofDigest::new(100, 200, 1);
        assert_eq!(digest.from_tick, 100);
        assert_eq!(digest.to_tick, 200);
        assert_eq!(digest.tick_range(), 100);
    }

    #[test]
    fn test_proof_digest_is_signed() {
        let digest = ProofDigest::new(0, 0, 0);
        assert!(!digest.is_signed());

        let sig = [1u8; 64];
        let signed = digest.with_signature(&sig);
        assert!(signed.is_signed());
    }

    #[test]
    fn test_proof_digest_builder_pattern() {
        let digest = ProofDigest::new(0, 100, 1)
            .with_merkle_root([1u8; 32])
            .with_frame_count(50);

        assert!(digest.has_merkle_root());
        assert_eq!(digest.frame_count, 50);
    }
}
