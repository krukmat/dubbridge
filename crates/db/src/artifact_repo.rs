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

// H1-T1: transaction-aware variants for atomic finalize.
pub async fn insert_artifact_record_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record: &ArtifactRecord,
) -> Result<(), DbError> {
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
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;
    Ok(())
}

/// H1-T1: idempotency check within a transaction — avoids a separate round-trip.
pub async fn exists_for_token_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ingest_token: Uuid,
) -> Result<bool, DbError> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM artifact_records WHERE ingest_token = $1)")
            .bind(ingest_token)
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::QueryFailed)?;
    Ok(exists)
}

// H1-T2: fail-closed — unknown stored kind must not silently coerce to OriginalMedia (ADR-008).
fn parse_kind(s: &str) -> Result<ArtifactKind, DbError> {
    match s {
        "original_media" => Ok(ArtifactKind::OriginalMedia),
        other => Err(DbError::UnknownStoredValue {
            field: "artifact_records.kind",
            value: other.to_owned(),
        }),
    }
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

    row.map(|r| {
        Ok(ArtifactRecord {
            id: r.id,
            asset_id: AssetId(r.asset_id),
            kind: parse_kind(&r.kind)?,
            ingest_token: r.ingest_token,
            storage_key: r.storage_key,
            content_type: r.content_type,
            size_bytes: r.size_bytes,
            checksum: r.checksum,
            created_at: r.created_at,
        })
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    // H1-T2: parse_kind must succeed for every known variant and fail for unknown values.
    #[test]
    fn parse_kind_known_variant_succeeds() {
        assert!(matches!(
            parse_kind("original_media"),
            Ok(ArtifactKind::OriginalMedia)
        ));
    }

    #[test]
    fn parse_kind_unknown_value_fails_closed() {
        let err = parse_kind("rendition").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "artifact_records.kind",
                ..
            }
        ));
        assert!(err.to_string().contains("rendition"));
    }
}
