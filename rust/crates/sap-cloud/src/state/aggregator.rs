//! GlobalStateAggregator - 글로벌 상태 집계
//!
//! PPR 매핑: AI_perceive_GlobalState

use sap_core::types::WorldState;
use std::collections::HashMap;

/// 글로벌 상태 집계기
pub struct GlobalStateAggregator {
    zone_states: HashMap<u32, WorldState>,
    last_updates: HashMap<u32, u64>,
    stats: AggregatorStats,
}

#[derive(Debug, Clone, Default)]
pub struct AggregatorStats {
    pub total_updates: u64,
    pub total_zones: usize,
}

#[derive(Debug, Clone)]
pub struct ZoneSummary {
    pub zone_id: u32,
    pub robot_count: usize,
    pub last_tick: u64,
    pub last_update_ns: u64,
}

impl GlobalStateAggregator {
    pub fn new() -> Self {
        Self {
            zone_states: HashMap::new(),
            last_updates: HashMap::new(),
            stats: AggregatorStats::default(),
        }
    }

    pub fn update_zone_state(&mut self, zone_id: u32, state: WorldState, timestamp_ns: u64) {
        self.zone_states.insert(zone_id, state);
        self.last_updates.insert(zone_id, timestamp_ns);
        self.stats.total_updates += 1;
        self.stats.total_zones = self.zone_states.len();
    }

    pub fn get_zone_state(&self, zone_id: u32) -> Option<&WorldState> {
        self.zone_states.get(&zone_id)
    }

    pub fn get_zone_summary(&self, zone_id: u32) -> Option<ZoneSummary> {
        let state = self.zone_states.get(&zone_id)?;
        let last_update = self.last_updates.get(&zone_id).copied().unwrap_or(0);

        Some(ZoneSummary {
            zone_id,
            robot_count: state.robot_count(),
            last_tick: state.tick,
            last_update_ns: last_update,
        })
    }

    pub fn get_all_zone_summaries(&self) -> Vec<ZoneSummary> {
        self.zone_states
            .keys()
            .filter_map(|&id| self.get_zone_summary(id))
            .collect()
    }

    pub fn get_stale_zones(&self, current_time_ns: u64, threshold_ns: u64) -> Vec<u32> {
        self.last_updates
            .iter()
            .filter(|(_, &last)| current_time_ns.saturating_sub(last) > threshold_ns)
            .map(|(&id, _)| id)
            .collect()
    }

    pub fn total_robot_count(&self) -> usize {
        self.zone_states.values().map(|s| s.robot_count()).sum()
    }

    pub fn zone_count(&self) -> usize {
        self.zone_states.len()
    }

    pub fn stats(&self) -> &AggregatorStats {
        &self.stats
    }

    pub fn remove_zone(&mut self, zone_id: u32) -> bool {
        let removed = self.zone_states.remove(&zone_id).is_some();
        self.last_updates.remove(&zone_id);
        if removed {
            self.stats.total_zones = self.zone_states.len();
        }
        removed
    }
}

impl Default for GlobalStateAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sap_core::types::RobotState;

    fn create_world_state(zone_id: u32, tick: u64, robot_count: usize) -> WorldState {
        let mut state = WorldState::new(zone_id).with_tick(tick, 0);
        for i in 0..robot_count {
            state.add_robot(RobotState::new(i as u64));
        }
        state
    }

    #[test]
    fn test_new() {
        let agg = GlobalStateAggregator::new();
        assert_eq!(agg.zone_count(), 0);
    }

    #[test]
    fn test_update_zone_state() {
        let mut agg = GlobalStateAggregator::new();
        agg.update_zone_state(1, create_world_state(1, 100, 5), 1_000_000_000);
        assert_eq!(agg.zone_count(), 1);
        assert!(agg.get_zone_state(1).is_some());
    }

    #[test]
    fn test_zone_summary() {
        let mut agg = GlobalStateAggregator::new();
        agg.update_zone_state(1, create_world_state(1, 100, 5), 1_000_000_000);
        let summary = agg.get_zone_summary(1).unwrap();
        assert_eq!(summary.zone_id, 1);
        assert_eq!(summary.robot_count, 5);
        assert_eq!(summary.last_tick, 100);
    }

    #[test]
    fn test_total_robot_count() {
        let mut agg = GlobalStateAggregator::new();
        agg.update_zone_state(1, create_world_state(1, 100, 5), 0);
        agg.update_zone_state(2, create_world_state(2, 100, 3), 0);
        agg.update_zone_state(3, create_world_state(3, 100, 7), 0);
        assert_eq!(agg.total_robot_count(), 15);
    }

    #[test]
    fn test_stale_zones() {
        let mut agg = GlobalStateAggregator::new();
        agg.update_zone_state(1, create_world_state(1, 100, 5), 1_000_000_000);
        agg.update_zone_state(2, create_world_state(2, 100, 3), 3_000_000_000);
        let stale = agg.get_stale_zones(5_000_000_000, 2_000_000_000);
        assert_eq!(stale.len(), 1);
        assert!(stale.contains(&1));
    }

    #[test]
    fn test_remove_zone() {
        let mut agg = GlobalStateAggregator::new();
        agg.update_zone_state(1, create_world_state(1, 100, 5), 0);
        assert!(agg.remove_zone(1));
        assert_eq!(agg.zone_count(), 0);
        assert!(!agg.remove_zone(1));
    }
}
