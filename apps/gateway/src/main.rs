use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use dubbridge_gateway::{build_app, state::GatewayState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = dubbridge_config::AppConfig::load()?;
    dubbridge_observability::init_tracing(&config.observability);
    let gateway = config
        .gateway_settings()
        .context("failed to resolve gateway settings")?
        .clone();

    let state = Arc::new(GatewayState::new(
        reqwest::Client::new(),
        config,
        gateway.clone(),
    ));
    let app = build_app(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], gateway.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!(
        port = gateway.port,
        upstream_api_base_url = %gateway.upstream_api_base_url,
        "starting gateway"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
