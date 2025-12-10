//! SimulationEngine - 시뮬레이션 엔진
//!
//! PPR 매핑: AI_process_Simulation

use crate::registry::RobotRegistry;
use crate::zone::ZoneManager;
use sap_core::types::{Position, Velocity};

/// 시뮬레이션 엔진
pub struct SimulationEngine {
    registry: RobotRegistry,
    zone_manager: ZoneManager,
    current_tick: u64,
    tick_interval_ns: u64,
    collision_radius: f32,
}

/// 충돌 이벤트
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub robot_a: u64,
    pub robot_b: u64,
    pub distance: f32,
    pub tick: u64,
}

/// 시뮬레이션 결과
#[derive(Debug, Clone, Default)]
pub struct SimulationResult {
    pub robots_updated: usize,
    pub collisions_detected: Vec<CollisionEvent>,
    pub zone_changes: Vec<(u64, u32, u32)>,
}

impl SimulationEngine {
    pub fn new(max_robots: usize) -> Self {
        Self {
            registry: RobotRegistry::new(max_robots),
            zone_manager: ZoneManager::new(),
            current_tick: 0,
            tick_interval_ns: 20_000_000, // 20ms (50Hz)
            collision_radius: 0.5,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(1000)
    }

    pub fn add_zone(&mut self, zone_id: u32, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        use crate::zone::manager::ZoneBoundary;
        self.zone_manager
            .add_zone(ZoneBoundary::new(zone_id, min_x, max_x, min_y, max_y));
    }

    pub fn register_robot(&mut self, robot_id: u64) -> bool {
        self.registry.register(robot_id).is_ok()
    }

    pub fn update_robot(&mut self, robot_id: u64, position: Position, velocity: Velocity) {
        let timestamp = self.current_tick * self.tick_interval_ns;
        self.registry
            .update_state(robot_id, position, velocity, timestamp);
        self.zone_manager.update_robot_zone(robot_id, position);
    }

    pub fn step(&mut self) -> SimulationResult {
        self.current_tick += 1;
        let mut result = SimulationResult::default();

        // 충돌 감지
        let robots = self.registry.get_all_robots();
        for i in 0..robots.len() {
            for j in (i + 1)..robots.len() {
                let id_a = robots[i];
                let id_b = robots[j];

                if let (Some(pos_a), Some(pos_b)) = (
                    self.registry.get_position(id_a),
                    self.registry.get_position(id_b),
                ) {
                    let dx = pos_a.x - pos_b.x;
                    let dy = pos_a.y - pos_b.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < self.collision_radius * 2.0 {
                        result.collisions_detected.push(CollisionEvent {
                            robot_a: id_a,
                            robot_b: id_b,
                            distance: dist,
                            tick: self.current_tick,
                        });
                    }
                }
            }
        }

        result.robots_updated = robots.len();
        result
    }

    pub fn get_position(&self, robot_id: u64) -> Option<Position> {
        self.registry.get_position(robot_id)
    }

    pub fn robot_count(&self) -> usize {
        self.registry.count()
    }

    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    pub fn zone_manager(&self) -> &ZoneManager {
        &self.zone_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let engine = SimulationEngine::with_default_config();
        assert_eq!(engine.current_tick(), 0);
        assert_eq!(engine.robot_count(), 0);
    }

    #[test]
    fn test_register_and_update() {
        let mut engine = SimulationEngine::with_default_config();
        assert!(engine.register_robot(1));

        let pos = Position::new(5.0, 5.0, 0.0);
        engine.update_robot(1, pos, Velocity::ZERO);

        assert_eq!(engine.get_position(1), Some(pos));
    }

    #[test]
    fn test_step() {
        let mut engine = SimulationEngine::with_default_config();
        engine.register_robot(1);
        engine.register_robot(2);

        let result = engine.step();

        assert_eq!(engine.current_tick(), 1);
        assert_eq!(result.robots_updated, 2);
    }

    #[test]
    fn test_collision_detection() {
        let mut engine = SimulationEngine::with_default_config();
        engine.register_robot(1);
        engine.register_robot(2);

        // 가까운 위치
        engine.update_robot(1, Position::new(0.0, 0.0, 0.0), Velocity::ZERO);
        engine.update_robot(2, Position::new(0.3, 0.0, 0.0), Velocity::ZERO);

        let result = engine.step();

        assert_eq!(result.collisions_detected.len(), 1);
    }

    #[test]
    fn test_no_collision() {
        let mut engine = SimulationEngine::with_default_config();
        engine.register_robot(1);
        engine.register_robot(2);

        // 먼 위치
        engine.update_robot(1, Position::new(0.0, 0.0, 0.0), Velocity::ZERO);
        engine.update_robot(2, Position::new(10.0, 0.0, 0.0), Velocity::ZERO);

        let result = engine.step();

        assert!(result.collisions_detected.is_empty());
    }

    #[test]
    fn test_zone_integration() {
        let mut engine = SimulationEngine::with_default_config();
        engine.add_zone(1, 0.0, 10.0, 0.0, 10.0);
        engine.register_robot(42);

        engine.update_robot(42, Position::new(5.0, 5.0, 0.0), Velocity::ZERO);

        assert_eq!(engine.zone_manager().get_robot_zone(42), Some(1));
    }
}
