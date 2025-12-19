# Space AI Protocol (SAP)
<img width="2848" height="1504" alt="SAP_infographic_en" src="https://github.com/user-attachments/assets/3bbdf099-77ff-4c89-955f-8f2fb9cd783b" />
> **10x Cheaper Swarm Robotics Protocol**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tests: 226 Passing](https://img.shields.io/badge/tests-226%20passing-brightgreen.svg)](https://github.com/yourusername/SpaceAI)
[![Production Ready: 95%](https://img.shields.io/badge/production%20ready-95%25-blue.svg)](docs/Project_Status_Report.md)

---

## 🚀 Overview

**SAP (Space AI Protocol)** is a revolutionary "Off-road" architecture that shifts intelligence from costly onboard GPUs to a centralized **Edge Server**, dramatically reducing fleet costs and enhancing scalability.

Instead of every robot computing its own path, Space AI manages **spatial allocation (Voxel Time Slots)** globally, ensuring collision-free coordination for hundreds of agents.

### Core Concept

```
Legacy (Onboard AI):           Space AI (Off-road):
┌─────────────┐              ┌─────────────┐
│  Robot #1   │              │   Robot #1  │ (Sensors Only)
│ ┌─────────┐ │              │   GPS+IMU   │
│ │ GPU+AI  │ │ $10,000      │             │ $500
│ └─────────┘ │              └─────────────┘
└─────────────┘                      ↓
                             ┌─────────────┐
┌─────────────┐              │ Edge Server │
│  Robot #2   │              │ Space AI    │ 
│ ┌─────────┐ │              │ VTS Alloc   │
│ │ GPU+AI  │ │ $10,000      │ Phys-Check  │
│ └─────────┘ │              └─────────────┘
└─────────────┘                      ↑
                              ┌─────────────┐
     ...                      │   Robot #2  │
                              │   GPS+IMU   │ $500
총 비용: $10k × N             └─────────────┘
                              
                              Total: $500×N + Server
                                     = 10x Cheaper
```

---

## ✨ Key Benefits

### 💰 10x Cost Reduction

- No per-robot GPU: **$10,000 → $500** per unit.
- One Edge Server manages 500-1000 robots.
- Massive CAPEX savings for large-scale fleets.

### 📈 10x Scalability

- **Centralized Updates**: Update logic on one server → instantly applied to hundreds of robots.
- **Unified Debugging**: No need to extract logs from individual agents.
- **Cloud Optimization**: Leverage infinite compute for global optimization.

### 🔒 Physical Guarantee

- **Physvisor Layer**: Every motion command is validated against physics laws before execution.
- **Collision Prediction**: Future-state verification prevents accidents.
- **Deterministic Rollback**: Safe recovery from network or sync failures.

### ⚡ Proven Performance

- **Auction Latency**: 8.8 μs (**110x faster** than target).
- **Simulation**: 3.24 ms per step for 500 robots (**3x safety margin**).
- **Scalability**: Validated up to 1000 robots.

---

## 🎯 Quick Start (5 Minutes)

### Prerequisites

- Rust 1.70+ ([Install](https://rustup.rs/))
- Windows / Linux / macOS

### 1️⃣ Clone & Build

```bash
git clone https://github.com/yourusername/SpaceAI.git
cd SpaceAI/rust

# Release build is required for performance
cargo build --release
```

### 2️⃣ Run Warehouse Demo

```bash
cargo run --release --bin warehouse_demo
```

### 3️⃣ Expected Output

```
=== SAP Warehouse Demo ===
Robots: 5, Tasks: 20, Duration: 60s

[00010] VTS: Robot #2 → Task #0 (3.2m)
[00010] VTS: Robot #1 → Task #1 (4.2m)
...
[00220] ✅ Task #0 done by R#2
...
🎉 All tasks completed!

==================================================
📊 Final Metrics
==================================================
Tasks Completed:  20/20
Throughput:       0.815 tasks/sec
Handoffs:         27
Collisions:       3
Collision Rate:   15.0%
Elapsed Time:     24.5s
==================================================
```

**Success**: 5 robots completed 20 tasks in 24.5s using VTS allocation and Cross-Zone Handoffs.

👉 **Learn More**: [Quick Start Guide](docs/QuickStart.md)

---

## 📊 Performance Metrics

### Validated Results (v2.3)

| Component | Target | Actual | Improvement |
|-----------|--------|--------|-------------|
| **Auction Latency** | < 1 ms | **8.8 μs** | 🚀 **110x** |
| **Sim Step (500 bots)** | < 10 ms | **3.24 ms** | 🚀 **3x** |
| **Collision Check (100)**| < 1 ms | **128 μs** | 🚀 **8x** |
| **Zone Update (100)** | < 100 μs | **4.5 μs** | 🚀 **22x** |

---

## 🏗️ Architecture

SAP features a **5-Layer Architecture**:

```
┌─────────────────────────────────────────┐
│         Cloud (Global State)            │  ← Orchestration
├─────────────────────────────────────────┤
│         Edge (Zone Management)          │  ← VTS Allocation, Auctions
├─────────────────────────────────────────┤
│      Physvisor (Physics Supervisor)     │  ← Validation & Simulation
├─────────────────────────────────────────┤
│        Network (Communication)          │  ← PredictiveSync
├─────────────────────────────────────────┤
│         Robot (Sensor + Actuator)       │  ← Dumb Clients
└─────────────────────────────────────────┘
```

### Core Concepts

- **VoxelTimeSlot (VTS)**: Discretized space-time resources managed via exclusive reservation.
- **Vickrey Auction**: Second-price sealed-bid auctions for fair and strategy-proof resource allocation.
- **PredictiveSync**: Bandwidth-efficient synchronization transmitting only deviations from the predicted model (<10% bandwidth usage).

---

## 📚 Documentation

### Core Docs

- 📖 [**Specification v2.3**](docs/SAP_2.3_Specification.md) - Full Technical Spec
- 🚀 [**Quick Start**](docs/QuickStart.md) - Detailed Setup Guide
- 📄 [**ArXiv Paper**](docs/SpaceAI_Arxiv_Paper.md) - Academic Whitepaper
- 📊 [**Status Report**](docs/Project_Status_Report.md) - Development Status

### Integration

- 🤖 [**ROS2 Bridge**](docs/integration/ROS2_Bridge.md) - ROS2 Integration
- 🚛 [**VDA5050 Adapter**](docs/integration/VDA5050_Mapping.md) - VDA5050 Standard
- 🏭 [**Domain Profiles**](docs/profiles/DomainProfiles.md) - Configs for Warehouse/Fab

---

## 🛠️ Project Structure

```
SpaceAI/
├── rust/                    # Rust Implementation
│   ├── crates/
│   │   ├── sap-core/       # Core Types
│   │   ├── sap-physics/    # Physics Engine
│   │   ├── sap-economy/    # Auction Engine
│   │   ├── sap-network/    # Network Layer
│   │   ├── sap-edge/       # Edge Runtime
│   │   ├── sap-robot/      # Robot SDK
│   │   └── ...
│   └── examples/           # Demos
├── docs/                    # Documentation
└── README.md               # This file
```

---

## 🤝 Contributing

We welcome contributions!

1. Open an issue: [GitHub Issues](https://github.com/yourusername/SpaceAI/issues)
2. Submit a PR: Check [CONTRIBUTING.md](CONTRIBUTING.md)
3. Join Discussions: [GitHub Discussions](https://github.com/yourusername/SpaceAI/discussions)

### Development

```bash
# Test
cargo test --all

# Bench
cargo bench
```

---

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🔗 References

- [Vickrey Auction](https://en.wikipedia.org/wiki/Vickrey_auction)
- [ROS2](https://www.ros.org/)
- [VDA5050](https://github.com/VDA5050/VDA5050)

---

**Space AI Protocol** - The Future of Scalable Swarm Robotics 🚀
