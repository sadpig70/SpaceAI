//! Cross-Zone 핸드오프 프로토콜
//!
//! PPR 매핑: AI_response_HandoffProtocol

use sap_core::types::{Position, RobotState};
use serde::{Deserialize, Serialize};

/// Zone 경계 정의
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneBoundary {
    /// 원본 Zone ID
    pub from_zone_id: u32,
    /// 목표 Zone ID
    pub to_zone_id: u32,
    /// 경계 시작 위치
    pub boundary_start: Position,
    /// 경계 끝 위치
    pub boundary_end: Position,
    /// 핸드오프 트리거 거리 (미터) - 이 거리 이내 시 핸드오프 시작
    pub trigger_distance_m: f32,
}

impl ZoneBoundary {
    /// 로봇이 경계 트리거 범위 내인지 확인
    pub fn is_in_trigger_range(&self, position: &Position) -> bool {
        let distance = self.distance_to_boundary(position);
        distance < self.trigger_distance_m
    }

    /// 위치에서 경계까지의 최소 거리 계산
    pub fn distance_to_boundary(&self, position: &Position) -> f32 {
        // 선분까지의 최소 거리 계산 (2D)
        let px = position.x;
        let py = position.y;
        let ax = self.boundary_start.x;
        let ay = self.boundary_start.y;
        let bx = self.boundary_end.x;
        let by = self.boundary_end.y;

        let ab_x = bx - ax;
        let ab_y = by - ay;
        let ap_x = px - ax;
        let ap_y = py - ay;

        let ab_sq = ab_x * ab_x + ab_y * ab_y;
        if ab_sq == 0.0 {
            // 시작점과 끝점이 동일한 경우
            return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
        }

        let t = ((ap_x * ab_x + ap_y * ab_y) / ab_sq).clamp(0.0, 1.0);
        let nearest_x = ax + t * ab_x;
        let nearest_y = ay + t * ab_y;

        ((px - nearest_x).powi(2) + (py - nearest_y).powi(2)).sqrt()
    }
}

/// 사전 VTS 할당 요청
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveAllocation {
    /// 요청 ID
    pub request_id: u128,
    /// 로봇 ID
    pub robot_id: u64,
    /// 현재 Zone ID
    pub from_zone_id: u32,
    /// 목표 Zone ID
    pub to_zone_id: u32,
    /// 예상 경계 진입 시각 (나노초)
    pub estimated_crossing_time_ns: u64,
    /// 요청된 VTS 목록 (voxel_id, t_start, t_end)
    pub requested_vts: Vec<(u64, u64, u64)>,
    /// 우선순위
    pub priority: u8,
    /// 요청 생성 시각
    pub created_at_ns: u64,
}

impl PredictiveAllocation {
    /// 새 사전 할당 요청 생성
    pub fn new(
        robot_id: u64,
        from_zone_id: u32,
        to_zone_id: u32,
        estimated_crossing_time_ns: u64,
    ) -> Self {
        Self {
            request_id: 0, // Cloud에서 할당
            robot_id,
            from_zone_id,
            to_zone_id,
            estimated_crossing_time_ns,
            requested_vts: Vec::new(),
            priority: 0,
            created_at_ns: 0,
        }
    }

    /// VTS 요청 추가
    pub fn add_vts(&mut self, voxel_id: u64, t_start_ns: u64, t_end_ns: u64) {
        self.requested_vts.push((voxel_id, t_start_ns, t_end_ns));
    }
}

/// 핸드오프 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandoffState {
    /// 미시작
    Idle,
    /// 사전 할당 요청됨
    PredictiveAllocRequested,
    /// 사전 할당 승인됨
    PredictiveAllocGranted,
    /// 핸드오프 요청 전송됨
    RequestSent,
    /// 핸드오프 승인 대기
    AwaitingApproval,
    /// 상태 전송 중
    TransferringState,
    /// 핸드오프 완료
    Completed,
    /// 핸드오프 실패
    Failed,
    /// 핸드오프 취소
    Cancelled,
}

/// 핸드오프 요청 (Edge → Edge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRequest {
    /// 요청 ID
    pub handoff_id: u128,
    /// 로봇 ID
    pub robot_id: u64,
    /// 현재 Zone (요청 발신)
    pub from_zone_id: u32,
    /// 목표 Zone (요청 수신)
    pub to_zone_id: u32,
    /// 로봇 현재 상태
    pub robot_state: RobotState,
    /// 현재 보유 티켓 ID
    pub ticket_id: u128,
    /// 사전 할당된 목표 Zone VTS ID (있으면)
    pub preallocated_vts_ids: Vec<u128>,
    /// 예상 경계 통과 시각 (나노초)
    pub expected_crossing_time_ns: u64,
    /// 요청 생성 시각 (나노초)
    pub created_at_ns: u64,
    /// 요청 만료 시각 (나노초)
    pub expires_at_ns: u64,
}

