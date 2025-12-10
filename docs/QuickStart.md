# SAP Quick Start Guide

**Goal**: Run the SAP Warehouse Demo in 5 minutes.

---

## Prerequisites

### Required

1. **Rust 1.70+**
   - Check: `rustc --version`
   - Install: <https://rustup.rs/>

2. **Git**
   - Check: `git --version`

### Recommended

- **OS**: Windows 10+, Linux, macOS
- **RAM**: 4GB+
- **Disk**: 1GB+

---

## Installation

### Step 1: Clone Repository

```bash
git clone https://github.com/yourusername/SpaceAI.git
cd SpaceAI
```

### Step 2: Build (Release Mode)

**Important**: Debug builds are slow. Always use `--release` for simulation.

```bash
cd rust
cargo build --release
```

**Estimated Time**: 1-3 minutes.

---

## Run Demo

### Method 1: Cargo Run (Recommended)

```bash
cd rust
cargo run --release --bin warehouse_demo
```

### Method 2: Manually

```bash
# Windows
.\target\release\warehouse_demo.exe

# Linux/macOS
./target/release/warehouse_demo
```

---

## Expected Output

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
Allocations:      20
Handoffs:         27
Collisions:       3
Collision Rate:   15.0%
Elapsed Time:     24.5s
==================================================
```

### Metrics Explained

- **Tasks Completed**: 100% completion rate.
- **Throughput**: Tasks processed per second.
- **Handoffs**: Number of times robots crossed zone boundaries (validating distributed protocol).
- **Elapsed Time**: 24.5s (faster than 60s timeout).

---

## Troubleshooting

### Q: `rustc` command not found

**A**: Install Rust via `rustup.rs` and restart terminal.

### Q: "linker 'cc' not found" (Linux)

**A**: Install build tools:

- Ubuntu: `sudo apt install build-essential`
- Fedora: `sudo yum install gcc`

### Q: Simulation is extremely slow

**A**: Did you forget `--release`? Debug builds are 10-100x slower.

### Q: Windows binary blocked?

**A**: Add `target/release` to antivirus exclusion list or use `cargo run`.

---

## Next Steps

1. **Read Specification**: [SAP_2.3_Specification.md](SAP_2.3_Specification.md)
2. **Explore Code**: `rust/examples/warehouse_demo.rs`
3. **Run Benchmarks**: `cargo bench`

---

**Happy Swarming!** 🚀
