// S3-P1: platform ingest domain aggregate, state machine, and command (ADR-025, ADR-008).

use thiserror::Error;
use uuid::Uuid;

use crate::rights::RightsBasis;

/// Supported ingest platforms. YouTube is the v1 connector; others added in later slices.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    YouTube,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::YouTube => write!(f, "youtube"),
        }
    }
}

/// Opaque reference to a platform source (URL or platform-native identifier).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceRef(pub String);

/// Opaque identifier for a platform ingest session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PlatformIngestSessionId(pub Uuid);

impl PlatformIngestSessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PlatformIngestSessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PlatformIngestSessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Platform ingest session lifecycle state machine (ADR-025).
///
/// Legal transitions:
///   Requested → RightsValidated → Resolving → Downloading → Downloaded
///   Requested → RejectedMissingRights
///   Resolving | Downloading → Failed
///
/// Invariant: Resolving is only reachable through RightsValidated.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformIngestStatus {
    Requested,
    RightsValidated,
    Resolving,
    Downloading,
    Downloaded,
    Failed,
    RejectedMissingRights,
}

impl std::fmt::Display for PlatformIngestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Requested => "requested",
            Self::RightsValidated => "rights_validated",
            Self::Resolving => "resolving",
            Self::Downloading => "downloading",
            Self::Downloaded => "downloaded",
            Self::Failed => "failed",
            Self::RejectedMissingRights => "rejected_missing_rights",
        };
        write!(f, "{s}")
    }
}

/// Errors produced by platform ingest command validation or state-machine transitions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlatformIngestError {
    #[error("rights basis is required to start a platform ingest session")]
    MissingRightsBasis,
    #[error("rights owner field is required")]
    MissingRightsOwner,
    #[error("proof reference is required")]
    MissingProofReference,
    /// ADR-025: owner credential reference is mandatory before any bytes are fetched.
    #[error("owner credential reference is required (ADR-025)")]
    MissingCredentialRef,
    #[error("source ref is required")]
    MissingSourceRef,
    #[error("illegal state transition from {from} to {to}")]
    IllegalTransition {
        from: &'static str,
        to: &'static str,
    },
}

/// Command to create and validate a new platform ingest session (ADR-008, ADR-025 fail-closed).
#[derive(Debug)]
pub struct StartPlatformIngestCommand {
    pub owner_id: Uuid,
    pub platform: Platform,
    pub source_ref: SourceRef,
    /// Opaque reference to owner credentials in the secrets store — never plaintext (ADR-025).
    pub credential_ref: Option<String>,
    /// Must be present and complete; session is rejected if missing (ADR-008).
    pub rights_basis: Option<RightsBasis>,
}

impl StartPlatformIngestCommand {
    /// Validates the command fail-closed. Returns `PlatformIngestError` on any missing
    /// required field so the session is rejected before any network IO begins (ADR-025).
    pub fn validate(&self) -> Result<(), PlatformIngestError> {
        match &self.rights_basis {
            None => return Err(PlatformIngestError::MissingRightsBasis),
            Some(basis) => {
                if basis.owner.trim().is_empty() {
                    return Err(PlatformIngestError::MissingRightsOwner);
                }
                if basis.proof_reference.trim().is_empty() {
                    return Err(PlatformIngestError::MissingProofReference);
                }
            }
        }
        if self.credential_ref.is_none() {
            return Err(PlatformIngestError::MissingCredentialRef);
        }
        if self.source_ref.0.trim().is_empty() {
            return Err(PlatformIngestError::MissingSourceRef);
        }
        Ok(())
    }
}

/// Aggregate root for a platform ingest session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlatformIngestSession {
    pub id: PlatformIngestSessionId,
    pub owner_id: Uuid,
    pub platform: Platform,
    pub source_ref: SourceRef,
    /// Stored by reference; never in plaintext (ADR-025).
    pub credential_ref: String,
    pub rights_basis: RightsBasis,
    pub status: PlatformIngestStatus,
    /// Set once the downloaded artifact is bridged to the asset pipeline.
    pub asset_id: Option<Uuid>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

