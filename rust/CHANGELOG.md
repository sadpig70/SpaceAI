# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-12-07

### Added

#### Core (sap-core)

- `Position`, `Velocity`, `Acceleration` 3D 벡터 타입
- `RobotState`, `WorldState` 상태 타입
- `VoxelTimeSlot`, `TransitTicket`, `Bid` 경제 타입
- `DeltaTickPacket`, `RollbackFrame` 패킷 타입
- `ValidationFrame`, `ValidationResult` 검증 타입
- 72개 단위 테스트

#### Physics (sap-physics)

- `PhysicsValidator` - 물리 제약 검증
- `KinematicsChecker` - 속도/가속도/저크 제한
- `CollisionPredictor` - 충돌 예측
- `CommandGate` - 정책 기반 명령 필터
- 26개 단위 테스트

#### Network (sap-network)

- `StateComparator` - 예측-실제 상태 비교
- `RollbackManager` - 스냅샷 기반 롤백
- `FailsafeManager` - 장애 대응 모드 전환
- 18개 단위 테스트

#### Economy (sap-economy)

- `VickreyAuction` - 제2가 밀봉 입찰 경매
- `PricingEngine` - 수요 기반 동적 가격
- `TicketManager` - 티켓 발행/검증/만료
- 19개 단위 테스트

#### Edge (sap-edge)

- `EdgeRuntime` - L2+L3+L4 통합 런타임
- 10개 통합 테스트

#### Robot (sap-robot)

- `RobotStateManager` - 로봇 상태 추적/예측
- `CommandBuilder` - 빌더 패턴 명령 생성
- `TicketRequester` - 티켓 요청/관리
- 20개 단위 테스트

#### Physvisor (sap-physvisor)

- `ZoneManager` - Zone 경계 관리
- `RobotRegistry` - 로봇 등록/상태 관리
- `SimulationEngine` - 틱 기반 시뮬레이션
- 16개 단위 테스트

#### Cloud (sap-cloud)

- `VtsAllocator` - VTS 할당/충돌 감지
- `GlobalStateAggregator` - Zone별 상태 집계
- 12개 단위 테스트

#### Benchmark (sap-bench)

- EdgeRuntime 처리량 벤치마크
- SimulationEngine 성능 벤치마크
- Criterion 기반 HTML 리포트

#### CI/CD

- GitHub Actions 워크플로우 (빌드/테스트/Clippy)
- 보안 감사 자동화 (cargo-audit)
- rustdoc 자동 배포

### Technical Details

- Rust 2021 Edition
- Workspace 구조 (9개 크레이트)
- 193개 총 테스트
- Clippy `-D warnings` 통과
