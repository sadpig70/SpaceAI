//! 검증 프레임 타입
//!
//! PPR 매핑: AI_make_ValidationFrame

use serde::{Deserialize, Serialize};

/// 단일 검증 프레임 (zk-Physics 준비용)
///
/// 각 틱마다 생성되며, 나중에 SNARK 회로 입력으로 사용 가능
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct ValidationFrame {
    /// 틱 번호
    pub tick: u64,

    /// 로봇 ID
    pub robot_id: u64,

    /// 명령 해시 (32바이트)
    pub cmd_hash: [u8; 32],

    /// 통과한 제약조건 비트맵 (각 비트 = 하나의 제약조건)
    ///
    /// bit 0: 속도 제한
    /// bit 1: 가속도 제한
    /// bit 2: 저크 제한
    /// bit 3: 지오펜스
    /// bit 4: 충돌 예측
    /// bit 5: 티켓 유효성
    /// bit 6-63: 예약
    pub constraints_passed_bitmap: u64,

    /// 검증 시각 (PTP 나노초)
    pub timestamp_ns: u64,

    /// Zone ID
    pub zone_id: u32,
}

impl ValidationFrame {
    /// 새 ValidationFrame 생성
    pub fn new(tick: u64, robot_id: u64, zone_id: u32) -> Self {
        Self {
            tick,
            robot_id,
            cmd_hash: [0u8; 32],
            constraints_passed_bitmap: 0,
            timestamp_ns: 0,
            zone_id,
        }
    }

    /// 명령 해시 설정
    pub fn with_cmd_hash(mut self, hash: [u8; 32]) -> Self {
        self.cmd_hash = hash;
        self
    }

    /// 타임스탬프 설정
    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }

    /// 제약조건 통과 설정
    pub fn set_constraint(&mut self, constraint_id: u8, passed: bool) {
        if constraint_id < 64 {
            if passed {
                self.constraints_passed_bitmap |= 1 << constraint_id;
            } else {
                self.constraints_passed_bitmap &= !(1 << constraint_id);
            }
        }
    }

    /// 제약조건 통과 여부 확인
    pub fn check_constraint(&self, constraint_id: u8) -> bool {
        if constraint_id < 64 {
            (self.constraints_passed_bitmap & (1 << constraint_id)) != 0
        } else {
            false
        }
    }

    /// 모든 제약조건 통과 여부
    pub fn all_passed(&self, required_mask: u64) -> bool {
        (self.constraints_passed_bitmap & required_mask) == required_mask
    }

    /// 통과한 제약조건 수
    pub fn passed_count(&self) -> u32 {
        self.constraints_passed_bitmap.count_ones()
    }
}

/// 제약조건 ID 상수
pub mod constraint_ids {
    pub const VELOCITY_LIMIT: u8 = 0;
    pub const ACCELERATION_LIMIT: u8 = 1;
    pub const JERK_LIMIT: u8 = 2;
    pub const GEOFENCE: u8 = 3;
    pub const COLLISION_PREDICTION: u8 = 4;
    pub const TICKET_VALIDITY: u8 = 5;
    pub const VTS_COMPLIANCE: u8 = 6;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_frame_new() {
        let frame = ValidationFrame::new(100, 42, 1);
        assert_eq!(frame.tick, 100);
        assert_eq!(frame.robot_id, 42);
        assert_eq!(frame.constraints_passed_bitmap, 0);
    }

    #[test]
    fn test_validation_frame_set_constraint() {
        let mut frame = ValidationFrame::new(1, 1, 1);

        frame.set_constraint(constraint_ids::VELOCITY_LIMIT, true);
        assert!(frame.check_constraint(constraint_ids::VELOCITY_LIMIT));

        frame.set_constraint(constraint_ids::GEOFENCE, true);
        assert!(frame.check_constraint(constraint_ids::GEOFENCE));

        assert!(!frame.check_constraint(constraint_ids::COLLISION_PREDICTION));
    }

    #[test]
    fn test_validation_frame_all_passed() {
        let mut frame = ValidationFrame::new(1, 1, 1);

        // 속도, 가속도, 저크만 필수
        let required = (1 << constraint_ids::VELOCITY_LIMIT)
            | (1 << constraint_ids::ACCELERATION_LIMIT)
            | (1 << constraint_ids::JERK_LIMIT);

        assert!(!frame.all_passed(required));

        frame.set_constraint(constraint_ids::VELOCITY_LIMIT, true);
        frame.set_constraint(constraint_ids::ACCELERATION_LIMIT, true);
        frame.set_constraint(constraint_ids::JERK_LIMIT, true);

        assert!(frame.all_passed(required));
    }

    #[test]
    fn test_validation_frame_passed_count() {
        let mut frame = ValidationFrame::new(1, 1, 1);
        assert_eq!(frame.passed_count(), 0);

        frame.set_constraint(0, true);
        frame.set_constraint(1, true);
        frame.set_constraint(2, true);

        assert_eq!(frame.passed_count(), 3);
    }
}
