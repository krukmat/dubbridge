// P1-T5.1: shared cookie extraction + session resolver
//
// extract_session_id — moved from auth/logout.rs (was private).
// resolve_session    — combines cookie extraction + store lookup in one step.
//                      All callers (logout, proxy) use the same fail-closed path:
//                      missing cookie or unknown/expired id → None.

use std::sync::Arc;

use axum::http::{HeaderMap, header};

use crate::{
    session::{SessionId, StoredSession},
    state::GatewayState,
};

// ── Cookie extraction ─────────────────────────────────────────────────────────
pub const MOBILE_SESSION_HEADER: &str = "x-dubbridge-session";

#[derive(Debug, PartialEq, Eq)]
pub enum SessionTransportError {
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionTransport {
    Cookie(SessionId),
    Header(SessionId),
    Both(SessionId),
}

impl SessionTransport {
    pub fn session_id(&self) -> &SessionId {
        match self {
            SessionTransport::Cookie(id)
            | SessionTransport::Header(id)
            | SessionTransport::Both(id) => id,
        }
    }

    pub fn uses_mobile_header(&self) -> bool {
        matches!(
            self,
            SessionTransport::Header(_) | SessionTransport::Both(_)
        )
    }
}

/// Extract the named session id from the `Cookie` request header.
/// Returns `None` if the header is absent or the named cookie is not present.
pub fn extract_session_id(headers: &HeaderMap, cookie_name: &str) -> Option<SessionId> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix(&format!("{}=", cookie_name)) {
            return Some(SessionId::from_cookie_value(value.trim()));
        }
    }
    None
}

/// Extract the explicit mobile session header transport.
pub fn extract_mobile_session_id(headers: &HeaderMap) -> Option<SessionId> {
    let header_value = headers.get(MOBILE_SESSION_HEADER)?.to_str().ok()?.trim();
    if header_value.is_empty() {
        return None;
    }
    Some(SessionId::from_cookie_value(header_value))
}

pub fn extract_session_transport(
    headers: &HeaderMap,
    cookie_name: &str,
) -> Result<Option<SessionTransport>, SessionTransportError> {
    let cookie_session_id = extract_session_id(headers, cookie_name);
    let header_session_id = extract_mobile_session_id(headers);

    match (cookie_session_id, header_session_id) {
        (Some(cookie_id), Some(header_id)) if cookie_id != header_id => {
            Err(SessionTransportError::Conflict)
        }
        (Some(cookie_id), Some(_)) => Ok(Some(SessionTransport::Both(cookie_id))),
        (Some(cookie_id), None) => Ok(Some(SessionTransport::Cookie(cookie_id))),
        (None, Some(header_id)) => Ok(Some(SessionTransport::Header(header_id))),
        (None, None) => Ok(None),
    }
}

// ── Session resolver ──────────────────────────────────────────────────────────

