//! RobotStateManager - 로봇 상태 관리자
//!
//! PPR 매핑: AI_make_RobotState

use sap_core::types::{Position, RobotState, Velocity};

/// 로봇 상태 관리자
pub struct RobotStateManager {
    robot_id: u64,
    current_state: RobotState,
    state_history: Vec<StateSnapshot>,
    history_capacity: usize,
    last_update_ns: u64,
}

/// 상태 스냅샷
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StateSnapshot {
    pub state: RobotState,
    pub timestamp_ns: u64,
    pub source: StateSource,
}

/// 상태 소스
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateSource {
    Sensor,
    ServerCorrection,
    LocalPrediction,
}

impl RobotStateManager {
    pub fn new(robot_id: u64) -> Self {
        Self {
            robot_id,
            current_state: RobotState::new(robot_id),
            state_history: Vec::new(),
            history_capacity: 100,
            last_update_ns: 0,
        }
    }

    pub fn with_position(robot_id: u64, position: Position) -> Self {
        let mut manager = Self::new(robot_id);
        manager.current_state.position = position;
        manager
    }

    pub fn update_from_sensor(
        &mut self,
        position: Position,
        velocity: Velocity,
        timestamp_ns: u64,
    ) {
        self.current_state.position = position;
        self.current_state.velocity = velocity;
        self.record_snapshot(StateSource::Sensor, timestamp_ns);
        self.last_update_ns = timestamp_ns;
    }

    pub fn apply_correction(&mut self, position: Position, velocity: Velocity, timestamp_ns: u64) {
        self.current_state.position = position;
        self.current_state.velocity = velocity;
        self.record_snapshot(StateSource::ServerCorrection, timestamp_ns);
        self.last_update_ns = timestamp_ns;
    }

    pub fn predict(&mut self, dt_ns: u64) -> Position {
        let dt_secs = dt_ns as f32 / 1_000_000_000.0;
        let current_pos = self.current_state.position;
        let current_vel = self.current_state.velocity;

        let predicted = Position::new(
            current_pos.x + current_vel.vx * dt_secs,
            current_pos.y + current_vel.vy * dt_secs,
            current_pos.z + current_vel.vz * dt_secs,
        );

        self.current_state.position = predicted;
        let new_timestamp = self.last_update_ns + dt_ns;
        self.record_snapshot(StateSource::LocalPrediction, new_timestamp);
        self.last_update_ns = new_timestamp;

        predicted
    }

    fn record_snapshot(&mut self, source: StateSource, timestamp_ns: u64) {
        if self.state_history.len() >= self.history_capacity {
            self.state_history.remove(0);
        }
        self.state_history.push(StateSnapshot {
            state: self.current_state.clone(),
            timestamp_ns,
            source,
        });
    }

    pub fn state(&self) -> &RobotState {
        &self.current_state
    }
    pub fn position(&self) -> Position {
        self.current_state.position
    }
    pub fn velocity(&self) -> Velocity {
        self.current_state.velocity
    }
    pub fn robot_id(&self) -> u64 {
        self.robot_id
    }
    pub fn last_update_ns(&self) -> u64 {
        self.last_update_ns
    }
    pub fn history_len(&self) -> usize {
        self.state_history.len()
    }

    pub fn compute_prediction_error(&self, server_position: Position) -> f32 {
        let local = self.current_state.position;
        let dx = server_position.x - local.x;
        let dy = server_position.y - local.y;
        let dz = server_position.z - local.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let manager = RobotStateManager::new(42);
        assert_eq!(manager.robot_id(), 42);
        assert_eq!(manager.position(), Position::ORIGIN);
    }

    #[test]
    fn test_with_position() {
        let pos = Position::new(1.0, 2.0, 0.0);
        let manager = RobotStateManager::with_position(42, pos);
        assert_eq!(manager.position(), pos);
    }

    #[test]
    fn test_update_from_sensor() {
        let mut manager = RobotStateManager::new(42);
        let pos = Position::new(5.0, 5.0, 0.0);
        let vel = Velocity::new(1.0, 0.0, 0.0);
        manager.update_from_sensor(pos, vel, 1_000_000_000);
        assert_eq!(manager.position(), pos);
        assert_eq!(manager.velocity(), vel);
        assert_eq!(manager.last_update_ns(), 1_000_000_000);
        assert_eq!(manager.history_len(), 1);
    }

    #[test]
    fn test_predict() {
        let mut manager = RobotStateManager::new(42);
        let pos = Position::new(0.0, 0.0, 0.0);
        let vel = Velocity::new(1.0, 0.0, 0.0);
        manager.update_from_sensor(pos, vel, 0);
        let predicted = manager.predict(1_000_000_000);
        assert!((predicted.x - 1.0).abs() < 0.001);
        assert!((predicted.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_prediction_error() {
        let mut manager = RobotStateManager::new(42);
        let pos = Position::new(1.0, 0.0, 0.0);
        manager.update_from_sensor(pos, Velocity::ZERO, 0);
        let server_pos = Position::new(1.1, 0.0, 0.0);
        let error = manager.compute_prediction_error(server_pos);
        assert!((error - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_history_capacity() {
        let mut manager = RobotStateManager::new(42);
        manager.history_capacity = 5;
        for i in 0..10 {
            manager.update_from_sensor(Position::new(i as f32, 0.0, 0.0), Velocity::ZERO, i);
        }
        assert_eq!(manager.history_len(), 5);
    }
}
