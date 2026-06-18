// S-200-T1b: HS256 access-token issuance and verification.
//
// This module is the in-house token issuer adopted by ADR-031 (FenixCRM parity):
// `apps/api` signs its own HS256 access tokens instead of validating RS256 tokens
// from an external authorization server.
//
// T1b-i delivered the signing half (`Claims` + `generate_jwt`).
// T1b-ii (this change) adds the verification half (`parse_jwt` + algorithm pinning).
// Algorithm pinning rejects any non-HS256 `alg` header — including `none` and RS256 —
// before any signature check, per ADR-031 §Decision 2 and §Risk R6.
// The swap of the protected-route `TokenVerifier` is S-200-T1c.
//
// Security note (ADR-031 §Risk R2): the HMAC secret both signs and verifies, so a
// leak lets an attacker mint tokens. The issuer therefore fails closed on an empty
// secret and never logs it. RS256 hardening is the deferred follow-up X-S-200-1.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Claims carried by a DubBridge-issued HS256 access token (ADR-031 §Decision 2).
///
/// `sub` is the account UUID and remains the sole source of `assets.uploader_id`
/// (ADR-008); it is stored in string form per the JWT registered-claim convention.
/// `scope` is a space-delimited list (RFC 8693 style).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub workspace_id: String,
    pub iat: u64,
    pub nbf: u64,
    pub exp: u64,
    #[serde(default)]
    pub scope: String,
}

/// Errors raised while signing an access token. Never carries the secret or token.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IssueError {
    #[error("HMAC signing secret must not be empty")]
    EmptySecret,
    #[error("system clock is before the UNIX epoch")]
    Clock,
    #[error("failed to sign access token")]
    Signing,
}

/// Errors raised while parsing (verifying) an access token. Never carries the secret.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("token algorithm is not HS256")]
    InvalidAlgorithm,
    #[error("token signature is invalid")]
    InvalidSignature,
    #[error("token has expired")]
    Expired,
    #[error("token is not yet valid (nbf)")]
    NotYetValid,
    #[error("token subject is not a valid UUID")]
    InvalidSubject,
    #[error("token workspace_id is not a valid UUID")]
    InvalidWorkspace,
    #[error("token is malformed or cannot be decoded")]
    MalformedToken,
}

/// Verify an HS256 access token and return its claims.
///
/// Algorithm pinning: `alg` in the header must be exactly `HS256`. Any other value
/// (including `none`, RS256, or any ECDSA variant) is rejected before the signature
/// check — per ADR-031 §Decision 2 and §Risk R6 (algorithm-substitution attack).
pub fn parse_jwt(token: &str, secret: &str) -> Result<Claims, ParseError> {
    let header = decode_header(token).map_err(|_| ParseError::MalformedToken)?;
    if header.alg != Algorithm::HS256 {
        return Err(ParseError::InvalidAlgorithm);
    }

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.validate_aud = false;
    validation.required_spec_claims.clear();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::InvalidSignature => ParseError::InvalidSignature,
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => ParseError::Expired,
        jsonwebtoken::errors::ErrorKind::ImmatureSignature => ParseError::NotYetValid,
        _ => ParseError::MalformedToken,
    })?;

    let claims = token_data.claims;
    Uuid::parse_str(&claims.sub).map_err(|_| ParseError::InvalidSubject)?;
    Uuid::parse_str(&claims.workspace_id).map_err(|_| ParseError::InvalidWorkspace)?;

    Ok(claims)
}

/// HS256 access-token issuer. Holds the signing key and the token lifetime.
pub struct Hs256Issuer {
    encoding_key: EncodingKey,
    expiry: Duration,
}

