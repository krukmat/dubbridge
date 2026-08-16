use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use futures_util::TryStreamExt;
use object_store::{ObjectStore, ObjectStoreExt, aws::AmazonS3Builder, buffered::BufWriter};
use tokio::io::AsyncWriteExt;

use crate::{adapter::StorageAdapter, config::StorageConfig, error::StorageError};

pub struct S3Adapter {
    store: Arc<dyn ObjectStore>,
    bucket: String,
}

impl S3Adapter {
    pub fn new(config: &StorageConfig) -> Result<Self, StorageError> {
        let mut builder = AmazonS3Builder::new().with_bucket_name(&config.bucket);
        if let Some(url) = &config.endpoint_url {
            builder = builder.with_endpoint(url).with_allow_http(true);
        }
        if let Some(region) = &config.region {
            builder = builder.with_region(region);
        }
        if let Some(access_key_id) = &config.access_key_id {
            builder = builder.with_access_key_id(access_key_id);
        }
        if let Some(secret_access_key) = &config.secret_access_key {
            builder = builder.with_secret_access_key(secret_access_key);
        }
        let store = builder
            .build()
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        Ok(Self {
            store: Arc::new(store),
            bucket: config.bucket.clone(),
        })
    }

    #[cfg(test)]
    pub(crate) fn new_for_tests(store: Arc<dyn ObjectStore>, bucket: impl Into<String>) -> Self {
        Self {
            store,
            bucket: bucket.into(),
        }
    }
}

fn map_os_error(key: &str, e: object_store::Error) -> StorageError {
    match e {
        object_store::Error::NotFound { .. } => StorageError::NotFound {
            key: key.to_string(),
        },
        _ => StorageError::Backend(e.to_string()),
    }
}

#[async_trait]
impl StorageAdapter for S3Adapter {
    async fn put(&self, key: &str, bytes: Vec<u8>) -> Result<String, StorageError> {
        let path = object_store::path::Path::from(key);
        self.store
            .put(&path, bytes.into())
            .await
            .map_err(|e| map_os_error(key, e))?;
        Ok(key.to_string())
    }

    async fn put_file(&self, key: &str, source: &Path) -> Result<String, StorageError> {
        let path = object_store::path::Path::from(key);
        let mut file = tokio::fs::File::open(source)
            .await
            .map_err(|e| StorageError::Io {
                key: key.to_string(),
                source: e,
            })?;
        let mut writer = BufWriter::new(Arc::clone(&self.store), path);

        tokio::io::copy(&mut file, &mut writer)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;
        writer
            .shutdown()
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;

        Ok(key.to_string())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = object_store::path::Path::from(key);
        let result = self
            .store
            .get(&path)
            .await
            .map_err(|e| map_os_error(key, e))?;
        result
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| StorageError::Backend(e.to_string()))
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = object_store::path::Path::from(key);
        self.store
            .delete(&path)
            .await
            .map_err(|e| map_os_error(key, e))
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        if prefix.trim().is_empty() {
            return Ok(Vec::new());
        }

        let path = object_store::path::Path::from(prefix);
        let mut stream = self.store.list(Some(&path));
        let mut keys = Vec::new();

        while let Some(meta) = stream
            .try_next()
            .await
            .map_err(|e| map_os_error(prefix, e))?
        {
            let key = meta.location.to_string();
            if key.starts_with(prefix) {
                keys.push(key);
            }
        }

