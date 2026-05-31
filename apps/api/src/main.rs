use std::{net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use dubbridge_api::{build_app, cleanup::cleanup_expired_ingestions, state::AppState};
use dubbridge_auth::{AuthConfig, RsaJwtTokenVerifier, SharedTokenVerifier};

// T1-T2: cleanup interval for expired pending ingestions.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dubbridge_observability::init_tracing();

    let config = dubbridge_config::AppConfig::from_env();
    let verifier = build_verifier(&config)?;
    let pool = dubbridge_db::create_pool(&config.database_url)
        .await
        .context("failed to create database pool")?;
    let storage_config = dubbridge_storage::StorageConfig::from_env(&config.storage_bucket);
    let storage = dubbridge_storage::build_adapter(&storage_config);
    let app_state = Arc::new(AppState::new(
        pool,
        storage,
        verifier.clone(),
        config.clone(),
    ));
    let api_port = app_state.config.api_port;
    let storage_bucket = app_state.config.storage_bucket.clone();
    let auth_verifier = app_state.verifier.clone();

    // T1-T2: spawn background task that periodically removes expired pending sessions
    // and their stored blobs. The task is detached; it runs until the process exits.
    let cleanup_pool = app_state.pool.clone();
    let cleanup_storage = dubbridge_storage::build_adapter(&storage_config);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            cleanup_expired_ingestions(&cleanup_pool, cleanup_storage.as_ref()).await;
        }
    });

    let app = build_app(app_state, auth_verifier);

    let addr = SocketAddr::from(([0, 0, 0, 0], api_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(
        port = api_port,
        storage_bucket = %storage_bucket,
        "starting api"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_verifier(config: &dubbridge_config::AppConfig) -> anyhow::Result<SharedTokenVerifier> {
    let auth = config
        .auth
        .as_ref()
        .ok_or_else(|| anyhow!("auth settings are required to start the API"))?;
    let auth_config = AuthConfig::new(
        auth.issuer.clone(),
        auth.audience.clone(),
        auth.rsa_public_key_path.clone(),
        Duration::from_secs(auth.clock_skew_leeway_seconds),
    );
    let verifier = RsaJwtTokenVerifier::new(auth_config)
        .context("failed to initialize JWT verifier from configured public key")?;

    Ok(Arc::new(verifier))
}
