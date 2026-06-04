// P1-T5.2: token expiry check + transparent refresh logic
// P1-T5.3: HTTP proxy handler + route mount (added in T5.3)
//
// ensure_fresh_token: decides whether the stored access token needs refreshing,
// executes the refresh grant if so, creates a new session, and returns the fresh
// token + new SessionId. Kept separate from the HTTP handler so it is testable
// in isolation via wiremock without building a full HTTP request.

use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{OriginalUri, State},
    http::{HeaderMap, Method, StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::any,
};

use crate::{
    auth::oauth_client::{OAuthError, OAuthExecutor, build_token_refresh_params},
    cookie::{build_session_cookie, clear_csrf_cookie, clear_session_cookie},
    cookie_ext::{MOBILE_SESSION_HEADER, SessionTransportError, resolve_session},
    session::{SessionId, StoredSession},
    state::GatewayState,
};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Refresh the token this many seconds before it actually expires.
/// Guards against clock skew and network latency between gateway and apps/api.
const REFRESH_WINDOW_SECS: u64 = 60;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error("no refresh token available")]
    NoRefreshToken,
    #[error("authorization server rejected refresh: {0}")]
    RefreshFailed(#[from] OAuthError),
    #[error("session store error during refresh")]
    StoreFailed,
}

// ── Token expiry helpers ──────────────────────────────────────────────────────

/// Compute absolute unix-second expiry from session creation time + expires_in.
/// Returns None if expires_in is absent (treat as non-expired — let apps/api decide).
fn token_expires_at(session: &StoredSession) -> Option<u64> {
    session
        .token_set
        .expires_in
        .map(|e| session.created_at_unix_secs + e)
}

