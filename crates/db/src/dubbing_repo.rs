// S-150-T1c-ii: fail-closed dubbing claim/current-pointer repository.
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    artifact::{ArtifactKind, DerivedArtifact, DubbingStatus},
    asset::AssetId,
    workspace::ProjectId,
};

use crate::{
    artifact_repo::{find_derived_artifact_for_asset, insert_derived_artifact_record},
    error::DbError,
};

const OPERATION: &str = "dubbing";

#[derive(Debug, Clone, Copy)]
pub struct DubbingClaimInput {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub generation_request_id: Uuid,
    pub source_translated_subtitle_artifact_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct DubbingGenerationClaim {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub generation_request_id: Uuid,
    pub source_artifact_id: Uuid,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct DubbingStatusSnapshot {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub status: DubbingStatus,
    pub error_detail: Option<String>,
    pub updated_at: OffsetDateTime,
    pub current_generation_request_id: Option<Uuid>,
    pub current_source_artifact_id: Option<Uuid>,
    pub current_manifest_artifact_id: Option<Uuid>,
    pub current_dubbed_audio_artifact_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct DubbingReadinessEvidence {
    pub status: Option<DubbingStatusSnapshot>,
    pub current_claim: Option<DubbingGenerationClaim>,
    pub current_source_translated_subtitle: Option<DerivedArtifact>,
    pub current_manifest: Option<DerivedArtifact>,
    pub current_dubbed_audio: Option<DerivedArtifact>,
}

impl DubbingReadinessEvidence {
    pub fn is_ready(&self) -> bool {
        let Some(status) = &self.status else {
            return false;
        };
        let Some(claim) = &self.current_claim else {
            return false;
        };
        let Some(source) = &self.current_source_translated_subtitle else {
            return false;
        };
        let Some(manifest) = &self.current_manifest else {
            return false;
        };
        let Some(audio) = &self.current_dubbed_audio else {
            return false;
        };

        status.status == DubbingStatus::Ready
            && status.current_generation_request_id == Some(claim.generation_request_id)
            && status.current_source_artifact_id == Some(claim.source_artifact_id)
            && status.current_manifest_artifact_id == Some(manifest.id)
            && status.current_dubbed_audio_artifact_id == Some(audio.id)
            && source.kind == ArtifactKind::TranslatedSubtitle
            && manifest.kind == ArtifactKind::DubbingManifest
            && audio.kind == ArtifactKind::DubbedAudio
            && manifest.parent_artifact_id == source.id
            && audio.parent_artifact_id == manifest.id
    }
}

#[derive(sqlx::FromRow)]
struct DubbingClaimRow {
    project_id: Uuid,
    asset_id: Uuid,
    target_language_id: Uuid,
    generation_request_id: Uuid,
    source_artifact_id: Uuid,
    created_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct DubbingStatusRow {
    project_id: Uuid,
    asset_id: Uuid,
    target_language_id: Uuid,
    status: String,
    error_detail: Option<String>,
    updated_at: OffsetDateTime,
    current_generation_request_id: Option<Uuid>,
    current_source_artifact_id: Option<Uuid>,
    current_manifest_artifact_id: Option<Uuid>,
    current_dubbed_audio_artifact_id: Option<Uuid>,
}

fn parse_dubbing_status(s: &str) -> Result<DubbingStatus, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "asset_dubbing_status.status",
        value: s.to_owned(),
    })
}

fn claim_from_row(row: DubbingClaimRow) -> DubbingGenerationClaim {
    DubbingGenerationClaim {
        project_id: ProjectId(row.project_id),
        asset_id: AssetId(row.asset_id),
        target_language_id: row.target_language_id,
        generation_request_id: row.generation_request_id,
        source_artifact_id: row.source_artifact_id,
        created_at: row.created_at,
    }
}

fn status_from_row(row: DubbingStatusRow) -> Result<DubbingStatusSnapshot, DbError> {
    Ok(DubbingStatusSnapshot {
        project_id: ProjectId(row.project_id),
        asset_id: AssetId(row.asset_id),
        target_language_id: row.target_language_id,
        status: parse_dubbing_status(&row.status)?,
        error_detail: row.error_detail,
        updated_at: row.updated_at,
        current_generation_request_id: row.current_generation_request_id,
        current_source_artifact_id: row.current_source_artifact_id,
        current_manifest_artifact_id: row.current_manifest_artifact_id,
        current_dubbed_audio_artifact_id: row.current_dubbed_audio_artifact_id,
    })
}

async fn require_translated_subtitle(
    pool: &PgPool,
    asset_id: AssetId,
    artifact_id: Uuid,
) -> Result<DerivedArtifact, DbError> {
    match find_derived_artifact_for_asset(pool, asset_id, artifact_id).await? {
        Some(artifact) if artifact.kind == ArtifactKind::TranslatedSubtitle => Ok(artifact),
        _ => Err(DbError::NotFound),
    }
}

async fn require_manifest(
    pool: &PgPool,
    asset_id: AssetId,
    artifact_id: Uuid,
) -> Result<DerivedArtifact, DbError> {
    match find_derived_artifact_for_asset(pool, asset_id, artifact_id).await? {
        Some(artifact) if artifact.kind == ArtifactKind::DubbingManifest => Ok(artifact),
        _ => Err(DbError::NotFound),
    }
}

async fn require_dubbed_audio(
    pool: &PgPool,
    asset_id: AssetId,
    artifact_id: Uuid,
) -> Result<DerivedArtifact, DbError> {
    match find_derived_artifact_for_asset(pool, asset_id, artifact_id).await? {
        Some(artifact) if artifact.kind == ArtifactKind::DubbedAudio => Ok(artifact),
        _ => Err(DbError::NotFound),
    }
}

async fn insert_claim_if_absent(
    tx: &mut Transaction<'_, Postgres>,
    input: DubbingClaimInput,
) -> Result<Option<DubbingGenerationClaim>, DbError> {
    let row = sqlx::query_as::<_, DubbingClaimRow>(
        r#"
        INSERT INTO localization_generation_claims (
            operation, project_id, asset_id, target_language_id, generation_request_id, source_artifact_id
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT DO NOTHING
        RETURNING project_id, asset_id, target_language_id, generation_request_id, source_artifact_id, created_at
        "#,
    )
    .bind(OPERATION)
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .bind(input.source_translated_subtitle_artifact_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(row.map(claim_from_row))
}

async fn get_claim_tx(
    tx: &mut Transaction<'_, Postgres>,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
    generation_request_id: Uuid,
) -> Result<Option<DubbingGenerationClaim>, DbError> {
    let row = sqlx::query_as::<_, DubbingClaimRow>(
        r#"
        SELECT project_id, asset_id, target_language_id, generation_request_id, source_artifact_id, created_at
        FROM localization_generation_claims
        WHERE operation = $1
          AND project_id = $2
          AND asset_id = $3
          AND target_language_id = $4
          AND generation_request_id = $5
        "#,
    )
    .bind(OPERATION)
    .bind(project_id.0)
    .bind(asset_id.0)
    .bind(target_language_id)
    .bind(generation_request_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(row.map(claim_from_row))
}

async fn set_current_generation_tx(
    tx: &mut Transaction<'_, Postgres>,
    input: DubbingClaimInput,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO asset_dubbing_status (
            project_id, asset_id, target_language_id, status, error_detail, updated_at,
            current_generation_request_id, current_source_artifact_id,
            current_manifest_artifact_id, current_dubbed_audio_artifact_id
        )
        VALUES ($1, $2, $3, 'in_progress', NULL, now(), $4, $5, NULL, NULL)
        ON CONFLICT (project_id, asset_id, target_language_id) DO UPDATE
            SET status = 'in_progress',
                error_detail = NULL,
                updated_at = now(),
                current_generation_request_id = EXCLUDED.current_generation_request_id,
                current_source_artifact_id = EXCLUDED.current_source_artifact_id,
                current_manifest_artifact_id = NULL,
                current_dubbed_audio_artifact_id = NULL
        "#,
    )
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .bind(input.source_translated_subtitle_artifact_id)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn claim_dubbing_generation(
    pool: &PgPool,
    input: DubbingClaimInput,
) -> Result<DubbingGenerationClaim, DbError> {
    require_translated_subtitle(
        pool,
        input.asset_id,
        input.source_translated_subtitle_artifact_id,
    )
    .await?;

    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    let inserted = insert_claim_if_absent(&mut tx, input).await?;

    let claim = if let Some(claim) = inserted {
        set_current_generation_tx(&mut tx, input).await?;
        claim
    } else {
        let claim = get_claim_tx(
            &mut tx,
            input.project_id,
            input.asset_id,
            input.target_language_id,
            input.generation_request_id,
        )
        .await?
        .ok_or(DbError::NotFound)?;

        if claim.source_artifact_id != input.source_translated_subtitle_artifact_id {
            return Err(DbError::Conflict);
        }

        claim
    };

    tx.commit().await.map_err(DbError::QueryFailed)?;
    Ok(claim)
}

pub async fn insert_dubbing_manifest_artifact(
    pool: &PgPool,
    asset_id: AssetId,
    source_translated_subtitle_artifact_id: Uuid,
    storage_key: &str,
    content_type: &str,
    size_bytes: i64,
    checksum: &str,
) -> Result<DerivedArtifact, DbError> {
    let source =
        require_translated_subtitle(pool, asset_id, source_translated_subtitle_artifact_id).await?;
    let artifact = DerivedArtifact::new(
        asset_id,
        source.id,
        ArtifactKind::DubbingManifest,
        storage_key.to_string(),
        content_type.to_string(),
        size_bytes,
        checksum.to_string(),
    );
    insert_derived_artifact_record(pool, &artifact).await?;
    Ok(artifact)
}

pub async fn insert_dubbed_audio_artifact(
    pool: &PgPool,
    asset_id: AssetId,
    manifest_artifact_id: Uuid,
    storage_key: &str,
    content_type: &str,
    size_bytes: i64,
    checksum: &str,
) -> Result<DerivedArtifact, DbError> {
    let manifest = require_manifest(pool, asset_id, manifest_artifact_id).await?;
    let artifact = DerivedArtifact::new(
        asset_id,
        manifest.id,
        ArtifactKind::DubbedAudio,
        storage_key.to_string(),
        content_type.to_string(),
        size_bytes,
        checksum.to_string(),
    );
    insert_derived_artifact_record(pool, &artifact).await?;
    Ok(artifact)
}

pub async fn get_dubbing_status(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
) -> Result<Option<DubbingStatusSnapshot>, DbError> {
    let row = sqlx::query_as::<_, DubbingStatusRow>(
        r#"
        SELECT project_id, asset_id, target_language_id, status, error_detail, updated_at,
               current_generation_request_id, current_source_artifact_id,
               current_manifest_artifact_id, current_dubbed_audio_artifact_id
        FROM asset_dubbing_status
        WHERE project_id = $1 AND asset_id = $2 AND target_language_id = $3
        "#,
    )
    .bind(project_id.0)
    .bind(asset_id.0)
    .bind(target_language_id)
    .fetch_optional(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    row.map(status_from_row).transpose()
}

fn current_source_matches(artifact: &DerivedArtifact) -> bool {
    artifact.kind == ArtifactKind::TranslatedSubtitle
}

fn current_manifest_matches(
    artifact: &DerivedArtifact,
    current_source_artifact_id: Option<Uuid>,
) -> bool {
    artifact.kind == ArtifactKind::DubbingManifest
        && current_source_artifact_id == Some(artifact.parent_artifact_id)
}

fn current_dubbed_audio_matches(
    artifact: &DerivedArtifact,
    current_manifest_artifact_id: Option<Uuid>,
) -> bool {
    artifact.kind == ArtifactKind::DubbedAudio
        && current_manifest_artifact_id == Some(artifact.parent_artifact_id)
}

pub async fn get_dubbing_readiness_evidence(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
) -> Result<DubbingReadinessEvidence, DbError> {
    let status = get_dubbing_status(pool, project_id, asset_id, target_language_id).await?;
    let Some(status_snapshot) = status else {
        return Ok(DubbingReadinessEvidence {
            status: None,
            current_claim: None,
            current_source_translated_subtitle: None,
            current_manifest: None,
            current_dubbed_audio: None,
        });
    };

    let current_claim = match status_snapshot.current_generation_request_id {
        Some(generation_request_id) => {
            let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
            let claim = get_claim_tx(
                &mut tx,
                project_id,
                asset_id,
                target_language_id,
                generation_request_id,
            )
            .await?;
            tx.rollback().await.map_err(DbError::QueryFailed)?;
            claim.filter(|claim| {
                Some(claim.source_artifact_id) == status_snapshot.current_source_artifact_id
            })
        }
        None => None,
    };

    let current_source_translated_subtitle = match status_snapshot.current_source_artifact_id {
        Some(artifact_id) => find_derived_artifact_for_asset(pool, asset_id, artifact_id)
            .await?
            .filter(current_source_matches),
        None => None,
    };

    let current_manifest = match status_snapshot.current_manifest_artifact_id {
        Some(artifact_id) => find_derived_artifact_for_asset(pool, asset_id, artifact_id)
            .await?
            .filter(|artifact| {
                current_manifest_matches(artifact, status_snapshot.current_source_artifact_id)
            }),
        None => None,
    };

    let current_dubbed_audio = match status_snapshot.current_dubbed_audio_artifact_id {
        Some(artifact_id) => find_derived_artifact_for_asset(pool, asset_id, artifact_id)
            .await?
            .filter(|artifact| {
                current_dubbed_audio_matches(artifact, status_snapshot.current_manifest_artifact_id)
            }),
        None => None,
    };

    Ok(DubbingReadinessEvidence {
        status: Some(status_snapshot),
        current_claim,
        current_source_translated_subtitle,
        current_manifest,
        current_dubbed_audio,
    })
}

pub async fn promote_dubbing_ready(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
    generation_request_id: Uuid,
    manifest_artifact_id: Uuid,
    dubbed_audio_artifact_id: Uuid,
) -> Result<DubbingStatusSnapshot, DbError> {
    let mut tx = pool.begin().await.map_err(DbError::QueryFailed)?;
    let claim = get_claim_tx(
        &mut tx,
        project_id,
        asset_id,
        target_language_id,
        generation_request_id,
    )
    .await?
    .ok_or(DbError::NotFound)?;

    let manifest = require_manifest(pool, asset_id, manifest_artifact_id).await?;
    if manifest.parent_artifact_id != claim.source_artifact_id {
        return Err(DbError::Conflict);
    }

    let dubbed_audio = require_dubbed_audio(pool, asset_id, dubbed_audio_artifact_id).await?;
    if dubbed_audio.parent_artifact_id != manifest.id {
        return Err(DbError::Conflict);
    }

    let row = sqlx::query_as::<_, DubbingStatusRow>(
        r#"
        UPDATE asset_dubbing_status
        SET status = 'ready',
            error_detail = NULL,
            updated_at = now(),
            current_manifest_artifact_id = $5,
            current_dubbed_audio_artifact_id = $6
        WHERE project_id = $1
          AND asset_id = $2
          AND target_language_id = $3
          AND current_generation_request_id = $4
          AND current_source_artifact_id = $7
          AND (
              current_manifest_artifact_id IS NULL
              OR current_manifest_artifact_id = $5
          )
          AND (
              current_dubbed_audio_artifact_id IS NULL
              OR current_dubbed_audio_artifact_id = $6
          )
        RETURNING project_id, asset_id, target_language_id, status, error_detail, updated_at,
                  current_generation_request_id, current_source_artifact_id,
                  current_manifest_artifact_id, current_dubbed_audio_artifact_id
        "#,
    )
    .bind(project_id.0)
    .bind(asset_id.0)
    .bind(target_language_id)
    .bind(generation_request_id)
    .bind(manifest.id)
    .bind(dubbed_audio.id)
    .bind(claim.source_artifact_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::QueryFailed)?
    .ok_or(DbError::Conflict)?;

    tx.commit().await.map_err(DbError::QueryFailed)?;
    status_from_row(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dubbing_status_known_variants() {
        assert_eq!(
            parse_dubbing_status("pending").unwrap(),
            DubbingStatus::Pending
        );
        assert_eq!(
            parse_dubbing_status("in_progress").unwrap(),
            DubbingStatus::InProgress
        );
        assert_eq!(parse_dubbing_status("ready").unwrap(), DubbingStatus::Ready);
        assert_eq!(
            parse_dubbing_status("failed").unwrap(),
            DubbingStatus::Failed
        );
    }

    #[test]
    fn parse_dubbing_status_unknown_fails_closed() {
        let err = parse_dubbing_status("superseded").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "asset_dubbing_status.status",
                ..
            }
        ));
    }
}
