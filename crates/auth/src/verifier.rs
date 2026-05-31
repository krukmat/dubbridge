use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::{AuthConfig, AuthenticatedPrincipal};

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

    if let Some(nbf) = nbf {
        if now.saturating_add(leeway) < nbf {
            return Err(TokenVerificationError::NotYetValid);
        }
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
}
