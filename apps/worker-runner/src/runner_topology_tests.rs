use std::{
    env, io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::anyhow;
use apalis::prelude::{
    Context as WorkerContext, Data, MemoryStorage, MessageQueue, Monitor, Worker, WorkerBuilder,
    WorkerBuilderExt, WorkerFactoryFn,
};
use async_trait::async_trait;
use dubbridge_db::{
    artifact_repo, preparation_repo, subtitle_repo, target_language_repo, transcription_repo,
    workspace_repo,
};
use dubbridge_domain::{
    artifact::{
        ArtifactKind, ArtifactRecord, PreparationStatus, SubtitleStatus, TranscriptionStatus,
    },
    asset::AssetId,
    workspace::{OrgId, Organization, Project, ProjectId, TargetLanguage},
};
use dubbridge_jobs::{
    PreparationJob, RedisPreparationJobQueue, RedisSubtitleJobQueue, RedisTranscriptionJobQueue,
    RedisTranslationJobQueue,
};
use dubbridge_providers::translation::{
    StubTranslationWorkerClient, TRANSLATION_SCHEMA_VERSION, TranslationOutput,
};
use dubbridge_providers::{AsrOutput, StubAsrWorkerClient};
use dubbridge_storage::{LocalFsAdapter, StorageAdapter};
use sqlx::PgPool;
use tempfile::TempDir;
use time::OffsetDateTime;
use tokio::{
    sync::{Mutex, Notify, oneshot},
    time::sleep,
};
use uuid::Uuid;

use crate::preparation_runtime::{HlsPackageOutput, HlsSegmentOutput, PreparationExecutor};
use crate::{
    SharedAsrWorkerClient, SharedPreparationExecutor, SharedStorage, SharedSubtitleQueue,
    SharedTranscriptionQueue, SharedTranslationQueue, SharedTranslationWorkerClient, WorkerRuntime,
    guard_worker_shutdown, resolve_asr_worker_path, run_monitor_with_signal,
    wait_for_shutdown_signal,
};

#[tokio::test]
async fn shutdown_waiter_resolves_for_ctrl_c_and_sigterm() {
    let ctrl_c = async { Ok(()) };
    let sigterm = std::future::pending::<io::Result<()>>();

    wait_for_shutdown_signal(ctrl_c, sigterm)
        .await
        .expect("ctrl-c path should resolve");

    let ctrl_c = std::future::pending::<io::Result<()>>();
    let sigterm = async { Ok(()) };

    wait_for_shutdown_signal(ctrl_c, sigterm)
        .await
        .expect("sigterm path should resolve");
}

#[test]
fn asr_worker_path_uses_override_when_present() {
    let repo = TempDir::new().expect("repo tempdir");
    let override_path = repo.path().join("custom/asr.py");
    std::fs::create_dir_all(override_path.parent().expect("override parent"))
        .expect("create override parent");
    std::fs::write(&override_path, "print('stub')").expect("write override worker");

    let resolved = resolve_asr_worker_path(
        std::path::Path::new("/repo/apps/worker-runner"),
        Some(override_path.clone()),
        None,
    )
    .expect("override path should be accepted");

    assert_eq!(resolved, override_path);
}

#[test]
fn asr_worker_path_defaults_to_repo_relative_script() {
    let repo = TempDir::new().expect("repo tempdir");
    let manifest_dir = repo.path().join("apps/worker-runner");
    let worker_path = repo.path().join("workers/asr-worker-py/main.py");
    std::fs::create_dir_all(worker_path.parent().expect("worker parent")).expect("create workers");
    std::fs::create_dir_all(&manifest_dir).expect("create manifest dir");
    std::fs::write(&worker_path, "print('stub')").expect("write worker");

    let resolved =
        resolve_asr_worker_path(&manifest_dir, None, None).expect("find worker from repo");

    assert_eq!(resolved, worker_path);
}

#[test]
fn asr_worker_path_can_be_found_from_current_exe_ancestors() {
    let repo = TempDir::new().expect("repo tempdir");
    let exe_path = repo.path().join("target/debug/dubbridge-worker-runner");
    let worker_path = repo.path().join("workers/asr-worker-py/main.py");
    std::fs::create_dir_all(exe_path.parent().expect("exe parent")).expect("create exe parent");
    std::fs::create_dir_all(worker_path.parent().expect("worker parent")).expect("create workers");
    std::fs::write(&worker_path, "print('stub')").expect("write worker");
    std::fs::write(&exe_path, "").expect("write exe placeholder");

    let resolved = resolve_asr_worker_path(
        std::path::Path::new("/manifest/fallback"),
        None,
        Some(exe_path),
    )
    .expect("find worker from current exe");

    assert_eq!(resolved, worker_path);
}

#[test]
fn asr_worker_path_fails_closed_when_script_is_missing() {
    let repo = TempDir::new().expect("repo tempdir");
    let manifest_dir = repo.path().join("apps/worker-runner");
    std::fs::create_dir_all(&manifest_dir).expect("create manifest dir");

    let error =
        resolve_asr_worker_path(&manifest_dir, None, None).expect_err("missing script should fail");

    assert!(error.to_string().contains("ASR worker script not found"));
}

#[tokio::test]
async fn worker_shutdown_does_not_start_new_jobs_after_signal() {
    let queue = MemoryStorage::new();
    let mut handle = queue.clone();
    handle.enqueue(1_u32).await.expect("enqueue first job");
    handle.enqueue(2_u32).await.expect("enqueue second job");

    #[derive(Clone)]
    struct ShutdownProbe {
        started: Arc<AtomicUsize>,
        completed: Arc<AtomicUsize>,
        release: Arc<Notify>,
        signal: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    }

    let started = Arc::new(AtomicUsize::new(0));
    let completed = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(Notify::new());
    let (signal_tx, signal_rx) = oneshot::channel();
    let probe = ShutdownProbe {
        started: started.clone(),
        completed: completed.clone(),
        release: release.clone(),
        signal: Arc::new(Mutex::new(Some(signal_tx))),
    };

    async fn blocking_job(
        _job: u32,
        probe: Data<ShutdownProbe>,
        worker: Worker<WorkerContext>,
    ) -> anyhow::Result<()> {
        guard_worker_shutdown(&worker, "shutdown-probe")?;
        probe.started.fetch_add(1, Ordering::SeqCst);
        if let Some(signal_tx) = probe.signal.lock().await.take() {
            signal_tx.send(()).expect("send shutdown signal");
        }
        probe.release.notified().await;
        probe.completed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    let monitor = Monitor::new().register(
        WorkerBuilder::new("shutdown-probe")
            .concurrency(1)
            .data(probe)
            .backend(queue)
            .build_fn(blocking_job),
    );

    let run = tokio::spawn(async move {
        run_monitor_with_signal(monitor, async move {
            signal_rx
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "signal sender dropped"))
        })
        .await
    });

    wait_until(
        Duration::from_secs(5),
        || started.load(Ordering::SeqCst) == 1,
        "the first job to start",
    )
    .await
    .expect("first job should start");
    assert_eq!(started.load(Ordering::SeqCst), 1);

    sleep(Duration::from_millis(150)).await;
    assert_eq!(
        started.load(Ordering::SeqCst),
        1,
        "shutdown should prevent a second job from starting"
    );

    release.notify_waiters();

    run.await
        .expect("join run task")
        .expect("monitor should stop cleanly");

    assert_eq!(started.load(Ordering::SeqCst), 1);
    assert_eq!(completed.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn worker_failure_does_not_stop_sibling_workers() {
    let success_a_backend = MemoryStorage::new();
    let success_b_backend = MemoryStorage::new();
    let failing_backend = MemoryStorage::new();

    success_a_backend
        .clone()
        .enqueue(1_u32)
        .await
        .expect("enqueue success A");
    success_b_backend
        .clone()
        .enqueue(2_u32)
        .await
        .expect("enqueue success B");
    failing_backend
        .clone()
        .enqueue(3_u32)
        .await
        .expect("enqueue failure");

    let success_a = Arc::new(AtomicUsize::new(0));
    let success_b = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));

    async fn ok_job(_job: u32, counter: Data<Arc<AtomicUsize>>) -> anyhow::Result<()> {
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn err_job(_job: u32, counter: Data<Arc<AtomicUsize>>) -> anyhow::Result<()> {
        counter.fetch_add(1, Ordering::SeqCst);
        Err(anyhow!("intentional failure"))
    }

    let monitor = Monitor::new()
        .register(
            WorkerBuilder::new("success-a")
                .concurrency(1)
                .data(success_a.clone())
                .backend(success_a_backend)
                .build_fn(ok_job),
        )
        .register(
            WorkerBuilder::new("failure")
                .concurrency(1)
                .data(failures.clone())
                .backend(failing_backend)
                .build_fn(err_job),
        )
        .register(
            WorkerBuilder::new("success-b")
                .concurrency(1)
                .data(success_b.clone())
                .backend(success_b_backend)
                .build_fn(ok_job),
        );

    run_monitor_with_signal(monitor, async {
        wait_until(
            Duration::from_secs(5),
            || {
                success_a.load(Ordering::SeqCst) == 1
                    && success_b.load(Ordering::SeqCst) == 1
                    && failures.load(Ordering::SeqCst) == 1
            },
            "workers should all observe their queued job",
        )
        .await?;
        Ok(())
    })
    .await
    .expect("monitor should stop cleanly");

    assert_eq!(success_a.load(Ordering::SeqCst), 1);
    assert_eq!(success_b.load(Ordering::SeqCst), 1);
    assert_eq!(failures.load(Ordering::SeqCst), 1);
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn redis_monitor_wires_preparation_transcription_and_subtitle_workers() {
    let Some(database_url) = env::var("DUBBRIDGE_DATABASE_URL").ok() else {
        return;
    };
    let Some(redis_url) = env::var("DUBBRIDGE_REDIS_URL").ok() else {
        return;
    };

    let pool = setup_pool(&database_url).await;
    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;
    let project_id = insert_project_with_targets(&pool, asset_id, "en", &["es"]).await;

    let storage_workspace = TempDir::new().expect("storage workspace");
    let storage = LocalFsAdapter::new(storage_workspace.path());
    storage
        .put(&source.storage_key, b"audio-bytes".to_vec())
        .await
        .expect("store source audio");

    preparation_repo::upsert_preparation_status(&pool, asset_id, PreparationStatus::Pending, None)
        .await
        .expect("set preparation pending");

    let shared_storage: SharedStorage = Arc::new(storage);

    let asr_workspace = TempDir::new().expect("asr workspace");
    let asr_output = write_stub_asr_output(asset_id.0, &asr_workspace);

    let preparation_backend = Arc::new(
        RedisPreparationJobQueue::connect(&redis_url)
            .await
            .expect("connect preparation backend"),
    );
    let transcription_backend = Arc::new(
        RedisTranscriptionJobQueue::connect(&redis_url)
            .await
            .expect("connect transcription backend"),
    );
    let subtitle_backend = Arc::new(
        RedisSubtitleJobQueue::connect(&redis_url)
            .await
            .expect("connect subtitle backend"),
    );
    let translation_backend = Arc::new(
        RedisTranslationJobQueue::connect(&redis_url)
            .await
            .expect("connect translation backend"),
    );

    let runtime = WorkerRuntime {
        worker_concurrency: 2,
        pool: pool.clone(),
        storage: shared_storage,
        preparation_executor: Arc::new(FakePreparationExecutor) as SharedPreparationExecutor,
        asr_client: Arc::new(StubAsrWorkerClient::ok(asr_output)) as SharedAsrWorkerClient,
        translation_client: Arc::new(StubTranslationWorkerClient::ok(TranslationOutput {
            schema_version: TRANSLATION_SCHEMA_VERSION,
            job_id: "unused-in-this-topology-test".into(),
            source_language: "en".into(),
            target_language: "es".into(),
            segments: vec![],
            status: "ok".into(),
        })) as SharedTranslationWorkerClient,
        preparation_backend: preparation_backend.clone(),
        transcription_backend: transcription_backend.clone(),
        subtitle_backend: subtitle_backend.clone(),
        translation_backend: translation_backend.clone(),
        transcription_enqueue: transcription_backend.clone() as SharedTranscriptionQueue,
        subtitle_enqueue: subtitle_backend.clone() as SharedSubtitleQueue,
        translation_enqueue: translation_backend.clone() as SharedTranslationQueue,
    };

    preparation_backend
        .enqueue_with_id(PreparationJob::new(
            asset_id.0,
            source.id,
            source.ingest_token,
        ))
        .await
        .expect("enqueue preparation job");

    runtime
        .run_with_signal(wait_for_end_to_end_ready(pool.clone(), asset_id))
        .await
        .expect("monitor should process all three queues");

    let preparation_status = preparation_repo::get_preparation_status(&pool, asset_id)
        .await
        .expect("get preparation status")
        .expect("preparation status row");
    assert_eq!(preparation_status.status, PreparationStatus::Ready);

    let transcription_status = transcription_repo::get_transcription_status(&pool, asset_id)
        .await
        .expect("get transcription status")
        .expect("transcription status row");
    assert_eq!(transcription_status.status, TranscriptionStatus::Ready);

    let subtitle_status = subtitle_repo::get_subtitle_status(&pool, asset_id)
        .await
        .expect("get subtitle status")
        .expect("subtitle status row");
    assert_eq!(subtitle_status.status, SubtitleStatus::Ready);

    // HP-1: localization fan-out replaces the legacy review-enqueue call --
    // no review_tasks row is created by the subtitle worker's post-ready dispatch.
    let review_task_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM review_tasks WHERE project_id = $1")
            .bind(project_id.0)
            .fetch_one(&pool)
            .await
            .expect("count review tasks");
    assert_eq!(
        review_task_count, 0,
        "localization fan-out must not create a legacy review task"
    );

    // HP-1: the one configured target language ("es") was acknowledged as
    // dispatched -- durable evidence the translation job was actually
    // enqueued onto RedisTranslationJobQueue and its dispatch persisted,
    // not merely attempted.
    let acknowledged_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM translation_dispatch_outbox \
         WHERE project_id = $1 AND asset_id = $2 AND delivery_state = 'acknowledged'",
    )
    .bind(project_id.0)
    .bind(asset_id.0)
    .fetch_one(&pool)
    .await
    .expect("count acknowledged translation dispatches");
    assert_eq!(
        acknowledged_count, 1,
        "the one configured target language's translation dispatch should be acknowledged"
    );

    let derived = preparation_repo::list_derived_artifacts(&pool, asset_id)
        .await
        .expect("list derived artifacts");
    assert!(
        derived
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::ProbeMetadata),
        "preparation worker should persist probe metadata"
    );
    assert!(
        derived
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::TranscriptText),
        "transcription worker should persist transcript text"
    );
    assert!(
        derived
            .iter()
            .any(|artifact| artifact.kind == ArtifactKind::Subtitle),
        "subtitle worker should persist subtitle artifact"
    );
}

