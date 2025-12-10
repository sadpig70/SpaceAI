//! CommandGate - 명령 필터링 게이트
//!
//! PPR 매핑: AI_make_CommandGate, AI_process_eBPFFilter

use super::MotionCommand;
use sap_core::validation::ValidationResult;

/// CommandGate - 명령 필터링 게이트
///
/// 정책 기반 명령 필터링 및 조정
pub struct CommandGate {
    /// 활성화된 정책 목록
    policies: Vec<Box<dyn CommandPolicy>>,

    /// 거부된 명령 카운터
    rejected_count: u64,

    /// 조정된 명령 카운터
    adjusted_count: u64,
}

/// 명령 정책 트레이트
pub trait CommandPolicy: Send + Sync {
    /// 정책 이름
    fn name(&self) -> &str;

    /// 명령 검사
    fn check(&self, cmd: &MotionCommand) -> PolicyResult;

    /// 명령 조정 (가능하면)
    fn adjust(&self, cmd: &MotionCommand) -> Option<MotionCommand>;
}

/// 정책 검사 결과
#[derive(Debug, Clone)]
pub enum PolicyResult {
    /// 통과
    Pass,

    /// 조정 필요
    Adjust(String),

    /// 거부
    Reject(String),
}

impl CommandGate {
    /// 새 CommandGate 생성
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            rejected_count: 0,
            adjusted_count: 0,
        }
    }

    /// 정책 추가
    pub fn add_policy(&mut self, policy: Box<dyn CommandPolicy>) {
        self.policies.push(policy);
    }

    /// 명령 필터링 (PPR: AI_make_CommandGate)
    ///
    /// 모든 정책을 순차적으로 적용
    pub fn filter(&mut self, cmd: &MotionCommand) -> GateResult {
        for policy in &self.policies {
            match policy.check(cmd) {
                PolicyResult::Pass => continue,
                PolicyResult::Adjust(reason) => {
                    self.adjusted_count += 1;
                    if let Some(adjusted_cmd) = policy.adjust(cmd) {
                        return GateResult::Adjusted {
                            original: cmd.clone(),
                            adjusted: adjusted_cmd,
                            reason,
                        };
                    }
                }
                PolicyResult::Reject(reason) => {
                    self.rejected_count += 1;
                    return GateResult::Rejected { reason };
                }
            }
        }

        GateResult::Passed
    }

    /// 통계 조회
    pub fn stats(&self) -> GateStats {
        GateStats {
            rejected_count: self.rejected_count,
            adjusted_count: self.adjusted_count,
            policy_count: self.policies.len(),
        }
    }

    /// 통계 리셋
    #[allow(dead_code)]
    pub fn reset_stats(&mut self) {
        self.rejected_count = 0;
        self.adjusted_count = 0;
    }
}

impl Default for CommandGate {
    fn default() -> Self {
        Self::new()
    }
}

/// 게이트 필터링 결과
#[derive(Debug, Clone)]
pub enum GateResult {
    /// 통과
    Passed,

    /// 조정됨
    Adjusted {
        original: MotionCommand,
        adjusted: MotionCommand,
        reason: String,
    },

    /// 거부됨
    Rejected { reason: String },
}

impl GateResult {
    /// ValidationResult로 변환
    #[allow(dead_code)]
    pub fn to_validation_result(&self) -> ValidationResult {
        match self {
            GateResult::Passed => ValidationResult::OK,
            GateResult::Adjusted { .. } => ValidationResult::ADJUST,
            GateResult::Rejected { .. } => ValidationResult::REJECT,
        }
    }

    /// 통과 여부
    pub fn is_passed(&self) -> bool {
        matches!(self, GateResult::Passed)
    }
}

/// 게이트 통계
#[derive(Debug, Clone)]
pub struct GateStats {
    pub rejected_count: u64,
    pub adjusted_count: u64,
    pub policy_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sap_core::types::Velocity;

    // ===== 테스트용 정책 구현 =====

    /// 속도 제한 정책
    struct VelocityLimitPolicy {
        max_velocity: f32,
    }

    impl VelocityLimitPolicy {
        fn new(max_velocity: f32) -> Self {
            Self { max_velocity }
        }
    }

    impl CommandPolicy for VelocityLimitPolicy {
        fn name(&self) -> &str {
            "VelocityLimitPolicy"
        }

        fn check(&self, cmd: &MotionCommand) -> PolicyResult {
            let speed = cmd.target_speed();
            if speed <= self.max_velocity {
                PolicyResult::Pass
            } else {
                PolicyResult::Adjust(format!(
                    "Velocity {} exceeds limit {}",
                    speed, self.max_velocity
                ))
            }
        }

        fn adjust(&self, cmd: &MotionCommand) -> Option<MotionCommand> {
            let clamped = cmd.target_velocity.clamp(self.max_velocity);
            Some(MotionCommand {
                target_velocity: clamped,
                ..cmd.clone()
            })
        }
    }

    /// 티켓 필수 정책
    struct TicketRequiredPolicy;

    impl CommandPolicy for TicketRequiredPolicy {
        fn name(&self) -> &str {
            "TicketRequiredPolicy"
        }

        fn check(&self, cmd: &MotionCommand) -> PolicyResult {
            if cmd.ticket_id > 0 {
                PolicyResult::Pass
            } else {
                PolicyResult::Reject("No valid ticket".to_string())
            }
        }

        fn adjust(&self, _cmd: &MotionCommand) -> Option<MotionCommand> {
            None // 조정 불가
        }
    }

    #[test]
    fn test_gate_pass() {
        let mut gate = CommandGate::new();
        gate.add_policy(Box::new(VelocityLimitPolicy::new(5.0)));

        let cmd = MotionCommand::new(1)
            .with_velocity(Velocity::new(3.0, 0.0, 0.0))
            .with_ticket(1);

        let result = gate.filter(&cmd);
        assert!(result.is_passed());
    }

    #[test]
    fn test_gate_adjust() {
        let mut gate = CommandGate::new();
        gate.add_policy(Box::new(VelocityLimitPolicy::new(5.0)));

        let cmd = MotionCommand::new(1)
            .with_velocity(Velocity::new(10.0, 0.0, 0.0))
            .with_ticket(1);

        let result = gate.filter(&cmd);

        match result {
            GateResult::Adjusted { adjusted, .. } => {
                assert!((adjusted.target_speed() - 5.0).abs() < 0.01);
            }
            _ => panic!("Expected Adjusted"),
        }
    }

    #[test]
    fn test_gate_reject() {
        let mut gate = CommandGate::new();
        gate.add_policy(Box::new(TicketRequiredPolicy));

        let cmd = MotionCommand::new(1); // No ticket

        let result = gate.filter(&cmd);

        match result {
            GateResult::Rejected { reason } => {
                assert!(reason.contains("ticket"));
            }
            _ => panic!("Expected Rejected"),
        }
    }

    #[test]
    fn test_gate_stats() {
        let mut gate = CommandGate::new();
        gate.add_policy(Box::new(VelocityLimitPolicy::new(5.0)));
        gate.add_policy(Box::new(TicketRequiredPolicy));

        let cmd1 = MotionCommand::new(1)
            .with_velocity(Velocity::new(10.0, 0.0, 0.0))
            .with_ticket(1);
        gate.filter(&cmd1);

        let cmd2 = MotionCommand::new(2); // No ticket
        gate.filter(&cmd2);

        let stats = gate.stats();
        assert_eq!(stats.policy_count, 2);
        assert!(stats.adjusted_count > 0 || stats.rejected_count > 0);
    }
}
