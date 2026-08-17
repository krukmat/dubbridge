#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
    tracing::info!(
        database_url_configured = true,
        "dubbridge-cli: applying migrations"
    );

    let pool = dubbridge_db::create_pool(&config.database_url).await?;
    sqlx::migrate!("../../infra/migrations").run(&pool).await?;

    tracing::info!("dubbridge-cli: migrations applied successfully");

    Ok(())
}
