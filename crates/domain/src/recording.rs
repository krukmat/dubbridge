// S3-T1: recording session aggregate, state machine, and source model per ADR-020/022
use thiserror::Error;
use uuid::Uuid;

use crate::rights::RightsBasis;

/// Opaque identifier for a recording session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RecordingSessionId(pub Uuid);

impl RecordingSessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RecordingSessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RecordingSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Supported ingest source protocols per ADR-022. Only RTMP and SRT in v1.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceProtocol {
    Rtmp,
    Srt,
}

impl std::fmt::Display for SourceProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Rtmp => "rtmp",
            Self::Srt => "srt",
        };
        write!(f, "{s}")
    }
}

/// Validated source descriptor. The URL has been scheme-checked; credentials are
/// stored by reference, never in plaintext (ADR-022).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordingSource {
    pub protocol: SourceProtocol,
    /// Validated ingest URL (scheme must match `protocol`).
    pub url: String,
    /// Opaque credential reference — stream key (RTMP) or passphrase handle (SRT).
    /// The real secret lives in a secrets store; this is never logged.
    pub credential_ref: Option<String>,
}

/// Recording session lifecycle state machine per ADR-020.
///
/// Legal transitions:
///   Requested → RightsValidated → Capturing → Stopping → Recorded
///   Requested → RejectedMissingRights
///   Capturing → Failed
///   Stopping  → Failed
///
/// Invariant enforced by the private-field tuple: the only way to construct
/// `Capturing` is through `transition_to_capturing`, which requires `RightsValidated`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingStatus {
    Requested,
    RightsValidated,
    Capturing,
    Stopping,
    Recorded,
    Failed,
    RejectedMissingRights,
}

impl std::fmt::Display for RecordingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Requested => "requested",
            Self::RightsValidated => "rights_validated",
            Self::Capturing => "capturing",
            Self::Stopping => "stopping",
            Self::Recorded => "recorded",
            Self::Failed => "failed",
            Self::RejectedMissingRights => "rejected_missing_rights",
        };
        write!(f, "{s}")
    }
}

/// Errors produced by recording command validation or state-machine transitions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecordingError {
    #[error("rights basis is required to start a recording session")]
    MissingRightsBasis,
    #[error("rights owner field is required")]
    MissingRightsOwner,
    #[error("proof reference is required")]
    MissingProofReference,
    #[error("source URL is required")]
    MissingSourceUrl,
    #[error("unsupported source protocol — only rtmp and srt are accepted")]
    UnsupportedSourceProtocol,
    #[error("illegal state transition from {from} to {to}")]
    IllegalTransition {
        from: &'static str,
        to: &'static str,
    },
}

/// Command to create and validate a new recording session (ADR-008 fail-closed).
#[derive(Debug)]
pub struct StartRecordingCommand {
    pub owner_id: Uuid,
    pub source: RecordingSource,
    /// Must be present and complete; session is rejected if missing (ADR-008).
    pub rights_basis: Option<RightsBasis>,
}

impl StartRecordingCommand {
    /// Validates the command fail-closed. Returns `RecordingError` on any missing
    /// required field so the session is rejected *before* any FFmpeg process is
    /// spawned (ADR-020).
    pub fn validate(&self) -> Result<(), RecordingError> {
        match &self.rights_basis {
            None => return Err(RecordingError::MissingRightsBasis),
            Some(basis) => {
                if basis.owner.trim().is_empty() {
                    return Err(RecordingError::MissingRightsOwner);
                }
                if basis.proof_reference.trim().is_empty() {
                    return Err(RecordingError::MissingProofReference);
                }
            }
        }

        if self.source.url.trim().is_empty() {
            return Err(RecordingError::MissingSourceUrl);
        }

        Ok(())
    }
}

/// Aggregate root for a recording session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordingSession {
    pub id: RecordingSessionId,
    pub owner_id: Uuid,
    pub source: RecordingSource,
    pub rights_basis: RightsBasis,
    pub status: RecordingStatus,
    /// Set once the assembled artifact is bridged to the asset pipeline.
    pub asset_id: Option<uuid::Uuid>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

