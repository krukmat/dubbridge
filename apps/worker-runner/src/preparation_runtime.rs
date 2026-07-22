use anyhow::{Context, bail};
use async_trait::async_trait;
use dubbridge_db::preparation_repo;
use dubbridge_domain::{artifact::PreparationStatus, asset::AssetId};
use dubbridge_jobs::{JobEnvelope, PreparationJob, TranscriptionJobQueue};
use dubbridge_storage::StorageAdapter;
use sqlx::PgPool;

use crate::{
    preparation_artifact_persistence::{
        load_source_artifact, persist_hls_artifacts, persist_probe_artifact,
    },
    subtitle_enqueue::prepare_transcription_post_ready,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HlsSegmentOutput {
    pub(crate) file_name: String,
    pub(crate) bytes: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HlsPackageOutput {
    pub(crate) manifest_bytes: Vec<u8>,
    pub(crate) segments: Vec<HlsSegmentOutput>,
}

#[allow(dead_code)]
#[async_trait]
pub(crate) trait PreparationExecutor: Send + Sync {
    async fn extract_probe_metadata(&self, source_bytes: &[u8]) -> anyhow::Result<Vec<u8>>;
    async fn transcode_hls(&self, source_bytes: &[u8]) -> anyhow::Result<HlsPackageOutput>;
}

#[allow(dead_code)]
pub(crate) async fn process_preparation_envelope(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    executor: &dyn PreparationExecutor,
    queue: &dyn TranscriptionJobQueue,
    envelope: JobEnvelope<PreparationJob>,
) -> anyhow::Result<()> {
    if envelope.job_type != PreparationJob::JOB_TYPE {
        bail!(
            "unsupported preparation job type '{}', expected '{}'",
            envelope.job_type,
            PreparationJob::JOB_TYPE
        );
    }

    process_preparation_job(pool, storage, executor, queue, envelope.payload).await
}

#[allow(dead_code)]
pub(crate) async fn process_preparation_job(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    executor: &dyn PreparationExecutor,
    queue: &dyn TranscriptionJobQueue,
    job: PreparationJob,
) -> anyhow::Result<()> {
    let asset_id = AssetId(job.asset_id);
    let source_artifact_id = job.source_artifact_id;

    let result = process_preparation_job_inner(pool, storage, executor, &job).await;
    if let Err(error) = result {
        let detail = format!("{error:#}");
        preparation_repo::upsert_preparation_status(
            pool,
            asset_id,
            PreparationStatus::Failed,
            Some(&detail),
        )
        .await
        .context("failed to persist preparation failure status")?;
        return Err(error);
    }

    prepare_transcription_post_ready(pool, queue, asset_id, source_artifact_id).await;
    Ok(())
}

async fn process_preparation_job_inner(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    executor: &dyn PreparationExecutor,
    job: &PreparationJob,
) -> anyhow::Result<()> {
    let asset_id = AssetId(job.asset_id);
    let asset_id_string = asset_id.to_string();
    let source = load_source_artifact(pool, job).await?;

    preparation_repo::upsert_preparation_status(
        pool,
        asset_id,
        PreparationStatus::InProgress,
        None,
    )
    .await
    .context("failed to mark preparation in progress")?;

    let source_bytes = storage.get(&source.storage_key).await.with_context(|| {
        format!(
            "failed to load source artifact bytes from '{}'",
            source.storage_key
        )
    })?;

    let probe_bytes = executor
        .extract_probe_metadata(&source_bytes)
        .await
        .context("probe stage failed")?;
    persist_probe_artifact(pool, storage, asset_id, &asset_id_string, &probe_bytes).await?;

    let hls_output = executor
        .transcode_hls(&source_bytes)
        .await
        .context("HLS stage failed")?;
    persist_hls_artifacts(pool, storage, asset_id, &asset_id_string, &hls_output).await?;

    assert_preparation_readiness(pool, asset_id).await?;
    preparation_repo::upsert_preparation_status(pool, asset_id, PreparationStatus::Ready, None)
        .await
        .context("failed to mark preparation ready")?;
    Ok(())
}

async fn assert_preparation_readiness(pool: &PgPool, asset_id: AssetId) -> anyhow::Result<()> {
    let readiness = preparation_repo::get_preparation_readiness_evidence(pool, asset_id)
        .await
        .context("failed to load preparation readiness evidence")?;
    if readiness.is_ready() {
        return Ok(());
    }

    bail!(
        "preparation readiness evidence incomplete: probe={}, manifest={}, segments={}",
        readiness.probe_metadata_count,
        readiness.hls_manifest_count,
        readiness.hls_segment_count
    );
}
