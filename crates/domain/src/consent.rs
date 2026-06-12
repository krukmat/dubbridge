// S-110-T1b: voice-consent domain entity per ADR-028
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentError {
    MissingEvidenceRef,
    UnknownScope(String),
    UnknownStatus(String),
}

impl std::fmt::Display for ConsentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEvidenceRef => write!(f, "evidence_ref is required for a consent grant"),
            Self::UnknownScope(s) => write!(f, "unknown consent scope: {s}"),
            Self::UnknownStatus(s) => write!(f, "unknown consent status: {s}"),
        }
    }
}

/// Synthesis scope covered by a consent row. Values must match the
/// voice_consents.scope CHECK constraint in migration 0013.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentScope {
    VoiceClone,
    TtsSynthesis,
}

impl std::fmt::Display for ConsentScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::VoiceClone => "voice_clone",
            Self::TtsSynthesis => "tts_synthesis",
        };
        write!(f, "{s}")
    }
}

impl FromStr for ConsentScope {
    type Err = ConsentError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "voice_clone" => Ok(Self::VoiceClone),
            "tts_synthesis" => Ok(Self::TtsSynthesis),
            other => Err(ConsentError::UnknownScope(other.to_string())),
        }
    }
}

/// Row status. Values must match the voice_consents.status CHECK constraint
/// in migration 0013. Current consent = latest row by happened_at (ADR-028).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    Grant,
    Revoke,
}

impl std::fmt::Display for ConsentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Grant => "grant",
            Self::Revoke => "revoke",
        };
        write!(f, "{s}")
    }
}

impl FromStr for ConsentStatus {
    type Err = ConsentError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grant" => Ok(Self::Grant),
            "revoke" => Ok(Self::Revoke),
            other => Err(ConsentError::UnknownStatus(other.to_string())),
        }
    }
}

/// A single append-only row in the voice_consents ledger (ADR-028).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRow {
    pub id: Uuid,
    pub asset_id: AssetId,
    pub scope: ConsentScope,
    pub status: ConsentStatus,
    /// Opaque reference to consent evidence — never stores bytes inline (ADR-025).
    /// Present on Grant rows; None on Revoke rows.
    pub evidence_ref: Option<String>,
    pub granted_by: Uuid,
    pub happened_at: OffsetDateTime,
}

/// Create a grant row. Fails closed if `evidence_ref` is empty or whitespace (ADR-028).
pub fn new_grant(
    asset_id: AssetId,
    scope: ConsentScope,
    evidence_ref: &str,
    granted_by: Uuid,
) -> Result<ConsentRow, ConsentError> {
    if evidence_ref.trim().is_empty() {
        return Err(ConsentError::MissingEvidenceRef);
    }
    Ok(ConsentRow {
        id: Uuid::new_v4(),
        asset_id,
        scope,
        status: ConsentStatus::Grant,
        evidence_ref: Some(evidence_ref.to_string()),
        granted_by,
        happened_at: OffsetDateTime::now_utc(),
    })
}

/// Create a revoke row. No evidence reference required (ADR-028).
pub fn new_revoke(asset_id: AssetId, scope: ConsentScope, granted_by: Uuid) -> ConsentRow {
    ConsentRow {
        id: Uuid::new_v4(),
        asset_id,
        scope,
        status: ConsentStatus::Revoke,
        evidence_ref: None,
        granted_by,
        happened_at: OffsetDateTime::now_utc(),
    }
}

/// Derive current consent status from an ordered slice of rows.
/// Current status = latest row by `happened_at` (ADR-028).
/// Returns `None` if no rows exist.
pub fn derive_status(rows: &[ConsentRow]) -> Option<ConsentStatus> {
    rows.iter()
        .max_by_key(|r| r.happened_at)
        .map(|r| r.status.clone())
}

