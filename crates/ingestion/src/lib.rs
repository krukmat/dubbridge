// H1-T1: app-neutral ingestion finalization service.
// Consumed by apps/api HTTP upload handler and apps/worker-runner recording bridge (S3-T5).
// Neither app may depend on the other; both depend on this crate.

use dubbridge_audit::emit_governance_audit;
use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord},
    asset::{Asset, IngestionStatus},
    audit::{AuditEvent, AuditEventKind},
    ingestion::{FinalizeIngestionCommand, IngestionError},
    rights::RightsRecord,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum IngestionServiceError {
    #[error("ingestion validation failed: {0}")]
    Validation(#[from] IngestionError),
    #[error("ingestion token has already been finalized")]
    AlreadyFinalized,
    #[error("ingestion session has expired")]
    SessionExpired,
    #[error("ingestion session not found")]
    SessionNotFound,
    #[error("database error: {0}")]
    Db(#[from] dubbridge_db::error::DbError),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Atomically finalizes one ingestion session.
///
/// H1-T1 atomicity contract:
///   1. Acquires a `SELECT … FOR UPDATE` row lock on the pending session. This
///      prevents cleanup (which uses `SKIP LOCKED`) from deleting the blob while
///      this transaction is in flight.
///   2. Validates rights fail-closed per ADR-008 inside the transaction.
///   3. Commits asset + rights_record + artifact_record + asset status +
///      audit_event + pending-row deletion as a single PostgreSQL transaction.
///      Any failure rolls back every write; no partial rows are left.
///
/// The caller (HTTP handler or worker-runner bridge) is responsible for the
/// object-store write (storage-first per ADR-006) before calling this function.
/// That write is not part of the transaction because cross-store atomicity is not
/// achievable; ADR-006 assigns orphan reconciliation to S2.
pub async fn finalize_ingestion_core(
    pool: &PgPool,
    ingest_token: Uuid,
    uploader_id: Uuid,
    artifact_kind: ArtifactKind,
) -> Result<Asset, IngestionServiceError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| IngestionServiceError::Db(dubbridge_db::error::DbError::QueryFailed(e)))?;

    // Lock the pending row for the duration of this transaction (H1-T1).
    // If the row is not found, distinguish two cases:
    //   - An artifact already exists → the session was previously finalized (409).
    //   - No artifact exists → the session never existed or was never created (404).
    let pending =
        match dubbridge_db::pending_ingestion_repo::lock_for_finalize(&mut tx, ingest_token).await?
        {
            Some(record) => record,
            None => {
                let already_done =
                    dubbridge_db::artifact_repo::exists_for_token_tx(&mut tx, ingest_token).await?;
                if already_done {
                    // H1-T3: persist durable audit row before returning — drop tx first so
                    // the pool connection is released before the audit pool write.
                    drop(tx);
                    emit_duplicate_rejection(pool, ingest_token).await?;
                    return Err(IngestionServiceError::AlreadyFinalized);
                }
                return Err(IngestionServiceError::SessionNotFound);
            }
        };

    if pending.expires_at < OffsetDateTime::now_utc() {
        return Err(IngestionServiceError::SessionExpired);
    }

    // Idempotency guard inside the transaction — no separate round-trip needed.
    // H1-T3: idempotency guard — emit durable audit before returning.
    if dubbridge_db::artifact_repo::exists_for_token_tx(&mut tx, ingest_token).await? {
        drop(tx);
        emit_duplicate_rejection(pool, ingest_token).await?;
        return Err(IngestionServiceError::AlreadyFinalized);
    }

    let command = FinalizeIngestionCommand {
        ingest_token,
        uploader_id: Some(uploader_id),
        rights_basis: pending.rights_basis.clone(),
        asset_title: pending.title.clone(),
        file_key: pending.storage_key.clone(),
        file_size_bytes: pending.file_size_bytes,
        content_type: pending.content_type.clone(),
    };
    command
        .validate()
        .map_err(IngestionServiceError::Validation)?;

    let rights_basis = pending
        .rights_basis
        .clone()
        .ok_or(IngestionServiceError::Validation(
            IngestionError::MissingRightsBasis,
        ))?;
    let asset =
        dubbridge_domain::asset::Asset::new_pending(command.asset_title.clone(), uploader_id);

    dubbridge_db::asset_repo::insert_asset_tx(&mut tx, &asset).await?;

    let rights_record = RightsRecord::new(asset.id, &rights_basis);
    dubbridge_db::rights_repo::insert_rights_record_tx(&mut tx, &rights_record).await?;

    let artifact_record = ArtifactRecord {
        id: Uuid::new_v4(),
        asset_id: asset.id,
        kind: artifact_kind,
        ingest_token,
        storage_key: pending.storage_key.clone(),
        content_type: pending.content_type.clone(),
        size_bytes: pending.file_size_bytes,
        checksum: pending.checksum.clone(),
        created_at: OffsetDateTime::now_utc(),
    };

    if let Err(e) =
        dubbridge_db::artifact_repo::insert_artifact_record_tx(&mut tx, &artifact_record).await
    {
        if is_unique_violation(&e) {
            tracing::info!(ingest_token = %ingest_token, "duplicate ingestion finalization rejected");
            return Err(IngestionServiceError::AlreadyFinalized);
        }
        return Err(IngestionServiceError::Db(e));
    }

    dubbridge_db::asset_repo::update_asset_status_tx(
        &mut tx,
        asset.id,
        &IngestionStatus::Finalized,
    )
    .await?;

    let audit_event = AuditEvent::new(
        Some(asset.id),
        AuditEventKind::IngestionFinalized,
        ingest_token,
        Some(format!("asset {} finalized", asset.id)),
    );
    dubbridge_db::audit_repo::insert_audit_event_tx(&mut tx, &audit_event).await?;

    // Pending row deleted inside the transaction — cleanup cannot race this delete.
    dubbridge_db::pending_ingestion_repo::delete_pending_ingestion_tx(&mut tx, ingest_token)
        .await?;

    tx.commit()
        .await
        .map_err(|e| IngestionServiceError::Db(dubbridge_db::error::DbError::QueryFailed(e)))?;

    tracing::info!(asset_id = %asset.id, ingest_token = %ingest_token, "ingestion finalized");

    dubbridge_db::asset_repo::find_asset_by_id(pool, asset.id)
        .await?
        .ok_or_else(|| {
            IngestionServiceError::Internal("asset disappeared after finalization".into())
        })
}

/// Emits the durable duplicate-finalization audit row (H1-T3). The caller must
/// drop its transaction first so the pool connection is free for this write.
async fn emit_duplicate_rejection(
    pool: &PgPool,
    ingest_token: Uuid,
) -> Result<(), IngestionServiceError> {
    let event = AuditEvent::new(
        None,
        AuditEventKind::IngestionRejectedDuplicateToken,
        ingest_token,
        Some("duplicate finalization rejected".into()),
    );
    emit_governance_audit(pool, &event)
        .await
        .map_err(|e| IngestionServiceError::Internal(e.to_string()))
}

fn is_unique_violation(error: &dubbridge_db::error::DbError) -> bool {
    match error {
        dubbridge_db::error::DbError::QueryFailed(sqlx::Error::Database(db_err)) => {
            db_err.code().as_deref() == Some("23505")
        }
        _ => false,
    }
}
