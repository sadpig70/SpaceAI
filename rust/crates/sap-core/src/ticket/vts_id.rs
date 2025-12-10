//! VtsId 타입 정의
//!
//! VoxelTimeSlot의 고유 식별자를 위한 해시 기반 ID 시스템.
//! zone_id, voxel_id, t_start_ns, t_end_ns를 조합하여 u128 해시 생성.
//!
//! PPR 매핑: AI_make_VtsId

use super::VoxelTimeSlot;
use serde::{Deserialize, Serialize};

/// VtsId - VoxelTimeSlot 고유 식별자 (128-bit)
///
/// 시공간 슬롯의 전역적으로 유일한 ID를 제공합니다.
/// zone_id + voxel_id + t_start_ns + t_end_ns를 해시하여 생성.
///
/// # Example
/// ```
/// use sap_core::ticket::{VtsId, VoxelTimeSlot};
///
/// let vts = VoxelTimeSlot::new(42, 1000, 2000);
/// let vts_id = VtsId::from_vts(1, &vts); // zone_id = 1
/// assert!(vts_id.value() != 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct VtsId(u128);

impl VtsId {
    /// 빈 VtsId (무효)
    pub const INVALID: Self = Self(0);

    /// zone_id와 VoxelTimeSlot으로부터 VtsId 생성
    ///
    /// FNV-1a 해시 알고리즘 변형을 사용하여 128-bit ID 생성.
    /// 충돌 확률이 매우 낮으며 (2^-64), 결정론적임.
    #[inline]
    pub fn from_vts(zone_id: u32, vts: &VoxelTimeSlot) -> Self {
        Self::from_components(zone_id, vts.voxel_id, vts.t_start_ns, vts.t_end_ns)
    }

    /// 개별 컴포넌트로부터 VtsId 생성
    ///
    /// # Arguments
    /// * `zone_id` - Zone 식별자
    /// * `voxel_id` - 3D 공간 복셀 ID
    /// * `t_start_ns` - 시작 시각 (나노초)
    /// * `t_end_ns` - 종료 시각 (나노초)
    pub fn from_components(zone_id: u32, voxel_id: u64, t_start_ns: u64, t_end_ns: u64) -> Self {
        // FNV-1a 128-bit 해시 상수
        const FNV_OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
        const FNV_PRIME: u128 = 0x0000000001000000000000000000013b;

        let mut hash = FNV_OFFSET;

        // zone_id 해시
        for byte in zone_id.to_le_bytes() {
            hash ^= byte as u128;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        // voxel_id 해시
        for byte in voxel_id.to_le_bytes() {
            hash ^= byte as u128;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        // t_start_ns 해시
        for byte in t_start_ns.to_le_bytes() {
            hash ^= byte as u128;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        // t_end_ns 해시
        for byte in t_end_ns.to_le_bytes() {
            hash ^= byte as u128;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        Self(hash)
    }

    /// 원시 u128 값으로부터 VtsId 생성 (역직렬화용)
    #[inline]
    pub const fn from_raw(value: u128) -> Self {
        Self(value)
    }

    /// 내부 u128 값 반환
    #[inline]
    pub const fn value(self) -> u128 {
        self.0
    }

    /// 유효한 ID인지 확인
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }

    /// 상위 64비트 반환 (단축 ID용)
    #[inline]
    pub const fn high(self) -> u64 {
        (self.0 >> 64) as u64
    }

    /// 하위 64비트 반환 (단축 ID용)
    #[inline]
    pub const fn low(self) -> u64 {
        self.0 as u64
    }
}

impl Default for VtsId {
    fn default() -> Self {
        Self::INVALID
    }
}

impl From<u128> for VtsId {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<VtsId> for u128 {
    fn from(id: VtsId) -> Self {
        id.0
    }
}

impl std::fmt::Display for VtsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VtsId({:032x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vts_id_from_components() {
        let id1 = VtsId::from_components(1, 100, 1000, 2000);
        let id2 = VtsId::from_components(1, 100, 1000, 2000);
        let id3 = VtsId::from_components(1, 100, 1000, 2001); // 다른 t_end

        // 동일 입력 → 동일 ID (결정론적)
        assert_eq!(id1, id2);

        // 다른 입력 → 다른 ID
        assert_ne!(id1, id3);

        // 유효성 검사
        assert!(id1.is_valid());
    }

    #[test]
    fn test_vts_id_from_vts() {
        let vts = VoxelTimeSlot::new(42, 1000, 2000);
        let id = VtsId::from_vts(1, &vts);

        assert!(id.is_valid());
        assert!(id.value() != 0);
    }

    #[test]
    fn test_vts_id_invalid() {
        assert!(!VtsId::INVALID.is_valid());
        assert_eq!(VtsId::INVALID.value(), 0);
    }

    #[test]
    fn test_vts_id_high_low() {
        let id = VtsId::from_components(1, 100, 1000, 2000);

        // high/low 분해 후 재조합
        let reconstructed = ((id.high() as u128) << 64) | (id.low() as u128);
        assert_eq!(id.value(), reconstructed);
    }

    #[test]
    fn test_vts_id_display() {
        let id = VtsId::from_raw(0x123456789abcdef0_fedcba9876543210);
        let display = format!("{}", id);
        assert!(display.contains("VtsId("));
    }

    #[test]
    fn test_vts_id_zone_matters() {
        let vts = VoxelTimeSlot::new(100, 1000, 2000);

        let id_zone1 = VtsId::from_vts(1, &vts);
        let id_zone2 = VtsId::from_vts(2, &vts);

        // 다른 zone → 다른 ID
        assert_ne!(id_zone1, id_zone2);
    }

    #[test]
    fn test_vts_id_collision_resistance() {
        // 다양한 입력으로 ID 생성, 충돌 검사
        let mut ids = std::collections::HashSet::new();

        for zone in 0..10 {
            for voxel in 0..100 {
                for t_start in (0..1000).step_by(100) {
                    let id = VtsId::from_components(zone, voxel, t_start, t_start + 100);
                    assert!(
                        ids.insert(id),
                        "Collision detected at zone={}, voxel={}, t_start={}",
                        zone,
                        voxel,
                        t_start
                    );
                }
            }
        }
    }
}
