//! Bid 타입 (경매용)
//!
//! PPR 매핑: AI_make_VickreyAuction

use serde::{Deserialize, Serialize};

/// 입찰 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    /// 입찰자 로봇 ID
    pub robot_id: u64,

    /// 입찰 금액 (밀리 단위)
    pub amount_milli: u64,

    /// 입찰 대상 경로 ID
    pub path_id: u64,

    /// 커밋 해시 (Commit-Reveal용)
    pub commit_hash: [u8; 32],

    /// nonce (Reveal 시 사용)
    pub nonce: [u8; 16],

    /// 입찰 타임스탬프 (나노초)
    pub timestamp_ns: u64,

    /// 입찰 상태
    pub status: BidStatus,
}

/// 입찰 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BidStatus {
    /// 커밋됨 (해시만 제출)
    Committed,

    /// 리빌됨 (실제 금액 공개)
    Revealed,

    /// 승리
    Won,

    /// 패배
    Lost,

    /// 취소됨
    Cancelled,

    /// 무효 (해시 불일치 등)
    Invalid,
}

impl Bid {
    /// 새 Bid 생성 (커밋 전)
    pub fn new(robot_id: u64, amount_milli: u64, path_id: u64) -> Self {
        Self {
            robot_id,
            amount_milli,
            path_id,
            commit_hash: [0u8; 32],
            nonce: [0u8; 16],
            timestamp_ns: 0,
            status: BidStatus::Committed,
        }
    }

    /// 커밋 해시 생성
    ///
    /// commit_hash = H(amount || nonce || path_id)
    pub fn compute_commit_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.amount_milli.hash(&mut hasher);
        self.nonce.hash(&mut hasher);
        self.path_id.hash(&mut hasher);

        let hash = hasher.finish();
        self.commit_hash[0..8].copy_from_slice(&hash.to_le_bytes());
    }

    /// 커밋 해시 검증 (Reveal 시)
    pub fn verify_commit(&self) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.amount_milli.hash(&mut hasher);
        self.nonce.hash(&mut hasher);
        self.path_id.hash(&mut hasher);

        let hash = hasher.finish();
        let mut expected = [0u8; 32];
        expected[0..8].copy_from_slice(&hash.to_le_bytes());

        self.commit_hash == expected
    }

    /// nonce 설정
    pub fn with_nonce(mut self, nonce: [u8; 16]) -> Self {
        self.nonce = nonce;
        self
    }

    /// 타임스탬프 설정
    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }

    /// Reveal 처리
    pub fn reveal(&mut self) -> bool {
        if self.status == BidStatus::Committed && self.verify_commit() {
            self.status = BidStatus::Revealed;
            true
        } else {
            self.status = BidStatus::Invalid;
            false
        }
    }

    /// 승리 처리
    pub fn mark_won(&mut self) {
        self.status = BidStatus::Won;
    }

    /// 패배 처리
    pub fn mark_lost(&mut self) {
        self.status = BidStatus::Lost;
    }

    /// 금액 반환 (f64, 표시용)
    pub fn amount(&self) -> f64 {
        self.amount_milli as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bid_new() {
        let bid = Bid::new(42, 150_000, 1);
        assert_eq!(bid.robot_id, 42);
        assert_eq!(bid.amount_milli, 150_000);
        assert_eq!(bid.status, BidStatus::Committed);
    }

    #[test]
    fn test_bid_commit_and_verify() {
        let nonce = [1u8; 16];
        let mut bid = Bid::new(1, 200_000, 1).with_nonce(nonce);

        bid.compute_commit_hash();
        assert!(bid.verify_commit());

        // 금액 변조 시 검증 실패
        bid.amount_milli = 300_000;
        assert!(!bid.verify_commit());
    }

    #[test]
    fn test_bid_reveal() {
        let nonce = [2u8; 16];
        let mut bid = Bid::new(1, 100_000, 1).with_nonce(nonce);
        bid.compute_commit_hash();

        assert!(bid.reveal());
        assert_eq!(bid.status, BidStatus::Revealed);
    }

    #[test]
    fn test_bid_reveal_invalid() {
        let mut bid = Bid::new(1, 100_000, 1);
        // 커밋 해시 없이 reveal 시도

        assert!(!bid.reveal());
        assert_eq!(bid.status, BidStatus::Invalid);
    }

    #[test]
    fn test_bid_amount() {
        let bid = Bid::new(1, 150_500, 1);
        assert!((bid.amount() - 150.5).abs() < 1e-6);
    }
}
