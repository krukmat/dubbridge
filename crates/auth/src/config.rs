use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    issuer: String,
    audience: String,
    rsa_public_key_path: PathBuf,
    clock_skew_leeway: Duration,
}

impl AuthConfig {
    pub fn new(
        issuer: impl Into<String>,
        audience: impl Into<String>,
        rsa_public_key_path: impl Into<PathBuf>,
        clock_skew_leeway: Duration,
    ) -> Self {
        Self {
            issuer: issuer.into(),
            audience: audience.into(),
            rsa_public_key_path: rsa_public_key_path.into(),
            clock_skew_leeway,
        }
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn rsa_public_key_path(&self) -> &Path {
        &self.rsa_public_key_path
    }

    pub fn clock_skew_leeway(&self) -> Duration {
        self.clock_skew_leeway
    }
}
