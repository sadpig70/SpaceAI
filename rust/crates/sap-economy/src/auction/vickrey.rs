//! VickreyAuction - 제2가 밀봉 경매
//!
//! PPR 매핑: AI_make_VickreyAuction

use sap_core::ticket::Bid;
use std::collections::HashMap;

/// 경매 설정
#[derive(Debug, Clone)]
pub struct AuctionConfig {
    /// 최소 입찰 금액 (milli 단위) - 이보다 낮은 입찰은 거부
    pub min_bid: u64,
    /// 예약 가격 (milli 단위) - 단일 입찰 시 낙찰 가격의 하한
    pub reserve_price: u64,
    /// 경매 마감 시간 (나노초, 0이면 무제한)
    pub deadline_ns: u64,
    /// VTS당 최대 입찰 수
    pub max_bids: usize,
}

impl Default for AuctionConfig {
    fn default() -> Self {
        Self {
            min_bid: 100,      // 최소 입찰: 100 milli
            reserve_price: 50, // 예약가: 50 milli (단일 입찰 시 하한)
            deadline_ns: 0,
            max_bids: 1000,
        }
    }
}

/// 입찰 항목
#[derive(Debug, Clone)]
pub struct BidEntry {
    pub robot_id: u64,
    pub bid_amount: u64,
    pub timestamp_ns: u64,
    pub vts_id: u64,
}

/// 경매 결과
#[derive(Debug, Clone)]
pub struct AuctionResult {
    pub winner_id: u64,
    pub winning_price: u64,
    pub original_bid: u64,
    pub vts_id: u64,
    pub completed_ns: u64,
}

/// Vickrey 경매
pub struct VickreyAuction {
    config: AuctionConfig,
    bids: HashMap<u64, Vec<BidEntry>>,
    results: Vec<AuctionResult>,
}

