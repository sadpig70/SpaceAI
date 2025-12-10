//! Delta Tick 패킷
//!
//! PPR 매핑: AI_make_DeltaTick

use crate::types::{Acceleration, Position, Velocity};
use serde::{Deserialize, Serialize};

/// Delta Tick 패킷 - 50ms마다 전송
///
/// 로봇의 현재 상태와 예측 대비 오차를 전송
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct DeltaTickPacket {
    /// Zone ID
    pub zone_id: u32,

    /// 로봇 ID
    pub robot_id: u64,

    /// 논리적 틱 번호
    pub tick: u64,

    /// 사용 중인 월드 모델 해시 (16바이트)
    pub model_hash: [u8; 16],

    /// PTP 동기화 타임스탬프 (나노초)
    pub timestamp_ns: u64,

    /// 현재 위치
    pub position: Position,

    /// 현재 속도
    pub velocity: Velocity,

    /// 현재 가속도
    pub acceleration: Acceleration,

    /// 방향각 (라디안)
    pub theta: f32,

    /// 예측 대비 위치 오차 (actual - predicted)
    pub delta_position: Position,

    /// 예측 대비 방향 오차
    pub delta_theta: f32,

    /// 목표 세그먼트/레인 ID
    pub target_segment: u32,

    /// 현재 티켓 ID
    pub ticket_id: u128,

    /// 예상 도착 시간 (밀리초)
    pub eta_ms: u32,

    /// 컨트롤러 온도 (섭씨)
    pub controller_temp_c: i16,

    /// 배터리 잔량 (밀리 퍼센트)
    pub battery_soc_milli: u16,
}

impl DeltaTickPacket {
    /// 새 DeltaTickPacket 생성
    pub fn new(zone_id: u32, robot_id: u64, tick: u64) -> Self {
        Self {
            zone_id,
            robot_id,
            tick,
            model_hash: [0u8; 16],
            timestamp_ns: 0,
            position: Position::ORIGIN,
            velocity: Velocity::ZERO,
            acceleration: Acceleration::ZERO,
            theta: 0.0,
            delta_position: Position::ORIGIN,
            delta_theta: 0.0,
            target_segment: 0,
            ticket_id: 0,
            eta_ms: 0,
            controller_temp_c: 25,
            battery_soc_milli: 65535, // Max for u16 (100% = 65535)
        }
    }

    /// 모션 데이터 설정
    pub fn with_motion(
        mut self,
        position: Position,
        velocity: Velocity,
        acceleration: Acceleration,
        theta: f32,
    ) -> Self {
        self.position = position;
        self.velocity = velocity;
        self.acceleration = acceleration;
        self.theta = theta;
        self
    }

    /// 델타 (예측 오차) 설정
    pub fn with_delta(mut self, delta_position: Position, delta_theta: f32) -> Self {
        self.delta_position = delta_position;
        self.delta_theta = delta_theta;
        self
    }

    /// 타임스탬프 설정
    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }

    /// 예측 오차 크기 (위치)
    ///
    /// PPR: AI_process_StateComparison
    #[inline]
    pub fn delta_magnitude(&self) -> f32 {
        self.delta_position.magnitude()
    }

    /// 롤백 필요 여부 판단
    ///
    /// 오차가 임계값을 초과하면 롤백 필요
    #[inline]
    pub fn needs_rollback(&self, threshold: f32) -> bool {
        self.delta_magnitude() > threshold
    }

    /// 속도 크기
    #[inline]
    pub fn speed(&self) -> f32 {
        self.velocity.magnitude()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_tick_new() {
        let packet = DeltaTickPacket::new(1, 42, 100);
        assert_eq!(packet.zone_id, 1);
        assert_eq!(packet.robot_id, 42);
        assert_eq!(packet.tick, 100);
    }

    #[test]
    fn test_delta_tick_delta_magnitude() {
        let packet = DeltaTickPacket::new(1, 1, 1).with_delta(Position::new(0.03, 0.04, 0.0), 0.0);

        assert!((packet.delta_magnitude() - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_delta_tick_needs_rollback() {
        let small_delta =
            DeltaTickPacket::new(1, 1, 1).with_delta(Position::new(0.05, 0.0, 0.0), 0.0);
        assert!(!small_delta.needs_rollback(0.1));

        let large_delta =
            DeltaTickPacket::new(1, 1, 1).with_delta(Position::new(0.15, 0.1, 0.0), 0.0);
        assert!(large_delta.needs_rollback(0.1));
    }

    #[test]
    fn test_delta_tick_serialization() {
        let packet = DeltaTickPacket::new(1, 42, 100).with_motion(
            Position::new(1.0, 2.0, 0.0),
            Velocity::new(1.0, 0.0, 0.0),
            Acceleration::ZERO,
            0.5,
        );

        let encoded = bincode::serialize(&packet).unwrap();
        let decoded: DeltaTickPacket = bincode::deserialize(&encoded).unwrap();

        assert_eq!(packet.robot_id, decoded.robot_id);
        assert_eq!(packet.position, decoded.position);
    }
}
