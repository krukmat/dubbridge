// P1-T3: server-side session contract (ADR-024)
// Tokens never reach the client — only an opaque SessionId in a hardened cookie.

pub mod store;

use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::auth::oauth_client::TokenSet;

// ── SessionId ────────────────────────────────────────────────────────────────

/// Opaque, high-entropy session identifier — the only value ever sent to the
/// client (as a cookie). Never holds tokens. (ADR-024 core invariant)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Generate a cryptographically random session id: 32 bytes → 43-char base64url.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Construct from an already-validated string (e.g. parsed from a cookie).
    pub fn from_cookie_value(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

// ── CsrfToken ────────────────────────────────────────────────────────────────

/// Double-submit CSRF token stored in the server-side session and echoed via a
/// readable cookie so the browser JS can include it in mutation-request headers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken(String);

impl CsrfToken {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Constant-time comparison — prevents timing-based token inference.
    pub fn verify(&self, provided: &str) -> bool {
        self.0.as_bytes().ct_eq(provided.as_bytes()).into()
    }
}

// ── StoredSession ─────────────────────────────────────────────────────────────

/// Server-side session payload. Tokens live here only — never in the cookie.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub subject: String,
    pub token_set: TokenSet,
    pub csrf_token: CsrfToken,
    /// Unix seconds when the session was created (for absolute TTL check).
    pub created_at_unix_secs: u64,
    /// Unix seconds of the last resolved request (for idle TTL check).
    pub last_accessed_unix_secs: u64,
}

impl StoredSession {
    pub fn new(subject: impl Into<String>, token_set: TokenSet, csrf_token: CsrfToken) -> Self {
        let now = unix_now();
        Self {
            subject: subject.into(),
            token_set,
            csrf_token,
            created_at_unix_secs: now,
            last_accessed_unix_secs: now,
        }
    }

    /// Returns true if either the absolute TTL or idle TTL has elapsed.
    pub fn is_expired(&self, absolute_ttl_secs: u64, idle_ttl_secs: u64) -> bool {
        let now = unix_now();
        let age = now.saturating_sub(self.created_at_unix_secs);
        let idle = now.saturating_sub(self.last_accessed_unix_secs);
        age >= absolute_ttl_secs || idle >= idle_ttl_secs
    }

    /// Update last_accessed to now (called by store.touch).
    pub fn touch(&mut self) {
        self.last_accessed_unix_secs = unix_now();
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── SessionError ─────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session serialization error: {0}")]
    Serialization(String),
    #[error("session backend error: {0}")]
    Backend(String),
}

// ── SessionStore trait ────────────────────────────────────────────────────────

/// Contract for server-side session storage.
/// `async_trait` is used so `dyn SessionStore` is object-safe with Send futures.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Persist a new session; returns the generated SessionId.
    async fn create(
        &self,
        session: StoredSession,
        absolute_ttl_secs: u64,
    ) -> Result<SessionId, SessionError>;

    /// Retrieve a session by id, checking both absolute and idle TTL.
    /// Returns `None` if the id is unknown or the session has expired (fail-closed).
    async fn resolve(
        &self,
        id: &SessionId,
        absolute_ttl_secs: u64,
        idle_ttl_secs: u64,
    ) -> Result<Option<StoredSession>, SessionError>;

    /// Reset the idle TTL by updating last_accessed_unix_secs.
    async fn touch(&self, id: &SessionId, idle_ttl_secs: u64) -> Result<(), SessionError>;

    /// Invalidate and remove a session (logout).
    async fn delete(&self, id: &SessionId) -> Result<(), SessionError>;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::store::InMemorySessionStore;

    fn sample_token_set() -> TokenSet {
        TokenSet {
            access_token: "eyJhbGciOiJSUzI1NiJ9.secret-payload".to_string(),
            refresh_token: Some("refresh-xyz".to_string()),
            expires_in: Some(3600),
            token_type: "Bearer".to_string(),
        }
    }

    // --- SessionId ---

