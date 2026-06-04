// P1-T4: /auth/login and /auth/callback handlers (ADR-024, ADR-018)
//
// GET /auth/login  → generate PKCE + state, store in pending_store, redirect to AS
// GET /auth/callback → validate state (single-use), exchange code, create session, set cookies

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use base64::Engine;
use serde::Deserialize;
use url::Url;

use crate::{
    auth::handoff::HandoffStore,
    auth::oauth_client::{
        OAuthError, OAuthExecutor, OAuthState, PkceChallenge, PkceVerifier,
        build_authorization_url, build_token_exchange_params,
    },
    auth::pending::PendingAuth,
    cookie::{build_csrf_cookie, build_session_cookie},
    session::{CsrfToken, StoredSession},
    state::GatewayState,
};

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize, Default)]
pub struct LoginParams {
    pub return_uri: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /auth/login
/// Generates PKCE verifier + challenge + state, stores pending entry, redirects.
pub async fn login_handler(
    State(app_state): State<Arc<GatewayState>>,
    Query(params): Query<LoginParams>,
) -> Response {
    let return_uri = match params.return_uri {
        Some(uri)
            if is_registered_mobile_return_uri(&app_state.gateway.mobile_return_uris, &uri) =>
        {
            Some(uri)
        }
        Some(_) => return StatusCode::BAD_REQUEST.into_response(),
        None => None,
    };

    let verifier = PkceVerifier::generate();
    let challenge = PkceChallenge::from_verifier(&verifier);
    let oauth_state = OAuthState::generate();

    app_state
        .pending_store
        .insert(oauth_state.as_str(), PendingAuth::new(verifier, return_uri));

    let oauth = &app_state.gateway.oauth;
    let url = match build_authorization_url(
        &oauth.authorization_url,
        &oauth.client_id,
        &oauth.redirect_url,
        &oauth_state,
        &challenge,
        Some("openid profile"),
    ) {
        Ok(u) => u,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    Redirect::to(&url).into_response()
}

/// GET /auth/callback?code=...&state=...
/// Validates state (single-use), exchanges code, creates session, sets cookies.
pub async fn callback_handler(
    State(app_state): State<Arc<GatewayState>>,
    Query(params): Query<CallbackParams>,
) -> Response {
    // Consume state atomically — single-use invariant (ADR-024)
    let pending = match app_state.pending_store.consume(&params.state) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let (verifier, return_uri) = pending.into_parts();

    if let Some(ref return_uri) = return_uri {
        if !is_registered_mobile_return_uri(&app_state.gateway.mobile_return_uris, return_uri) {
            return StatusCode::BAD_REQUEST.into_response();
        }
    }

    let oauth = &app_state.gateway.oauth;
    let exchange_params = build_token_exchange_params(
        &oauth.client_id,
        &params.code,
        &verifier,
        &oauth.redirect_url,
    );

    let executor = OAuthExecutor::new(&app_state.http_client);
    let token_set = match executor
        .send_token_request(
            &oauth.token_url,
            exchange_params,
            oauth.client_secret.as_deref(),
        )
        .await
    {
        Ok(ts) => ts,
        Err(OAuthError::ServerError { .. }) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    // Extract subject from JWT payload (decode middle segment, no sig verify).
    // Signature verification is apps/api's responsibility (ADR-023).
    let subject =
        extract_jwt_subject(&token_set.access_token).unwrap_or_else(|| "unknown".to_string());

    let csrf = CsrfToken::generate();
    let session = StoredSession::new(subject, token_set, csrf.clone());

    let session_cfg = &app_state.gateway.session;
    let session_id = match app_state
        .session_store
        .create(session, session_cfg.absolute_ttl_seconds)
        .await
    {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Some(return_uri) = return_uri {
        return mobile_callback_redirect(&app_state.handoff_store, session_id, &return_uri);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        build_session_cookie(&session_id, &session_cfg.cookie_name),
    );
    headers.append(
        header::SET_COOKIE,
        build_csrf_cookie(&csrf, &format!("{}_csrf", session_cfg.cookie_name)),
    );

    (StatusCode::OK, headers).into_response()
}

fn is_registered_mobile_return_uri(allowlist: &[String], candidate: &str) -> bool {
    allowlist.iter().any(|uri| uri == candidate) && Url::parse(candidate).is_ok()
}

fn mobile_callback_redirect(
    handoff_store: &Arc<HandoffStore>,
    session_id: crate::session::SessionId,
    return_uri: &str,
) -> Response {
    let handoff_code = handoff_store.issue(session_id);
    let mut url = match Url::parse(return_uri) {
        Ok(url) => url,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    url.query_pairs_mut()
        .append_pair("handoff_code", &handoff_code);

    Redirect::to(url.as_ref()).into_response()
}

// ── JWT subject extraction ─────────────────────────────────────────────────────

/// Decode the `sub` claim from the JWT payload without verifying the signature.
/// Returns `None` if the token is malformed or the claim is absent.
fn extract_jwt_subject(token: &str) -> Option<String> {
    let payload_b64 = token.splitn(3, '.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(payload_b64))
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value.get("sub")?.as_str().map(str::to_string)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;
    use url::Url;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, method, path},
    };

    use crate::{
        auth::{
            handoff::HandoffStore,
            login::{callback_handler, login_handler},
            oauth_client::PkceVerifier,
            pending::{PendingAuth, PendingAuthStore},
        },
        session::store::InMemorySessionStore,
        state::GatewayState,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_gateway_settings(token_url: &str) -> dubbridge_config::GatewaySettings {
        dubbridge_config::GatewaySettings {
            port: 8081,
            upstream_api_base_url: "http://localhost:8080".to_string(),
            mobile_return_uris: vec![
                "dubbridge://auth/callback".to_string(),
                "https://mobile.local.dubbridge.example/auth/callback".to_string(),
            ],
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

    fn make_state(token_url: &str) -> Arc<GatewayState> {
        let gw = make_gateway_settings(token_url);
        let cfg = make_app_config(gw.clone());
        Arc::new(GatewayState::new(
            reqwest::Client::new(),
            cfg,
            gw,
            Arc::new(InMemorySessionStore::new()),
            Arc::new(PendingAuthStore::with_default_ttl()),
            Arc::new(HandoffStore::with_default_ttl()),
        ))
    }

    fn build_login_router(token_url: &str) -> Router {
        Router::new()
            .route("/auth/login", get(login_handler))
            .with_state(make_state(token_url))
    }

    fn build_callback_router(state: Arc<GatewayState>) -> Router {
        Router::new()
            .route("/auth/callback", get(callback_handler))
            .with_state(state)
    }

    // ── /auth/login tests ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn login_returns_redirect() {
        let app = build_login_router("http://localhost:9000/oauth/token");
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    }

    #[tokio::test]
    async fn login_with_unregistered_return_uri_returns_400() {
        let app = build_login_router("http://localhost:9000/oauth/token");
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/login?return_uri=https%3A%2F%2Fevil.example%2Fcallback")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn login_redirect_location_contains_required_params() {
        let app = build_login_router("http://localhost:9000/oauth/token");
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let location = resp
            .headers()
            .get("location")
            .expect("login must set Location header")
            .to_str()
            .unwrap();

        assert!(
            location.contains("response_type=code"),
            "missing response_type"
        );
        assert!(
            location.contains("code_challenge="),
            "missing code_challenge"
        );
        assert!(
            location.contains("code_challenge_method=S256"),
            "missing S256"
        );
        assert!(location.contains("state="), "missing state param");
        assert!(
            location.contains("client_id=test-client"),
            "missing client_id"
        );
    }

    // ── /auth/callback tests ──────────────────────────────────────────────────

    #[tokio::test]
    async fn callback_with_unknown_state_returns_400() {
        let app = build_callback_router(make_state("http://localhost:9000/oauth/token"));
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/auth/callback?code=somecode&state=never-inserted")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn callback_with_valid_code_and_state_returns_200_with_hardened_session_cookie() {
        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=authorization_code"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEifQ.sig",
                "refresh_token": "refresh-xyz",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url);

        // Pre-insert pending entry so callback can consume it
        let test_state = "test-state-valid-callback";
        state
            .pending_store
            .insert_verifier_only(test_state, PkceVerifier::generate());

        let resp = build_callback_router(state)
            .oneshot(
                Request::builder()
                    .uri(format!("/auth/callback?code=auth-code&state={test_state}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let cookies: Vec<_> = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();

        let session_cookie = cookies
            .iter()
            .find(|c| c.contains("dubbridge_session="))
            .expect("session cookie must be set after successful callback");

        assert!(session_cookie.contains("HttpOnly"), "must be HttpOnly");
        assert!(session_cookie.contains("Secure"), "must be Secure");
        assert!(
            session_cookie.contains("SameSite=Lax"),
            "must be SameSite=Lax"
        );
    }

    #[tokio::test]
    async fn callback_access_token_never_appears_in_response_headers() {
        // ADR-018 / ADR-024: access token must never reach the client
        let access_token = "super-secret-access-token-XYZ-NEVER-LEAK";

        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": access_token,
                "token_type": "Bearer",
                "expires_in": 3600
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url);
        let test_state = "state-no-token-leak";
        state
            .pending_store
            .insert_verifier_only(test_state, PkceVerifier::generate());

        let resp = build_callback_router(state)
            .oneshot(
                Request::builder()
                    .uri(format!("/auth/callback?code=code&state={test_state}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        for (name, value) in resp.headers() {
            assert!(
                !value.to_str().unwrap_or("").contains(access_token),
                "header '{}' must not contain the access token (ADR-018/ADR-024)",
                name
            );
        }
    }

    #[tokio::test]
    async fn callback_state_is_single_use() {
        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEifQ.sig",
                "token_type": "Bearer",
                "expires_in": 3600
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url);
        state
            .pending_store
            .insert_verifier_only("reuse-state", PkceVerifier::generate());

        let app = build_callback_router(state);

        let r1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/callback?code=code&state=reuse-state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r1.status(), StatusCode::OK, "first call must succeed");

        let r2 = app
            .oneshot(
                Request::builder()
                    .uri("/auth/callback?code=code&state=reuse-state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            r2.status(),
            StatusCode::BAD_REQUEST,
            "second call must be rejected (single-use)"
        );
    }

    #[tokio::test]
    async fn callback_with_invalid_grant_from_as_returns_401() {
        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "authorization code already used"
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url);
        state
            .pending_store
            .insert_verifier_only("state-invalid-grant", PkceVerifier::generate());

        let resp = build_callback_router(state)
            .oneshot(
                Request::builder()
                    .uri("/auth/callback?code=bad-code&state=state-invalid-grant")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn callback_with_mobile_return_uri_redirects_with_handoff_code_only() {
        let mock_as = MockServer::start().await;
        let access_token = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJtb2JpbGUtdXNlciJ9.sig";
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": access_token,
                "refresh_token": "refresh-mobile",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url);
        state.pending_store.insert(
            "mobile-state",
            PendingAuth::new(
                PkceVerifier::generate(),
                Some("dubbridge://auth/callback".to_string()),
            ),
        );

        let resp = build_callback_router(state)
            .oneshot(
                Request::builder()
                    .uri("/auth/callback?code=mobile-code&state=mobile-state")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        assert!(
            resp.headers().get_all("set-cookie").iter().next().is_none(),
            "mobile callback must not set browser cookies"
        );

        let location = resp
            .headers()
            .get("location")
            .expect("mobile callback must set Location")
            .to_str()
            .unwrap();
        let redirect = Url::parse(location).expect("redirect location must be a valid URI");
        let pairs: Vec<_> = redirect.query_pairs().collect();
        assert_eq!(
            pairs.len(),
            1,
            "mobile callback must return only handoff_code"
        );
        assert_eq!(pairs[0].0, "handoff_code");
        assert_eq!(
            pairs[0].1.len(),
            43,
            "handoff code must be opaque 32-byte base64url"
        );
        assert!(
            !location.contains(access_token),
            "redirect URI must not contain access token"
        );
        assert!(
            !location.contains("refresh-mobile"),
            "redirect URI must not contain refresh token"
        );
        assert!(
            !location.contains("dubbridge_session"),
            "redirect URI must not contain session cookie name or session id"
        );
    }

    #[tokio::test]
    async fn callback_with_unregistered_mobile_return_uri_fails_closed() {
        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEifQ.sig",
                "refresh_token": "refresh-xyz",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url);
        state.pending_store.insert(
            "mobile-state-bad-uri",
            PendingAuth::new(
                PkceVerifier::generate(),
                Some("https://evil.example/callback".to_string()),
            ),
        );

        let resp = build_callback_router(state)
            .oneshot(
                Request::builder()
                    .uri("/auth/callback?code=mobile-code&state=mobile-state-bad-uri")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert!(
            resp.headers().get("location").is_none(),
            "fail-closed callback must not redirect to attacker URI"
        );
        assert!(
            resp.headers().get_all("set-cookie").iter().next().is_none(),
            "fail-closed callback must not expose session cookies"
        );
    }
}
