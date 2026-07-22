use anyhow::{Context, bail};
use dubbridge_db::transcription_repo;
use dubbridge_domain::{
    artifact::TranscriptionStatus,
    asset::AssetId,
};
use dubbridge_jobs::{JobEnvelope, SubtitleJobQueue, TranscriptionJob};
use dubbridge_providers::{AsrInput, AsrOutput, AsrWorkerClient};
use dubbridge_storage::{StorageAdapter, alignment_key, transcript_key};
use sqlx::PgPool;
use tempfile::TempDir;
use tokio::fs;

use crate::{checksum_hex, subtitle_enqueue};

#[allow(dead_code)]
pub(crate) async fn process_transcription_envelope(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn AsrWorkerClient,
    subtitle_queue: &dyn SubtitleJobQueue,
    envelope: JobEnvelope<TranscriptionJob>,
) -> anyhow::Result<()> {
    if envelope.job_type != TranscriptionJob::JOB_TYPE {
        bail!(
            "unsupported transcription job type '{}', expected '{}'",
            envelope.job_type,
            TranscriptionJob::JOB_TYPE
        );
    }

    process_transcription_job(pool, storage, client, subtitle_queue, envelope.payload).await
}

#[allow(dead_code)]
pub(crate) async fn process_transcription_job(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn AsrWorkerClient,
    subtitle_queue: &dyn SubtitleJobQueue,
    job: TranscriptionJob,
) -> anyhow::Result<()> {
    let asset_id = AssetId(job.asset_id);

    let result =
        process_transcription_job_inner(pool, storage, client, subtitle_queue, &job).await;
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

async fn process_transcription_job_inner(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    client: &dyn AsrWorkerClient,
    subtitle_queue: &dyn SubtitleJobQueue,
    job: &TranscriptionJob,
) -> anyhow::Result<()> {
    let asset_id = AssetId(job.asset_id);

    transcription_repo::upsert_transcription_status(
        pool,
        asset_id,
        TranscriptionStatus::InProgress,
        None,
    )
    .await
    .context("failed to mark transcription in progress")?;

    let source = load_source_audio(pool, storage, job).await?;
    let workspace = write_audio_workspace(&source.audio_bytes).await?;
    let asr_output = transcribe_source(client, job, &workspace.audio_uri)?;
    let artifact_bytes = read_asr_artifacts(&asr_output).await?;

    store_transcription_artifacts(
        pool,
        storage,
        source.asset_id,
        source.source_artifact_id,
        &artifact_bytes.transcript_bytes,
        &artifact_bytes.alignment_bytes,
    )
    .await?;
    ensure_transcription_ready(pool, source.asset_id).await?;

    subtitle_enqueue::prepare_subtitle_post_ready(pool, subtitle_queue, source.asset_id).await;

    Ok(())
}

struct SourceAudio {
    asset_id: AssetId,
    source_artifact_id: uuid::Uuid,
    audio_bytes: Vec<u8>,
}

async fn load_source_audio(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    job: &TranscriptionJob,
) -> anyhow::Result<SourceAudio> {
    let asset_id = AssetId(job.asset_id);
    let source_artifact_id = job.source_artifact_id;
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

    Ok(SourceAudio {
        asset_id,
        source_artifact_id,
        audio_bytes,
    })
}

struct AudioWorkspace {
    _workspace: TempDir,
    audio_uri: String,
}

async fn write_audio_workspace(audio_bytes: &[u8]) -> anyhow::Result<AudioWorkspace> {
    let workspace = TempDir::new().context("failed to create transcription workspace")?;
    let audio_path = workspace.path().join("source-audio.bin");
    fs::write(&audio_path, audio_bytes)
        .await
        .context("failed to write audio temp file")?;

    Ok(AudioWorkspace {
        _workspace: workspace,
        audio_uri: format!(
            "file://{}",
            audio_path.to_str().context("non-UTF-8 audio path")?
        ),
    })
}

fn transcribe_source(
    client: &dyn AsrWorkerClient,
    job: &TranscriptionJob,
    audio_uri: &str,
) -> anyhow::Result<AsrOutput> {
    client
        .transcribe(AsrInput {
            job_id: job.asset_id.to_string(),
            audio_uri: audio_uri.to_string(),
            language_hint: job.source_language.clone(),
        })
        .map_err(|e| anyhow::anyhow!("ASR worker error: {e}"))
}

struct AsrArtifactBytes {
    transcript_bytes: Vec<u8>,
    alignment_bytes: Vec<u8>,
}

async fn read_asr_artifacts(output: &AsrOutput) -> anyhow::Result<AsrArtifactBytes> {
    let transcript_bytes = read_file_uri(&output.transcript_uri)
        .await
        .context("failed to read transcript URI")?;
    let alignment_bytes = read_file_uri(&output.alignment_uri)
        .await
        .context("failed to read alignment URI")?;

    Ok(AsrArtifactBytes {
        transcript_bytes,
        alignment_bytes,
    })
}

async fn store_transcription_artifacts(
    pool: &PgPool,
    storage: &(dyn StorageAdapter + Send + Sync),
    asset_id: AssetId,
    source_artifact_id: uuid::Uuid,
    transcript_bytes: &[u8],
    alignment_bytes: &[u8],
) -> anyhow::Result<()> {
    let asset_id_str = asset_id.to_string();
    let transcript_key = transcript_key(&asset_id_str);
    let alignment_key = alignment_key(&asset_id_str);

    storage
        .put(&transcript_key, transcript_bytes.to_vec())
        .await
        .with_context(|| format!("failed to store transcript at '{transcript_key}'"))?;
    storage
        .put(&alignment_key, alignment_bytes.to_vec())
        .await
        .with_context(|| format!("failed to store alignment at '{alignment_key}'"))?;

    transcription_repo::insert_transcript_artifacts(
        pool,
        asset_id,
        source_artifact_id,
        transcription_repo::TranscriptArtifactMeta {
            storage_key: &transcript_key,
            size_bytes: i64::try_from(transcript_bytes.len())
                .context("transcript exceeds i64 size limit")?,
            checksum: &checksum_hex(transcript_bytes),
        },
        transcription_repo::TranscriptArtifactMeta {
            storage_key: &alignment_key,
            size_bytes: i64::try_from(alignment_bytes.len())
                .context("alignment exceeds i64 size limit")?,
            checksum: &checksum_hex(alignment_bytes),
        },
    )
    .await
    .context("failed to persist transcript artifacts")?;

    Ok(())
}

async fn ensure_transcription_ready(pool: &PgPool, asset_id: AssetId) -> anyhow::Result<()> {
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
    use std::env;

    use super::*;
    use dubbridge_db::{artifact_repo, subtitle_repo, workspace_repo};
    use dubbridge_domain::{
        artifact::{ArtifactKind, ArtifactRecord, SubtitleStatus},
        workspace::{OrgId, Organization, Project, ProjectId, TargetLanguage},
    };
    use dubbridge_jobs::{InMemorySubtitleJobQueue, JobEnvelope, QueueError};
    use dubbridge_providers::{AsrError, StubAsrWorkerClient};
    use dubbridge_storage::LocalFsAdapter;
    use time::OffsetDateTime;
    use uuid::Uuid;

    async fn setup_pool() -> Option<PgPool> {
        let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("../../infra/migrations")
            .run(&pool)
            .await
            .expect("migrations");
        sqlx::query(
            "TRUNCATE TABLE pending_ingestions, audit_events, artifact_records, rights_records, assets, asset_preparation_status, asset_transcription_status, asset_subtitle_status RESTART IDENTITY CASCADE",
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

    async fn insert_project_with_targets(
        pool: &PgPool,
        asset_id: AssetId,
        source_lang: &str,
        target_langs: &[&str],
    ) -> ProjectId {
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

        for target_lang in target_langs {
            workspace_repo::upsert_target_language(
                pool,
                &TargetLanguage {
                    id: Uuid::new_v4(),
                    project_id,
                    source_lang: source_lang.into(),
                    target_lang: (*target_lang).into(),
                    created_at: OffsetDateTime::now_utc(),
                },
            )
            .await
            .expect("insert target language");
        }

        project_id
    }

    fn stub_asr_output(
        asset_id: Uuid,
        workspace: &tempfile::TempDir,
    ) -> (AsrOutput, Vec<u8>, Vec<u8>) {
        let transcript_path = workspace.path().join("transcript.json");
        let alignment_path = workspace.path().join("alignment.json");
        let transcript_bytes = br#"{"words":[]}"#.to_vec();
        let alignment_bytes = br#"{"segments":[]}"#.to_vec();
        std::fs::write(&transcript_path, &transcript_bytes).expect("write transcript");
        std::fs::write(&alignment_path, &alignment_bytes).expect("write alignment");

        (
            AsrOutput {
                job_id: asset_id.to_string(),
                transcript_uri: format!("file://{}", transcript_path.display()),
                alignment_uri: format!("file://{}", alignment_path.display()),
                status: "ok".into(),
            },
            transcript_bytes,
            alignment_bytes,
        )
    }

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
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        let (asr_output, _t, _a) = stub_asr_output(asset_id.0, &asr_workspace);
        let client = StubAsrWorkerClient::ok(asr_output);
        let subtitle_queue = InMemorySubtitleJobQueue::default();

        process_transcription_job(
            &pool,
            &storage,
            &client,
            &subtitle_queue,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect("process transcription job");

        let status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(status.status, TranscriptionStatus::Ready);

        let ready = transcription_repo::get_transcription_readiness_evidence(&pool, asset_id)
            .await
            .expect("readiness");
        assert!(ready);
    }

    #[tokio::test]
    async fn process_transcription_job_enqueues_first_subtitle_target_after_ready() {
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
        let project_id = insert_project_with_targets(&pool, asset_id, "en", &["fr", "de"]).await;

        let (asr_output, _t, _a) = stub_asr_output(asset_id.0, &asr_workspace);
        let client = StubAsrWorkerClient::ok(asr_output);
        let subtitle_queue = InMemorySubtitleJobQueue::default();

        process_transcription_job(
            &pool,
            &storage,
            &client,
            &subtitle_queue,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect("process transcription job");

        let transcription_status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get status")
            .expect("status row");
        assert_eq!(transcription_status.status, TranscriptionStatus::Ready);

        let subtitle_status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get subtitle status")
            .expect("subtitle row");
        assert_eq!(subtitle_status.status, SubtitleStatus::Pending);

        let jobs = subtitle_queue.queued_jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].asset_id, asset_id.0);
        assert_eq!(jobs[0].project_id, project_id.0);
        assert_eq!(jobs[0].target_language, "de");
    }

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
        let subtitle_queue = InMemorySubtitleJobQueue::default();

        let err = process_transcription_job(
            &pool,
            &storage,
            &client,
            &subtitle_queue,
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
        assert!(subtitle_queue.queued_jobs().is_empty());
    }

    #[tokio::test]
    async fn process_transcription_job_preserves_ready_when_subtitle_queue_fails() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        struct FailingSubtitleQueue;
        impl SubtitleJobQueue for FailingSubtitleQueue {
            fn enqueue(&self, _job: dubbridge_jobs::SubtitleJob) -> Result<(), QueueError> {
                Err(QueueError::Unavailable("subtitle queue down".into()))
            }
        }

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
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        let (asr_output, _t, _a) = stub_asr_output(asset_id.0, &asr_workspace);
        let client = StubAsrWorkerClient::ok(asr_output);

        process_transcription_job(
            &pool,
            &storage,
            &client,
            &FailingSubtitleQueue,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect("transcription should remain successful");

        let transcription_status = transcription_repo::get_transcription_status(&pool, asset_id)
            .await
            .expect("get transcription status")
            .expect("transcription row");
        assert_eq!(transcription_status.status, TranscriptionStatus::Ready);

        let subtitle_status = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .expect("get subtitle status")
            .expect("subtitle row");
        assert_eq!(subtitle_status.status, SubtitleStatus::Failed);
        assert!(
            subtitle_status
                .error_detail
                .as_deref()
                .unwrap_or("")
                .contains("subtitle queue down")
        );
    }

    #[tokio::test]
    async fn process_transcription_envelope_rejects_wrong_job_type() {
        let Some(pool) = setup_pool().await else {
            return;
        };

        let asset_id = insert_asset(&pool).await;
        let source = insert_source_artifact(&pool, asset_id).await;
        let storage_workspace = TempDir::new().expect("storage workspace");
        let storage = LocalFsAdapter::new(storage_workspace.path());
        let subtitle_queue = InMemorySubtitleJobQueue::default();
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
            &subtitle_queue,
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

    #[tokio::test]
    async fn process_transcription_job_persists_transcript_lineage() {
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
        insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

        let (asr_output, _t, _a) = stub_asr_output(asset_id.0, &asr_workspace);
        let client = StubAsrWorkerClient::ok(asr_output);
        let subtitle_queue = InMemorySubtitleJobQueue::default();

        process_transcription_job(
            &pool,
            &storage,
            &client,
            &subtitle_queue,
            TranscriptionJob::new(asset_id.0, source.id, "en"),
        )
        .await
        .expect("process transcription job");

        let derived = dubbridge_db::preparation_repo::list_derived_artifacts(&pool, asset_id)
            .await
            .expect("list derived artifacts");
        let transcript = derived
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::TranscriptText)
            .expect("transcript artifact");
        let alignment = derived
            .iter()
            .find(|artifact| artifact.kind == ArtifactKind::WordAlignment)
            .expect("alignment artifact");
        assert_eq!(transcript.parent_artifact_id, source.id);
        assert_eq!(alignment.parent_artifact_id, source.id);
    }
}
