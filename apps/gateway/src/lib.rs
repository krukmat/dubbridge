pub mod auth; // public credential relay routes
pub mod proxy; // bearer-only HTTP proxy handler
pub mod state;

use std::sync::Arc;
use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;

use crate::{auth::auth_router, proxy::proxy_router, state::GatewayState};

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
    component: &'static str,
    component_status: &'static str,
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
        component: "gateway",
        component_status: "ok",
    })
}

async fn ready(State(state): State<Arc<GatewayState>>) -> (StatusCode, Json<HealthResponse>) {
    let url = format!("{}/health/ready", state.gateway.upstream_api_base_url);

    let client = &state.http_client;
    let timeout = Duration::from_secs(2);

    let result = client.get(&url).timeout(timeout).send().await;

    match result {
        Ok(resp) => {
            if resp.status().is_success() {
                (
                    StatusCode::OK,
                    Json(HealthResponse {
                        service: "gateway",
                        status: "ready",
                        component: "api",
                        component_status: "ok",
                    }),
                )
            } else {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(HealthResponse {
                        service: "gateway",
                        status: "not_ready",
                        component: "api",
                        component_status: "unreachable",
                    }),
                )
            }
        }
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                service: "gateway",
                status: "not_ready",
                component: "api",
                component_status: "unreachable",
            }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::{build_app, state::GatewayState};

    #[tokio::test]
    async fn health_live_is_public_and_independent() {
        let state = Arc::new(GatewayState::new(
            reqwest::Client::new(),
            sample_config(),
            sample_gateway_settings(),
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
    }

    #[tokio::test]
    async fn health_ready_ok_when_upstream_ok() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health/ready"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let mut settings = sample_gateway_settings();
        settings.upstream_api_base_url = mock_server.uri();

        let state = Arc::new(GatewayState::new(
            reqwest::Client::new(),
            sample_config(),
            settings,
        ));
        let app = build_app(state);

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

    #[tokio::test]
    async fn health_ready_unavailable_when_upstream_503() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health/ready"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let mut settings = sample_gateway_settings();
        settings.upstream_api_base_url = mock_server.uri();

        let state = Arc::new(GatewayState::new(
            reqwest::Client::new(),
            sample_config(),
            settings,
        ));
        let app = build_app(state);

        let ready = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn health_ready_unavailable_when_upstream_connection_fails() {
        // Use a port that is likely closed or a non-routable address
        let mut settings = sample_gateway_settings();
        settings.upstream_api_base_url = "http://127.0.0.1:1".to_string(); // Port 1 is typically closed

        let state = Arc::new(GatewayState::new(
            reqwest::Client::new(),
            sample_config(),
            settings,
        ));
        let app = build_app(state);

        let ready = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    fn sample_config() -> dubbridge_config::AppConfig {
        dubbridge_config::AppConfig {
            env: dubbridge_config::AppEnv::Local,
            api_port: 8080,
            database_url: "postgres://dubbridge:dubbridge@localhost:5432/dubbridge".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            worker_concurrency: 4,
            storage: dubbridge_config::StorageSettings {
                base_path: "/tmp/dubbridge-storage".to_string(),
                bucket: "dubbridge-local".to_string(),
                ..Default::default()
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
            mobile_return_uris: vec!["dubbridge://auth/callback".to_string()],
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
