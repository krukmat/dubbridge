use anyhow::{Context, bail};
use dubbridge_db::translation_repo;
use dubbridge_domain::{asset::AssetId, workspace::ProjectId};
use dubbridge_jobs::{JobEnvelope, TranslationJob};
use dubbridge_providers::translation::{
    LegacySubtitleSegment, TranslationWorkerClient, normalize_legacy_segments,
};
use dubbridge_storage::{StorageAdapter, translated_subtitle_key};
use sqlx::PgPool;
use uuid::Uuid;

use crate::checksum_hex;

#[allow(dead_code)]
pub(crate) async fn process_translation_envelope(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn TranslationWorkerClient,
    envelope: JobEnvelope<TranslationJob>,
) -> anyhow::Result<()> {
    if envelope.job_type != TranslationJob::JOB_TYPE {
        bail!(
            "unsupported translation job type '{}', expected '{}'",
            envelope.job_type,
            TranslationJob::JOB_TYPE
        );
    }

    process_translation_job(pool, storage, client, envelope.payload).await
}

#[allow(dead_code)]
pub(crate) async fn process_translation_job(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn TranslationWorkerClient,
    job: TranslationJob,
) -> anyhow::Result<()> {
    let project_id = ProjectId(job.project_id);
    let asset_id = AssetId(job.asset_id);
    let target_language_id = job.target_language_id;

    let result = process_translation_job_inner(pool, storage, client, &job).await;
    if let Err(error) = result {
        let detail = format!("{error:#}");
        if let Err(mark_error) = translation_repo::mark_translation_failed(
            pool,
            project_id,
            asset_id,
            target_language_id,
            job.generation_request_id,
            &detail,
        )
        .await
        {
            tracing::warn!(
                ?mark_error,
                asset_id = %job.asset_id,
                target_language_id = %target_language_id,
                "failed to record translation failure status"
            );
        }
        return Err(error);
    }

    Ok(())
}

async fn process_translation_job_inner(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn TranslationWorkerClient,
    job: &TranslationJob,
) -> anyhow::Result<()> {
    let project_id = ProjectId(job.project_id);
    let asset_id = AssetId(job.asset_id);

    let source = load_source_subtitle(pool, storage, job).await?;
    let languages = resolve_languages(pool, project_id, job.target_language_id).await?;

    let input = normalize_legacy_segments(
        &job.source_subtitle_artifact_id.to_string(),
        &job.generation_request_id.to_string(),
        &languages.source_lang,
        &languages.target_lang,
        &source.segments,
    )
    .map_err(|e| anyhow::anyhow!("failed to normalize source subtitle segments: {e}"))?;

    let output = client
        .translate(input)
        .map_err(|e| anyhow::anyhow!("translation worker error: {e}"))?;

    let translated_bytes =
        serde_json::to_vec(&output.segments).context("failed to serialize translated segments")?;

    let translated_artifact = store_translated_subtitle_artifact(
        pool,
        storage,
        asset_id,
        job.target_language_id,
        job.source_subtitle_artifact_id,
        &translated_bytes,
    )
    .await?;

    promote_and_verify_ready(pool, project_id, asset_id, job, translated_artifact.id).await?;

    Ok(())
}

struct SourceSubtitle {
    segments: Vec<LegacySubtitleSegment>,
}

async fn load_source_subtitle(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    job: &TranslationJob,
) -> anyhow::Result<SourceSubtitle> {
    let asset_id = AssetId(job.asset_id);
    let source_artifact = dubbridge_db::artifact_repo::find_derived_artifact_for_asset(
        pool,
        asset_id,
        job.source_subtitle_artifact_id,
    )
    .await
    .context("failed to load source subtitle artifact")?
    .ok_or_else(|| {
        anyhow::anyhow!(
            "source subtitle artifact {} not found for asset {}",
            job.source_subtitle_artifact_id,
            job.asset_id
        )
    })?;

    let bytes = storage
        .get(&source_artifact.storage_key)
        .await
        .with_context(|| {
            format!(
                "failed to load source subtitle from '{}'",
                source_artifact.storage_key
            )
        })?;

    let segments: Vec<LegacySubtitleSegment> =
        serde_json::from_slice(&bytes).context("failed to parse source subtitle segments")?;

    Ok(SourceSubtitle { segments })
}

struct Languages {
    source_lang: String,
    target_lang: String,
}

async fn resolve_languages(
    pool: &PgPool,
    project_id: ProjectId,
    target_language_id: Uuid,
) -> anyhow::Result<Languages> {
    let target_languages =
        dubbridge_db::target_language_repo::list_target_languages(pool, project_id)
            .await
            .context("failed to load target languages for project")?;

    let target_language = target_languages
        .into_iter()
        .find(|tl| tl.id == target_language_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "target language {target_language_id} not found for project {}",
                project_id.0
            )
        })?;

    Ok(Languages {
        source_lang: target_language.source_lang,
        target_lang: target_language.target_lang,
    })
}

async fn store_translated_subtitle_artifact(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    target_language_id: Uuid,
    source_subtitle_artifact_id: Uuid,
    translated_bytes: &[u8],
) -> anyhow::Result<dubbridge_domain::artifact::DerivedArtifact> {
    let asset_id_str = asset_id.to_string();
    let storage_key = translated_subtitle_key(&asset_id_str, &target_language_id.to_string());

    storage
        .put(&storage_key, translated_bytes.to_vec())
        .await
        .with_context(|| format!("failed to store translated subtitle at '{storage_key}'"))?;

    let artifact = translation_repo::insert_translated_subtitle_artifact(
        pool,
        asset_id,
        source_subtitle_artifact_id,
        &storage_key,
        "application/json",
        i64::try_from(translated_bytes.len())
            .context("translated subtitle exceeds i64 size limit")?,
        &checksum_hex(translated_bytes),
    )
    .await
    .context("failed to persist translated subtitle artifact")?;

    Ok(artifact)
}

async fn promote_and_verify_ready(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    job: &TranslationJob,
    translated_subtitle_artifact_id: Uuid,
) -> anyhow::Result<()> {
    translation_repo::promote_translation_ready(
        pool,
        project_id,
        asset_id,
        job.target_language_id,
        job.generation_request_id,
        translated_subtitle_artifact_id,
    )
    .await
    .context("failed to promote translation to ready")?;

    let ready_evidence = translation_repo::get_translation_readiness_evidence(
        pool,
        project_id,
        asset_id,
        job.target_language_id,
    )
    .await
    .context("failed to reload translation readiness evidence after promotion")?;

    if !ready_evidence.is_ready() {
        bail!("translation readiness evidence incomplete after promotion");
    }

    Ok(())
}

#[cfg(test)]
#[path = "translation_runtime_tests.rs"]
mod tests;
