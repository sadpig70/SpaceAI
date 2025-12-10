# SAP Domain Profiles

Space AI supports configurable **Domain Profiles** to optimize behavior for different environments.

## 1. WAREHOUSE (Default)

Optimized for logistics centers with moderate speed and standard precision.

```toml
[profile]
name = "WAREHOUSE"
voxel_size = 1.0  # meters
max_velocity = 2.5 # m/s
time_sync = "NTP"
safety_margin = 0.2
```

## 2. FAB (Semiconductor)

High-precision, low-latency environment for cleanrooms.

```toml
[profile]
name = "FAB"
voxel_size = 0.5  # meters
max_velocity = 0.8 # m/s
time_sync = "PTP" # Requires IEEE 1588
safety_margin = 0.05 # Tighter packing
```

## 3. HOSPITAL

Safety-critical environment with mixed human traffic.

```toml
[profile]
name = "HOSPITAL"
voxel_size = 0.8
max_velocity = 1.0
time_sync = "NTP"
safety_margin = 0.4 # Larger buffer for human safety
human_detection = true
```

## Usage

Specify profile in `config.toml`:

```toml
[system]
domain_profile = "WAREHOUSE"
```
