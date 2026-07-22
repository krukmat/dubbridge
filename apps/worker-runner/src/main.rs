use anyhow::Context;
use dubbridge_db::create_pool;
use preparation_media_executor::SubprocessPreparationExecutor;
use sha2::{Digest, Sha256};

mod preparation_artifact_persistence;
mod preparation_media_executor;
mod preparation_runtime;
#[cfg(test)]
mod preparation_runtime_tests;
mod subtitle_enqueue;
mod subtitle_runtime;
mod transcription_runtime;

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

pub(crate) fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
