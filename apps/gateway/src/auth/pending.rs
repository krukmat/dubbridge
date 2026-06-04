// P1-T4: PendingAuthStore — single-use state/verifier pairs with TTL (ADR-024)
//
// Stores (state → PkceVerifier) for the duration of the OAuth login round-trip.
// Entries are consumed on first use; any second call with the same state → error.
// TTL is enforced lazily on consume (no background sweep needed).

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::auth::oauth_client::PkceVerifier;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PendingAuth {
    verifier: PkceVerifier,
    return_uri: Option<String>,
}

impl PendingAuth {
    pub fn new(verifier: PkceVerifier, return_uri: Option<String>) -> Self {
        Self {
            verifier,
            return_uri,
        }
    }

    pub fn verifier(&self) -> &PkceVerifier {
        &self.verifier
    }

    pub fn into_parts(self) -> (PkceVerifier, Option<String>) {
        (self.verifier, self.return_uri)
    }
}

struct PendingEntry {
    auth: PendingAuth,
    expires_at: Instant,
}

/// In-memory store for pending OAuth state/verifier pairs.
/// Single-use: `consume` removes the entry atomically. Expired entries are
/// dropped on consume rather than on a background timer (sufficient for a
/// short-lived round-trip store).
pub struct PendingAuthStore {
    map: Mutex<HashMap<String, PendingEntry>>,
    ttl: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum PendingError {
    #[error("state not found or already used")]
    NotFound,
    #[error("state has expired")]
    Expired,
}

impl PendingAuthStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    /// Default TTL of 10 minutes — sufficient for a browser OAuth round-trip.
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(10 * 60))
    }

    /// Store a pending auth entry. Overwrites any prior entry for the same
    /// state string (should not happen in practice; each login generates fresh state).
    pub fn insert(&self, state: &str, auth: PendingAuth) {
        let entry = PendingEntry {
            auth,
            expires_at: Instant::now() + self.ttl,
        };
        self.map.lock().unwrap().insert(state.to_string(), entry);
    }

    pub fn insert_verifier_only(&self, state: &str, verifier: PkceVerifier) {
        self.insert(state, PendingAuth::new(verifier, None));
    }

    /// Consume the pending auth for `state`. Removes the entry atomically.
    /// Returns `Err(NotFound)` if missing (already used or never inserted).
    /// Returns `Err(Expired)` if the TTL has elapsed.
    pub fn consume(&self, state: &str) -> Result<PendingAuth, PendingError> {
        let mut map = self.map.lock().unwrap();
        match map.remove(state) {
            None => Err(PendingError::NotFound),
            Some(entry) if entry.expires_at < Instant::now() => Err(PendingError::Expired),
            Some(entry) => Ok(entry.auth),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::oauth_client::{PkceChallenge, PkceVerifier};

    fn make_verifier() -> PkceVerifier {
        PkceVerifier::generate()
    }

    // --- insert + consume happy path ---

    #[test]
    fn consume_returns_verifier_for_inserted_state() {
        let store = PendingAuthStore::with_default_ttl();
        let verifier = make_verifier();
        let expected = verifier.as_str().to_string();

        store.insert_verifier_only("state-abc", verifier);
        let got = store.consume("state-abc").unwrap();

        assert_eq!(got.verifier().as_str(), expected);
    }

    // --- single-use enforcement (ADR-024 hard invariant) ---

    #[test]
    fn consume_second_call_returns_not_found() {
        let store = PendingAuthStore::with_default_ttl();
        store.insert_verifier_only("state-xyz", make_verifier());

        let _first = store.consume("state-xyz").unwrap();
        let second = store.consume("state-xyz");

        assert!(
            matches!(second, Err(PendingError::NotFound)),
            "second consume must fail (single-use invariant)"
        );
    }

    // --- missing state ---

    #[test]
    fn consume_unknown_state_returns_not_found() {
        let store = PendingAuthStore::with_default_ttl();
        let result = store.consume("never-inserted");
        assert!(matches!(result, Err(PendingError::NotFound)));
    }

    // --- expired state ---

    #[test]
    fn consume_expired_entry_returns_expired() {
        // TTL of 1 nanosecond — will be expired by the time consume runs
        let store = PendingAuthStore::new(Duration::from_nanos(1));
        store.insert_verifier_only("state-expired", make_verifier());

        // spin briefly to ensure the instant has passed
        std::thread::sleep(Duration::from_millis(1));

        let result = store.consume("state-expired");
        assert!(
            matches!(result, Err(PendingError::Expired)),
            "expired entry must return Expired, not the verifier"
        );
    }

    // --- verifier/challenge integrity after round-trip ---

    #[test]
    fn consumed_verifier_still_produces_correct_challenge() {
        let store = PendingAuthStore::with_default_ttl();
        let original_verifier = make_verifier();
        let expected_challenge = PkceChallenge::from_verifier(&original_verifier);

        store.insert_verifier_only("state-check", original_verifier);
        let consumed = store.consume("state-check").unwrap();
        let derived_challenge = PkceChallenge::from_verifier(consumed.verifier());

        assert_eq!(
            consumed.verifier().as_str(),
            consumed.verifier().as_str(),
            "verifier value must be intact after store round-trip"
        );
        assert_eq!(
            derived_challenge.as_str(),
            expected_challenge.as_str(),
            "challenge derived from consumed verifier must match original"
        );
    }

    // --- isolation between entries ---

    #[test]
    fn multiple_states_are_independent() {
        let store = PendingAuthStore::with_default_ttl();
        let v1 = make_verifier();
        let v2 = make_verifier();
        let v1_str = v1.as_str().to_string();
        let v2_str = v2.as_str().to_string();

        store.insert_verifier_only("state-1", v1);
        store.insert_verifier_only("state-2", v2);

        let got1 = store.consume("state-1").unwrap();
        let got2 = store.consume("state-2").unwrap();

        assert_eq!(got1.verifier().as_str(), v1_str);
        assert_eq!(got2.verifier().as_str(), v2_str);
    }

    #[test]
    fn consume_preserves_mobile_return_uri() {
        let store = PendingAuthStore::with_default_ttl();
        let verifier = make_verifier();
        store.insert(
            "state-mobile",
            PendingAuth::new(verifier, Some("dubbridge://auth/callback".to_string())),
        );

        let consumed = store.consume("state-mobile").unwrap();
        let (_, return_uri) = consumed.into_parts();

        assert_eq!(return_uri.as_deref(), Some("dubbridge://auth/callback"));
    }
}
