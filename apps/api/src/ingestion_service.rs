// S3-T0: transport-agnostic finalization core — callable by both the HTTP upload
// handler (ArtifactKind::OriginalMedia) and the future recording bridge (T5,
// ArtifactKind::RecordedStreamMedia).
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord},
    asset::{Asset, IngestionStatus},
    audit::{AuditEvent, AuditEventKind},
    ingestion::{FinalizeIngestionCommand, IngestionError},
};

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

/// Core finalization logic shared by the HTTP upload handler and the recording bridge.
///
/// Caller is responsible for supplying the correct `artifact_kind`:
/// - HTTP upload handler → `ArtifactKind::OriginalMedia`
/// - Recording bridge (T5) → `ArtifactKind::RecordedStreamMedia`
///
/// The `pending` record must have already been loaded and its expiry checked by
/// the caller, OR the caller can pass `None` to have this function load and check it.
/// This overload accepts an already-loaded `pending` to avoid a redundant DB round-trip
/// from the HTTP handler (which already fetched it for expiry enforcement).
pub async fn finalize_ingestion_core(
    pool: &PgPool,
    ingest_token: Uuid,
    uploader_id: Uuid,
    artifact_kind: ArtifactKind,
    pending: dubbridge_db::pending_ingestion_repo::PendingIngestionRecord,
) -> Result<Asset, IngestionServiceError> {
    // Idempotency guard: reject if an artifact already exists for this token.
    if dubbridge_db::artifact_repo::find_original_by_ingest_token(pool, ingest_token)
        .await?
        .is_some()
    {
        tracing::info!(ingest_token = %ingest_token, "duplicate ingestion finalization rejected");
        dubbridge_db::pending_ingestion_repo::delete_pending_ingestion(pool, ingest_token).await?;
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

    command.validate().map_err(IngestionServiceError::Validation)?;

    let rights_basis = pending.rights_basis.clone().expect("validated rights basis");
    let asset = Asset::new_pending(command.asset_title.clone(), uploader_id);
    dubbridge_db::asset_repo::insert_asset(pool, &asset).await?;

    let rights_record =
        dubbridge_domain::rights::RightsRecord::new(asset.id, &rights_basis);
    dubbridge_db::rights_repo::insert_rights_record(pool, &rights_record).await?;

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

    if let Err(error) =
        dubbridge_db::artifact_repo::insert_artifact_record(pool, &artifact_record).await
    {
        if is_unique_violation(&error) {
            tracing::info!(ingest_token = %ingest_token, "duplicate ingestion finalization rejected");
            return Err(IngestionServiceError::AlreadyFinalized);
        }
        return Err(IngestionServiceError::Db(error));
    }

    dubbridge_db::asset_repo::update_asset_status(pool, asset.id, &IngestionStatus::Finalized)
        .await?;

    let audit_event = AuditEvent::new(
        Some(asset.id),
        AuditEventKind::IngestionFinalized,
        ingest_token,
        Some(format!("asset {} finalized from upload", asset.id)),
    );
    dubbridge_db::audit_repo::insert_audit_event(pool, &audit_event).await?;

    tracing::info!(asset_id = %asset.id, ingest_token = %ingest_token, "ingestion finalized");
    dubbridge_db::pending_ingestion_repo::delete_pending_ingestion(pool, ingest_token).await?;

    dubbridge_db::asset_repo::find_asset_by_id(pool, asset.id)
        .await?
        .ok_or_else(|| IngestionServiceError::Internal("asset disappeared after finalization".into()))
}

fn is_unique_violation(error: &dubbridge_db::error::DbError) -> bool {
    match error {
        dubbridge_db::error::DbError::QueryFailed(sqlx::Error::Database(database_error)) => {
            database_error.code().as_deref() == Some("23505")
        }
        _ => false,
    }
}
