//! auction 모듈 - Vickrey 경매 시스템

mod vickrey;

pub use vickrey::{AuctionConfig, AuctionResult, BidEntry, VickreyAuction};
