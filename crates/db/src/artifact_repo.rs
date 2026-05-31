// T3: S1 repository — artifact record insert and idempotency lookup
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::artifact::{ArtifactKind, ArtifactRecord};
use dubbridge_domain::asset::AssetId;

use crate::error::DbError;

pub async fn insert_artifact_record(pool: &PgPool, record: &ArtifactRecord) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO artifact_records (id, asset_id, kind, ingest_token, storage_key, content_type, size_bytes, checksum, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(record.id)
    .bind(record.asset_id.0)
    .bind(record.kind.to_string())
    .bind(record.ingest_token)
    .bind(&record.storage_key)
    .bind(&record.content_type)
    .bind(record.size_bytes)
    .bind(&record.checksum)
    .bind(record.created_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct ArtifactRow {
    id: Uuid,
    asset_id: Uuid,
    kind: String,
    ingest_token: Uuid,
    storage_key: String,
    content_type: String,
    size_bytes: i64,
    checksum: String,
    created_at: OffsetDateTime,
}

/// Idempotency guard — returns existing artifact if this token was already finalized.
pub async fn find_original_by_ingest_token(
    pool: &PgPool,
    ingest_token: Uuid,
) -> Result<Option<ArtifactRecord>, DbError> {
    let row = sqlx::query_as::<_, ArtifactRow>(
        r#"
        SELECT id, asset_id, kind, ingest_token, storage_key, content_type, size_bytes, checksum, created_at
        FROM artifact_records WHERE ingest_token = $1
        "#,
    )
    .bind(ingest_token)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(row.map(|r| ArtifactRecord {
        id: r.id,
        asset_id: AssetId(r.asset_id),
        kind: if r.kind == "original_media" {
            ArtifactKind::OriginalMedia
        } else {
            ArtifactKind::OriginalMedia
        },
        ingest_token: r.ingest_token,
        storage_key: r.storage_key,
        content_type: r.content_type,
        size_bytes: r.size_bytes,
        checksum: r.checksum,
        created_at: r.created_at,
    }))
}
