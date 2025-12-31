# SAP Domain Profiles

**Version**: 1.0  
**Date**: December 8, 2025  

---

## Overview

Domain profiles are SAP parameter sets optimized for specific environments.
Each domain has unique physical, operational, and security requirements.

---

## Profile Summary

| Profile | Environment | VTS Resolution | Collision Threshold | Security Level |
|---------|-------------|---------------|---------------------|----------------|
| **WAREHOUSE** | Logistics warehouse | 1.0m | 0.15m | Medium |
| **FAB** | Semiconductor fab | 0.5m | 0.05m | High |
| **HOSPITAL** | Hospital | 0.8m | 0.20m | High |

---

## WAREHOUSE (Logistics)

### Environment Characteristics

- Wide aisles (2.5m+)
- High-speed transport (2+ m/s)
- High robot density (50+ units/zone)

### Recommended Parameters

```toml
[profile.warehouse]
# VTS settings
voxel_size_m = 1.0
time_slot_ms = 100
max_vts_per_robot = 50

# Physics limits
max_velocity_ms = 2.5
max_acceleration_ms2 = 2.0
max_angular_velocity_rads = 2.0
safety_margin_m = 0.2

# Collision prediction
collision_threshold_m = 0.15
prediction_horizon_s = 3.0
dynamic_horizon = true

# Security
signature_required = true
replay_window_ms = 10000
```

### Robot Types

- `VehicleProfile::amr()` recommended
- Optimized for differential drive

---

## FAB (Semiconductor Factory)

### Environment Characteristics

- Narrow aisles (1.2m)
- Ultra-precision movement
- Cleanroom environment (minimize vibration)

### Recommended Parameters

```toml
[profile.fab]
# VTS settings (high resolution)
voxel_size_m = 0.5
time_slot_ms = 50
max_vts_per_robot = 100

# Physics limits (strict)
max_velocity_ms = 0.8
max_acceleration_ms2 = 0.5
max_jerk_ms3 = 2.0
safety_margin_m = 0.1

# Collision prediction (sensitive)
collision_threshold_m = 0.05
prediction_horizon_s = 5.0
rollback_threshold_m = 0.03

# Security (maximum)
signature_required = true
replay_window_ms = 3000
sybil_check = true
```

### Special Requirements

- **PTP synchronization required** (NTP not allowed)
- **Vibration suppression**: Strict jerk limits
- **Cleanroom certification**: Verification during robot registration

---

## HOSPITAL

### Environment Characteristics

- Human-robot coexistence
- Frequent dynamic obstacles
- Quiet operation required

### Recommended Parameters

```toml
[profile.hospital]
# VTS settings
voxel_size_m = 0.8
time_slot_ms = 200
max_vts_per_robot = 30

# Physics limits (safety first)
max_velocity_ms = 1.0
max_acceleration_ms2 = 0.8
safety_margin_m = 0.5  # Account for humans

# Collision prediction (conservative)
collision_threshold_m = 0.20
prediction_horizon_s = 4.0
human_detection = true

# Security
signature_required = true
patient_area_restriction = true
```

### Special Requirements

- **Human detection**: LIDAR + camera fusion
- **Noise limit**: Below 50dB
- **Emergency stop**: Response time < 100ms

---

## Parameter Conflict Priority

1. **Safety** > Performance > Throughput
2. Domain profile > Global settings > Defaults
3. Dynamic adjustment allowed only within range

---

## Profile Extension

When adding new domain profiles:

```rust
impl DomainProfile {
    pub fn custom(name: &str) -> ProfileBuilder {
        ProfileBuilder::new(name)
            .base(Self::warehouse())  // Inherit from base profile
    }
}
```
