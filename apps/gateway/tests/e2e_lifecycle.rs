use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Request, Response, StatusCode},
};
use dubbridge_gateway::{build_app, state::GatewayState};
use tower::ServiceExt;

fn make_gateway_settings(
    token_url: &str,
    upstream_api_base_url: &str,
) -> dubbridge_config::GatewaySettings {
    dubbridge_config::GatewaySettings {
        port: 8081,
        upstream_api_base_url: upstream_api_base_url.to_string(),
        mobile_return_uris: vec!["dubbridge://auth/callback".to_string()],
        oauth: dubbridge_config::GatewayOAuthSettings {
            authorization_url: "http://localhost:9000/oauth/authorize".to_string(),
            token_url: token_url.to_string(),
            client_id: "test-client".to_string(),
            client_secret: Some("test-secret".to_string()),
            redirect_url: "http://localhost:8081/auth/callback".to_string(),
        },
        session: dubbridge_config::GatewaySessionSettings {
            cookie_name: "dubbridge_session".to_string(),
            absolute_ttl_seconds: 28_800,
            idle_ttl_seconds: 1_800,
        },
    }
}

fn make_app_config(gateway: dubbridge_config::GatewaySettings) -> dubbridge_config::AppConfig {
    dubbridge_config::AppConfig {
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
        gateway: Some(gateway.clone()),
    }
}

fn make_state(token_url: &str, upstream_api_base_url: &str) -> Arc<GatewayState> {
    let gateway = make_gateway_settings(token_url, upstream_api_base_url);
    let config = make_app_config(gateway.clone());
    Arc::new(GatewayState::new(reqwest::Client::new(), config, gateway))
}

fn build_test_app(token_url: &str, upstream_api_base_url: &str) -> Router {
    build_app(make_state(token_url, upstream_api_base_url))
}

async fn read_response(response: Response<Body>) -> (StatusCode, HeaderMap, String) {
    let (parts, body) = response.into_parts();
    let body = to_bytes(body, usize::MAX).await.unwrap();
    (
        parts.status,
        parts.headers,
        String::from_utf8(body.to_vec()).unwrap(),
    )
}

async fn request(app: &Router, request: Request<Body>) -> (StatusCode, HeaderMap, String) {
    let response = app.clone().oneshot(request).await.unwrap();
    read_response(response).await
}

#[tokio::test]
async fn e2e_browser_oauth_routes_are_retired() {
    let app = build_test_app("http://localhost:9/oauth/token", "http://127.0.0.1:1");

    let (login_status, login_headers, login_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/auth/login")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(login_status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(login_headers.get("content-type").is_none());
    assert!(login_body.is_empty());

    let (callback_status, callback_headers, callback_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/auth/callback?code=legacy&state=legacy")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(callback_status, StatusCode::NOT_FOUND);
    assert!(callback_headers.get("content-type").is_none());
    assert!(callback_body.is_empty());

    let (logout_status, logout_headers, logout_body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(logout_status, StatusCode::NOT_FOUND);
    assert!(logout_headers.get("content-type").is_none());
    assert!(logout_body.is_empty());
}

#[tokio::test]
async fn e2e_mobile_session_redeem_route_is_retired() {
    let app = build_test_app("http://localhost:9/oauth/token", "http://127.0.0.1:1");

    let (status, headers, body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/mobile/session")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"handoff_code":"legacy-handoff"}"#))
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(headers.get("content-type").is_none());
    assert!(body.is_empty());
}
