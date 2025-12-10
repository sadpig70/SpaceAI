# SAP ↔ ROS2 Bridge 설계

**버전**: 1.0  
**작성일**: 2025-12-08  

---

## 개요

SAP(Spatial Allocation Protocol)와 ROS2 시스템 간의 인터페이스 설계입니다.
이 문서는 ROS2 기반 로봇을 SAP 생태계에 통합하기 위한 어댑터 개념을 정의합니다.

---

## 아키텍처

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

## 토픽 매핑

### ROS2 → SAP

| ROS2 Topic | ROS2 Type | SAP Type | 설명 |
|------------|-----------|----------|------|
| `/tf` | `tf2_msgs/TFMessage` | `Position` | 로봇 위치 (base_link → map) |
| `/odom` | `nav_msgs/Odometry` | `Velocity` | 로봇 속도 |
| `/robot_status` | `std_msgs/String` | `RobotState.status` | 상태 코드 |

### SAP → ROS2

| SAP Type | ROS2 Topic | ROS2 Type | 설명 |
|----------|------------|-----------|------|
| `MotionCommand` | `/cmd_vel` | `geometry_msgs/Twist` | 속도 명령 |
| `RecoveryCommand` | `/sap/recovery` | `sap_msgs/Recovery` | 복구 명령 |
| `TransitTicket` | `/sap/ticket` | `sap_msgs/Ticket` | 할당된 티켓 |

---

## 커스텀 메시지 정의

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

## 변환 로직

### Position 변환

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

### Velocity 변환

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

## 구현 요구사항

1. **노드 구성**: `sap_bridge_node` (Lifecycle Node)
2. **QoS**: Sensor 데이터는 Best Effort, 명령은 Reliable
3. **타임스탬프**: ROS2 Time → Unix Nanoseconds 변환
4. **프레임 ID**: `map` 프레임 기준 좌표 사용

---

## 다음 단계

1. `sap_msgs` ROS2 패키지 생성
2. `sap_bridge` C++/Python 노드 구현
3. Gazebo 시뮬레이션 테스트
