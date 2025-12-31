# SAP ↔ ROS2 Bridge Design

**Version**: 1.0  
**Date**: December 8, 2025  

---

## Overview

This document defines the interface design between SAP (Spatial Allocation Protocol) and ROS2 systems.
It specifies the adapter concept for integrating ROS2-based robots into the SAP ecosystem.

---

## Architecture

```text
┌─────────────────────────────────────────────┐
│             ROS2 Robot                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ /tf      │  │ /cmd_vel │  │ /odom    │  │
│  └────┬─────┘  └────▲─────┘  └────┬─────┘  │
└───────│─────────────│─────────────│────────┘
        │             │             │
┌───────▼─────────────▼─────────────▼────────┐
│           sap-ros2-bridge (Node)           │
│  ┌──────────────────────────────────────┐  │
│  │  TransformListener → Position        │  │
│  │  Velocity → cmd_vel                  │  │
│  │  SAP Ticket → custom_msgs/Ticket     │  │
│  └──────────────────────────────────────┘  │
└─────────────────────│──────────────────────┘
                      │ TCP/WebSocket
                      ▼
               ┌──────────────┐
               │  SAP Edge    │
               └──────────────┘
```

---

## Topic Mapping

### ROS2 → SAP

| ROS2 Topic | ROS2 Type | SAP Type | Description |
|------------|-----------|----------|-------------|
| `/tf` | `tf2_msgs/TFMessage` | `Position` | Robot position (base_link → map) |
| `/odom` | `nav_msgs/Odometry` | `Velocity` | Robot velocity |
| `/robot_status` | `std_msgs/String` | `RobotState.status` | Status code |

### SAP → ROS2

| SAP Type | ROS2 Topic | ROS2 Type | Description |
|----------|------------|-----------|-------------|
| `MotionCommand` | `/cmd_vel` | `geometry_msgs/Twist` | Velocity command |
| `RecoveryCommand` | `/sap/recovery` | `sap_msgs/Recovery` | Recovery command |
| `TransitTicket` | `/sap/ticket` | `sap_msgs/Ticket` | Assigned ticket |

---

## Custom Message Definitions

### sap_msgs/Ticket.msg

```yaml
# SAP Transit Ticket
uint64 ticket_id
uint32 zone_id
uint64 voxel_id
uint64 t_start_ns
uint64 t_end_ns
uint8 status  # 0=PENDING, 1=ACTIVE, 2=EXPIRED
```

### sap_msgs/Recovery.msg

```yaml
# SAP Recovery Command
uint8 level  # 0=NONE, 1=DECELERATE, 2=STOP, 3=EMERGENCY
float32 target_velocity
string reason
```

### sap_msgs/VTSRequest.msg

```yaml
# VTS Allocation Request
uint64 robot_id
uint32 zone_id
geometry_msgs/Point destination
uint64 deadline_ns
uint8 priority
```

---

## Conversion Logic

### Position Conversion

```python
# ROS2 tf → SAP Position
def tf_to_sap_position(transform: TransformStamped) -> Position:
    return Position(
        x=transform.transform.translation.x,
        y=transform.transform.translation.y,
        z=transform.transform.translation.z,
        theta=yaw_from_quaternion(transform.transform.rotation)
    )
```

### Velocity Conversion

```python
# SAP Velocity → ROS2 Twist
def sap_velocity_to_twist(velocity: Velocity) -> Twist:
    twist = Twist()
    twist.linear.x = velocity.vx
    twist.linear.y = velocity.vy
    twist.angular.z = velocity.omega
    return twist
```

---

## Implementation Requirements

1. **Node Configuration**: `sap_bridge_node` (Lifecycle Node)
2. **QoS**: Sensor data uses Best Effort, commands use Reliable
3. **Timestamp**: Convert ROS2 Time → Unix Nanoseconds
4. **Frame ID**: Use `map` frame coordinates

---

## Next Steps

1. Create `sap_msgs` ROS2 package
2. Implement `sap_bridge` C++/Python node
3. Test with Gazebo simulation
