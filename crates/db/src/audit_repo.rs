// T3: S1 repository — audit event insert per ADR-018
use sqlx::PgPool;

use dubbridge_domain::audit::AuditEvent;

use crate::error::DbError;

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
