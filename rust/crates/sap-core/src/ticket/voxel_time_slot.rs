//! VoxelTimeSlot 타입
//!
//! PPR 매핑: AI_make_VoxelTimeSlot

use serde::{Deserialize, Serialize};

/// VoxelTimeSlot - 공간-시간 슬롯 (경제 단위)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct VoxelTimeSlot {
    /// 3D 공간 복셀 ID
    pub voxel_id: u64,

    /// 시작 시각 (PTP 나노초)
    pub t_start_ns: u64,

    /// 종료 시각 (PTP 나노초)
    pub t_end_ns: u64,
}

impl VoxelTimeSlot {
    /// 새 VoxelTimeSlot 생성
    pub const fn new(voxel_id: u64, t_start_ns: u64, t_end_ns: u64) -> Self {
        Self {
            voxel_id,
            t_start_ns,
            t_end_ns,
        }
    }

    /// 슬롯 지속 시간 (나노초)
    #[inline]
    pub fn duration_ns(&self) -> u64 {
        self.t_end_ns.saturating_sub(self.t_start_ns)
    }

    /// 슬롯 지속 시간 (밀리초)
    #[inline]
    pub fn duration_ms(&self) -> u64 {
        self.duration_ns() / 1_000_000
    }

    /// 특정 시각이 슬롯 내에 있는지 확인
    #[inline]
    pub fn contains_time(&self, timestamp_ns: u64) -> bool {
        timestamp_ns >= self.t_start_ns && timestamp_ns < self.t_end_ns
    }

    /// 두 슬롯이 시간적으로 겹치는지 확인
    #[inline]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.t_start_ns < other.t_end_ns && other.t_start_ns < self.t_end_ns
    }

    /// 동일 복셀인지 확인
    #[inline]
    pub fn same_voxel(&self, other: &Self) -> bool {
        self.voxel_id == other.voxel_id
    }

    /// 충돌 확인 (동일 복셀 + 시간 겹침)
    #[inline]
    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.same_voxel(other) && self.overlaps(other)
    }
}

/// VoxelTimeSlot 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelTimeSlotMeta {
    /// 기본 VTS
    pub vts: VoxelTimeSlot,

    /// 동시 허용 로봇 수
    pub max_robots: u16,

    /// 독점 여부 (true면 1대만 허용)
    pub exclusive: bool,

    /// 현재 예약 수
    pub reserved_count: u16,

    /// 기본 가격 (밀리 단위)
    pub base_price_milli: u64,
}

impl VoxelTimeSlotMeta {
    /// 새 메타데이터 생성
    pub fn new(vts: VoxelTimeSlot, max_robots: u16, exclusive: bool) -> Self {
        Self {
            vts,
            max_robots: if exclusive { 1 } else { max_robots },
            exclusive,
            reserved_count: 0,
            base_price_milli: 100_000, // 기본 100 단위
        }
    }

    /// 예약 가능 여부
    #[inline]
    pub fn can_reserve(&self) -> bool {
        self.reserved_count < self.max_robots
    }

    /// 예약 추가
    pub fn add_reservation(&mut self) -> bool {
        if self.can_reserve() {
            self.reserved_count += 1;
            true
        } else {
            false
        }
    }

    /// 예약 취소
    pub fn cancel_reservation(&mut self) {
        self.reserved_count = self.reserved_count.saturating_sub(1);
    }

    /// 혼잡도 (0.0 ~ 1.0)
    #[inline]
    pub fn congestion(&self) -> f32 {
        if self.max_robots == 0 {
            1.0
        } else {
            self.reserved_count as f32 / self.max_robots as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vts_new() {
        let vts = VoxelTimeSlot::new(42, 1000, 2000);
        assert_eq!(vts.voxel_id, 42);
        assert_eq!(vts.duration_ns(), 1000);
    }

    #[test]
    fn test_vts_contains_time() {
        let vts = VoxelTimeSlot::new(1, 1000, 2000);
        assert!(vts.contains_time(1500));
        assert!(!vts.contains_time(500));
        assert!(!vts.contains_time(2000)); // 종료 시각은 포함 안 함
    }

    #[test]
    fn test_vts_overlaps() {
        let vts1 = VoxelTimeSlot::new(1, 1000, 2000);
        let vts2 = VoxelTimeSlot::new(1, 1500, 2500);
        let vts3 = VoxelTimeSlot::new(1, 2000, 3000);

        assert!(vts1.overlaps(&vts2));
        assert!(!vts1.overlaps(&vts3)); // 경계는 겹치지 않음
    }

    #[test]
    fn test_vts_conflicts() {
        let vts1 = VoxelTimeSlot::new(1, 1000, 2000);
        let vts2 = VoxelTimeSlot::new(1, 1500, 2500); // 같은 복셀, 겹침
        let vts3 = VoxelTimeSlot::new(2, 1500, 2500); // 다른 복셀

        assert!(vts1.conflicts_with(&vts2));
        assert!(!vts1.conflicts_with(&vts3));
    }

    #[test]
    fn test_vts_meta_reservation() {
        let vts = VoxelTimeSlot::new(1, 0, 1000);
        let mut meta = VoxelTimeSlotMeta::new(vts, 3, false);

        assert!(meta.can_reserve());
        assert!(meta.add_reservation());
        assert!(meta.add_reservation());
        assert!(meta.add_reservation());
        assert!(!meta.can_reserve());
        assert!(!meta.add_reservation());

        assert!((meta.congestion() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vts_meta_exclusive() {
        let vts = VoxelTimeSlot::new(1, 0, 1000);
        let mut meta = VoxelTimeSlotMeta::new(vts, 10, true); // exclusive 무시

        assert_eq!(meta.max_robots, 1);
        assert!(meta.add_reservation());
        assert!(!meta.add_reservation());
    }
}
