// T3: S1 repository — artifact record insert and idempotency lookup
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact};
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

#[derive(sqlx::FromRow)]
struct DerivedArtifactRow {
    id: Uuid,
    asset_id: Uuid,
    parent_artifact_id: Uuid,
    kind: String,
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
// S-120-T2: preparation-derived kinds added.
pub(crate) fn parse_artifact_kind_strict(s: &str) -> Result<ArtifactKind, DbError> {
    match s {
        "original_media" => Ok(ArtifactKind::OriginalMedia),
        "recorded_stream_media" => Ok(ArtifactKind::RecordedStreamMedia),
        "downloaded_platform_media" => Ok(ArtifactKind::DownloadedPlatformMedia),
        "probe_metadata" => Ok(ArtifactKind::ProbeMetadata),
        "hls_manifest" => Ok(ArtifactKind::HlsManifest),
        "hls_segment" => Ok(ArtifactKind::HlsSegment),
        "transcript_text" => Ok(ArtifactKind::TranscriptText),
        "word_alignment" => Ok(ArtifactKind::WordAlignment),
        "subtitle" => Ok(ArtifactKind::Subtitle),
        "translated_subtitle" => Ok(ArtifactKind::TranslatedSubtitle),
        "dubbed_audio_segment" => Ok(ArtifactKind::DubbedAudioSegment),
        "dubbing_manifest" => Ok(ArtifactKind::DubbingManifest),
        "dubbed_audio" => Ok(ArtifactKind::DubbedAudio),
        other => Err(DbError::UnknownStoredValue {
            field: "artifact_records.kind",
            value: other.to_owned(),
        }),
    }
}

pub async fn insert_derived_artifact_record(
    pool: &PgPool,
    artifact: &DerivedArtifact,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO artifact_records
            (id, asset_id, kind, parent_artifact_id, storage_key, content_type, size_bytes, checksum, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(artifact.id)
    .bind(artifact.asset_id.0)
    .bind(artifact.kind.to_string())
    .bind(artifact.parent_artifact_id)
    .bind(&artifact.storage_key)
    .bind(&artifact.content_type)
    .bind(artifact.size_bytes)
    .bind(&artifact.checksum)
    .bind(artifact.created_at)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn find_derived_artifact_for_asset(
    pool: &PgPool,
    asset_id: AssetId,
    artifact_id: Uuid,
) -> Result<Option<DerivedArtifact>, DbError> {
    let row = sqlx::query_as::<_, DerivedArtifactRow>(
        r#"
        SELECT id, asset_id, parent_artifact_id, kind, storage_key, content_type, size_bytes, checksum, created_at
        FROM artifact_records
        WHERE id = $1 AND asset_id = $2 AND parent_artifact_id IS NOT NULL
        "#,
    )
    .bind(artifact_id)
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(|r| {
        Ok(DerivedArtifact {
            id: r.id,
            asset_id: AssetId(r.asset_id),
            parent_artifact_id: r.parent_artifact_id,
            kind: parse_artifact_kind_strict(&r.kind)?,
            storage_key: r.storage_key,
            content_type: r.content_type,
            size_bytes: r.size_bytes,
            checksum: r.checksum,
            created_at: r.created_at,
        })
    })
    .transpose()
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
            kind: parse_artifact_kind_strict(&r.kind)?,
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
            parse_artifact_kind_strict("original_media"),
            Ok(ArtifactKind::OriginalMedia)
        ));
    }

    // S-120-T2: new preparation-derived kinds must parse successfully (fail-closed).
    #[test]
    fn parse_kind_preparation_variants_succeed() {
        assert!(matches!(
            parse_artifact_kind_strict("probe_metadata"),
            Ok(ArtifactKind::ProbeMetadata)
        ));
        assert!(matches!(
            parse_artifact_kind_strict("hls_manifest"),
            Ok(ArtifactKind::HlsManifest)
        ));
        assert!(matches!(
            parse_artifact_kind_strict("hls_segment"),
            Ok(ArtifactKind::HlsSegment)
        ));
    }

    #[test]
    fn parse_kind_localization_variants_succeed() {
        assert!(matches!(
            parse_artifact_kind_strict("translated_subtitle"),
            Ok(ArtifactKind::TranslatedSubtitle)
        ));
        assert!(matches!(
            parse_artifact_kind_strict("dubbed_audio_segment"),
            Ok(ArtifactKind::DubbedAudioSegment)
        ));
        assert!(matches!(
            parse_artifact_kind_strict("dubbing_manifest"),
            Ok(ArtifactKind::DubbingManifest)
        ));
        assert!(matches!(
            parse_artifact_kind_strict("dubbed_audio"),
            Ok(ArtifactKind::DubbedAudio)
        ));
    }

    #[test]
    fn parse_kind_unknown_value_fails_closed() {
        let err = parse_artifact_kind_strict("rendition").unwrap_err();
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
