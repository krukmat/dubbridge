mod axum;
mod config;
mod credentials;
mod issuer;
mod membership;
mod principal;
mod service;
mod verifier;

pub use self::axum::{SharedTokenVerifier, authenticate_bearer, require_scope};
pub use config::AuthConfig;
pub use credentials::{
    BCRYPT_COST, CredentialError, MIN_PASSWORD_CHARS, hash_password, normalize_required,
    validate_password_for_register, verify_password,
};
pub use issuer::{Claims, Hs256Issuer, IssueError, ParseError, parse_jwt};
pub use membership::OrgMemberPrincipal;
pub use principal::AuthenticatedPrincipal;
pub use service::{
    AccessTokenIssuer, AccountStore, AuthService, AuthServiceError, AuthSuccess,
    DEFAULT_AUTH_SCOPES, PgAccountStore,
};
pub use verifier::{
    Hs256TokenVerifier, RsaJwtTokenVerifier, TokenVerificationError, TokenVerifier,
    VerifierInitError,
};
