// S1-T4: storage configuration — preserves MinIO/S3 switchover boundary (ADR-006)
pub use dubbridge_config::StorageBackend;
use dubbridge_config::StorageSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Runtime backend selector.
    pub backend: StorageBackend,
    /// Bucket name (MinIO/S3) or directory name (local).
    pub bucket: String,
    /// Filesystem base path used by LocalFsAdapter.
    pub base_path: String,
    /// Optional S3-compatible endpoint URL (None → AWS S3 defaults).
    pub endpoint_url: Option<String>,
    /// S3-compatible region (e.g. DigitalOcean Spaces `nyc3`).
    pub region: Option<String>,
    /// Static credential: access key ID. Env-only (ADR-026, Decision 4).
    pub access_key_id: Option<String>,
    /// Static credential: secret access key. Env-only (ADR-026, Decision 4).
    pub secret_access_key: Option<String>,
}

impl From<&StorageSettings> for StorageConfig {
    fn from(settings: &StorageSettings) -> Self {
        Self {
            backend: settings.backend.clone(),
            bucket: settings.bucket.clone(),
            base_path: settings.base_path.clone(),
            endpoint_url: settings.endpoint_url.clone(),
            region: settings.region.clone(),
            access_key_id: settings.access_key_id.clone(),
            secret_access_key: settings.secret_access_key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_config::StorageBackend;

    fn sample_settings() -> StorageSettings {
        StorageSettings {
            base_path: "/var/dubbridge/storage".to_string(),
            bucket: "dubbridge-local".to_string(),
            endpoint_url: Some("http://localhost:9000".to_string()),
            region: Some("nyc3".to_string()),
            access_key_id: Some("test-access-key".to_string()),
            secret_access_key: Some("test-secret-key".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn from_storage_settings_copies_base_path() {
        let settings = sample_settings();
        let cfg = StorageConfig::from(&settings);
        assert_eq!(cfg.base_path, "/var/dubbridge/storage");
    }

    #[test]
    fn from_storage_settings_copies_backend() {
        let mut settings = sample_settings();
        settings.backend = StorageBackend::S3;
        let cfg = StorageConfig::from(&settings);
        assert_eq!(cfg.backend, StorageBackend::S3);
    }

    #[test]
    fn from_storage_settings_copies_bucket() {
        let settings = sample_settings();
        let cfg = StorageConfig::from(&settings);
        assert_eq!(cfg.bucket, "dubbridge-local");
    }

    #[test]
    fn from_storage_settings_copies_endpoint_url() {
        let settings = sample_settings();
        let cfg = StorageConfig::from(&settings);
        assert_eq!(cfg.endpoint_url.as_deref(), Some("http://localhost:9000"));
    }

    #[test]
    fn from_storage_settings_preserves_none_endpoint_url() {
        let mut settings = sample_settings();
        settings.endpoint_url = None;
        let cfg = StorageConfig::from(&settings);
        assert!(cfg.endpoint_url.is_none());
    }

    #[test]
    fn from_storage_settings_copies_region() {
        let settings = sample_settings();
        let cfg = StorageConfig::from(&settings);
        assert_eq!(cfg.region.as_deref(), Some("nyc3"));
    }

    #[test]
    fn from_storage_settings_copies_credentials() {
        let settings = sample_settings();
        let cfg = StorageConfig::from(&settings);
        assert_eq!(cfg.access_key_id.as_deref(), Some("test-access-key"));
        assert_eq!(cfg.secret_access_key.as_deref(), Some("test-secret-key"));
    }

    #[test]
    fn from_storage_settings_preserves_none_credentials() {
        let mut settings = sample_settings();
        settings.access_key_id = None;
        settings.secret_access_key = None;
        let cfg = StorageConfig::from(&settings);
        assert!(cfg.access_key_id.is_none());
        assert!(cfg.secret_access_key.is_none());
    }
}
