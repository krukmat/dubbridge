use std::{collections::BTreeMap, sync::Arc};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Request, Response, StatusCode},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use dubbridge_gateway::{
    auth::{handoff::HandoffStore, pending::PendingAuthStore},
    build_app,
    cookie_ext::MOBILE_SESSION_HEADER,
    session::store::InMemorySessionStore,
    state::GatewayState,
};
use tower::ServiceExt;
use url::Url;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, method, path},
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

fn make_state(token_url: &str, upstream_api_base_url: &str) -> Arc<GatewayState> {
    let gateway = make_gateway_settings(token_url, upstream_api_base_url);
    let config = make_app_config(gateway.clone());
    Arc::new(GatewayState::new(
        reqwest::Client::new(),
        config,
        gateway,
        Arc::new(InMemorySessionStore::new()),
        Arc::new(PendingAuthStore::with_default_ttl()),
        Arc::new(HandoffStore::with_default_ttl()),
    ))
}

fn build_test_app(token_url: &str, upstream_api_base_url: &str) -> Router {
    build_app(make_state(token_url, upstream_api_base_url))
}

fn jwt_with_subject(subject: &str) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(format!(r#"{{"sub":"{subject}"}}"#).as_bytes());
    format!("{header}.{payload}.sig")
}

fn parse_state_from_location(location: &str) -> String {
    let url = Url::parse(location).expect("login redirect must be a valid absolute URL");
    url.query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .expect("login redirect must include state")
}

fn merge_set_cookie_headers(headers: &HeaderMap, jar: &mut BTreeMap<String, String>) {
    for header_value in headers.get_all("set-cookie") {
        let cookie = header_value
            .to_str()
            .expect("set-cookie headers must be valid ascii");
        let Some((name, value)) = cookie
            .split(';')
            .next()
            .and_then(|pair| pair.split_once('='))
        else {
            continue;
        };

        if cookie.contains("Max-Age=0") {
            jar.remove(name);
        } else {
            jar.insert(name.to_string(), value.to_string());
        }
    }
}

fn cookie_header(jar: &BTreeMap<String, String>) -> String {
    jar.iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn response_contains_secret(headers: &HeaderMap, body: &str, secret: &str) -> bool {
    if body.contains(secret) {
        return true;
    }

    headers.iter().any(|(name, value)| {
        name.as_str().contains(secret)
            || value
                .to_str()
                .map(|as_str| as_str.contains(secret))
                .unwrap_or(false)
    })
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
async fn e2e_login_refresh_logout_lifecycle_is_deterministic() {
    let initial_access_token = jwt_with_subject("user-e2e");
    let refreshed_access_token = "refreshed-access-token-never-leak";

    let auth_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": initial_access_token,
            "refresh_token": "refresh-token-1",
            "expires_in": 0,
            "token_type": "Bearer"
        })))
        .expect(1)
        .mount(&auth_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": refreshed_access_token,
            "refresh_token": "refresh-token-2",
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .expect(1)
        .mount(&auth_server)
        .await;

    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/assets"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-upstream", "ok")
                .set_body_string("asset-list-ok"),
        )
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app(
        &format!("{}/oauth/token", auth_server.uri()),
        &upstream.uri(),
    );

    let (login_status, login_headers, login_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/auth/login")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(login_status, StatusCode::SEE_OTHER);
    assert!(login_body.is_empty());

    let login_location = login_headers
        .get("location")
        .expect("login redirect must set location")
        .to_str()
        .unwrap();
    let login_state = parse_state_from_location(login_location);

    let (callback_status, callback_headers, callback_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri(format!("/auth/callback?code=code-123&state={login_state}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(callback_status, StatusCode::OK);
    assert!(
        !response_contains_secret(&callback_headers, &callback_body, &initial_access_token),
        "callback response must not expose the initial access token"
    );
    assert!(
        !response_contains_secret(&callback_headers, &callback_body, refreshed_access_token),
        "callback response must not expose the refreshed access token"
    );

    let mut jar = BTreeMap::new();
    merge_set_cookie_headers(&callback_headers, &mut jar);
    let initial_session_cookie = jar
        .get("dubbridge_session")
        .cloned()
        .expect("callback must set the session cookie");
    assert!(
        !initial_session_cookie.contains(&initial_access_token),
        "session cookie must never contain the initial access token"
    );

    let (proxy_status, proxy_headers, proxy_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/assets?view=full")
            .header("cookie", cookie_header(&jar))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(proxy_status, StatusCode::OK);
    assert_eq!(proxy_body, "asset-list-ok");
    assert_eq!(proxy_headers.get("x-upstream").unwrap(), "ok");
    assert!(
        !response_contains_secret(&proxy_headers, &proxy_body, &initial_access_token),
        "proxy response must not expose the initial access token"
    );
    assert!(
        !response_contains_secret(&proxy_headers, &proxy_body, refreshed_access_token),
        "proxy response must not expose the refreshed access token"
    );

    let stale_cookie_header = cookie_header(&jar);
    merge_set_cookie_headers(&proxy_headers, &mut jar);
    let refreshed_session_cookie = jar
        .get("dubbridge_session")
        .cloned()
        .expect("refresh path must rotate the session cookie");
    assert_ne!(refreshed_session_cookie, initial_session_cookie);
    assert!(
        !refreshed_session_cookie.contains(refreshed_access_token),
        "rotated session cookie must never contain the refreshed access token"
    );

    let upstream_requests = upstream.received_requests().await.unwrap();
    assert_eq!(upstream_requests.len(), 1);
    assert_eq!(upstream_requests[0].url.path(), "/assets");
    assert_eq!(upstream_requests[0].url.query(), Some("view=full"));
    assert_eq!(
        upstream_requests[0]
            .headers
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap(),
        format!("Bearer {refreshed_access_token}")
    );
    assert!(
        upstream_requests[0].headers.get("cookie").is_none(),
        "gateway must not forward the browser cookie upstream"
    );

    let (logout_status, logout_headers, logout_body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .header("cookie", cookie_header(&jar))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(logout_status, StatusCode::OK);
    assert!(logout_body.is_empty());
    assert!(
        !response_contains_secret(&logout_headers, &logout_body, &initial_access_token),
        "logout response must not expose the initial access token"
    );
    assert!(
        !response_contains_secret(&logout_headers, &logout_body, refreshed_access_token),
        "logout response must not expose the refreshed access token"
    );

    let (post_logout_status, _, _) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/assets?view=full")
            .header("cookie", stale_cookie_header)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(post_logout_status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        upstream.received_requests().await.unwrap().len(),
        1,
        "logout must remove the session so stale client cookies cannot hit upstream"
    );
}

#[tokio::test]
async fn e2e_access_tokens_never_appear_in_client_visible_responses_or_cookies() {
    let initial_access_token = jwt_with_subject("user-no-leak");

    let auth_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": initial_access_token,
            "refresh_token": "refresh-token-1",
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .expect(1)
        .mount(&auth_server)
        .await;

    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/profile"))
        .respond_with(ResponseTemplate::new(200).set_body_string("profile-ok"))
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app(
        &format!("{}/oauth/token", auth_server.uri()),
        &upstream.uri(),
    );

    let (_, login_headers, _) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/auth/login")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let login_state =
        parse_state_from_location(login_headers.get("location").unwrap().to_str().unwrap());

    let (_, callback_headers, callback_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri(format!("/auth/callback?code=code-456&state={login_state}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert!(
        !response_contains_secret(&callback_headers, &callback_body, &initial_access_token),
        "callback response must keep the access token server-side"
    );

    let mut jar = BTreeMap::new();
    merge_set_cookie_headers(&callback_headers, &mut jar);
    let cookie_snapshot = cookie_header(&jar);
    assert!(
        !cookie_snapshot.contains(&initial_access_token),
        "browser cookies must not contain the access token"
    );

    let (_, proxy_headers, proxy_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/profile")
            .header("cookie", cookie_snapshot)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(proxy_body, "profile-ok");
    assert!(
        !response_contains_secret(&proxy_headers, &proxy_body, &initial_access_token),
        "proxied response must not expose the access token back to the client"
    );
}

#[tokio::test]
async fn e2e_mobile_handoff_refresh_logout_lifecycle_is_deterministic() {
    let initial_access_token = jwt_with_subject("user-mobile-e2e");
    let refreshed_access_token = "mobile-refreshed-access-token-never-leak";
    let refreshed_refresh_token = "mobile-refresh-token-never-leak";
    let return_uri = "dubbridge://auth/callback";

    let auth_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": initial_access_token,
            "refresh_token": "mobile-refresh-token-1",
            "expires_in": 0,
            "token_type": "Bearer"
        })))
        .expect(1)
        .mount(&auth_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": refreshed_access_token,
            "refresh_token": refreshed_refresh_token,
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .expect(1)
        .mount(&auth_server)
        .await;

    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/assets"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-upstream", "mobile-ok")
                .set_body_string("mobile-asset-list-ok"),
        )
        .expect(1)
        .mount(&upstream)
        .await;

    let app = build_test_app(
        &format!("{}/oauth/token", auth_server.uri()),
        &upstream.uri(),
    );

    let (login_status, login_headers, login_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/auth/login?return_uri=dubbridge%3A%2F%2Fauth%2Fcallback")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(login_status, StatusCode::SEE_OTHER);
    assert!(login_body.is_empty());

    let login_location = login_headers
        .get("location")
        .expect("login redirect must set location")
        .to_str()
        .unwrap();
    let login_state = parse_state_from_location(login_location);

    let (callback_status, callback_headers, callback_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri(format!(
                "/auth/callback?code=mobile-code&state={login_state}"
            ))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(callback_status, StatusCode::SEE_OTHER);
    assert!(callback_body.is_empty());
    assert!(
        callback_headers
            .get_all("set-cookie")
            .iter()
            .next()
            .is_none(),
        "mobile callback must not emit cookies"
    );

    let callback_location = callback_headers
        .get("location")
        .expect("mobile callback must redirect")
        .to_str()
        .unwrap();
    assert!(callback_location.starts_with(return_uri));
    assert!(
        !response_contains_secret(&callback_headers, callback_location, &initial_access_token),
        "mobile callback location must not expose the initial access token"
    );
    assert!(
        !response_contains_secret(
            &callback_headers,
            callback_location,
            "mobile-refresh-token-1"
        ),
        "mobile callback location must not expose the initial refresh token"
    );

    let callback_url = Url::parse(callback_location).unwrap();
    let handoff_code = callback_url
        .query_pairs()
        .find(|(key, _)| key == "handoff_code")
        .map(|(_, value)| value.into_owned())
        .expect("mobile callback must include handoff_code");

    let (redeem_status, redeem_headers, redeem_body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/mobile/session")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({ "handoff_code": handoff_code }).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(redeem_status, StatusCode::OK);
    assert!(
        !response_contains_secret(&redeem_headers, &redeem_body, &initial_access_token),
        "redeem response must not expose the initial access token"
    );
    assert!(
        !response_contains_secret(&redeem_headers, &redeem_body, "mobile-refresh-token-1"),
        "redeem response must not expose the initial refresh token"
    );

    let redeem_json: serde_json::Value = serde_json::from_str(&redeem_body).unwrap();
    let initial_session_ref = redeem_json
        .get("session_ref")
        .and_then(|value| value.as_str())
        .expect("redeem response must contain session_ref")
        .to_string();
    assert_eq!(redeem_json.as_object().map(|obj| obj.len()), Some(1));

    let (proxy_status, proxy_headers, proxy_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/assets?view=mobile")
            .header(MOBILE_SESSION_HEADER, initial_session_ref.as_str())
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(proxy_status, StatusCode::OK);
    assert_eq!(proxy_body, "mobile-asset-list-ok");
    assert_eq!(proxy_headers.get("x-upstream").unwrap(), "mobile-ok");
    assert!(
        !response_contains_secret(&proxy_headers, &proxy_body, &initial_access_token),
        "mobile proxy response must not expose the initial access token"
    );
    assert!(
        !response_contains_secret(&proxy_headers, &proxy_body, refreshed_access_token),
        "mobile proxy response must not expose the refreshed access token"
    );
    assert!(
        !response_contains_secret(&proxy_headers, &proxy_body, refreshed_refresh_token),
        "mobile proxy response must not expose the refreshed refresh token"
    );
    assert!(
        proxy_headers.get_all("set-cookie").iter().next().is_none(),
        "mobile proxy path must not emit browser cookies"
    );

    let rotated_session_ref = proxy_headers
        .get(MOBILE_SESSION_HEADER)
        .expect("mobile refresh must return rotated session ref header")
        .to_str()
        .unwrap()
        .to_string();
    assert_ne!(rotated_session_ref, initial_session_ref);
    assert!(
        !rotated_session_ref.contains(refreshed_access_token),
        "rotated session ref must never contain the access token"
    );
    assert!(
        !rotated_session_ref.contains(refreshed_refresh_token),
        "rotated session ref must never contain the refresh token"
    );

    let upstream_requests = upstream.received_requests().await.unwrap();
    assert_eq!(upstream_requests.len(), 1);
    assert_eq!(upstream_requests[0].url.path(), "/assets");
    assert_eq!(upstream_requests[0].url.query(), Some("view=mobile"));
    assert_eq!(
        upstream_requests[0]
            .headers
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap(),
        format!("Bearer {refreshed_access_token}")
    );
    assert!(
        upstream_requests[0]
            .headers
            .get(MOBILE_SESSION_HEADER)
            .is_none(),
        "gateway must not forward X-Dubbridge-Session upstream"
    );

    let (logout_status, logout_headers, logout_body) = request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .header(MOBILE_SESSION_HEADER, rotated_session_ref.as_str())
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(logout_status, StatusCode::OK);
    assert!(logout_body.is_empty());
    assert!(
        logout_headers.get_all("set-cookie").iter().next().is_none(),
        "mobile logout must not emit cookie clears"
    );
    assert!(
        !response_contains_secret(&logout_headers, &logout_body, refreshed_access_token),
        "logout response must not expose the refreshed access token"
    );
    assert!(
        !response_contains_secret(&logout_headers, &logout_body, refreshed_refresh_token),
        "logout response must not expose the refreshed refresh token"
    );

    let (post_logout_status, post_logout_headers, post_logout_body) = request(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/assets?view=mobile")
            .header(MOBILE_SESSION_HEADER, initial_session_ref.as_str())
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(post_logout_status, StatusCode::UNAUTHORIZED);
    assert!(
        !response_contains_secret(
            &post_logout_headers,
            &post_logout_body,
            refreshed_access_token
        ),
        "stale-session response must not expose the refreshed access token"
    );
    assert_eq!(
        upstream.received_requests().await.unwrap().len(),
        1,
        "stale mobile session refs must not reach upstream after logout"
    );
}