impl VickreyAuction {
    pub fn new(config: AuctionConfig) -> Self {
        Self {
            config,
            bids: HashMap::new(),
            results: Vec::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(AuctionConfig::default())
    }

    pub fn submit_bid(&mut self, bid: BidEntry) -> Result<(), AuctionError> {
        if bid.bid_amount < self.config.min_bid {
            return Err(AuctionError::BidTooLow {
                min: self.config.min_bid,
                actual: bid.bid_amount,
            });
        }

        let entries = self.bids.entry(bid.vts_id).or_default();

        if entries.len() >= self.config.max_bids {
            return Err(AuctionError::TooManyBids);
        }

        if entries.iter().any(|e| e.robot_id == bid.robot_id) {
            return Err(AuctionError::DuplicateBid);
        }

        entries.push(bid);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn submit_from_bid(&mut self, bid: &Bid, vts_id: u64) -> Result<(), AuctionError> {
        let entry = BidEntry {
            robot_id: bid.robot_id,
            bid_amount: bid.amount_milli,
            timestamp_ns: bid.timestamp_ns,
            vts_id,
        };
        self.submit_bid(entry)
    }

    pub fn settle(&mut self, vts_id: u64, current_time_ns: u64) -> Option<AuctionResult> {
        let entries = self.bids.get_mut(&vts_id)?;

        if entries.is_empty() {
            return None;
        }

        entries.sort_by(|a, b| b.bid_amount.cmp(&a.bid_amount));

        let winner = &entries[0];

        let second_price = if entries.len() > 1 {
            entries[1].bid_amount
        } else {
            // 단일 입찰 시 예약 가격 사용 (min_bid보다 낮을 수 있음)
            self.config.reserve_price
        };

        let result = AuctionResult {
            winner_id: winner.robot_id,
            winning_price: second_price,
            original_bid: winner.bid_amount,
            vts_id,
            completed_ns: current_time_ns,
        };

        self.results.push(result.clone());
        self.bids.remove(&vts_id);

        Some(result)
    }

    #[allow(dead_code)]
    pub fn settle_all(&mut self, current_time_ns: u64) -> Vec<AuctionResult> {
        let vts_ids: Vec<u64> = self.bids.keys().copied().collect();
        vts_ids
            .into_iter()
            .filter_map(|vts_id| self.settle(vts_id, current_time_ns))
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_bids(&self, vts_id: u64) -> Option<&Vec<BidEntry>> {
        self.bids.get(&vts_id)
    }

    pub fn total_bid_count(&self) -> usize {
        self.bids.values().map(|v| v.len()).sum()
    }

    #[allow(dead_code)]
    pub fn recent_results(&self, count: usize) -> &[AuctionResult] {
        let start = self.results.len().saturating_sub(count);
        &self.results[start..]
    }
}

/// 경매 오류
#[derive(Debug, Clone)]
pub enum AuctionError {
    BidTooLow {
        min: u64,
        actual: u64,
    },
    TooManyBids,
    DuplicateBid,
    #[allow(dead_code)]
    AuctionClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_bid() {
        let mut auction = VickreyAuction::with_default_config();
        let bid = BidEntry {
            robot_id: 1,
            bid_amount: 500,
            timestamp_ns: 1000,
            vts_id: 100,
        };
        assert!(auction.submit_bid(bid).is_ok());
        assert_eq!(auction.total_bid_count(), 1);
    }

    #[test]
    fn test_bid_too_low() {
        let mut auction = VickreyAuction::with_default_config();
        let bid = BidEntry {
            robot_id: 1,
            bid_amount: 50,
            timestamp_ns: 1000,
            vts_id: 100,
        };
        assert!(matches!(
            auction.submit_bid(bid),
            Err(AuctionError::BidTooLow { .. })
        ));
    }

    #[test]
    fn test_duplicate_bid() {
        let mut auction = VickreyAuction::with_default_config();
        let bid1 = BidEntry {
            robot_id: 1,
            bid_amount: 500,
            timestamp_ns: 1000,
            vts_id: 100,
        };
        let bid2 = BidEntry {
            robot_id: 1,
            bid_amount: 600,
            timestamp_ns: 2000,
            vts_id: 100,
        };
        auction.submit_bid(bid1).unwrap();
        assert!(matches!(
            auction.submit_bid(bid2),
            Err(AuctionError::DuplicateBid)
        ));
    }

    #[test]
    fn test_vickrey_settle() {
        let mut auction = VickreyAuction::with_default_config();
        auction
            .submit_bid(BidEntry {
                robot_id: 1,
                bid_amount: 500,
                timestamp_ns: 1000,
                vts_id: 100,
            })
            .unwrap();
        auction
            .submit_bid(BidEntry {
                robot_id: 2,
                bid_amount: 800,
                timestamp_ns: 2000,
                vts_id: 100,
            })
            .unwrap();
        auction
            .submit_bid(BidEntry {
                robot_id: 3,
                bid_amount: 600,
                timestamp_ns: 3000,
                vts_id: 100,
            })
            .unwrap();
        let result = auction.settle(100, 5000).unwrap();
        assert_eq!(result.winner_id, 2);
        assert_eq!(result.winning_price, 600);
        assert_eq!(result.original_bid, 800);
    }

    #[test]
    fn test_single_bid_settle() {
        let mut auction = VickreyAuction::with_default_config();
        auction
            .submit_bid(BidEntry {
                robot_id: 1,
                bid_amount: 500,
                timestamp_ns: 1000,
                vts_id: 100,
            })
            .unwrap();
        let result = auction.settle(100, 5000).unwrap();
        assert_eq!(result.winner_id, 1);
        // 단일 입찰 시 reserve_price(50)가 적용됨
        assert_eq!(result.winning_price, 50);
    }

    #[test]
    fn test_settle_all() {
        let mut auction = VickreyAuction::with_default_config();
        auction
            .submit_bid(BidEntry {
                robot_id: 1,
                bid_amount: 500,
                timestamp_ns: 1000,
                vts_id: 100,
            })
            .unwrap();
        auction
            .submit_bid(BidEntry {
                robot_id: 2,
                bid_amount: 800,
                timestamp_ns: 2000,
                vts_id: 100,
            })
            .unwrap();
        auction
            .submit_bid(BidEntry {
                robot_id: 3,
                bid_amount: 300,
                timestamp_ns: 1000,
                vts_id: 200,
            })
            .unwrap();
        let results = auction.settle_all(5000);
        assert_eq!(results.len(), 2);
    }
}
