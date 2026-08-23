use dubbridge_jobs::TranslationJobQueue;
use dubbridge_providers::SegmentationProvider;

#[allow(dead_code)]
pub(crate) async fn process_subtitle_envelope(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    translation_queue: &(dyn TranslationJobQueue + Send + Sync),
    envelope: dubbridge_jobs::JobEnvelope<dubbridge_jobs::SubtitleJob>,
) -> anyhow::Result<()> {
    if envelope.job_type != dubbridge_jobs::SubtitleJob::JOB_TYPE {
        anyhow::bail!(
            "unsupported subtitle job type '{}', expected '{}'",
            envelope.job_type,
            dubbridge_jobs::SubtitleJob::JOB_TYPE
        );
    }

    process_subtitle_job(pool, storage, translation_queue, envelope.payload).await
}

#[allow(dead_code)]
pub(crate) async fn process_subtitle_job(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    translation_queue: &(dyn TranslationJobQueue + Send + Sync),
    job: dubbridge_jobs::SubtitleJob,
) -> anyhow::Result<()> {
    let asset_id = dubbridge_domain::asset::AssetId(job.asset_id);

    let result = process_subtitle_job_inner(pool, storage, translation_queue, &job).await;
    if let Err(error) = result {
        let detail = format!("{error:#}");
        let _ = dubbridge_db::subtitle_repo::upsert_subtitle_status(
            pool,
            asset_id,
            dubbridge_domain::artifact::SubtitleStatus::Failed,
            Some(&detail),
        )
        .await;
        return Err(error);
    }

    Ok(())
}

async fn process_subtitle_job_inner(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    translation_queue: &(dyn TranslationJobQueue + Send + Sync),
    job: &dubbridge_jobs::SubtitleJob,
) -> anyhow::Result<()> {
    let asset_id = dubbridge_domain::asset::AssetId(job.asset_id);

    dubbridge_db::subtitle_repo::upsert_subtitle_status(
        pool,
        asset_id,
        dubbridge_domain::artifact::SubtitleStatus::InProgress,
        None,
    )
    .await?;

    let artifacts = dubbridge_db::preparation_repo::list_derived_artifacts(pool, asset_id).await?;
    let alignment_artifact = artifacts
        .into_iter()
        .rev()
        .find(|a| a.kind == dubbridge_domain::artifact::ArtifactKind::WordAlignment)
        .ok_or_else(|| {
            anyhow::anyhow!("missing upstream word alignment for asset {}", asset_id.0)
        })?;

    let bytes = storage.get(&alignment_artifact.storage_key).await?;
    let raw: RawAlignmentFile = serde_json::from_slice(&bytes)?;
    let words = raw_words_to_provider(&raw.words);
    let segments = dubbridge_providers::RustSegmentationProvider
        .segment(&words)
        .map_err(|e| anyhow::anyhow!("segmentation failed: {}", e.message))?;

    let subtitle_bytes = serde_json::to_vec(&segments)?;
    storage
        .put(
            &dubbridge_storage::subtitle_key(&job.asset_id.to_string()),
            subtitle_bytes.clone(),
        )
        .await?;

    dubbridge_db::subtitle_repo::insert_subtitle_artifact(
        pool,
        asset_id,
        alignment_artifact.id,
        &dubbridge_storage::subtitle_key(&job.asset_id.to_string()),
        "application/json",
        subtitle_bytes.len() as i64,
        &checksum_hex(&subtitle_bytes),
    )
    .await?;

    dubbridge_db::subtitle_repo::upsert_subtitle_status(
        pool,
        asset_id,
        dubbridge_domain::artifact::SubtitleStatus::Ready,
        None,
    )
    .await?;

    let ready =
        dubbridge_db::subtitle_repo::get_subtitle_readiness_evidence(pool, asset_id).await?;
    if !ready {
        anyhow::bail!("subtitle readiness evidence incomplete after Ready status write");
    }

    dispatch_post_ready(pool, asset_id, translation_queue, alignment_artifact.id).await?;

    Ok(())
}

