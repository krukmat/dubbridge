// S-130-T1: repository for transcription status and transcript/alignment artifacts
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::artifact::{
    ArtifactKind, DerivedArtifact, TranscriptionStatus, TranscriptionStatusRecord,
};
use dubbridge_domain::asset::AssetId;

use crate::error::DbError;

fn parse_transcription_status(s: &str) -> Result<TranscriptionStatus, DbError> {
    match s {
        "pending" => Ok(TranscriptionStatus::Pending),
        "in_progress" => Ok(TranscriptionStatus::InProgress),
        "ready" => Ok(TranscriptionStatus::Ready),
        "failed" => Ok(TranscriptionStatus::Failed),
        other => Err(DbError::UnknownStoredValue {
            field: "asset_transcription_status.status",
            value: other.to_owned(),
        }),
    }
}

#[derive(sqlx::FromRow)]
struct TranscriptionStatusRow {
    asset_id: Uuid,
    status: String,
    error_detail: Option<String>,
    updated_at: OffsetDateTime,
}

/// Upsert the transcription status for an asset.
pub async fn upsert_transcription_status(
    pool: &PgPool,
    asset_id: AssetId,
    status: TranscriptionStatus,
    error_detail: Option<&str>,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO asset_transcription_status (asset_id, status, error_detail, updated_at)
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

/// Return the current transcription status for an asset, or `None` if not yet initialised.
pub async fn get_transcription_status(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<Option<TranscriptionStatusRecord>, DbError> {
    let row = sqlx::query_as::<_, TranscriptionStatusRow>(
        r#"
        SELECT asset_id, status, error_detail, updated_at
        FROM asset_transcription_status
        WHERE asset_id = $1
        "#,
    )
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(|r| {
        Ok(TranscriptionStatusRecord {
            asset_id: AssetId(r.asset_id),
            status: parse_transcription_status(&r.status)?,
            error_detail: r.error_detail,
            updated_at: r.updated_at,
        })
    })
    .transpose()
}

/// Fetch the `ArtifactRecord` that will be the audio source for transcription.
///
/// Fails if the artifact does not exist or belongs to a different asset.
pub async fn get_source_artifact_for_transcription(
    pool: &PgPool,
    asset_id: AssetId,
    source_artifact_id: Uuid,
) -> Result<dubbridge_domain::artifact::ArtifactRecord, DbError> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        asset_id: Uuid,
        kind: String,
        storage_key: String,
        content_type: String,
        size_bytes: i64,
        checksum: String,
        ingest_token: Uuid,
        created_at: OffsetDateTime,
    }

    let row = sqlx::query_as::<_, Row>(
        r#"
        SELECT id, asset_id, kind, storage_key, content_type,
               size_bytes, checksum, ingest_token, created_at
        FROM artifact_records
        WHERE id = $1 AND asset_id = $2
        "#,
    )
    .bind(source_artifact_id)
    .bind(asset_id.0)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::NotFound)?;

    use dubbridge_domain::artifact::parse_artifact_kind;
    let kind = parse_artifact_kind(&row.kind);

    Ok(dubbridge_domain::artifact::ArtifactRecord {
        id: row.id,
        asset_id: AssetId(row.asset_id),
        kind,
        storage_key: row.storage_key,
        content_type: row.content_type,
        size_bytes: row.size_bytes,
        checksum: row.checksum,
        ingest_token: row.ingest_token,
        created_at: row.created_at,
    })
}

/// Artifact metadata needed to persist one transcript or alignment file.
pub struct TranscriptArtifactMeta<'a> {
    pub storage_key: &'a str,
    pub size_bytes: i64,
    pub checksum: &'a str,
}

/// Persist a `TranscriptText` and a `WordAlignment` derived artifact for the asset.
///
/// Both artifacts are linked to `source_artifact_id` via `parent_artifact_id`.
/// The caller is responsible for uploading the content to storage before calling this.
pub async fn insert_transcript_artifacts(
    pool: &PgPool,
    asset_id: AssetId,
    source_artifact_id: Uuid,
    transcript: TranscriptArtifactMeta<'_>,
    alignment: TranscriptArtifactMeta<'_>,
) -> Result<(DerivedArtifact, DerivedArtifact), DbError> {
    let transcript_artifact = DerivedArtifact::new(
        asset_id,
        source_artifact_id,
        ArtifactKind::TranscriptText,
        transcript.storage_key.to_string(),
        "application/json".to_string(),
        transcript.size_bytes,
        transcript.checksum.to_string(),
    );

    let alignment_artifact = DerivedArtifact::new(
        asset_id,
        source_artifact_id,
        ArtifactKind::WordAlignment,
        alignment.storage_key.to_string(),
        "application/json".to_string(),
        alignment.size_bytes,
        alignment.checksum.to_string(),
    );

    for artifact in [&transcript_artifact, &alignment_artifact] {
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
    }

    Ok((transcript_artifact, alignment_artifact))
}

/// Return `true` when both `TranscriptText` and `WordAlignment` derived artifacts
/// exist for the asset. Fail-closed: any DB error propagates.
pub async fn get_transcription_readiness_evidence(
    pool: &PgPool,
    asset_id: AssetId,
) -> Result<bool, DbError> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(DISTINCT kind)
        FROM artifact_records
        WHERE asset_id = $1
          AND kind IN ('transcript_text', 'word_alignment')
        "#,
    )
    .bind(asset_id.0)
    .fetch_one(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(count == 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_transcription_status_all_known_variants() {
        assert_eq!(
            parse_transcription_status("pending").unwrap(),
            TranscriptionStatus::Pending
        );
        assert_eq!(
            parse_transcription_status("in_progress").unwrap(),
            TranscriptionStatus::InProgress
        );
        assert_eq!(
            parse_transcription_status("ready").unwrap(),
            TranscriptionStatus::Ready
        );
        assert_eq!(
            parse_transcription_status("failed").unwrap(),
            TranscriptionStatus::Failed
        );
    }

    #[test]
    fn parse_transcription_status_unknown_fails_closed() {
        let err = parse_transcription_status("skipped").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "asset_transcription_status.status",
                ..
            }
        ));
        assert!(err.to_string().contains("skipped"));
    }
}
