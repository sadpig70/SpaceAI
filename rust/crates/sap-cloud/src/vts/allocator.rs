//! VtsAllocator - VoxelTimeSlot 할당자
//!
//! PPR 매핑: AI_make_VtsAllocator

use sap_core::ticket::VoxelTimeSlot;
use std::collections::HashMap;

/// VTS 할당자
pub struct VtsAllocator {
    allocated: HashMap<u64, VtsAllocation>,
    pending: Vec<VtsRequest>,
    next_vts_id: u64,
    zone_limits: HashMap<u32, usize>,
}

#[derive(Debug, Clone)]
pub struct VtsAllocation {
    pub vts: VoxelTimeSlot,
    pub robot_id: u64,
    pub allocated_at_ns: u64,
}

#[derive(Debug, Clone)]
pub struct VtsRequest {
    pub request_id: u64,
    pub robot_id: u64,
    pub zone_id: u32,
    pub voxel_id: u64,
    pub t_start_ns: u64,
    pub t_end_ns: u64,
    pub priority: u8,
}

impl VtsAllocator {
    pub fn new() -> Self {
        Self {
            allocated: HashMap::new(),
            pending: Vec::new(),
            next_vts_id: 1,
            zone_limits: HashMap::new(),
        }
    }

    pub fn set_zone_limit(&mut self, zone_id: u32, limit: usize) {
        self.zone_limits.insert(zone_id, limit);
    }

    pub fn request(&mut self, req: VtsRequest) -> u64 {
        let request_id = self.pending.len() as u64 + 1;
        let mut req = req;
        req.request_id = request_id;
        self.pending.push(req);
        request_id
    }

    pub fn allocate(&mut self, request_id: u64, current_time_ns: u64) -> Option<VtsAllocation> {
        let idx = self
            .pending
            .iter()
            .position(|r| r.request_id == request_id)?;
        let req = self.pending.remove(idx);

        if self.has_conflict(&req) {
            return None;
        }

        if let Some(&limit) = self.zone_limits.get(&req.zone_id) {
            let count = self.count_allocations_in_zone(req.zone_id);
            if count >= limit {
                return None;
            }
        }

        let vts_id = self.next_vts_id;
        self.next_vts_id += 1;

        let vts = VoxelTimeSlot::new(req.voxel_id, req.t_start_ns, req.t_end_ns);

        let allocation = VtsAllocation {
            vts,
            robot_id: req.robot_id,
            allocated_at_ns: current_time_ns,
        };

        self.allocated.insert(vts_id, allocation.clone());
        Some(allocation)
    }

    fn has_conflict(&self, req: &VtsRequest) -> bool {
        self.allocated.values().any(|alloc| {
            alloc.vts.voxel_id == req.voxel_id
                && alloc.vts.t_start_ns < req.t_end_ns
                && alloc.vts.t_end_ns > req.t_start_ns
        })
    }

    fn count_allocations_in_zone(&self, _zone_id: u32) -> usize {
        self.allocated.len()
    }

    pub fn release(&mut self, vts_id: u64) -> bool {
        self.allocated.remove(&vts_id).is_some()
    }

    pub fn cleanup_expired(&mut self, current_time_ns: u64) -> usize {
        let expired: Vec<u64> = self
            .allocated
            .iter()
            .filter(|(_, a)| a.vts.t_end_ns < current_time_ns)
            .map(|(&id, _)| id)
            .collect();

        let count = expired.len();
        for id in expired {
            self.allocated.remove(&id);
        }
        count
    }

    pub fn get_allocation(&self, vts_id: u64) -> Option<&VtsAllocation> {
        self.allocated.get(&vts_id)
    }

    pub fn get_robot_allocations(&self, robot_id: u64) -> Vec<&VtsAllocation> {
        self.allocated
            .values()
            .filter(|a| a.robot_id == robot_id)
            .collect()
    }

    pub fn allocated_count(&self) -> usize {
        self.allocated.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for VtsAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_request(robot_id: u64, voxel_id: u64, t_start: u64, t_end: u64) -> VtsRequest {
        VtsRequest {
            request_id: 0,
            robot_id,
            zone_id: 1,
            voxel_id,
            t_start_ns: t_start,
            t_end_ns: t_end,
            priority: 0,
        }
    }

    #[test]
    fn test_request_and_allocate() {
        let mut alloc = VtsAllocator::new();
        let req_id = alloc.request(create_request(1, 100, 1000, 2000));
        let result = alloc.allocate(req_id, 0);
        assert!(result.is_some());
        assert_eq!(alloc.allocated_count(), 1);
    }

    #[test]
    fn test_conflict_detection() {
        let mut alloc = VtsAllocator::new();
        let r1 = alloc.request(create_request(1, 100, 1000, 2000));
        alloc.allocate(r1, 0);
        let r2 = alloc.request(create_request(2, 100, 1500, 2500));
        let result = alloc.allocate(r2, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_conflict_different_voxel() {
        let mut alloc = VtsAllocator::new();
        let r1 = alloc.request(create_request(1, 100, 1000, 2000));
        alloc.allocate(r1, 0);
        let r2 = alloc.request(create_request(2, 200, 1500, 2500));
        let result = alloc.allocate(r2, 0);
        assert!(result.is_some());
        assert_eq!(alloc.allocated_count(), 2);
    }

    #[test]
    fn test_no_conflict_different_time() {
        let mut alloc = VtsAllocator::new();
        let r1 = alloc.request(create_request(1, 100, 1000, 2000));
        alloc.allocate(r1, 0);
        let r2 = alloc.request(create_request(2, 100, 2000, 3000));
        let result = alloc.allocate(r2, 0);
        assert!(result.is_some());
    }

    #[test]
    fn test_release() {
        let mut alloc = VtsAllocator::new();
        let r1 = alloc.request(create_request(1, 100, 1000, 2000));
        alloc.allocate(r1, 0);
        assert!(alloc.release(1));
        assert_eq!(alloc.allocated_count(), 0);
    }

    #[test]
    fn test_cleanup_expired() {
        let mut alloc = VtsAllocator::new();
        let r1 = alloc.request(create_request(1, 100, 1000, 2000));
        let r2 = alloc.request(create_request(2, 200, 1000, 5000));
        alloc.allocate(r1, 0);
        alloc.allocate(r2, 0);
        let cleaned = alloc.cleanup_expired(3000);
        assert_eq!(cleaned, 1);
        assert_eq!(alloc.allocated_count(), 1);
    }
}