impl HandoffRequest {
    /// 새 핸드오프 요청 생성
    pub fn new(
        robot_id: u64,
        from_zone_id: u32,
        to_zone_id: u32,
        robot_state: RobotState,
        ticket_id: u128,
    ) -> Self {
        Self {
            handoff_id: 0, // 발신 Edge에서 할당
            robot_id,
            from_zone_id,
            to_zone_id,
            robot_state,
            ticket_id,
            preallocated_vts_ids: Vec::new(),
            expected_crossing_time_ns: 0,
            created_at_ns: 0,
            expires_at_ns: 0,
        }
    }

    /// 요청 만료 여부
    pub fn is_expired(&self, current_time_ns: u64) -> bool {
        current_time_ns >= self.expires_at_ns
    }
}

/// 핸드오프 응답 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandoffStatus {
    /// 승인
    Accepted,
    /// 거부 - 용량 초과
    RejectedCapacityFull,
    /// 거부 - VTS 충돌
    RejectedVtsConflict,
    /// 거부 - 로봇 알 수 없음
    RejectedUnknownRobot,
    /// 거부 - 시간 초과
    RejectedTimeout,
    /// 보류 - 추가 정보 필요
    Pending,
}

/// 핸드오프 응답 (Edge → Edge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffResponse {
    /// 요청 ID (HandoffRequest.handoff_id)
    pub handoff_id: u128,
    /// 응답 상태
    pub status: HandoffStatus,
    /// 목표 Zone에서 새로 할당된 티켓 ID (승인 시)
    pub new_ticket_id: Option<u128>,
    /// 에러 메시지 (거부 시)
    pub error_message: Option<String>,
    /// 응답 시각 (나노초)
    pub responded_at_ns: u64,
}

impl HandoffResponse {
    /// 승인 응답 생성
    pub fn accept(handoff_id: u128, new_ticket_id: u128, responded_at_ns: u64) -> Self {
        Self {
            handoff_id,
            status: HandoffStatus::Accepted,
            new_ticket_id: Some(new_ticket_id),
            error_message: None,
            responded_at_ns,
        }
    }

    /// 거부 응답 생성
    pub fn reject(
        handoff_id: u128,
        status: HandoffStatus,
        error: impl Into<String>,
        responded_at_ns: u64,
    ) -> Self {
        Self {
            handoff_id,
            status,
            new_ticket_id: None,
            error_message: Some(error.into()),
            responded_at_ns,
        }
    }

    /// 승인 여부
    pub fn is_accepted(&self) -> bool {
        self.status == HandoffStatus::Accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_boundary_trigger() {
        let boundary = ZoneBoundary {
            from_zone_id: 1,
            to_zone_id: 2,
            boundary_start: Position::new(10.0, 0.0, 0.0),
            boundary_end: Position::new(10.0, 10.0, 0.0),
            trigger_distance_m: 1.0,
        };

        // 경계에서 0.5m 떨어진 위치 - 트리거 범위 내
        let pos_in = Position::new(9.5, 5.0, 0.0);
        assert!(boundary.is_in_trigger_range(&pos_in));

        // 경계에서 2m 떨어진 위치 - 트리거 범위 밖
        let pos_out = Position::new(8.0, 5.0, 0.0);
        assert!(!boundary.is_in_trigger_range(&pos_out));
    }

    #[test]
    fn test_predictive_allocation() {
        let mut alloc = PredictiveAllocation::new(42, 1, 2, 1_000_000_000);
        alloc.add_vts(100, 1_000_000_000, 2_000_000_000);
        alloc.add_vts(101, 2_000_000_000, 3_000_000_000);

        assert_eq!(alloc.robot_id, 42);
        assert_eq!(alloc.requested_vts.len(), 2);
    }

    #[test]
    fn test_handoff_request() {
        let state = RobotState::new(42);
        let req = HandoffRequest::new(42, 1, 2, state, 12345);

        assert_eq!(req.robot_id, 42);
        assert_eq!(req.from_zone_id, 1);
        assert_eq!(req.to_zone_id, 2);
    }

    #[test]
    fn test_handoff_response_accept() {
        let resp = HandoffResponse::accept(1, 99999, 1_000_000_000);
        assert!(resp.is_accepted());
        assert_eq!(resp.new_ticket_id, Some(99999));
    }

    #[test]
    fn test_handoff_response_reject() {
        let resp = HandoffResponse::reject(
            1,
            HandoffStatus::RejectedCapacityFull,
            "Zone at capacity",
            1_000_000_000,
        );
        assert!(!resp.is_accepted());
        assert!(resp.error_message.is_some());
    }
}
