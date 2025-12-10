//! StateComparator - 상태 비교 및 동기화
//!
//! PPR 매핑: AI_process_StateComparison

use sap_core::packet::DeltaTickPacket;

/// 상태 비교기
///
/// 예측 상태와 실제 상태를 비교하여 롤백 필요 여부 결정
pub struct StateComparator {
    /// 롤백 임계값 (미터)
    rollback_threshold: f32,

    /// 경고 임계값 (롤백 임계값의 일정 비율)
    warning_threshold: f32,

    /// 최근 비교 결과 기록
    history: Vec<ComparisonMetrics>,

    /// 히스토리 최대 크기
    history_capacity: usize,
}

/// 비교 메트릭스
#[derive(Debug, Clone)]
pub struct ComparisonMetrics {
    /// 로봇 ID
    pub robot_id: u64,

    /// 틱 번호
    pub tick: u64,

    /// 위치 델타 (미터)
    pub position_delta: f32,

    /// 방향 델타 (라디안)
    pub theta_delta: f32,

    /// 타임스탬프
    pub timestamp_ns: u64,
}

/// 동기화 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncResult {
    /// 동기화 상태 양호
    InSync,

    /// 경고 (드리프트 감지, 모니터링 필요)
    Warning,

    /// 롤백 필요
    NeedsRollback,
}

impl StateComparator {
    /// 새 StateComparator 생성
    pub fn new(rollback_threshold: f32) -> Self {
        Self {
            rollback_threshold,
            warning_threshold: rollback_threshold * 0.7,
            history: Vec::new(),
            history_capacity: 1000,
        }
    }

    /// 기본 설정으로 생성 (10cm 임계값)
    pub fn with_default_config() -> Self {
        Self::new(0.1)
    }

    /// 상태 비교 (PPR: AI_process_StateComparison)
    ///
    /// DeltaTickPacket의 delta 필드를 분석하여 동기화 상태 판단
    pub fn compare(&mut self, packet: &DeltaTickPacket) -> SyncResult {
        let delta_mag = packet.delta_magnitude();

        // 메트릭스 기록
        let metrics = ComparisonMetrics {
            robot_id: packet.robot_id,
            tick: packet.tick,
            position_delta: delta_mag,
            theta_delta: packet.delta_theta.abs(),
            timestamp_ns: packet.timestamp_ns,
        };

        self.record_metrics(metrics);

        // 결과 결정
        if delta_mag > self.rollback_threshold {
            SyncResult::NeedsRollback
        } else if delta_mag > self.warning_threshold {
            SyncResult::Warning
        } else {
            SyncResult::InSync
        }
    }

    /// 직접 델타 값으로 비교
    pub fn compare_delta(
        &mut self,
        robot_id: u64,
        tick: u64,
        position_delta: f32,
        theta_delta: f32,
    ) -> SyncResult {
        let metrics = ComparisonMetrics {
            robot_id,
            tick,
            position_delta,
            theta_delta,
            timestamp_ns: 0,
        };

        self.record_metrics(metrics);

        if position_delta > self.rollback_threshold {
            SyncResult::NeedsRollback
        } else if position_delta > self.warning_threshold {
            SyncResult::Warning
        } else {
            SyncResult::InSync
        }
    }

    /// 메트릭스 기록
    fn record_metrics(&mut self, metrics: ComparisonMetrics) {
        if self.history.len() >= self.history_capacity {
            self.history.remove(0);
        }
        self.history.push(metrics);
    }

    /// 특정 로봇의 최근 평균 델타
    pub fn average_delta(&self, robot_id: u64, count: usize) -> Option<f32> {
        let deltas: Vec<f32> = self
            .history
            .iter()
            .rev()
            .filter(|m| m.robot_id == robot_id)
            .take(count)
            .map(|m| m.position_delta)
            .collect();

        if deltas.is_empty() {
            None
        } else {
            Some(deltas.iter().sum::<f32>() / deltas.len() as f32)
        }
    }

    /// 롤백 발생 빈도 (최근 N개 중 롤백 필요 비율)
    pub fn rollback_frequency(&self, robot_id: u64, count: usize) -> f32 {
        let recent: Vec<&ComparisonMetrics> = self
            .history
            .iter()
            .rev()
            .filter(|m| m.robot_id == robot_id)
            .take(count)
            .collect();

        if recent.is_empty() {
            return 0.0;
        }

        let rollback_count = recent
            .iter()
            .filter(|m| m.position_delta > self.rollback_threshold)
            .count();

        rollback_count as f32 / recent.len() as f32
    }

    /// 임계값 조회
    pub fn rollback_threshold(&self) -> f32 {
        self.rollback_threshold
    }

    /// 히스토리 클리어
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sap_core::types::Position;

    #[test]
    fn test_comparator_in_sync() {
        let mut comparator = StateComparator::new(0.1);
        let result = comparator.compare_delta(1, 100, 0.05, 0.0);
        assert_eq!(result, SyncResult::InSync);
    }

    #[test]
    fn test_comparator_warning() {
        let mut comparator = StateComparator::new(0.1);
        // 0.07 < delta < 0.1 -> Warning (threshold * 0.7)
        let result = comparator.compare_delta(1, 100, 0.08, 0.0);
        assert_eq!(result, SyncResult::Warning);
    }

    #[test]
    fn test_comparator_needs_rollback() {
        let mut comparator = StateComparator::new(0.1);
        let result = comparator.compare_delta(1, 100, 0.15, 0.0);
        assert_eq!(result, SyncResult::NeedsRollback);
    }

    #[test]
    fn test_comparator_with_packet() {
        let mut comparator = StateComparator::new(0.1);

        let mut packet = DeltaTickPacket::new(1, 42, 100);
        packet = packet.with_delta(Position::new(0.2, 0.0, 0.0), 0.0);

        let result = comparator.compare(&packet);
        assert_eq!(result, SyncResult::NeedsRollback);
    }

    #[test]
    fn test_average_delta() {
        let mut comparator = StateComparator::new(0.1);

        comparator.compare_delta(1, 1, 0.05, 0.0);
        comparator.compare_delta(1, 2, 0.07, 0.0);
        comparator.compare_delta(1, 3, 0.09, 0.0);

        let avg = comparator.average_delta(1, 3).unwrap();
        assert!((avg - 0.07).abs() < 0.001);
    }

    #[test]
    fn test_rollback_frequency() {
        let mut comparator = StateComparator::new(0.1);

        // 4개 중 2개가 rollback 필요
        comparator.compare_delta(1, 1, 0.05, 0.0); // InSync
        comparator.compare_delta(1, 2, 0.15, 0.0); // Rollback
        comparator.compare_delta(1, 3, 0.08, 0.0); // Warning
        comparator.compare_delta(1, 4, 0.20, 0.0); // Rollback

        let freq = comparator.rollback_frequency(1, 4);
        assert!((freq - 0.5).abs() < 0.01);
    }
}
