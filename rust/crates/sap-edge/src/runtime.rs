//! EdgeRuntime - L2+L3+L4 통합 런타임
//!
//! 물리 검증, 동기화, 경제 시스템을 통합

use sap_core::types::WorldState;
use sap_core::validation::ValidationResult;
use sap_economy::auction::{AuctionResult, BidEntry, VickreyAuction};
use sap_economy::pricing::PricingEngine;
use sap_economy::ticket::TicketManager;
use sap_network::failsafe::{FailsafeAction, FailsafeManager};
use sap_network::rollback::{RollbackManager, RollbackReason};
use sap_network::sync::{StateComparator, SyncResult};
use sap_physics::command::MotionCommand;
use sap_physics::validator::{PhysicsValidator, PhysicsValidatorConfig};

/// Edge Runtime - 전체 L2+L3+L4 통합
pub struct EdgeRuntime {
    zone_id: u32,
    physics_validator: PhysicsValidator,
    state_comparator: StateComparator,
    rollback_manager: RollbackManager,
    failsafe_manager: FailsafeManager,
    auction: VickreyAuction,
    pricing_engine: PricingEngine,
    ticket_manager: TicketManager,
    current_tick: u64,
    stats: RuntimeStats,
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeStats {
    pub total_commands: u64,
    pub passed_commands: u64,
    pub adjusted_commands: u64,
    pub rejected_commands: u64,
    pub rollback_count: u64,
    pub auction_count: u64,
    pub ticket_issued: u64,
}

impl EdgeRuntime {
    pub fn new(zone_id: u32) -> Self {
        Self {
            zone_id,
            physics_validator: PhysicsValidator::with_default_config(),
            state_comparator: StateComparator::with_default_config(),
            rollback_manager: RollbackManager::with_default_config(zone_id),
            failsafe_manager: FailsafeManager::with_default_config(zone_id),
            auction: VickreyAuction::with_default_config(),
            pricing_engine: PricingEngine::with_default_config(),
            ticket_manager: TicketManager::new(zone_id),
            current_tick: 0,
            stats: RuntimeStats::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_config(zone_id: u32, physics_config: PhysicsValidatorConfig) -> Self {
        Self {
            zone_id,
            physics_validator: PhysicsValidator::new(physics_config),
            state_comparator: StateComparator::with_default_config(),
            rollback_manager: RollbackManager::with_default_config(zone_id),
            failsafe_manager: FailsafeManager::with_default_config(zone_id),
            auction: VickreyAuction::with_default_config(),
            pricing_engine: PricingEngine::with_default_config(),
            ticket_manager: TicketManager::new(zone_id),
            current_tick: 0,
            stats: RuntimeStats::default(),
        }
    }

    pub fn tick(&mut self, timestamp_ns: u64) {
        self.current_tick += 1;
        #[allow(clippy::manual_is_multiple_of)]
        if self.current_tick % 10 == 0 {
            let state = WorldState::new(self.zone_id).with_tick(self.current_tick, timestamp_ns);
            self.rollback_manager
                .save_snapshot(self.current_tick, state);
        }
        self.ticket_manager.cleanup_expired(timestamp_ns);
    }

    pub fn process_command(&mut self, cmd: &MotionCommand, timestamp_ns: u64) -> CommandResult {
        self.stats.total_commands += 1;
        let validation = self.physics_validator.validate(cmd, &[], timestamp_ns);
        match validation {
            ValidationResult::OK => {
                self.stats.passed_commands += 1;
                self.rollback_manager.reset_consecutive(cmd.robot_id);
                CommandResult::Passed
            }
            ValidationResult::ADJUST => {
                self.stats.adjusted_commands += 1;
                CommandResult::Adjusted {
                    reason: "velocity/acceleration clamped".to_string(),
                }
            }
            ValidationResult::REJECT => {
                self.stats.rejected_commands += 1;
                CommandResult::Rejected {
                    reason: "collision or constraint violation".to_string(),
                }
            }
        }
    }

    pub fn check_sync(
        &mut self,
        robot_id: u64,
        position_delta: f32,
        timestamp_ns: u64,
    ) -> SyncCheckResult {
        let sync_result =
            self.state_comparator
                .compare_delta(robot_id, self.current_tick, position_delta, 0.0);
        match sync_result {
            SyncResult::InSync => SyncCheckResult::InSync,
            SyncResult::Warning => SyncCheckResult::Warning,
            SyncResult::NeedsRollback => {
                match self.rollback_manager.execute_rollback(
                    robot_id,
                    self.current_tick,
                    RollbackReason::PredictionError {
                        delta: position_delta,
                    },
                    timestamp_ns,
                ) {
                    Ok(frame) => {
                        self.stats.rollback_count += 1;
                        SyncCheckResult::RolledBack {
                            to_tick: frame.rollback_tick,
                        }
                    }
                    Err(_) => SyncCheckResult::RollbackFailed,
                }
            }
        }
    }

    pub fn check_failsafe(&mut self, current_time_ns: u64) -> FailsafeAction {
        self.failsafe_manager.check_and_decide(current_time_ns)
    }

    pub fn register_edge(&mut self, edge_id: u32) {
        self.failsafe_manager.register_edge(edge_id);
    }

    pub fn receive_heartbeat(&mut self, edge_id: u32, timestamp_ns: u64) {
        self.failsafe_manager
            .receive_heartbeat(edge_id, timestamp_ns);
    }

    pub fn submit_bid(
        &mut self,
        robot_id: u64,
        vts_id: u64,
        amount: u64,
        timestamp_ns: u64,
    ) -> Result<(), String> {
        let bid = BidEntry {
            robot_id,
            bid_amount: amount,
            timestamp_ns,
            vts_id,
        };
        self.auction
            .submit_bid(bid)
            .map_err(|e| format!("{:?}", e))?;
        self.pricing_engine.record_demand(vts_id);
        Ok(())
    }

    pub fn settle_auction(&mut self, vts_id: u64, timestamp_ns: u64) -> Option<AuctionResult> {
        let result = self.auction.settle(vts_id, timestamp_ns)?;
        let _ticket = self.ticket_manager.issue_ticket(
            result.winner_id,
            vts_id,
            timestamp_ns,
            timestamp_ns + 60_000_000_000,
        );
        self.pricing_engine
            .record_transaction(vts_id, result.winning_price);
        self.stats.auction_count += 1;
        self.stats.ticket_issued += 1;
        Some(result)
    }

    pub fn quote_price(&mut self, vts_id: u64, timestamp_ns: u64) -> u64 {
        self.pricing_engine.quote(vts_id, timestamp_ns).price
    }

    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }
    pub fn zone_id(&self) -> u32 {
        self.zone_id
    }
}

#[derive(Debug, Clone)]
pub enum CommandResult {
    Passed,
    Adjusted { reason: String },
    Rejected { reason: String },
}

#[derive(Debug, Clone)]
pub enum SyncCheckResult {
    InSync,
    Warning,
    RolledBack { to_tick: u64 },
    RollbackFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sap_core::types::{Acceleration, Position, Velocity};

