//! 시간 유틸리티

#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

/// 현재 시간 (나노초, Unix epoch 기준)
#[cfg(feature = "std")]
pub fn current_time_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

/// no_std 환경에서는 0 반환 (외부에서 주입 필요)
#[cfg(not(feature = "std"))]
pub fn current_time_ns() -> u64 {
    0
}

/// 나노초 → 밀리초 변환
#[inline]
pub const fn ns_to_ms(ns: u64) -> u64 {
    ns / 1_000_000
}

/// 밀리초 → 나노초 변환
#[inline]
pub const fn ms_to_ns(ms: u64) -> u64 {
    ms * 1_000_000
}

/// 나노초 → 초 변환 (f64)
#[inline]
#[allow(dead_code)]
pub fn ns_to_secs(ns: u64) -> f64 {
    ns as f64 / 1_000_000_000.0
}

/// 초 → 나노초 변환
#[inline]
#[allow(dead_code)]
pub fn secs_to_ns(secs: f64) -> u64 {
    (secs * 1_000_000_000.0) as u64
}

/// 두 타임스탬프 간 경과 시간 (밀리초)
#[inline]
#[allow(dead_code)]
pub fn elapsed_ms(start_ns: u64, end_ns: u64) -> u64 {
    ns_to_ms(end_ns.saturating_sub(start_ns))
}

/// 틱 번호 → 타임스탬프 변환 (50ms 틱 기준)
#[inline]
#[allow(dead_code)]
pub const fn tick_to_ns(tick: u64, tick_interval_ms: u64) -> u64 {
    tick * tick_interval_ms * 1_000_000
}

/// 타임스탬프 → 틱 번호 변환 (50ms 틱 기준)
#[inline]
#[allow(dead_code)]
pub fn ns_to_tick(timestamp_ns: u64, tick_interval_ms: u64) -> u64 {
    if tick_interval_ms == 0 {
        return 0;
    }
    timestamp_ns / (tick_interval_ms * 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns_to_ms() {
        assert_eq!(ns_to_ms(1_000_000), 1);
        assert_eq!(ns_to_ms(50_000_000), 50);
        assert_eq!(ns_to_ms(1_000_000_000), 1000);
    }

    #[test]
    fn test_ms_to_ns() {
        assert_eq!(ms_to_ns(1), 1_000_000);
        assert_eq!(ms_to_ns(50), 50_000_000);
    }

    #[test]
    fn test_ns_to_secs() {
        let ns = 1_500_000_000; // 1.5초
        assert!((ns_to_secs(ns) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_elapsed_ms() {
        let start = 1_000_000_000; // 1s
        let end = 1_500_000_000; // 1.5s
        assert_eq!(elapsed_ms(start, end), 500);
    }

    #[test]
    fn test_tick_conversion() {
        let tick = 100;
        let tick_interval = 50; // 50ms

        let ns = tick_to_ns(tick, tick_interval);
        assert_eq!(ns, 5_000_000_000); // 100 * 50ms = 5s

        let back = ns_to_tick(ns, tick_interval);
        assert_eq!(back, tick);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_current_time_ns() {
        let time1 = current_time_ns();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let time2 = current_time_ns();
        assert!(time2 > time1);
    }
}
