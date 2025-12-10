//! PricingEngine - 동적 가격 결정 엔진
//!
//! PPR 매핑: AI_make_PricingEngine

use std::collections::HashMap;

/// 가격 결정 설정
#[derive(Debug, Clone)]
pub struct PricingConfig {
    /// 기본 가격
    pub base_price: u64,
    /// 최소 가격
    pub min_price: u64,
    /// 최대 가격
    pub max_price: u64,
    /// 수요 민감도 (0.0-1.0)
    pub demand_sensitivity: f32,
    /// 시간 민감도 (0.0-1.0)
    pub time_sensitivity: f32,
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            base_price: 100,
            min_price: 10,
            max_price: 10000,
            demand_sensitivity: 0.5,
            time_sensitivity: 0.3,
        }
    }
}

/// 가격 견적
#[derive(Debug, Clone)]
pub struct PriceQuote {
    /// VTS ID
    pub vts_id: u64,
    /// 견적 가격
    pub price: u64,
    /// 기본 가격
    pub base_price: u64,
    /// 수요 승수
    pub demand_multiplier: f32,
    /// 시간 승수
    pub time_multiplier: f32,
    /// 유효 기간 (나노초)
    pub valid_until_ns: u64,
}

/// 가격 결정 엔진 (PPR: AI_make_PricingEngine)
pub struct PricingEngine {
    config: PricingConfig,
    /// VTS별 수요 카운트
    demand_counts: HashMap<u64, u32>,
    /// VTS별 최근 거래 가격
    last_prices: HashMap<u64, u64>,
    /// 견적 생성 카운트
    quote_count: u64,
}

impl PricingEngine {
    pub fn new(config: PricingConfig) -> Self {
        Self {
            config,
            demand_counts: HashMap::new(),
            last_prices: HashMap::new(),
            quote_count: 0,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(PricingConfig::default())
    }

    /// 수요 증가 기록
    pub fn record_demand(&mut self, vts_id: u64) {
        *self.demand_counts.entry(vts_id).or_insert(0) += 1;
    }

    /// 거래 완료 기록
    pub fn record_transaction(&mut self, vts_id: u64, price: u64) {
        self.last_prices.insert(vts_id, price);
        // 수요 카운트 감소
        if let Some(count) = self.demand_counts.get_mut(&vts_id) {
            *count = count.saturating_sub(1);
        }
    }

    /// 가격 견적 생성 (PPR: AI_make_PricingEngine)
    pub fn quote(&mut self, vts_id: u64, current_time_ns: u64) -> PriceQuote {
        let demand = *self.demand_counts.get(&vts_id).unwrap_or(&0);
        let last_price = *self
            .last_prices
            .get(&vts_id)
            .unwrap_or(&self.config.base_price);

        // 수요 승수: 수요가 높을수록 가격 상승
        let demand_multiplier = 1.0 + (demand as f32 * 0.1 * self.config.demand_sensitivity);

        // 시간 승수 (간단한 시간 기반 조정)
        let time_multiplier = 1.0 + (self.config.time_sensitivity * 0.1);

        // 최종 가격 계산
        let computed_price = (last_price as f32 * demand_multiplier * time_multiplier) as u64;
        let price = computed_price.clamp(self.config.min_price, self.config.max_price);

        self.quote_count += 1;

        PriceQuote {
            vts_id,
            price,
            base_price: last_price,
            demand_multiplier,
            time_multiplier,
            valid_until_ns: current_time_ns + 60_000_000_000, // 60초 유효
        }
    }

    /// 현재 수요 조회
    pub fn get_demand(&self, vts_id: u64) -> u32 {
        *self.demand_counts.get(&vts_id).unwrap_or(&0)
    }

    /// 마지막 거래 가격 조회
    pub fn get_last_price(&self, vts_id: u64) -> Option<u64> {
        self.last_prices.get(&vts_id).copied()
    }

    /// 견적 생성 횟수
    pub fn quote_count(&self) -> u64 {
        self.quote_count
    }

    /// 수요 카운트 초기화
    pub fn reset_demand(&mut self, vts_id: u64) {
        self.demand_counts.remove(&vts_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_quote() {
        let mut engine = PricingEngine::with_default_config();

        let quote = engine.quote(100, 1_000_000_000);

        assert_eq!(quote.vts_id, 100);
        assert!(quote.price >= 10);
        assert!(quote.price <= 10000);
    }

    #[test]
    fn test_demand_increases_price() {
        let mut engine = PricingEngine::with_default_config();

        // 수요 없을 때
        let quote1 = engine.quote(100, 1_000_000_000);

        // 수요 추가
        engine.record_demand(100);
        engine.record_demand(100);
        engine.record_demand(100);

        let quote2 = engine.quote(100, 2_000_000_000);

        assert!(quote2.price >= quote1.price);
        assert!(quote2.demand_multiplier > 1.0);
    }

    #[test]
    fn test_transaction_record() {
        let mut engine = PricingEngine::with_default_config();

        engine.record_transaction(100, 500);

        assert_eq!(engine.get_last_price(100), Some(500));

        let quote = engine.quote(100, 1_000_000_000);
        assert_eq!(quote.base_price, 500);
    }

    #[test]
    fn test_price_bounds() {
        let config = PricingConfig {
            min_price: 50,
            max_price: 200,
            ..Default::default()
        };
        let mut engine = PricingEngine::new(config);

        // 낮은 가격 기록
        engine.record_transaction(100, 10);
        let quote1 = engine.quote(100, 1_000_000_000);
        assert!(quote1.price >= 50);

        // 높은 가격 기록
        engine.record_transaction(200, 10000);
        for _ in 0..100 {
            engine.record_demand(200);
        }
        let quote2 = engine.quote(200, 2_000_000_000);
        assert!(quote2.price <= 200);
    }

    #[test]
    fn test_quote_count() {
        let mut engine = PricingEngine::with_default_config();

        engine.quote(100, 1_000_000_000);
        engine.quote(200, 2_000_000_000);
        engine.quote(300, 3_000_000_000);

        assert_eq!(engine.quote_count(), 3);
    }
}