async fn wait_until(
    timeout: Duration,
    predicate: impl Fn() -> bool,
    label: &str,
) -> io::Result<()> {
    let start = Instant::now();
    while !predicate() {
        if start.elapsed() > timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("timed out while waiting for {label}"),
            ));
        }
        sleep(Duration::from_millis(25)).await;
    }
    Ok(())
}

async fn wait_for_end_to_end_ready(pool: PgPool, asset_id: AssetId) -> io::Result<()> {
    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(20) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "timed out waiting for subtitle-ready end-to-end pipeline",
            ));
        }

        let subtitle_ready = subtitle_repo::get_subtitle_status(&pool, asset_id)
            .await
            .map_err(|error| io::Error::other(error.to_string()))?
            .is_some_and(|status| status.status == SubtitleStatus::Ready);
        let review_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM review_tasks")
            .fetch_one(&pool)
            .await
            .map_err(|error| io::Error::other(error.to_string()))?;

        if subtitle_ready && review_count == 1 {
            return Ok(());
        }

        sleep(Duration::from_millis(50)).await;
    }
}

async fn setup_pool(database_url: &str) -> PgPool {
    let pool = PgPool::connect(database_url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    sqlx::query(
        "TRUNCATE TABLE review_tasks, target_languages, project_assets, projects, org_members, organizations, pending_ingestions, audit_events, artifact_records, rights_records, assets, asset_preparation_status, asset_transcription_status, asset_subtitle_status RESTART IDENTITY CASCADE",
    )
    .execute(&pool)
    .await
    .expect("truncate");
    pool
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
        target_language_repo::upsert_target_language(
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

fn write_stub_asr_output(asset_id: Uuid, workspace: &TempDir) -> AsrOutput {
    let transcript_path = workspace.path().join("transcript.json");
    let alignment_path = workspace.path().join("alignment.json");
    std::fs::write(&transcript_path, br#"{"text":"hello"}"#).expect("write transcript");
    std::fs::write(
        &alignment_path,
        br#"{"words":[{"word":"hello","start":0.0,"end":0.5},{"word":"world","start":0.6,"end":1.0}]}"#,
    )
    .expect("write alignment");

    AsrOutput {
        job_id: asset_id.to_string(),
        transcript_uri: format!("file://{}", transcript_path.display()),
        alignment_uri: format!("file://{}", alignment_path.display()),
        status: "ok".into(),
    }
}

#[derive(Default)]
struct FakePreparationExecutor;

#[async_trait]
impl PreparationExecutor for FakePreparationExecutor {
    async fn extract_probe_metadata(&self, _source_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
        Ok(
            br#"{"format":{"duration":"1.0"},"streams":[{"codec_type":"audio","codec_name":"aac"}]}"#
                .to_vec(),
        )
    }

    async fn transcode_hls(&self, _source_bytes: &[u8]) -> anyhow::Result<HlsPackageOutput> {
        Ok(HlsPackageOutput {
            manifest_bytes: b"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:1
#EXT-X-PLAYLIST-TYPE:VOD
#EXTINF:1.0,
segment_00000.ts
#EXT-X-ENDLIST
"
            .to_vec(),
            segments: vec![HlsSegmentOutput {
                file_name: "segment_00000.ts".into(),
                bytes: b"segment".to_vec(),
            }],
        })
    }
}
