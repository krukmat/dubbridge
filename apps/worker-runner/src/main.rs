#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dubbridge_observability::init_tracing();

    let config = dubbridge_config::AppConfig::from_env();
    tracing::info!(
        redis_url = %config.redis_url,
        worker_concurrency = config.worker_concurrency,
        "starting worker runner"
    );

    let queue = dubbridge_jobs::default_queue();
    tracing::info!(queue = %queue, "worker runner skeleton initialized");
    Ok(())
}
