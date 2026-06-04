// P1-T4: POST /auth/logout handler (ADR-024)
//
// Invalidates the server-side session and clears both cookies.
// Idempotent: if no session cookie is present, returns 200 noop (no 4xx).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{
    cookie::{clear_csrf_cookie, clear_session_cookie},
    cookie_ext::{SessionTransportError, extract_session_transport},
    state::GatewayState,
};

// ── Handler ───────────────────────────────────────────────────────────────────

/// POST /auth/logout
/// Extracts session id from cookie, deletes the session, clears both cookies.
/// Idempotent: absent or unrecognized session cookie → 200 with cleared cookies.
pub async fn logout_handler(
    State(app_state): State<Arc<GatewayState>>,
    headers: HeaderMap,
) -> Response {
    let session_cfg = &app_state.gateway.session;
    let transport = match extract_session_transport(&headers, &session_cfg.cookie_name) {
        Ok(transport) => transport,
        Err(SessionTransportError::Conflict) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let mobile_path = transport
        .as_ref()
        .map(|transport| transport.uses_mobile_header())
        .unwrap_or(false);

    if let Some(transport) = transport {
        // Best-effort delete — session may already be expired or gone; never 500 on delete failure
        let _ = app_state.session_store.delete(transport.session_id()).await;
    }

    if mobile_path {
        return StatusCode::OK.into_response();
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        header::SET_COOKIE,
        clear_session_cookie(&session_cfg.cookie_name),
    );
    resp_headers.append(
        header::SET_COOKIE,
        clear_csrf_cookie(&format!("{}_csrf", session_cfg.cookie_name)),
    );

    (StatusCode::OK, resp_headers).into_response()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::post,
    };
    use tower::ServiceExt;

    use crate::{
        auth::{
            handoff::HandoffStore, logout::logout_handler, oauth_client::TokenSet,
            pending::PendingAuthStore,
        },
        cookie_ext::MOBILE_SESSION_HEADER,
        session::{CsrfToken, StoredSession, store::InMemorySessionStore},
        state::GatewayState,
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

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

    fn build_logout_router(state: Arc<GatewayState>) -> Router {
        Router::new()
            .route("/auth/logout", post(logout_handler))
            .with_state(state)
    }

    fn sample_token_set() -> TokenSet {
        TokenSet {
            access_token: "eyJhbGciOiJSUzI1NiJ9.secret".to_string(),
            refresh_token: Some("refresh-xyz".to_string()),
            expires_in: Some(3600),
            token_type: "Bearer".to_string(),
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn logout_without_session_cookie_returns_200_noop() {
        // Idempotent: no session cookie → 200 (no session to invalidate)
        let app = build_logout_router(make_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "logout without cookie must be 200 noop"
        );
    }

    #[tokio::test]
    async fn logout_without_session_cookie_still_clears_cookies_in_response() {
        // Even on noop, the response should clear cookies to handle stale client state
        let app = build_logout_router(make_state());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies: Vec<_> = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();

        let has_cleared_session = cookies
            .iter()
            .any(|c| c.contains("dubbridge_session=") && c.contains("Max-Age=0"));
        assert!(
            has_cleared_session,
            "logout must clear session cookie (Max-Age=0)"
        );
    }

    #[tokio::test]
    async fn logout_with_valid_session_returns_200_and_deletes_session() {
        let state = make_state();

        // Create a session to be deleted by logout
        let session = StoredSession::new("user-alice", sample_token_set(), CsrfToken::generate());
        let session_id = state.session_store.create(session, 28_800).await.unwrap();

        let app = build_logout_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", session_id.as_str()),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        // Session must be gone from the store
        let resolved = state
            .session_store
            .resolve(&session_id, 28_800, 1_800)
            .await
            .unwrap();
        assert!(resolved.is_none(), "session must be deleted after logout");
    }

    #[tokio::test]
    async fn logout_response_clears_both_session_and_csrf_cookies() {
        let state = make_state();
        let session = StoredSession::new("user-bob", sample_token_set(), CsrfToken::generate());
        let session_id = state.session_store.create(session, 28_800).await.unwrap();

        let app = build_logout_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", session_id.as_str()),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies: Vec<_> = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();

        let has_cleared_session = cookies
            .iter()
            .any(|c| c.contains("dubbridge_session=") && c.contains("Max-Age=0"));
        let has_cleared_csrf = cookies
            .iter()
            .any(|c| c.contains("dubbridge_session_csrf=") && c.contains("Max-Age=0"));

        assert!(
            has_cleared_session,
            "session cookie must be cleared (Max-Age=0)"
        );
        assert!(has_cleared_csrf, "CSRF cookie must be cleared (Max-Age=0)");
    }

    #[tokio::test]
    async fn logout_is_idempotent_on_already_deleted_session() {
        let state = make_state();
        let session = StoredSession::new("user-carol", sample_token_set(), CsrfToken::generate());
        let session_id = state.session_store.create(session, 28_800).await.unwrap();

        // First logout
        state.session_store.delete(&session_id).await.unwrap();

        // Second logout with the same (now-deleted) session id — must still be 200
        let app = build_logout_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .header(
                        "cookie",
                        format!("dubbridge_session={}", session_id.as_str()),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "logout on already-deleted session must be idempotent (200)"
        );
    }

    #[tokio::test]
    async fn logout_with_mobile_header_returns_200_and_no_set_cookie_headers() {
        let state = make_state();
        let session = StoredSession::new("user-mobile", sample_token_set(), CsrfToken::generate());
        let session_id = state.session_store.create(session, 28_800).await.unwrap();

        let app = build_logout_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .header(MOBILE_SESSION_HEADER, session_id.as_str())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            resp.headers().get_all("set-cookie").iter().next().is_none(),
            "mobile logout must not emit browser cookie clears"
        );

        let resolved = state
            .session_store
            .resolve(&session_id, 28_800, 1_800)
            .await
            .unwrap();
        assert!(resolved.is_none(), "mobile logout must delete the session");
    }

    #[tokio::test]
    async fn logout_with_mismatched_cookie_and_mobile_header_returns_401() {
        let state = make_state();
        let session = StoredSession::new("user-mobile", sample_token_set(), CsrfToken::generate());
        let cookie_session_id = state.session_store.create(session, 28_800).await.unwrap();
        let different_session_id = crate::session::SessionId::generate();

        let app = build_logout_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
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

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
