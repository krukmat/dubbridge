#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
    tracing::info!(
        env = ?config.env,
        log_format = ?config.observability.log_format,
        redis_url = %config.redis_url,
        worker_concurrency = config.worker_concurrency,
        "starting worker runner"
    );

    let queue = dubbridge_jobs::default_queue();
    tracing::info!(queue = %queue, "worker runner skeleton initialized");
    Ok(())
}
