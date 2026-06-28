use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, bail};
use async_trait::async_trait;
use dubbridge_db::{create_pool, preparation_repo, transcription_repo, workspace_repo};
use dubbridge_domain::{
    artifact::{ArtifactRecord, PreparationStatus, TranscriptionStatus},
    asset::AssetId,
};
use dubbridge_jobs::{JobEnvelope, PreparationJob, TranscriptionJob, TranscriptionJobQueue};
use dubbridge_media::{
    HLS_MANIFEST_FILE_NAME, HLS_SEGMENT_FILE_EXTENSION, canonical_ffprobe_json, ffmpeg_hls_command,
    ffprobe_command, validate_hls_outputs,
};
use dubbridge_providers::{AsrInput, AsrOutput, AsrWorkerClient};
use dubbridge_storage::{
    StorageAdapter, alignment_key, hls_manifest_key, hls_segment_key, probe_metadata_key,
    transcript_key,
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tempfile::TempDir;
use tokio::{fs, process::Command};

// T5b delivers the preparation runtime and its tests before the queue-consumer
// loop is wired into `main`; these items are intentionally live for the next
// integration step even though the current binary entrypoint still logs startup.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct HlsSegmentOutput {
    file_name: String,
    bytes: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct HlsPackageOutput {
    manifest_bytes: Vec<u8>,
    segments: Vec<HlsSegmentOutput>,
}

#[allow(dead_code)]
#[async_trait]
trait PreparationExecutor: Send + Sync {
    async fn extract_probe_metadata(&self, source_bytes: &[u8]) -> anyhow::Result<Vec<u8>>;
    async fn transcode_hls(&self, source_bytes: &[u8]) -> anyhow::Result<HlsPackageOutput>;
}

struct SubprocessPreparationExecutor;

#[async_trait]
impl PreparationExecutor for SubprocessPreparationExecutor {
    async fn extract_probe_metadata(&self, source_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
        let (_workspace, input_path) = write_source_workspace(source_bytes).await?;
        let command = ffprobe_command(path_to_string(&input_path)?);
        let output = run_command(&command).await?;
        let stdout =
            String::from_utf8(output.stdout).context("ffprobe stdout is not valid UTF-8")?;
        canonical_ffprobe_json(&stdout)
    }

    async fn transcode_hls(&self, source_bytes: &[u8]) -> anyhow::Result<HlsPackageOutput> {
        let (workspace, input_path) = write_source_workspace(source_bytes).await?;
        let output_dir = workspace.path().join("hls");
        fs::create_dir_all(&output_dir)
            .await
            .context("failed to create HLS output directory")?;

        let command =
            ffmpeg_hls_command(path_to_string(&input_path)?, path_to_string(&output_dir)?);
        run_command(&command).await?;

        let manifest_path = output_dir.join(HLS_MANIFEST_FILE_NAME);
        let manifest_bytes = fs::read(&manifest_path).await.with_context(|| {
            format!("failed to read HLS manifest at {}", manifest_path.display())
        })?;
        let manifest_raw =
            std::str::from_utf8(&manifest_bytes).context("HLS manifest is not valid UTF-8")?;
        let segments = read_hls_segments(&output_dir).await?;
        let segment_names = segments
            .iter()
            .map(|segment| segment.file_name.as_str())
            .collect::<Vec<_>>();
        validate_hls_outputs(manifest_raw, &segment_names)?;

        Ok(HlsPackageOutput {
            manifest_bytes,
            segments,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
    let pool = create_pool(&config.database_url)
        .await
        .context("failed to create database pool for worker runner")?;
    let storage_config = dubbridge_storage::StorageConfig::from(&config.storage);
    let storage = dubbridge_storage::build_adapter(&storage_config)
        .map_err(|e| anyhow::anyhow!("failed to initialize configured storage backend: {e}"))?;
    let storage_reference = storage.object_url("__startup_probe__");
    tracing::info!(
        env = ?config.env,
        log_format = ?config.observability.log_format,
        redis_url = %config.redis_url,
        worker_concurrency = config.worker_concurrency,
        storage_backend = ?config.storage.backend,
        storage_bucket = %config.storage.bucket,
        storage_reference = %storage_reference,
        "starting worker runner"
    );

    let queue = dubbridge_jobs::default_queue();
    let _executor = SubprocessPreparationExecutor;
    let _pool = pool;
    let _storage = storage;
    tracing::info!(queue = %queue, "worker runner preparation handler initialized");
    Ok(())
}

#[allow(dead_code)]
async fn process_preparation_envelope(
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
async fn process_preparation_job(
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

    // Preparation succeeded: fire-and-forget transcription enqueue.
    // Enqueue failure writes TranscriptionStatus::Failed but does not fail preparation.
    enqueue_transcription_if_ready(pool, queue, asset_id, source_artifact_id).await;

    Ok(())
}

/// Resolve source language and enqueue a TranscriptionJob after preparation succeeds.
///
/// All error paths write `TranscriptionStatus::Failed` with an observable detail and return
/// without propagating — preparation readiness is never jeopardised by enqueue failures.
#[allow(dead_code)]
async fn record_transcription_failure(pool: &PgPool, asset_id: AssetId, detail: &str) {
    let _ = transcription_repo::upsert_transcription_status(
        pool,
        asset_id,
        TranscriptionStatus::Failed,
        Some(detail),
    )
    .await;
}

async fn enqueue_transcription_if_ready(
    pool: &PgPool,
    queue: &dyn TranscriptionJobQueue,
    asset_id: AssetId,
    source_artifact_id: uuid::Uuid,
) {
    if let Err(e) = try_enqueue_transcription(pool, queue, asset_id, source_artifact_id).await {
        record_transcription_failure(pool, asset_id, &e).await;
    }
}

async fn transcription_already_underway(pool: &PgPool, asset_id: AssetId) -> Result<bool, String> {
    match transcription_repo::get_transcription_status(pool, asset_id).await {
        Ok(Some(r))
            if matches!(
                r.status,
                TranscriptionStatus::Pending
                    | TranscriptionStatus::InProgress
                    | TranscriptionStatus::Ready
            ) =>
        {
            Ok(true)
        }
        Ok(_) => Ok(false),
        Err(e) => {
            tracing::warn!(asset_id = %asset_id, error = %e, "failed to read transcription status; recording Failed");
            Err(e.to_string())
        }
    }
}

async fn resolve_source_language(pool: &PgPool, asset_id: AssetId) -> Result<String, String> {
    let result = workspace_repo::get_source_language_for_asset(pool, asset_id)
        .await
        .map_err(|e| {
            tracing::warn!(asset_id = %asset_id, error = %e, "failed to resolve source language");
            e.to_string()
        })?;
    result.ok_or_else(|| {
        let detail = "no target_languages row found for asset project";
        tracing::warn!(asset_id = %asset_id, detail, "transcription enqueue failed");
        detail.to_string()
    })
}

async fn try_enqueue_transcription(
    pool: &PgPool,
    queue: &dyn TranscriptionJobQueue,
    asset_id: AssetId,
    source_artifact_id: uuid::Uuid,
) -> Result<(), String> {
    if transcription_already_underway(pool, asset_id).await? {
        return Ok(());
    }

    let source_language = resolve_source_language(pool, asset_id).await?;

    // Write Pending before dispatching to the queue.
    transcription_repo::upsert_transcription_status(pool, asset_id, TranscriptionStatus::Pending, None)
        .await
        .map_err(|e| {
            tracing::warn!(asset_id = %asset_id, error = %e, "failed to write TranscriptionStatus::Pending");
            e.to_string()
        })?;

    // Enqueue.
    queue
        .enqueue(TranscriptionJob::new(
            asset_id.0,
            source_artifact_id,
            source_language,
        ))
        .map_err(|e| {
            tracing::warn!(asset_id = %asset_id, error = %e, "failed to enqueue TranscriptionJob");
            e.to_string()
        })
}

#[allow(dead_code)]
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

    let readiness = preparation_repo::get_preparation_readiness_evidence(pool, asset_id)
        .await
        .context("failed to load preparation readiness evidence")?;
    if !readiness.is_ready() {
        bail!(
            "preparation readiness evidence incomplete: probe={}, manifest={}, segments={}",
            readiness.probe_metadata_count,
            readiness.hls_manifest_count,
            readiness.hls_segment_count
        );
    }

    preparation_repo::upsert_preparation_status(pool, asset_id, PreparationStatus::Ready, None)
        .await
        .context("failed to mark preparation ready")?;

    Ok(())
}

#[allow(dead_code)]
async fn load_source_artifact(
    pool: &PgPool,
    job: &PreparationJob,
) -> anyhow::Result<ArtifactRecord> {
    let asset_id = AssetId(job.asset_id);
    let source = preparation_repo::find_source_artifact(pool, asset_id)
        .await
        .context("failed to load source artifact for preparation")?
        .ok_or_else(|| anyhow::anyhow!("source artifact missing for asset {asset_id}"))?;

    if source.id != job.source_artifact_id {
        bail!(
            "preparation job source artifact mismatch for asset {asset_id}: expected {}, found {}",
            job.source_artifact_id,
            source.id
        );
    }

    Ok(source)
}

#[allow(dead_code)]
async fn persist_probe_artifact(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    asset_id_string: &str,
    probe_bytes: &[u8],
) -> anyhow::Result<()> {
    let storage_key = probe_metadata_key(asset_id_string);
    storage
        .put(&storage_key, probe_bytes.to_vec())
        .await
        .with_context(|| format!("failed to store probe metadata at '{storage_key}'"))?;

    preparation_repo::insert_probe_metadata_artifact(
        pool,
        asset_id,
        &storage_key,
        i64::try_from(probe_bytes.len()).context("probe metadata exceeds i64 size limit")?,
        &checksum_hex(probe_bytes),
    )
    .await
    .context("failed to persist probe metadata artifact")?;

    Ok(())
}

#[allow(dead_code)]
async fn persist_hls_artifacts(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    asset_id_string: &str,
    hls_output: &HlsPackageOutput,
) -> anyhow::Result<()> {
    let manifest_raw = std::str::from_utf8(&hls_output.manifest_bytes)
        .context("HLS manifest is not valid UTF-8")?;
    let segment_names = hls_output
        .segments
        .iter()
        .map(|segment| segment.file_name.as_str())
        .collect::<Vec<_>>();
    validate_hls_outputs(manifest_raw, &segment_names).context("HLS output validation failed")?;

    let manifest_key = hls_manifest_key(asset_id_string);
    storage
        .put(&manifest_key, hls_output.manifest_bytes.clone())
        .await
        .with_context(|| format!("failed to store HLS manifest at '{manifest_key}'"))?;

    let mut segment_metadata = Vec::with_capacity(hls_output.segments.len());
    for segment in &hls_output.segments {
        let storage_key = hls_segment_key(asset_id_string, &segment.file_name);
        storage
            .put(&storage_key, segment.bytes.clone())
            .await
            .with_context(|| format!("failed to store HLS segment at '{storage_key}'"))?;
        segment_metadata.push((
            storage_key,
            i64::try_from(segment.bytes.len()).context("HLS segment exceeds i64 size limit")?,
            checksum_hex(&segment.bytes),
        ));
    }

    preparation_repo::insert_hls_artifacts(
        pool,
        asset_id,
        &manifest_key,
        i64::try_from(hls_output.manifest_bytes.len())
            .context("HLS manifest exceeds i64 size limit")?,
        &checksum_hex(&hls_output.manifest_bytes),
        &segment_metadata,
    )
    .await
    .context("failed to persist HLS derived artifacts")?;

    Ok(())
}

#[allow(dead_code)]
async fn write_source_workspace(source_bytes: &[u8]) -> anyhow::Result<(TempDir, PathBuf)> {
    let workspace = TempDir::new().context("failed to create temporary preparation workspace")?;
    let input_path = workspace.path().join("source-media.bin");
    fs::write(&input_path, source_bytes)
        .await
        .with_context(|| {
            format!(
                "failed to write temporary source file at {}",
                input_path.display()
            )
        })?;
    Ok((workspace, input_path))
}

#[allow(dead_code)]
async fn run_command(command: &[String]) -> anyhow::Result<std::process::Output> {
    let binary = command
        .first()
        .ok_or_else(|| anyhow::anyhow!("empty command"))?;
    let output = Command::new(binary)
        .args(&command[1..])
        .stdin(Stdio::null())
        .output()
        .await
        .with_context(|| format!("failed to spawn command '{}'", binary))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "command '{}' failed with status {}: {}",
            binary,
            output.status,
            stderr.trim()
        );
    }

    Ok(output)
}

#[allow(dead_code)]
async fn read_hls_segments(output_dir: &Path) -> anyhow::Result<Vec<HlsSegmentOutput>> {
    let mut entries = fs::read_dir(output_dir).await.with_context(|| {
        format!(
            "failed to read HLS output directory {}",
            output_dir.display()
        )
    })?;
    let mut segment_paths = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .context("failed while iterating HLS output directory")?
    {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| format!(".{ext}") == HLS_SEGMENT_FILE_EXTENSION)
        {
            segment_paths.push(path);
        }
    }
    segment_paths.sort();

    let mut segments = Vec::with_capacity(segment_paths.len());
    for path in segment_paths {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "HLS segment path '{}' has no valid file name",
                    path.display()
                )
            })?;
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("failed to read HLS segment at {}", path.display()))?;
        segments.push(HlsSegmentOutput { file_name, bytes });
    }

    Ok(segments)
}

