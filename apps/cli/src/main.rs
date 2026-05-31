#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dubbridge_observability::init_tracing();

    let config = dubbridge_config::AppConfig::from_env();
    println!(
        "dubbridge cli skeleton ready on api port {} with storage bucket {}",
        config.api_port, config.storage_bucket
    );

    Ok(())
}
