//! Cross-Zone 핸드오프 모듈
//!
//! 로봇이 한 Zone에서 다른 Zone으로 이동할 때의 인수인계 프로토콜을 정의합니다.
//!
//! # 핸드오프 절차 (AI 평가 반영)
//!
//! 1. **사전 할당**: 로봇이 Zone 경계에 접근하면 목표 Zone의 VTS를 미리 예약
//! 2. **핸드오프 요청**: 현재 Edge가 목표 Edge에게 로봇 인수인계 요청
//! 3. **상태 전송**: 로봇 상태 및 티켓 정보 전송
//! 4. **확인 응답**: 목표 Edge가 인수 완료 확인
//! 5. **해제**: 이전 Edge가 로봇 관리 해제
//!
//! PPR 매핑: AI_make_CrossZoneHandoff

mod protocol;

pub use protocol::{
    HandoffRequest, HandoffResponse, HandoffState, HandoffStatus, PredictiveAllocation,
    ZoneBoundary,
};
