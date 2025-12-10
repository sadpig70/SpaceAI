//! # SAP Physvisor
//!
//! SAP v2.0 Physvisor - Zone 관리 및 다중 로봇 시뮬레이션 (L3 MidTier)
//!
//! Physvisor는 여러 Zone을 관리하고, 물리 시뮬레이션을 통해 로봇 간
//! 충돌 가능성을 예측하는 중간 계층 서비스입니다.
//!
//! ## 주요 기능
//!
//! - **Zone 관리**: 다중 Zone의 상태와 용량 관리
//! - **로봇 레지스트리**: Zone 내 활성 로봇 추적
//! - **시뮬레이션**: 미래 충돌 예측 및 경로 검증
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use sap_physvisor::{ZoneManager, RobotRegistry, SimulationEngine};
//! use sap_core::types::Position;
//!
//! // 1. Zone 관리자 생성
//! let mut zone_mgr = ZoneManager::new(1); // zone_id = 1
//!
//! // 2. 로봇 레지스트리 생성
//! let mut registry = RobotRegistry::new();
//! registry.register(42, zone_id);
//!
//! // 3. 시뮬레이션 엔진 생성
//! let mut sim = SimulationEngine::new();
//! sim.set_zone(zone_id);
//!
//! // 4. 시뮬레이션 실행
//! let result = sim.simulate_tick();
//! if result.has_collision() {
//!     // 충돌 경고 처리
//! }
//! ```
//!
//! ## 모듈 구조
//!
//! | 모듈 | 설명 | 주요 타입 |
//! |------|------|----------|
//! | [`zone`] | Zone 관리 | [`ZoneManager`] |
//! | [`registry`] | 로봇 레지스트리 | [`RobotRegistry`] |
//! | [`simulation`] | 시뮬레이션 엔진 | [`SimulationEngine`] |
//!
//! ## 시스템 위치
//!
//! ```text
//! ┌─────────────┐
//! │   Cloud     │  L5
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │  Physvisor  │  L3 ◀── 이 크레이트
//! │  - Zone Mgr │
//! │  - Registry │
//! │  - SimEngine│
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │    Edge     │  L4
//! └──────┬──────┘
//!        │
//! ┌──────▼──────┐
//! │   Robot     │  L0
//! └─────────────┘
//! ```
//!
//! ## PPR 매핑
//!
//! - `AI_make_ZoneManager` → [`ZoneManager`]
//! - `AI_perceive_RobotRegistry` → [`RobotRegistry`]
//! - `AI_process_Simulation` → [`SimulationEngine`]
//!
//! ## 관련 크레이트
//!
//! - [`sap_core`] - 핵심 타입
//! - [`sap_physics`] - 물리 검증
//! - [`sap_edge`] - Edge 런타임

pub mod registry;
pub mod simulation;
pub mod zone;

// 주요 타입 re-export
pub use registry::RobotRegistry;
pub use simulation::SimulationEngine;
pub use zone::ZoneManager;
