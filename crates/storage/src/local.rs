// S1-T4: LocalFsAdapter — dev-only implementation of StorageAdapter
use std::path::{Path, PathBuf};

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

    async fn list_keys_from_dir(
        &self,
        root: &Path,
        prefix: &str,
    ) -> Result<Vec<String>, StorageError> {
        let mut keys = Vec::new();
        let mut dirs = vec![root.to_path_buf()];

        while let Some(dir) = dirs.pop() {
            let mut entries = fs::read_dir(&dir).await.map_err(|e| StorageError::Io {
                key: prefix.to_string(),
                source: e,
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| StorageError::Io {
                key: prefix.to_string(),
                source: e,
            })? {
                let path = entry.path();
                let file_type = entry.file_type().await.map_err(|e| StorageError::Io {
                    key: prefix.to_string(),
                    source: e,
                })?;

                if file_type.is_dir() {
                    dirs.push(path);
                    continue;
                }

                if file_type.is_file() {
                    let relative = path.strip_prefix(&self.base_path).map_err(|_| {
                        StorageError::Backend(
                            "local storage enumeration escaped base path".to_string(),
                        )
                    })?;
                    let key = relative.to_string_lossy().replace('\\', "/");
                    if key.starts_with(prefix) {
                        keys.push(key);
                    }
                }
            }
        }

        keys.sort();
        Ok(keys)
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

    async fn put_file(&self, key: &str, source: &Path) -> Result<String, StorageError> {
        let path = self.base_path.join(key);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::Io {
                    key: key.to_string(),
                    source: e,
                })?;
        }

        fs::copy(source, &path)
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

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        if prefix.trim().is_empty() {
            return Ok(Vec::new());
        }

        let root = self.base_path.join(prefix);
        match fs::metadata(&root).await {
            Ok(metadata) if metadata.is_dir() => self.list_keys_from_dir(&root, prefix).await,
            Ok(metadata) if metadata.is_file() => Ok(vec![prefix.to_string()]),
            Ok(_) => Ok(Vec::new()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(StorageError::Io {
                key: prefix.to_string(),
                source: e,
            }),
        }
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

    #[tokio::test]
    async fn put_file_copies_source_contents() {
        let (a, dir) = adapter();
        let source = dir.path().join("source.bin");
        fs::write(&source, b"copied data").await.unwrap();

        let stored_key = a.put_file("assets/test/copied.bin", &source).await.unwrap();
        assert_eq!(stored_key, "assets/test/copied.bin");
        assert_eq!(
            a.get("assets/test/copied.bin").await.unwrap(),
            b"copied data"
        );
    }

    #[tokio::test]
    async fn list_keys_empty_or_missing_prefix_returns_empty() {
        let (a, _dir) = adapter();
        assert_eq!(a.list_keys("").await.unwrap(), Vec::<String>::new());
        assert_eq!(
            a.list_keys("ingests/missing/").await.unwrap(),
            Vec::<String>::new()
        );
    }

    #[tokio::test]
    async fn list_keys_returns_sorted_canonical_ingest_keys() {
        let (a, _dir) = adapter();
        a.put("ingests/token-b/zeta.mp4", vec![1]).await.unwrap();
        a.put("ingests/token-a/alpha.mp4", vec![2]).await.unwrap();
        a.put("assets/other/file.bin", vec![3]).await.unwrap();
        a.put("ingests/token-a/beta.mp4", vec![4]).await.unwrap();

        assert_eq!(
            a.list_keys(crate::INGESTS_PREFIX).await.unwrap(),
            vec![
                "ingests/token-a/alpha.mp4".to_string(),
                "ingests/token-a/beta.mp4".to_string(),
                "ingests/token-b/zeta.mp4".to_string(),
            ]
        );
    }
}