impl PlatformIngestSession {
    /// Creates a new session in `Requested` state. Caller must have already called
    /// `StartPlatformIngestCommand::validate()`.
    pub fn new(cmd: &StartPlatformIngestCommand) -> Self {
        let now = time::OffsetDateTime::now_utc();
        Self {
            id: PlatformIngestSessionId::new(),
            owner_id: cmd.owner_id,
            platform: cmd.platform.clone(),
            source_ref: cmd.source_ref.clone(),
            credential_ref: cmd
                .credential_ref
                .clone()
                .expect("validate() must be called before new()"),
            rights_basis: cmd
                .rights_basis
                .clone()
                .expect("validate() must be called before new()"),
            status: PlatformIngestStatus::Requested,
            asset_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Transitions Requested → RightsValidated.
    pub fn validate_rights(&mut self) -> Result<(), PlatformIngestError> {
        match self.status {
            PlatformIngestStatus::Requested => {
                self.status = PlatformIngestStatus::RightsValidated;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(PlatformIngestError::IllegalTransition {
                from: "requested",
                to: "rights_validated",
            }),
        }
    }

    /// Transitions RightsValidated → Resolving.
    /// Enforces the invariant: Resolving is only reachable through RightsValidated.
    pub fn start_resolving(&mut self) -> Result<(), PlatformIngestError> {
        match self.status {
            PlatformIngestStatus::RightsValidated => {
                self.status = PlatformIngestStatus::Resolving;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(PlatformIngestError::IllegalTransition {
                from: "non-rights-validated",
                to: "resolving",
            }),
        }
    }

    /// Transitions Resolving → Downloading.
    pub fn start_downloading(&mut self) -> Result<(), PlatformIngestError> {
        match self.status {
            PlatformIngestStatus::Resolving => {
                self.status = PlatformIngestStatus::Downloading;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(PlatformIngestError::IllegalTransition {
                from: "non-resolving",
                to: "downloading",
            }),
        }
    }

    /// Transitions Downloading → Downloaded.
    pub fn mark_downloaded(&mut self) -> Result<(), PlatformIngestError> {
        match self.status {
            PlatformIngestStatus::Downloading => {
                self.status = PlatformIngestStatus::Downloaded;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(PlatformIngestError::IllegalTransition {
                from: "non-downloading",
                to: "downloaded",
            }),
        }
    }

    /// Transitions Requested → RejectedMissingRights.
    pub fn reject_missing_rights(&mut self) -> Result<(), PlatformIngestError> {
        match self.status {
            PlatformIngestStatus::Requested => {
                self.status = PlatformIngestStatus::RejectedMissingRights;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(PlatformIngestError::IllegalTransition {
                from: "non-requested",
                to: "rejected_missing_rights",
            }),
        }
    }

    /// Transitions Resolving or Downloading → Failed.
    pub fn mark_failed(&mut self) -> Result<(), PlatformIngestError> {
        match self.status {
            PlatformIngestStatus::Resolving | PlatformIngestStatus::Downloading => {
                self.status = PlatformIngestStatus::Failed;
                self.updated_at = time::OffsetDateTime::now_utc();
                Ok(())
            }
            _ => Err(PlatformIngestError::IllegalTransition {
                from: "non-resolving-or-downloading",
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
            owner: "Content Owner LLC".to_string(),
            license_type: LicenseType::LicensedDistribution,
            source_type: SourceType::AuthorizedS3,
            proof_reference: "contract-2026-yt-001".to_string(),
        }
    }

    fn valid_command() -> StartPlatformIngestCommand {
        StartPlatformIngestCommand {
            owner_id: Uuid::new_v4(),
            platform: Platform::YouTube,
            source_ref: SourceRef("https://youtube.com/watch?v=abc123".to_string()),
            credential_ref: Some("oauth-ref-xyz".to_string()),
            rights_basis: Some(valid_basis()),
        }
    }

    // --- StartPlatformIngestCommand::validate() fail-closed (ADR-008, ADR-025) ---

    #[test]
    fn validate_accepts_valid_command() {
        assert!(valid_command().validate().is_ok());
    }

    #[test]
    fn validate_rejects_missing_rights_basis() {
        let cmd = StartPlatformIngestCommand {
            rights_basis: None,
            ..valid_command()
        };
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn validate_rejects_missing_credential_ref() {
        // ADR-025: owner credential reference is mandatory before any bytes are fetched.
        let cmd = StartPlatformIngestCommand {
            credential_ref: None,
            ..valid_command()
        };
        assert!(cmd.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_source_ref() {
        let cmd = StartPlatformIngestCommand {
            source_ref: SourceRef(String::new()),
            ..valid_command()
        };
        assert!(cmd.validate().is_err());
    }

    // --- State machine: no path to Resolving/Downloading without RightsValidated ---

    #[test]
    fn resolving_requires_rights_validated_first() {
        let cmd = valid_command();
        let mut session = PlatformIngestSession::new(&cmd);
        assert_eq!(session.status, PlatformIngestStatus::Requested);
        assert!(session.start_resolving().is_err());
        assert_eq!(session.status, PlatformIngestStatus::Requested);
    }

    #[test]
    fn rejected_session_cannot_reach_resolving() {
        let cmd = valid_command();
        let mut session = PlatformIngestSession::new(&cmd);
        session.reject_missing_rights().unwrap();
        assert_eq!(session.status, PlatformIngestStatus::RejectedMissingRights);
        assert!(session.start_resolving().is_err());
        assert_eq!(session.status, PlatformIngestStatus::RejectedMissingRights);
    }

    #[test]
    fn valid_full_lifecycle_to_downloaded() {
        let cmd = valid_command();
        let mut session = PlatformIngestSession::new(&cmd);
        session.validate_rights().unwrap();
        assert_eq!(session.status, PlatformIngestStatus::RightsValidated);
        session.start_resolving().unwrap();
        assert_eq!(session.status, PlatformIngestStatus::Resolving);
        session.start_downloading().unwrap();
        assert_eq!(session.status, PlatformIngestStatus::Downloading);
        session.mark_downloaded().unwrap();
        assert_eq!(session.status, PlatformIngestStatus::Downloaded);
    }

    #[test]
    fn resolving_state_can_transition_to_failed() {
        let cmd = valid_command();
        let mut session = PlatformIngestSession::new(&cmd);
        session.validate_rights().unwrap();
        session.start_resolving().unwrap();
        session.mark_failed().unwrap();
        assert_eq!(session.status, PlatformIngestStatus::Failed);
    }

    #[test]
    fn downloading_state_can_transition_to_failed() {
        let cmd = valid_command();
        let mut session = PlatformIngestSession::new(&cmd);
        session.validate_rights().unwrap();
        session.start_resolving().unwrap();
        session.start_downloading().unwrap();
        session.mark_failed().unwrap();
        assert_eq!(session.status, PlatformIngestStatus::Failed);
    }

    #[test]
    fn status_display_all_variants() {
        assert_eq!(PlatformIngestStatus::Requested.to_string(), "requested");
        assert_eq!(
            PlatformIngestStatus::RightsValidated.to_string(),
            "rights_validated"
        );
        assert_eq!(PlatformIngestStatus::Resolving.to_string(), "resolving");
        assert_eq!(PlatformIngestStatus::Downloading.to_string(), "downloading");
        assert_eq!(PlatformIngestStatus::Downloaded.to_string(), "downloaded");
        assert_eq!(PlatformIngestStatus::Failed.to_string(), "failed");
        assert_eq!(
            PlatformIngestStatus::RejectedMissingRights.to_string(),
            "rejected_missing_rights"
        );
    }
}
