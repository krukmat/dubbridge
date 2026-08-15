use dubbridge_providers::SegmentationProvider;

#[allow(dead_code)]
pub(crate) async fn process_subtitle_envelope(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    envelope: dubbridge_jobs::JobEnvelope<dubbridge_jobs::SubtitleJob>,
) -> anyhow::Result<()> {
    if envelope.job_type != dubbridge_jobs::SubtitleJob::JOB_TYPE {
        anyhow::bail!(
            "unsupported subtitle job type '{}', expected '{}'",
            envelope.job_type,
            dubbridge_jobs::SubtitleJob::JOB_TYPE
        );
    }

    process_subtitle_job(pool, storage, envelope.payload).await
}

#[allow(dead_code)]
pub(crate) async fn process_subtitle_job(
    pool: &sqlx::PgPool,
    storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync),
    job: dubbridge_jobs::SubtitleJob,
) -> anyhow::Result<()> {
    let asset_id = dubbridge_domain::asset::AssetId(job.asset_id);

    let result = process_subtitle_job_inner(pool, storage, &job).await;
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

    dispatch_post_ready(pool, asset_id, job).await?;

    Ok(())
}

async fn dispatch_post_ready(
    pool: &sqlx::PgPool,
    asset_id: dubbridge_domain::asset::AssetId,
    job: &dubbridge_jobs::SubtitleJob,
) -> anyhow::Result<()> {
    let Some(route) =
        dubbridge_db::target_language_repo::get_asset_subtitle_route(pool, asset_id).await?
    else {
        return Ok(());
    };
    crate::review_enqueue::prepare_review_post_ready(
        pool,
        asset_id,
        dubbridge_domain::workspace::ProjectId(job.project_id),
        &route.target_language,
    )
    .await;
    Ok(())
}

use crate::subtitle_alignment::{RawAlignmentFile, raw_words_to_provider};

fn checksum_hex(bytes: &[u8]) -> String {
    crate::checksum_hex(bytes)
}
