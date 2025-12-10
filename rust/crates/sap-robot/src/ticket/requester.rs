//! TicketRequester - 티켓 요청 및 관리
//!
//! PPR 매핑: AI_request_TransitTicket

use sap_core::ticket::TransitTicket;
use sap_core::types::Position;
use std::collections::HashMap;

/// 티켓 요청자
///
/// 로봇이 서버에 티켓을 요청하고 로컬에서 관리
pub struct TicketRequester {
    robot_id: u64,
    /// 활성 티켓 (ticket_id -> TransitTicket)
    active_tickets: HashMap<u128, TransitTicket>,
    /// 대기 중인 요청 (request_id -> TicketRequest)
    pending_requests: HashMap<u64, TicketRequest>,
    /// 요청 카운터
    request_counter: u64,
}

/// 티켓 요청
#[derive(Debug, Clone)]
pub struct TicketRequest {
    pub request_id: u64,
    pub robot_id: u64,
    pub destination: Position,
    pub priority: u8,
    pub max_price: u64,
    pub created_at_ns: u64,
}

impl TicketRequester {
    /// 새 TicketRequester 생성
    pub fn new(robot_id: u64) -> Self {
        Self {
            robot_id,
            active_tickets: HashMap::new(),
            pending_requests: HashMap::new(),
            request_counter: 0,
        }
    }

    /// 티켓 요청 생성
    pub fn create_request(
        &mut self,
        destination: Position,
        priority: u8,
        max_price: u64,
        timestamp_ns: u64,
    ) -> TicketRequest {
        self.request_counter += 1;

        let request = TicketRequest {
            request_id: self.request_counter,
            robot_id: self.robot_id,
            destination,
            priority,
            max_price,
            created_at_ns: timestamp_ns,
        };

        self.pending_requests
            .insert(request.request_id, request.clone());
        request
    }

    /// 티켓 수신 및 등록
    pub fn receive_ticket(&mut self, request_id: u64, ticket: TransitTicket) -> bool {
        if self.pending_requests.remove(&request_id).is_some() {
            self.active_tickets.insert(ticket.ticket_id, ticket);
            true
        } else {
            false
        }
    }

    /// 유효한 티켓 조회
    pub fn get_valid_ticket(&self, current_time_ns: u64) -> Option<&TransitTicket> {
        self.active_tickets
            .values()
            .find(|t| t.is_valid(current_time_ns))
    }

    /// 특정 티켓 조회
    pub fn get_ticket(&self, ticket_id: u128) -> Option<&TransitTicket> {
        self.active_tickets.get(&ticket_id)
    }

    /// 티켓 유효성 확인
    pub fn is_ticket_valid(&self, ticket_id: u128, current_time_ns: u64) -> bool {
        self.active_tickets
            .get(&ticket_id)
            .map(|t| t.is_valid(current_time_ns))
            .unwrap_or(false)
    }

    /// 만료 티켓 정리
    pub fn cleanup_expired(&mut self, current_time_ns: u64) -> usize {
        let expired: Vec<u128> = self
            .active_tickets
            .iter()
            .filter(|(_, t)| t.is_expired(current_time_ns))
            .map(|(&id, _)| id)
            .collect();

        let count = expired.len();
        for id in expired {
            self.active_tickets.remove(&id);
        }
        count
    }

    /// 요청 취소
    pub fn cancel_request(&mut self, request_id: u64) -> bool {
        self.pending_requests.remove(&request_id).is_some()
    }

    /// 활성 티켓 수
    pub fn active_ticket_count(&self) -> usize {
        self.active_tickets.len()
    }

    /// 대기 요청 수
    pub fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }

    /// 로봇 ID 조회
    pub fn robot_id(&self) -> u64 {
        self.robot_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ticket(ticket_id: u128, valid_from: u64, valid_to: u64) -> TransitTicket {
        TransitTicket::new(ticket_id, 42, 1).with_validity(valid_from, valid_to)
    }

    #[test]
    fn test_new() {
        let requester = TicketRequester::new(42);
        assert_eq!(requester.robot_id(), 42);
        assert_eq!(requester.active_ticket_count(), 0);
        assert_eq!(requester.pending_request_count(), 0);
    }

    #[test]
    fn test_create_request() {
        let mut requester = TicketRequester::new(42);

        let request =
            requester.create_request(Position::new(10.0, 20.0, 0.0), 1, 1000, 1_000_000_000);

        assert_eq!(request.request_id, 1);
        assert_eq!(request.robot_id, 42);
        assert_eq!(requester.pending_request_count(), 1);
    }

    #[test]
    fn test_receive_ticket() {
        let mut requester = TicketRequester::new(42);

        let request = requester.create_request(Position::ORIGIN, 0, 1000, 0);
        let ticket = create_test_ticket(100, 0, 10_000_000_000);

        assert!(requester.receive_ticket(request.request_id, ticket));
        assert_eq!(requester.active_ticket_count(), 1);
        assert_eq!(requester.pending_request_count(), 0);
    }

    #[test]
    fn test_receive_ticket_invalid_request() {
        let mut requester = TicketRequester::new(42);
        let ticket = create_test_ticket(100, 0, 10_000_000_000);

        // 존재하지 않는 request_id
        assert!(!requester.receive_ticket(999, ticket));
        assert_eq!(requester.active_ticket_count(), 0);
    }

    #[test]
    fn test_is_ticket_valid() {
        let mut requester = TicketRequester::new(42);
        let request = requester.create_request(Position::ORIGIN, 0, 1000, 0);
        let ticket = create_test_ticket(100, 1_000_000_000, 5_000_000_000);

        requester.receive_ticket(request.request_id, ticket);

        assert!(!requester.is_ticket_valid(100, 500_000_000)); // 아직 시작 안 함
        assert!(requester.is_ticket_valid(100, 3_000_000_000)); // 유효
        assert!(!requester.is_ticket_valid(100, 10_000_000_000)); // 만료
    }

    #[test]
    fn test_cleanup_expired() {
        let mut requester = TicketRequester::new(42);

        let r1 = requester.create_request(Position::ORIGIN, 0, 1000, 0);
        let r2 = requester.create_request(Position::ORIGIN, 0, 1000, 0);

        requester.receive_ticket(r1.request_id, create_test_ticket(100, 0, 1_000_000_000));
        requester.receive_ticket(r2.request_id, create_test_ticket(200, 0, 5_000_000_000));

        let cleaned = requester.cleanup_expired(3_000_000_000);

        assert_eq!(cleaned, 1);
        assert_eq!(requester.active_ticket_count(), 1);
    }

    #[test]
    fn test_cancel_request() {
        let mut requester = TicketRequester::new(42);
        let request = requester.create_request(Position::ORIGIN, 0, 1000, 0);

        assert!(requester.cancel_request(request.request_id));
        assert_eq!(requester.pending_request_count(), 0);
        assert!(!requester.cancel_request(request.request_id)); // 이미 취소됨
    }
}
