# SAP Quick Start Guide

**목표**: 5분 내에 SAP Warehouse Demo 실행하기

---

## 사전 요구사항

### 필수

1. **Rust 1.70 이상**
   - 설치 확인: `rustc --version`
   - 미설치 시: <https://rustup.rs/> 에서 설치

2. **Git**
   - 설치 확인: `git --version`

### 권장

- **운영체제**: Windows 10+, Linux, macOS
- **메모리**: 4GB 이상
- **디스크**: 1GB 여유 공간

---

## 설치 방법

### Step 1: Rust 설치 (미설치 시)

#### Windows

```powershell
# PowerShell에서 실행
Invoke-WebRequest -Uri https://win.rustup.rs/ -OutFile rustup-init.exe
.\rustup-init.exe
```

#### Linux/macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### 설치 확인

```bash
rustc --version
# 출력 예: rustc 1.75.0 (82e1608df 2023-12-21)
```

### Step 2: 프로젝트 클론

```bash
git clone https://github.com/yourusername/SpaceAI.git
cd SpaceAI
```

### Step 3: 빌드 (릴리스 모드)

```bash
cd rust
cargo build --release
```

**예상 소요 시간**: 1-3분 (첫 빌드)

**예상 출력**:

```
   Compiling sap-core v2.0.0
   Compiling sap-physics v2.0.0
   ...
   Compiling sap-examples v0.1.0
    Finished release [optimized] target(s) in 3.39s
```

---

## 데모 실행

### 방법 1: Cargo Run (권장)

```bash
cd rust
cargo run --release --bin warehouse_demo
```

### 방법 2: 직접 실행

```bash
cd rust
./target/release/warehouse_demo      # Linux/macOS
.\target\release\warehouse_demo.exe  # Windows
```

### 방법 3: 모든 테스트 실행

```bash
cd rust
cargo test --all --release
```

---

## 예상 출력

### 시작 메시지

```
=== SAP Warehouse Demo ===
Robots: 5, Tasks: 20, Duration: 60s
```

### 실행 로그

```
[00010] VTS: Robot #2 → Task #0 (3.2m)
[00010] VTS: Robot #1 → Task #1 (4.2m)
[00010] VTS: Robot #5 → Task #2 (3.5m)
[00010] VTS: Robot #3 → Task #3 (6.6m)
[00010] VTS: Robot #4 → Task #4 (4.4m)

[00022] ✅ Task #1 done by R#4
[00049] ✅ Task #4 done by R#2
[00062] ✅ Task #3 done by R#1
[00065] ✅ Task #2 done by R#3
[00065] ✅ Task #0 done by R#5

[00070] 🔄 Handoff: R#2 at boundary
[00076] 🔄 Handoff: R#2 at boundary

...
```

### ASCII 맵 (10틱마다 출력)

```
╔════════════════════╗
║          |         ║
║   3      |         ║
║          |     4   ║
║    5     |         ║
║          |         ║
║     1    |    2    ║
║          |         ║
╚════════════════════╝
  Zone A    |    Zone B
```

- 숫자 = 로봇 ID
- `|` = Zone 경계

### 최종 메트릭

```
🎉 All tasks completed!

==================================================
📊 Final Metrics
==================================================
Tasks Completed:  20/20
Throughput:       0.815 tasks/sec
Allocations:      20
Handoffs:         27
Collisions:       3
Collision Rate:   15.0%
Elapsed Time:     24.5s
==================================================
```

**의미**:

- **Tasks Completed**: 20/20 태스크 완료 (100%)
- **Throughput**: 초당 0.815개 태스크 처리
- **Handoffs**: Cross-Zone 인수인계 27회
- **Collisions**: 충돌 감지 3회 (15%)
- **Elapsed Time**: 실제 실행 24.5초 (목표 60초보다 빠름)

---

## 트러블슈팅

### 문제 1: `rustc: command not found`

**원인**: Rust가 설치되지 않음

**해결**:

```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### 문제 2: `error: linker 'cc' not found`

**원인**: C 컴파일러 미설치 (Linux)

**해결**:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora/RHEL
sudo yum install gcc

# macOS
xcode-select --install
```

### 문제 3: 빌드가 느림

**원인**: Debug 모드로 빌드

**해결**: `--release` 플래그 사용

```bash
cargo build --release  # ← --release 필수!
```

### 문제 4: `error: package collision` 메시지

**원인**: workspace 의존성 문제

**해결**:

