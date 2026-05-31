// S1-T4: LocalFsAdapter — dev-only implementation of StorageAdapter
use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::adapter::StorageAdapter;
use crate::error::StorageError;

pub struct LocalFsAdapter {
    base_path: PathBuf,
}

impl LocalFsAdapter {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

#[async_trait]
impl StorageAdapter for LocalFsAdapter {
    async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<String, StorageError> {
        let path = self.base_path.join(key);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::Io {
                    key: key.to_string(),
                    source: e,
                })?;
        }

        fs::write(&path, &bytes)
            .await
            .map_err(|e| StorageError::Io {
                key: key.to_string(),
                source: e,
            })?;

        Ok(key.to_string())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.base_path.join(key);

        fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    key: key.to_string(),
                }
            } else {
                StorageError::Io {
                    key: key.to_string(),
                    source: e,
                }
            }
        })
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.base_path.join(key);

        fs::remove_file(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    key: key.to_string(),
                }
            } else {
                StorageError::Io {
                    key: key.to_string(),
                    source: e,
                }
            }
        })
    }

    fn object_url(&self, key: &str) -> String {
        let path = self.base_path.join(key);
        format!("file://{}", path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn adapter() -> (LocalFsAdapter, TempDir) {
        let dir = TempDir::new().unwrap();
        (LocalFsAdapter::new(dir.path()), dir)
    }

    #[tokio::test]
    async fn put_get_round_trip() {
        let (a, _dir) = adapter();
        let key = "assets/test/file.mp4";
        let data = b"hello dubbridge".to_vec();

        let stored_key = a.put(key, data.clone()).await.unwrap();
        assert_eq!(stored_key, key);

        let retrieved = a.get(key).await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn put_creates_parent_dirs() {
        let (a, _dir) = adapter();
        let key = "deep/nested/dir/file.bin";
        a.put(key, vec![1, 2, 3]).await.unwrap();
        assert!(a.get(key).await.is_ok());
    }

    #[tokio::test]
    async fn get_missing_returns_not_found() {
        let (a, _dir) = adapter();
        let err = a.get("missing/key.bin").await.unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[tokio::test]
    async fn delete_removes_object() {
        let (a, _dir) = adapter();
        let key = "to/delete.bin";
        a.put(key, vec![9]).await.unwrap();
        a.delete(key).await.unwrap();
        assert!(matches!(
            a.get(key).await.unwrap_err(),
            StorageError::NotFound { .. }
        ));
    }

    #[tokio::test]
    async fn object_url_returns_file_url() {
        let (a, dir) = adapter();
        let url = a.object_url("assets/123/file.mp4");
        assert!(url.starts_with("file://"));
        assert!(url.contains(dir.path().to_str().unwrap()));
        assert!(url.ends_with("assets/123/file.mp4"));
    }
}
