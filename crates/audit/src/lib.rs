// H1-T3: governance audit boundary per ADR-018.
// Single entry point that couples durable PostgreSQL persistence with correlated
// tracing. Callers must not write audit rows or emit governance traces directly.
//
// Fail-closed policy: if the DB write fails, the function returns Err. The caller
// is responsible for the response — typically 500 Internal Server Error, because a
// governance event that cannot be audited must not be silently swallowed.
//
// This crate re-uses types from crates/domain and persistence from crates/db.
// It must not duplicate either.

use dubbridge_domain::audit::AuditEvent;
use sqlx::PgPool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditEmitError {
    #[error("audit persistence failed: {0}")]
    Db(#[from] dubbridge_db::error::DbError),
}

fn has_valid_correlation_contract(event: &AuditEvent) -> bool {
    event.has_valid_ingestion_correlation()
        || event.has_valid_recording_correlation()
        || event.has_valid_platform_ingest_correlation()
        || event.has_valid_workspace_correlation()
        || event.has_valid_consent_correlation()
        || event.has_valid_review_correlation()
        || event.has_valid_playback_correlation()
        || event.has_valid_auth_correlation()
}

/// Emits one governance audit event durably.
///
/// Persists the event to `audit_events` and emits a correlated trace span. The
/// accepted correlation shape is family-specific and defined by the X26-T3c-a
/// contract matrix.
///
/// Fail-closed: returns `Err` if the DB write fails so callers can surface a 500
/// rather than silently losing the audit trail.
pub async fn emit_governance_audit(
    pool: &PgPool,
    event: &AuditEvent,
) -> Result<(), AuditEmitError> {
    // X26-T3c-a matrix: every emitted event must satisfy its family-specific
    // correlation shape before the durable audit write is attempted.
    assert!(
        has_valid_correlation_contract(event),
        "audit event violates X26-T3c-a correlation contract"
    );
    dubbridge_db::audit_repo::insert_audit_event(pool, event)
        .await
        .map_err(AuditEmitError::Db)?;

    tracing::info!(
        ingest_token = event.ingest_token.map(|t| t.to_string()),
        recording_session_id = event.recording_session_id.map(|s| s.to_string()),
        platform_ingest_session_id = event.platform_ingest_session_id.map(|s| s.to_string()),
        event_kind   = %event.event_kind,
        "governance audit emitted"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    // H1-T3: the fail-closed policy and the AuditEmitError type are unit-testable
    // without a live DB. DB-integration tests live in apps/api/tests/ingestion_test.rs.

    use super::*;
    use dubbridge_domain::{asset::AssetId, audit::AuditEventKind};

    #[test]
    fn audit_emit_error_message_contains_source() {
        let db_err = dubbridge_db::error::DbError::QueryFailed(sqlx::Error::RowNotFound);
        let emit_err = AuditEmitError::Db(db_err);
        assert!(emit_err.to_string().contains("audit persistence failed"));
    }

    #[test]
    fn every_constructor_family_satisfies_the_audit_boundary_contract() {
        let asset_id = AssetId::new();
        let events = vec![
            AuditEvent::new(
                Some(asset_id),
                AuditEventKind::IngestionFinalized,
                AssetId::new().0,
                None,
            ),
            AuditEvent::new_recording(
                Some(asset_id),
                AuditEventKind::RecordingRecorded,
                AssetId::new().0,
                Some(AssetId::new().0),
                None,
            ),
            AuditEvent::new_platform_ingest(
                Some(asset_id),
                AuditEventKind::PlatformIngestDownloaded,
                AssetId::new().0,
                None,
            ),
            AuditEvent::new_workspace_event(AuditEventKind::OrgCreated, None),
            AuditEvent::new_consent(asset_id, AuditEventKind::ConsentGranted, None),
            AuditEvent::new_review_event(asset_id, AuditEventKind::ReviewApproved, None),
            AuditEvent::new_playback_event(
                asset_id,
                AuditEventKind::PlaybackGrantIssued,
                None,
            ),
            AuditEvent::new_auth_event(AuditEventKind::AuthLoginSucceeded, None),
        ];

        assert!(
            events.iter().all(has_valid_correlation_contract),
            "a current AuditEvent constructor violates the X26-T3c-a matrix"
        );
    }

    #[test]
    fn malformed_event_would_trip_the_boundary_assert_before_persistence() {
        let mut event = AuditEvent::new(
            Some(AssetId::new()),
            AuditEventKind::IngestionFinalized,
            AssetId::new().0,
            None,
        );
        event.ingest_token = None;

        let panic = std::panic::catch_unwind(|| {
            assert!(
                has_valid_correlation_contract(&event),
                "audit event violates X26-T3c-a correlation contract"
            );
        });
        assert!(panic.is_err());
    }
}