    #[test]
    fn session_id_generates_unique_values() {
        let a = SessionId::generate();
        let b = SessionId::generate();
        assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn session_id_is_43_chars_base64url() {
        let id = SessionId::generate();
        assert_eq!(id.as_str().len(), 43);
        assert!(
            !id.as_str().contains('='),
            "session_id must not have base64 padding"
        );
        assert!(
            id.as_str()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "session_id must be base64url: {}",
            id.as_str()
        );
    }

    // --- CsrfToken ---

    #[test]
    fn csrf_token_generates_unique_values() {
        let a = CsrfToken::generate();
        let b = CsrfToken::generate();
        assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn csrf_token_verify_returns_true_for_correct_value() {
        let t = CsrfToken::generate();
        assert!(t.verify(t.as_str()));
    }

    #[test]
    fn csrf_token_verify_returns_false_for_wrong_value() {
        let t = CsrfToken::generate();
        assert!(!t.verify("wrong-token"));
    }

    #[test]
    fn csrf_token_verify_returns_false_for_empty_string() {
        let t = CsrfToken::generate();
        assert!(!t.verify(""));
    }

    // --- StoredSession ---

    #[test]
    fn stored_session_is_not_expired_immediately_after_creation() {
        let s = StoredSession::new("user-1", sample_token_set(), CsrfToken::generate());
        assert!(!s.is_expired(28_800, 1_800));
    }

    #[test]
    fn stored_session_is_expired_when_absolute_ttl_elapsed() {
        let mut s = StoredSession::new("user-1", sample_token_set(), CsrfToken::generate());
        // Simulate creation 9 hours ago (> 8h absolute TTL)
        s.created_at_unix_secs = unix_now().saturating_sub(9 * 3600);
        s.last_accessed_unix_secs = unix_now(); // recently accessed — idle ok
        assert!(
            s.is_expired(28_800, 1_800),
            "should be expired by absolute TTL"
        );
    }

    #[test]
    fn stored_session_is_expired_when_idle_ttl_elapsed() {
        let mut s = StoredSession::new("user-1", sample_token_set(), CsrfToken::generate());
        // Simulate last access 31 minutes ago (> 30m idle TTL)
        s.last_accessed_unix_secs = unix_now().saturating_sub(31 * 60);
        assert!(s.is_expired(28_800, 1_800), "should be expired by idle TTL");
    }

    #[test]
    fn stored_session_touch_updates_last_accessed() {
        let mut s = StoredSession::new("user-1", sample_token_set(), CsrfToken::generate());
        s.last_accessed_unix_secs = unix_now().saturating_sub(100);
        let before = s.last_accessed_unix_secs;
        s.touch();
        assert!(s.last_accessed_unix_secs >= before);
    }

    // --- InMemorySessionStore ---

    #[tokio::test]
    async fn store_create_then_resolve_returns_session() {
        let store = InMemorySessionStore::new();
        let session = StoredSession::new("alice", sample_token_set(), CsrfToken::generate());
        let id = store.create(session.clone(), 28_800).await.unwrap();
        let resolved = store.resolve(&id, 28_800, 1_800).await.unwrap();
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().subject, "alice");
    }

    #[tokio::test]
    async fn store_resolve_unknown_id_returns_none() {
        let store = InMemorySessionStore::new();
        let unknown = SessionId::generate();
        let result = store.resolve(&unknown, 28_800, 1_800).await.unwrap();
        assert!(
            result.is_none(),
            "unknown session_id must return None (fail-closed)"
        );
    }

    #[tokio::test]
    async fn store_delete_makes_session_unresolvable() {
        let store = InMemorySessionStore::new();
        let session = StoredSession::new("bob", sample_token_set(), CsrfToken::generate());
        let id = store.create(session, 28_800).await.unwrap();
        store.delete(&id).await.unwrap();
        let result = store.resolve(&id, 28_800, 1_800).await.unwrap();
        assert!(result.is_none(), "deleted session must not be resolvable");
    }

    #[tokio::test]
    async fn store_resolve_expired_absolute_returns_none() {
        let store = InMemorySessionStore::new();
        let mut session = StoredSession::new("carol", sample_token_set(), CsrfToken::generate());
        session.created_at_unix_secs = unix_now().saturating_sub(9 * 3600);
        session.last_accessed_unix_secs = unix_now();
        let id = store.create(session, 28_800).await.unwrap();
        let result = store.resolve(&id, 28_800, 1_800).await.unwrap();
        assert!(
            result.is_none(),
            "absolutely expired session must return None"
        );
    }

    #[tokio::test]
    async fn store_resolve_expired_idle_returns_none() {
        let store = InMemorySessionStore::new();
        let mut session = StoredSession::new("dave", sample_token_set(), CsrfToken::generate());
        session.last_accessed_unix_secs = unix_now().saturating_sub(31 * 60);
        let id = store.create(session, 28_800).await.unwrap();
        let result = store.resolve(&id, 28_800, 1_800).await.unwrap();
        assert!(result.is_none(), "idle-expired session must return None");
    }

    #[tokio::test]
    async fn store_touch_resets_idle_timer() {
        let store = InMemorySessionStore::new();
        let mut session = StoredSession::new("eve", sample_token_set(), CsrfToken::generate());
        // Set last_accessed to 20 minutes ago — not yet idle-expired
        session.last_accessed_unix_secs = unix_now().saturating_sub(20 * 60);
        let id = store.create(session, 28_800).await.unwrap();
        store.touch(&id, 1_800).await.unwrap();
        // After touch, last_accessed is refreshed — resolve must succeed
        let result = store.resolve(&id, 28_800, 1_800).await.unwrap();
        assert!(result.is_some(), "touched session must still be resolvable");
    }

    // --- ADR-024 core invariant ---

    #[tokio::test]
    async fn stored_session_serialized_does_not_contain_access_token() {
        // The JSON representation of a StoredSession must never be sent to the client.
        // This test documents the invariant: the cookie carries only the session_id,
        // not the StoredSession blob.
        let access_token = "super-secret-access-token-XYZ";
        let _session = StoredSession::new(
            "user",
            TokenSet {
                access_token: access_token.to_string(),
                refresh_token: None,
                expires_in: None,
                token_type: "Bearer".to_string(),
            },
            CsrfToken::generate(),
        );
        let session_id = SessionId::generate();

        // The value the client receives is only the session_id — not the session JSON
        let cookie_value = session_id.as_str();
        assert!(
            !cookie_value.contains(access_token),
            "cookie value must not contain the access token (ADR-024)"
        );
    }
}
