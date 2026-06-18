use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::{AuthConfig, AuthenticatedPrincipal, ParseError, parse_jwt};

const ACCESS_TOKEN_TYPES: [&str; 2] = ["at+jwt", "application/at+jwt"];

pub trait TokenVerifier {
    fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, TokenVerificationError>;
}

pub struct RsaJwtTokenVerifier {
    config: AuthConfig,
    decoding_key: DecodingKey,
    validation: Validation,
}

#[derive(Debug, Error)]
pub enum VerifierInitError {
    #[error("failed to read RSA public key from {path}: {source}")]
    ReadPublicKey {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("configured RSA public key is invalid")]
    InvalidPublicKey,
    #[error("HMAC verification secret must not be empty")]
    InvalidSecret,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TokenVerificationError {
    #[error("access token is malformed")]
    MalformedToken,
    #[error("access token algorithm is not allowed")]
    InvalidAlgorithm,
    #[error("access token typ is not allowed")]
    InvalidType,
    #[error("access token signature is invalid")]
    InvalidSignature,
    #[error("access token issuer is invalid")]
    InvalidIssuer,
    #[error("access token audience is invalid")]
    InvalidAudience,
    #[error("access token has expired")]
    Expired,
    #[error("access token is not valid yet")]
    NotYetValid,
    #[error("access token subject is invalid")]
    InvalidSubject,
}

impl RsaJwtTokenVerifier {
    pub fn new(config: AuthConfig) -> Result<Self, VerifierInitError> {
        let public_key_pem = fs::read(config.rsa_public_key_path()).map_err(|source| {
            VerifierInitError::ReadPublicKey {
                path: config.rsa_public_key_path().display().to_string(),
                source,
            }
        })?;
        let decoding_key = DecodingKey::from_rsa_pem(&public_key_pem)
            .map_err(|_| VerifierInitError::InvalidPublicKey)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;
        validation.required_spec_claims.clear();

        Ok(Self {
            config,
            decoding_key,
            validation,
        })
    }
}

impl TokenVerifier for RsaJwtTokenVerifier {
    fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, TokenVerificationError> {
        let header = decode_header(token).map_err(|_| TokenVerificationError::MalformedToken)?;

        if header.alg != Algorithm::RS256 {
            return Err(TokenVerificationError::InvalidAlgorithm);
        }

        match header.typ.as_deref() {
            Some(token_type) if ACCESS_TOKEN_TYPES.contains(&token_type) => {}
            _ => return Err(TokenVerificationError::InvalidType),
        }

        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(map_decode_error)?;
        let claims = token_data.claims;

        if claims.iss != self.config.issuer() {
            return Err(TokenVerificationError::InvalidIssuer);
        }

        if !claims.aud.contains(self.config.audience()) {
            return Err(TokenVerificationError::InvalidAudience);
        }

        validate_times(claims.exp, claims.nbf, self.config.clock_skew_leeway())?;

        let subject_id =
            Uuid::parse_str(&claims.sub).map_err(|_| TokenVerificationError::InvalidSubject)?;

        Ok(AuthenticatedPrincipal::new(
            subject_id,
            claims.scope_tokens(),
        ))
    }
}

/// HS256 access-token verifier that implements [`TokenVerifier`] by delegating
/// to [`parse_jwt`] from the in-house issuer module (ADR-031 §Decision 2).
pub struct Hs256TokenVerifier {
    secret: String,
}

impl Hs256TokenVerifier {
    /// Build a verifier. Fails closed when the secret is empty or whitespace.
    pub fn new(secret: &str) -> Result<Self, VerifierInitError> {
        if secret.trim().is_empty() {
            return Err(VerifierInitError::InvalidSecret);
        }
        Ok(Self {
            secret: secret.to_owned(),
        })
    }
}

impl TokenVerifier for Hs256TokenVerifier {
    fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, TokenVerificationError> {
        let claims = parse_jwt(token, &self.secret).map_err(|e| match e {
            ParseError::InvalidAlgorithm => TokenVerificationError::InvalidAlgorithm,
            ParseError::InvalidSignature => TokenVerificationError::InvalidSignature,
            ParseError::Expired => TokenVerificationError::Expired,
            ParseError::NotYetValid => TokenVerificationError::NotYetValid,
            ParseError::InvalidSubject | ParseError::InvalidWorkspace => {
                TokenVerificationError::InvalidSubject
            }
            ParseError::MalformedToken => TokenVerificationError::MalformedToken,
        })?;

        let subject_id =
            Uuid::parse_str(&claims.sub).map_err(|_| TokenVerificationError::InvalidSubject)?;

        Ok(AuthenticatedPrincipal::new(
            subject_id,
            claims.scope.split_whitespace().map(str::to_owned),
        ))
    }
}

fn validate_times(
    exp: u64,
    nbf: Option<u64>,
    leeway: Duration,
) -> Result<(), TokenVerificationError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TokenVerificationError::MalformedToken)?
        .as_secs();
    let leeway = leeway.as_secs();

    if now > exp.saturating_add(leeway) {
        return Err(TokenVerificationError::Expired);
    }

    if let Some(nbf) = nbf
        && now.saturating_add(leeway) < nbf
    {
        return Err(TokenVerificationError::NotYetValid);
    }

    Ok(())
}