#[allow(dead_code)]
fn path_to_string(path: &Path) -> anyhow::Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 path '{}'", path.display()))
}

#[allow(dead_code)]
fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[allow(dead_code)]
async fn process_transcription_envelope(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn AsrWorkerClient,
    envelope: JobEnvelope<TranscriptionJob>,
) -> anyhow::Result<()> {
    if envelope.job_type != TranscriptionJob::JOB_TYPE {
        bail!(
            "unsupported transcription job type '{}', expected '{}'",
            envelope.job_type,
            TranscriptionJob::JOB_TYPE
        );
    }
    process_transcription_job(pool, storage, client, envelope.payload).await
}

#[allow(dead_code)]
async fn process_transcription_job(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn AsrWorkerClient,
    job: TranscriptionJob,
) -> anyhow::Result<()> {
    let asset_id = AssetId(job.asset_id);

    let result = process_transcription_job_inner(pool, storage, client, &job).await;
    if let Err(error) = result {
        let detail = format!("{error:#}");
        let _ = transcription_repo::upsert_transcription_status(
            pool,
            asset_id,
            TranscriptionStatus::Failed,
            Some(&detail),
        )
        .await;
        return Err(error);
    }

    Ok(())
}

#[allow(dead_code)]
async fn process_transcription_job_inner(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn AsrWorkerClient,
    job: &TranscriptionJob,
) -> anyhow::Result<()> {
    let asset_id = AssetId(job.asset_id);
    let asset_id_str = asset_id.to_string();
    let source_artifact_id = job.source_artifact_id;

    transcription_repo::upsert_transcription_status(
        pool,
        asset_id,
        TranscriptionStatus::InProgress,
        None,
    )
    .await
    .context("failed to mark transcription in progress")?;

    // Download source audio bytes and write to a temp file for the ASR worker.
    let source_artifact = transcription_repo::get_source_artifact_for_transcription(
        pool,
        asset_id,
        source_artifact_id,
    )
    .await
    .context("failed to load source artifact for transcription")?;

    let audio_bytes = storage
        .get(&source_artifact.storage_key)
        .await
        .with_context(|| {
            format!(
                "failed to load source audio from '{}'",
                source_artifact.storage_key
            )
        })?;

    let workspace = TempDir::new().context("failed to create transcription workspace")?;
    let audio_path = workspace.path().join("source-audio.bin");
    fs::write(&audio_path, &audio_bytes)
        .await
        .context("failed to write audio temp file")?;
    let audio_uri = format!(
        "file://{}",
        audio_path.to_str().context("non-UTF-8 audio path")?
    );

    // Call the ASR client.
    let asr_output: AsrOutput = client
        .transcribe(AsrInput {
            job_id: job.asset_id.to_string(),
            audio_uri,
            language_hint: job.source_language.clone(),
        })
        .map_err(|e| anyhow::anyhow!("ASR worker error: {e}"))?;

    // Read transcript and alignment from the URIs returned by the worker.
    let transcript_bytes = read_file_uri(&asr_output.transcript_uri)
        .await
        .context("failed to read transcript URI")?;
    let alignment_bytes = read_file_uri(&asr_output.alignment_uri)
        .await
        .context("failed to read alignment URI")?;

    // Upload both to storage under canonical keys.
    let t_key = transcript_key(&asset_id_str);
    let a_key = alignment_key(&asset_id_str);

    storage
        .put(&t_key, transcript_bytes.clone())
        .await
        .with_context(|| format!("failed to store transcript at '{t_key}'"))?;
    storage
        .put(&a_key, alignment_bytes.clone())
        .await
        .with_context(|| format!("failed to store alignment at '{a_key}'"))?;

    // Persist both derived artifacts.
    transcription_repo::insert_transcript_artifacts(
        pool,
        asset_id,
        source_artifact_id,
        transcription_repo::TranscriptArtifactMeta {
            storage_key: &t_key,
            size_bytes: i64::try_from(transcript_bytes.len())
                .context("transcript exceeds i64 size limit")?,
            checksum: &checksum_hex(&transcript_bytes),
        },
        transcription_repo::TranscriptArtifactMeta {
            storage_key: &a_key,
            size_bytes: i64::try_from(alignment_bytes.len())
                .context("alignment exceeds i64 size limit")?,
            checksum: &checksum_hex(&alignment_bytes),
        },
    )
    .await
    .context("failed to persist transcript artifacts")?;

    // Gate Ready on evidence of both artifact kinds.
    let ready = transcription_repo::get_transcription_readiness_evidence(pool, asset_id)
        .await
        .context("failed to load transcription readiness evidence")?;

    if !ready {
        bail!("transcription readiness evidence incomplete after artifact insertion");
    }

    transcription_repo::upsert_transcription_status(
        pool,
        asset_id,
        TranscriptionStatus::Ready,
        None,
    )
    .await
    .context("failed to mark transcription ready")?;

    Ok(())
}

