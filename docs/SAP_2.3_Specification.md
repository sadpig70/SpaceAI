# SAP 2.3 - Space AI Protocol Specification

**Version**: 2.3  
**License**: MIT License  
**Date**: 2025-12-10  
**Production Readiness**: ✅ 95% (Verified)

---

## 1. Overview

**SAP (Space AI Protocol)** is a spatiotemporal trading protocol for coordinating swarms of Autonomous Mobile Robots (AMRs). By treating space as a tradable asset (**Voxel Time Slot**), SAP enables centralized optimization with decentralized execution.

### Key Values

| Value | Description |
|-------|-------------|
| **Safety** | Physics-based pre-validation prevents collisions |
| **Fairness** | Vickrey Auction mechanism ensures truthful bidding |
| **Efficiency** | Dynamic pricing suppresses congestion |
| **Resilience** | PredictiveSync + Deterministic Rollback |

---

## 2. System Architecture

### 5-Layer Stack

```
1. Cloud Layer (Global State)
   - Multi-zone orchestration
   - Long-term analytics

2. Edge Layer (Zone Management)
   - Core runtime (soft real-time)
   - VTS Allocation & Auction Engine

3. Physvisor Layer (Physics Supervisor)
   - Kinematic validation (Safety Kernel)
   - Future state simulation

4. Network Layer
   - Custom UDP protocol
   - PredictiveSync (delta compression)

5. Robot Layer
   - Dumb Client (Sensors + Actuators)
   - Failsafe execution
```

---

## 3. Core Concepts

### 3.1 Voxel Time Slot (VTS)

A **VTS** is the fundamental unit of resource allocation.

- **Micro-Space**: 3D cubic voxel (e.g., $0.5m \times 0.5m \times 2.0m$)
- **Micro-Time**: Exclusive time interval $[t_{start}, t_{end})$

Constraint: $\forall i \neq j, VTS_i \cap VTS_j = \emptyset$

### 3.2 TransitTicket

A cryptographically signed token granting permission to occupy a VTS.

- **Fields**: `robot_id`, `vts_id`, `time_window`, `signature`
- **usage**: Robot MUST hold a valid ticket to move.

### 3.3 Vickrey Auction

Mechanism for allocating contentious VTS.

- **Bid**: $B_i = f(priority, urgency)$
- **Winner**: Highest bidder
- **Price**: Second highest bid (Vickrey Price)

---

## 4. Key Algorithms

### 4.1 PredictiveSync

Reduces network bandwidth by capitalizing on deterministic motion.

1. **Edge** simulates Robot state $S'_{t+\Delta}$.
2. **Robot** reports actual state $S_{t+\Delta}$.
3. **Sync**: Only if $|S - S'| > \epsilon$ (threshold), a correction packet is sent.
4. **Result**: 90% bandwidth reduction.

### 4.2 Deterministic Rollback

Handles state divergence in distributed systems.

1. System maintains snapshots every $N$ ticks.
2. If validation fails at $T$, system rolls back to $T_{valid}$.
3. All affected actors receive Trajectory Correction commands.

---

## 5. Interfaces & API

### 5.1 Robot SDK (`sap-robot`)

```rust
// Basic loop
let mut client = RobotClient::new(config);
loop {
    let sensors = hardware.read();
    let command = client.step(sensors)?;
    hardware.drive(command.velocity);
}
```

### 5.2 Edge API

- `POST /v1/zone/register`: Register new zone
- `POST /v1/robot/command`: Inject high-level task

---

## 6. Security Model

- **Trust**: Hierarchical (Cloud > Edge > Robot)
- **Crypto**: Ed25519 for Ticket Signing
- **Time**: PTP (IEEE 1588) for high precision, NTP for standard.

### Time Synchronization

- **Max Skew**: $<\frac{1}{2} \times \frac{voxel\_size}{v_{max}}$
- **FAB Profile**: Requires hardware PTP ($\sim 1 \mu s$)
- **Warehouse Profile**: Standard NTP ($\sim 10 ms$) is sufficient.

---

## 7. Performance Specification

| Metric | Target | Actual (v2.3) |
|--------|--------|---------------|
| **Auction Latency** | < 1 ms | **8.8 μs** |
| **Zone Update (100 bots)** | < 100 µs | **4.5 μs** |
| **Sim Step (500 bots)** | < 10 ms | **3.24 ms** |

---

## 8. Appendix

### Domain Profiles

- **WAREHOUSE**: $1.0m$ voxels, $2.5 m/s$ max speed.
- **FAB**: $0.5m$ voxels, $0.8 m/s$ max speed, High precision.
- **HOSPITAL**: $0.8m$ voxels, $1.0 m/s$ max speed, Safety priority.

---

**End of Specification**