fn map_decode_error(error: jsonwebtoken::errors::Error) -> TokenVerificationError {
    use jsonwebtoken::errors::ErrorKind;

    match error.kind() {
        ErrorKind::InvalidSignature => TokenVerificationError::InvalidSignature,
        ErrorKind::InvalidAlgorithm => TokenVerificationError::InvalidAlgorithm,
        ErrorKind::ExpiredSignature => TokenVerificationError::Expired,
        ErrorKind::ImmatureSignature => TokenVerificationError::NotYetValid,
        ErrorKind::InvalidIssuer => TokenVerificationError::InvalidIssuer,
        ErrorKind::InvalidAudience => TokenVerificationError::InvalidAudience,
        ErrorKind::InvalidSubject => TokenVerificationError::InvalidSubject,
        _ => TokenVerificationError::MalformedToken,
    }
}

#[derive(Debug, Deserialize)]
struct Claims {
    iss: String,
    aud: AudienceClaim,
    sub: String,
    exp: u64,
    #[serde(default)]
    nbf: Option<u64>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    scp: Option<ScopeClaim>,
}

impl Claims {
    fn scope_tokens(&self) -> Vec<String> {
        let mut scopes = Vec::new();

        if let Some(scope) = &self.scope {
            scopes.extend(
                scope
                    .split_whitespace()
                    .map(str::trim)
                    .filter(|scope| !scope.is_empty())
                    .map(ToOwned::to_owned),
            );
        }

        if let Some(scope_claim) = &self.scp {
            scopes.extend(scope_claim.tokens());
        }

        scopes
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AudienceClaim {
    One(String),
    Many(Vec<String>),
}

impl AudienceClaim {
    fn contains(&self, expected: &str) -> bool {
        match self {
            Self::One(audience) => audience == expected,
            Self::Many(audiences) => audiences.iter().any(|audience| audience == expected),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ScopeClaim {
    One(String),
    Many(Vec<String>),
}

impl ScopeClaim {
    fn tokens(&self) -> Vec<String> {
        match self {
            Self::One(scopes) => scopes
                .split_whitespace()
                .map(str::trim)
                .filter(|scope| !scope.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
            Self::Many(scopes) => scopes
                .iter()
                .map(|scope| scope.trim())
                .filter(|scope| !scope.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    // S-200-T1a characterization baseline (RS256, pre-S-200).
    // These tests pin the *current* RsaJwtTokenVerifier accept/reject contract so the
    // RS256 -> HS256 inversion in S-200-T1c is provably deliberate, not an accidental
    // regression. The key invariant to be inverted by T1c is asserted in
    // `verify_rejects_algorithm_substitution` (an HS256-signed token is rejected today).
    use std::{
        fs,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde::Serialize;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    use super::*;

    const TEST_PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
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
    const TEST_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAri+Y8vpeVdjKVncWklJa
SxN6iMfKmFwReYcy31bOv+gcCfHI8MoaJ/GFzZtUi9ifBJxEfMiz/JMT6GRPcjd+
a6JLap+CBH9eY7GTymDsQSlhe4rkflmjk5fy7Qoj21OVQ5Jj+/82dFazc4Rr9Rsv
2fwqfC6j6r3vIQ9lHkOVg9mkYxQh/fNwrOciXjP6gySwz2JZj0656BEteUN2WwuT
ODiD+VcI7XyeAA93AC7z2quEZ/qoGaPZGxOenhxOGnX0PimqqbuVTa5McWenUZa0
iNQBL2mr+ypCZwcTvSkbQan1+eeh4QmRsr/DVHZ7wZnBIXVFvBrlNfhSK+YTnVhN
TQIDAQAB
-----END PUBLIC KEY-----";
    const ALT_PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAwGBlWdrZa1+XFyMOJ5ecIWdLCNmjqDiU5Vg2+jGpjmtS1qkW
sa7YTjQeG0/hbzlg2+ldGk76tiuotkYUpzhIrmpSxldrJlcFEDlJYcNVtpoH03Cl
trJh10o+kJgd4k2E5qH/pXCWPKYaP1E/jLzZ0kRIUGW2S0B+IiuHjvviHrFJbCms
am2Hf6wm3XmbCWdGHJuH1e7VykVupqWzL61LbpUEXhKMtieutGHwDQj/agSWpEEn
3IKqS/dRLqwx7pTxukSsdaOW4nyx8uT1QSelBrYa2pQjpcJ9Aqo2pjAqdVkQeSc3
IgCghgJJkPJFzs8OW+UNpBoCdtLoQcABtc5exwIDAQABAoIBAGPLXaggJDtUCh/h
en1FY6PcXotHw2MHfw3+Ff7x9kBAImfirTgdOG5REPEgHhTBkiJiS37TS7Fmso1j
i2E3jFSWKTxkqtvntmO1JAEIAJXKD5c6z2wX2zOAezYtAwubJP8zOFfNMNJjZChG
cI4EhYZTD6RNLySYXxZAuICO370FWo/p2lulkOib3KpJ68sKMukmTQggQ/U1/q4H
qxKEIjihyO1R5O8r4ofgJX7bUvGlUM5psPrGo4HFLcIFixyR03TBsKtLRi66Mc8V
ACLSbEvHH8XV5S59slxSK2scgVQHpmmMl/a5LLCiIK/Yhwc54XpKNDhIkcRAcbxp
4vhQFGkCgYEA3jkbZkdxvjlMR3sNUppetPXCHDv9G3j1b7rtkh9ahoQumbWMaS/G
fT1sgKlHTtBPRkvl4hVWCKwY1WxR4IQ/REUv8ZxaV/6HNHkGsdtrKZSkQBuAHrVD
kNBCmX2wjiRUbTqMUD8E6hCd8+8b57k4nSNGzAJ3wG42SQti1HeI+YsCgYEA3Z3v
PZBMWKlKk1JwkxCJ4aFuCFRCOUN65ynHjrkOQzY6VsFph7ReALG1+AXbn/tVcADt
uNlgneqnErpp8SKNCdrCLKiRI6WKD+GYsPYbL+USzJM5vMDS5EKTSlFVrBxM2vVF
BmmmbImJSDyC1G3JwxB0hVaRbYdZU2K0Mu1MvzUCgYEAsD9GteXwFljHYOH9fQgm
GQvZeh8x7XoP3x+4kG4BlDJ96zcMN9jGakovJhQcFCwu06gamScm5xXnVE3m2lTJ
ANKG5e+Fz8h1X26lmqZV5dKYOqgVA0XsYoxqZeZEA1hZBZCr/HEe6q1nOTLpRO2o
MyjpW6CRbbN7po87QRvVLWUCgYAnaleKk4eAnVtuKFNtVJuxTYzMXnAIzz+krYGY
mME4owRtOakTQbkWVoUOv7v4EDN54DBnmAHfFETyx8Tf5k0/W3D9kF2AAYk0meMW
Vi8vrYZSbDzwnTrk7hJUPXMHUWE58DV+lnvLAgswldKPBZfE4cBXlrX2zQPOGNgD
1sC3oQKBgEXxRGJSGMHtV1Xavy/SVQaLt1QSZ0PKh04aMhTeVx4/avSIiu9H1uyC
kVd/An7O44WxAYGh9pHHhn7YVZ+g3YrcVcyQt9bFgETyYqecCVgQ7fTFychxKTLc
/6N5mOTMZ47GcdXDAOUeHqUf3wHKc3v0iE/z/vtyyGUfOm69UUti
-----END RSA PRIVATE KEY-----";

    #[derive(Clone, Serialize)]
    struct TestClaims<'a> {
        iss: &'a str,
        aud: Vec<&'a str>,
        sub: &'a str,
        exp: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        nbf: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        scp: Option<Vec<&'a str>>,
    }

    fn write_public_key() -> NamedTempFile {
        let key_file = NamedTempFile::new().expect("temp file");
        fs::write(key_file.path(), TEST_PUBLIC_KEY_PEM).expect("write public key");
        key_file
    }

    fn verifier(leeway: Duration) -> RsaJwtTokenVerifier {
        let key_file = write_public_key();
        let config = AuthConfig::new(
            "https://issuer.dubbridge.test",
            "dubbridge-api",
            key_file.path(),
            leeway,
        );
        RsaJwtTokenVerifier::new(config).expect("build verifier")
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_secs()
    }

    fn valid_claims() -> TestClaims<'static> {
        TestClaims {
            iss: "https://issuer.dubbridge.test",
            aud: vec!["dubbridge-api", "other-audience"],
            sub: "550e8400-e29b-41d4-a716-446655440000",
            exp: now_secs() + 300,
            nbf: None,
            scope: Some("assets:ingest assets:read"),
            scp: None,
        }
    }

    fn encode_rs256_token(claims: &TestClaims<'_>, typ: Option<&str>) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.typ = typ.map(str::to_string);

        encode(
            &header,
            claims,
            &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).expect("encoding key"),
        )
        .expect("encode RS256 token")
    }

    #[test]
    fn verify_accepts_valid_token_and_parses_scopes() {
        let verifier = verifier(Duration::from_secs(30));
        let token = encode_rs256_token(&valid_claims(), Some("at+jwt"));

        let principal = verifier.verify_access_token(&token).expect("valid token");

        assert_eq!(
            principal.subject_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid"),
        );
        assert!(principal.has_scope("assets:ingest"));
        assert!(principal.has_scope("assets:read"));
        assert!(!principal.has_scope("recordings:write"));
    }

    #[test]
    fn verify_accepts_application_access_token_typ() {
        let verifier = verifier(Duration::from_secs(30));
        let token = encode_rs256_token(&valid_claims(), Some("application/at+jwt"));

        let principal = verifier.verify_access_token(&token).expect("valid token");

        assert!(principal.has_scope("assets:ingest"));
    }

    #[test]
    fn verify_rejects_invalid_signature() {
        let verifier = verifier(Duration::from_secs(30));
        let mut header = Header::new(Algorithm::RS256);
        header.typ = Some("at+jwt".to_string());
        let token = encode(
            &header,
            &valid_claims(),
            &EncodingKey::from_rsa_pem(ALT_PRIVATE_KEY_PEM.as_bytes()).expect("alt key"),
        )
        .expect("encode token");

        let error = verifier
            .verify_access_token(&token)
            .expect_err("invalid signature");

        assert_eq!(error, TokenVerificationError::InvalidSignature);
    }

    #[test]
    fn verify_rejects_invalid_typ() {
        let verifier = verifier(Duration::from_secs(30));
        let token = encode_rs256_token(&valid_claims(), Some("JWT"));

        let error = verifier
            .verify_access_token(&token)
            .expect_err("invalid typ");

        assert_eq!(error, TokenVerificationError::InvalidType);
    }

    #[test]
    fn verify_rejects_invalid_issuer() {
        let verifier = verifier(Duration::from_secs(30));
        let mut claims = valid_claims();
        claims.iss = "https://wrong-issuer.dubbridge.test";
        let token = encode_rs256_token(&claims, Some("at+jwt"));

        let error = verifier
            .verify_access_token(&token)
            .expect_err("invalid issuer");

        assert_eq!(error, TokenVerificationError::InvalidIssuer);
    }

    #[test]
    fn verify_rejects_invalid_audience() {
        let verifier = verifier(Duration::from_secs(30));
        let mut claims = valid_claims();
        claims.aud = vec!["someone-else"];
        let token = encode_rs256_token(&claims, Some("at+jwt"));

        let error = verifier
            .verify_access_token(&token)
            .expect_err("invalid audience");

        assert_eq!(error, TokenVerificationError::InvalidAudience);
    }

    #[test]
    fn verify_rejects_expired_token() {
        let verifier = verifier(Duration::from_secs(0));
        let mut claims = valid_claims();
        claims.exp = now_secs() - 1;
        let token = encode_rs256_token(&claims, Some("at+jwt"));

        let error = verifier
            .verify_access_token(&token)
            .expect_err("expired token");

        assert_eq!(error, TokenVerificationError::Expired);
    }

    #[test]
    fn verify_rejects_invalid_uuid_subject() {
        let verifier = verifier(Duration::from_secs(30));
        let mut claims = valid_claims();
        claims.sub = "not-a-uuid";
        let token = encode_rs256_token(&claims, Some("at+jwt"));

        let error = verifier
            .verify_access_token(&token)
            .expect_err("invalid subject");

        assert_eq!(error, TokenVerificationError::InvalidSubject);
    }

    #[test]
    fn verify_rejects_future_nbf_beyond_leeway() {
        let verifier = verifier(Duration::from_secs(5));
        let mut claims = valid_claims();
        claims.nbf = Some(now_secs() + 60);
        let token = encode_rs256_token(&claims, Some("at+jwt"));

        let error = verifier
            .verify_access_token(&token)
            .expect_err("future nbf");

        assert_eq!(error, TokenVerificationError::NotYetValid);
    }

    #[test]
    fn verify_accepts_nbf_within_leeway() {
        let verifier = verifier(Duration::from_secs(30));
        let mut claims = valid_claims();
        claims.nbf = Some(now_secs() + 10);
        claims.scp = Some(vec!["recordings:write", "assets:read"]);
        claims.scope = Some("assets:ingest");
        let token = encode_rs256_token(&claims, Some("at+jwt"));

        let principal = verifier
            .verify_access_token(&token)
            .expect("nbf within leeway");

        assert!(principal.has_scope("assets:ingest"));
        assert!(principal.has_scope("assets:read"));
        assert!(principal.has_scope("recordings:write"));
    }

    // T1a baseline: RsaJwtTokenVerifier rejects HS256 (algorithm-substitution).
    // S-200-T1c inverted the system-level invariant: the system now ACCEPTS HS256 and
    // rejects RS256. That is tested in `hs256_verifier_rejects_rs256_algorithm` below.
    // This test remains to pin RsaJwtTokenVerifier's own behavior.
    #[test]
    fn verify_rejects_algorithm_substitution() {
        let verifier = verifier(Duration::from_secs(30));
        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("at+jwt".to_string());
        let token = encode(
            &header,
            &valid_claims(),
            &EncodingKey::from_secret(b"not-an-rsa-key"),
        )
        .expect("encode HS256 token");

        let error = verifier
            .verify_access_token(&token)
            .expect_err("invalid algorithm");

        assert_eq!(error, TokenVerificationError::InvalidAlgorithm);
    }

    // Gap pinned by S-200-T1a: a non-JWT string fails `decode_header` -> MalformedToken,
    // before any algorithm, typ, signature, or claim check.
    #[test]
    fn verify_rejects_malformed_token() {
        let verifier = verifier(Duration::from_secs(30));

        let error = verifier
            .verify_access_token("not-a-jwt-token")
            .expect_err("malformed token");

        assert_eq!(error, TokenVerificationError::MalformedToken);
    }

    // Gap pinned by S-200-T1a: a missing `typ` header (None) is rejected as InvalidType,
    // distinct from the wrong-`typ` branch covered by `verify_rejects_invalid_typ`.
    #[test]
    fn verify_rejects_missing_typ() {
        let verifier = verifier(Duration::from_secs(30));
        let token = encode_rs256_token(&valid_claims(), None);

        let error = verifier
            .verify_access_token(&token)
            .expect_err("missing typ");

        assert_eq!(error, TokenVerificationError::InvalidType);
    }

    // -------------------------------------------------------------------------
    // S-200-T1c-i: Hs256TokenVerifier tests
    // -------------------------------------------------------------------------

    use crate::{Claims as Hs256Claims, Hs256Issuer};

    const HS256_SECRET: &str = "hs256-verifier-test-secret-0123456789";

    fn hs256_verifier() -> Hs256TokenVerifier {
        Hs256TokenVerifier::new(HS256_SECRET).expect("hs256 verifier")
    }

    fn valid_hs256_token() -> String {
        Hs256Issuer::new(HS256_SECRET, Duration::from_secs(3600))
            .expect("issuer")
            .generate_jwt(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid"),
                Uuid::new_v4(),
                &["assets:read".to_string()],
            )
            .expect("token")
    }

    // HP: valid HS256 token → Ok(AuthenticatedPrincipal) with correct subject + scopes.
    #[test]
    fn hs256_verifier_accepts_valid_token() {
        let verifier = hs256_verifier();
        let token = valid_hs256_token();

        let principal = verifier
            .verify_access_token(&token)
            .expect("valid hs256 token");

        assert_eq!(
            principal.subject_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid"),
        );
        assert!(principal.has_scope("assets:read"));
    }

    // AC-1/2: empty and whitespace secret → InvalidSecret (fail-closed).
    #[test]
    fn hs256_verifier_rejects_empty_secret() {
        assert!(matches!(
            Hs256TokenVerifier::new(""),
            Err(VerifierInitError::InvalidSecret),
        ));
        assert!(matches!(
            Hs256TokenVerifier::new("   "),
            Err(VerifierInitError::InvalidSecret),
        ));
    }

    // AC-4 (inversion of T1a): RS256-signed token → InvalidAlgorithm.
    #[test]
    fn hs256_verifier_rejects_rs256_algorithm() {
        let verifier = hs256_verifier();
        let token = encode_rs256_token(&valid_claims(), Some("at+jwt"));

        let error = verifier
            .verify_access_token(&token)
            .expect_err("RS256 rejected by HS256 verifier");

        assert_eq!(error, TokenVerificationError::InvalidAlgorithm);
    }

    // AC-5: alg:none token → MalformedToken or InvalidAlgorithm.
    #[test]
    fn hs256_verifier_rejects_alg_none() {
        let verifier = hs256_verifier();
        let header_b64 = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0";
        let payload_b64 = "eyJzdWIiOiJ0ZXN0IiwiaWF0IjowfQ";
        let token = format!("{header_b64}.{payload_b64}.");

        let err = verifier
            .verify_access_token(&token)
            .expect_err("alg:none rejected");

        assert!(
            matches!(
                err,
                TokenVerificationError::MalformedToken | TokenVerificationError::InvalidAlgorithm
            ),
            "expected MalformedToken or InvalidAlgorithm, got {err:?}",
        );
    }

    // AC-6: wrong secret → InvalidSignature.
    #[test]
    fn hs256_verifier_rejects_wrong_secret() {
        let verifier = Hs256TokenVerifier::new("completely-different-secret").expect("verifier");
        let token = valid_hs256_token();

        let error = verifier
            .verify_access_token(&token)
            .expect_err("wrong secret");

        assert_eq!(error, TokenVerificationError::InvalidSignature);
    }

    // AC-7: expired token (exp > 60s in the past, exceeding default jsonwebtoken leeway).
    #[test]
    fn hs256_verifier_rejects_expired_token() {
        let verifier = hs256_verifier();
        let now = now_secs();
        let claims = Hs256Claims {
            sub: Uuid::new_v4().to_string(),
            workspace_id: Uuid::new_v4().to_string(),
            iat: now - 7200,
            nbf: now - 7200,
            exp: now - 120,
            scope: String::new(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(HS256_SECRET.as_bytes()),
        )
        .expect("encode");

        let error = verifier
            .verify_access_token(&token)
            .expect_err("expired token");

        assert_eq!(error, TokenVerificationError::Expired);
    }

    // AC-8: future nbf (> 60s ahead, exceeding default jsonwebtoken leeway) → NotYetValid.
    #[test]
    fn hs256_verifier_rejects_future_nbf() {
        let verifier = hs256_verifier();
        let now = now_secs();
        let claims = Hs256Claims {
            sub: Uuid::new_v4().to_string(),
            workspace_id: Uuid::new_v4().to_string(),
            iat: now,
            nbf: now + 120,
            exp: now + 3600,
            scope: String::new(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(HS256_SECRET.as_bytes()),
        )
        .expect("encode");

        let error = verifier
            .verify_access_token(&token)
            .expect_err("future nbf");

        assert_eq!(error, TokenVerificationError::NotYetValid);
    }

    // AC-9: non-UUID sub → InvalidSubject.
    #[test]
    fn hs256_verifier_rejects_non_uuid_sub() {
        let verifier = hs256_verifier();
        let now = now_secs();
        let claims = Hs256Claims {
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
            &EncodingKey::from_secret(HS256_SECRET.as_bytes()),
        )
        .expect("encode");

        let error = verifier
            .verify_access_token(&token)
            .expect_err("non-UUID sub");

        assert_eq!(error, TokenVerificationError::InvalidSubject);
    }

    // AC-10: malformed input → MalformedToken.
    #[test]
    fn hs256_verifier_rejects_malformed_input() {
        let verifier = hs256_verifier();

        let error = verifier
            .verify_access_token("not-a-jwt")
            .expect_err("malformed");

        assert_eq!(error, TokenVerificationError::MalformedToken);
    }

    // ParseError::InvalidWorkspace branch: non-UUID workspace_id → InvalidSubject.
    #[test]
    fn hs256_verifier_rejects_non_uuid_workspace_id() {
        let verifier = hs256_verifier();
        let now = now_secs();
        let claims = Hs256Claims {
            sub: Uuid::new_v4().to_string(),
            workspace_id: "not-a-uuid".to_string(),
            iat: now,
            nbf: now,
            exp: now + 3600,
            scope: String::new(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(HS256_SECRET.as_bytes()),
        )
        .expect("encode");

        let error = verifier
            .verify_access_token(&token)
            .expect_err("non-UUID workspace_id");

        assert_eq!(error, TokenVerificationError::InvalidSubject);
    }
}
