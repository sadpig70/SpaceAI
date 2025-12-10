//! # SAP Economy
//!
//! SAP v1.2 L4 S-MEV - 공간 경제 엔진
//!
//! ## 모듈 구조
//!
//! - `auction`: Vickrey 경매 시스템
//! - `pricing`: 동적 가격 결정
//! - `ticket`: TransitTicket 관리
//!
//! ## PPR 매핑
//!
//! - `AI_make_VickreyAuction` → `VickreyAuction`
//! - `AI_make_PricingEngine` → `PricingEngine`
//! - `AI_make_TransitTicketManager` → `TicketManager`

pub mod auction;
pub mod pricing;
pub mod ticket;

// 주요 타입 re-export
pub use auction::{AuctionResult, VickreyAuction};
pub use pricing::{PriceQuote, PricingEngine};
pub use ticket::TicketManager;