        keys.sort();
        Ok(keys)
    }

    fn object_url(&self, key: &str) -> String {
        format!("s3://{}/{}", self.bucket, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageBackend;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    fn adapter() -> S3Adapter {
        S3Adapter::new_for_tests(
            Arc::new(object_store::memory::InMemory::new()),
            "test-bucket",
        )
    }

    // S-230-T1 HP-1: a real round-trip against an S3-compatible endpoint,
    // exercising S3Adapter::new (not new_for_tests / InMemory) with explicit
    // endpoint/region/static-credential wiring end to end. Requires local
    // MinIO (`infra/local/docker-compose.yml`), which is S3-compatible and
    // exercises the same AmazonS3Builder wiring DigitalOcean Spaces uses.
    fn minio_config_for_test() -> StorageConfig {
        let endpoint_url = std::env::var("DUBBRIDGE_STORAGE_TEST_ENDPOINT").expect(
            "DUBBRIDGE_STORAGE_TEST_ENDPOINT must be set when running ignored S3 integration tests",
        );
        let access_key_id = std::env::var("DUBBRIDGE_STORAGE_TEST_ACCESS_KEY_ID")
            .expect("DUBBRIDGE_STORAGE_TEST_ACCESS_KEY_ID must be set when running ignored S3 integration tests");
        let secret_access_key = std::env::var("DUBBRIDGE_STORAGE_TEST_SECRET_ACCESS_KEY")
            .expect("DUBBRIDGE_STORAGE_TEST_SECRET_ACCESS_KEY must be set when running ignored S3 integration tests");
        let bucket = std::env::var("DUBBRIDGE_STORAGE_TEST_BUCKET")
            .unwrap_or_else(|_| "dubbridge-local".to_string());

        StorageConfig {
            backend: StorageBackend::S3,
            bucket,
            base_path: String::new(),
            endpoint_url: Some(endpoint_url),
            region: Some("us-east-1".to_string()),
            access_key_id: Some(access_key_id),
            secret_access_key: Some(secret_access_key),
        }
    }

    #[tokio::test]
    #[ignore = "requires local MinIO + DUBBRIDGE_STORAGE_TEST_* env vars; run explicitly via qa-test-s3"]
    async fn s3_adapter_new_real_put_get_round_trip_against_s3_compatible_endpoint() {
        let config = minio_config_for_test();
        let adapter = S3Adapter::new(&config)
            .expect("S3Adapter::new with explicit endpoint/region/credentials");

        let key = format!("s-230-t1-roundtrip/{}.bin", uuid::Uuid::new_v4());
        let data = b"dubbridge s-230-t1 real s3-compatible round trip".to_vec();

        let stored_key = adapter.put(&key, data.clone()).await.expect("put");
        assert_eq!(stored_key, key);

        let retrieved = adapter.get(&key).await.expect("get");
        assert_eq!(retrieved, data);

        adapter.delete(&key).await.expect("cleanup delete");
    }

    #[tokio::test]
    async fn put_get_round_trip() {
        let a = adapter();
        let data = b"dubbridge s3 test".to_vec();
        let stored_key = a.put("ingests/abc/file.mp4", data.clone()).await.unwrap();
        assert_eq!(stored_key, "ingests/abc/file.mp4");
        let retrieved = a.get("ingests/abc/file.mp4").await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn delete_then_get_returns_not_found() {
        let a = adapter();
        let key = "ingests/abc/to-delete.mp4";
        a.put(key, vec![1, 2, 3]).await.unwrap();
        a.delete(key).await.unwrap();
        let err = a.get(key).await.unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[tokio::test]
    async fn get_missing_key_returns_not_found() {
        let a = adapter();
        let err = a.get("ingests/missing/file.mp4").await.unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[tokio::test]
    async fn put_file_round_trip() {
        let a = adapter();
        let source = NamedTempFile::new().unwrap();
        let mut file = tokio::fs::File::from_std(source.reopen().unwrap());
        file.write_all(b"streamed to s3").await.unwrap();
        file.flush().await.unwrap();
        drop(file);

        let stored_key = a
            .put_file("ingests/abc/from-file.mp4", source.path())
            .await
            .unwrap();
        assert_eq!(stored_key, "ingests/abc/from-file.mp4");
        assert_eq!(
            a.get("ingests/abc/from-file.mp4").await.unwrap(),
            b"streamed to s3"
        );
    }

    #[test]
    fn object_url_returns_s3_scheme() {
        let a = adapter();
        assert_eq!(
            a.object_url("ingests/abc/file.mp4"),
            "s3://test-bucket/ingests/abc/file.mp4"
        );
    }

    #[tokio::test]
    async fn list_keys_empty_or_missing_prefix_returns_empty() {
        let a = adapter();
        assert_eq!(a.list_keys("").await.unwrap(), Vec::<String>::new());
        assert_eq!(
            a.list_keys("ingests/missing/").await.unwrap(),
            Vec::<String>::new()
        );
    }

    #[tokio::test]
    async fn list_keys_returns_sorted_canonical_ingest_keys() {
        let a = adapter();
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