impl Hs256Issuer {
    /// Build an issuer. Fails closed when the secret is empty or whitespace so a
    /// misconfigured deployment can never sign tokens with a blank key.
    pub fn new(secret: &str, expiry: Duration) -> Result<Self, IssueError> {
        if secret.trim().is_empty() {
            return Err(IssueError::EmptySecret);
        }

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            expiry,
        })
    }

    /// Sign an HS256 access token for `sub` within `workspace_id` carrying `scopes`.
    /// `exp = iat + expiry`; `nbf = iat`.
    pub fn generate_jwt(
        &self,
        sub: Uuid,
        workspace_id: Uuid,
        scopes: &[String],
    ) -> Result<String, IssueError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| IssueError::Clock)?
            .as_secs();

        let claims = Claims {
            sub: sub.to_string(),
            workspace_id: workspace_id.to_string(),
            iat: now,
            nbf: now,
            exp: now.saturating_add(self.expiry.as_secs()),
            scope: scopes.join(" "),
        };

        let header = Header::new(Algorithm::HS256);

        encode(&header, &claims, &self.encoding_key).map_err(|_| IssueError::Signing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use jsonwebtoken::{DecodingKey, Validation, decode, decode_header};

    const SECRET: &str = "test-hmac-secret-key-0123456789";

    fn issuer(expiry_secs: u64) -> Hs256Issuer {
        Hs256Issuer::new(SECRET, Duration::from_secs(expiry_secs)).expect("issuer")
    }

    // Decode without enforcing time/audience so tests assert the claims as written,
    // while still verifying the HS256 signature against `secret`.
    fn decode_with(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;
        validation.required_spec_claims.clear();

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
    }

    fn scopes(values: &[&str]) -> Vec<String> {
        values.iter().map(ToString::to_string).collect()
    }

    // HP-1: generate -> decode round-trips every claim.
    #[test]
    fn generate_round_trips_claims() {
        let issuer = issuer(3600);
        let sub = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        let token = issuer
            .generate_jwt(
                sub,
                workspace_id,
                &scopes(&["assets:read", "assets:ingest"]),
            )
            .expect("token");
        let claims = decode_with(&token, SECRET).expect("decode");

        assert_eq!(claims.sub, sub.to_string());
        assert_eq!(claims.workspace_id, workspace_id.to_string());
        assert_eq!(claims.scope, "assets:read assets:ingest");
        assert_eq!(claims.iat, claims.nbf);
        assert_eq!(claims.exp, claims.iat + 3600);
    }

    // Crypto: the signed header is HS256.
    #[test]
    fn signed_header_is_hs256() {
        let issuer = issuer(3600);
        let token = issuer
            .generate_jwt(Uuid::new_v4(), Uuid::new_v4(), &scopes(&["assets:read"]))
            .expect("token");

        let header = decode_header(&token).expect("header");

        assert_eq!(header.alg, Algorithm::HS256);
    }

    // EC-2: the configured expiry is applied exactly, including a 1s boundary.
    #[test]
    fn expiry_is_applied_exactly() {
        for expiry in [1_u64, 3600, 86_400] {
            let issuer = issuer(expiry);
            let token = issuer
                .generate_jwt(Uuid::new_v4(), Uuid::new_v4(), &[])
                .expect("token");
            let claims = decode_with(&token, SECRET).expect("decode");

            assert_eq!(claims.exp - claims.iat, expiry);
        }
    }

    // Crypto: a token signed with one secret does not verify under another.
    #[test]
    fn token_does_not_verify_under_a_different_secret() {
        let issuer = issuer(3600);
        let token = issuer
            .generate_jwt(Uuid::new_v4(), Uuid::new_v4(), &scopes(&["assets:read"]))
            .expect("token");

        let error = decode_with(&token, "a-completely-different-secret").expect_err("mismatch");

        assert_eq!(
            error.kind(),
            &jsonwebtoken::errors::ErrorKind::InvalidSignature,
        );
    }

    // No scopes -> empty scope string (not "null", not a panic).
    #[test]
    fn empty_scopes_produce_empty_scope_string() {
        let issuer = issuer(3600);
        let token = issuer
            .generate_jwt(Uuid::new_v4(), Uuid::new_v4(), &[])
            .expect("token");
        let claims = decode_with(&token, SECRET).expect("decode");

        assert_eq!(claims.scope, "");
    }

    // EC-1: an empty or whitespace secret fails closed; no issuer, no token.
    #[test]
    fn new_rejects_empty_secret() {
        assert_eq!(
            Hs256Issuer::new("", Duration::from_secs(3600)).err(),
            Some(IssueError::EmptySecret),
        );
        assert_eq!(
            Hs256Issuer::new("   ", Duration::from_secs(3600)).err(),
            Some(IssueError::EmptySecret),
        );
    }

    // -------------------------------------------------------------------------
    // parse_jwt tests (S-200-T1b-ii)
    // -------------------------------------------------------------------------

    fn token_for(sub: Uuid, workspace_id: Uuid, expiry_secs: u64) -> String {
        issuer(expiry_secs)
            .generate_jwt(sub, workspace_id, &scopes(&["assets:read"]))
            .expect("token")
    }

    // HP-1: generate -> parse round-trips all claims.
    #[test]
    fn parse_round_trips_claims() {
        let sub = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let token = token_for(sub, workspace_id, 3600);

        let claims = parse_jwt(&token, SECRET).expect("parse ok");

        assert_eq!(claims.sub, sub.to_string());
        assert_eq!(claims.workspace_id, workspace_id.to_string());
        assert_eq!(claims.scope, "assets:read");
    }

    // AC-5: wrong secret -> InvalidSignature.
    #[test]
    fn parse_rejects_wrong_secret() {
        let token = token_for(Uuid::new_v4(), Uuid::new_v4(), 3600);

        assert_eq!(
            parse_jwt(&token, "completely-different-secret").err(),
            Some(ParseError::InvalidSignature),
        );
    }

    // AC-6: expired token -> Expired.
    #[test]
    fn parse_rejects_expired_token() {
        use jsonwebtoken::{EncodingKey, Header, encode};

        // Craft a token with exp in the past.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            workspace_id: Uuid::new_v4().to_string(),
            iat: now - 7200,
            nbf: now - 7200,
            exp: now - 3600,
            scope: String::new(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .expect("encode");

        assert_eq!(parse_jwt(&token, SECRET).err(), Some(ParseError::Expired));
    }

    // AC-7: nbf in the future -> NotYetValid.
    #[test]
    fn parse_rejects_future_nbf() {
        use jsonwebtoken::{EncodingKey, Header, encode};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            workspace_id: Uuid::new_v4().to_string(),
            iat: now,
            nbf: now + 3600,
            exp: now + 7200,
            scope: String::new(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .expect("encode");

        assert_eq!(
            parse_jwt(&token, SECRET).err(),
            Some(ParseError::NotYetValid),
        );
    }

    // AC-8: sub that is not a UUID -> InvalidSubject.
    #[test]
    fn parse_rejects_non_uuid_sub() {
        use jsonwebtoken::{EncodingKey, Header, encode};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = Claims {
            sub: "not-a-uuid".to_string(),
            workspace_id: Uuid::new_v4().to_string(),
            iat: now,
            nbf: now,
            exp: now + 3600,
            scope: String::new(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .expect("encode");

        assert_eq!(
            parse_jwt(&token, SECRET).err(),
            Some(ParseError::InvalidSubject),
        );
    }

    // AC-9: non-JWT input -> MalformedToken.
    #[test]
    fn parse_rejects_malformed_input() {
        assert_eq!(
            parse_jwt("not.a.jwt.at.all", SECRET).err(),
            Some(ParseError::MalformedToken),
        );
        assert_eq!(
            parse_jwt("", SECRET).err(),
            Some(ParseError::MalformedToken),
        );
    }

    // AC-2: alg:none token is rejected.
    //
    // jsonwebtoken 9.x has no `Algorithm::None` variant, so `decode_header` fails to
    // deserialize the `alg` field and returns an error — before our pinning check runs.
    // That maps to `MalformedToken`, not `InvalidAlgorithm`. Both reject the token;
    // the security invariant (alg:none is never accepted) is preserved either way.
    //
    // Header `{"alg":"none","typ":"JWT"}` base64url-no-pad = eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0
    #[test]
    fn parse_rejects_alg_none() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Hardcoded base64url-no-pad of `{"alg":"none","typ":"JWT"}`
        let header_b64 = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0";
        // Payload is irrelevant; we only need a syntactically valid 3-part structure.
        let payload_b64 = "eyJzdWIiOiJ0ZXN0IiwiaWF0IjowfQ";
        let token = format!("{header_b64}.{payload_b64}.");
        let _ = now; // suppress unused warning

        let err = parse_jwt(&token, SECRET).expect_err("alg:none must be rejected");
        assert!(
            matches!(
                err,
                ParseError::MalformedToken | ParseError::InvalidAlgorithm
            ),
            "expected MalformedToken or InvalidAlgorithm, got {err:?}",
        );
    }

    // AC-3: RS256-signed header -> InvalidAlgorithm (before any signature check).
    // Uses the same hardcoded test key as crates/auth/src/verifier.rs to avoid new deps.
    #[test]
    fn parse_rejects_rs256_algorithm() {
        const RS256_PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEpQIBAAKCAQEAri+Y8vpeVdjKVncWklJaSxN6iMfKmFwReYcy31bOv+gcCfHI
8MoaJ/GFzZtUi9ifBJxEfMiz/JMT6GRPcjd+a6JLap+CBH9eY7GTymDsQSlhe4rk
flmjk5fy7Qoj21OVQ5Jj+/82dFazc4Rr9Rsv2fwqfC6j6r3vIQ9lHkOVg9mkYxQh
/fNwrOciXjP6gySwz2JZj0656BEteUN2WwuTODiD+VcI7XyeAA93AC7z2quEZ/qo
GaPZGxOenhxOGnX0PimqqbuVTa5McWenUZa0iNQBL2mr+ypCZwcTvSkbQan1+eeh
4QmRsr/DVHZ7wZnBIXVFvBrlNfhSK+YTnVhNTQIDAQABAoIBAFD2csNNuJdyguoZ
xHYSrUGENkppgzO6Z6zzOKQy9zqgKpg8uEejyPCUBLuC3ZN7Br7f272clHxr6K72
IS9Xt/1TnHZc2dQ5V6hDHAzPbEEFePgxoO9RvwSVdibTtcL2YMTdwHebMrZ0rkid
Xoi5ME7ENQMvsqUjz4iwXTHp/A8iT7wRTxf24vanYgVzpgYNn6aK+t6JmGJci7MR
GZAVVPPe1SGE6AsN2VlL3iXHW1vMtVmaQIiedAvw7dMIoyMUjVUma2Qa4CBkQK5a
oXBT5docKJxoaedCS7WJ8r4ianXNW66j1D6R2GyuEbV2U+NfNIvP39UL2j22Nefu
CZ7oj0kCgYEA42aIaWhV/s9aOV8YcTy7789AxJc8SxYr8RgpZjIERRKjELc4+9vN
huZ9+0fdUMcKEZbIWHLkISdNWcHtJ7Ua873YaLxpMhOyYVP81MRxAMQvfoYV1pZD
fV52kt9NGeoShS3LXI+Hous0+AQ6zvolf3G1myMdrB+oLin+0Jr0cGMCgYEAxBe/
tvXOboqVhuHZENDbtILKOELYmzFjw8C2/hT4IsoRJbzUvS6LPFprYnX3PCcTpeS/
U+EucWPBv+kWtcCS/EBZUhvLENuFWNTQMTR5icPrd4v0oaVGFCY3Ysv3VGvhULpd
j86st5QBViZl6x8y11oGdsV3Ho0fr9jtJfUcQo8CgYEAj8ZYIS0SkhTP2s5BSfc4
bBsBvEpSmLbf+YNpSW/+Ox3Zc8wkfzkt7Uj2BlYdm+D8gLpw2Vtq2Xtb8JAoPZ7H
96vkk/PsHvlNIzRS+sNpHy7rSHfGfvJqoW2EUsBUoznXk9Ssa01kWKGVz+n8tLh7
1OQ0Cm5daGJrlyR+M66FNjkCgYEAiJqexL9SUrGaXv+ArvVAyOyAIVd3/A0ZGepr
0G8dOWcZMPfuH+iHuMCopEvXswDp8Bx9qNpq9zTuaVngpzcDblUJpGiWOyUiLPL8
IfsTXASvSXWnMuCnBCxnUx0SLK6GpS1fNmpc6fpiP/i58WSnj1w4uo7vX8oiM+dj
tZieWkMCgYEAyuN80nDUC+cP0cACzsCVG6DfZy1hSFNSYFka+VTrlqyLDy+a3A2k
wMIYtI9i1qQR3Vd2BrQNviwpShJPgrg6qB5nmi7a0PJS99hOidvwQSniweyhsAKB
0EqrCxP4VKtt560f6yTNKo9pFTv7jzb+U06uR1o+YEGybad8rp7DZ+E=
-----END RSA PRIVATE KEY-----";

        let encoding_key =
            EncodingKey::from_rsa_pem(RS256_PRIVATE_KEY_PEM.as_bytes()).expect("encoding key");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            workspace_id: Uuid::new_v4().to_string(),
            iat: now,
            nbf: now,
            exp: now + 3600,
            scope: String::new(),
        };
        let token =
            encode(&Header::new(Algorithm::RS256), &claims, &encoding_key).expect("rs256 token");

        assert_eq!(
            parse_jwt(&token, SECRET).err(),
            Some(ParseError::InvalidAlgorithm),
        );
    }
}
