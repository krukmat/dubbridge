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
    matchers::{method, path},
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
async fn api_proxy_forwards_client_bearer_and_real_ip_without_session() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/assets"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-upstream", "ok")
                .set_body_string("asset-list"),
        )
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app("http://localhost:9/oauth/token", &upstream.uri());
    let (status, headers, body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/assets?view=full")
            .header(header::AUTHORIZATION, "Bearer mobile-jwt-token")
            .header("x-real-ip", "203.0.113.10")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get("x-upstream").unwrap(), "ok");
    assert_eq!(body, "asset-list");

    let requests = upstream
        .received_requests()
        .await
        .expect("upstream requests");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url.path(), "/assets");
    assert_eq!(requests[0].url.query(), Some("view=full"));
    assert_eq!(
        requests[0]
            .headers
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap(),
        "Bearer mobile-jwt-token"
    );
    assert_eq!(
        requests[0]
            .headers
            .get("x-real-ip")
            .unwrap()
            .to_str()
            .unwrap(),
        "203.0.113.10"
    );
    assert!(requests[0].headers.get("cookie").is_none());
}

#[tokio::test]
async fn api_proxy_without_bearer_or_session_returns_401() {
    let upstream = MockServer::start().await;
    let app = build_test_app("http://localhost:9/oauth/token", &upstream.uri());
    let (status, _headers, body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/assets")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.is_empty());
    assert_eq!(
        upstream.received_requests().await.expect("requests").len(),
        0
    );
}

#[tokio::test]
async fn api_proxy_relays_upstream_server_error_for_direct_bearer_flow() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/assets"))
        .respond_with(
            ResponseTemplate::new(503)
                .insert_header("content-type", "application/json")
                .set_body_raw(r#"{"error":"upstream unavailable"}"#, "application/json"),
        )
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app("http://localhost:9/oauth/token", &upstream.uri());
    let (status, _headers, body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/assets")
            .header(header::AUTHORIZATION, "Bearer mobile-jwt-token")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, r#"{"error":"upstream unavailable"}"#);
}
