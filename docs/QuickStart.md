# SAP Quick Start Guide

**Time required**: 5 minutes  
**Version**: SAP 2.3  
**Difficulty**: Beginner

---

## 📋 Prerequisites

### Minimum Requirements

- OS: Windows, Linux, or macOS
- Rust: 1.70 or later ([Install](https://rustup.rs/))
- Mem: 4GB RAM
- Disk: 1GB free space

### Recommended Environment

- OS: Ubuntu 22.04 LTS
- Rust: Latest stable
- Mem: 8GB+ RAM
- CPU: Multi-core processor

---

## 🚀 Installation

### Step 1: Clone Repository

```bash
git clone https://github.com/yourusername/SpaceAI.git
cd SpaceAI/rust
```

### Step 2: Build

```bash
# Release build (required for performance)
cargo build --release

# Expected time: 1-2 minutes
```

**Important**: Release build is mandatory. Debug builds are too slow for demos.

---

## 🎯 Run Warehouse Demo

### Quick Run

```bash
cargo run --release --bin warehouse_demo
```

### Expected Output

```
=== SAP Warehouse Demo ===
Robots: 5, Tasks: 20, Duration: 60s

[Simulation Start]
[00010] VTS: Robot #2 → Task #0 (distance: 3.2m)
[00010] VTS: Robot #1 → Task #1 (distance: 4.2m)
[00015] Auction: Robot #3 wins VTS_12 (bid: 15.2)
...
[00220] ✅ Task #0 completed by Robot #2
[00235] ✅ Task #1 completed by Robot #1
...

[Simulation Complete]
==================================================
📊 Final Metrics
==================================================
Tasks Completed:  20/20
Throughput:       0.815 tasks/sec
Cross-Zone Handoffs: 27
Collisions Detected: 3
Collision Rate:   15.0%
Execution Time:   24.5s
==================================================
```

### What Just Happened?

1. **5 robots** collaborated to complete **20 tasks**
2. Used **VTS (Voxel Time Slot)** allocation for collision-free coordination
3. **Vickrey Auction** resolved resource contention
4. **Cross-Zone Handoffs** enabled efficient task distribution

---

## 📊 Understanding the Results

### Key Metrics

| Metric | Value | Description |
|--------|-------|-------------|
| **Tasks Completed** | 20/20 | 100% success rate |
| **Throughput** | 0.815 tasks/sec | Average task completion rate |
| **Handoffs** | 27 | Tasks transferred between zones |
| **Collisions** | 3 (15%) | Collision detection events |
| **Execution Time** | 24.5s | Total simulation time |

### Success Indicators

✅ **All tasks completed**: System successfully coordinated 5 robots  
✅ **High throughput**: Efficient resource allocation  
✅ **Cross-zone handoffs**: Demonstrated multi-zone coordination  
⚠️ **Some collisions**: Path planning can be further optimized

---

## 🔧 Customizing the Demo

### Configuration

Edit `rust/examples/warehouse_demo.rs`:

```rust
// Change robot count
let num_robots = 10;  // Default: 5

// Change task count
let num_tasks = 50;   // Default: 20

// Change duration
let duration_secs = 120;  // Default: 60
```

### Rebuild and Run

```bash
cargo build --release --bin warehouse_demo
cargo run --release --bin warehouse_demo
```

---

## 🧪 Run Tests

### All Tests

```bash
cd rust
cargo test --all --release
```

**Expected**: 226 tests passing (100%)

### Specific Crate

```bash
# Test physics engine
cargo test -p sap-physics --release

# Test auction system
cargo test -p sap-economy --release
```

---

## 📈 Run Benchmarks

### Prerequisites

```bash
# Install criterion (if not already)
cargo install cargo-criterion
```

### Run All Benchmarks

```bash
cargo bench --all
```

### View Results

Benchmark results saved to: `target/criterion/reports/index.html`

Open in browser to see detailed performance charts.

### Expected Performance

| Benchmark | Target | Actual |
|-----------|--------|--------|
| Auction (100 bids) | <1ms | **8.8μs** |
| Simulation (500 robots) | <10ms | **3.24ms** |
| Zone update | <100μs | **4.5μs** |

---

## 🌐 Integration Examples

### ROS2 Integration

See: [`docs/integration/ROS2_Bridge.md`](integration/ROS2_Bridge.md)

Quick start:

```bash
# Install ROS2 humble (if not already)
# Clone sap-ros2-bridge package
# ros2 run sap_bridge sap_bridge_node
```

### VDA5050 Integration

See: [`docs/integration/VDA5050_Mapping.md`](integration/VDA5050_Mapping.md)

Quick start:

```bash
# Configure MQTT broker
# Run VDA5050 adapter
# cargo run --bin vda5050_adapter
```

---

## 📚 Next Steps

### Documentation

- **Full Specification**: [`SAP_2.3_Specification.md`](SAP_2.3_Specification.md)
- **Project Status**: [`Project_Status_Report.md`](Project_Status_Report.md)
- **Domain Profiles**: [`profiles/DomainProfiles.md`](profiles/DomainProfiles.md)

### Development

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all

# Build documentation
cargo doc --no-deps --open
```

### Production Deployment

For production deployment guides:

1. **Hardware requirements** → See `SAP_2.3_Specification.md` §12
2. **Network configuration** → See `SAP_2.3_Specification.md` §11
3. **Security setup** → See `SAP_2.3_Specification.md` §10

---

## 🐛 Troubleshooting

### Build Errors

**Problem**: `cargo build` fails  
**Solution**:

```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Demo Doesn't Run

**Problem**: `warehouse_demo` crashes  
**Solution**:

```bash
# Check Rust version
rustc --version  # Should be 1.70+

# Run with debug output
RUST_LOG=debug cargo run --release --bin warehouse_demo
```

### Performance Issues

**Problem**: Demo is slow  
**Solution**:

- **Use release build**: `--release` flag is required
- **Check CPU**: Multi-core processor recommended
- **Close other applications**: Free up system resources

---

## 💬 Getting Help

### Resources

- **GitHub Issues**: [Report bugs](https://github.com/yourusername/SpaceAI/issues)
- **Discussions**: [Ask questions](https://github.com/yourusername/SpaceAI/discussions)
- **Documentation**: Browse `docs/` folder

### Community

- **Contributors**: See [CONTRIBUTING.md](../CONTRIBUTING.md)
- **License**: MIT License

---

## ✅ You're Ready

Congratulations! You've successfully:

- ✅ Installed SAP
- ✅ Run the warehouse demo
- ✅ Understood key concepts (VTS, Auction, Handoffs)
- ✅ Explored performance benchmarks

**Next**: Dive deeper into the [Full Specification](SAP_2.3_Specification.md) or try integrating with [ROS2](integration/ROS2_Bridge.md)!

---

**Quick Start Guide** | SAP 2.3 | Last updated: 2025-12-10
