//! ZoneManager - Zone 관리자
//!
//! PPR 매핑: AI_make_ZoneManager

use sap_core::types::Position;
use std::collections::HashMap;

/// Zone 경계 정의
#[derive(Debug, Clone)]
pub struct ZoneBoundary {
    pub zone_id: u32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl ZoneBoundary {
    pub fn new(zone_id: u32, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            zone_id,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= self.min_x && pos.x <= self.max_x && pos.y >= self.min_y && pos.y <= self.max_y
    }

    pub fn area(&self) -> f32 {
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }
}

/// Zone 관리자
pub struct ZoneManager {
    zones: HashMap<u32, ZoneBoundary>,
    robot_zones: HashMap<u64, u32>,
}

impl ZoneManager {
    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
            robot_zones: HashMap::new(),
        }
    }

    pub fn add_zone(&mut self, boundary: ZoneBoundary) {
        self.zones.insert(boundary.zone_id, boundary);
    }

    pub fn get_zone(&self, zone_id: u32) -> Option<&ZoneBoundary> {
        self.zones.get(&zone_id)
    }

    pub fn find_zone_for_position(&self, pos: Position) -> Option<u32> {
        self.zones
            .iter()
            .find(|(_, z)| z.contains(pos))
            .map(|(&id, _)| id)
    }

    pub fn update_robot_zone(&mut self, robot_id: u64, pos: Position) -> Option<u32> {
        if let Some(zone_id) = self.find_zone_for_position(pos) {
            self.robot_zones.insert(robot_id, zone_id);
            Some(zone_id)
        } else {
            None
        }
    }

    pub fn get_robot_zone(&self, robot_id: u64) -> Option<u32> {
        self.robot_zones.get(&robot_id).copied()
    }

    pub fn robots_in_zone(&self, zone_id: u32) -> Vec<u64> {
        self.robot_zones
            .iter()
            .filter(|(_, &z)| z == zone_id)
            .map(|(&r, _)| r)
            .collect()
    }

    pub fn zone_count(&self) -> usize {
        self.zones.len()
    }

    pub fn robot_count(&self) -> usize {
        self.robot_zones.len()
    }

    pub fn remove_robot(&mut self, robot_id: u64) -> bool {
        self.robot_zones.remove(&robot_id).is_some()
    }
}

impl Default for ZoneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_boundary_contains() {
        let zone = ZoneBoundary::new(1, 0.0, 10.0, 0.0, 10.0);
        assert!(zone.contains(Position::new(5.0, 5.0, 0.0)));
        assert!(!zone.contains(Position::new(15.0, 5.0, 0.0)));
    }

    #[test]
    fn test_zone_manager_add_zone() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(ZoneBoundary::new(1, 0.0, 10.0, 0.0, 10.0));
        assert_eq!(mgr.zone_count(), 1);
    }

    #[test]
    fn test_find_zone_for_position() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(ZoneBoundary::new(1, 0.0, 10.0, 0.0, 10.0));
        mgr.add_zone(ZoneBoundary::new(2, 10.0, 20.0, 0.0, 10.0));

        assert_eq!(
            mgr.find_zone_for_position(Position::new(5.0, 5.0, 0.0)),
            Some(1)
        );
        assert_eq!(
            mgr.find_zone_for_position(Position::new(15.0, 5.0, 0.0)),
            Some(2)
        );
        assert_eq!(
            mgr.find_zone_for_position(Position::new(25.0, 5.0, 0.0)),
            None
        );
    }

    #[test]
    fn test_update_robot_zone() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(ZoneBoundary::new(1, 0.0, 10.0, 0.0, 10.0));

        let zone = mgr.update_robot_zone(42, Position::new(5.0, 5.0, 0.0));
        assert_eq!(zone, Some(1));
        assert_eq!(mgr.get_robot_zone(42), Some(1));
    }

    #[test]
    fn test_robots_in_zone() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(ZoneBoundary::new(1, 0.0, 10.0, 0.0, 10.0));
        mgr.add_zone(ZoneBoundary::new(2, 10.0, 20.0, 0.0, 10.0));

        mgr.update_robot_zone(1, Position::new(5.0, 5.0, 0.0));
        mgr.update_robot_zone(2, Position::new(5.0, 5.0, 0.0));
        mgr.update_robot_zone(3, Position::new(15.0, 5.0, 0.0));

        let robots = mgr.robots_in_zone(1);
        assert_eq!(robots.len(), 2);
    }
}
