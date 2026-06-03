// S3-P1: platform connector trait boundary (ADR-025).
// Pure request-builder / IO-executor split — no DB dependency.

use std::future::Future;
use std::path::Path;

use thiserror::Error;

// Platform and SourceRef are domain types — import from domain to avoid circular dependency.
pub use dubbridge_domain::platform_ingest::{Platform, SourceRef};

/// Opaque reference to owner credentials stored in a secrets store (ADR-025).
/// The real secret lives in the secrets store; this reference is never logged.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectorCredential {
    /// Reference key into the secrets store — not the secret itself.
    pub credential_ref: String,
}

/// Manual Debug impl — never exposes the credential_ref value in logs (ADR-025, ADR-018).
impl std::fmt::Debug for ConnectorCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectorCredential")
            .field("credential_ref", &"[redacted]")
            .finish()
    }
}

/// Metadata resolved from the platform before downloading (ADR-025 resolve step).
#[derive(Debug, Clone)]
pub struct RemoteMediaMetadata {
    pub title: String,
    pub duration_secs: Option<u64>,
    pub platform: Platform,
    pub source_ref: SourceRef,
}

/// Result of a completed download to local staging storage (ADR-025 download step).
#[derive(Debug)]
pub struct DownloadedMedia {
    pub staging_path: std::path::PathBuf,
    pub size_bytes: u64,
    pub content_type: String,
}

/// Errors produced by connector operations (ADR-025).
#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("unauthorized: credential does not grant access to this resource")]
    Unauthorized,
    #[error("not found: resource does not exist on the platform")]
    NotFound,
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("credential error: {0}")]
    CredentialError(String),
}

/// Connector contract per ADR-025: pure request-builder / IO-executor split.
///
/// Implementations MUST NOT depend on `crates/db`. Only `crates/domain` and
/// `crates/config` are permitted as internal crate dependencies.
pub trait PlatformConnector {
    /// The platform this connector handles.
    fn platform(&self) -> Platform;

    /// Resolves ownership and metadata for the given source without transferring bytes.
    /// Returns `Unauthorized` if the credential does not grant access (ADR-025 fail-closed).
    fn resolve(
        &self,
        source: &SourceRef,
        cred: &ConnectorCredential,
    ) -> impl Future<Output = Result<RemoteMediaMetadata, ConnectorError>> + Send;

    /// Downloads the owner-authorized media to `dest` (ADR-025 download step).
    /// Credential is stored by reference and never logged (ADR-025, ADR-018).
    fn download(
        &self,
        source: &SourceRef,
        cred: &ConnectorCredential,
        dest: &Path,
    ) -> impl Future<Output = Result<DownloadedMedia, ConnectorError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_display_youtube() {
        assert_eq!(Platform::YouTube.to_string(), "youtube");
    }

    #[test]
    fn connector_credential_debug_redacts_value() {
        let cred = ConnectorCredential {
            credential_ref: "super-secret-oauth-token".to_string(),
        };
        let debug_output = format!("{cred:?}");
        assert!(debug_output.contains("[redacted]"));
        assert!(!debug_output.contains("super-secret-oauth-token"));
    }

    #[test]
    fn connector_error_display_variants() {
        assert!(
            ConnectorError::Unauthorized
                .to_string()
                .contains("unauthorized")
        );
        assert!(ConnectorError::NotFound.to_string().contains("not found"));
        assert!(
            ConnectorError::UnsupportedPlatform("vimeo".to_string())
                .to_string()
                .contains("vimeo")
        );
        assert!(
            ConnectorError::NetworkError("timeout".to_string())
                .to_string()
                .contains("timeout")
        );
        assert!(
            ConnectorError::CredentialError("expired".to_string())
                .to_string()
                .contains("expired")
        );
    }
}
