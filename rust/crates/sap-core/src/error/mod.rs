//! 에러 타입 정의 모듈

use thiserror::Error;

/// SAP 통합 에러 타입
#[derive(Error, Debug)]
pub enum SapError {
    // === 검증 에러 ===
    #[error("Velocity limit exceeded: {actual} > {limit}")]
    VelocityExceeded { actual: f32, limit: f32 },

    #[error("Acceleration limit exceeded: {actual} > {limit}")]
    AccelerationExceeded { actual: f32, limit: f32 },

    #[error("Jerk limit exceeded: {actual} > {limit}")]
    JerkExceeded { actual: f32, limit: f32 },

    #[error("Collision predicted with obstacle {obstacle_id} in {time:.2}s")]
    CollisionPredicted { obstacle_id: u64, time: f32 },

    #[error("Geofence violation: geofence_id={geofence_id}")]
    GeofenceViolation { geofence_id: u32 },

    // === 티켓 에러 ===
    #[error("Invalid ticket: {ticket_id}")]
    InvalidTicket { ticket_id: u128 },

    #[error("Ticket expired: {ticket_id}")]
    TicketExpired { ticket_id: u128 },

    #[error("VTS violation: vts_id={vts_id}")]
    VTSViolation { vts_id: u64 },

    // === 네트워크 에러 ===
    #[error("Network disconnected")]
    NetworkDisconnected,

    #[error("Edge node unavailable")]
    EdgeUnavailable,

    #[error("PTP sync lost")]
    PTPSyncLost,

    #[error("Packet parse error: {0}")]
    PacketParseError(String),

    // === 경제 에러 ===
    #[error("Insufficient stake: required={required}, actual={actual}")]
    InsufficientStake { required: u64, actual: u64 },

    #[error("Low reputation: score={score}, required={required}")]
    LowReputation { score: u32, required: u32 },

    #[error("Auction failed: {reason}")]
    AuctionFailed { reason: String },

    #[error("Bid hash mismatch")]
    BidHashMismatch,

    // === 보안 에러 ===
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Replay attack detected: nonce={nonce}")]
    ReplayAttackDetected { nonce: u64 },

    #[error("Invalid sequence: expected={expected}, actual={actual}")]
    InvalidSequence { expected: u64, actual: u64 },

    #[error("Message expired: age={age_ms}ms, max={max_ms}ms")]
    MessageExpired { age_ms: u64, max_ms: u64 },

    #[error("Unknown signer: robot_id={robot_id}")]
    UnknownSigner { robot_id: u64 },

    // === 핸드오프 에러 ===
    #[error("Handoff rejected: {reason}")]
    HandoffRejected { reason: String },

    #[error("Zone capacity exceeded: zone_id={zone_id}")]
    ZoneCapacityExceeded { zone_id: u32 },

    // === 일반 에러 ===
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result 타입 별칭
pub type Result<T> = std::result::Result<T, SapError>;

impl SapError {
    /// 에러가 재시도 가능한지 여부
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SapError::NetworkDisconnected | SapError::EdgeUnavailable | SapError::PTPSyncLost
        )
    }

    /// 에러가 치명적인지 여부
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            SapError::CollisionPredicted { .. } | SapError::GeofenceViolation { .. }
        )
    }

    /// 에러 코드 반환 (로깅용)
    pub fn error_code(&self) -> u32 {
        match self {
            SapError::VelocityExceeded { .. } => 1001,
            SapError::AccelerationExceeded { .. } => 1002,
            SapError::JerkExceeded { .. } => 1003,
            SapError::CollisionPredicted { .. } => 1004,
            SapError::GeofenceViolation { .. } => 1005,
            SapError::InvalidTicket { .. } => 2001,
            SapError::TicketExpired { .. } => 2002,
            SapError::VTSViolation { .. } => 2003,
            SapError::NetworkDisconnected => 3001,
            SapError::EdgeUnavailable => 3002,
            SapError::PTPSyncLost => 3003,
            SapError::PacketParseError(_) => 3004,
            SapError::InsufficientStake { .. } => 4001,
            SapError::LowReputation { .. } => 4002,
            SapError::AuctionFailed { .. } => 4003,
            SapError::BidHashMismatch => 4004,
            // 보안 에러: 5xxx
            SapError::SignatureVerificationFailed => 5001,
            SapError::ReplayAttackDetected { .. } => 5002,
            SapError::InvalidSequence { .. } => 5003,
            SapError::MessageExpired { .. } => 5004,
            SapError::UnknownSigner { .. } => 5005,
            // 핸드오프 에러: 6xxx
            SapError::HandoffRejected { .. } => 6001,
            SapError::ZoneCapacityExceeded { .. } => 6002,
            // 일반 에러: 9xxx
            SapError::SerializationError(_) => 9001,
            SapError::InternalError(_) => 9999,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SapError::VelocityExceeded {
            actual: 10.0,
            limit: 5.0,
        };
        assert!(err.to_string().contains("Velocity limit exceeded"));
    }

    #[test]
    fn test_error_is_retryable() {
        assert!(SapError::NetworkDisconnected.is_retryable());
        assert!(!SapError::CollisionPredicted {
            obstacle_id: 1,
            time: 0.5
        }
        .is_retryable());
    }

    #[test]
    fn test_error_is_fatal() {
        assert!(SapError::CollisionPredicted {
            obstacle_id: 1,
            time: 0.5
        }
        .is_fatal());
        assert!(!SapError::NetworkDisconnected.is_fatal());
    }

    #[test]
    fn test_error_code() {
        assert_eq!(
            SapError::VelocityExceeded {
                actual: 0.0,
                limit: 0.0
            }
            .error_code(),
            1001
        );
        assert_eq!(SapError::NetworkDisconnected.error_code(), 3001);
    }
}