/// Returns true iff the current consent status is Grant.
pub fn is_active(rows: &[ConsentRow]) -> bool {
    matches!(derive_status(rows), Some(ConsentStatus::Grant))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset() -> AssetId {
        AssetId::new()
    }

    fn actor() -> Uuid {
        Uuid::new_v4()
    }

    // HP-1: new_grant with valid evidence → derive_status = Some(Grant); is_active = true
    #[test]
    fn hp1_grant_produces_active_status() {
        let row = new_grant(asset(), ConsentScope::VoiceClone, "ref-001", actor()).unwrap();
        assert_eq!(
            derive_status(std::slice::from_ref(&row)),
            Some(ConsentStatus::Grant)
        );
        assert!(is_active(&[row]));
    }

    // HP-2: grant then revoke → derive_status = Some(Revoke); is_active = false; both rows present
    #[test]
    fn hp2_revoke_after_grant_produces_inactive_status() {
        let asset_id = asset();
        let actor_id = actor();
        let grant = new_grant(asset_id, ConsentScope::VoiceClone, "ref-001", actor_id).unwrap();
        // Ensure revoke has a later timestamp
        std::thread::sleep(std::time::Duration::from_millis(1));
        let revoke = new_revoke(asset_id, ConsentScope::VoiceClone, actor_id);
        let rows = vec![grant, revoke];
        assert_eq!(rows.len(), 2);
        assert_eq!(derive_status(&rows), Some(ConsentStatus::Revoke));
        assert!(!is_active(&rows));
    }

    // EC-1: empty evidence_ref on new_grant → Err(MissingEvidenceRef)
    #[test]
    fn ec1_empty_evidence_ref_rejected() {
        let err = new_grant(asset(), ConsentScope::VoiceClone, "", actor()).unwrap_err();
        assert_eq!(err, ConsentError::MissingEvidenceRef);
    }

    // EC-1 variant: whitespace-only evidence_ref → Err(MissingEvidenceRef)
    #[test]
    fn ec1_whitespace_evidence_ref_rejected() {
        let err = new_grant(asset(), ConsentScope::VoiceClone, "   ", actor()).unwrap_err();
        assert_eq!(err, ConsentError::MissingEvidenceRef);
    }

    // EC-2: unknown scope string → Err(UnknownScope)
    #[test]
    fn ec2_unknown_scope_fails_closed() {
        let err = ConsentScope::from_str("dubbing").unwrap_err();
        assert!(matches!(err, ConsentError::UnknownScope(_)));
    }

    // EC-2 variant: unknown status string → Err(UnknownStatus)
    #[test]
    fn ec2_unknown_status_fails_closed() {
        let err = ConsentStatus::from_str("pending").unwrap_err();
        assert!(matches!(err, ConsentError::UnknownStatus(_)));
    }

    // derive_status on empty slice returns None (no panic)
    #[test]
    fn derive_status_empty_slice_returns_none() {
        assert_eq!(derive_status(&[]), None);
        assert!(!is_active(&[]));
    }

    // Display impls match CHECK constraint values exactly (migration 0013)
    #[test]
    fn display_matches_migration_check_values() {
        assert_eq!(ConsentScope::VoiceClone.to_string(), "voice_clone");
        assert_eq!(ConsentScope::TtsSynthesis.to_string(), "tts_synthesis");
        assert_eq!(ConsentStatus::Grant.to_string(), "grant");
        assert_eq!(ConsentStatus::Revoke.to_string(), "revoke");
    }

    // new_revoke always has evidence_ref = None
    #[test]
    fn new_revoke_has_no_evidence_ref() {
        let row = new_revoke(asset(), ConsentScope::TtsSynthesis, actor());
        assert_eq!(row.evidence_ref, None);
        assert_eq!(row.status, ConsentStatus::Revoke);
    }

    // ConsentError Display covers all three variants
    #[test]
    fn consent_error_display_covers_all_variants() {
        assert_eq!(
            ConsentError::MissingEvidenceRef.to_string(),
            "evidence_ref is required for a consent grant",
        );
        assert!(
            ConsentError::UnknownScope("dubbing".into())
                .to_string()
                .contains("dubbing")
        );
        assert!(
            ConsentError::UnknownStatus("pending".into())
                .to_string()
                .contains("pending")
        );
    }

    // new_grant sets expected fields
    #[test]
    fn new_grant_sets_expected_fields() {
        let asset_id = asset();
        let actor_id = actor();
        let row = new_grant(asset_id, ConsentScope::TtsSynthesis, "ref-tts", actor_id).unwrap();
        assert_eq!(row.asset_id, asset_id);
        assert_eq!(row.scope, ConsentScope::TtsSynthesis);
        assert_eq!(row.status, ConsentStatus::Grant);
        assert_eq!(row.evidence_ref.as_deref(), Some("ref-tts"));
        assert_eq!(row.granted_by, actor_id);
    }

    // new_revoke sets expected fields
    #[test]
    fn new_revoke_sets_expected_fields() {
        let asset_id = asset();
        let actor_id = actor();
        let row = new_revoke(asset_id, ConsentScope::VoiceClone, actor_id);
        assert_eq!(row.asset_id, asset_id);
        assert_eq!(row.scope, ConsentScope::VoiceClone);
        assert_eq!(row.granted_by, actor_id);
    }

    // ConsentScope::from_str roundtrips both known variants
    #[test]
    fn consent_scope_from_str_roundtrips() {
        assert_eq!(
            "voice_clone".parse::<ConsentScope>().unwrap(),
            ConsentScope::VoiceClone,
        );
        assert_eq!(
            "tts_synthesis".parse::<ConsentScope>().unwrap(),
            ConsentScope::TtsSynthesis,
        );
    }

    // ConsentStatus::from_str roundtrips both known variants
    #[test]
    fn consent_status_from_str_roundtrips() {
        assert_eq!(
            "grant".parse::<ConsentStatus>().unwrap(),
            ConsentStatus::Grant
        );
        assert_eq!(
            "revoke".parse::<ConsentStatus>().unwrap(),
            ConsentStatus::Revoke
        );
    }
}