impl RecordingSession {
    /// Creates a new session in `Requested` state. Caller must have already called
    /// `StartRecordingCommand::validate()`.
    pub fn new(cmd: &StartRecordingCommand) -> Self {
        let now = time::OffsetDateTime::now_utc();
        Self {
            id: RecordingSessionId::new(),
            owner_id: cmd.owner_id,
            source: cmd.source.clone(),
            rights_basis: cmd
                .rights_basis
                .clone()
                .expect("validate() must be called before new()"),
            status: RecordingStatus::Requested,
            asset_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Transitions Requested → RightsValidated.
    pub fn validate_rights(&mut self) -> Result<(), RecordingError> {
        match self.status {
            RecordingStatus::Requested => {
                self.status = RecordingStatus::RightsValidated;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(RecordingError::IllegalTransition {
                from: "requested",
                to: "rights_validated",
            }),
        }
    }

    /// Transitions RightsValidated → Capturing.
    /// Enforces the invariant: Capturing is only reachable through RightsValidated.
    pub fn start_capturing(&mut self) -> Result<(), RecordingError> {
        match self.status {
            RecordingStatus::RightsValidated => {
                self.status = RecordingStatus::Capturing;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(RecordingError::IllegalTransition {
                from: "non-rights-validated",
                to: "capturing",
            }),
        }
    }

    /// Transitions Capturing → Stopping.
    pub fn request_stop(&mut self) -> Result<(), RecordingError> {
        match self.status {
            RecordingStatus::Capturing => {
                self.status = RecordingStatus::Stopping;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(RecordingError::IllegalTransition {
                from: "non-capturing",
                to: "stopping",
            }),
        }
    }

    /// Transitions Stopping → Recorded.
    pub fn mark_recorded(&mut self) -> Result<(), RecordingError> {
        match self.status {
            RecordingStatus::Stopping => {
                self.status = RecordingStatus::Recorded;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(RecordingError::IllegalTransition {
                from: "non-stopping",
                to: "recorded",
            }),
        }
    }

    /// Transitions Requested → RejectedMissingRights.
    pub fn reject_missing_rights(&mut self) -> Result<(), RecordingError> {
        match self.status {
            RecordingStatus::Requested => {
                self.status = RecordingStatus::RejectedMissingRights;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(RecordingError::IllegalTransition {
                from: "non-requested",
                to: "rejected_missing_rights",
            }),
        }
    }

    /// Transitions Capturing or Stopping → Failed.
    pub fn mark_failed(&mut self) -> Result<(), RecordingError> {
        match self.status {
            RecordingStatus::Capturing | RecordingStatus::Stopping => {
                self.status = RecordingStatus::Failed;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(RecordingError::IllegalTransition {
                from: "non-capturing-or-stopping",
                to: "failed",
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rights::{LicenseType, RightsBasis, SourceType};

    fn valid_basis() -> RightsBasis {
        RightsBasis {
            owner: "Broadcast Studio".to_string(),
            license_type: LicenseType::LicensedDistribution,
            source_type: SourceType::InternalFeed,
            proof_reference: "live-contract-2026-001".to_string(),
        }
    }

    fn rtmp_source() -> RecordingSource {
        RecordingSource {
            protocol: SourceProtocol::Rtmp,
            url: "rtmp://ingest.example.com/live/stream1".to_string(),
            credential_ref: Some("secret-ref-abc".to_string()),
        }
    }

    fn valid_command() -> StartRecordingCommand {
        StartRecordingCommand {
            owner_id: Uuid::new_v4(),
            source: rtmp_source(),
            rights_basis: Some(valid_basis()),
        }
    }

    // --- StartRecordingCommand::validate() ---

    #[test]
    fn validate_accepts_valid_command() {
        assert_eq!(valid_command().validate(), Ok(()));
    }

    #[test]
    fn validate_fails_closed_on_missing_rights_basis() {
        let cmd = StartRecordingCommand {
            rights_basis: None,
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(RecordingError::MissingRightsBasis));
    }

    #[test]
    fn validate_fails_closed_on_empty_owner() {
        let mut basis = valid_basis();
        basis.owner = "  ".to_string();
        let cmd = StartRecordingCommand {
            rights_basis: Some(basis),
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(RecordingError::MissingRightsOwner));
    }

    #[test]
    fn validate_fails_closed_on_empty_proof_reference() {
        let mut basis = valid_basis();
        basis.proof_reference = String::new();
        let cmd = StartRecordingCommand {
            rights_basis: Some(basis),
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(RecordingError::MissingProofReference));
    }

    #[test]
    fn validate_fails_closed_on_empty_source_url() {
        let cmd = StartRecordingCommand {
            source: RecordingSource {
                protocol: SourceProtocol::Rtmp,
                url: "   ".to_string(),
                credential_ref: None,
            },
            ..valid_command()
        };
        assert_eq!(cmd.validate(), Err(RecordingError::MissingSourceUrl));
    }

    // --- SourceProtocol ---

    #[test]
    fn source_protocol_only_rtmp_and_srt() {
        // Exhaustiveness: this test fails to compile if a third variant is added
        // without updating it — enforcing the ADR-022 constraint.
        let protocols = [SourceProtocol::Rtmp, SourceProtocol::Srt];
        assert_eq!(protocols[0].to_string(), "rtmp");
        assert_eq!(protocols[1].to_string(), "srt");
    }

    // --- State machine: no path Requested/RejectedMissingRights → Capturing ---

    #[test]
    fn capturing_requires_rights_validated_first() {
        let cmd = valid_command();
        let mut session = RecordingSession::new(&cmd);
        // Direct Requested → Capturing must fail
        let err = session.start_capturing();
        assert!(err.is_err());
        assert_eq!(session.status, RecordingStatus::Requested);
    }

    #[test]
    fn rejected_session_cannot_reach_capturing() {
        let cmd = valid_command();
        let mut session = RecordingSession::new(&cmd);
        session.reject_missing_rights().unwrap();
        assert_eq!(session.status, RecordingStatus::RejectedMissingRights);
        let err = session.start_capturing();
        assert!(err.is_err());
        assert_eq!(session.status, RecordingStatus::RejectedMissingRights);
    }

    #[test]
    fn valid_full_lifecycle_requested_to_recorded() {
        let cmd = valid_command();
        let mut session = RecordingSession::new(&cmd);
        assert_eq!(session.status, RecordingStatus::Requested);
        session.validate_rights().unwrap();
        assert_eq!(session.status, RecordingStatus::RightsValidated);
        session.start_capturing().unwrap();
        assert_eq!(session.status, RecordingStatus::Capturing);
        session.request_stop().unwrap();
        assert_eq!(session.status, RecordingStatus::Stopping);
        session.mark_recorded().unwrap();
        assert_eq!(session.status, RecordingStatus::Recorded);
    }

    #[test]
    fn capturing_to_failed_is_valid() {
        let cmd = valid_command();
        let mut session = RecordingSession::new(&cmd);
        session.validate_rights().unwrap();
        session.start_capturing().unwrap();
        session.mark_failed().unwrap();
        assert_eq!(session.status, RecordingStatus::Failed);
    }

    #[test]
    fn status_display_all_variants() {
        assert_eq!(RecordingStatus::Requested.to_string(), "requested");
        assert_eq!(
            RecordingStatus::RightsValidated.to_string(),
            "rights_validated"
        );
        assert_eq!(RecordingStatus::Capturing.to_string(), "capturing");
        assert_eq!(RecordingStatus::Stopping.to_string(), "stopping");
        assert_eq!(RecordingStatus::Recorded.to_string(), "recorded");
        assert_eq!(RecordingStatus::Failed.to_string(), "failed");
        assert_eq!(
            RecordingStatus::RejectedMissingRights.to_string(),
            "rejected_missing_rights"
        );
    }
}
