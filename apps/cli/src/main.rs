#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::from_env();
    dubbridge_observability::init_tracing(&config.observability);
    println!(
        "dubbridge cli skeleton ready on api port {} with storage bucket {}",
        config.api_port, config.storage.bucket
    );

    Ok(())
}
