pub mod cleanup; // T1-T2
pub mod consent_gate; // S-110-T2a
pub mod dto;
pub mod ingestion_service; // S3-T0: transport-agnostic finalization core
pub mod middleware;
pub mod playback_api_error;
pub mod playback_audit;
pub mod playback_policy;
pub mod playback_service; // S-125-T4a-i: playback-grant issuance skeleton
pub mod review_gate; // S-160-T2a
pub mod routes;
pub mod state;
pub mod workspace_service;

use std::sync::Arc;

use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
}

pub fn build_app(state: Arc<AppState>, verifier: dubbridge_auth::SharedTokenVerifier) -> Router {
    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .merge(routes::auth::router())
        .merge(routes::compliance::router(verifier.clone()))
        .merge(routes::ingestion::router(verifier.clone()))
        .merge(routes::notifications::router(verifier.clone()))
        .merge(routes::playback::router(state.clone(), verifier.clone()))
        .merge(routes::review::router(state.pool.clone(), verifier.clone()))
        .merge(routes::workspace::router(state.pool.clone(), verifier))
        .with_state(state)
}

async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api",
        status: "live",
    })
}

async fn ready() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api",
        status: "ready",
    })
}
