// P1-T3: hardened session cookie + CSRF helpers (ADR-024 T0 decisions)
//
// Cookie attributes (T0 decisions, ADR-024):
//   - HttpOnly  : JS cannot read the session cookie
//   - Secure    : HTTPS only
//   - SameSite=Lax : allows top-level navigations (login redirect) but blocks
//                    cross-site sub-resource requests
//   - Path=/    : host-scoped
//
// CSRF strategy (T0 decisions, ADR-024):
//   - Double-submit: CsrfToken stored server-side + echoed in a readable cookie
//     (non-HttpOnly). Mutation requests must include it in X-CSRF-Token header.

use axum::http::HeaderValue;

use crate::session::{CsrfToken, SessionId};

// ── Session cookie ────────────────────────────────────────────────────────────

/// Build the `Set-Cookie` header value that sets the hardened session cookie.
/// Only the opaque `session_id` is written to the cookie — never tokens.
pub fn build_session_cookie(session_id: &SessionId, cookie_name: &str) -> HeaderValue {
    let value = format!(
        "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/",
        cookie_name,
        session_id.as_str()
    );
    HeaderValue::from_str(&value).expect("session cookie header value must be valid ASCII")
}

/// Build the `Set-Cookie` header value that clears (expires) the session cookie.
pub fn clear_session_cookie(cookie_name: &str) -> HeaderValue {
    let value = format!(
        "{}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0",
        cookie_name
    );
    HeaderValue::from_str(&value).expect("clear session cookie header value must be valid ASCII")
}

// ── CSRF cookie ───────────────────────────────────────────────────────────────

/// Build the `Set-Cookie` header value for the CSRF double-submit cookie.
/// This cookie is intentionally NOT HttpOnly so the browser JS can read it
/// and include it in the `X-CSRF-Token` request header.
pub fn build_csrf_cookie(csrf_token: &CsrfToken, csrf_cookie_name: &str) -> HeaderValue {
    let value = format!(
        "{}={}; Secure; SameSite=Lax; Path=/",
        csrf_cookie_name,
        csrf_token.as_str()
    );
    HeaderValue::from_str(&value).expect("csrf cookie header value must be valid ASCII")
}

/// Build the `Set-Cookie` header value that clears the CSRF cookie.
pub fn clear_csrf_cookie(csrf_cookie_name: &str) -> HeaderValue {
    let value = format!(
        "{}=; Secure; SameSite=Lax; Path=/; Max-Age=0",
        csrf_cookie_name
    );
    HeaderValue::from_str(&value).expect("clear csrf cookie header value must be valid ASCII")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn access_token() -> &'static str {
        "eyJhbGciOiJSUzI1NiJ9.super-secret-payload"
    }

    // --- Session cookie attributes ---

    #[test]
    fn session_cookie_contains_http_only() {
        let id = SessionId::generate();
        let hv = build_session_cookie(&id, "dubbridge_session");
        assert!(
            hv.to_str().unwrap().contains("HttpOnly"),
            "session cookie must be HttpOnly"
        );
    }

    #[test]
    fn session_cookie_contains_secure() {
        let id = SessionId::generate();
        let hv = build_session_cookie(&id, "dubbridge_session");
        assert!(
            hv.to_str().unwrap().contains("Secure"),
            "session cookie must be Secure"
        );
    }

    #[test]
    fn session_cookie_contains_samesite_lax() {
        let id = SessionId::generate();
        let hv = build_session_cookie(&id, "dubbridge_session");
        assert!(
            hv.to_str().unwrap().contains("SameSite=Lax"),
            "session cookie must use SameSite=Lax"
        );
    }

    #[test]
    fn session_cookie_is_host_scoped_path() {
        let id = SessionId::generate();
        let hv = build_session_cookie(&id, "dubbridge_session");
        assert!(
            hv.to_str().unwrap().contains("Path=/"),
            "session cookie must be scoped to Path=/"
        );
    }

    #[test]
    fn session_cookie_contains_session_id_value() {
        let id = SessionId::generate();
        let hv = build_session_cookie(&id, "dubbridge_session");
        assert!(
            hv.to_str().unwrap().contains(id.as_str()),
            "session cookie must embed the session_id"
        );
    }

    // --- ADR-024 core invariant: cookie value must not contain an access token ---

    #[test]
    fn session_cookie_does_not_contain_access_token() {
        let id = SessionId::generate();
        let hv = build_session_cookie(&id, "dubbridge_session");
        assert!(
            !hv.to_str().unwrap().contains(access_token()),
            "session cookie must NEVER contain the access token (ADR-024)"
        );
    }

    // --- Clear cookie ---

    #[test]
    fn clear_session_cookie_sets_max_age_zero() {
        let hv = clear_session_cookie("dubbridge_session");
        assert!(
            hv.to_str().unwrap().contains("Max-Age=0"),
            "clear cookie must set Max-Age=0"
        );
    }

    #[test]
    fn clear_session_cookie_retains_security_attributes() {
        let hv = clear_session_cookie("dubbridge_session");
        let s = hv.to_str().unwrap();
        assert!(s.contains("HttpOnly"));
        assert!(s.contains("Secure"));
        assert!(s.contains("SameSite=Lax"));
    }

    // --- CSRF cookie ---

    #[test]
    fn csrf_cookie_does_not_have_http_only() {
        // Must be readable by browser JS for double-submit pattern
        let csrf = CsrfToken::generate();
        let hv = build_csrf_cookie(&csrf, "dubbridge_csrf");
        assert!(
            !hv.to_str().unwrap().contains("HttpOnly"),
            "csrf cookie must NOT be HttpOnly (JS must read it)"
        );
    }

    #[test]
    fn csrf_cookie_contains_secure_and_samesite() {
        let csrf = CsrfToken::generate();
        let hv = build_csrf_cookie(&csrf, "dubbridge_csrf");
        let s = hv.to_str().unwrap();
        assert!(s.contains("Secure"));
        assert!(s.contains("SameSite=Lax"));
    }

    #[test]
    fn csrf_cookie_contains_csrf_token_value() {
        let csrf = CsrfToken::generate();
        let hv = build_csrf_cookie(&csrf, "dubbridge_csrf");
        assert!(
            hv.to_str().unwrap().contains(csrf.as_str()),
            "csrf cookie must embed the csrf token"
        );
    }

    #[test]
    fn clear_csrf_cookie_sets_max_age_zero() {
        let hv = clear_csrf_cookie("dubbridge_csrf");
        assert!(hv.to_str().unwrap().contains("Max-Age=0"));
    }
}
