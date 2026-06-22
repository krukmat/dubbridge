// S-125-T3b-i: crates/playback scaffold (ADR-032).
use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ManifestRewriteError {
    #[error("manifest is missing the #EXTM3U header")]
    MissingHeader,
    #[error("segment reference missing for {0}")]
    MissingSegmentReference(String),
}

/// Rewrite every media-segment URI in a prepared `.m3u8` so it points at
/// `routed_base` instead of the raw object-store key prefix.
/// Non-segment tag lines pass through unchanged; order is preserved.
/// Returns `Err(MissingHeader)` if the input does not start with `#EXTM3U`.
pub fn rewrite_manifest(manifest: &str, routed_base: &str) -> Result<String, ManifestRewriteError> {
    let base = routed_base.trim_end_matches('/');
    rewrite_manifest_with_refs(
        manifest,
        &manifest
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(|line| {
                let trimmed = line.trim();
                let name = trimmed.rsplit('/').next().unwrap_or(trimmed).to_string();
                (name.clone(), format!("{base}/{name}"))
            })
            .collect(),
    )
}

/// Rewrite every media-segment URI using the exact per-segment references supplied
/// in `segment_refs`, keyed by segment filename.
pub fn rewrite_manifest_with_refs(
    manifest: &str,
    segment_refs: &HashMap<String, String>,
) -> Result<String, ManifestRewriteError> {
    let mut lines = manifest.lines().peekable();
    match lines.peek() {
        Some(first) if first.trim() == "#EXTM3U" => {}
        _ => return Err(ManifestRewriteError::MissingHeader),
    }
    let mut out = String::with_capacity(manifest.len());
    for line in lines {
        if line.starts_with('#') || line.trim().is_empty() {
            out.push_str(line);
        } else {
            let trimmed = line.trim();
            let name = trimmed.rsplit('/').next().unwrap_or(trimmed);
            let reference = segment_refs
                .get(name)
                .ok_or_else(|| ManifestRewriteError::MissingSegmentReference(name.to_string()))?;
            out.push_str(reference);
        }
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:6\n#EXT-X-PLAYLIST-TYPE:VOD\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n#EXTINF:6.0,\nprepared/asset-123/segment_00001.ts\n#EXT-X-ENDLIST\n";
    const ROUTED_BASE: &str = "https://cdn.example.com/play/abc";

    #[test]
    fn rewrites_segment_uris_to_routed_base() {
        let out = rewrite_manifest(VALID_MANIFEST, ROUTED_BASE).unwrap();
        assert!(out.contains("https://cdn.example.com/play/abc/segment_00000.ts"));
        assert!(out.contains("https://cdn.example.com/play/abc/segment_00001.ts"));
        assert!(out.contains("#EXTM3U"));
        assert!(out.contains("#EXTINF:6.0,"));
        assert!(out.contains("#EXT-X-ENDLIST"));
    }

    #[test]
    fn all_segments_rewritten_in_order() {
        let out = rewrite_manifest(VALID_MANIFEST, ROUTED_BASE).unwrap();
        let segs: Vec<&str> = out
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .collect();
        assert_eq!(segs.len(), 2);
        assert!(segs[0].ends_with("segment_00000.ts"));
        assert!(segs[1].ends_with("segment_00001.ts"));
    }

    #[test]
    fn missing_extm3u_returns_error() {
        let err = rewrite_manifest("#EXT-X-VERSION:3\nsegment.ts\n", ROUTED_BASE).unwrap_err();
        assert_eq!(err, ManifestRewriteError::MissingHeader);
    }

    #[test]
    fn empty_input_returns_error() {
        let err = rewrite_manifest("", ROUTED_BASE).unwrap_err();
        assert_eq!(err, ManifestRewriteError::MissingHeader);
    }

    #[test]
    fn output_contains_no_raw_storage_key() {
        let out = rewrite_manifest(VALID_MANIFEST, ROUTED_BASE).unwrap();
        assert!(!out.contains("s3://"));
        assert!(!out.contains("minio"));
        assert!(!out.contains("prepared/asset-123/segment"));
    }

    #[test]
    fn trailing_slash_on_base_is_normalised() {
        let out = rewrite_manifest(VALID_MANIFEST, "https://cdn.example.com/play/abc/").unwrap();
        assert!(out.contains("https://cdn.example.com/play/abc/segment_00000.ts"));
        assert!(!out.contains("//segment"));
    }

    #[test]
    fn rewrite_manifest_with_refs_uses_exact_supplied_reference() {
        let refs = HashMap::from([
            (
                "segment_00000.ts".to_string(),
                "https://signed.example.com/seg0?token=a".to_string(),
            ),
            (
                "segment_00001.ts".to_string(),
                "https://signed.example.com/seg1?token=b".to_string(),
            ),
        ]);

        let out = rewrite_manifest_with_refs(VALID_MANIFEST, &refs).unwrap();

        assert!(out.contains("https://signed.example.com/seg0?token=a"));
        assert!(out.contains("https://signed.example.com/seg1?token=b"));
    }

    #[test]
    fn rewrite_manifest_with_refs_fails_closed_when_reference_is_missing() {
        let refs = HashMap::from([(
            "segment_00000.ts".to_string(),
            "https://signed.example.com/seg0?token=a".to_string(),
        )]);

        let err = rewrite_manifest_with_refs(VALID_MANIFEST, &refs).unwrap_err();

        assert_eq!(
            err,
            ManifestRewriteError::MissingSegmentReference("segment_00001.ts".to_string())
        );
    }

    #[test]
    fn crlf_segment_lines_do_not_leak_carriage_return() {
        let crlf_manifest = "#EXTM3U\r\n#EXTINF:6.0,\r\nprepared/asset-123/segment_00000.ts\r\n";
        let out = rewrite_manifest(crlf_manifest, ROUTED_BASE).unwrap();
        assert!(out.contains("https://cdn.example.com/play/abc/segment_00000.ts"));
        assert!(!out.contains("segment_00000.ts\r"));
    }
}
