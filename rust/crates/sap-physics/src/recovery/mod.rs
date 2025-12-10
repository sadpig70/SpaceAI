//! 물리적 복구 모듈
//!
//! 롤백 발생 시 로봇의 **물리적 복구 동작**을 정의합니다.
//! 논리적 롤백(RollbackManager)과 달리, 이 모듈은 실제 로봇 하드웨어에
//! 전달할 안전 명령을 생성합니다.
//!
//! # 복구 계층
//!
//! | 수준 | 이름 | 설명 |
//! |------|------|------|
//! | L0 | EmergencyStop | 즉시 정지 (최고 감속) |
//! | L1 | SafeDeceleration | 물리 제약 내 부드러운 감속 |
//! | L2 | SafeHold | 현재 위치 유지 |
//! | L3 | PathReplanning | 새 경로로 재계획 |
//!
//! PPR 매핑: AI_response_PhysicalRecovery

mod command;

pub use command::{RecoveryCommand, RecoveryLevel, RecoveryResult};
