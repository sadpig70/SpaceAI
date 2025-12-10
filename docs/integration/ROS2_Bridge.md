# ROS 2 Integration Guide

This guide describes how to bridge **SAP Edge** with **ROS 2** (Robot Operating System).

## 1. Overview

The `sap_ros2_bridge` node translates SAP TransitTickets and VTS definitions into ROS 2 navigation goals, and reports ROS 2 odometry back to the SAP Edge.

## 2. Topic Mapping

| Direction | ROS 2 Topic | Type | SAP Concept | Description |
|-----------|-------------|------|-------------|-------------|
| **RX (Sub)** | `/cmd_vel` | `geometry_msgs/Twist` | `MotionCommand` | Velocity output from Edge |
| **TX (Pub)** | `/odom` | `nav_msgs/Odometry` | `RobotState` | Current pose & velocity |
| **RX (Sub)** | `/sap/ticket` | `sap_msgs/TransitTicket` | `TransitTicket` | Granting VTS permission |
| **TX (Pub)** | `/sap/request`| `sap_msgs/VTSRequest` | `VTSRequest` | Robot requesting space |

## 3. Custom Messages (`sap_msgs`)

### TransitTicket.msg

```
uint128 ticket_id
uint64 vts_id
uint64 start_time_ns
uint64 end_time_ns
uint8[] signature
```

## 4. Usage

```bash
# Launch Bridge
ros2 launch sap_bridge bridge.launch.py edge_ip:=192.168.1.100
```
