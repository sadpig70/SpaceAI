//! 해시 유틸리티

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 32바이트 해시 계산 (간소화 버전)
pub fn compute_hash<T: Hash>(data: &T) -> [u8; 32] {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();

    let mut result = [0u8; 32];
    result[0..8].copy_from_slice(&hash.to_le_bytes());
    result
}

/// 바이트 데이터 해시
pub fn compute_hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();

    let mut result = [0u8; 32];
    result[0..8].copy_from_slice(&hash.to_le_bytes());
    result
}

/// Merkle Root 계산 (간소화 버전)
pub fn compute_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }

    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut level: Vec<[u8; 32]> = leaves.to_vec();

    while level.len() > 1 {
        let mut next_level = Vec::new();

        for chunk in level.chunks(2) {
            let mut combined = [0u8; 64];
            combined[0..32].copy_from_slice(&chunk[0]);

            if chunk.len() > 1 {
                combined[32..64].copy_from_slice(&chunk[1]);
            } else {
                combined[32..64].copy_from_slice(&chunk[0]); // 홀수면 복제
            }

            next_level.push(compute_hash_bytes(&combined));
        }

        level = next_level;
    }

    level[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let data = "test data";
        let hash1 = compute_hash(&data);
        let hash2 = compute_hash(&data);
        assert_eq!(hash1, hash2);

        let hash3 = compute_hash(&"different data");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_merkle_root_single() {
        let leaf = [1u8; 32];
        let root = compute_merkle_root(&[leaf]);
        assert_eq!(root, leaf);
    }

    #[test]
    fn test_merkle_root_two() {
        let leaf1 = [1u8; 32];
        let leaf2 = [2u8; 32];
        let root = compute_merkle_root(&[leaf1, leaf2]);

        // root != leaf1 and root != leaf2
        assert_ne!(root, leaf1);
        assert_ne!(root, leaf2);
    }

    #[test]
    fn test_merkle_root_empty() {
        let root = compute_merkle_root(&[]);
        assert_eq!(root, [0u8; 32]);
    }
}