/// Extract the session id from the request headers and resolve it against the
/// session store. Returns `None` (fail-closed) if the cookie is absent, the id
/// is unknown, or the session has expired. Touches the session on success to
/// reset the idle TTL.
pub async fn resolve_session(
    state: &Arc<GatewayState>,
    headers: &HeaderMap,
) -> Result<Option<(SessionId, StoredSession, SessionTransport)>, SessionTransportError> {
    let session_cfg = &state.gateway.session;
    let Some(transport) = extract_session_transport(headers, &session_cfg.cookie_name)? else {
        return Ok(None);
    };
    let session_id = transport.session_id().clone();

    let session = match state
        .session_store
        .resolve(
            &session_id,
            session_cfg.absolute_ttl_seconds,
            session_cfg.idle_ttl_seconds,
        )
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) | Err(_) => return Ok(None),
    };

    // Touch to reset idle TTL — best-effort, never fails the request
    let _ = state
        .session_store
        .touch(&session_id, session_cfg.idle_ttl_seconds)
        .await;

    Ok(Some((session_id, session, transport)))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::{HeaderMap, HeaderValue, header};

    use crate::{
        auth::{handoff::HandoffStore, oauth_client::TokenSet, pending::PendingAuthStore},
        cookie_ext::{
            MOBILE_SESSION_HEADER, SessionTransport, SessionTransportError,
            extract_mobile_session_id, extract_session_id, extract_session_transport,
            resolve_session,
        },
        session::{CsrfToken, SessionId, StoredSession, store::InMemorySessionStore},
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

    fn headers_with_cookie(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_str(value).unwrap());
        h
    }

    fn headers_with_mobile_session(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(MOBILE_SESSION_HEADER, HeaderValue::from_str(value).unwrap());
        h
    }

    fn sample_session() -> StoredSession {
        StoredSession::new(
            "user-1",
            TokenSet {
                access_token: "access-tok".to_string(),
                refresh_token: Some("refresh-tok".to_string()),
                expires_in: Some(3600),
                token_type: "Bearer".to_string(),
            },
            CsrfToken::generate(),
        )
    }

    // ── extract_session_id ────────────────────────────────────────────────────

    #[test]
    fn extract_returns_session_id_when_cookie_present() {
        let headers = headers_with_cookie("dubbridge_session=ABC123");
        let result = extract_session_id(&headers, "dubbridge_session");
        assert_eq!(result.unwrap().as_str(), "ABC123");
    }

    #[test]
    fn extract_returns_none_when_cookie_absent() {
        let headers = headers_with_cookie("other_cookie=XYZ");
        let result = extract_session_id(&headers, "dubbridge_session");
        assert!(result.is_none());
    }

    #[test]
    fn extract_returns_none_when_no_cookie_header() {
        let headers = HeaderMap::new();
        let result = extract_session_id(&headers, "dubbridge_session");
        assert!(result.is_none());
    }

    #[test]
    fn extract_returns_correct_value_from_multi_cookie_header() {
        let headers = headers_with_cookie("other=AAA; dubbridge_session=SID-XYZ; another=BBB");
        let result = extract_session_id(&headers, "dubbridge_session");
        assert_eq!(result.unwrap().as_str(), "SID-XYZ");
    }

    #[test]
    fn extract_mobile_returns_session_id_when_header_present() {
        let headers = headers_with_mobile_session("MOBILE-SID-1");
        let result = extract_mobile_session_id(&headers);
        assert_eq!(result.unwrap().as_str(), "MOBILE-SID-1");
    }

    #[test]
    fn extract_mobile_returns_none_when_header_absent() {
        let headers = HeaderMap::new();
        let result = extract_mobile_session_id(&headers);
        assert!(result.is_none());
    }

    #[test]
    fn extract_session_transport_returns_header_variant() {
        let headers = headers_with_mobile_session("mobile-only");
        let result = extract_session_transport(&headers, "dubbridge_session").unwrap();

        assert!(matches!(
            result,
            Some(SessionTransport::Header(id)) if id.as_str() == "mobile-only"
        ));
    }

    // ── resolve_session ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn resolve_returns_session_for_live_session() {
        let state = make_state();
        let session = sample_session();
        let id = state
            .session_store
            .create(session.clone(), 28_800)
            .await
            .unwrap();

        let headers = headers_with_cookie(&format!("dubbridge_session={}", id.as_str()));
        let result = resolve_session(&state, &headers).await;

        assert!(result.is_ok());
        let (resolved_id, resolved_session, transport) = result.unwrap().unwrap();
        assert_eq!(resolved_id.as_str(), id.as_str());
        assert_eq!(resolved_session.subject, "user-1");
        assert!(matches!(transport, SessionTransport::Cookie(_)));
    }

    #[tokio::test]
    async fn resolve_returns_none_for_unknown_session_id() {
        let state = make_state();
        let fake_id = SessionId::generate();
        let headers = headers_with_cookie(&format!("dubbridge_session={}", fake_id.as_str()));
        let result = resolve_session(&state, &headers).await;
        assert!(
            result.unwrap().is_none(),
            "unknown session id must return None"
        );
    }

    #[tokio::test]
    async fn resolve_returns_none_when_cookie_absent() {
        let state = make_state();
        let headers = HeaderMap::new();
        let result = resolve_session(&state, &headers).await;
        assert!(
            result.unwrap().is_none(),
            "absent session transport must return None"
        );
    }

    #[tokio::test]
    async fn resolve_returns_none_for_expired_session() {
        let state = make_state();
        let mut session = sample_session();
        // Force session to appear expired (absolute TTL elapsed)
        session.created_at_unix_secs = session.created_at_unix_secs.saturating_sub(9 * 3600);
        let id = state.session_store.create(session, 28_800).await.unwrap();

        let headers = headers_with_cookie(&format!("dubbridge_session={}", id.as_str()));
        let result = resolve_session(&state, &headers).await;
        assert!(
            result.unwrap().is_none(),
            "expired session must return None"
        );
    }

    #[tokio::test]
    async fn resolve_returns_session_for_mobile_header_transport() {
        let state = make_state();
        let session = sample_session();
        let id = state.session_store.create(session, 28_800).await.unwrap();

        let headers = headers_with_mobile_session(id.as_str());
        let result = resolve_session(&state, &headers).await;

        let (resolved_id, resolved_session, transport) = result.unwrap().unwrap();
        assert_eq!(resolved_id.as_str(), id.as_str());
        assert_eq!(resolved_session.subject, "user-1");
        assert!(matches!(transport, SessionTransport::Header(_)));
    }

    #[tokio::test]
    async fn resolve_rejects_mismatched_cookie_and_mobile_header() {
        let state = make_state();
        let cookie_id = state
            .session_store
            .create(sample_session(), 28_800)
            .await
            .unwrap();
        let header_id = SessionId::generate();

        let mut headers = headers_with_cookie(&format!("dubbridge_session={}", cookie_id.as_str()));
        headers.insert(
            MOBILE_SESSION_HEADER,
            HeaderValue::from_str(header_id.as_str()).unwrap(),
        );

        let result = resolve_session(&state, &headers).await;

        assert!(matches!(result, Err(SessionTransportError::Conflict)));
    }

    #[tokio::test]
    async fn resolve_accepts_matching_cookie_and_mobile_header() {
        let state = make_state();
        let session = sample_session();
        let id = state.session_store.create(session, 28_800).await.unwrap();

        let mut headers = headers_with_cookie(&format!("dubbridge_session={}", id.as_str()));
        headers.insert(
            MOBILE_SESSION_HEADER,
            HeaderValue::from_str(id.as_str()).unwrap(),
        );

        let result = resolve_session(&state, &headers).await;

        let (resolved_id, _, transport) = result.unwrap().unwrap();
        assert_eq!(resolved_id.as_str(), id.as_str());
        assert!(matches!(transport, SessionTransport::Both(_)));
    }
}