async fn dispatch_post_ready(
    pool: &sqlx::PgPool,
    asset_id: dubbridge_domain::asset::AssetId,
    translation_queue: &(dyn TranslationJobQueue + Send + Sync),
    word_alignment_parent_artifact_id: uuid::Uuid,
) -> anyhow::Result<()> {
    let jobs = crate::translation_fanout::fan_out_localization(
        pool,
        asset_id,
        word_alignment_parent_artifact_id,
    )
    .await
    .map_err(|e| anyhow::anyhow!("localization fan-out failed: {e}"))?;

    for job in jobs {
        enqueue_translation_job(pool, translation_queue, &job).await;
    }

    Ok(())
}

/// Enqueue one translation job, isolating this target's failure from its
/// siblings: a queue error is recorded via `translation_dispatch_enqueue_failure`
/// and logged, never propagated, so one unreachable-Redis target cannot abort
/// the fan-out loop or discard already-enqueued sibling dispatches.
async fn enqueue_translation_job(
    pool: &sqlx::PgPool,
    translation_queue: &(dyn TranslationJobQueue + Send + Sync),
    job: &dubbridge_jobs::TranslationJob,
) {
    match translation_queue.enqueue(job.clone()).await {
        Ok(()) => record_translation_dispatch_acknowledged(pool, job).await,
        Err(error) => record_translation_dispatch_enqueue_failed(pool, job, &error).await,
    }
}

async fn record_translation_dispatch_acknowledged(
    pool: &sqlx::PgPool,
    job: &dubbridge_jobs::TranslationJob,
) {
    let ack = dubbridge_db::translation_delivery_repo::translation_dispatch_acknowledge(
        pool,
        dubbridge_db::translation_delivery_repo::TranslationDispatchAcknowledgementInput {
            project_id: dubbridge_domain::workspace::ProjectId(job.project_id),
            asset_id: dubbridge_domain::asset::AssetId(job.asset_id),
            target_language_id: job.target_language_id,
            generation_request_id: job.generation_request_id,
        },
    )
    .await;
    if let Err(error) = ack {
        tracing::warn!(
            ?error,
            asset_id = %job.asset_id,
            target_language_id = %job.target_language_id,
            "failed to acknowledge enqueued translation dispatch"
        );
    }
}

async fn record_translation_dispatch_enqueue_failed(
    pool: &sqlx::PgPool,
    job: &dubbridge_jobs::TranslationJob,
    error: &dubbridge_jobs::QueueError,
) {
    tracing::warn!(
        ?error,
        asset_id = %job.asset_id,
        target_language_id = %job.target_language_id,
        "failed to enqueue translation job; marking dispatch enqueue_failed"
    );
    let mark = dubbridge_db::translation_delivery_repo::translation_dispatch_enqueue_failure(
        pool,
        dubbridge_db::translation_delivery_repo::TranslationDispatchFailureInput {
            project_id: dubbridge_domain::workspace::ProjectId(job.project_id),
            asset_id: dubbridge_domain::asset::AssetId(job.asset_id),
            target_language_id: job.target_language_id,
            generation_request_id: job.generation_request_id,
            error_detail: format!("{error:#}"),
        },
    )
    .await;
    warn_on_enqueue_failure_record_error(job, mark);
}

fn warn_on_enqueue_failure_record_error(
    job: &dubbridge_jobs::TranslationJob,
    mark: Result<
        dubbridge_db::translation_delivery_repo::TranslationDispatchFailureResult,
        dubbridge_db::error::DbError,
    >,
) {
    if let Err(db_error) = mark {
        tracing::warn!(
            ?db_error,
            asset_id = %job.asset_id,
            target_language_id = %job.target_language_id,
            "failed to record translation dispatch enqueue failure"
        );
    }
}

use crate::subtitle_alignment::{RawAlignmentFile, raw_words_to_provider};

fn checksum_hex(bytes: &[u8]) -> String {
    crate::checksum_hex(bytes)
}
