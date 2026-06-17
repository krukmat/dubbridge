// S1-T4: StorageAdapter trait — object-safe async boundary (ADR-006)
use std::path::Path;

use async_trait::async_trait;

use crate::error::StorageError;

/// Object-safe async storage boundary. Implementations: LocalFsAdapter (dev),
/// MinIO/S3 adapter (production, S2). Switch is controlled by StorageConfig.
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<String, StorageError>;
    async fn put_file(&self, key: &str, path: &Path) -> Result<String, StorageError>;
    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
    /// Reconciliation-friendly listing seam that returns canonical storage keys only.
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StorageError>;
    /// Synchronous — returns a stable addressable URL for the stored object.
    fn object_url(&self, key: &str) -> String;
}
