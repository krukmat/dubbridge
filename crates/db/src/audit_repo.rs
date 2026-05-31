// T3: S1 repository — audit event insert per ADR-018
use sqlx::PgPool;

use dubbridge_domain::audit::AuditEvent;

use crate::error::DbError;

// H1-T1: transaction-aware variant so the success audit row commits atomically
// with the asset, rights, and artifact rows.
pub async fn insert_audit_event_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &AuditEvent,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (id, asset_id, event_kind, ingest_token, detail, happened_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(event.id)
    .bind(event.asset_id.as_ref().map(|a| a.0))
    .bind(event.event_kind.to_string())
    .bind(event.ingest_token)
    .bind(&event.detail)
    .bind(event.happened_at)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

pub async fn insert_audit_event(pool: &PgPool, event: &AuditEvent) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (id, asset_id, event_kind, ingest_token, detail, happened_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(event.id)
    .bind(event.asset_id.as_ref().map(|a| a.0))
    .bind(event.event_kind.to_string())
    .bind(event.ingest_token)
    .bind(&event.detail)
    .bind(event.happened_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}
