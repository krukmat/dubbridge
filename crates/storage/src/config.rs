// S1-T4: storage configuration — preserves MinIO/S3 switchover boundary (ADR-006)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Bucket name (MinIO/S3) or directory name (local).
    pub bucket: String,
    /// Filesystem base path used by LocalFsAdapter.
    pub base_path: String,
    /// Optional S3-compatible endpoint URL (None → AWS S3 defaults).
    pub endpoint_url: Option<String>,
}

impl StorageConfig {
    pub fn from_env(bucket: &str) -> Self {
        Self {
            bucket: bucket.to_string(),
            base_path: std::env::var("DUBBRIDGE_STORAGE_BASE_PATH")
                .unwrap_or_else(|_| "/tmp/dubbridge-storage".to_string()),
            endpoint_url: std::env::var("DUBBRIDGE_STORAGE_ENDPOINT").ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T1-T3: env var isolation via temp_env to avoid thread-safety issues with set_var
    #[test]
    fn from_env_uses_default_base_path_when_var_absent() {
        temp_env::with_var_unset("DUBBRIDGE_STORAGE_BASE_PATH", || {
            let cfg = StorageConfig::from_env("test-bucket");
            assert_eq!(cfg.base_path, "/tmp/dubbridge-storage");
        });
    }

    #[test]
    fn from_env_reads_custom_base_path() {
        temp_env::with_var("DUBBRIDGE_STORAGE_BASE_PATH", Some("/custom/path"), || {
            let cfg = StorageConfig::from_env("test-bucket");
            assert_eq!(cfg.base_path, "/custom/path");
        });
    }

    #[test]
    fn from_env_sets_bucket_from_argument() {
        temp_env::with_var_unset("DUBBRIDGE_STORAGE_BASE_PATH", || {
            let cfg = StorageConfig::from_env("my-bucket");
            assert_eq!(cfg.bucket, "my-bucket");
        });
    }

    #[test]
    fn from_env_endpoint_url_absent_when_var_unset() {
        temp_env::with_var_unset("DUBBRIDGE_STORAGE_ENDPOINT", || {
            let cfg = StorageConfig::from_env("bucket");
            assert!(cfg.endpoint_url.is_none());
        });
    }

    #[test]
    fn from_env_reads_endpoint_url() {
        temp_env::with_var(
            "DUBBRIDGE_STORAGE_ENDPOINT",
            Some("http://localhost:9000"),
            || {
                let cfg = StorageConfig::from_env("bucket");
                assert_eq!(cfg.endpoint_url.unwrap(), "http://localhost:9000");
            },
        );
    }
}
