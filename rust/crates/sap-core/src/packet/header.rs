//! 패킷 헤더
//!
//! 모든 SAP 패킷의 공통 헤더

use serde::{Deserialize, Serialize};

/// 패킷 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketType {
    /// Zone 비콘
    ZoneBeacon = 0x01,

    /// 로봇 Hello
    Hello = 0x02,

    /// Zone Grant
    ZoneGrant = 0x03,

    /// Delta Tick (상태 동기화)
    DeltaTick = 0x10,

    /// 롤백 프레임
    RollbackFrame = 0x11,

    /// 위반 알림
    ViolationAlert = 0x12,

    /// 검증 OK
    VerificationOk = 0x13,

    /// 가격 견적 요청
    QuoteRequest = 0x20,

    /// 가격 견적 응답
    QuoteResponse = 0x21,

    /// 입찰 커밋
    BidCommit = 0x22,

    /// 입찰 리빌
    BidReveal = 0x23,

    /// 티켓 발행
    TicketIssue = 0x24,

    /// 종료 보고
    ExitReport = 0x30,
}

/// 공통 패킷 헤더
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub struct PacketHeader {
    /// 프로토콜 버전
    pub version: u8,

    /// 패킷 타입
    pub packet_type: u8,

    /// Zone ID
    pub zone_id: u32,

    /// 로봇 ID
    pub robot_id: u64,

    /// 틱 번호
    pub tick: u64,

    /// PTP 타임스탬프 (나노초)
    pub timestamp_ns: u64,

    /// 페이로드 길이 (바이트)
    pub payload_len: u32,

    /// 서명 오프셋 (0이면 서명 없음)
    pub sig_offset: u16,

    /// 서명 길이
    pub sig_len: u16,
}

impl PacketHeader {
    /// 현재 프로토콜 버전
    pub const VERSION: u8 = 1;

    /// 헤더 크기 (바이트)
    pub const SIZE: usize = 40;

    /// 새 헤더 생성
    pub fn new(packet_type: PacketType, zone_id: u32, robot_id: u64) -> Self {
        Self {
            version: Self::VERSION,
            packet_type: packet_type as u8,
            zone_id,
            robot_id,
            tick: 0,
            timestamp_ns: 0,
            payload_len: 0,
            sig_offset: 0,
            sig_len: 0,
        }
    }

    /// 틱 설정
    pub fn with_tick(mut self, tick: u64, timestamp_ns: u64) -> Self {
        self.tick = tick;
        self.timestamp_ns = timestamp_ns;
        self
    }

    /// 페이로드 길이 설정
    pub fn with_payload(mut self, len: u32) -> Self {
        self.payload_len = len;
        self
    }

    /// 서명 정보 설정
    pub fn with_signature(mut self, offset: u16, len: u16) -> Self {
        self.sig_offset = offset;
        self.sig_len = len;
        self
    }

    /// 패킷 타입 파싱
    pub fn packet_type(&self) -> Option<PacketType> {
        match self.packet_type {
            0x01 => Some(PacketType::ZoneBeacon),
            0x02 => Some(PacketType::Hello),
            0x03 => Some(PacketType::ZoneGrant),
            0x10 => Some(PacketType::DeltaTick),
            0x11 => Some(PacketType::RollbackFrame),
            0x12 => Some(PacketType::ViolationAlert),
            0x13 => Some(PacketType::VerificationOk),
            0x20 => Some(PacketType::QuoteRequest),
            0x21 => Some(PacketType::QuoteResponse),
            0x22 => Some(PacketType::BidCommit),
            0x23 => Some(PacketType::BidReveal),
            0x24 => Some(PacketType::TicketIssue),
            0x30 => Some(PacketType::ExitReport),
            _ => None,
        }
    }

    /// 서명 포함 여부
    pub fn has_signature(&self) -> bool {
        self.sig_len > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_header_new() {
        let header = PacketHeader::new(PacketType::DeltaTick, 1, 42);
        assert_eq!(header.version, 1);
        assert_eq!(header.zone_id, 1);
        assert_eq!(header.robot_id, 42);
    }

    #[test]
    fn test_packet_header_packet_type() {
        let header = PacketHeader::new(PacketType::RollbackFrame, 1, 1);
        assert_eq!(header.packet_type(), Some(PacketType::RollbackFrame));
    }

    #[test]
    fn test_packet_header_serialization() {
        let header = PacketHeader::new(PacketType::DeltaTick, 1, 42)
            .with_tick(100, 1_000_000_000)
            .with_payload(64);

        let encoded = bincode::serialize(&header).unwrap();
        let decoded: PacketHeader = bincode::deserialize(&encoded).unwrap();

        assert_eq!(header.tick, decoded.tick);
        assert_eq!(header.payload_len, decoded.payload_len);
    }
}