```bash
cargo clean
cargo build --release
```

### 문제 5: Windows에서 바이너리 실행 안 됨

**원인**: 백신 프로그램이 차단

**해결**:

1. `target/release` 폴더를 백신 예외에 추가
2. 또는 `cargo run` 사용

---

## 추가 실험

### 벤치마크 실행

```bash
cd rust
cargo bench --bench edge_runtime
cargo bench --bench simulation
```

**예상 출력**:

```
EdgeRuntime::auction/bids/100
                        time:   [7.99 µs 8.80 µs 10.24 µs]
                        thrpt:  [9.77 Melem/s 11.37 Melem/s 12.51 Melem/s]

SimulationEngine::step/robots/500
                        time:   [3.09 ms 3.24 ms 3.50 ms]
                        thrpt:  [142.81 Kelem/s 154.23 Kelem/s 161.63 Kelem/s]
```

### 테스트 실행

```bash
cd rust
cargo test --all --release
```

**예상 결과**: `test result: ok. 226 passed; 0 failed`

### API 문서 열기

```bash
cd rust
cargo doc --open
```

브라우저가 열리며 Rust API 문서를 볼 수 있습니다.

---

## 다음 단계

### 1. 명세서 읽기

[SAP_2.3_Specification.md](SAP_2.3_Specification.md)에서 전체 프로토콜을 확인하세요.

특히:

- §3: 시스템 아키텍처 (5계층)
- §4: 핵심 개념 (VTS, TransitTicket)
- §6: 핵심 알고리즘 (Vickrey, PredictiveSync, Rollback)

### 2. 코드 탐색

```bash
cd rust/examples
cat warehouse_demo.rs  # 데모 코드 읽기
```

주요 파일:

- `warehouse_demo.rs`: Standalone 시뮬레이션
- `crates/sap-edge/src/runtime.rs`: Edge 런타임
- `crates/sap-economy/src/auction/vickrey.rs`: Vickrey 경매

### 3. 기여하기

이슈를 등록하거나 Pull Request를 보내주세요!

- [GitHub Issues](https://github.com/yourusername/SpaceAI/issues)
- [Contributing Guide](../CONTRIBUTING.md)

---

## FAQ

### Q1: Demo가 60초가 아니라 24초에 끝나는데?

**A**: Demo는 "60초 동안 실행" 또는 "20개 태스크 완료 시" 중 먼저 도달하는 조건으로 종료됩니다. 로봇들이 효율적으로 작동하여 24.5초에 모든 태스크를 완료했습니다.

### Q2: 충돌률 15%는 괜찮은 건가요?

**A**: Demo는 간단한 충돌 감지만 구현했습니다. 실제 배포에서는 경로 계획 알고리즘(RRT*, A*)과 통합하여 충돌률을 <1%로 낮출 수 있습니다.

### Q3: 실제 로봇에서 실행할 수 있나요?

**A**: 현재는 시뮬레이션 데모입니다. 실제 로봇 통합을 위해서는:

- ROS2 Bridge 구현 (설계 완료, [문서](integration/ROS2_Bridge.md))
- VDA5050 Adapter 구현 (설계 완료, [문서](integration/VDA5050_Mapping.md))
- GPS/IMU 센서 통합

### Q4: Windows에서 실행이 느린데?

**A**: 반드시 `--release` 플래그를 사용하세요. Debug 빌드는 10-100배 느립니다.

```bash
# 느림 ❌
cargo run --bin warehouse_demo

# 빠름 ✅
cargo run --release --bin warehouse_demo
```

### Q5: 5대보다 많은 로봇으로 테스트하려면?

**A**: `warehouse_demo.rs`의 상수를 수정하세요:

```rust
const NUM_ROBOTS: usize = 10;  // 5 → 10으로 변경
const NUM_TASKS: usize = 40;   // 20 → 40으로 변경
```

재빌드:

```bash
cargo build --release
cargo run --release --bin warehouse_demo
```

---

## 성공 확인

다음 출력이 보이면 성공입니다:

```
✅ Tasks Completed: 20/20
✅ Throughput: 0.8+ tasks/sec
✅ Handoffs: 20+ (Cross-Zone 동작)
✅ Elapsed Time: 60초 이내
```

🎉 **축하합니다! SAP를 성공적으로 실행했습니다!**

---

**다음**: [명세서](SAP_2.3_Specification.md) | [프로젝트 상태](Project_Status_Report.md) | [기여하기](../CONTRIBUTING.md)
