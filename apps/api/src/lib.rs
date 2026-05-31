pub mod cleanup; // T1-T2
pub mod dto;
pub mod ingestion_service; // S3-T0: transport-agnostic finalization core
pub mod routes;
pub mod state;

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
        .merge(routes::ingestion::router(verifier))
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
