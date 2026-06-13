// S-110-T2a: fail-closed consent gate per ADR-008 / ADR-028
// S-110-T2b: durable audit wiring per ADR-018
use sqlx::PgPool;

use dubbridge_audit::emit_governance_audit;
use dubbridge_db::consent_repo;
use dubbridge_db::error::DbError;
use dubbridge_domain::asset::AssetId;
use dubbridge_domain::audit::{AuditEvent, AuditEventKind};
use dubbridge_domain::consent::{ConsentRow, ConsentScope, ConsentStatus};

#[derive(Debug, PartialEq)]
pub enum ConsentGateError {
    /// No active (unrevoked) consent exists for the requested asset + scope.
    NoActiveConsent {
        asset_id: AssetId,
        scope: ConsentScope,
    },
    /// Underlying DB error (consent read/write).
    Db(String),
    /// Audit persistence failed (ADR-018: must not be silenced).
    AuditFailed(String),
}

impl std::fmt::Display for ConsentGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveConsent { asset_id, scope } => write!(
                f,
                "no active consent for asset {} scope {}",
                asset_id.0, scope
            ),
            Self::Db(msg) => write!(f, "consent gate db error: {msg}"),
            Self::AuditFailed(msg) => write!(f, "consent gate audit failed: {msg}"),
        }
    }
}

impl From<DbError> for ConsentGateError {
    fn from(e: DbError) -> Self {
        Self::Db(e.to_string())
    }
}

impl From<dubbridge_audit::AuditEmitError> for ConsentGateError {
    fn from(e: dubbridge_audit::AuditEmitError) -> Self {
        Self::AuditFailed(e.to_string())
    }
}

/// Synchronous fail-closed evaluation of a pre-fetched consent status (ADR-008).
///
/// Separated from the async DB fetch so the decision logic is unit-testable
/// without a live database. `require_active_consent` is the public entry point
/// that fetches and then delegates here.
pub fn require_active_consent_with(
    status: Option<ConsentStatus>,
    asset_id: AssetId,
    scope: &ConsentScope,
) -> Result<(), ConsentGateError> {
    match status {
        Some(ConsentStatus::Grant) => Ok(()),
        Some(ConsentStatus::Revoke) | None => Err(ConsentGateError::NoActiveConsent {
            asset_id,
            scope: scope.clone(),
        }),
    }
}

/// Returns a `ConsentCheckDenied` audit event when the check result is denied,
/// or `None` when the check passed. Pure function — unit-testable without DB.
pub fn denied_check_audit_event(
    check: &Result<(), ConsentGateError>,
    asset_id: AssetId,
    scope: &ConsentScope,
) -> Option<AuditEvent> {
    match check {
        Ok(_) => None,
        Err(_) => Some(AuditEvent::new_consent(
            asset_id,
            AuditEventKind::ConsentCheckDenied,
            Some(format!("scope={scope}")),
        )),
    }
}

/// Maps a `ConsentStatus` to the corresponding audit event kind.
/// Pure function — unit-testable without DB.
pub fn audit_kind_for_status(status: ConsentStatus) -> AuditEventKind {
    match status {
        ConsentStatus::Grant => AuditEventKind::ConsentGranted,
        ConsentStatus::Revoke => AuditEventKind::ConsentRevoked,
    }
}

/// Fail-closed consent precondition (ADR-008, ADR-028).
///
/// Fetches the latest consent status for `asset_id` + `scope` and delegates to
/// `require_active_consent_with`. Emits `ConsentCheckDenied` audit when denied
/// (ADR-018). Returns `Ok(())` only when the latest row is a Grant.
///
/// Fail-closed on audit: if the audit write fails, returns `Err(AuditFailed)`.
pub async fn require_active_consent(
    pool: &PgPool,
    asset_id: AssetId,
    scope: &ConsentScope,
) -> Result<(), ConsentGateError> {
    let status = consent_repo::latest_consent_status(pool, asset_id, scope).await?;
    let check = require_active_consent_with(status, asset_id, scope);
    if let Some(event) = denied_check_audit_event(&check, asset_id, scope) {
        emit_governance_audit(pool, &event).await?;
    }
    check
}

