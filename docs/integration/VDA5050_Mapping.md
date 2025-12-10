# VDA 5050 Integration Guide

**SAP** supports the **VDA 5050** standard for AGV communication via an adapter layer.

## 1. Concept Mapping

| VDA 5050 | Space AI | Notes |
|----------|----------|-------|
| **AGV** | **Robot** | The physical agent |
| **Master Control** | **Edge/Cloud** | Central fleet manager |
| **Order** | **Task/VTS** | High level mission |
| **Node** | **Voxel** | Discrete spatial point |
| **Edge** | **Transition** | Movement between voxels |

## 2. MQTT Topics

SAP Adapter listens to and publishes to standard VDA topics:

- `uagv/v2/{manufacturer}/{serialNumber}/state` (Pub)
- `uagv/v2/{manufacturer}/{serialNumber}/order` (Sub)
- `uagv/v2/{manufacturer}/{serialNumber}/instantActions` (Sub)

## 3. Implementation Status

- **State Report**: Partially implemented (Position, Velocity, Battery).
- **Order Parsing**: Supported (Nodes & Edges mapped to VTS sequences).
- **Visualization**: Supported.
