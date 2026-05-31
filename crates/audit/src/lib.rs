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

/// Emits one governance audit event durably.
///
/// Persists the event to `audit_events` and emits a correlated trace span.
/// Both share `ingest_token` as the correlation identifier (ADR-018).
///
/// Fail-closed: returns `Err` if the DB write fails so callers can surface a 500
/// rather than silently losing the audit trail.
pub async fn emit_governance_audit(
    pool: &PgPool,
    event: &AuditEvent,
) -> Result<(), AuditEmitError> {
    dubbridge_db::audit_repo::insert_audit_event(pool, event)
        .await
        .map_err(AuditEmitError::Db)?;

    // S3-T1: ingest_token is now Option<Uuid>; format as string for structured log field.
    tracing::info!(
        ingest_token = event.ingest_token.map(|t| t.to_string()),
        recording_session_id = event.recording_session_id.map(|s| s.to_string()),
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

    #[test]
    fn audit_emit_error_message_contains_source() {
        let db_err = dubbridge_db::error::DbError::QueryFailed(sqlx::Error::RowNotFound);
        let emit_err = AuditEmitError::Db(db_err);
        assert!(emit_err.to_string().contains("audit persistence failed"));
    }
}
