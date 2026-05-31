// T1: S1 domain — original media artifact record
// S3-T1: added RecordedStreamMedia variant and parse_artifact_kind() (F3 fix)
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

/// Artifact kind — S1 tracks upload artifacts; S3 adds recorded stream media.
/// Probe, rendition, HLS, transcript etc. are added in later slices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    OriginalMedia,
    /// S3-T1: assembled MP4 produced by the recording bridge (ADR-021).
    RecordedStreamMedia,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OriginalMedia => "original_media",
            Self::RecordedStreamMedia => "recorded_stream_media",
        };
        write!(f, "{s}")
    }
}

/// Parses a DB-stored kind string into `ArtifactKind`.
///
/// Unknown strings fall back to `OriginalMedia` so a mislabeled row does not
/// panic the process. The recording bridge (S3-T5) uses this to ensure
/// `RecordedStreamMedia` artifacts read back with the correct kind (F3).
pub fn parse_artifact_kind(s: &str) -> ArtifactKind {
    match s {
        "recorded_stream_media" => ArtifactKind::RecordedStreamMedia,
        _ => ArtifactKind::OriginalMedia,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: Uuid,
    pub asset_id: AssetId,
    pub kind: ArtifactKind,
    /// Unique token that ties this artifact to one finalization request.
    /// UNIQUE constraint in DB prevents duplicate artifacts for the same ingest.
    pub ingest_token: Uuid,
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    /// SHA-256 hex digest of the stored object — required per ADR-006.
    pub checksum: String,
    pub created_at: OffsetDateTime,
}

impl ArtifactRecord {
    pub fn new_original(
        asset_id: AssetId,
        ingest_token: Uuid,
        storage_key: String,
        content_type: String,
        size_bytes: i64,
        checksum: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            kind: ArtifactKind::OriginalMedia,
            ingest_token,
            storage_key,
            content_type,
            size_bytes,
            checksum,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // S3-T1: F3 acceptance criteria — parse_artifact_kind round-trips
    #[test]
    fn parse_recorded_stream_media() {
        assert_eq!(
            parse_artifact_kind("recorded_stream_media"),
            ArtifactKind::RecordedStreamMedia
        );
    }

    #[test]
    fn parse_original_media() {
        assert_eq!(
            parse_artifact_kind("original_media"),
            ArtifactKind::OriginalMedia
        );
    }

    #[test]
    fn parse_unknown_falls_back_to_original_media() {
        assert_eq!(
            parse_artifact_kind("probe_output"),
            ArtifactKind::OriginalMedia
        );
    }

    #[test]
    fn artifact_kind_display_all_variants() {
        assert_eq!(ArtifactKind::OriginalMedia.to_string(), "original_media");
        assert_eq!(
            ArtifactKind::RecordedStreamMedia.to_string(),
            "recorded_stream_media"
        );
    }
}
