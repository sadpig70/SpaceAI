//! Transit Ticket 타입
//!
//! PPR 매핑: AI_response_TransitTicket

use super::VoxelTimeSlot;
use serde::{Deserialize, Serialize};

/// Transit Ticket - 통행 티켓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitTicket {
    /// 티켓 고유 ID
    pub ticket_id: u128,

    /// 로봇 ID
    pub robot_id: u64,

    /// Zone ID
    pub zone_id: u32,

    /// 예약된 VoxelTimeSlot 목록
    pub vts_list: Vec<VoxelTimeSlot>,

    /// 유효 시작 시각 (나노초)
    pub valid_from_ns: u64,

    /// 유효 종료 시각 (나노초)
    pub valid_to_ns: u64,

    /// 최대 속도 프로파일 ID
    pub max_speed_profile: u32,

    /// 우선순위 클래스 (0 = 일반, 높을수록 우선)
    pub priority_class: u8,

    /// 총 가격 (밀리 단위)
    pub total_price_milli: u64,

    /// S-MEV 서명 (64바이트)
    /// S-MEV 서명 (64바이트)
    pub smev_sig: Vec<u8>,

    /// 발행 시각
    pub issued_at_ns: u64,
}

impl TransitTicket {
    /// 새 TransitTicket 생성
    pub fn new(ticket_id: u128, robot_id: u64, zone_id: u32) -> Self {
        Self {
            ticket_id,
            robot_id,
            zone_id,
            vts_list: Vec::new(),
            valid_from_ns: 0,
            valid_to_ns: 0,
            max_speed_profile: 0,
            priority_class: 0,
            total_price_milli: 0,
            smev_sig: vec![0u8; 64],
            issued_at_ns: 0,
        }
    }

    /// VTS 목록 설정
    pub fn with_vts(mut self, vts_list: Vec<VoxelTimeSlot>) -> Self {
        if let (Some(first), Some(last)) = (vts_list.first(), vts_list.last()) {
            self.valid_from_ns = first.t_start_ns;
            self.valid_to_ns = last.t_end_ns;
        }
        self.vts_list = vts_list;
        self
    }

    /// 유효 기간 설정
    pub fn with_validity(mut self, from_ns: u64, to_ns: u64) -> Self {
        self.valid_from_ns = from_ns;
        self.valid_to_ns = to_ns;
        self
    }

    /// 가격 및 우선순위 설정
    pub fn with_pricing(mut self, price_milli: u64, priority: u8) -> Self {
        self.total_price_milli = price_milli;
        self.priority_class = priority;
        self
    }

    /// 서명 설정
    pub fn with_signature(mut self, sig: &[u8]) -> Self {
        self.smev_sig = sig.to_vec();
        self
    }

    /// 티켓 유효 여부 확인
    #[inline]
    pub fn is_valid(&self, current_time_ns: u64) -> bool {
        current_time_ns >= self.valid_from_ns && current_time_ns < self.valid_to_ns
    }

    /// 만료 여부
    #[inline]
    pub fn is_expired(&self, current_time_ns: u64) -> bool {
        current_time_ns >= self.valid_to_ns
    }

    /// VTS 수
    #[inline]
    pub fn vts_count(&self) -> usize {
        self.vts_list.len()
    }

    /// 특정 시각에 유효한 VTS 반환
    pub fn current_vts(&self, timestamp_ns: u64) -> Option<&VoxelTimeSlot> {
        self.vts_list
            .iter()
            .find(|vts| vts.contains_time(timestamp_ns))
    }

    /// 잔여 VTS 수
    pub fn remaining_vts(&self, current_time_ns: u64) -> usize {
        self.vts_list
            .iter()
            .filter(|vts| vts.t_end_ns > current_time_ns)
            .count()
    }

    /// 서명 여부
    pub fn is_signed(&self) -> bool {
        self.smev_sig.iter().any(|&b| b != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transit_ticket_new() {
        let ticket = TransitTicket::new(1, 42, 1);
        assert_eq!(ticket.ticket_id, 1);
        assert_eq!(ticket.robot_id, 42);
        assert!(ticket.vts_list.is_empty());
    }

    #[test]
    fn test_transit_ticket_with_vts() {
        let vts_list = vec![
            VoxelTimeSlot::new(1, 1000, 2000),
            VoxelTimeSlot::new(2, 2000, 3000),
            VoxelTimeSlot::new(3, 3000, 4000),
        ];

        let ticket = TransitTicket::new(1, 1, 1).with_vts(vts_list);

        assert_eq!(ticket.vts_count(), 3);
        assert_eq!(ticket.valid_from_ns, 1000);
        assert_eq!(ticket.valid_to_ns, 4000);
    }

    #[test]
    fn test_transit_ticket_is_valid() {
        let ticket = TransitTicket::new(1, 1, 1).with_validity(1000, 5000);

        assert!(!ticket.is_valid(500));
        assert!(ticket.is_valid(1000));
        assert!(ticket.is_valid(3000));
        assert!(!ticket.is_valid(5000));
    }

    #[test]
    fn test_transit_ticket_current_vts() {
        let vts_list = vec![
            VoxelTimeSlot::new(1, 1000, 2000),
            VoxelTimeSlot::new(2, 2000, 3000),
        ];

        let ticket = TransitTicket::new(1, 1, 1).with_vts(vts_list);

        assert!(ticket.current_vts(500).is_none());
        assert_eq!(ticket.current_vts(1500).unwrap().voxel_id, 1);
        assert_eq!(ticket.current_vts(2500).unwrap().voxel_id, 2);
    }

    #[test]
    fn test_transit_ticket_remaining_vts() {
        let vts_list = vec![
            VoxelTimeSlot::new(1, 1000, 2000),
            VoxelTimeSlot::new(2, 2000, 3000),
            VoxelTimeSlot::new(3, 3000, 4000),
        ];

        let ticket = TransitTicket::new(1, 1, 1).with_vts(vts_list);

        assert_eq!(ticket.remaining_vts(500), 3);
        assert_eq!(ticket.remaining_vts(2500), 2);
        assert_eq!(ticket.remaining_vts(4000), 0);
    }
}
