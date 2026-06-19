#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::from_env();
    dubbridge_observability::init_tracing(&config.observability);
    tracing::info!(
        api_port = config.api_port,
        storage_bucket = %config.storage.bucket,
        "dubbridge cli skeleton ready"
    );

    Ok(())
}
