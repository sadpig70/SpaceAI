//! # SAP Cloud
//!
//! SAP v1.2 Cloud - VTS 할당 및 글로벌 상태 관리
//!
//! ## 모듈 구조
//!
//! - `vts`: VoxelTimeSlot 할당
//! - `state`: 글로벌 상태 집계
//! - `handoff`: Cross-Zone 핸드오프 프로토콜
//!
//! ## PPR 매핑
//!
//! - `AI_make_VtsAllocator` → `VtsAllocator`
//! - `AI_perceive_GlobalState` → `GlobalStateAggregator`
//! - `AI_make_CrossZoneHandoff` → `handoff`

pub mod handoff;
pub mod state;
pub mod vts;

// 주요 타입 re-export
pub use handoff::{
    HandoffRequest, HandoffResponse, HandoffStatus, PredictiveAllocation, ZoneBoundary,
};
pub use state::GlobalStateAggregator;
pub use vts::VtsAllocator;
