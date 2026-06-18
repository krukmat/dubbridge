use bcrypt::{hash, verify};
use thiserror::Error;

pub const BCRYPT_COST: u32 = 12;
pub const MIN_PASSWORD_CHARS: usize = 12;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CredentialError {
    #[error("{field} is required")]
    MissingField { field: &'static str },
    #[error("password must be at least {min_len} characters")]
    PasswordTooShort { min_len: usize },
    #[error("failed to hash password")]
    Hashing,
    #[error("stored password hash is invalid")]
    InvalidStoredHash,
}

pub fn normalize_required(value: &str, field: &'static str) -> Result<String, CredentialError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CredentialError::MissingField { field });
    }
    Ok(trimmed.to_string())
}

pub fn validate_password_for_register(password: &str) -> Result<(), CredentialError> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(CredentialError::PasswordTooShort {
            min_len: MIN_PASSWORD_CHARS,
        });
    }
    Ok(())
}

pub fn hash_password(password: &str) -> Result<String, CredentialError> {
    hash(password, BCRYPT_COST).map_err(|_| CredentialError::Hashing)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, CredentialError> {
    verify(password, password_hash).map_err(|_| CredentialError::InvalidStoredHash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bcrypt::HashParts;
    use std::str::FromStr;

    #[test]
    fn hash_password_uses_configured_bcrypt_cost() {
        let password_hash = hash_password("correct horse battery staple").expect("hash");
        let parts = HashParts::from_str(&password_hash).expect("hash parts");

        assert_eq!(parts.get_cost(), BCRYPT_COST);
    }

    #[test]
    fn hash_password_round_trips_with_verify_password() {
        let password_hash = hash_password("correct horse battery staple").expect("hash");

        assert!(verify_password("correct horse battery staple", &password_hash).expect("verify"));
    }

    #[test]
    fn verify_password_returns_false_for_wrong_password() {
        let password_hash = hash_password("correct horse battery staple").expect("hash");

        assert!(!verify_password("wrong password", &password_hash).expect("verify"));
    }

    #[test]
    fn verify_password_rejects_invalid_hash() {
        let error = verify_password("password", "not-a-bcrypt-hash").unwrap_err();

        assert_eq!(error, CredentialError::InvalidStoredHash);
    }

    #[test]
    fn normalize_required_trims_and_accepts_non_empty_values() {
        assert_eq!(
            normalize_required("  owner@example.com  ", "email").expect("value"),
            "owner@example.com"
        );
    }

    #[test]
    fn normalize_required_rejects_empty_values() {
        let error = normalize_required("   ", "email").unwrap_err();

        assert_eq!(error, CredentialError::MissingField { field: "email" });
    }

    #[test]
    fn validate_password_for_register_accepts_minimum_length() {
        validate_password_for_register("abcdefghijkl").expect("valid password");
    }

    #[test]
    fn validate_password_for_register_rejects_short_password() {
        let error = validate_password_for_register("short").unwrap_err();

        assert_eq!(
            error,
            CredentialError::PasswordTooShort {
                min_len: MIN_PASSWORD_CHARS
            }
        );
    }
}
