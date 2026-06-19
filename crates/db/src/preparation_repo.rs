// S-120-T2: repository for derived artifacts and asset preparation status
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::artifact::{
    ArtifactKind, ArtifactRecord, DerivedArtifact, PreparationStatus, PreparationStatusRecord,
};
use dubbridge_domain::asset::AssetId;

use crate::error::DbError;

fn parse_kind(s: &str) -> Result<ArtifactKind, DbError> {
    match s {
        "original_media" => Ok(ArtifactKind::OriginalMedia),
        "recorded_stream_media" => Ok(ArtifactKind::RecordedStreamMedia),
        "downloaded_platform_media" => Ok(ArtifactKind::DownloadedPlatformMedia),
        "probe_metadata" => Ok(ArtifactKind::ProbeMetadata),
        "hls_manifest" => Ok(ArtifactKind::HlsManifest),
        "hls_segment" => Ok(ArtifactKind::HlsSegment),
        other => Err(DbError::UnknownStoredValue {
            field: "artifact_records.kind",
            value: other.to_owned(),
        }),
    }
}

fn parse_status(s: &str) -> Result<PreparationStatus, DbError> {
    match s {
        "pending" => Ok(PreparationStatus::Pending),
        "in_progress" => Ok(PreparationStatus::InProgress),
        "ready" => Ok(PreparationStatus::Ready),
        "failed" => Ok(PreparationStatus::Failed),
        other => Err(DbError::UnknownStoredValue {
            field: "asset_preparation_status.status",
            value: other.to_owned(),
        }),
    }
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

#[derive(sqlx::FromRow)]
struct PreparationStatusRow {
    asset_id: Uuid,
    status: String,
    error_detail: Option<String>,
    updated_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct PreparationReadinessEvidenceRow {
    probe_metadata_count: i64,
    hls_manifest_count: i64,
    hls_segment_count: i64,
}

#[derive(sqlx::FromRow)]
struct SourceArtifactRow {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparationReadinessEvidence {
    pub probe_metadata_count: i64,
    pub hls_manifest_count: i64,
    pub hls_segment_count: i64,
}

impl PreparationReadinessEvidence {
    pub fn is_ready(&self) -> bool {
        self.probe_metadata_count >= 1
            && self.hls_manifest_count >= 1
            && self.hls_segment_count >= 1
    }
}

/// Insert a derived artifact row (probe metadata, HLS manifest, HLS segment).
/// The artifact is linked to `parent_artifact_id` rather than an ingest_token.
pub async fn insert_derived_artifact(
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

/// Return all derived artifacts for an asset, ordered by creation time.
pub async fn list_derived_artifacts(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Vec<DerivedArtifact>, DbError> {
    let rows = sqlx::query_as::<_, DerivedArtifactRow>(
        r#"
        SELECT id, asset_id, parent_artifact_id, kind, storage_key, content_type, size_bytes, checksum, created_at
        FROM artifact_records
        WHERE asset_id = $1 AND parent_artifact_id IS NOT NULL
        ORDER BY created_at ASC
        "#,
    )
    .bind(asset_id.0)
    .fetch_all(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    rows.into_iter()
        .map(|r| {
            Ok(DerivedArtifact {
                id: r.id,
                asset_id: AssetId(r.asset_id),
                parent_artifact_id: r.parent_artifact_id,
                kind: parse_kind(&r.kind)?,
                storage_key: r.storage_key,
                content_type: r.content_type,
                size_bytes: r.size_bytes,
                checksum: r.checksum,
                created_at: r.created_at,
            })
        })
        .collect()
}

/// Return the source artifact for an asset (the row with `parent_artifact_id IS NULL`).
pub async fn find_source_artifact(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Option<ArtifactRecord>, DbError> {
    let row = sqlx::query_as::<_, SourceArtifactRow>(
        r#"
        SELECT id, asset_id, kind, ingest_token, storage_key, content_type, size_bytes, checksum, created_at
        FROM artifact_records
        WHERE asset_id = $1 AND parent_artifact_id IS NULL
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(asset_id.0)
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

/// Persist a canonical probe-metadata artifact linked to the source artifact.
/// This function does not mark the asset `Ready`; HLS readiness remains a later step.
pub async fn insert_probe_metadata_artifact(
    pool: &PgPool,
    asset_id: AssetId,
    storage_key: &str,
    size_bytes: i64,
    checksum: &str,
) -> Result<DerivedArtifact, DbError> {
    let source = find_source_artifact(pool, asset_id)
        .await?
        .ok_or(DbError::NotFound)?;
    let artifact = DerivedArtifact::new(
        asset_id,
        source.id,
        ArtifactKind::ProbeMetadata,
        storage_key.to_string(),
        "application/json".to_string(),
        size_bytes,
        checksum.to_string(),
    );
    insert_derived_artifact(pool, &artifact).await?;
    Ok(artifact)
}

/// Persist a complete HLS package linked to the source artifact.
/// The package is represented as one manifest artifact plus one-or-more segment artifacts.
pub async fn insert_hls_artifacts(
    pool: &PgPool,
    asset_id: AssetId,
    manifest_storage_key: &str,
    manifest_size_bytes: i64,
    manifest_checksum: &str,
    segments: &[(String, i64, String)],
) -> Result<(DerivedArtifact, Vec<DerivedArtifact>), DbError> {
    let source = find_source_artifact(pool, asset_id)
        .await?
        .ok_or(DbError::NotFound)?;

    let manifest = DerivedArtifact::new(
        asset_id,
        source.id,
        ArtifactKind::HlsManifest,
        manifest_storage_key.to_string(),
        "application/vnd.apple.mpegurl".to_string(),
        manifest_size_bytes,
        manifest_checksum.to_string(),
    );
    insert_derived_artifact(pool, &manifest).await?;

    let mut inserted_segments = Vec::with_capacity(segments.len());
    for (storage_key, size_bytes, checksum) in segments {
        let artifact = DerivedArtifact::new(
            asset_id,
            source.id,
            ArtifactKind::HlsSegment,
            storage_key.clone(),
            "video/mp2t".to_string(),
            *size_bytes,
            checksum.clone(),
        );
        insert_derived_artifact(pool, &artifact).await?;
        inserted_segments.push(artifact);
    }

    Ok((manifest, inserted_segments))
}

/// Return the current preparation status for an asset, or `None` if not yet initialised.
pub async fn get_preparation_status(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Option<PreparationStatusRecord>, DbError> {
    let row = sqlx::query_as::<_, PreparationStatusRow>(
        r#"
        SELECT asset_id, status, error_detail, updated_at
        FROM asset_preparation_status
        WHERE asset_id = $1
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(|r| {
        Ok(PreparationStatusRecord {
            asset_id: AssetId(r.asset_id),
            status: parse_status(&r.status)?,
            error_detail: r.error_detail,
            updated_at: r.updated_at,
        })
    })
    .transpose()
}

/// Upsert the preparation status for an asset. Uses INSERT … ON CONFLICT DO UPDATE
/// so it is safe to call from first initialisation through terminal transitions.
pub async fn upsert_preparation_status(
    pool: &PgPool,
    asset_id: AssetId,
    status: PreparationStatus,
    error_detail: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO asset_preparation_status (asset_id, status, error_detail, updated_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (asset_id) DO UPDATE
            SET status       = EXCLUDED.status,
                error_detail = EXCLUDED.error_detail,
                updated_at   = EXCLUDED.updated_at
        "#,
    )
    .bind(asset_id.0)
    .bind(status.to_string())
    .bind(error_detail)
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

/// Summarize the derived-artifact evidence that T5b uses before writing `Ready`.
pub async fn get_preparation_readiness_evidence(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<PreparationReadinessEvidence, DbError> {
    let row = sqlx::query_as::<_, PreparationReadinessEvidenceRow>(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN kind = 'probe_metadata' THEN 1 ELSE 0 END), 0)::BIGINT AS probe_metadata_count,
            COALESCE(SUM(CASE WHEN kind = 'hls_manifest' THEN 1 ELSE 0 END), 0)::BIGINT AS hls_manifest_count,
            COALESCE(SUM(CASE WHEN kind = 'hls_segment' THEN 1 ELSE 0 END), 0)::BIGINT AS hls_segment_count
        FROM artifact_records
        WHERE asset_id = $1 AND parent_artifact_id IS NOT NULL
        "#,
    )
    .bind(asset_id.0)
    .fetch_one(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(PreparationReadinessEvidence {
        probe_metadata_count: row.probe_metadata_count,
        hls_manifest_count: row.hls_manifest_count,
        hls_segment_count: row.hls_segment_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kind_all_known_variants() {
        assert!(matches!(
            parse_kind("probe_metadata"),
            Ok(ArtifactKind::ProbeMetadata)
        ));
        assert!(matches!(
            parse_kind("hls_manifest"),
            Ok(ArtifactKind::HlsManifest)
        ));
        assert!(matches!(
            parse_kind("hls_segment"),
            Ok(ArtifactKind::HlsSegment)
        ));
        assert!(matches!(
            parse_kind("original_media"),
            Ok(ArtifactKind::OriginalMedia)
        ));
    }

    #[test]
    fn parse_kind_unknown_fails_closed() {
        let err = parse_kind("thumbnail").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "artifact_records.kind",
                ..
            }
        ));
        assert!(err.to_string().contains("thumbnail"));
    }

    #[test]
    fn parse_status_all_known_variants() {
        assert_eq!(parse_status("pending").unwrap(), PreparationStatus::Pending);
        assert_eq!(
            parse_status("in_progress").unwrap(),
            PreparationStatus::InProgress
        );
        assert_eq!(parse_status("ready").unwrap(), PreparationStatus::Ready);
        assert_eq!(parse_status("failed").unwrap(), PreparationStatus::Failed);
    }

    #[test]
    fn parse_status_unknown_fails_closed() {
        let err = parse_status("skipped").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "asset_preparation_status.status",
                ..
            }
        ));
        assert!(err.to_string().contains("skipped"));
    }

    #[test]
    fn preparation_readiness_evidence_requires_probe_manifest_and_segment() {
        let ready = PreparationReadinessEvidence {
            probe_metadata_count: 1,
            hls_manifest_count: 1,
            hls_segment_count: 2,
        };
        let missing_manifest = PreparationReadinessEvidence {
            probe_metadata_count: 1,
            hls_manifest_count: 0,
            hls_segment_count: 2,
        };

        assert!(ready.is_ready());
        assert!(!missing_manifest.is_ready());
    }
}
