# SAP 도메인 프로파일

**버전**: 1.0  
**작성일**: 2025-12-08  

---

## 개요

도메인 프로파일은 특정 환경에 최적화된 SAP 파라미터 집합입니다.
각 도메인은 고유한 물리적, 운영적, 보안적 요구사항을 가집니다.

---

## 프로파일 요약

| 프로파일 | 환경 | VTS 해상도 | 충돌 임계값 | 보안 레벨 |
|----------|------|-----------|------------|----------|
| **WAREHOUSE** | 물류 창고 | 1.0m | 0.15m | Medium |
| **FAB** | 반도체 공장 | 0.5m | 0.05m | High |
| **HOSPITAL** | 병원 | 0.8m | 0.20m | High |

---

## WAREHOUSE (물류 창고)

### 환경 특성

- 넓은 통로 (2.5m+)
- 고속 운반 (2+ m/s)
- 고밀도 로봇 (50+ 대/Zone)

### 권장 파라미터

```toml
[profile.warehouse]
# VTS 설정
voxel_size_m = 1.0
time_slot_ms = 100
max_vts_per_robot = 50

# 물리 제한
max_velocity_ms = 2.5
max_acceleration_ms2 = 2.0
max_angular_velocity_rads = 2.0
safety_margin_m = 0.2

# 충돌 예측
collision_threshold_m = 0.15
prediction_horizon_s = 3.0
dynamic_horizon = true

# 보안
signature_required = true
replay_window_ms = 10000
```

### 로봇 유형

- `VehicleProfile::amr()` 권장
- Differential drive 최적화

---

## FAB (반도체 공장)

### 환경 특성

- 좁은 통로 (1.2m)
- 초정밀 이동
- 클린룸 환경 (진동 최소화)

### 권장 파라미터

```toml
[profile.fab]
# VTS 설정 (고해상도)
voxel_size_m = 0.5
time_slot_ms = 50
max_vts_per_robot = 100

# 물리 제한 (엄격)
max_velocity_ms = 0.8
max_acceleration_ms2 = 0.5
max_jerk_ms3 = 2.0
safety_margin_m = 0.1

# 충돌 예측 (민감)
collision_threshold_m = 0.05
prediction_horizon_s = 5.0
rollback_threshold_m = 0.03

# 보안 (최고)
signature_required = true
replay_window_ms = 3000
sybil_check = true
```

### 특수 요구사항

- **PTP 동기화 필수** (NTP 불가)
- **진동 억제**: 저크 제한 엄격
- **클린룸 인증**: 로봇 등록 시 검증

---

## HOSPITAL (병원)

### 환경 특성

- 사람 혼재 환경
- 동적 장애물 빈번
- 조용한 운행 필수

### 권장 파라미터

```toml
[profile.hospital]
# VTS 설정
voxel_size_m = 0.8
time_slot_ms = 200
max_vts_per_robot = 30

# 물리 제한 (안전 우선)
max_velocity_ms = 1.0
max_acceleration_ms2 = 0.8
safety_margin_m = 0.5  # 사람 고려

# 충돌 예측 (보수적)
collision_threshold_m = 0.20
prediction_horizon_s = 4.0
human_detection = true

# 보안
signature_required = true
patient_area_restriction = true
```

### 특수 요구사항

- **사람 감지**: LiDAR + 카메라 융합
- **소음 제한**: 50dB 이하
- **긴급 정지**: 반응 시간 < 100ms

---

## 파라미터 충돌 시 우선순위

1. **안전** > 성능 > 처리량
2. 도메인 프로파일 > 글로벌 설정 > 기본값
3. 동적 조정은 범위 내에서만 허용

---

## 프로파일 확장

새 도메인 프로파일 추가 시:

```rust
impl DomainProfile {
    pub fn custom(name: &str) -> ProfileBuilder {
        ProfileBuilder::new(name)
            .base(Self::warehouse())  // 기본 프로파일 상속
    }
}
```
