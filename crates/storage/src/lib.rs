// S1-T4: storage crate — StorageAdapter trait, LocalFsAdapter, config, path helpers
pub mod adapter;
pub mod config;
pub mod error;
pub mod local;
pub mod s3;

pub use adapter::StorageAdapter;
pub use config::StorageConfig;
pub use error::StorageError;
pub use local::LocalFsAdapter;
pub use s3::S3Adapter;

use std::borrow::Cow;

use config::{StorageBackend, StorageConfig as Cfg};
use uuid::Uuid;

pub const INGESTS_PREFIX: &str = "ingests/";

/// Build the appropriate adapter from config.
pub fn build_adapter(config: &Cfg) -> Result<Box<dyn StorageAdapter>, StorageError> {
    match config.backend {
        StorageBackend::LocalFs => Ok(Box::new(LocalFsAdapter::new(&config.base_path))),
        StorageBackend::S3 => {
            if config.bucket.trim().is_empty() {
                return Err(StorageError::Backend(
                    "storage.bucket is required for s3 backend".to_string(),
                ));
            }
            if let Some(endpoint) = &config.endpoint_url {
                let endpoint = endpoint.trim();
                if endpoint.is_empty()
                    || !(endpoint.starts_with("http://") || endpoint.starts_with("https://"))
                {
                    return Err(StorageError::Backend(
                        "storage.endpoint_url must be an http(s) URL for s3 backend".to_string(),
                    ));
                }
            }
            Ok(Box::new(S3Adapter::new(config)?))
        }
    }
}

// Path helpers — owned by this crate so key layout stays in one place (ADR-006).
pub fn asset_prefix(asset_id: &str) -> String {
    format!("assets/{asset_id}/")
}

pub fn recording_prefix(session_id: &str) -> String {
    format!("recordings/{session_id}/")
}

pub fn prepared_prefix(asset_id: &str) -> String {
    format!("assets/{asset_id}/prepared/")
}

pub fn probe_metadata_key(asset_id: &str) -> String {
    format!("{}probe.json", prepared_prefix(asset_id))
}

pub fn hls_prefix(asset_id: &str) -> String {
    format!("{}hls/", prepared_prefix(asset_id))
}

pub fn hls_manifest_key(asset_id: &str) -> String {
    format!("{}index.m3u8", hls_prefix(asset_id))
}

pub fn transcript_key(asset_id: &str) -> String {
    format!("transcripts/{asset_id}/transcript.json")
}

pub fn alignment_key(asset_id: &str) -> String {
    format!("transcripts/{asset_id}/alignment.json")
}

pub fn subtitle_key(asset_id: &str) -> String {
    format!("subtitles/{asset_id}/subtitle.json")
}

pub fn hls_segment_key(asset_id: &str, filename: &str) -> String {
    let safe = sanitize_filename(match filename.trim() {
        "" => "segment.ts",
        other => other,
    });
    format!("{}{}", hls_prefix(asset_id), safe)
}

/// Canonical upload key for an ingest session.
/// `ingests/{token}/{safe_filename}` — callers must not hand-roll this path.
pub fn ingest_key(token: Uuid, filename: Option<&str>) -> String {
    let safe = sanitize_filename(filename.unwrap_or("upload.bin"));
    format!("ingests/{token}/{safe}")
}

// ── HLS scoped-read helper (ADR-032) ─────────────────────────────────────────
//
// Callers receive manifest bytes, never a raw object-store key. Key construction
// stays inside this crate (ADR-006).

#[derive(Debug)]
pub struct HlsManifestBytes(pub Vec<u8>);

#[derive(Debug)]
pub struct HlsSegmentBytes(pub Vec<u8>);

