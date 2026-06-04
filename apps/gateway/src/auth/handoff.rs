// P1-T7.2: single-use mobile handoff code store (ADR-024 mobile seam)
//
// Stores (handoff_code -> SessionId) for a very short redemption window.
// Entries are consumed on first successful redemption; expiry is enforced lazily.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;

use crate::session::SessionId;

struct HandoffEntry {
    session_id: SessionId,
    expires_at: Instant,
}

pub struct HandoffStore {
    map: Mutex<HashMap<String, HandoffEntry>>,
    ttl: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum HandoffError {
    #[error("handoff code not found or already used")]
    NotFound,
    #[error("handoff code has expired")]
    Expired,
}

impl HandoffStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    /// Default TTL of 90 seconds per the T7.1 contract.
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(90))
    }

    pub fn issue(&self, session_id: SessionId) -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let code = URL_SAFE_NO_PAD.encode(bytes);

        let entry = HandoffEntry {
            session_id,
            expires_at: Instant::now() + self.ttl,
        };

        self.map.lock().unwrap().insert(code.clone(), entry);
        code
    }

    pub fn consume(&self, code: &str) -> Result<SessionId, HandoffError> {
        let mut map = self.map.lock().unwrap();
        match map.remove(code) {
            None => Err(HandoffError::NotFound),
            Some(entry) if entry.expires_at < Instant::now() => Err(HandoffError::Expired),
            Some(entry) => Ok(entry.session_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_returns_43_char_base64url_code() {
        let store = HandoffStore::with_default_ttl();
        let code = store.issue(SessionId::generate());

        assert_eq!(code.len(), 43);
        assert!(
            code.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn consume_returns_original_session_id() {
        let store = HandoffStore::with_default_ttl();
        let session_id = SessionId::generate();
        let expected = session_id.as_str().to_string();

        let code = store.issue(session_id);
        let resolved = store.consume(&code).unwrap();

        assert_eq!(resolved.as_str(), expected);
    }

    #[test]
    fn consume_is_single_use() {
        let store = HandoffStore::with_default_ttl();
        let code = store.issue(SessionId::generate());

        let _first = store.consume(&code).unwrap();
        let second = store.consume(&code);

        assert!(matches!(second, Err(HandoffError::NotFound)));
    }

    #[test]
    fn consume_expired_code_returns_expired() {
        let store = HandoffStore::new(Duration::from_nanos(1));
        let code = store.issue(SessionId::generate());
        std::thread::sleep(Duration::from_millis(1));

        let result = store.consume(&code);

        assert!(matches!(result, Err(HandoffError::Expired)));
    }
}
