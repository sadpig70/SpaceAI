//! TicketManager - TransitTicket 관리자
//!
//! PPR 매핑: AI_make_TransitTicketManager

use sap_core::ticket::TransitTicket;
use std::collections::HashMap;

/// TransitTicket 관리자
pub struct TicketManager {
    /// 활성 티켓 (ticket_id -> TransitTicket)
    active_tickets: HashMap<u128, TransitTicket>,
    /// 만료된 티켓 ID 목록
    expired_tickets: Vec<u128>,
    /// 발행 카운터
    issue_counter: u128,
    /// Zone ID
    zone_id: u32,
}

impl TicketManager {
    pub fn new(zone_id: u32) -> Self {
        Self {
            active_tickets: HashMap::new(),
            expired_tickets: Vec::new(),
            issue_counter: 0,
            zone_id,
        }
    }

    /// 티켓 발행
    pub fn issue_ticket(
        &mut self,
        robot_id: u64,
        _vts_id: u64,
        valid_from_ns: u64,
        valid_to_ns: u64,
    ) -> TransitTicket {
        self.issue_counter += 1;
        let ticket_id = self.issue_counter;

        let ticket = TransitTicket::new(ticket_id, robot_id, self.zone_id)
            .with_validity(valid_from_ns, valid_to_ns);

        self.active_tickets.insert(ticket_id, ticket.clone());
        ticket
    }

    /// 티켓 검증
    pub fn validate(&self, ticket_id: u128, current_time_ns: u64) -> TicketValidation {
        match self.active_tickets.get(&ticket_id) {
            Some(ticket) => {
                if current_time_ns < ticket.valid_from_ns {
                    TicketValidation::NotYetValid
                } else if current_time_ns > ticket.valid_to_ns {
                    TicketValidation::Expired
                } else {
                    TicketValidation::Valid
                }
            }
            None => TicketValidation::NotFound,
        }
    }

    /// 티켓 조회
    pub fn get_ticket(&self, ticket_id: u128) -> Option<&TransitTicket> {
        self.active_tickets.get(&ticket_id)
    }

    /// 로봇의 활성 티켓 조회
    pub fn get_robot_tickets(&self, robot_id: u64) -> Vec<&TransitTicket> {
        self.active_tickets
            .values()
            .filter(|t| t.robot_id == robot_id)
            .collect()
    }

    /// 만료 티켓 정리
    pub fn cleanup_expired(&mut self, current_time_ns: u64) -> usize {
        let expired: Vec<u128> = self
            .active_tickets
            .iter()
            .filter(|(_, t)| current_time_ns > t.valid_to_ns)
            .map(|(&id, _)| id)
            .collect();

        let count = expired.len();
        for id in expired {
            self.active_tickets.remove(&id);
            self.expired_tickets.push(id);
        }
        count
    }

    /// 티켓 취소
    pub fn revoke(&mut self, ticket_id: u128) -> bool {
        if self.active_tickets.remove(&ticket_id).is_some() {
            self.expired_tickets.push(ticket_id);
            true
        } else {
            false
        }
    }

    /// 활성 티켓 수
    pub fn active_count(&self) -> usize {
        self.active_tickets.len()
    }

    /// 발행 총 수
    pub fn total_issued(&self) -> u128 {
        self.issue_counter
    }

    /// Zone ID 조회
    pub fn zone_id(&self) -> u32 {
        self.zone_id
    }
}

/// 티켓 검증 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketValidation {
    Valid,
    NotYetValid,
    Expired,
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_ticket() {
        let mut manager = TicketManager::new(1);

        let ticket = manager.issue_ticket(42, 100, 1000, 5000);

        assert_eq!(ticket.robot_id, 42);
        assert_eq!(manager.active_count(), 1);
        assert_eq!(manager.total_issued(), 1);
    }

    #[test]
    fn test_validate_valid() {
        let mut manager = TicketManager::new(1);
        let ticket = manager.issue_ticket(42, 100, 1000, 5000);

        let result = manager.validate(ticket.ticket_id, 3000);
        assert_eq!(result, TicketValidation::Valid);
    }

    #[test]
    fn test_validate_not_yet_valid() {
        let mut manager = TicketManager::new(1);
        let ticket = manager.issue_ticket(42, 100, 5000, 10000);

        let result = manager.validate(ticket.ticket_id, 1000);
        assert_eq!(result, TicketValidation::NotYetValid);
    }

    #[test]
    fn test_validate_expired() {
        let mut manager = TicketManager::new(1);
        let ticket = manager.issue_ticket(42, 100, 1000, 5000);

        let result = manager.validate(ticket.ticket_id, 10000);
        assert_eq!(result, TicketValidation::Expired);
    }

    #[test]
    fn test_validate_not_found() {
        let manager = TicketManager::new(1);

        let result = manager.validate(999, 3000);
        assert_eq!(result, TicketValidation::NotFound);
    }

    #[test]
    fn test_cleanup_expired() {
        let mut manager = TicketManager::new(1);

        manager.issue_ticket(1, 100, 1000, 2000);
        manager.issue_ticket(2, 100, 1000, 5000);
        manager.issue_ticket(3, 100, 1000, 3000);

        let cleaned = manager.cleanup_expired(4000);

        assert_eq!(cleaned, 2); // 2000, 3000 만료
        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_revoke() {
        let mut manager = TicketManager::new(1);
        let ticket = manager.issue_ticket(42, 100, 1000, 5000);

        assert!(manager.revoke(ticket.ticket_id));
        assert_eq!(manager.active_count(), 0);
        assert!(!manager.revoke(ticket.ticket_id)); // 이미 취소됨
    }

    #[test]
    fn test_get_robot_tickets() {
        let mut manager = TicketManager::new(1);

        manager.issue_ticket(1, 100, 1000, 5000);
        manager.issue_ticket(1, 200, 2000, 6000);
        manager.issue_ticket(2, 300, 3000, 7000);

        let tickets = manager.get_robot_tickets(1);
        assert_eq!(tickets.len(), 2);
    }
}
