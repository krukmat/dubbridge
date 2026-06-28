// P1-T5.3: HTTP proxy handler + route mount

use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{OriginalUri, State},
    http::{HeaderMap, Method, StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{any, get},
};

use crate::state::GatewayState;

/// Builds the `/api/*` sub-router. State is inherited from the parent router.
pub fn proxy_router() -> Router<Arc<GatewayState>> {
    Router::new()
        // Playback manifest and segment are authorized by grantId in the path, not by session
        // Bearer. The native video player cannot attach session headers, so these routes must
        // bypass the session-Bearer gate and let the upstream API validate the grant.
        .route(
            "/assets/{id}/playback/{grant_id}/manifest",
            get(public_proxy_handler),
        )
        .route(
            "/assets/{id}/playback/segments/{filename}",
            get(public_proxy_handler),
        )
        .route("/", any(proxy_handler))
        .route("/{*path}", any(proxy_handler))
}

/// Grant-scoped passthrough for playback manifest and segment routes.
/// No session Bearer required — the upstream API validates via the grantId path parameter.
pub async fn public_proxy_handler(
    State(app_state): State<Arc<GatewayState>>,
    method: Method,
    headers: HeaderMap,
    OriginalUri(original_uri): OriginalUri,
    body: Body,
) -> Response {
    let upstream_url =
        match build_upstream_url(&app_state.gateway.upstream_api_base_url, &original_uri) {
            Ok(url) => url,
            Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
        };

    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let mut upstream_request = app_state
        .http_client
        .request(method, upstream_url)
        .body(body_bytes.to_vec());

    for (name, value) in &headers {
        if matches!(
            name,
            &header::COOKIE | &header::AUTHORIZATION | &header::HOST
        ) {
            continue;
        }
        if name.as_str().eq_ignore_ascii_case("x-dubbridge-session") {
            continue;
        }
        upstream_request = upstream_request.header(name, value);
    }

    let upstream_response = match upstream_request.send().await {
        Ok(response) => response,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    relay_upstream_response(upstream_response).await
}

/// Authenticated catch-all proxy into `apps/api`.
pub async fn proxy_handler(
    State(app_state): State<Arc<GatewayState>>,
    method: Method,
    headers: HeaderMap,
    OriginalUri(original_uri): OriginalUri,
    body: Body,
) -> Response {
    let Some(access_token) = extract_bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let upstream_url =
        match build_upstream_url(&app_state.gateway.upstream_api_base_url, &original_uri) {
            Ok(url) => url,
            Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
        };

    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let mut upstream_request = app_state
        .http_client
        .request(method, upstream_url)
        .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
        .body(body_bytes.to_vec());

    for (name, value) in &headers {
        if matches!(
            name,
            &header::COOKIE | &header::AUTHORIZATION | &header::HOST
        ) {
            continue;
        }
        if name.as_str().eq_ignore_ascii_case("x-dubbridge-session") {
            continue;
        }
        upstream_request = upstream_request.header(name, value);
    }

    let upstream_response = match upstream_request.send().await {
        Ok(response) => response,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    relay_upstream_response(upstream_response).await
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

fn build_upstream_url(base_url: &str, original_uri: &Uri) -> Result<String, ()> {
    let path_and_query = original_uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/api");
    let stripped = path_and_query.strip_prefix("/api").ok_or(())?;
    let relative = if stripped.is_empty() || stripped.starts_with('?') {
        "/"
    } else {
        stripped
    };
    let query_suffix = if stripped.starts_with('?') {
        stripped
    } else {
        ""
    };

    Ok(format!(
        "{}{}{}",
        base_url.trim_end_matches('/'),
        relative,
        query_suffix
    ))
}

async fn relay_upstream_response(upstream_response: reqwest::Response) -> Response {
    let status = upstream_response.status();
    let upstream_headers = upstream_response.headers().clone();
    let body = match upstream_response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    let mut response_headers = HeaderMap::new();
    for (name, value) in &upstream_headers {
        if matches!(name, &header::SET_COOKIE | &header::TRANSFER_ENCODING) {
            continue;
        }
        response_headers.append(name, value.clone());
    }

    (status, response_headers, body).into_response()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, Request, StatusCode, header},
    };
    use tower::ServiceExt;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::{build_app, proxy::extract_bearer_token, state::GatewayState};

    fn make_state(upstream_api_base_url: &str) -> Arc<GatewayState> {
        let gw = dubbridge_config::GatewaySettings {
            port: 8081,
            upstream_api_base_url: upstream_api_base_url.to_string(),
            mobile_return_uris: vec!["dubbridge://auth/callback".to_string()],
            oauth: dubbridge_config::GatewayOAuthSettings {
                authorization_url: "http://localhost:9000/oauth/authorize".to_string(),
                token_url: "http://localhost:9000/oauth/token".to_string(),
                client_id: "test-client".to_string(),
                client_secret: Some("test-secret".to_string()),
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
        Arc::new(GatewayState::new(reqwest::Client::new(), cfg, gw))
    }

    async fn read_body_text(body: Body) -> String {
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[test]
    fn extract_bearer_token_returns_token_without_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, "Bearer jwt-token".parse().unwrap());

        assert_eq!(extract_bearer_token(&headers).as_deref(), Some("jwt-token"));
    }

    #[test]
    fn extract_bearer_token_returns_none_for_non_bearer_authorization() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, "Basic abc123".parse().unwrap());

        assert_eq!(extract_bearer_token(&headers), None);
    }

    #[tokio::test]
    async fn proxy_without_bearer_returns_401_and_skips_upstream() {
        let upstream = MockServer::start().await;
        let app = build_app(make_state(&upstream.uri()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/tracks?kind=test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(upstream.received_requests().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn proxy_ignores_legacy_cookie_session_transport() {
        let upstream = MockServer::start().await;
        let app = build_app(make_state(&upstream.uri()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets")
                    .header("cookie", "dubbridge_session=legacy-session-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(upstream.received_requests().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn proxy_ignores_legacy_mobile_session_header_transport() {
        let upstream = MockServer::start().await;
        let app = build_app(make_state(&upstream.uri()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets")
                    .header("x-dubbridge-session", "legacy-mobile-session-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(upstream.received_requests().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn proxy_prefers_client_bearer_and_strips_sensitive_headers() {
        let upstream = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/tracks/upload"))
            .respond_with(
                ResponseTemplate::new(201)
                    .insert_header("x-upstream-id", "abc123")
                    .set_body_string("proxied-ok"),
            )
            .mount(&upstream)
            .await;

        let app = build_app(make_state(&upstream.uri()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tracks/upload?part=7")
                    .header("cookie", "dubbridge_session=legacy-session-id")
                    .header("x-dubbridge-session", "legacy-mobile-session-id")
                    .header("authorization", "Bearer client-supplied-token")
                    .header("host", "malicious-client.example")
                    .header("content-type", "application/json")
                    .header("accept", "application/json")
                    .header("x-request-id", "req-123")
                    .body(Body::from("{\"chunk\":7}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers().get("x-upstream-id").unwrap(), "abc123");
        assert_eq!(read_body_text(response.into_body()).await, "proxied-ok");

        let requests = upstream.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.url.path(), "/tracks/upload");
        assert_eq!(request.url.query(), Some("part=7"));
        assert_eq!(
            request
                .headers
                .get("authorization")
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer client-supplied-token"
        );
        assert!(request.headers.get("cookie").is_none());
        assert!(request.headers.get("x-dubbridge-session").is_none());
        assert_ne!(
            request.headers.get("host").unwrap().to_str().unwrap(),
            "malicious-client.example"
        );
        assert_eq!(
            request
                .headers
                .get("x-request-id")
                .unwrap()
                .to_str()
                .unwrap(),
            "req-123"
        );
        assert_eq!(request.body, br#"{"chunk":7}"#);
    }

    #[tokio::test]
    async fn proxy_relays_upstream_403_and_strips_outbound_set_cookie_and_transfer_encoding() {
        let upstream = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/forbidden"))
            .respond_with(
                ResponseTemplate::new(403)
                    .insert_header("set-cookie", "api_session=bad")
                    .insert_header("transfer-encoding", "chunked")
                    .insert_header("x-upstream-error", "denied")
                    .set_body_string("forbidden"),
            )
            .mount(&upstream)
            .await;

        let app = build_app(make_state(&upstream.uri()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/forbidden")
                    .header("authorization", "Bearer client-supplied-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(response.headers().get("set-cookie").is_none());
        assert!(response.headers().get("transfer-encoding").is_none());
        assert_eq!(
            response.headers().get("x-upstream-error").unwrap(),
            "denied"
        );
        assert_eq!(read_body_text(response.into_body()).await, "forbidden");
    }

    #[tokio::test]
    async fn playback_manifest_proxied_without_bearer() {
        let upstream = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/assets/asset-abc/playback/grant-xyz/manifest"))
            .respond_with(ResponseTemplate::new(200).set_body_string("#EXTM3U"))
            .mount(&upstream)
            .await;

        let app = build_app(make_state(&upstream.uri()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets/asset-abc/playback/grant-xyz/manifest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(read_body_text(response.into_body()).await, "#EXTM3U");
        let requests = upstream.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].headers.get("authorization").is_none());
    }

    #[tokio::test]
    async fn playback_segment_proxied_without_bearer() {
        let upstream = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/assets/asset-abc/playback/segments/seg0.ts"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8, 1, 2]))
            .mount(&upstream)
            .await;

        let app = build_app(make_state(&upstream.uri()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets/asset-abc/playback/segments/seg0.ts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let requests = upstream.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].headers.get("authorization").is_none());
    }
}
