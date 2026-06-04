pub mod auth; // P1-T2: OAuth client (PKCE, token exchange/refresh) + P1-T4: routes
pub mod cookie; // P1-T3: hardened session cookie + CSRF helpers
pub mod cookie_ext; // P1-T5.1: shared cookie extraction + session resolver
pub mod proxy; // P1-T5.2: token refresh logic + P1-T5.3: HTTP proxy handler
pub mod session; // P1-T3: server-side session store (ADR-024)
pub mod state;

use std::sync::Arc;

use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::{auth::auth_router, proxy::proxy_router, state::GatewayState};

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
}

pub fn build_app(state: Arc<GatewayState>) -> Router {
    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        // P1-T4: mount auth routes under /auth (state propagated from parent)
        .nest("/auth", auth_router())
        // P1-T5.3: authenticated API proxy under /api
        .nest("/api", proxy_router())
        .with_state(state)
}

async fn live() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "gateway",
        status: "live",
    })
}

async fn ready() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "gateway",
        status: "ready",
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{build_app, session::store::InMemorySessionStore, state::GatewayState};

    #[tokio::test]
    async fn health_endpoints_are_public() {
        let store = Arc::new(InMemorySessionStore::new());
        let pending = Arc::new(crate::auth::pending::PendingAuthStore::with_default_ttl());
        let state = Arc::new(GatewayState::new(
            reqwest::Client::new(),
            sample_config(),
            sample_gateway_settings(),
            store,
            pending,
        ));
        let app = build_app(state);

        let live = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(live.status(), StatusCode::OK);

        let ready = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ready.status(), StatusCode::OK);
    }

    fn sample_config() -> dubbridge_config::AppConfig {
        dubbridge_config::AppConfig {
            env: dubbridge_config::AppEnv::Local,
            api_port: 8080,
            database_url: "postgres://dubbridge:dubbridge@localhost:5432/dubbridge".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            worker_concurrency: 4,
            storage: dubbridge_config::StorageSettings {
                backend: dubbridge_config::StorageBackend::LocalFs,
                base_path: "/tmp/dubbridge-storage".to_string(),
                bucket: "dubbridge-local".to_string(),
                endpoint_url: None,
            },
            observability: dubbridge_config::ObsSettings {
                log_format: dubbridge_config::LogFormat::Pretty,
                filter: "info".to_string(),
            },
            auth: None,
            gateway: Some(sample_gateway_settings()),
        }
    }

    fn sample_gateway_settings() -> dubbridge_config::GatewaySettings {
        dubbridge_config::GatewaySettings {
            port: 8081,
            upstream_api_base_url: "http://localhost:8080".to_string(),
            oauth: dubbridge_config::GatewayOAuthSettings {
                authorization_url: "http://localhost:9000/oauth/authorize".to_string(),
                token_url: "http://localhost:9000/oauth/token".to_string(),
                client_id: "dubbridge-web-local".to_string(),
                client_secret: Some("secret".to_string()),
                redirect_url: "http://localhost:8081/auth/callback".to_string(),
            },
            session: dubbridge_config::GatewaySessionSettings {
                cookie_name: "dubbridge_session".to_string(),
                absolute_ttl_seconds: 28_800,
                idle_ttl_seconds: 1_800,
            },
        }
    }
}