/// Read the content of a `file://` URI synchronously, returning the raw bytes.
#[allow(dead_code)]
async fn read_file_uri(uri: &str) -> anyhow::Result<Vec<u8>> {
    let path = uri
        .strip_prefix("file://")
        .ok_or_else(|| anyhow::anyhow!("unsupported URI scheme in '{uri}'"))?;
    fs::read(path)
        .await
        .with_context(|| format!("failed to read file at '{path}'"))
}

#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use super::*;
    use dubbridge_db::{artifact_repo, transcription_repo, workspace_repo};
    use dubbridge_domain::artifact::ArtifactKind;
    use dubbridge_domain::workspace::{OrgId, Organization, Project, ProjectId, TargetLanguage};
    use dubbridge_jobs::InMemoryTranscriptionJobQueue;
    use dubbridge_providers::{AsrError, AsrOutput, StubAsrWorkerClient};
    use dubbridge_storage::LocalFsAdapter;
    use time::OffsetDateTime;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    async fn setup_pool() -> Option<PgPool> {
        let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("../../infra/migrations")
            .run(&pool)
            .await
            .expect("migrations");
        sqlx::query(
            "TRUNCATE TABLE pending_ingestions, audit_events, artifact_records, rights_records, assets, asset_preparation_status, asset_transcription_status RESTART IDENTITY CASCADE",
        )
        .execute(&pool)
        .await
        .expect("truncate");
        Some(pool)
    }

    async fn insert_asset(pool: &PgPool) -> AssetId {
        let asset_id = AssetId::new();
        sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
            .bind(asset_id.0)
            .bind("test-asset")
            .bind(Uuid::new_v4())
            .bind("finalized")
            .execute(pool)
            .await
            .expect("insert asset");
        asset_id
    }

    async fn insert_source_artifact(pool: &PgPool, asset_id: AssetId) -> ArtifactRecord {
        let record = ArtifactRecord::new_original(
            asset_id,
            Uuid::new_v4(),
            format!("ingest/{asset_id}/source.mp4"),
            "video/mp4".into(),
            1024,
            "sourcesum".into(),
        );
        artifact_repo::insert_artifact_record(pool, &record)
            .await
            .expect("insert source artifact");
        record
    }

    fn valid_probe_bytes() -> Vec<u8> {
        canonical_ffprobe_json(
            r#"{
              "streams":[
                {"codec_type":"video","codec_name":"h264"},
                {"codec_type":"audio","codec_name":"aac"}
              ],
              "format":{"format_name":"mp4","duration":"10.000000"}
            }"#,
        )
        .expect("canonical probe bytes")
    }

    fn valid_hls_output() -> HlsPackageOutput {
        HlsPackageOutput {
            manifest_bytes: b"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:6
#EXT-X-PLAYLIST-TYPE:VOD
#EXTINF:6.0,
segment_00000.ts
#EXT-X-ENDLIST
"
            .to_vec(),
            segments: vec![HlsSegmentOutput {
                file_name: "segment_00000.ts".to_string(),
                bytes: b"segment-bytes".to_vec(),
            }],
        }
    }

    async fn assert_status(pool: &PgPool, asset_id: AssetId, expected: PreparationStatus) {
        let status = preparation_repo::get_preparation_status(pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, expected);
    }

    struct FakePreparationExecutor {
        pool: PgPool,
        asset_id: AssetId,
        stage_log: Arc<Mutex<Vec<&'static str>>>,
        probe_result: Result<Vec<u8>, String>,
        hls_result: Result<HlsPackageOutput, String>,
    }

    #[async_trait]
    impl PreparationExecutor for FakePreparationExecutor {
        async fn extract_probe_metadata(&self, _source_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
            assert_status(&self.pool, self.asset_id, PreparationStatus::InProgress).await;
            self.stage_log.lock().await.push("probe");
            self.probe_result.clone().map_err(anyhow::Error::msg)
        }

        async fn transcode_hls(&self, _source_bytes: &[u8]) -> anyhow::Result<HlsPackageOutput> {
            assert_status(&self.pool, self.asset_id, PreparationStatus::InProgress).await;
            self.stage_log.lock().await.push("hls");
            self.hls_result.clone().map_err(anyhow::Error::msg)
        }
    }

    #[tokio::test]
    async fn process_preparation_job_marks_ready_when_probe_and_hls_exist() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let workspace = TempDir::new().expect("temp dir");
        let storage = LocalFsAdapter::new(workspace.path());
        storage
            .put(&source.storage_key, b"source-media-bytes".to_vec())
            .await
            .expect("persist source bytes");
        preparation_repo::upsert_preparation_status(
            &pool,
            asset_id,
            PreparationStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        let stage_log = Arc::new(Mutex::new(Vec::new()));
        let executor = FakePreparationExecutor {
            pool: pool.clone(),
            asset_id,
            stage_log: stage_log.clone(),
            probe_result: Ok(valid_probe_bytes()),
            hls_result: Ok(valid_hls_output()),
        };

        let queue = InMemoryTranscriptionJobQueue::default();
        process_preparation_job(
            &pool,
            &storage,
            &executor,
            &queue,
            PreparationJob::new(asset_id.0, source.id, source.ingest_token),
        )
        .await
        .expect("process preparation job");

        assert_status(&pool, asset_id, PreparationStatus::Ready).await;
        let readiness = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness evidence");
        let derived = preparation_repo::list_derived_artifacts(&pool, asset_id)
            .await
            .expect("derived artifacts");

        assert!(readiness.is_ready());
        assert_eq!(readiness.probe_metadata_count, 1);
        assert_eq!(readiness.hls_manifest_count, 1);
        assert_eq!(readiness.hls_segment_count, 1);
        assert_eq!(derived.len(), 3);
        assert_eq!(stage_log.lock().await.as_slice(), &["probe", "hls"]);
    }

    #[tokio::test]
    async fn process_preparation_job_marks_failed_when_hls_stage_fails() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let workspace = TempDir::new().expect("temp dir");
        let storage = LocalFsAdapter::new(workspace.path());
        storage
            .put(&source.storage_key, b"source-media-bytes".to_vec())
            .await
            .expect("persist source bytes");
        preparation_repo::upsert_preparation_status(
            &pool,
            asset_id,
            PreparationStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        let executor = FakePreparationExecutor {
            pool: pool.clone(),
            asset_id,
            stage_log: Arc::new(Mutex::new(Vec::new())),
            probe_result: Ok(valid_probe_bytes()),
            hls_result: Err("ffmpeg transcode failed in fake executor".to_string()),
        };

        let queue = InMemoryTranscriptionJobQueue::default();
        let error = process_preparation_job(
            &pool,
            &storage,
            &executor,
            &queue,
            PreparationJob::new(asset_id.0, source.id, source.ingest_token),
        )
        .await
        .expect_err("HLS failure must fail job");

        assert!(error.to_string().contains("HLS stage failed"));
        let status = preparation_repo::get_preparation_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        let readiness = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness evidence");
        let derived = preparation_repo::list_derived_artifacts(&pool, asset_id)
            .await
            .expect("derived artifacts");

        assert_eq!(status.status, PreparationStatus::Failed);
        assert!(
            status
                .error_detail
                .as_deref()
                .expect("error detail")
                .contains("ffmpeg transcode failed in fake executor")
        );
        assert!(!readiness.is_ready());
        assert_eq!(readiness.probe_metadata_count, 1);
        assert_eq!(readiness.hls_manifest_count, 0);
        assert_eq!(readiness.hls_segment_count, 0);
        assert_eq!(derived.len(), 1);
        assert_eq!(derived[0].kind, ArtifactKind::ProbeMetadata);
    }

    #[tokio::test]
    async fn process_preparation_job_does_not_mark_ready_when_hls_output_is_invalid() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let workspace = TempDir::new().expect("temp dir");
        let storage = LocalFsAdapter::new(workspace.path());
        storage
            .put(&source.storage_key, b"source-media-bytes".to_vec())
            .await
            .expect("persist source bytes");
        preparation_repo::upsert_preparation_status(
            &pool,
            asset_id,
            PreparationStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        let executor = FakePreparationExecutor {
            pool: pool.clone(),
            asset_id,
            stage_log: Arc::new(Mutex::new(Vec::new())),
            probe_result: Ok(valid_probe_bytes()),
            hls_result: Ok(HlsPackageOutput {
                manifest_bytes: b"#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-ENDLIST\n".to_vec(),
                segments: Vec::new(),
            }),
        };

        let queue = InMemoryTranscriptionJobQueue::default();
        let error = process_preparation_job(
            &pool,
            &storage,
            &executor,
            &queue,
            PreparationJob::new(asset_id.0, source.id, source.ingest_token),
        )
        .await
        .expect_err("invalid HLS output must fail readiness gate");

        assert!(error.to_string().contains("HLS output validation failed"));
        let status = preparation_repo::get_preparation_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        let readiness = preparation_repo::get_preparation_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness evidence");

        assert_eq!(status.status, PreparationStatus::Failed);
        assert!(!readiness.is_ready());
        assert_eq!(readiness.probe_metadata_count, 1);
        assert_eq!(readiness.hls_manifest_count, 0);
        assert_eq!(readiness.hls_segment_count, 0);
    }

    #[tokio::test]
    async fn process_preparation_envelope_rejects_wrong_job_type() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let workspace = TempDir::new().expect("temp dir");
        let storage = LocalFsAdapter::new(workspace.path());
        let executor = FakePreparationExecutor {
            pool: pool.clone(),
            asset_id,
            stage_log: Arc::new(Mutex::new(Vec::new())),
            probe_result: Ok(valid_probe_bytes()),
            hls_result: Ok(valid_hls_output()),
        };

        let queue = InMemoryTranscriptionJobQueue::default();
        let error = process_preparation_envelope(
            &pool,
            &storage,
            &executor,
            &queue,
            JobEnvelope::new(
                "other-job",
                PreparationJob::new(asset_id.0, source.id, source.ingest_token),
            ),
        )
        .await
        .expect_err("unexpected job type must fail");

        assert!(
            error
                .to_string()
                .contains("unsupported preparation job type")
        );
    }

    // ── T2 enqueue tests ─────────────────────────────────────────────────────────

    async fn insert_project_with_target_language(
        pool: &PgPool,
        asset_id: AssetId,
        source_lang: &str,
    ) {
        let org_id = OrgId(Uuid::new_v4());
        let org = Organization {
            id: org_id,
            name: "test-org".into(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };
        workspace_repo::insert_org(pool, &org)
            .await
            .expect("insert org");

        let project_id = ProjectId(Uuid::new_v4());
        let project = Project {
            id: project_id,
            org_id,
            name: "test-project".into(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };
        workspace_repo::insert_project(pool, &project)
            .await
            .expect("insert project");

        sqlx::query(
            "INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .expect("link asset to project");

        let tl = TargetLanguage {
            id: Uuid::new_v4(),
            project_id,
            source_lang: source_lang.into(),
            target_lang: "es".into(),
            created_at: OffsetDateTime::now_utc(),
        };
        workspace_repo::upsert_target_language(pool, &tl)
            .await
            .expect("insert target language");
    }

    async fn run_full_preparation(
        pool: &PgPool,
        asset_id: AssetId,
        source: &ArtifactRecord,
        queue: &dyn TranscriptionJobQueue,
    ) {
        let workspace = TempDir::new().expect("temp dir");
        let storage = LocalFsAdapter::new(workspace.path());
        storage
            .put(&source.storage_key, b"source-media-bytes".to_vec())
            .await
            .expect("persist source bytes");
        preparation_repo::upsert_preparation_status(
            pool,
            asset_id,
            PreparationStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        let executor = FakePreparationExecutor {
            pool: pool.clone(),
            asset_id,
            stage_log: Arc::new(Mutex::new(Vec::new())),
            probe_result: Ok(valid_probe_bytes()),
            hls_result: Ok(valid_hls_output()),
        };

        process_preparation_job(
            pool,
            &storage,
            &executor,
            queue,
            PreparationJob::new(asset_id.0, source.id, source.ingest_token),
        )
        .await
        .expect("process preparation job");
    }

    // HP-1: preparation completion enqueues exactly one TranscriptionJob with the resolved source_language.
    #[tokio::test]
    async fn preparation_ready_enqueues_transcription_job_with_source_language() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        insert_project_with_target_language(&pool, asset_id, "en").await;

        let queue = InMemoryTranscriptionJobQueue::default();
        run_full_preparation(&pool, asset_id, &source, &queue).await;

        let jobs = queue.queued_jobs();
        assert_eq!(jobs.len(), 1, "exactly one job should be enqueued");
        assert_eq!(jobs[0].asset_id, asset_id.0);
        assert_eq!(jobs[0].source_artifact_id, source.id);
        assert_eq!(jobs[0].source_language, "en");
    }

    // HP-2: preparation completion writes TranscriptionStatus::Pending before dispatching.
    #[tokio::test]
    async fn preparation_ready_writes_transcription_pending_status() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        insert_project_with_target_language(&pool, asset_id, "pt-BR").await;

        let queue = InMemoryTranscriptionJobQueue::default();
        run_full_preparation(&pool, asset_id, &source, &queue).await;

        let status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get transcription status")
            .expect("status row present");

        assert_eq!(status.status, TranscriptionStatus::Pending);
        assert!(status.error_detail.is_none());
    }

    // HP-3: resolved source_language matches target_languages.source_lang for the asset's project.
    #[tokio::test]
    async fn enqueued_source_language_matches_target_languages_row() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        insert_project_with_target_language(&pool, asset_id, "fr").await;

        let queue = InMemoryTranscriptionJobQueue::default();
        run_full_preparation(&pool, asset_id, &source, &queue).await;

        let jobs = queue.queued_jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].source_language, "fr");
    }

    // EC-1: queue enqueue failure produces TranscriptionStatus::Failed with observable error_detail.
    #[tokio::test]
    async fn enqueue_failure_records_transcription_failed_status() {
        use dubbridge_jobs::QueueError;

        let Some(pool) = setup_pool().await else {
            return;
        };

        struct FailingQueue;
        impl TranscriptionJobQueue for FailingQueue {
            fn enqueue(&self, _job: TranscriptionJob) -> Result<(), QueueError> {
                Err(QueueError::Unavailable("queue is down".into()))
            }
        }

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        insert_project_with_target_language(&pool, asset_id, "en").await;

        run_full_preparation(&pool, asset_id, &source, &FailingQueue).await;

        // Preparation must still be Ready.
        assert_status(&pool, asset_id, PreparationStatus::Ready).await;

        let ts = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");

        assert_eq!(ts.status, TranscriptionStatus::Failed);
        assert!(
            ts.error_detail
                .as_deref()
                .unwrap_or("")
                .contains("queue is down")
        );
    }

    // EC-2: asset with no target_languages row produces TranscriptionStatus::Failed; preparation stays Ready.
    #[tokio::test]
    async fn missing_target_languages_row_records_transcription_failed() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        // Intentionally no project or target_languages row.

        let queue = InMemoryTranscriptionJobQueue::default();
        run_full_preparation(&pool, asset_id, &source, &queue).await;

        assert_status(&pool, asset_id, PreparationStatus::Ready).await;

        let ts = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");

        assert_eq!(ts.status, TranscriptionStatus::Failed);
        assert!(
            ts.error_detail
                .as_deref()
                .unwrap_or("")
                .contains("no target_languages row")
        );
        assert!(queue.queued_jobs().is_empty());
    }

    // EC-3: duplicate preparation completion does not enqueue a second TranscriptionJob.
    #[tokio::test]
    async fn duplicate_preparation_completion_does_not_enqueue_second_job() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        insert_project_with_target_language(&pool, asset_id, "en").await;

        let queue = InMemoryTranscriptionJobQueue::default();
        run_full_preparation(&pool, asset_id, &source, &queue).await;
        assert_eq!(queue.queued_jobs().len(), 1);

        // Second completion: transcription is already Pending; hook must skip.
        enqueue_transcription_if_ready(&pool, &queue, asset_id, source.id).await;

        assert_eq!(
            queue.queued_jobs().len(),
            1,
            "no second job should be enqueued"
        );
    }

    // ── T3 process_transcription_job tests ───────────────────────────────────────

    fn stub_asr_output(
        asset_id: Uuid,
        workspace: &tempfile::TempDir,
    ) -> (AsrOutput, Vec<u8>, Vec<u8>) {
        let transcript_path = workspace.path().join("transcript.json");
        let alignment_path = workspace.path().join("alignment.json");
        let t_bytes = br#"{"words":[]}"#.to_vec();
        let a_bytes = br#"{"segments":[]}"#.to_vec();
        std::fs::write(&transcript_path, &t_bytes).expect("write transcript");
        std::fs::write(&alignment_path, &a_bytes).expect("write alignment");
        let output = AsrOutput {
            job_id: asset_id.to_string(),
            transcript_uri: format!("file://{}", transcript_path.display()),
            alignment_uri: format!("file://{}", alignment_path.display()),
            status: "ok".into(),
        };
        (output, t_bytes, a_bytes)
    }

    // HP-1: successful ASR call stores both artifacts and writes TranscriptionStatus::Ready.
    #[tokio::test]
    async fn process_transcription_job_marks_ready_when_both_artifacts_stored() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let asr_workspace = TempDir::new().expect("asr workspace");
        let storage_workspace = TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());
        storage
            .put(&source.storage_key, b"audio-bytes".to_vec())
            .await
            .expect("store source audio");

        transcription_repo::upsert_transcription_status(
            &pool,
            asset_id,
            TranscriptionStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        let (asr_output, _t, _a) = stub_asr_output(asset_id.0, &asr_workspace);
        let client = StubAsrWorkerClient::ok(asr_output);

        process_transcription_job(
            &pool,
            &storage,
            &client,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect("process transcription job");

        let status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, TranscriptionStatus::Ready);
        assert!(status.error_detail.is_none());

        let ready = transcription_repo::get_transcription_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness");
        assert!(ready);
    }

    // HP-2: both artifacts are stored with the correct parent_artifact_id lineage.
    #[tokio::test]
    async fn process_transcription_job_artifacts_have_correct_lineage() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let asr_workspace = TempDir::new().expect("asr workspace");
        let storage_workspace = TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());
        storage
            .put(&source.storage_key, b"audio-bytes".to_vec())
            .await
            .expect("store source");

        transcription_repo::upsert_transcription_status(
            &pool,
            asset_id,
            TranscriptionStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        let (asr_output, _t, _a) = stub_asr_output(asset_id.0, &asr_workspace);
        let client = StubAsrWorkerClient::ok(asr_output);

        process_transcription_job(
            &pool,
            &storage,
            &client,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect("process transcription job");

        let derived = preparation_repo::list_derived_artifacts(&pool, asset_id)
            .await
            .expect("list derived artifacts");

        let transcript = derived
            .iter()
            .find(|a| a.kind == ArtifactKind::TranscriptText)
            .expect("transcript artifact");
        let alignment = derived
            .iter()
            .find(|a| a.kind == ArtifactKind::WordAlignment)
            .expect("alignment artifact");

        assert_eq!(transcript.parent_artifact_id, source.id);
        assert_eq!(alignment.parent_artifact_id, source.id);
    }

    // HP-3: status transitions InProgress → Ready in order.
    #[tokio::test]
    async fn process_transcription_job_transitions_in_progress_then_ready() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let asr_workspace = TempDir::new().expect("asr workspace");
        let storage_workspace = TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());
        storage
            .put(&source.storage_key, b"audio-bytes".to_vec())
            .await
            .expect("store source");

        transcription_repo::upsert_transcription_status(
            &pool,
            asset_id,
            TranscriptionStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        // Use a spy: the StubAsrWorkerClient is called synchronously from within
        // the handler; by the time the call returns the status must be InProgress.
        // We verify Ready at the end.
        let (asr_output, _t, _a) = stub_asr_output(asset_id.0, &asr_workspace);
        let client = StubAsrWorkerClient::ok(asr_output);

        process_transcription_job(
            &pool,
            &storage,
            &client,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect("process transcription job");

        let status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, TranscriptionStatus::Ready);
    }

    // EC-1: ASR worker error produces TranscriptionStatus::Failed with error detail.
    #[tokio::test]
    async fn process_transcription_job_marks_failed_on_asr_error() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let storage_workspace = TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());
        storage
            .put(&source.storage_key, b"audio-bytes".to_vec())
            .await
            .expect("store source");

        transcription_repo::upsert_transcription_status(
            &pool,
            asset_id,
            TranscriptionStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        let client = StubAsrWorkerClient::err(AsrError {
            job_id: asset_id.0.to_string(),
            error_code: "MODEL_LOAD_FAILED".into(),
            message: "whisper model not found".into(),
        });

        let err = process_transcription_job(
            &pool,
            &storage,
            &client,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect_err("ASR error must fail the job");

        assert!(err.to_string().contains("ASR worker error"));

        let status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, TranscriptionStatus::Failed);
        assert!(
            status
                .error_detail
                .as_deref()
                .unwrap_or("")
                .contains("whisper model not found")
        );
    }

    // EC-2: storage upload failure after successful ASR call produces Failed.
    #[tokio::test]
    async fn process_transcription_job_marks_failed_on_storage_error() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        // Use a read-only directory as storage root so put() fails.
        let storage_root = TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_root.path());
        // Write the source audio so the first get() succeeds.
        storage
            .put(&source.storage_key, b"audio-bytes".to_vec())
            .await
            .expect("store source");

        transcription_repo::upsert_transcription_status(
            &pool,
            asset_id,
            TranscriptionStatus::Pending,
            None,
        )
        .await
        .expect("set pending");

        // ASR output points to a non-existent file → read_file_uri will fail.
        let client = StubAsrWorkerClient::ok(AsrOutput {
            job_id: asset_id.0.to_string(),
            transcript_uri: "file:///nonexistent/transcript.json".into(),
            alignment_uri: "file:///nonexistent/alignment.json".into(),
            status: "ok".into(),
        });

        let err = process_transcription_job(
            &pool,
            &storage,
            &client,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect_err("storage error must fail the job");

        assert!(err.to_string().contains("transcript URI"));

        let status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, TranscriptionStatus::Failed);
    }

    // EC-3: wrong envelope job type is rejected.
    #[tokio::test]
    async fn process_transcription_envelope_rejects_wrong_job_type() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let storage_workspace = TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());

        let client = StubAsrWorkerClient::ok(AsrOutput {
            job_id: asset_id.0.to_string(),
            transcript_uri: "file:///ignored".into(),
            alignment_uri: "file:///ignored".into(),
            status: "ok".into(),
        });

        let err = process_transcription_envelope(
            &pool,
            &storage,
            &client,
            JobEnvelope::new(
                "media_preparation",
                TranscriptionJob::new(asset_id.0, source.id, "en"),
            ),
        )
        .await
        .expect_err("wrong job type must fail");

        assert!(
            err.to_string()
                .contains("unsupported transcription job type")
        );
    }
}
