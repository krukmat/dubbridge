// S1-T4: storage configuration — preserves MinIO/S3 switchover boundary (ADR-006)
pub use dubbridge_config::StorageBackend;
use dubbridge_config::StorageSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Runtime backend selector.
    pub backend: StorageBackend,
    /// Bucket name (MinIO/S3) or directory name (local).
    pub bucket: String,
    /// Filesystem base path used by LocalFsAdapter.
    pub base_path: String,
    /// Optional S3-compatible endpoint URL (None → AWS S3 defaults).
    pub endpoint_url: Option<String>,
}

impl From<&StorageSettings> for StorageConfig {
    fn from(settings: &StorageSettings) -> Self {
        Self {
            backend: settings.backend.clone(),
            bucket: settings.bucket.clone(),
            base_path: settings.base_path.clone(),
            endpoint_url: settings.endpoint_url.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_config::StorageBackend;

    fn sample_settings() -> StorageSettings {
        StorageSettings {
            backend: StorageBackend::LocalFs,
            base_path: "/var/dubbridge/storage".to_string(),
            bucket: "dubbridge-local".to_string(),
            endpoint_url: Some("http://localhost:9000".to_string()),
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
}
