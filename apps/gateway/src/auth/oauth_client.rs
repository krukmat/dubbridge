// P1-T2: OAuth 2.0 client — pure builder + IO executor (ADR-024, ADR-018)

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ── Types ────────────────────────────────────────────────────────────────────

/// PKCE code_verifier: high-entropy random string, base64url-encoded (RFC 7636).
/// 32 random bytes → 43 base64url chars, all unreserved (no padding).
pub struct PkceVerifier(String);

/// PKCE code_challenge: BASE64URL(SHA256(ASCII(verifier))) — S256 method.
pub struct PkceChallenge(String);

/// Single-use random CSRF state parameter for the authorization request.
pub struct OAuthState(String);

/// Typed token response from the authorization server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

/// Typed errors for all OAuth operations.
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("authorization server error '{error}': {error_description}")]
    ServerError {
        error: String,
        error_description: String,
    },
    #[error("invalid token response: {0}")]
    InvalidResponse(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url parse error: {0}")]
    UrlParse(String),
}

// ── PKCE helpers ─────────────────────────────────────────────────────────────

impl PkceVerifier {
    /// Generate a cryptographically random PKCE code_verifier.
    /// 32 bytes from OS entropy → 43-char base64url string (no padding, all unreserved).
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PkceChallenge {
    /// Derive the S256 challenge: BASE64URL(SHA256(ASCII(code_verifier))).
    pub fn from_verifier(verifier: &PkceVerifier) -> Self {
        let digest = Sha256::digest(verifier.as_str().as_bytes());
        Self(URL_SAFE_NO_PAD.encode(digest))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl OAuthState {
    /// Generate a cryptographically random single-use state value.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ── Pure builders ─────────────────────────────────────────────────────────────

/// Build the authorization redirect URL.
/// Pure: no IO — testable against expected query params without network.
pub fn build_authorization_url(
    authorization_base_url: &str,
    client_id: &str,
    redirect_uri: &str,
    state: &OAuthState,
    challenge: &PkceChallenge,
    scope: Option<&str>,
) -> Result<String, OAuthError> {
    let mut url =
        url::Url::parse(authorization_base_url).map_err(|e| OAuthError::UrlParse(e.to_string()))?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("response_type", "code");
        q.append_pair("client_id", client_id);
        q.append_pair("redirect_uri", redirect_uri);
        q.append_pair("state", state.as_str());
        q.append_pair("code_challenge", challenge.as_str());
        q.append_pair("code_challenge_method", "S256");
        if let Some(s) = scope {
            q.append_pair("scope", s);
        }
    }
    Ok(url.to_string())
}

/// Build token-exchange form params (pure — no client secret, no IO).
/// The secret is injected by the IO executor only, never stored in builder output
/// (ADR-018 redaction invariant).
pub fn build_token_exchange_params(
    client_id: &str,
    code: &str,
    verifier: &PkceVerifier,
    redirect_uri: &str,
) -> Vec<(String, String)> {
    vec![
        ("grant_type".into(), "authorization_code".into()),
        ("client_id".into(), client_id.into()),
        ("code".into(), code.into()),
        ("code_verifier".into(), verifier.as_str().into()),
        ("redirect_uri".into(), redirect_uri.into()),
    ]
}

/// Build token-refresh form params (pure — no client secret, no IO).
pub fn build_token_refresh_params(client_id: &str, refresh_token: &str) -> Vec<(String, String)> {
    vec![
        ("grant_type".into(), "refresh_token".into()),
        ("client_id".into(), client_id.into()),
        ("refresh_token".into(), refresh_token.into()),
    ]
}

// ── Token response parsing ────────────────────────────────────────────────────

/// Parse a JSON body into a TokenSet or a typed OAuthError.
/// Separated from IO so it can be unit-tested directly against serde_json::Value.
pub fn parse_token_response(value: serde_json::Value) -> Result<TokenSet, OAuthError> {
    // Error response takes precedence (RFC 6749 §5.2)
    if let Some(error) = value.get("error").and_then(|v| v.as_str()) {
        return Err(OAuthError::ServerError {
            error: error.to_string(),
            error_description: value
                .get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("no description")
                .to_string(),
        });
    }

    let access_token = value
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OAuthError::InvalidResponse("missing access_token".into()))?
        .to_string();

    Ok(TokenSet {
        access_token,
        refresh_token: value
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        expires_in: value.get("expires_in").and_then(|v| v.as_u64()),
        token_type: value
            .get("token_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Bearer")
            .to_string(),
    })
}

// ── IO executor ───────────────────────────────────────────────────────────────

/// Sends OAuth token requests to the authorization server.
/// Borrows a `reqwest::Client` from `GatewayState` — does not own it.
pub struct OAuthExecutor<'a> {
    client: &'a reqwest::Client,
}

impl<'a> OAuthExecutor<'a> {
    pub fn new(client: &'a reqwest::Client) -> Self {
        Self { client }
    }

