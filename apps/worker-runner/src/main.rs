#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
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
    tracing::info!(queue = %queue, "worker runner skeleton initialized");
    Ok(())
}
