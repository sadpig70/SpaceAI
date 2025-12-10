# SAP 2.0 - Space AI Protocol

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-193%20passed-brightgreen.svg)](#testing)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**SAP (Space AI Protocol)** 는 자율 이동 로봇(AMR) 군집 제어를 위한 실시간 공간 거래 프로토콜입니다.

## 🏗️ 아키텍처

```
┌──────────────────────────────────────────────────────────────┐
│                      SAP v1.2 Stack                          │
├──────────────────────────────────────────────────────────────┤
│  L5: sap-cloud    │ VTS 할당, 글로벌 상태 집계              │
├───────────────────┼──────────────────────────────────────────┤
│  L4: sap-economy  │ S-MEV 경매, 동적 가격, 티켓 관리        │
├───────────────────┼──────────────────────────────────────────┤
│  L3: sap-network  │ PredictiveSync, 롤백, 장애 대응         │
├───────────────────┼──────────────────────────────────────────┤
│  L2: sap-physics  │ TrustOS 물리 검증, 명령 게이트          │
├───────────────────┼──────────────────────────────────────────┤
│  L1: sap-core     │ 핵심 타입, 패킷, 검증 프레임            │
└──────────────────────────────────────────────────────────────┘
```

## 📦 크레이트

| 크레이트 | 설명 | 테스트 |
|----------|------|--------|
| `sap-core` | 핵심 타입 (Position, RobotState, Packet) | 72 |
| `sap-physics` | 물리 검증 (PhysicsValidator, CommandGate) | 26 |
| `sap-network` | 동기화 (StateComparator, RollbackManager) | 18 |
| `sap-economy` | 경제 시스템 (VickreyAuction, PricingEngine) | 19 |
| `sap-edge` | 통합 런타임 (EdgeRuntime) | 10 |
| `sap-robot` | 로봇 SDK (RobotStateManager, CommandBuilder) | 20 |
| `sap-physvisor` | Zone 관리 (SimulationEngine) | 16 |
| `sap-cloud` | 클라우드 (VtsAllocator, GlobalStateAggregator) | 12 |
| `sap-bench` | 성능 벤치마크 | - |

## 🚀 빠른 시작

### 설치

```toml
[dependencies]
sap-core = { path = "crates/sap-core" }
sap-edge = { path = "crates/sap-edge" }
```

### 기본 사용법

```rust
use sap_edge::EdgeRuntime;
use sap_core::types::{Position, Velocity, Acceleration};
use sap_physics::command::MotionCommand;

fn main() {
    // Edge 런타임 생성
    let mut runtime = EdgeRuntime::new(1);
    
    // 틱 진행
    runtime.tick(1_000_000_000);
    
    // 명령 처리
    let cmd = MotionCommand {
        robot_id: 42,
        current_position: Position::ORIGIN,
        target_velocity: Velocity::new(1.0, 0.0, 0.0),
        target_acceleration: Acceleration::ZERO,
        ticket_id: 1,
    };
    
    let result = runtime.process_command(&cmd, 1_000_000_000);
    println!("Result: {:?}", result);
}
```

### 경매 시스템

```rust
use sap_edge::EdgeRuntime;

fn main() {
    let mut runtime = EdgeRuntime::new(1);
    
    // 입찰 제출
    runtime.submit_bid(1, 100, 500, 1_000_000_000).unwrap();
    runtime.submit_bid(2, 100, 800, 2_000_000_000).unwrap();
    
    // 경매 결산 (Vickrey: 2등 가격)
    let result = runtime.settle_auction(100, 3_000_000_000).unwrap();
    
    println!("Winner: {}, Price: {}", result.winner_id, result.winning_price);
    // Winner: 2, Price: 500
}
```

## 🧪 테스트

```bash
# 전체 테스트
cargo test --all

# 특정 크레이트 테스트
cargo test -p sap-core
cargo test -p sap-edge

# Clippy 검사
cargo clippy --all -- -D warnings
```

## 📊 벤치마크

```bash
# 전체 벤치마크
cargo bench -p sap-bench

# 특정 벤치마크
cargo bench -p sap-bench -- edge_runtime
cargo bench -p sap-bench -- simulation
```

### 벤치마크 항목

- **EdgeRuntime**
  - `process_command`: 명령 처리 처리량 (1/10/100 로봇)
  - `tick`: 틱 처리 시간
  - `auction`: 경매 결산 (10/50/100 입찰)

- **SimulationEngine**
  - `step`: 시뮬레이션 스텝 (10/100/500 로봇)
  - `collision`: 충돌 감지 (밀집/희소)
  - `zone`: Zone 업데이트 (100 로봇)

## 📚 문서

```bash
# API 문서 생성
cargo doc --no-deps --all-features --open
```

## 🔧 개발

### 디렉토리 구조

```
rust/
├── Cargo.toml              # 워크스페이스 설정
├── crates/
│   ├── sap-core/           # L1 핵심 타입
│   ├── sap-physics/        # L2 물리 검증
│   ├── sap-network/        # L3 동기화
│   ├── sap-economy/        # L4 경제
│   ├── sap-edge/           # 통합 런타임
│   ├── sap-robot/          # 로봇 SDK
│   ├── sap-physvisor/      # Zone 관리
│   ├── sap-cloud/          # 클라우드
│   └── sap-bench/          # 벤치마크
├── rustfmt.toml            # 포맷팅 규칙
└── clippy.toml             # Lint 규칙
```

### CI/CD

GitHub Actions 워크플로우:

- `rust-ci.yml`: 빌드 + 테스트 + Clippy + 커버리지
- `security-audit.yml`: 의존성 보안 감사
- `documentation.yml`: rustdoc 자동 배포

## 📄 라이선스

MIT License - [LICENSE](LICENSE) 참조

## 👥 기여

1. Fork
2. Feature branch 생성 (`git checkout -b feature/amazing`)
3. Commit (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. Pull Request 생성
