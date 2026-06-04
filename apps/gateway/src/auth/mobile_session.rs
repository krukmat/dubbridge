// P1-T7.3: POST /auth/mobile/session handoff redemption (ADR-024 mobile seam)
//
// Exchanges a short-lived one-time handoff code for the opaque gateway session
// reference. Never returns access tokens or refresh tokens.

use std::sync::Arc;

use axum::Json;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{auth::handoff::HandoffError, state::GatewayState};

#[derive(Serialize)]
pub struct MobileSessionResponse {
    pub session_ref: String,
}

pub async fn mobile_session_handler(
    State(app_state): State<Arc<GatewayState>>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let Some(handoff_code) = payload
        .get("handoff_code")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let session_id = match app_state.handoff_store.consume(handoff_code) {
        Ok(id) => id,
        Err(HandoffError::NotFound | HandoffError::Expired) => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    (
        StatusCode::OK,
        Json(MobileSessionResponse {
            session_ref: session_id.as_str().to_string(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::post,
    };
    use serde_json::json;
    use tower::ServiceExt;

    use crate::{
        auth::{
            handoff::HandoffStore, mobile_session::mobile_session_handler,
            pending::PendingAuthStore,
        },
        session::store::InMemorySessionStore,
        state::GatewayState,
    };

    fn make_state() -> Arc<GatewayState> {
        let gw = dubbridge_config::GatewaySettings {
            port: 8081,
            upstream_api_base_url: "http://localhost:8080".to_string(),
            mobile_return_uris: vec!["dubbridge://auth/callback".to_string()],
            oauth: dubbridge_config::GatewayOAuthSettings {
                authorization_url: "http://localhost:9000/oauth/authorize".to_string(),
                token_url: "http://localhost:9000/oauth/token".to_string(),
                client_id: "test-client".to_string(),
                client_secret: Some("secret".to_string()),
                redirect_url: "http://localhost:8081/auth/callback".to_string(),
            },
            session: dubbridge_config::GatewaySessionSettings {
                cookie_name: "dubbridge_session".to_string(),
                absolute_ttl_seconds: 28_800,
                idle_ttl_seconds: 1_800,
            },
        };
        let cfg = dubbridge_config::AppConfig {
            env: dubbridge_config::AppEnv::Local,
            api_port: 8080,
            database_url: "postgres://x:x@localhost/x".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            worker_concurrency: 1,
            storage: dubbridge_config::StorageSettings {
                backend: dubbridge_config::StorageBackend::LocalFs,
                base_path: "/tmp".to_string(),
                bucket: "local".to_string(),
                endpoint_url: None,
            },
            observability: dubbridge_config::ObsSettings {
                log_format: dubbridge_config::LogFormat::Pretty,
                filter: "info".to_string(),
            },
            auth: None,
            gateway: Some(gw.clone()),
        };
        Arc::new(GatewayState::new(
            reqwest::Client::new(),
            cfg,
            gw,
            Arc::new(InMemorySessionStore::new()),
            Arc::new(PendingAuthStore::with_default_ttl()),
            Arc::new(HandoffStore::with_default_ttl()),
        ))
    }

    fn build_router(state: Arc<GatewayState>) -> Router {
        Router::new()
            .route("/auth/mobile/session", post(mobile_session_handler))
            .with_state(state)
    }

    #[tokio::test]
    async fn redeem_valid_handoff_code_returns_only_session_ref() {
        let state = make_state();
        let session_id = crate::session::SessionId::generate();
        let expected = session_id.as_str().to_string();
        let handoff_code = state.handoff_store.issue(session_id);

        let response = build_router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/mobile/session")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "handoff_code": handoff_code }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            value.get("session_ref").and_then(|v| v.as_str()),
            Some(expected.as_str())
        );
        assert_eq!(value.as_object().map(|obj| obj.len()), Some(1));
    }

    #[tokio::test]
    async fn redeem_same_handoff_code_twice_returns_401_on_second_call() {
        let state = make_state();
        let handoff_code = state
            .handoff_store
            .issue(crate::session::SessionId::generate());
        let app = build_router(state);
        let request_body = json!({ "handoff_code": handoff_code }).to_string();

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/mobile/session")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/mobile/session")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn redeem_missing_handoff_code_field_returns_400() {
        let response = build_router(make_state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/mobile/session")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn redeem_non_string_handoff_code_returns_400() {
        let response = build_router(make_state())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/mobile/session")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{ "handoff_code": 7 }"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
