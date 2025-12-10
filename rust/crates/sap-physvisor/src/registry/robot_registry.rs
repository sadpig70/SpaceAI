//! RobotRegistry - 로봇 레지스트리
//!
//! PPR 매핑: AI_perceive_RobotRegistry

use sap_core::types::{Position, RobotState, Velocity};
use std::collections::HashMap;

/// 로봇 레지스트리
pub struct RobotRegistry {
    robots: HashMap<u64, RobotState>,
    max_robots: usize,
}

impl RobotRegistry {
    pub fn new(max_robots: usize) -> Self {
        Self {
            robots: HashMap::new(),
            max_robots,
        }
    }

    pub fn with_default_capacity() -> Self {
        Self::new(1000)
    }

    pub fn register(&mut self, robot_id: u64) -> Result<(), RegistryError> {
        if self.robots.len() >= self.max_robots {
            return Err(RegistryError::CapacityExceeded);
        }
        if self.robots.contains_key(&robot_id) {
            return Err(RegistryError::AlreadyRegistered);
        }
        self.robots.insert(robot_id, RobotState::new(robot_id));
        Ok(())
    }

    pub fn unregister(&mut self, robot_id: u64) -> bool {
        self.robots.remove(&robot_id).is_some()
    }

    pub fn update_state(
        &mut self,
        robot_id: u64,
        position: Position,
        velocity: Velocity,
        timestamp_ns: u64,
    ) -> bool {
        if let Some(state) = self.robots.get_mut(&robot_id) {
            state.position = position;
            state.velocity = velocity;
            state.timestamp_ns = timestamp_ns;
            true
        } else {
            false
        }
    }

    pub fn get_state(&self, robot_id: u64) -> Option<&RobotState> {
        self.robots.get(&robot_id)
    }

    pub fn get_position(&self, robot_id: u64) -> Option<Position> {
        self.robots.get(&robot_id).map(|s| s.position)
    }

    pub fn get_all_robots(&self) -> Vec<u64> {
        self.robots.keys().copied().collect()
    }

    pub fn get_robots_in_radius(&self, center: Position, radius: f32) -> Vec<u64> {
        self.robots
            .iter()
            .filter(|(_, state)| {
                let dx = state.position.x - center.x;
                let dy = state.position.y - center.y;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .map(|(&id, _)| id)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.robots.len()
    }

    pub fn is_registered(&self, robot_id: u64) -> bool {
        self.robots.contains_key(&robot_id)
    }
}

#[derive(Debug, Clone)]
pub enum RegistryError {
    CapacityExceeded,
    AlreadyRegistered,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register() {
        let mut registry = RobotRegistry::with_default_capacity();
        assert!(registry.register(1).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_register_duplicate() {
        let mut registry = RobotRegistry::with_default_capacity();
        registry.register(1).unwrap();
        assert!(matches!(
            registry.register(1),
            Err(RegistryError::AlreadyRegistered)
        ));
    }

    #[test]
    fn test_capacity_exceeded() {
        let mut registry = RobotRegistry::new(2);
        registry.register(1).unwrap();
        registry.register(2).unwrap();
        assert!(matches!(
            registry.register(3),
            Err(RegistryError::CapacityExceeded)
        ));
    }

    #[test]
    fn test_update_state() {
        let mut registry = RobotRegistry::with_default_capacity();
        registry.register(1).unwrap();

        let pos = Position::new(10.0, 20.0, 0.0);
        let vel = Velocity::new(1.0, 0.0, 0.0);

        assert!(registry.update_state(1, pos, vel, 1_000_000_000));
        assert_eq!(registry.get_position(1), Some(pos));
    }

    #[test]
    fn test_robots_in_radius() {
        let mut registry = RobotRegistry::with_default_capacity();
        registry.register(1).unwrap();
        registry.register(2).unwrap();
        registry.register(3).unwrap();

        registry.update_state(1, Position::new(0.0, 0.0, 0.0), Velocity::ZERO, 0);
        registry.update_state(2, Position::new(5.0, 0.0, 0.0), Velocity::ZERO, 0);
        registry.update_state(3, Position::new(100.0, 0.0, 0.0), Velocity::ZERO, 0);

        let nearby = registry.get_robots_in_radius(Position::ORIGIN, 10.0);
        assert_eq!(nearby.len(), 2);
    }
}
