//! 월드 상태 타입
//!
//! PPR 매핑: AI_perceive_WorldState, AI_process_StateComparison

use super::RobotState;
use serde::{Deserialize, Serialize};

/// 전체 월드 상태 (Zone 내 모든 로봇 + 장애물)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Zone ID
    pub zone_id: u32,

    /// 현재 틱 번호
    pub tick: u64,

    /// PTP 동기화된 타임스탬프 (나노초)
    pub timestamp_ns: u64,

    /// Zone 내 모든 로봇 상태
    pub robots: Vec<RobotState>,

    /// 정적 장애물 목록 (간소화: 위치만)
    pub static_obstacles: Vec<crate::types::Position>,

    /// 동적 장애물 (사람, 비등록 물체 등)
    pub dynamic_obstacles: Vec<DynamicObstacle>,

    /// 현재 유효한 VTS 할당 목록 (로봇ID → VTS 목록)
    pub vts_allocations: std::collections::HashMap<u64, Vec<VtsAllocationInfo>>,
}

/// VTS 할당 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VtsAllocationInfo {
    /// VTS ID
    pub vts_id: u128,
    /// voxel ID
    pub voxel_id: u64,
    /// 시작 시간 (나노초)
    pub t_start_ns: u64,
    /// 종료 시간 (나노초)
    pub t_end_ns: u64,
    /// 할당 티켓 ID
    pub ticket_id: u128,
}

/// 동적 장애물
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicObstacle {
    /// 장애물 ID
    pub id: u64,

    /// 현재 위치
    pub position: crate::types::Position,

    /// 예상 속도
    pub velocity: crate::types::Velocity,

    /// 바운딩 박스 반경 (미터)
    pub radius: f32,
}

impl WorldState {
    /// 새 WorldState 생성
    pub fn new(zone_id: u32) -> Self {
        Self {
            zone_id,
            tick: 0,
            timestamp_ns: 0,
            robots: Vec::new(),
            static_obstacles: Vec::new(),
            dynamic_obstacles: Vec::new(),
            vts_allocations: std::collections::HashMap::new(),
        }
    }

    /// 틱 업데이트
    pub fn with_tick(mut self, tick: u64, timestamp_ns: u64) -> Self {
        self.tick = tick;
        self.timestamp_ns = timestamp_ns;
        self
    }

    /// 로봇 추가
    pub fn add_robot(&mut self, robot: RobotState) {
        self.robots.push(robot);
    }

    /// 특정 로봇 상태 조회
    pub fn get_robot(&self, robot_id: u64) -> Option<&RobotState> {
        self.robots.iter().find(|r| r.robot_id == robot_id)
    }

    /// 특정 로봇 상태 업데이트
    pub fn update_robot(&mut self, robot: RobotState) {
        if let Some(existing) = self
            .robots
            .iter_mut()
            .find(|r| r.robot_id == robot.robot_id)
        {
            *existing = robot;
        } else {
            self.robots.push(robot);
        }
    }

    /// Zone 내 로봇 수
    pub fn robot_count(&self) -> usize {
        self.robots.len()
    }

    /// 월드 상태 해시 (롤백용)
    pub fn compute_hash(&self) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 간소화된 해시: zone_id + tick + 로봇 수
        self.zone_id.hash(&mut hasher);
        self.tick.hash(&mut hasher);
        self.robots.len().hash(&mut hasher);

        let hash = hasher.finish();
        let mut result = [0u8; 32];
        result[0..8].copy_from_slice(&hash.to_le_bytes());
        result
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Position, Velocity};

    #[test]
    fn test_world_state_new() {
        let world = WorldState::new(1);
        assert_eq!(world.zone_id, 1);
        assert_eq!(world.robot_count(), 0);
    }

    #[test]
    fn test_world_state_add_robot() {
        let mut world = WorldState::new(1);
        world.add_robot(RobotState::new(42));
        assert_eq!(world.robot_count(), 1);
        assert!(world.get_robot(42).is_some());
    }

    #[test]
    fn test_world_state_update_robot() {
        let mut world = WorldState::new(1);
        world.add_robot(RobotState::new(42));

        let updated =
            RobotState::new(42).with_motion(Position::new(10.0, 0.0, 0.0), Velocity::ZERO);
        world.update_robot(updated);

        let robot = world.get_robot(42).unwrap();
        assert_eq!(robot.position.x, 10.0);
    }

    #[test]
    fn test_world_state_hash() {
        let world1 = WorldState::new(1).with_tick(100, 0);
        let world2 = WorldState::new(1).with_tick(100, 0);
        let world3 = WorldState::new(1).with_tick(101, 0);

        assert_eq!(world1.compute_hash(), world2.compute_hash());
        assert_ne!(world1.compute_hash(), world3.compute_hash());
    }
}
