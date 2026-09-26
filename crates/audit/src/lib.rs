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
        || event.has_valid_p2p_correlation()
        || event.has_valid_p3_correlation()
}

/// Emits one governance audit event durably.
///
/// Persists the event to `audit_events` and emits a correlated trace span. The
/// accepted correlation shape is family-specific and defined by the audit
/// contract matrix. P2 events use publication/lineage ids and never fabricate an
/// ingest token.
///
/// Fail-closed: returns `Err` if the DB write fails so callers can surface a 500
/// rather than silently losing the audit trail.
pub async fn emit_governance_audit(
    pool: &PgPool,
    event: &AuditEvent,
) -> Result<(), AuditEmitError> {
    assert!(
        has_valid_correlation_contract(event),
        "audit event violates correlation contract"
    );
    dubbridge_db::audit_repo::insert_audit_event(pool, event)
        .await
        .map_err(AuditEmitError::Db)?;

    tracing::info!(
        ingest_token = event.ingest_token.map(|t| t.to_string()),
        recording_session_id = event.recording_session_id.map(|s| s.to_string()),
        platform_ingest_session_id = event.platform_ingest_session_id.map(|s| s.to_string()),
        correlation_id = event.correlation_id.map(|s| s.to_string()),
        publication_id = event.publication_id.map(|s| s.to_string()),
        lineage_id = event.lineage_id.map(|s| s.to_string()),
        event_kind = %event.event_kind,
        "governance audit emitted"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
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
        let publication_id = AssetId::new().0;
        let lineage_id = AssetId::new().0;
        let events = [
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
            AuditEvent::new_playback_event(asset_id, AuditEventKind::PlaybackGrantIssued, None),
            AuditEvent::new_auth_event(AuditEventKind::AuthLoginSucceeded, None),
            AuditEvent::new_p2p_event(
                asset_id,
                AuditEventKind::P2pPublicationReady,
                publication_id,
                lineage_id,
                None,
            ),
            AuditEvent::new_p3_event(
                Some(asset_id),
                AuditEventKind::P2pInvitationClaimed,
                AssetId::new().0,
                Some(publication_id),
                Some(lineage_id),
                None,
            ),
        ];

        assert!(
            events.iter().all(has_valid_correlation_contract),
            "a current AuditEvent constructor violates the correlation matrix"
        );
    }

    #[test]
    fn malformed_p2p_event_would_trip_the_boundary_assert_before_persistence() {
        let asset_id = AssetId::new();
        let mut event = AuditEvent::new_p2p_event(
            asset_id,
            AuditEventKind::P2pPublicationReady,
            AssetId::new().0,
            AssetId::new().0,
            None,
        );
        event.correlation_id = None;

        let panic = std::panic::catch_unwind(|| {
            assert!(
                has_valid_correlation_contract(&event),
                "audit event violates correlation contract"
            );
        });
        assert!(panic.is_err());
    }
}
