use std::{
    env,
    future::Future,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use apalis::prelude::{
    Context as WorkerContext, Data, Monitor, Worker, WorkerBuilder, WorkerBuilderExt,
    WorkerFactoryFn,
};
use dubbridge_db::create_pool;
use dubbridge_jobs::{
    RedisPreparationJobQueue, RedisSubtitleJobQueue, RedisTranscriptionJobQueue,
    RedisTranslationJobQueue, SubtitleJobQueue, TranscriptionJobQueue, TranslationJobQueue,
};
use dubbridge_providers::{AsrWorkerClient, SubprocessAsrWorkerClient};
use dubbridge_storage::StorageAdapter;
use preparation_media_executor::SubprocessPreparationExecutor;
use preparation_runtime::PreparationExecutor;
use sha2::{Digest, Sha256};

mod preparation_artifact_persistence;
mod preparation_media_executor;
mod preparation_runtime;
#[cfg(test)]
mod preparation_runtime_tests;
// Superseded by translation_fanout::fan_out_localization (S-150-T2c-vi-a);
// no caller remains. Left in place, unused, for S-150-T2c-vi-b to delete
// together with its BDD/doc sync -- out of this task's scope.
#[allow(dead_code)]
mod review_enqueue;
#[cfg(test)]
mod runner_topology_tests;
mod subtitle_alignment;
mod subtitle_enqueue;
mod subtitle_runtime;
#[cfg(test)]
mod subtitle_runtime_tests;
mod transcription_runtime;
mod translation_fanout;
#[cfg(test)]
mod translation_fanout_tests;

const ASR_WORKER_RELATIVE_PATH: &str = "workers/asr-worker-py/main.py";
const WORKER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

type SharedStorage = Arc<dyn StorageAdapter>;
type SharedPreparationExecutor = Arc<dyn PreparationExecutor>;
type SharedAsrWorkerClient = Arc<dyn AsrWorkerClient>;
type SharedTranscriptionQueue = Arc<dyn TranscriptionJobQueue>;
type SharedSubtitleQueue = Arc<dyn SubtitleJobQueue>;
type SharedTranslationQueue = Arc<dyn TranslationJobQueue>;

struct WorkerRuntime {
    worker_concurrency: usize,
    pool: sqlx::PgPool,
    storage: SharedStorage,
    preparation_executor: SharedPreparationExecutor,
    asr_client: SharedAsrWorkerClient,
    preparation_backend: Arc<RedisPreparationJobQueue>,
    transcription_backend: Arc<RedisTranscriptionJobQueue>,
    subtitle_backend: Arc<RedisSubtitleJobQueue>,
    // Not yet consumed by a worker (no translation runtime exists until
    // S-150-T3c); kept for connect-time parity with the other three
    // backends and to let a future translation worker attach via .backend().
    #[allow(dead_code)]
    translation_backend: Arc<RedisTranslationJobQueue>,
    transcription_enqueue: SharedTranscriptionQueue,
    subtitle_enqueue: SharedSubtitleQueue,
    translation_enqueue: SharedTranslationQueue,
}

impl WorkerRuntime {
    async fn connect(config: &dubbridge_config::AppConfig) -> anyhow::Result<Self> {
        let pool = create_pool(&config.database_url)
            .await
            .context("failed to create database pool for worker runner")?;
        let storage_config = dubbridge_storage::StorageConfig::from(&config.storage);
        let storage = dubbridge_storage::build_adapter(&storage_config)
            .map_err(|e| anyhow::anyhow!("failed to initialize configured storage backend: {e}"))?;

        let preparation_backend = Arc::new(
            RedisPreparationJobQueue::connect(&config.redis_url)
                .await
                .context("failed to connect preparation Redis queue")?,
        );
        let transcription_backend = Arc::new(
            RedisTranscriptionJobQueue::connect(&config.redis_url)
                .await
                .context("failed to connect transcription Redis queue")?,
        );
        let subtitle_backend = Arc::new(
            RedisSubtitleJobQueue::connect(&config.redis_url)
                .await
                .context("failed to connect subtitle Redis queue")?,
        );
        let translation_backend = Arc::new(
            RedisTranslationJobQueue::connect(&config.redis_url)
                .await
                .context("failed to connect translation Redis queue")?,
        );

        Ok(Self {
            worker_concurrency: config.worker_concurrency,
            pool,
            storage: Arc::from(storage),
            preparation_executor: Arc::new(SubprocessPreparationExecutor),
            asr_client: Arc::new(SubprocessAsrWorkerClient::new(asr_worker_command()?)),
            preparation_backend,
            transcription_enqueue: transcription_backend.clone(),
            transcription_backend,
            subtitle_enqueue: subtitle_backend.clone(),
            subtitle_backend,
            translation_enqueue: translation_backend.clone(),
            translation_backend,
        })
    }

    fn build_monitor(&self) -> Monitor {
        let monitor = Monitor::new();
        let monitor = self.register_preparation_worker(monitor);
        let monitor = self.register_transcription_worker(monitor);
        self.register_subtitle_worker(monitor)
    }

    async fn run_with_signal<S>(&self, signal: S) -> io::Result<()>
    where
        S: Send + Future<Output = io::Result<()>>,
    {
        run_monitor_with_signal(self.build_monitor(), signal).await
    }

    fn register_preparation_worker(&self, monitor: Monitor) -> Monitor {
        monitor.register(
            WorkerBuilder::new("worker-runner-preparation")
                .concurrency(self.worker_concurrency)
                .data(self.pool.clone())
                .data(self.storage.clone())
                .data(self.preparation_executor.clone())
                .data(self.transcription_enqueue.clone())
                .backend(self.preparation_backend.backend())
                .build_fn(run_preparation_job),
        )
    }

    fn register_transcription_worker(&self, monitor: Monitor) -> Monitor {
        monitor.register(
            WorkerBuilder::new("worker-runner-transcription")
                .concurrency(self.worker_concurrency)
                .data(self.pool.clone())
                .data(self.storage.clone())
                .data(self.asr_client.clone())
                .data(self.subtitle_enqueue.clone())
                .backend(self.transcription_backend.backend())
                .build_fn(run_transcription_job),
        )
    }

    fn register_subtitle_worker(&self, monitor: Monitor) -> Monitor {
        monitor.register(
            WorkerBuilder::new("worker-runner-subtitle")
                .concurrency(self.worker_concurrency)
                .data(self.pool.clone())
                .data(self.storage.clone())
                .data(self.translation_enqueue.clone())
                .backend(self.subtitle_backend.backend())
                .build_fn(run_subtitle_job),
        )
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
    let runtime = WorkerRuntime::connect(&config).await?;
    let storage_reference = runtime.storage.object_url("__startup_probe__");

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

    runtime
        .run_with_signal(shutdown_signal())
        .await
        .context("worker runner monitor exited with io error")?;

    tracing::info!("worker runner stopped");
    Ok(())
}

fn asr_worker_command() -> anyhow::Result<Vec<String>> {
    let python = resolve_asr_worker_python()?;
    let worker_path = resolve_asr_worker_path(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        env::var_os("DUBBRIDGE_ASR_WORKER_PATH").map(PathBuf::from),
        env::current_exe().ok(),
    )?;
    Ok(vec![python, worker_path.display().to_string()])
}

fn resolve_asr_worker_path(
    manifest_dir: &Path,
    override_path: Option<PathBuf>,
    current_exe: Option<PathBuf>,
) -> anyhow::Result<PathBuf> {
    if let Some(path) = override_path {
        return validate_asr_worker_path(path, "DUBBRIDGE_ASR_WORKER_PATH");
    }

    let candidate = {
        find_asr_worker_path(current_exe.as_deref())
            .or_else(|| find_asr_worker_path(Some(manifest_dir)))
            .unwrap_or_else(|| manifest_dir.join("../../").join(ASR_WORKER_RELATIVE_PATH))
    };

    validate_asr_worker_path(candidate, "discovered fallback path")
}

fn find_asr_worker_path(start: Option<&Path>) -> Option<PathBuf> {
    start.and_then(|path| {
        path.ancestors().find_map(|ancestor| {
            let candidate = ancestor.join(ASR_WORKER_RELATIVE_PATH);
            candidate.is_file().then_some(candidate)
        })
    })
}

fn resolve_asr_worker_python() -> anyhow::Result<String> {
    let python = env::var("DUBBRIDGE_ASR_WORKER_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let python = python.trim();
    if python.is_empty() {
        anyhow::bail!("DUBBRIDGE_ASR_WORKER_PYTHON must not be empty");
    }
    Ok(python.to_string())
}

fn validate_asr_worker_path(path: PathBuf, source: &str) -> anyhow::Result<PathBuf> {
    if path.is_file() {
        return Ok(path);
    }

    anyhow::bail!(
        "ASR worker script not found via {source}: {}",
        path.display()
    );
}

async fn run_monitor_with_signal<S>(monitor: Monitor, signal: S) -> io::Result<()>
where
    S: Send + Future<Output = io::Result<()>>,
{
    monitor
        .shutdown_timeout(WORKER_SHUTDOWN_TIMEOUT)
        .run_with_signal(signal)
        .await
}

#[cfg(unix)]
async fn shutdown_signal() -> io::Result<()> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = signal(SignalKind::terminate()).map_err(|error| {
        io::Error::other(format!("failed to register SIGTERM handler: {error}"))
    })?;
    wait_for_shutdown_signal(tokio::signal::ctrl_c(), async move {
        terminate.recv().await;
        Ok(())
    })
    .await
}

#[cfg(not(unix))]
async fn shutdown_signal() -> io::Result<()> {
    tokio::signal::ctrl_c().await
}

async fn wait_for_shutdown_signal<C, T>(ctrl_c: C, sigterm: T) -> io::Result<()>
where
    C: Future<Output = io::Result<()>>,
    T: Future<Output = io::Result<()>>,
{
    tokio::select! {
        outcome = ctrl_c => outcome,
        outcome = sigterm => outcome,
    }
}

async fn run_preparation_job(
    job: dubbridge_jobs::PreparationJob,
    pool: Data<sqlx::PgPool>,
    storage: Data<SharedStorage>,
    executor: Data<SharedPreparationExecutor>,
    transcription_queue: Data<SharedTranscriptionQueue>,
    worker: Worker<WorkerContext>,
) -> anyhow::Result<()> {
    guard_worker_shutdown(&worker, "preparation")?;
    preparation_runtime::process_preparation_job(
        &pool,
        storage.as_ref(),
        &**executor,
        &**transcription_queue,
        job,
    )
    .await
}

async fn run_transcription_job(
    job: dubbridge_jobs::TranscriptionJob,
    pool: Data<sqlx::PgPool>,
    storage: Data<SharedStorage>,
    asr_client: Data<SharedAsrWorkerClient>,
    subtitle_queue: Data<SharedSubtitleQueue>,
    worker: Worker<WorkerContext>,
) -> anyhow::Result<()> {
    guard_worker_shutdown(&worker, "transcription")?;
    transcription_runtime::process_transcription_job(
        &pool,
        storage.as_ref(),
        &**asr_client,
        &**subtitle_queue,
        job,
    )
    .await
}

async fn run_subtitle_job(
    job: dubbridge_jobs::SubtitleJob,
    pool: Data<sqlx::PgPool>,
    storage: Data<SharedStorage>,
    translation_queue: Data<SharedTranslationQueue>,
    worker: Worker<WorkerContext>,
) -> anyhow::Result<()> {
    guard_worker_shutdown(&worker, "subtitle")?;
    subtitle_runtime::process_subtitle_job(&pool, storage.as_ref(), &**translation_queue, job).await
}

fn guard_worker_shutdown(worker: &Worker<WorkerContext>, queue_name: &str) -> anyhow::Result<()> {
    if worker.is_shutting_down() {
        anyhow::bail!("worker shutdown in progress; refusing new {queue_name} job");
    }
    Ok(())
}

pub(crate) fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
