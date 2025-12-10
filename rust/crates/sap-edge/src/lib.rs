//! # SAP Edge
//!
//! SAP v1.2 Edge Node Runtime - L2+L3+L4 통합 런타임
//!
//! ## 아키텍처
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           SAP Edge Runtime                   │
//! ├─────────────────────────────────────────────┤
//! │  L4 S-MEV    │ VickreyAuction, PricingEngine │
//! ├──────────────┼──────────────────────────────┤
//! │  L3 Sync     │ RollbackManager, Failsafe    │
//! ├──────────────┼──────────────────────────────┤
//! │  L2 TrustOS  │ PhysicsValidator, CommandGate│
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## 주요 컴포넌트
//!
//! - `EdgeRuntime`: 통합 런타임 (통합 테스트용)

pub mod runtime;

pub use runtime::EdgeRuntime;

// 핵심 크레이트 re-export
pub use sap_core as core;
pub use sap_economy as economy;
pub use sap_network as network;
pub use sap_physics as physics;
