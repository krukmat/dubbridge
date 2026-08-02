// S-150-T1c-ii: fail-closed translation claim/current-pointer repository.
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_domain::{
    artifact::{ArtifactKind, DerivedArtifact, TranslationStatus},
    asset::AssetId,
    workspace::ProjectId,
};

use crate::{
    artifact_repo::{find_derived_artifact_for_asset, insert_derived_artifact_record},
    error::DbError,
};

const OPERATION: &str = "translation";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationClaimMode {
    InitialDelivery,
    ExplicitRegeneration,
}

#[derive(Debug, Clone, Copy)]
pub struct TranslationClaimInput {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub generation_request_id: Uuid,
    pub source_subtitle_artifact_id: Uuid,
    pub expected_initial_generation_request_id: Uuid,
    pub mode: TranslationClaimMode,
}

#[derive(Debug, Clone)]
pub struct TranslationGenerationClaim {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub generation_request_id: Uuid,
    pub source_artifact_id: Uuid,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct TranslationStatusSnapshot {
    pub project_id: ProjectId,
    pub asset_id: AssetId,
    pub target_language_id: Uuid,
    pub status: TranslationStatus,
    pub error_detail: Option<String>,
    pub updated_at: OffsetDateTime,
    pub current_generation_request_id: Option<Uuid>,
    pub current_source_artifact_id: Option<Uuid>,
    pub current_translated_subtitle_artifact_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct TranslationReadinessEvidence {
    pub status: Option<TranslationStatusSnapshot>,
    pub current_claim: Option<TranslationGenerationClaim>,
    pub current_source_subtitle: Option<DerivedArtifact>,
    pub current_translated_subtitle: Option<DerivedArtifact>,
}

impl TranslationReadinessEvidence {
    pub fn is_ready(&self) -> bool {
        let Some(status) = &self.status else {
            return false;
        };
        let Some(claim) = &self.current_claim else {
            return false;
        };
        let Some(source) = &self.current_source_subtitle else {
            return false;
        };
        let Some(translated) = &self.current_translated_subtitle else {
            return false;
        };

        status.status == TranslationStatus::Ready
            && status.current_generation_request_id == Some(claim.generation_request_id)
            && status.current_source_artifact_id == Some(claim.source_artifact_id)
            && status.current_translated_subtitle_artifact_id == Some(translated.id)
            && source.kind == ArtifactKind::Subtitle
            && translated.kind == ArtifactKind::TranslatedSubtitle
            && translated.parent_artifact_id == source.id
    }
}

#[derive(sqlx::FromRow)]
struct TranslationClaimRow {
    project_id: Uuid,
    asset_id: Uuid,
    target_language_id: Uuid,
    generation_request_id: Uuid,
    source_artifact_id: Uuid,
    created_at: OffsetDateTime,
}

#[derive(sqlx::FromRow)]
struct TranslationStatusRow {
    project_id: Uuid,
    asset_id: Uuid,
    target_language_id: Uuid,
    status: String,
    error_detail: Option<String>,
    updated_at: OffsetDateTime,
    current_generation_request_id: Option<Uuid>,
    current_source_artifact_id: Option<Uuid>,
    current_translated_subtitle_artifact_id: Option<Uuid>,
}

fn parse_translation_status(s: &str) -> Result<TranslationStatus, DbError> {
    s.parse().map_err(|_| DbError::UnknownStoredValue {
        field: "asset_translation_status.status",
        value: s.to_owned(),
    })
}

fn claim_from_row(row: TranslationClaimRow) -> TranslationGenerationClaim {
    TranslationGenerationClaim {
        project_id: ProjectId(row.project_id),
        asset_id: AssetId(row.asset_id),
        target_language_id: row.target_language_id,
        generation_request_id: row.generation_request_id,
        source_artifact_id: row.source_artifact_id,
        created_at: row.created_at,
    }
}

fn status_from_row(row: TranslationStatusRow) -> Result<TranslationStatusSnapshot, DbError> {
    Ok(TranslationStatusSnapshot {
        project_id: ProjectId(row.project_id),
        asset_id: AssetId(row.asset_id),
        target_language_id: row.target_language_id,
        status: parse_translation_status(&row.status)?,
        error_detail: row.error_detail,
        updated_at: row.updated_at,
        current_generation_request_id: row.current_generation_request_id,
        current_source_artifact_id: row.current_source_artifact_id,
        current_translated_subtitle_artifact_id: row.current_translated_subtitle_artifact_id,
    })
}

fn validate_translation_claim_mode(input: &TranslationClaimInput) -> Result<(), DbError> {
    match input.mode {
        TranslationClaimMode::InitialDelivery
            if input.generation_request_id != input.expected_initial_generation_request_id =>
        {
            Err(DbError::Conflict)
        }
        TranslationClaimMode::ExplicitRegeneration
            if input.generation_request_id == input.expected_initial_generation_request_id =>
        {
            Err(DbError::Conflict)
        }
        _ => Ok(()),
    }
}

async fn require_source_subtitle(
    pool: &PgPool,
    asset_id: AssetId,
    artifact_id: Uuid,
) -> Result<DerivedArtifact, DbError> {
    match find_derived_artifact_for_asset(pool, asset_id, artifact_id).await? {
        Some(artifact) if artifact.kind == ArtifactKind::Subtitle => Ok(artifact),
        _ => Err(DbError::NotFound),
    }
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

async fn insert_claim_if_absent(
    tx: &mut Transaction<'_, Postgres>,
    input: TranslationClaimInput,
) -> Result<Option<TranslationGenerationClaim>, DbError> {
    let row = sqlx::query_as::<_, TranslationClaimRow>(
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
    .bind(input.source_subtitle_artifact_id)
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
) -> Result<Option<TranslationGenerationClaim>, DbError> {
    let row = sqlx::query_as::<_, TranslationClaimRow>(
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
    input: TranslationClaimInput,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO asset_translation_status (
            project_id, asset_id, target_language_id, status, error_detail, updated_at,
            current_generation_request_id, current_source_artifact_id, current_translated_subtitle_artifact_id
        )
        VALUES ($1, $2, $3, 'in_progress', NULL, now(), $4, $5, NULL)
        ON CONFLICT (project_id, asset_id, target_language_id) DO UPDATE
            SET status = 'in_progress',
                error_detail = NULL,
                updated_at = now(),
                current_generation_request_id = EXCLUDED.current_generation_request_id,
                current_source_artifact_id = EXCLUDED.current_source_artifact_id,
                current_translated_subtitle_artifact_id = NULL
        "#,
    )
    .bind(input.project_id.0)
    .bind(input.asset_id.0)
    .bind(input.target_language_id)
    .bind(input.generation_request_id)
    .bind(input.source_subtitle_artifact_id)
    .execute(&mut **tx)
    .await
    .map_err(DbError::QueryFailed)?;

    Ok(())
}

pub async fn claim_translation_generation(
    pool: &PgPool,
    input: TranslationClaimInput,
) -> Result<TranslationGenerationClaim, DbError> {
    validate_translation_claim_mode(&input)?;
    require_source_subtitle(pool, input.asset_id, input.source_subtitle_artifact_id).await?;

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

        if claim.source_artifact_id != input.source_subtitle_artifact_id {
            return Err(DbError::Conflict);
        }

        claim
    };

    tx.commit().await.map_err(DbError::QueryFailed)?;
    Ok(claim)
}

pub async fn insert_translated_subtitle_artifact(
    pool: &PgPool,
    asset_id: AssetId,
    source_subtitle_artifact_id: Uuid,
    storage_key: &str,
    content_type: &str,
    size_bytes: i64,
    checksum: &str,
) -> Result<DerivedArtifact, DbError> {
    let source = require_source_subtitle(pool, asset_id, source_subtitle_artifact_id).await?;
    let artifact = DerivedArtifact::new(
        asset_id,
        source.id,
        ArtifactKind::TranslatedSubtitle,
        storage_key.to_string(),
        content_type.to_string(),
        size_bytes,
        checksum.to_string(),
    );
    insert_derived_artifact_record(pool, &artifact).await?;
    Ok(artifact)
}

pub async fn get_translation_status(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
) -> Result<Option<TranslationStatusSnapshot>, DbError> {
    let row = sqlx::query_as::<_, TranslationStatusRow>(
        r#"
        SELECT project_id, asset_id, target_language_id, status, error_detail, updated_at,
               current_generation_request_id, current_source_artifact_id, current_translated_subtitle_artifact_id
        FROM asset_translation_status
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
    artifact.kind == ArtifactKind::Subtitle
}

fn current_output_matches(
    artifact: &DerivedArtifact,
    current_source_artifact_id: Option<Uuid>,
) -> bool {
    artifact.kind == ArtifactKind::TranslatedSubtitle
        && current_source_artifact_id == Some(artifact.parent_artifact_id)
}

pub async fn get_translation_readiness_evidence(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
) -> Result<TranslationReadinessEvidence, DbError> {
    let status = get_translation_status(pool, project_id, asset_id, target_language_id).await?;
    let Some(status_snapshot) = status else {
        return Ok(TranslationReadinessEvidence {
            status: None,
            current_claim: None,
            current_source_subtitle: None,
            current_translated_subtitle: None,
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

    let current_source_subtitle = match status_snapshot.current_source_artifact_id {
        Some(artifact_id) => find_derived_artifact_for_asset(pool, asset_id, artifact_id)
            .await?
            .filter(current_source_matches),
        None => None,
    };

    let current_translated_subtitle = match status_snapshot.current_translated_subtitle_artifact_id
    {
        Some(artifact_id) => find_derived_artifact_for_asset(pool, asset_id, artifact_id)
            .await?
            .filter(|artifact| {
                current_output_matches(artifact, status_snapshot.current_source_artifact_id)
            }),
        None => None,
    };

    Ok(TranslationReadinessEvidence {
        status: Some(status_snapshot),
        current_claim,
        current_source_subtitle,
        current_translated_subtitle,
    })
}

pub async fn promote_translation_ready(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
    generation_request_id: Uuid,
    translated_subtitle_artifact_id: Uuid,
) -> Result<TranslationStatusSnapshot, DbError> {
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

    let translated =
        require_translated_subtitle(pool, asset_id, translated_subtitle_artifact_id).await?;
    if translated.parent_artifact_id != claim.source_artifact_id {
        return Err(DbError::Conflict);
    }

    let row = sqlx::query_as::<_, TranslationStatusRow>(
        r#"
        UPDATE asset_translation_status
        SET status = 'ready',
            error_detail = NULL,
            updated_at = now(),
            current_translated_subtitle_artifact_id = $5
        WHERE project_id = $1
          AND asset_id = $2
          AND target_language_id = $3
          AND current_generation_request_id = $4
          AND current_source_artifact_id = $6
          AND (
              current_translated_subtitle_artifact_id IS NULL
              OR current_translated_subtitle_artifact_id = $5
          )
        RETURNING project_id, asset_id, target_language_id, status, error_detail, updated_at,
                  current_generation_request_id, current_source_artifact_id, current_translated_subtitle_artifact_id
        "#,
    )
    .bind(project_id.0)
    .bind(asset_id.0)
    .bind(target_language_id)
    .bind(generation_request_id)
    .bind(translated.id)
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
    fn parse_translation_status_known_variants() {
        assert_eq!(
            parse_translation_status("pending").unwrap(),
            TranslationStatus::Pending
        );
        assert_eq!(
            parse_translation_status("in_progress").unwrap(),
            TranslationStatus::InProgress
        );
        assert_eq!(
            parse_translation_status("ready").unwrap(),
            TranslationStatus::Ready
        );
        assert_eq!(
            parse_translation_status("failed").unwrap(),
            TranslationStatus::Failed
        );
    }

    #[test]
    fn parse_translation_status_unknown_fails_closed() {
        let err = parse_translation_status("superseded").unwrap_err();
        assert!(matches!(
            err,
            DbError::UnknownStoredValue {
                field: "asset_translation_status.status",
                ..
            }
        ));
    }

    #[test]
    fn explicit_regeneration_cannot_use_reserved_initial_request_id() {
        let request_id = Uuid::new_v4();
        let input = TranslationClaimInput {
            project_id: ProjectId::new(),
            asset_id: AssetId::new(),
            target_language_id: Uuid::new_v4(),
            generation_request_id: request_id,
            source_subtitle_artifact_id: Uuid::new_v4(),
            expected_initial_generation_request_id: request_id,
            mode: TranslationClaimMode::ExplicitRegeneration,
        };

        assert!(matches!(
            validate_translation_claim_mode(&input),
            Err(DbError::Conflict)
        ));
    }
}
