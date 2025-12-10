//! 리플레이 공격 방어 모듈
//!
//! Nonce와 시퀀스 기반 리플레이 공격 방어를 구현합니다.

use std::collections::HashMap;

/// Nonce 생성기
pub struct NonceGenerator {
    /// 마지막 생성 시각 (나노초)
    last_time_ns: u64,
    /// 카운터 (같은 시각 내 구분용)
    counter: u32,
}

impl NonceGenerator {
    /// 새 Nonce 생성기
    pub fn new() -> Self {
        Self {
            last_time_ns: 0,
            counter: 0,
        }
    }

    /// 새 Nonce 생성
    /// 상위 32비트: 타임스탬프 일부, 하위 32비트: 카운터
    pub fn generate(&mut self, current_time_ns: u64) -> u64 {
        if current_time_ns == self.last_time_ns {
            self.counter += 1;
        } else {
            self.last_time_ns = current_time_ns;
            self.counter = 0;
        }

        let time_part = (current_time_ns / 1_000_000) as u32; // 밀리초 단위
        ((time_part as u64) << 32) | (self.counter as u64)
    }
}

impl Default for NonceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 시퀀스 추적기 (발신자별)
pub struct SequenceTracker {
    /// 발신자별 마지막 시퀀스
    sequences: HashMap<u64, u64>,
    /// 최대 허용 점프 (DoS 방어)
    max_jump: u64,
}

impl SequenceTracker {
    /// 새 시퀀스 추적기
    pub fn new(max_jump: u64) -> Self {
        Self {
            sequences: HashMap::new(),
            max_jump,
        }
    }

    /// 시퀀스 유효성 검사 및 업데이트
    /// 반환: true = 유효, false = 리플레이 또는 비정상
    pub fn check_and_update(&mut self, sender_id: u64, sequence: u64) -> bool {
        let last_seq = self.sequences.get(&sender_id).copied().unwrap_or(0);

        // 시퀀스는 단조 증가해야 함
        if sequence <= last_seq {
            return false; // 리플레이 감지
        }

        // 너무 큰 점프는 거부 (DoS 방어)
        if sequence > last_seq + self.max_jump {
            return false;
        }

        self.sequences.insert(sender_id, sequence);
        true
    }

    /// 발신자의 현재 시퀀스 조회
    pub fn current_sequence(&self, sender_id: u64) -> u64 {
        self.sequences.get(&sender_id).copied().unwrap_or(0)
    }

    /// 발신자 제거 (연결 해제 시)
    pub fn remove_sender(&mut self, sender_id: u64) {
        self.sequences.remove(&sender_id);
    }

    /// 등록된 발신자 수
    pub fn sender_count(&self) -> usize {
        self.sequences.len()
    }
}

impl Default for SequenceTracker {
    fn default() -> Self {
        Self::new(1000) // 기본 최대 점프: 1000
    }
}

/// 리플레이 방어 가드 (통합)
pub struct ReplayGuard {
    /// 시퀀스 추적기
    sequence_tracker: SequenceTracker,
    /// Nonce 캐시 (최근 N개 저장)
    nonce_cache: Vec<u64>,
    /// 캐시 최대 크기
    max_cache_size: usize,
    /// 메시지 유효 시간 (나노초)
    validity_window_ns: u64,
}

impl ReplayGuard {
    /// 새 리플레이 가드
    pub fn new(max_cache_size: usize, validity_window_ns: u64) -> Self {
        Self {
            sequence_tracker: SequenceTracker::default(),
            nonce_cache: Vec::with_capacity(max_cache_size),
            max_cache_size,
            validity_window_ns,
        }
    }

    /// 메시지 유효성 검사
    pub fn validate(
        &mut self,
        sender_id: u64,
        nonce: u64,
        sequence: u64,
        timestamp_ns: u64,
        current_time_ns: u64,
    ) -> Result<(), ReplayError> {
        // 1. 타임스탬프 검사 (너무 오래된/미래 메시지 거부)
        if timestamp_ns + self.validity_window_ns < current_time_ns {
            return Err(ReplayError::Expired);
        }
        if timestamp_ns > current_time_ns + self.validity_window_ns {
            return Err(ReplayError::FutureTimestamp);
        }

        // 2. 시퀀스 검사
        if !self.sequence_tracker.check_and_update(sender_id, sequence) {
            return Err(ReplayError::InvalidSequence);
        }

        // 3. Nonce 중복 검사
        if self.nonce_cache.contains(&nonce) {
            return Err(ReplayError::DuplicateNonce);
        }

        // Nonce 캐시 업데이트
        if self.nonce_cache.len() >= self.max_cache_size {
            self.nonce_cache.remove(0);
        }
        self.nonce_cache.push(nonce);

        Ok(())
    }
}

impl Default for ReplayGuard {
    fn default() -> Self {
        Self::new(10000, 5_000_000_000) // 10000개 캐시, 5초 유효
    }
}

/// 리플레이 에러
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    /// 만료된 메시지
    Expired,
    /// 미래 타임스탬프
    FutureTimestamp,
    /// 잘못된 시퀀스 (리플레이 또는 점프)
    InvalidSequence,
    /// 중복 Nonce
    DuplicateNonce,
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expired => write!(f, "Message has expired"),
            Self::FutureTimestamp => write!(f, "Message timestamp is in the future"),
            Self::InvalidSequence => write!(f, "Invalid sequence number"),
            Self::DuplicateNonce => write!(f, "Duplicate nonce detected"),
        }
    }
}

impl std::error::Error for ReplayError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_generator() {
        let mut gen = NonceGenerator::new();
        let n1 = gen.generate(1_000_000_000);
        let n2 = gen.generate(1_000_000_000);
        let n3 = gen.generate(2_000_000_000);

        assert_ne!(n1, n2); // 같은 시각이라도 다른 nonce
        assert_ne!(n2, n3);
    }

    #[test]
    fn test_sequence_tracker() {
        let mut tracker = SequenceTracker::new(100);

        assert!(tracker.check_and_update(1, 1)); // 첫 시퀀스
        assert!(tracker.check_and_update(1, 2)); // 증가
        assert!(!tracker.check_and_update(1, 2)); // 리플레이
        assert!(!tracker.check_and_update(1, 1)); // 과거
        assert!(tracker.check_and_update(1, 3)); // 정상 증가
        assert!(!tracker.check_and_update(1, 200)); // 너무 큰 점프
    }

    #[test]
    fn test_replay_guard_valid() {
        let mut guard = ReplayGuard::new(100, 5_000_000_000);
        let current = 10_000_000_000u64;

        let result = guard.validate(1, 12345, 1, current - 1_000_000_000, current);
        assert!(result.is_ok());
    }

    #[test]
    fn test_replay_guard_expired() {
        let mut guard = ReplayGuard::new(100, 5_000_000_000);
        let current = 10_000_000_000u64;

        let result = guard.validate(1, 12345, 1, current - 10_000_000_000, current);
        assert_eq!(result, Err(ReplayError::Expired));
    }

    #[test]
    fn test_replay_guard_duplicate_nonce() {
        let mut guard = ReplayGuard::new(100, 5_000_000_000);
        let current = 10_000_000_000u64;

        guard.validate(1, 12345, 1, current, current).unwrap();
        let result = guard.validate(1, 12345, 2, current, current);
        assert_eq!(result, Err(ReplayError::DuplicateNonce));
    }
}
