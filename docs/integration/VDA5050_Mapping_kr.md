# SAP ↔ VDA5050 매핑

**버전**: 1.0  
**작성일**: 2025-12-08  
**VDA5050 버전**: 2.0.0

---

## 개요

VDA5050은 AGV(무인반송차) 통신을 위한 독일 자동차 산업 협회 표준입니다.
이 문서는 SAP 프로토콜과 VDA5050 간의 필드 매핑을 정의합니다.

---

## 핵심 개념 매핑

| VDA5050 개념 | SAP 개념 | 설명 |
|-------------|---------|------|
| AGV | Robot | 자율 이동 로봇 |
| Order | TransitTicket | 이동 할당 |
| Node | VoxelTimeSlot | 공간-시간 할당 |
| Edge | Path Segment | 경로 구간 |
| Action | MotionCommand | 동작 명령 |
| Master Control | Edge Server | 중앙 제어 |

---

## State 매핑 (VDA5050 → SAP)

### AGV Position → SAP Position

| VDA5050 Field | SAP Field | 타입 | 변환 |
|---------------|-----------|------|------|
| `agvPosition.x` | `position.x` | f64 → f32 | 직접 |
| `agvPosition.y` | `position.y` | f64 → f32 | 직접 |
| `agvPosition.theta` | `position.theta` | f64 → f32 | 직접 |
| `agvPosition.mapId` | `zone_id` | string → u32 | 해시/매핑 |
| `agvPosition.positionInitialized` | - | bool | N/A |

### Velocity → SAP Velocity

| VDA5050 Field | SAP Field | 단위 |
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

| VDA5050 safetyState | SAP 처리 |
|--------------------|---------|
| `fieldViolation: true` | GeofenceViolation 에러 |
| `eStop: AUTOACK/MANUAL/REMOTE/NONE` | EmergencyStop 명령 |

---

## Order 매핑 (SAP → VDA5050)

### TransitTicket → VDA5050 Order

| SAP Field | VDA5050 Field | 변환 |
|-----------|---------------|------|
| `ticket_id` | `orderId` | u128 → UUID string |
| `zone_id` | `zoneSetId` | u32 → string |
| `vts.voxel_id` | `nodes[].nodeId` | u64 → string |
| `vts.t_start_ns` | `nodes[].sequenceId` | 순서 기반 |

### MotionCommand → VDA5050 Action

| SAP MotionCommand | VDA5050 actionType |
|------------------|-------------------|
| Move | `drive` |
| Stop | `cancelOrder` |
| Rotate | `pick` (회전 후 대기) |

---

## 에러 매핑

| VDA5050 errorType | SAP SapError |
|-------------------|--------------|
| `orderError` | `InvalidTicket` |
| `orderNoRoute` | `AuctionFailed` |
| `validationError` | `VTSViolation` |
| `noRouteToTarget` | `GeofenceViolation` |

---

## 구현 가이드

### 1. 어댑터 구조

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
│  │  - broker 연결               │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

### 2. MQTT 토픽

| 방향 | 토픽 패턴 | 설명 |
|------|----------|------|
| AGV → MC | `vda5050/v2/{manufacturer}/{serialNumber}/state` | 상태 보고 |
| MC → AGV | `vda5050/v2/{manufacturer}/{serialNumber}/order` | 명령 전달 |

### 3. 주의사항

- **타임스탬프**: VDA5050은 ISO 8601 문자열, SAP는 나노초 정수
- **좌표계**: VDA5050은 우수 좌표계 (RHS), SAP도 동일
- **map_id**: VDA5050 string → SAP zone_id 매핑 테이블 필요

---

## 다음 단계

1. `vda5050-adapter` Rust 크레이트 생성
2. MQTT 클라이언트 통합 (rumqttc)
3. 실제 AGV 테스트
