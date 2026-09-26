// MVP0-P2P P2.T6b: P2 state transitions whose audit evidence must share
// the exact PostgreSQL commit boundary.

use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
    p2p_publication::{K1LineageId, P2pPublicationId},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{audit_repo::insert_audit_event_tx, error::DbError};

/// Enter reconciliation and persist the correlated ADR-018 event atomically.
/// The transition is intentionally narrow: only an actively publishing lineage
/// may enter reconciliation from this operation.
pub async fn enter_publication_reconciliation(
    pool: &PgPool,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    reason: &str,
) -> Result<(), DbError> {
    if reason.trim().is_empty() {
        return Err(DbError::Conflict);
    }

    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    let asset_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE p2p_publications
           SET state = 'reconciling',
               failure_detail = NULL,
               updated_at = now()
         WHERE id = $1
           AND lineage_id = $2
           AND state = 'publishing'
        RETURNING asset_id
        "#,
    )
    .bind(publication_id.0)
    .bind(lineage_id.0)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)?;

    let event = AuditEvent::new_p2p_event(
        AssetId(asset_id),
        AuditEventKind::P2pPublicationReconciliationEntered,
        publication_id.0,
        lineage_id.0,
        Some(reason.to_owned()),
    );
    insert_audit_event_tx(&mut tx, &event).await?;
    tx.commit().await.map_err(DbError::QueryFailed)
}
