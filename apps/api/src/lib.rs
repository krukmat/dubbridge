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
use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use sqlx::PgPool;

use crate::state::AppState;

const READINESS_PROBE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
struct ComponentCheck {
    component: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
struct ReadinessResponse {
    service: &'static str,
    status: &'static str,
    checks: Vec<ComponentCheck>,
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

async fn probe_postgres(pool: &PgPool) -> bool {
    tokio::time::timeout(
        READINESS_PROBE_TIMEOUT,
        sqlx::query("SELECT 1").execute(pool),
    )
    .await
    .map(|result| result.is_ok())
    .unwrap_or(false)
}

async fn probe_redis(redis_url: &str) -> bool {
    let Ok(client) = redis::Client::open(redis_url) else {
        return false;
    };
    let connect = redis::aio::ConnectionManager::new(client);
    let Ok(Ok(mut conn)) = tokio::time::timeout(READINESS_PROBE_TIMEOUT, connect).await else {
        return false;
    };
    tokio::time::timeout(
        READINESS_PROBE_TIMEOUT,
        redis::cmd("PING").query_async::<()>(&mut conn),
    )
    .await
    .map(|result| result.is_ok())
    .unwrap_or(false)
}

async fn probe_storage(storage: &(dyn dubbridge_storage::StorageAdapter + Send + Sync)) -> bool {
    tokio::time::timeout(READINESS_PROBE_TIMEOUT, storage.list_keys(""))
        .await
        .map(|result| result.is_ok())
        .unwrap_or(false)
}

fn build_readiness_response(
    postgres_ok: bool,
    redis_ok: bool,
    storage_ok: bool,
) -> (StatusCode, Json<ReadinessResponse>) {
    let checks = vec![
        ComponentCheck {
            component: "postgres",
            status: if postgres_ok { "ok" } else { "unreachable" },
        },
        ComponentCheck {
            component: "redis",
            status: if redis_ok { "ok" } else { "unreachable" },
        },
        ComponentCheck {
            component: "storage",
            status: if storage_ok { "ok" } else { "unreachable" },
        },
    ];

    let all_ok = postgres_ok && redis_ok && storage_ok;
    let status_code = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            service: "api",
            status: if all_ok { "ready" } else { "degraded" },
            checks,
        }),
    )
}

async fn ready(State(state): State<Arc<AppState>>) -> (StatusCode, Json<ReadinessResponse>) {
    let (postgres_ok, redis_ok, storage_ok) = tokio::join!(
        probe_postgres(&state.pool),
        probe_redis(&state.config.redis_url),
        probe_storage(state.storage.as_ref()),
    );

    build_readiness_response(postgres_ok, redis_ok, storage_ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[tokio::test]
    async fn probe_postgres_returns_false_on_unreachable_connection() {
        let pool = PgPool::connect_lazy("postgres://user:pass@127.0.0.1:1/nonexistent")
            .expect("lazy pool");
        assert!(!probe_postgres(&pool).await);
    }

    #[tokio::test]
    async fn probe_redis_returns_false_on_unreachable_connection() {
        assert!(!probe_redis("redis://127.0.0.1:1").await);
    }

    #[tokio::test]
    async fn probe_postgres_bounded_by_timeout_on_hung_connection() {
        // A listener that accepts the TCP connection but never responds,
        // simulating a hung dependency. The probe must return within a
        // bounded time instead of hanging forever.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind hung listener");
        let addr = listener.local_addr().expect("local addr");
        std::thread::spawn(move || {
            // Accept and hold the connection open without responding.
            let _ = listener.accept();
            std::thread::sleep(Duration::from_secs(30));
        });

        let pool = PgPool::connect_lazy(&format!(
            "postgres://user:pass@{}:{}/nonexistent",
            addr.ip(),
            addr.port()
        ))
        .expect("lazy pool");

        let result = tokio::time::timeout(Duration::from_secs(5), probe_postgres(&pool)).await;
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn readiness_all_ok_returns_200_and_ready_status() {
        let (status_code, json_response) = build_readiness_response(true, true, true);
        assert_eq!(status_code, StatusCode::OK);
        let response = json_response.0;
        assert_eq!(response.status, "ready");
        assert_eq!(response.checks.len(), 3);
        assert_eq!(response.checks[0].component, "postgres");
        assert_eq!(response.checks[0].status, "ok");
        assert_eq!(response.checks[1].component, "redis");
        assert_eq!(response.checks[1].status, "ok");
        assert_eq!(response.checks[2].component, "storage");
        assert_eq!(response.checks[2].status, "ok");
    }

    #[test]
    fn readiness_postgres_unreachable_returns_503_and_marks_only_postgres() {
        let (status_code, json_response) = build_readiness_response(false, true, true);
        assert_eq!(status_code, StatusCode::SERVICE_UNAVAILABLE);
        let response = json_response.0;
        assert_eq!(response.status, "degraded");
        assert_eq!(response.checks.len(), 3);
        assert_eq!(response.checks[0].component, "postgres");
        assert_eq!(response.checks[0].status, "unreachable");
        assert_eq!(response.checks[1].component, "redis");
        assert_eq!(response.checks[1].status, "ok");
        assert_eq!(response.checks[2].component, "storage");
        assert_eq!(response.checks[2].status, "ok");
    }
}