/// True if the access token has expired or will expire within REFRESH_WINDOW_SECS.
pub(crate) fn needs_refresh(session: &StoredSession) -> bool {
    match token_expires_at(session) {
        None => false,
        Some(expires_at) => unix_now() + REFRESH_WINDOW_SECS >= expires_at,
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── ensure_fresh_token ────────────────────────────────────────────────────────

/// Returns a valid access token and its associated SessionId.
/// If the stored token is still valid, returns it as-is (no network call).
/// If expired (or within the refresh window), performs exactly one refresh grant:
///   - on success: creates a new session, deletes the old one, returns new token + id
///   - on AS error: deletes the old session, returns RefreshFailed
///   - no refresh token: returns NoRefreshToken (session left intact for caller)
pub(crate) async fn ensure_fresh_token(
    state: &Arc<GatewayState>,
    old_id: SessionId,
    session: StoredSession,
) -> Result<(String, SessionId), RefreshError> {
    if !needs_refresh(&session) {
        return Ok((session.token_set.access_token.clone(), old_id));
    }

    let refresh_token = session
        .token_set
        .refresh_token
        .clone()
        .ok_or(RefreshError::NoRefreshToken)?;

    let oauth = &state.gateway.oauth;
    let params = build_token_refresh_params(&oauth.client_id, &refresh_token);
    let executor = OAuthExecutor::new(&state.http_client);

    let new_tokens = executor
        .send_token_request(&oauth.token_url, params, oauth.client_secret.as_deref())
        .await
        .map_err(|e| {
            // Best-effort delete on refresh failure — session is no longer valid
            // (fire-and-forget; we cannot await here without async block)
            let _ = e; // error mapped below after store delete attempt
            e
        });

    // If refresh failed, delete old session before propagating the error
    if let Err(ref e) = new_tokens {
        let _ = state.session_store.delete(&old_id).await;
        return Err(RefreshError::RefreshFailed(match e {
            OAuthError::ServerError {
                error,
                error_description,
            } => OAuthError::ServerError {
                error: error.clone(),
                error_description: error_description.clone(),
            },
            OAuthError::InvalidResponse(msg) => OAuthError::InvalidResponse(msg.clone()),
            OAuthError::Http(_) => {
                // reqwest::Error is not Clone — reconstruct as InvalidResponse
                OAuthError::InvalidResponse("http error during refresh".into())
            }
            OAuthError::UrlParse(msg) => OAuthError::UrlParse(msg.clone()),
        }));
    }

    let new_tokens = new_tokens.unwrap();
    let session_cfg = &state.gateway.session;

    // Reuse existing csrf_token — client already holds it in the CSRF cookie.
    // Generating a new one here would break the double-submit invariant until
    // the client re-reads the cookie.
    let new_session = StoredSession::new(
        session.subject.clone(),
        new_tokens.clone(),
        session.csrf_token.clone(),
    );

    let new_id = state
        .session_store
        .create(new_session, session_cfg.absolute_ttl_seconds)
        .await
        .map_err(|_| RefreshError::StoreFailed)?;

    // Best-effort delete of old session — if it fails the old entry will
    // expire naturally; it won't be resolvable because the new id replaces it.
    let _ = state.session_store.delete(&old_id).await;

    Ok((new_tokens.access_token, new_id))
}

// ── HTTP proxy (T5.3) ────────────────────────────────────────────────────────

/// Builds the `/api/*` sub-router. State is inherited from the parent router.
pub fn proxy_router() -> Router<Arc<GatewayState>> {
    Router::new()
        .route("/", any(proxy_handler))
        .route("/{*path}", any(proxy_handler))
}

/// Authenticated catch-all proxy into `apps/api`.
pub async fn proxy_handler(
    State(app_state): State<Arc<GatewayState>>,
    method: Method,
    headers: HeaderMap,
    OriginalUri(original_uri): OriginalUri,
    body: Body,
) -> Response {
    let session_cfg = &app_state.gateway.session;
    let csrf_cookie_name = format!("{}_csrf", session_cfg.cookie_name);

    let (session_id, session, transport) = match resolve_session(&app_state, &headers).await {
        Ok(Some(resolved)) => resolved,
        Ok(None) | Err(SessionTransportError::Conflict) => {
            return unauthorized_response(&session_cfg.cookie_name, &csrf_cookie_name);
        }
    };

    let (access_token, effective_session_id, session_refreshed) =
        match ensure_fresh_token(&app_state, session_id.clone(), session).await {
            Ok((access_token, effective_session_id)) => {
                let refreshed = effective_session_id != session_id;
                (access_token, effective_session_id, refreshed)
            }
            Err(RefreshError::StoreFailed) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            Err(RefreshError::NoRefreshToken) => {
                let _ = app_state.session_store.delete(&session_id).await;
                return unauthorized_response(&session_cfg.cookie_name, &csrf_cookie_name);
            }
            Err(RefreshError::RefreshFailed(_)) => {
                return unauthorized_response(&session_cfg.cookie_name, &csrf_cookie_name);
            }
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
        if name.as_str().eq_ignore_ascii_case(MOBILE_SESSION_HEADER) {
            continue;
        }
        upstream_request = upstream_request.header(name, value);
    }

    let upstream_response = match upstream_request.send().await {
        Ok(response) => response,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };

    relay_upstream_response(
        upstream_response,
        session_refreshed.then_some(effective_session_id),
        &session_cfg.cookie_name,
        transport.uses_mobile_header(),
    )
    .await
}

fn unauthorized_response(session_cookie_name: &str, csrf_cookie_name: &str) -> Response {
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::SET_COOKIE,
        clear_session_cookie(session_cookie_name),
    );
    response_headers.append(header::SET_COOKIE, clear_csrf_cookie(csrf_cookie_name));

    (StatusCode::UNAUTHORIZED, response_headers).into_response()
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

async fn relay_upstream_response(
    upstream_response: reqwest::Response,
    refreshed_session_id: Option<SessionId>,
    session_cookie_name: &str,
    mobile_transport: bool,
) -> Response {
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

    if let Some(session_id) = refreshed_session_id {
        if mobile_transport {
            if let Ok(header_value) = header::HeaderValue::from_str(session_id.as_str()) {
                response_headers.insert(MOBILE_SESSION_HEADER, header_value);
            } else {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        } else {
            response_headers.append(
                header::SET_COOKIE,
                build_session_cookie(&session_id, session_cookie_name),
            );
        }
    }

    (status, response_headers, body).into_response()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, method, path},
    };

    use crate::{
        auth::{handoff::HandoffStore, oauth_client::TokenSet, pending::PendingAuthStore},
        build_app,
        cookie_ext::MOBILE_SESSION_HEADER,
        proxy::{ensure_fresh_token, needs_refresh},
        session::{CsrfToken, StoredSession, store::InMemorySessionStore},
        state::GatewayState,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_state(token_url: &str, upstream_api_base_url: &str) -> Arc<GatewayState> {
        let gw = dubbridge_config::GatewaySettings {
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

    fn session_expiring_in(secs_from_now: i64, with_refresh: bool) -> StoredSession {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let created_at = if secs_from_now >= 0 {
            now
        } else {
            now.saturating_sub((-secs_from_now) as u64 + 120)
        };

        let expires_in = if secs_from_now >= 0 {
            secs_from_now as u64
        } else {
            60u64.saturating_sub((-secs_from_now) as u64)
        };

        let mut s = StoredSession::new(
            "user-1",
            TokenSet {
                access_token: "original-access-token".to_string(),
                refresh_token: if with_refresh {
                    Some("refresh-tok".to_string())
                } else {
                    None
                },
                expires_in: Some(expires_in),
                token_type: "Bearer".to_string(),
            },
            CsrfToken::generate(),
        );
        s.created_at_unix_secs = created_at;
        s
    }

    fn session_with_no_expiry() -> StoredSession {
        StoredSession::new(
            "user-1",
            TokenSet {
                access_token: "no-expiry-token".to_string(),
                refresh_token: Some("refresh-tok".to_string()),
                expires_in: None,
                token_type: "Bearer".to_string(),
            },
            CsrfToken::generate(),
        )
    }

    async fn read_body_text(body: Body) -> String {
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    async fn create_session(
        state: &Arc<GatewayState>,
        session: StoredSession,
    ) -> crate::session::SessionId {
        state
            .session_store
            .create(session, state.gateway.session.absolute_ttl_seconds)
            .await
            .unwrap()
    }

    // ── needs_refresh unit tests ───────────────────────────────────────────────

    #[test]
    fn needs_refresh_false_when_expires_in_absent() {
        let session = session_with_no_expiry();
        assert!(
            !needs_refresh(&session),
            "absent expires_in must not trigger refresh"
        );
    }

    #[test]
    fn needs_refresh_false_when_token_has_plenty_of_life() {
        // expires in 3600 s from now — well outside the 60 s window
        let session = session_expiring_in(3600, true);
        assert!(!needs_refresh(&session));
    }

    #[test]
    fn needs_refresh_true_when_token_is_expired() {
        // created 120 s ago with expires_in=60 → expired 60 s ago
        let session = session_expiring_in(-60, true);
        assert!(
            needs_refresh(&session),
            "expired token must trigger refresh"
        );
    }

    #[test]
    fn needs_refresh_true_when_within_refresh_window() {
        // expires in 30 s — inside the 60 s window
        let mut session = StoredSession::new(
            "user-1",
            TokenSet {
                access_token: "tok".to_string(),
                refresh_token: Some("ref".to_string()),
                expires_in: Some(90),
                token_type: "Bearer".to_string(),
            },
            CsrfToken::generate(),
        );
        // Shift created_at so only 30 s remain
        session.created_at_unix_secs = session.created_at_unix_secs.saturating_sub(60);
        assert!(
            needs_refresh(&session),
            "token within refresh window must trigger refresh"
        );
    }

    // ── ensure_fresh_token tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn ensure_fresh_token_returns_existing_token_when_not_expired() {
        // No mock registered — any outbound request would panic the test
        let state = make_state("http://localhost:9999/oauth/token", "http://localhost:8080");
        let session = session_expiring_in(3600, true);
        let id = state
            .session_store
            .create(session.clone(), 28_800)
            .await
            .unwrap();

        let (token, returned_id) = ensure_fresh_token(&state, id.clone(), session)
            .await
            .unwrap();

        assert_eq!(token, "original-access-token");
        assert_eq!(
            returned_id.as_str(),
            id.as_str(),
            "id must be unchanged when no refresh"
        );
    }

    #[tokio::test]
    async fn ensure_fresh_token_returns_no_refresh_token_err_when_refresh_absent() {
        let state = make_state("http://localhost:9999/oauth/token", "http://localhost:8080");
        let session = session_expiring_in(-60, false); // expired, no refresh token
        let id = state
            .session_store
            .create(session.clone(), 28_800)
            .await
            .unwrap();

        let result = ensure_fresh_token(&state, id, session).await;
        assert!(
            matches!(result, Err(crate::proxy::RefreshError::NoRefreshToken)),
            "absent refresh token must return NoRefreshToken"
        );
    }

    #[tokio::test]
    async fn ensure_fresh_token_refreshes_and_returns_new_token_and_id() {
        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "new-access-token",
                "refresh_token": "new-refresh-token",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .expect(1) // exactly one refresh call
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url, "http://localhost:8080");
        let session = session_expiring_in(-60, true); // expired, has refresh token
        let old_id = state
            .session_store
            .create(session.clone(), 28_800)
            .await
            .unwrap();

        let (new_token, new_id) = ensure_fresh_token(&state, old_id.clone(), session)
            .await
            .unwrap();

        assert_eq!(new_token, "new-access-token");
        assert_ne!(
            new_id.as_str(),
            old_id.as_str(),
            "new session id must differ from old"
        );

        // Old session must be deleted
        let old_resolved = state
            .session_store
            .resolve(&old_id, 28_800, 1_800)
            .await
            .unwrap();
        assert!(
            old_resolved.is_none(),
            "old session must be deleted after refresh"
        );

        // New session must exist and carry the new token
        let new_resolved = state
            .session_store
            .resolve(&new_id, 28_800, 1_800)
            .await
            .unwrap()
            .expect("new session must exist in store");
        assert_eq!(new_resolved.token_set.access_token, "new-access-token");
    }

    #[tokio::test]
    async fn ensure_fresh_token_deletes_old_session_on_refresh_failure() {
        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "refresh token expired"
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url, "http://localhost:8080");
        let session = session_expiring_in(-60, true);
        let old_id = state
            .session_store
            .create(session.clone(), 28_800)
            .await
            .unwrap();

        let result = ensure_fresh_token(&state, old_id.clone(), session).await;
        assert!(
            matches!(result, Err(crate::proxy::RefreshError::RefreshFailed(_))),
            "AS invalid_grant must return RefreshFailed"
        );

        // Old session must be deleted even on failure
        let old_resolved = state
            .session_store
            .resolve(&old_id, 28_800, 1_800)
            .await
            .unwrap();
        assert!(
            old_resolved.is_none(),
            "old session must be deleted even when refresh fails"
        );
    }

    #[tokio::test]
    async fn ensure_fresh_token_preserves_csrf_token_across_refresh() {
        let mock_as = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "new-token",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .mount(&mock_as)
            .await;

        let token_url = format!("{}/oauth/token", mock_as.uri());
        let state = make_state(&token_url, "http://localhost:8080");
        let session = session_expiring_in(-60, true);
        let original_csrf = session.csrf_token.as_str().to_string();
        let old_id = state
            .session_store
            .create(session.clone(), 28_800)
            .await
            .unwrap();

        let (_, new_id) = ensure_fresh_token(&state, old_id, session).await.unwrap();

        let new_session = state
            .session_store
            .resolve(&new_id, 28_800, 1_800)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            new_session.csrf_token.as_str(),
            original_csrf,
            "csrf_token must be preserved across refresh (client still holds the old cookie)"
        );
    }

    // ── proxy handler tests ───────────────────────────────────────────────────

    #[tokio::test]
    async fn proxy_without_session_returns_401_clears_cookies_and_skips_upstream() {
        let upstream = MockServer::start().await;
        let state = make_state("http://localhost:9999/oauth/token", &upstream.uri());
        let app = build_app(state);

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

        let cookies: Vec<_> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|value| value.to_str().unwrap().to_string())
            .collect();
        assert!(cookies.iter().any(|cookie| {
            cookie.contains("dubbridge_session=") && cookie.contains("Max-Age=0")
        }));
        assert!(cookies.iter().any(|cookie| {
            cookie.contains("dubbridge_session_csrf=") && cookie.contains("Max-Age=0")
        }));
        assert_eq!(upstream.received_requests().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn proxy_with_unknown_session_returns_401_and_skips_upstream() {
        let upstream = MockServer::start().await;
        let state = make_state("http://localhost:9999/oauth/token", &upstream.uri());
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/tracks")
                    .header("cookie", "dubbridge_session=missing-session-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(upstream.received_requests().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn proxy_with_mobile_session_header_resolves_same_server_side_session() {
        let auth_server = MockServer::start().await;
        let upstream = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/assets"))
            .respond_with(ResponseTemplate::new(200).set_body_string("via-header"))
            .expect(1)
            .mount(&upstream)
            .await;

        let state = make_state(
            &format!("{}/oauth/token", auth_server.uri()),
            &upstream.uri(),
        );
        let session_id = create_session(&state, session_expiring_in(3600, true)).await;
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets")
                    .header(MOBILE_SESSION_HEADER, session_id.as_str())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = read_body_text(response.into_body()).await;
        assert_eq!(body, "via-header");
    }

    #[tokio::test]
    async fn proxy_with_mismatched_cookie_and_mobile_header_returns_401_and_skips_upstream() {
        let auth_server = MockServer::start().await;
        let upstream = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/assets"))
            .respond_with(ResponseTemplate::new(200).set_body_string("must-not-run"))
            .expect(0)
            .mount(&upstream)
            .await;

        let state = make_state(
            &format!("{}/oauth/token", auth_server.uri()),
            &upstream.uri(),
        );
        let cookie_session_id = create_session(&state, session_expiring_in(3600, true)).await;
        let different_session_id = crate::session::SessionId::generate();
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", cookie_session_id.as_str()),
                    )
                    .header(MOBILE_SESSION_HEADER, different_session_id.as_str())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn proxy_with_jwt_like_mobile_header_returns_401_and_never_forwards_as_bearer() {
        let auth_server = MockServer::start().await;
        let upstream = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/assets"))
            .respond_with(ResponseTemplate::new(200).set_body_string("must-not-run"))
            .expect(0)
            .mount(&upstream)
            .await;

        let app = build_app(make_state(
            &format!("{}/oauth/token", auth_server.uri()),
            &upstream.uri(),
        ));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets")
                    .header(MOBILE_SESSION_HEADER, "eyJhbGciOiJIUzI1NiJ9.payload.sig")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn proxy_refresh_with_mobile_header_returns_rotated_session_ref_header_and_no_cookie() {
        let auth_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-access-token",
                "refresh_token": "fresh-refresh-token",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .expect(1)
            .mount(&auth_server)
            .await;

        let upstream = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/assets"))
            .respond_with(ResponseTemplate::new(200).set_body_string("mobile-refresh-ok"))
            .expect(1)
            .mount(&upstream)
            .await;

        let state = make_state(
            &format!("{}/oauth/token", auth_server.uri()),
            &upstream.uri(),
        );
        let session_id = create_session(&state, session_expiring_in(-60, true)).await;
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/assets")
                    .header(MOBILE_SESSION_HEADER, session_id.as_str())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let rotated_session_ref = response
            .headers()
            .get(MOBILE_SESSION_HEADER)
            .expect("mobile refresh must return rotated session ref header")
            .to_str()
            .unwrap()
            .to_string();
        assert_ne!(rotated_session_ref, session_id.as_str());
        assert!(
            response
                .headers()
                .get_all("set-cookie")
                .iter()
                .next()
                .is_none(),
            "mobile refresh path must not set browser cookies"
        );

        let body = read_body_text(response.into_body()).await;
        assert_eq!(body, "mobile-refresh-ok");
    }

    #[tokio::test]
    async fn proxy_forwards_request_with_live_token_and_strips_sensitive_headers() {
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

        let state = make_state("http://localhost:9999/oauth/token", &upstream.uri());
        let session_id = create_session(&state, session_expiring_in(3600, true)).await;
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/tracks/upload?part=7")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", session_id.as_str()),
                    )
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
            "Bearer original-access-token"
        );
        assert!(request.headers.get("cookie").is_none());
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
    async fn proxy_refreshes_expired_token_sets_new_session_cookie_and_relays_upstream() {
        let upstream = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tracks/secure"))
            .respond_with(ResponseTemplate::new(200).set_body_string("after-refresh"))
            .mount(&upstream)
            .await;

        let auth_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-access-token",
                "refresh_token": "fresh-refresh-token",
                "expires_in": 3600,
                "token_type": "Bearer"
            })))
            .expect(1)
            .mount(&auth_server)
            .await;

        let token_url = format!("{}/oauth/token", auth_server.uri());
        let state = make_state(&token_url, &upstream.uri());
        let old_session_id = create_session(&state, session_expiring_in(-60, true)).await;
        let app = build_app(Arc::clone(&state));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/tracks/secure")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", old_session_id.as_str()),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let cookies: Vec<_> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|value| value.to_str().unwrap().to_string())
            .collect();
        let session_cookie = cookies
            .iter()
            .find(|cookie| cookie.contains("dubbridge_session=") && !cookie.contains("Max-Age=0"))
            .expect("refresh path must set a new session cookie");
        assert!(!session_cookie.contains("fresh-access-token"));
        assert_eq!(read_body_text(response.into_body()).await, "after-refresh");

        let upstream_requests = upstream.received_requests().await.unwrap();
        assert_eq!(upstream_requests.len(), 1);
        assert_eq!(
            upstream_requests[0]
                .headers
                .get("authorization")
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer fresh-access-token"
        );

        let old_session = state
            .session_store
            .resolve(&old_session_id, 28_800, 1_800)
            .await
            .unwrap();
        assert!(
            old_session.is_none(),
            "old session must be gone after refresh"
        );
    }

    #[tokio::test]
    async fn proxy_refresh_failure_returns_401_clears_cookies_and_skips_upstream() {
        let upstream = MockServer::start().await;

        let auth_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
                "error_description": "refresh token expired"
            })))
            .mount(&auth_server)
            .await;

        let token_url = format!("{}/oauth/token", auth_server.uri());
        let state = make_state(&token_url, &upstream.uri());
        let session_id = create_session(&state, session_expiring_in(-60, true)).await;
        let app = build_app(Arc::clone(&state));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/tracks")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", session_id.as_str()),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let cookies: Vec<_> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|value| value.to_str().unwrap().to_string())
            .collect();
        assert!(cookies.iter().any(|cookie| {
            cookie.contains("dubbridge_session=") && cookie.contains("Max-Age=0")
        }));
        assert!(cookies.iter().any(|cookie| {
            cookie.contains("dubbridge_session_csrf=") && cookie.contains("Max-Age=0")
        }));
        assert_eq!(upstream.received_requests().await.unwrap().len(), 0);

        let resolved = state
            .session_store
            .resolve(&session_id, 28_800, 1_800)
            .await
            .unwrap();
        assert!(
            resolved.is_none(),
            "refresh failure must invalidate the stale session"
        );
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

        let state = make_state("http://localhost:9999/oauth/token", &upstream.uri());
        let session_id = create_session(&state, session_expiring_in(3600, true)).await;
        let app = build_app(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/forbidden")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", session_id.as_str()),
                    )
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
}