    /// Send a token request (exchange or refresh).
    /// The client secret is injected here — the only point it touches params —
    /// so it never persists in builder output or logs (ADR-018).
    pub async fn send_token_request(
        &self,
        token_url: &str,
        mut params: Vec<(String, String)>,
        client_secret: Option<&str>,
    ) -> Result<TokenSet, OAuthError> {
        if let Some(secret) = client_secret {
            params.push(("client_secret".into(), secret.into()));
        }

        let body: serde_json::Value = self
            .client
            .post(token_url)
            .form(&params)
            .send()
            .await?
            .json()
            .await?;

        parse_token_response(body)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // --- PkceVerifier ---

    #[test]
    fn pkce_verifier_length_is_in_valid_range() {
        let v = PkceVerifier::generate();
        let len = v.as_str().len();
        assert!(
            (43..=128).contains(&len),
            "verifier length {len} is outside [43, 128]"
        );
    }

    #[test]
    fn pkce_verifier_uses_only_unreserved_chars() {
        // RFC 7636: unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
        // base64url chars (A-Z a-z 0-9 - _) are a strict subset of unreserved
        let v = PkceVerifier::generate();
        assert!(
            v.as_str()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-._~".contains(c)),
            "verifier contains reserved chars: {}",
            v.as_str()
        );
    }

    #[test]
    fn pkce_verifier_is_different_each_call() {
        let a = PkceVerifier::generate();
        let b = PkceVerifier::generate();
        assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn pkce_verifier_has_no_padding() {
        let v = PkceVerifier::generate();
        assert!(
            !v.as_str().contains('='),
            "verifier must not contain base64 padding"
        );
    }

    // --- PkceChallenge ---

    #[test]
    fn pkce_challenge_differs_from_verifier() {
        let v = PkceVerifier::generate();
        let c = PkceChallenge::from_verifier(&v);
        assert_ne!(v.as_str(), c.as_str());
    }

    #[test]
    fn pkce_challenge_is_base64url_no_padding() {
        let v = PkceVerifier::generate();
        let c = PkceChallenge::from_verifier(&v);
        assert!(
            !c.as_str().contains('='),
            "challenge must not contain padding"
        );
        assert!(
            c.as_str()
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'),
            "challenge is not valid base64url: {}",
            c.as_str()
        );
    }

    #[test]
    fn pkce_challenge_is_43_chars_for_sha256_output() {
        // SHA-256 → 32 bytes → 43 base64url chars (no padding)
        let v = PkceVerifier::generate();
        let c = PkceChallenge::from_verifier(&v);
        assert_eq!(
            c.as_str().len(),
            43,
            "S256 challenge must be exactly 43 chars (32 SHA-256 bytes base64url-encoded)"
        );
    }

    #[test]
    fn pkce_challenge_is_deterministic_for_same_verifier() {
        let v = PkceVerifier::generate();
        let c1 = PkceChallenge::from_verifier(&v);
        let c2 = PkceChallenge::from_verifier(&v);
        assert_eq!(c1.as_str(), c2.as_str());
    }

    // --- OAuthState ---

    #[test]
    fn oauth_state_is_different_each_call() {
        let a = OAuthState::generate();
        let b = OAuthState::generate();
        assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn oauth_state_is_not_empty() {
        let s = OAuthState::generate();
        assert!(!s.as_str().is_empty());
    }

    // --- build_authorization_url ---

    #[test]
    fn authorization_url_includes_all_required_params() {
        let state = OAuthState::generate();
        let verifier = PkceVerifier::generate();
        let challenge = PkceChallenge::from_verifier(&verifier);

        let url = build_authorization_url(
            "https://auth.example.com/oauth/authorize",
            "my-client",
            "https://gateway.example.com/auth/callback",
            &state,
            &challenge,
            None,
        )
        .unwrap();

        assert!(url.contains("response_type=code"), "missing response_type");
        assert!(url.contains("client_id=my-client"), "missing client_id");
        assert!(url.contains("code_challenge_method=S256"), "missing method");
        assert!(url.contains("code_challenge="), "missing challenge");
        assert!(url.contains(state.as_str()), "state value not found in url");
        assert!(url.contains("redirect_uri="), "missing redirect_uri");
    }

    #[test]
    fn authorization_url_with_scope_includes_scope() {
        let state = OAuthState::generate();
        let verifier = PkceVerifier::generate();
        let challenge = PkceChallenge::from_verifier(&verifier);

        let url = build_authorization_url(
            "https://auth.example.com/oauth/authorize",
            "my-client",
            "https://gateway.example.com/auth/callback",
            &state,
            &challenge,
            Some("openid profile"),
        )
        .unwrap();

        assert!(url.contains("scope="), "missing scope");
    }

    #[test]
    fn authorization_url_rejects_invalid_base_url() {
        let state = OAuthState::generate();
        let verifier = PkceVerifier::generate();
        let challenge = PkceChallenge::from_verifier(&verifier);

        let result = build_authorization_url(
            "not-a-valid-url",
            "client",
            "https://redirect.example.com/callback",
            &state,
            &challenge,
            None,
        );

        assert!(
            matches!(result, Err(OAuthError::UrlParse(_))),
            "expected UrlParse error"
        );
    }

    // --- build_token_exchange_params ---

    #[test]
    fn token_exchange_params_contain_all_required_fields() {
        let verifier = PkceVerifier::generate();
        let params = build_token_exchange_params(
            "my-client",
            "auth-code-abc",
            &verifier,
            "https://gateway.example.com/auth/callback",
        );

        let map: std::collections::HashMap<_, _> = params.into_iter().collect();
        assert_eq!(
            map.get("grant_type").map(String::as_str),
            Some("authorization_code")
        );
        assert_eq!(map.get("client_id").map(String::as_str), Some("my-client"));
        assert_eq!(map.get("code").map(String::as_str), Some("auth-code-abc"));
        assert_eq!(
            map.get("code_verifier").map(String::as_str),
            Some(verifier.as_str())
        );
        assert!(map.contains_key("redirect_uri"), "missing redirect_uri");
    }

    #[test]
    fn token_exchange_params_do_not_contain_client_secret() {
        // ADR-018: secret injected by executor only, never in builder output
        let verifier = PkceVerifier::generate();
        let params =
            build_token_exchange_params("client", "code", &verifier, "https://redirect.example");
        for (k, _) in &params {
            assert_ne!(
                k, "client_secret",
                "client_secret must not appear in token exchange builder output"
            );
        }
    }

    // --- build_token_refresh_params ---

    #[test]
    fn token_refresh_params_contain_all_required_fields() {
        let params = build_token_refresh_params("my-client", "refresh-token-xyz");

        let map: std::collections::HashMap<_, _> = params.into_iter().collect();
        assert_eq!(
            map.get("grant_type").map(String::as_str),
            Some("refresh_token")
        );
        assert_eq!(map.get("client_id").map(String::as_str), Some("my-client"));
        assert_eq!(
            map.get("refresh_token").map(String::as_str),
            Some("refresh-token-xyz")
        );
    }

    #[test]
    fn token_refresh_params_do_not_contain_client_secret() {
        // ADR-018: secret injected by executor only, never in builder output
        let params = build_token_refresh_params("client", "refresh-token");
        for (k, _) in &params {
            assert_ne!(
                k, "client_secret",
                "client_secret must not appear in token refresh builder output"
            );
        }
    }

    // --- parse_token_response ---

    #[test]
    fn parse_token_response_returns_token_set_for_valid_response() {
        let json = serde_json::json!({
            "access_token": "eyJhbGciOiJSUzI1NiJ9.test",
            "refresh_token": "refresh-xyz",
            "expires_in": 3600,
            "token_type": "Bearer"
        });

        let ts = parse_token_response(json).unwrap();
        assert_eq!(ts.access_token, "eyJhbGciOiJSUzI1NiJ9.test");
        assert_eq!(ts.refresh_token.as_deref(), Some("refresh-xyz"));
        assert_eq!(ts.expires_in, Some(3600));
        assert_eq!(ts.token_type, "Bearer");
    }

    #[test]
    fn parse_token_response_uses_bearer_default_when_token_type_absent() {
        let json = serde_json::json!({ "access_token": "tok" });
        let ts = parse_token_response(json).unwrap();
        assert_eq!(ts.token_type, "Bearer");
        assert!(ts.refresh_token.is_none());
        assert!(ts.expires_in.is_none());
    }

    #[test]
    fn parse_token_response_returns_server_error_on_error_field() {
        let json = serde_json::json!({
            "error": "invalid_grant",
            "error_description": "the provided authorization grant is invalid"
        });

        let result = parse_token_response(json);
        assert!(
            matches!(
                result,
                Err(OAuthError::ServerError { ref error, .. }) if error == "invalid_grant"
            ),
            "expected ServerError(invalid_grant), got: {result:?}"
        );
    }

    #[test]
    fn parse_token_response_returns_invalid_response_when_access_token_absent() {
        let json = serde_json::json!({ "token_type": "Bearer", "expires_in": 3600 });
        let result = parse_token_response(json);
        assert!(
            matches!(result, Err(OAuthError::InvalidResponse(_))),
            "expected InvalidResponse, got: {result:?}"
        );
    }

    #[test]
    fn parse_token_response_server_error_is_typed_not_raw_string() {
        // Invariant: server errors produce a typed variant, never a free-form string
        // that could inadvertently log a reflected secret (ADR-018)
        let json = serde_json::json!({
            "error": "invalid_client",
            "error_description": "invalid client secret"
        });
        assert!(matches!(
            parse_token_response(json),
            Err(OAuthError::ServerError { .. })
        ));
    }
}