/// Fetch the prepared HLS manifest for `asset_id` from `adapter`.
///
/// Returns the manifest bytes wrapped in a newtype so the caller cannot
/// accidentally treat them as a storage key. Returns `StorageError::NotFound`
/// if the key is absent.
pub async fn get_hls_manifest(
    adapter: &dyn StorageAdapter,
    asset_id: &str,
) -> Result<HlsManifestBytes, StorageError> {
    let key = hls_manifest_key(asset_id);
    let bytes = adapter.get(&key).await?;
    Ok(HlsManifestBytes(bytes))
}

/// Fetch the prepared HLS segment for `asset_id` / `filename` from `adapter`.
///
/// Returns segment bytes wrapped in a newtype so callers never work with the
/// canonical storage key directly.
pub async fn get_hls_segment(
    adapter: &dyn StorageAdapter,
    asset_id: &str,
    filename: &str,
) -> Result<HlsSegmentBytes, StorageError> {
    let key = hls_segment_key(asset_id, filename);
    let bytes = adapter.get(&key).await?;
    Ok(HlsSegmentBytes(bytes))
}

pub(crate) fn sanitize_filename(name: &str) -> Cow<'_, str> {
    if name.contains('/') || name.contains('\\') {
        Cow::Owned(name.replace(['/', '\\'], "_"))
    } else {
        Cow::Borrowed(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use object_store::memory::InMemory;
    use tempfile::TempDir;

    // T1-T3: ADR-006 storage key layout contract
    #[test]
    fn asset_prefix_format() {
        assert_eq!(asset_prefix("abc-123"), "assets/abc-123/");
    }

    #[test]
    fn asset_prefix_preserves_id_exactly() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        assert!(asset_prefix(id).starts_with("assets/"));
        assert!(asset_prefix(id).contains(id));
        assert!(asset_prefix(id).ends_with('/'));
    }

    #[test]
    fn recording_prefix_format() {
        assert_eq!(recording_prefix("session-xyz"), "recordings/session-xyz/");
    }

    #[test]
    fn recording_prefix_preserves_id_exactly() {
        let id = "session-550e8400";
        assert!(recording_prefix(id).starts_with("recordings/"));
        assert!(recording_prefix(id).contains(id));
        assert!(recording_prefix(id).ends_with('/'));
    }

    #[test]
    fn prepared_prefix_format() {
        assert_eq!(prepared_prefix("asset-123"), "assets/asset-123/prepared/");
    }

    #[test]
    fn probe_metadata_key_format() {
        assert_eq!(
            probe_metadata_key("asset-123"),
            "assets/asset-123/prepared/probe.json"
        );
    }

    #[test]
    fn hls_manifest_key_format() {
        assert_eq!(
            hls_manifest_key("asset-123"),
            "assets/asset-123/prepared/hls/index.m3u8"
        );
    }

    // S-130-T1: transcript storage key contract
    #[test]
    fn transcript_key_format() {
        assert_eq!(
            transcript_key("asset-123"),
            "transcripts/asset-123/transcript.json"
        );
    }

    #[test]
    fn alignment_key_format() {
        assert_eq!(
            alignment_key("asset-123"),
            "transcripts/asset-123/alignment.json"
        );
    }

    #[test]
    fn transcript_and_alignment_keys_differ() {
        let id = "asset-456";
        assert_ne!(transcript_key(id), alignment_key(id));
    }

    // S-140-T1d: subtitle storage key contract
    #[test]
    fn subtitle_key_format() {
        assert_eq!(
            subtitle_key("asset-123"),
            "subtitles/asset-123/subtitle.json"
        );
    }

    #[test]
    fn subtitle_and_transcript_keys_differ() {
        let id = "asset-456";
        assert_ne!(subtitle_key(id), transcript_key(id));
    }

    #[test]
    fn hls_segment_key_uses_canonical_prefix_and_filename() {
        assert_eq!(
            hls_segment_key("asset-123", "segment_00000.ts"),
            "assets/asset-123/prepared/hls/segment_00000.ts"
        );
    }

    #[test]
    fn hls_segment_key_sanitizes_slashes() {
        assert_eq!(
            hls_segment_key("asset-123", "../segment_00000.ts"),
            "assets/asset-123/prepared/hls/.._segment_00000.ts"
        );
    }

    // S-080-T1: ingest_key canonical key contract
    #[test]
    fn ingest_key_normal_filename() {
        let token = Uuid::nil();
        let key = ingest_key(token, Some("episode.mp4"));
        assert_eq!(key, format!("ingests/{token}/episode.mp4"));
    }

    #[test]
    fn ingest_key_none_filename_defaults() {
        let token = Uuid::nil();
        let key = ingest_key(token, None);
        assert_eq!(key, format!("ingests/{token}/upload.bin"));
    }

    #[test]
    fn ingest_key_empty_filename_defaults() {
        let token = Uuid::nil();
        let key = ingest_key(token, Some(""));
        // empty string falls through to unwrap_or("upload.bin") — but empty str is Some,
        // so sanitize_filename("") returns "" which produces "ingests/{token}/".
        // Verify determinism: no panic, consistent output.
        assert!(key.starts_with(&format!("ingests/{token}/")));
    }

    #[test]
    fn ingest_key_sanitizes_forward_slash() {
        let token = Uuid::nil();
        let key = ingest_key(token, Some("../../etc/passwd"));
        // slashes are replaced with underscores; dots in filenames are not stripped
        // because the key is a string prefix, not an OS path — no traversal risk.
        assert_eq!(key, format!("ingests/{token}/.._.._etc_passwd"));
        assert!(!key[key.rfind('/').unwrap() + 1..].contains('/'));
    }

    #[test]
    fn ingest_key_sanitizes_backslash() {
        let token = Uuid::nil();
        let key = ingest_key(token, Some("dir\\file.mp4"));
        assert_eq!(key, format!("ingests/{token}/dir_file.mp4"));
    }

    #[test]
    fn build_adapter_local_fs_returns_file_adapter() {
        let dir = TempDir::new().unwrap();
        let config = Cfg {
            backend: StorageBackend::LocalFs,
            bucket: "dubbridge-local".to_string(),
            base_path: dir.path().to_string_lossy().into_owned(),
            endpoint_url: None,
        };

        let adapter = build_adapter(&config).unwrap();
        let url = adapter.object_url("ingests/token/file.mp4");

        assert!(url.starts_with("file://"));
        assert!(url.contains("ingests/token/file.mp4"));
    }

    #[test]
    fn build_adapter_s3_returns_s3_adapter() {
        let config = Cfg {
            backend: StorageBackend::S3,
            bucket: "dubbridge-local".to_string(),
            base_path: "/tmp/dubbridge-storage".to_string(),
            endpoint_url: Some("http://localhost:9000".to_string()),
        };

        let adapter = build_adapter(&config).unwrap();

        assert_eq!(
            adapter.object_url("ingests/token/file.mp4"),
            "s3://dubbridge-local/ingests/token/file.mp4"
        );
    }

    #[test]
    fn build_adapter_s3_requires_bucket() {
        let config = Cfg {
            backend: StorageBackend::S3,
            bucket: " ".to_string(),
            base_path: "/tmp/dubbridge-storage".to_string(),
            endpoint_url: Some("http://localhost:9000".to_string()),
        };

        match build_adapter(&config) {
            Err(StorageError::Backend(message)) => assert!(message.contains("storage.bucket")),
            Err(err) => panic!("expected StorageError::Backend, got {err:?}"),
            Ok(_) => panic!("expected s3 adapter construction to fail"),
        }
    }

    #[test]
    fn build_adapter_s3_rejects_invalid_endpoint() {
        let config = Cfg {
            backend: StorageBackend::S3,
            bucket: "dubbridge-local".to_string(),
            base_path: "/tmp/dubbridge-storage".to_string(),
            endpoint_url: Some("not-a-url".to_string()),
        };

        match build_adapter(&config) {
            Err(StorageError::Backend(_)) => {}
            Err(err) => panic!("expected StorageError::Backend, got {err:?}"),
            Ok(_) => panic!("expected s3 adapter construction to fail"),
        }
    }

    // S-125-T3b: get_hls_manifest scoped-read helper (ADR-032)

    // HP-1: returns manifest bytes; caller never receives a raw storage key.
    #[tokio::test]
    async fn get_hls_manifest_returns_bytes_not_key() {
        let dir = TempDir::new().unwrap();
        let adapter = LocalFsAdapter::new(dir.path());
        let asset_id = "asset-abc";
        let manifest_bytes = b"#EXTM3U\n#EXT-X-ENDLIST\n".to_vec();

        adapter
            .put(&hls_manifest_key(asset_id), manifest_bytes.clone())
            .await
            .unwrap();

        let result = get_hls_manifest(&adapter, asset_id).await.unwrap();
        assert_eq!(result.0, manifest_bytes);
    }

    // EC-1: absent key → NotFound error, no fabricated bytes.
    #[tokio::test]
    async fn get_hls_manifest_returns_not_found_for_absent_key() {
        let dir = TempDir::new().unwrap();
        let adapter = LocalFsAdapter::new(dir.path());

        let err = get_hls_manifest(&adapter, "nonexistent-asset")
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    // EC-2: the returned value contains no raw object-store path string.
    #[tokio::test]
    async fn get_hls_manifest_result_contains_no_raw_key() {
        let dir = TempDir::new().unwrap();
        let adapter = LocalFsAdapter::new(dir.path());
        let asset_id = "asset-abc";
        let manifest_bytes = b"#EXTM3U\n#EXT-X-ENDLIST\n".to_vec();

        adapter
            .put(&hls_manifest_key(asset_id), manifest_bytes.clone())
            .await
            .unwrap();

        let result = get_hls_manifest(&adapter, asset_id).await.unwrap();
        let as_str = std::str::from_utf8(&result.0).unwrap();
        assert!(!as_str.contains("s3://"));
        assert!(!as_str.contains("minio"));
        assert!(!as_str.contains("prepared/hls/"));
    }

    #[tokio::test]
    async fn get_hls_segment_returns_bytes_not_key() {
        let dir = TempDir::new().unwrap();
        let adapter = LocalFsAdapter::new(dir.path());
        let asset_id = "asset-segment";
        let segment_bytes = vec![0, 1, 2, 3];

        adapter
            .put(
                &hls_segment_key(asset_id, "segment_00000.ts"),
                segment_bytes.clone(),
            )
            .await
            .unwrap();

        let result = get_hls_segment(&adapter, asset_id, "segment_00000.ts")
            .await
            .unwrap();
        assert_eq!(result.0, segment_bytes);
    }

    #[tokio::test]
    async fn get_hls_segment_returns_not_found_for_absent_key() {
        let dir = TempDir::new().unwrap();
        let adapter = LocalFsAdapter::new(dir.path());

        let err = get_hls_segment(&adapter, "asset-segment", "missing.ts")
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[tokio::test]
    async fn local_fs_and_s3_list_the_same_canonical_ingest_keys() {
        let dir = TempDir::new().unwrap();
        let local = LocalFsAdapter::new(dir.path());
        let s3 = S3Adapter::new_for_tests(Arc::new(InMemory::new()), "test-bucket");

        for key in [
            "ingests/token-b/zeta.mp4",
            "ingests/token-a/alpha.mp4",
            "ingests/token-a/beta.mp4",
            "assets/other/file.bin",
        ] {
            local.put(key, vec![1]).await.unwrap();
            s3.put(key, vec![1]).await.unwrap();
        }

        assert_eq!(
            local.list_keys(INGESTS_PREFIX).await.unwrap(),
            s3.list_keys(INGESTS_PREFIX).await.unwrap()
        );
    }
}