    fn create_test_command(robot_id: u64, vel_magnitude: f32) -> MotionCommand {
        MotionCommand {
            robot_id,
            current_position: Position::ORIGIN,
            target_velocity: Velocity::new(vel_magnitude, 0.0, 0.0),
            target_acceleration: Acceleration::new(1.0, 0.0, 0.0),
            ticket_id: 1,
        }
    }

    #[test]
    fn test_edge_runtime_new() {
        let runtime = EdgeRuntime::new(1);
        assert_eq!(runtime.zone_id(), 1);
        assert_eq!(runtime.current_tick(), 0);
    }

    #[test]
    fn test_tick_advances() {
        let mut runtime = EdgeRuntime::new(1);
        runtime.tick(1_000_000_000);
        assert_eq!(runtime.current_tick(), 1);
        runtime.tick(2_000_000_000);
        assert_eq!(runtime.current_tick(), 2);
    }

    #[test]
    fn test_process_command_passed() {
        let mut runtime = EdgeRuntime::new(1);
        let cmd = create_test_command(42, 2.0);
        let result = runtime.process_command(&cmd, 1_000_000_000);
        assert!(matches!(result, CommandResult::Passed));
        assert_eq!(runtime.stats().passed_commands, 1);
    }

    #[test]
    fn test_process_command_adjusted() {
        let mut runtime = EdgeRuntime::new(1);
        let cmd = create_test_command(42, 10.0);
        let result = runtime.process_command(&cmd, 1_000_000_000);
        assert!(matches!(result, CommandResult::Adjusted { .. }));
        assert_eq!(runtime.stats().adjusted_commands, 1);
    }