/// Append a consent row and emit a durable audit event (ADR-018, ADR-028).
///
/// Inserts the row first (append-only per ADR-028), then emits audit.
/// If audit fails, returns `Err(AuditFailed)` — the row is already written
/// (append-only ledger; no rollback is needed or possible).
pub async fn append_consent_audited(
    pool: &PgPool,
    row: &ConsentRow,
) -> Result<(), ConsentGateError> {
    consent_repo::append_consent(pool, row).await?;
    let event = AuditEvent::new_consent(
        row.asset_id,
        audit_kind_for_status(row.status.clone()),
        Some(format!("scope={}", row.scope)),
    );
    emit_governance_audit(pool, &event).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_domain::asset::AssetId;
    use dubbridge_domain::consent::ConsentScope;

    fn asset() -> AssetId {
        AssetId::new()
    }

    // HP-1: Some(Grant) → Ok(()) — no audit emitted (sync path)
    #[test]
    fn grant_status_maps_to_ok() {
        let result = require_active_consent_with(
            Some(ConsentStatus::Grant),
            asset(),
            &ConsentScope::VoiceClone,
        );
        assert!(result.is_ok());
    }

    // EC-1 (T2a): None → Err(NoActiveConsent)
    #[test]
    fn none_status_maps_to_no_active_consent() {
        let asset_id = asset();
        let scope = ConsentScope::VoiceClone;
        let result = require_active_consent_with(None, asset_id, &scope);
        assert!(matches!(
            result,
            Err(ConsentGateError::NoActiveConsent { .. })
        ));
    }

    // EC-2 (T2a): Some(Revoke) → Err(NoActiveConsent)
    #[test]
    fn revoke_status_maps_to_no_active_consent() {
        let result = require_active_consent_with(
            Some(ConsentStatus::Revoke),
            asset(),
            &ConsentScope::TtsSynthesis,
        );
        assert!(matches!(
            result,
            Err(ConsentGateError::NoActiveConsent { .. })
        ));
    }

    // EC-3 (T2a): scope mismatch → repo returns None → Err
    #[test]
    fn scope_mismatch_none_maps_to_no_active_consent() {
        let result = require_active_consent_with(None, asset(), &ConsentScope::TtsSynthesis);
        assert!(matches!(
            result,
            Err(ConsentGateError::NoActiveConsent { .. })
        ));
    }

    // Error carries the correct asset_id and scope.
    #[test]
    fn error_carries_asset_and_scope() {
        let asset_id = asset();
        let scope = ConsentScope::VoiceClone;
        let err = require_active_consent_with(None, asset_id, &scope).unwrap_err();
        match err {
            ConsentGateError::NoActiveConsent {
                asset_id: a,
                scope: s,
            } => {
                assert_eq!(a, asset_id);
                assert_eq!(s, scope);
            }
            _ => panic!("unexpected error variant"),
        }
    }

    // Display includes asset and scope.
    #[test]
    fn error_display_includes_asset_and_scope() {
        let asset_id = asset();
        let err = ConsentGateError::NoActiveConsent {
            asset_id,
            scope: ConsentScope::VoiceClone,
        };
        let msg = err.to_string();
        assert!(msg.contains("voice_clone"));
        assert!(msg.contains(&asset_id.0.to_string()));
    }

    // DbError converts to ConsentGateError::Db.
    #[test]
    fn db_error_converts_to_gate_error() {
        let gate_err = ConsentGateError::from(DbError::NotFound);
        assert!(matches!(gate_err, ConsentGateError::Db(_)));
    }

    // ConsentGateError::Db display includes the message.
    #[test]
    fn db_error_display_includes_message() {
        let err = ConsentGateError::Db("timeout".to_string());
        assert!(err.to_string().contains("timeout"));
    }

    // AuditFailed display includes the message.
    #[test]
    fn audit_failed_display_includes_message() {
        let err = ConsentGateError::AuditFailed("disk full".to_string());
        assert!(err.to_string().contains("disk full"));
        assert!(err.to_string().contains("audit failed"));
    }

    // AuditEmitError converts to ConsentGateError::AuditFailed.
    #[test]
    fn audit_emit_error_converts_to_audit_failed() {
        let db_err = DbError::NotFound;
        let emit_err = dubbridge_audit::AuditEmitError::Db(db_err);
        let gate_err = ConsentGateError::from(emit_err);
        assert!(matches!(gate_err, ConsentGateError::AuditFailed(_)));
    }

    // denied_check_audit_event returns None on Ok result.
    #[test]
    fn denied_audit_event_is_none_on_ok() {
        let ok: Result<(), ConsentGateError> = Ok(());
        let event = denied_check_audit_event(&ok, asset(), &ConsentScope::VoiceClone);
        assert!(event.is_none());
    }

    // denied_check_audit_event returns Some(ConsentCheckDenied) on Err result.
    #[test]
    fn denied_audit_event_is_some_on_err() {
        let asset_id = asset();
        let scope = ConsentScope::VoiceClone;
        let err: Result<(), ConsentGateError> = Err(ConsentGateError::NoActiveConsent {
            asset_id,
            scope: scope.clone(),
        });
        let event = denied_check_audit_event(&err, asset_id, &scope).expect("expected event");
        assert_eq!(
            event.event_kind,
            dubbridge_domain::audit::AuditEventKind::ConsentCheckDenied
        );
        assert_eq!(event.asset_id, Some(asset_id));
        assert!(
            event
                .detail
                .as_deref()
                .unwrap_or("")
                .contains("voice_clone")
        );
    }

    // audit_kind_for_status maps Grant → ConsentGranted.
    #[test]
    fn audit_kind_grant_maps_to_consent_granted() {
        assert_eq!(
            audit_kind_for_status(ConsentStatus::Grant),
            dubbridge_domain::audit::AuditEventKind::ConsentGranted
        );
    }

    // audit_kind_for_status maps Revoke → ConsentRevoked.
    #[test]
    fn audit_kind_revoke_maps_to_consent_revoked() {
        assert_eq!(
            audit_kind_for_status(ConsentStatus::Revoke),
            dubbridge_domain::audit::AuditEventKind::ConsentRevoked
        );
    }
}
