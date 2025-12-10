//! # SAP Robot SDK
//!
//! SAP v2.0 Robot SDK - 로봇 클라이언트 라이브러리
//!
//! 이 크레이트는 SAP 프로토콜을 사용하는 로봇에 탑재되는 SDK입니다.
//! Edge 서버와 통신하여 공간 할당을 받고, 물리적 이동 명령을 생성합니다.
//!
//! ## 주요 기능
//!
//! - **상태 관리**: 로봇의 현재 위치, 속도, 티켓 상태 추적
//! - **명령 생성**: 이동 명령 생성 및 서명
//! - **티켓 요청**: Edge에 VTS 할당 요청
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use sap_robot::{CommandBuilder, RobotStateManager, TicketRequester};
//! use sap_core::types::{Position, Velocity};
//!
//! // 1. 로봇 상태 관리자 생성
//! let mut state_mgr = RobotStateManager::new(42); // robot_id = 42
//!
//! // 2. 현재 상태 업데이트
//! state_mgr.update_position(Position::new(5.0, 3.0, 0.0));
//! state_mgr.update_velocity(Velocity::new(0.5, 0.0, 0.0));
//!
//! // 3. 티켓 요청자 생성
//! let mut ticket_req = TicketRequester::new(42);
//!
//! // 4. VTS 할당 요청 (Edge에서 처리)
//! // let ticket = ticket_req.request_vts(zone_id, destination, deadline);
//!
//! // 5. 명령 빌더로 이동 명령 생성
//! let cmd = CommandBuilder::new(42)
//!     .with_velocity(Velocity::new(1.0, 0.0, 0.0))
//!     .with_ticket(12345u128)
//!     .build();
//! ```
//!
//! ## 모듈 구조
//!
//! | 모듈 | 설명 | 주요 타입 |
//! |------|------|----------|
//! | [`state`] | 로봇 상태 관리 | [`RobotStateManager`] |
//! | [`command`] | 명령 생성/서명 | [`CommandBuilder`] |
//! | [`ticket`] | 티켓 요청/검증 | [`TicketRequester`] |
//!
//! ## 아키텍처
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              Robot (sap-robot)          │
//! │  ┌─────────────┐    ┌────────────────┐  │
//! │  │ StateManager│───▶│ CommandBuilder │  │
//! │  └─────────────┘    └────────────────┘  │
//! │         │                   │           │
//! │         ▼                   ▼           │
//! │  ┌─────────────────────────────────┐    │
//! │  │        TicketRequester          │    │
//! │  └─────────────────────────────────┘    │
//! └────────────────────│────────────────────┘
//!                      ▼
//!              ┌───────────────┐
//!              │  Edge Server  │
//!              └───────────────┘
//! ```
//!
//! ## PPR 매핑
//!
//! - `AI_make_RobotState` → [`RobotStateManager`]
//! - `AI_make_MotionCommand` → [`CommandBuilder`]
//! - `AI_request_TransitTicket` → [`TicketRequester`]
//!
//! ## 관련 크레이트
//!
//! - [`sap_core`] - 핵심 타입 (Position, Velocity, RobotState)
//! - [`sap_physics`] - 물리 검증 (로봇에서 자체 검증 시)

pub mod command;
pub mod state;
pub mod ticket;

// 주요 타입 re-export
pub use command::CommandBuilder;
pub use state::RobotStateManager;
pub use ticket::TicketRequester;