    #[test]
    fn test_sync_check_in_sync() {
        let mut runtime = EdgeRuntime::new(1);
        runtime.tick(1_000_000_000);
        let result = runtime.check_sync(42, 0.05, 1_000_000_000);
        assert!(matches!(result, SyncCheckResult::InSync));
    }

    #[test]
    fn test_sync_check_needs_rollback() {
        let mut runtime = EdgeRuntime::new(1);
        for i in 1..=10 {
            runtime.tick(i * 50_000_000);
        }
        let result = runtime.check_sync(42, 0.5, 500_000_000);
        assert!(matches!(result, SyncCheckResult::RolledBack { .. }));
        assert_eq!(runtime.stats().rollback_count, 1);
    }

    #[test]
    fn test_auction_flow() {
        let mut runtime = EdgeRuntime::new(1);
        runtime.submit_bid(1, 100, 500, 1_000_000_000).unwrap();
        runtime.submit_bid(2, 100, 800, 2_000_000_000).unwrap();
        runtime.submit_bid(3, 100, 600, 3_000_000_000).unwrap();
        let result = runtime.settle_auction(100, 5_000_000_000).unwrap();
        assert_eq!(result.winner_id, 2);
        assert_eq!(result.winning_price, 600);
        assert_eq!(runtime.stats().auction_count, 1);
        assert_eq!(runtime.stats().ticket_issued, 1);
    }

    #[test]
    fn test_price_quote() {
        let mut runtime = EdgeRuntime::new(1);
        let price1 = runtime.quote_price(100, 1_000_000_000);
        runtime.submit_bid(1, 100, 500, 1_000_000_000).unwrap();
        runtime.submit_bid(2, 100, 600, 2_000_000_000).unwrap();
        let price2 = runtime.quote_price(100, 3_000_000_000);
        assert!(price2 >= price1);
    }

    #[test]
    fn test_failsafe_healthy() {
        let mut runtime = EdgeRuntime::new(1);
        runtime.register_edge(1);
        runtime.receive_heartbeat(1, 1_000_000_000);
        let action = runtime.check_failsafe(1_050_000_000);
        assert!(matches!(action, FailsafeAction::None));
    }

    #[test]
    fn test_integrated_scenario() {
        let mut runtime = EdgeRuntime::new(1);
        runtime.register_edge(1);
        runtime.receive_heartbeat(1, 0);
        runtime.submit_bid(42, 100, 500, 100_000_000).unwrap();
        runtime.submit_bid(43, 100, 700, 200_000_000).unwrap();
        let auction_result = runtime.settle_auction(100, 300_000_000).unwrap();
        assert_eq!(auction_result.winner_id, 43);
        for i in 1..=10 {
            runtime.tick(i * 50_000_000);
            runtime.receive_heartbeat(1, i * 50_000_000);
            let cmd = create_test_command(43, 2.0);
            let result = runtime.process_command(&cmd, i * 50_000_000);
            assert!(matches!(result, CommandResult::Passed));
        }
        let sync_result = runtime.check_sync(43, 0.03, 500_000_000);
        assert!(matches!(sync_result, SyncCheckResult::InSync));
        let failsafe = runtime.check_failsafe(500_000_000);
        assert!(matches!(failsafe, FailsafeAction::None));
        assert_eq!(runtime.stats().total_commands, 10);
        assert_eq!(runtime.stats().passed_commands, 10);
        assert_eq!(runtime.stats().auction_count, 1);
    }
}
