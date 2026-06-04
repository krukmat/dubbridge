use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use dubbridge_gateway::{
    auth::pending::PendingAuthStore, build_app, session::store::RedisSessionStore,
    state::GatewayState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
    let gateway = config
        .gateway_settings()
        .context("failed to resolve gateway settings")?
        .clone();
    // P1-T3: wire Redis session store; T5 will add transparent token refresh
    let session_store = Arc::new(
        RedisSessionStore::new(&config.redis_url)
            .await
            .context("failed to connect to Redis session store")?,
    );
    // P1-T4: in-process pending store for OAuth state/verifier pairs (TTL 10 min)
    let pending_store = Arc::new(PendingAuthStore::with_default_ttl());

    let state = Arc::new(GatewayState::new(
        reqwest::Client::new(),
        config,
        gateway.clone(),
        session_store,
        pending_store,
    ));
    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], gateway.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(
        port = gateway.port,
        upstream_api_base_url = %gateway.upstream_api_base_url,
        oauth_authorization_url = %gateway.oauth.authorization_url,
        oauth_token_url = %gateway.oauth.token_url,
        "starting gateway"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
