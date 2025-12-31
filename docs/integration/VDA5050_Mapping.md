# SAP ↔ VDA5050 Mapping

**Version**: 1.0  
**Date**: December 8, 2025  
**VDA5050 Version**: 2.0.0

---

## Overview

VDA5050 is a German automotive industry association standard for AGV (Automated Guided Vehicle) communication.
This document defines field mapping between SAP protocol and VDA5050.

---

## Core Concept Mapping

| VDA5050 Concept | SAP Concept | Description |
|-----------------|-------------|-------------|
| AGV | Robot | Autonomous mobile robot |
| Order | TransitTicket | Movement allocation |
| Node | VoxelTimeSlot | Spatiotemporal allocation |
| Edge | Path Segment | Path section |
| Action | MotionCommand | Motion command |
| Master Control | Edge Server | Central control |

---

## State Mapping (VDA5050 → SAP)

### AGV Position → SAP Position

| VDA5050 Field | SAP Field | Type | Conversion |
|---------------|-----------|------|------------|
| `agvPosition.x` | `position.x` | f64 → f32 | Direct |
| `agvPosition.y` | `position.y` | f64 → f32 | Direct |
| `agvPosition.theta` | `position.theta` | f64 → f32 | Direct |
| `agvPosition.mapId` | `zone_id` | string → u32 | Hash/Mapping |
| `agvPosition.positionInitialized` | - | bool | N/A |

### Velocity → SAP Velocity

| VDA5050 Field | SAP Field | Unit |
|---------------|-----------|------|
| `velocity.vx` | `velocity.vx` | m/s |
| `velocity.vy` | `velocity.vy` | m/s |
| `velocity.omega` | `velocity.omega` | rad/s |

### Operating Mode → SAP Status

| VDA5050 operatingMode | SAP RobotState.status |
|----------------------|----------------------|
| `AUTOMATIC` | Active |
| `SEMIAUTOMATIC` | Active |
| `MANUAL` | Idle |
| `SERVICE` | Maintenance |
| `TEACHIN` | Idle |

### Safety State → SAP

| VDA5050 safetyState | SAP Handling |
|--------------------|--------------|
| `fieldViolation: true` | GeofenceViolation error |
| `eStop: AUTOACK/MANUAL/REMOTE/NONE` | EmergencyStop command |

---

## Order Mapping (SAP → VDA5050)

### TransitTicket → VDA5050 Order

| SAP Field | VDA5050 Field | Conversion |
|-----------|---------------|------------|
| `ticket_id` | `orderId` | u128 → UUID string |
| `zone_id` | `zoneSetId` | u32 → string |
| `vts.voxel_id` | `nodes[].nodeId` | u64 → string |
| `vts.t_start_ns` | `nodes[].sequenceId` | Sequence-based |

### MotionCommand → VDA5050 Action

| SAP MotionCommand | VDA5050 actionType |
|------------------|-------------------|
| Move | `drive` |
| Stop | `cancelOrder` |
| Rotate | `pick` (rotate then wait) |

---

## Error Mapping

| VDA5050 errorType | SAP SapError |
|-------------------|--------------|
| `orderError` | `InvalidTicket` |
| `orderNoRoute` | `AuctionFailed` |
| `validationError` | `VTSViolation` |
| `noRouteToTarget` | `GeofenceViolation` |

---

## Implementation Guide

### 1. Adapter Structure

```text
┌─────────────────────────────────────┐
│        vda5050-adapter              │
│  ┌─────────────────────────────┐   │
│  │  State Translator           │   │
│  │  - VDA State → SAP State    │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │  Order Translator           │   │
│  │  - SAP Ticket → VDA Order   │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │  MQTT Client                │   │
│  │  - Broker connection        │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

### 2. MQTT Topics

| Direction | Topic Pattern | Description |
|-----------|--------------|-------------|
| AGV → MC | `vda5050/v2/{manufacturer}/{serialNumber}/state` | Status report |
| MC → AGV | `vda5050/v2/{manufacturer}/{serialNumber}/order` | Command delivery |

### 3. Important Notes

- **Timestamp**: VDA5050 uses ISO 8601 strings, SAP uses nanosecond integers
- **Coordinate System**: VDA5050 uses right-hand system (RHS), same as SAP
- **map_id**: Mapping table required for VDA5050 string → SAP zone_id

---

## Next Steps

1. Create `vda5050-adapter` Rust crate
2. Integrate MQTT client (rumqttc)
3. Test with actual AGVs
