//! 3D 위치 타입
//!
//! PPR 매핑: AI_perceive_* 함수들의 위치 데이터

use serde::{Deserialize, Serialize};

/// 3D 공간 위치 (미터 단위)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[repr(C)]
pub struct Position {
    /// X 좌표 (미터)
    pub x: f32,
    /// Y 좌표 (미터)
    pub y: f32,
    /// Z 좌표 (미터)
    pub z: f32,
}

impl Position {
    /// 새 Position 생성
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 원점 (0, 0, 0)
    pub const ORIGIN: Self = Self::new(0.0, 0.0, 0.0);

    /// 두 위치 사이의 유클리드 거리
    #[inline]
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D 거리 (XY 평면)
    #[inline]
    pub fn distance_2d(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// 원점으로부터의 거리 (magnitude)
    #[inline]
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// 두 위치의 차이 (delta)
    #[inline]
    pub fn delta(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// 스칼라 곱
    #[inline]
    pub fn scale(&self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(1.0, 2.0, 3.0);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
    }

    #[test]
    fn test_position_distance() {
        let a = Position::new(0.0, 0.0, 0.0);
        let b = Position::new(3.0, 4.0, 0.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_position_magnitude() {
        let pos = Position::new(3.0, 4.0, 0.0);
        assert!((pos.magnitude() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_position_add() {
        let a = Position::new(1.0, 2.0, 3.0);
        let b = Position::new(4.0, 5.0, 6.0);
        let c = a + b;
        assert_eq!(c, Position::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_position_serialization() {
        let pos = Position::new(1.5, 2.5, 3.5);
        let encoded = bincode::serialize(&pos).unwrap();
        let decoded: Position = bincode::deserialize(&encoded).unwrap();
        assert_eq!(pos, decoded);
    }
}
