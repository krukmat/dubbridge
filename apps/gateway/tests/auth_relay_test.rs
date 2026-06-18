use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, Response, StatusCode, header},
};
use dubbridge_gateway::{build_app, state::GatewayState};
use tower::ServiceExt;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string, method, path},
};

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

fn build_test_app(token_url: &str, upstream_api_base_url: &str) -> Router {
    let gateway = make_gateway_settings(token_url, upstream_api_base_url);
    let config = make_app_config(gateway.clone());
    let state = Arc::new(GatewayState::new(reqwest::Client::new(), config, gateway));
    build_app(state)
}

async fn read_response(response: Response<Body>) -> (StatusCode, axum::http::HeaderMap, String) {
    let (parts, body) = response.into_parts();
    let body = to_bytes(body, usize::MAX).await.expect("body");
    (
        parts.status,
        parts.headers,
        String::from_utf8(body.to_vec()).expect("utf8"),
    )
}

async fn request(
    app: &Router,
    request: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, String) {
    let response = app.clone().oneshot(request).await.expect("response");
    read_response(response).await
}

#[tokio::test]
async fn post_login_relays_upstream_status_headers_and_json() {
    let upstream = MockServer::start().await;
    let login_body = r#"{"email":"owner@example.com","password":"correct horse battery staple"}"#;
    let upstream_json = r#"{"token":"jwt-login","userId":"user-1","workspaceId":"workspace-1"}"#;
    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .and(body_string(login_body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .insert_header("x-upstream-auth", "login")
                .set_body_raw(upstream_json, "application/json"),
        )
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app("http://localhost:9/oauth/token", &upstream.uri());
    let (status, headers, body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(login_body))
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get("x-upstream-auth").unwrap(), "login");
    assert_eq!(body, upstream_json);
}

#[tokio::test]
async fn post_register_relays_created_response_verbatim() {
    let upstream = MockServer::start().await;
    let register_body = r#"{"email":"owner@example.com","password":"correct horse battery staple","workspaceName":"DubBridge"}"#;
    let upstream_json = r#"{"token":"jwt-register","userId":"user-2","workspaceId":"workspace-2"}"#;
    Mock::given(method("POST"))
        .and(path("/auth/register"))
        .and(body_string(register_body))
        .respond_with(
            ResponseTemplate::new(201)
                .insert_header("content-type", "application/json")
                .set_body_raw(upstream_json, "application/json"),
        )
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app("http://localhost:9/oauth/token", &upstream.uri());
    let (status, _headers, body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/register")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(register_body))
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body, upstream_json);
}

#[tokio::test]
async fn auth_relay_preserves_upstream_client_error_payload() {
    let upstream = MockServer::start().await;
    let register_body = r#"{"email":"owner@example.com","password":"correct horse battery staple","workspaceName":"DubBridge"}"#;
    let upstream_json = r#"{"error":"account already exists"}"#;
    Mock::given(method("POST"))
        .and(path("/auth/register"))
        .and(body_string(register_body))
        .respond_with(
            ResponseTemplate::new(409)
                .insert_header("content-type", "application/json")
                .set_body_raw(upstream_json, "application/json"),
        )
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app("http://localhost:9/oauth/token", &upstream.uri());
    let (status, _headers, body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/register")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(register_body))
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body, upstream_json);
}

#[tokio::test]
async fn auth_relay_returns_bad_gateway_when_upstream_is_unreachable() {
    let app = build_test_app("http://localhost:9/oauth/token", "http://127.0.0.1:1");
    let (status, _headers, body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"email":"owner@example.com","password":"correct horse battery staple"}"#,
            ))
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert!(body.is_empty());
}

#[tokio::test]
async fn mobile_session_redeem_route_is_not_exposed_anymore() {
    let app = build_test_app("http://localhost:9/oauth/token", "http://127.0.0.1:1");
    let (status, headers, body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/mobile/session")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"handoff_code":"legacy-code"}"#))
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(headers.get(header::CONTENT_TYPE).is_none());
    assert!(body.is_empty());
}

#[tokio::test]
async fn get_login_redirect_route_is_not_exposed_anymore() {
    let app = build_test_app("http://localhost:9/oauth/token", "http://127.0.0.1:1");
    let (status, headers, body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/auth/login")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert!(headers.get(header::CONTENT_TYPE).is_none());
    assert!(body.is_empty());
}

#[tokio::test]
async fn callback_route_is_not_exposed_anymore() {
    let app = build_test_app("http://localhost:9/oauth/token", "http://127.0.0.1:1");
    let (status, headers, body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/auth/callback?code=legacy&state=legacy")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(headers.get(header::CONTENT_TYPE).is_none());
    assert!(body.is_empty());
}

#[tokio::test]
async fn logout_route_is_not_exposed_anymore() {
    let app = build_test_app("http://localhost:9/oauth/token", "http://127.0.0.1:1");
    let (status, headers, body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(headers.get(header::CONTENT_TYPE).is_none());
    assert!(body.is_empty());
}
