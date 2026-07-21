// T1: S1 domain — original media artifact record
// S3-T1: added RecordedStreamMedia variant and parse_artifact_kind() (F3 fix)
// S-120-T2: added derived artifact kinds, DerivedArtifact, and PreparationStatus
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

/// Artifact kind — S1 tracks upload artifacts; S3 adds recorded stream media;
/// S-120 adds preparation-derived kinds (probe metadata, HLS manifest/segment).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    OriginalMedia,
    /// S3-T1: assembled MP4 produced by the recording bridge (ADR-021).
    RecordedStreamMedia,
    /// S3-P1: owner-authorized media downloaded from a platform connector.
    DownloadedPlatformMedia,
    /// S-120: ffprobe JSON output persisted as a derived artifact.
    ProbeMetadata,
    /// S-120: HLS `.m3u8` playlist derived from the source artifact.
    HlsManifest,
    /// S-120: individual HLS media segment derived from the source artifact.
    HlsSegment,
    /// S-130: full-text transcript derived from the source artifact.
    TranscriptText,
    /// S-130: word-level alignment derived from the source artifact.
    WordAlignment,
    /// S-140: subtitle file (e.g., VTT/SRT) derived from the source artifact.
    Subtitle,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OriginalMedia => "original_media",
            Self::RecordedStreamMedia => "recorded_stream_media",
            Self::DownloadedPlatformMedia => "downloaded_platform_media",
            Self::ProbeMetadata => "probe_metadata",
            Self::HlsManifest => "hls_manifest",
            Self::HlsSegment => "hls_segment",
            Self::TranscriptText => "transcript_text",
            Self::WordAlignment => "word_alignment",
            Self::Subtitle => "subtitle",
        };
        write!(f, "{s}")
    }
}

/// Parses a DB-stored kind string into `ArtifactKind`.
///
/// Unknown strings fall back to `OriginalMedia` — used by legacy code paths
/// that tolerate unknown kinds. Fail-closed paths use `parse_artifact_kind_strict`
/// in the DB layer instead.
pub fn parse_artifact_kind(s: &str) -> ArtifactKind {
    match s {
        "recorded_stream_media" => ArtifactKind::RecordedStreamMedia,
        "downloaded_platform_media" => ArtifactKind::DownloadedPlatformMedia,
        "probe_metadata" => ArtifactKind::ProbeMetadata,
        "hls_manifest" => ArtifactKind::HlsManifest,
        "hls_segment" => ArtifactKind::HlsSegment,
        "transcript_text" => ArtifactKind::TranscriptText,
        "word_alignment" => ArtifactKind::WordAlignment,
        "subtitle" => ArtifactKind::Subtitle,
        _ => ArtifactKind::OriginalMedia,
    }
}

/// S-120-T2: Preparation readiness state for an asset.
///
/// Downstream slices (S-125, S-130, S-160) must treat any non-`Ready` state
/// as a hard precondition failure and reject or defer work accordingly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreparationStatus {
    Pending,
    InProgress,
    Ready,
    /// Preparation failed. `error_detail` in the persisted row records why.
    Failed,
}

impl std::fmt::Display for PreparationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Ready => "ready",
            Self::Failed => "failed",
        };
        write!(f, "{s}")
    }
}

/// S-120-T2: Persisted preparation status record for an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationStatusRecord {
    pub asset_id: AssetId,
    pub status: PreparationStatus,
    pub error_detail: Option<String>,
    pub updated_at: OffsetDateTime,
}

/// S-120-T2: A derived artifact row — produced by a preparation stage from a
/// source artifact. Uses `parent_artifact_id` instead of `ingest_token`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedArtifact {
    pub id: Uuid,
    pub asset_id: AssetId,
    pub parent_artifact_id: Uuid,
    pub kind: ArtifactKind,
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub checksum: String,
    pub created_at: OffsetDateTime,
}

impl DerivedArtifact {
    pub fn new(
        asset_id: AssetId,
        parent_artifact_id: Uuid,
        kind: ArtifactKind,
        storage_key: String,
        content_type: String,
        size_bytes: i64,
        checksum: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            parent_artifact_id,
            kind,
            storage_key,
            content_type,
            size_bytes,
            checksum,
            created_at: OffsetDateTime::now_utc(),
        }
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

/// S-130-T1: Transcription readiness state for an asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionStatus {
    Pending,
    InProgress,
    Ready,
    /// Transcription failed. `error_detail` in the persisted row records why.
    Failed,
}

impl std::fmt::Display for TranscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Ready => "ready",
            Self::Failed => "failed",
        };
        write!(f, "{s}")
    }
}

/// S-130-T1: Persisted transcription status record for an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionStatusRecord {
    pub asset_id: AssetId,
    pub status: TranscriptionStatus,
    pub error_detail: Option<String>,
    pub updated_at: OffsetDateTime,
}

/// S-140-T1a: Subtitle readiness state for an asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleStatus {
    Pending,
    InProgress,
    Ready,
    /// Subtitle generation failed. `error_detail` in the persisted row records why.
    Failed,
}

impl std::fmt::Display for SubtitleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Ready => "ready",
            Self::Failed => "failed",
        };
        write!(f, "{s}")
    }
}

/// S-140-T1a: Persisted subtitle status record for an asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleStatusRecord {
    pub asset_id: AssetId,
    pub status: SubtitleStatus,
    pub error_detail: Option<String>,
    pub updated_at: OffsetDateTime,
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

    // S3-P1: DownloadedPlatformMedia — written TDD-first, implemented in sub-task 4
    #[test]
    fn parse_downloaded_platform_media() {
        assert_eq!(
            parse_artifact_kind("downloaded_platform_media"),
            ArtifactKind::DownloadedPlatformMedia
        );
    }

    #[test]
    fn artifact_kind_display_downloaded_platform_media() {
        assert_eq!(
            ArtifactKind::DownloadedPlatformMedia.to_string(),
            "downloaded_platform_media"
        );
    }

    // S-120-T2: preparation-derived artifact kinds
    #[test]
    fn parse_probe_metadata() {
        assert_eq!(
            parse_artifact_kind("probe_metadata"),
            ArtifactKind::ProbeMetadata
        );
    }

    #[test]
    fn parse_hls_manifest() {
        assert_eq!(
            parse_artifact_kind("hls_manifest"),
            ArtifactKind::HlsManifest
        );
    }

    #[test]
    fn parse_hls_segment() {
        assert_eq!(parse_artifact_kind("hls_segment"), ArtifactKind::HlsSegment);
    }

    #[test]
    fn artifact_kind_display_preparation_variants() {
        assert_eq!(ArtifactKind::ProbeMetadata.to_string(), "probe_metadata");
        assert_eq!(ArtifactKind::HlsManifest.to_string(), "hls_manifest");
        assert_eq!(ArtifactKind::HlsSegment.to_string(), "hls_segment");
    }

    #[test]
    fn preparation_status_display_all_variants() {
        assert_eq!(PreparationStatus::Pending.to_string(), "pending");
        assert_eq!(PreparationStatus::InProgress.to_string(), "in_progress");
        assert_eq!(PreparationStatus::Ready.to_string(), "ready");
        assert_eq!(PreparationStatus::Failed.to_string(), "failed");
    }

    // S-130-T1: transcript artifact kinds
    #[test]
    fn parse_transcript_text() {
        assert_eq!(
            parse_artifact_kind("transcript_text"),
            ArtifactKind::TranscriptText
        );
    }

    #[test]
    fn parse_word_alignment() {
        assert_eq!(
            parse_artifact_kind("word_alignment"),
            ArtifactKind::WordAlignment
        );
    }

    #[test]
    fn artifact_kind_display_transcript_variants() {
        assert_eq!(ArtifactKind::TranscriptText.to_string(), "transcript_text");
        assert_eq!(ArtifactKind::WordAlignment.to_string(), "word_alignment");
    }

    #[test]
    fn transcription_status_display_all_variants() {
        assert_eq!(TranscriptionStatus::Pending.to_string(), "pending");
        assert_eq!(TranscriptionStatus::InProgress.to_string(), "in_progress");
        assert_eq!(TranscriptionStatus::Ready.to_string(), "ready");
        assert_eq!(TranscriptionStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn derived_artifact_new_sets_fields() {
        let asset_id = AssetId(Uuid::new_v4());
        let parent_id = Uuid::new_v4();
        let da = DerivedArtifact::new(
            asset_id,
            parent_id,
            ArtifactKind::ProbeMetadata,
            "probe/key".into(),
            "application/json".into(),
            512,
            "abc123".into(),
        );
        assert_eq!(da.asset_id, asset_id);
        assert_eq!(da.parent_artifact_id, parent_id);
        assert_eq!(da.kind, ArtifactKind::ProbeMetadata);
        assert_eq!(da.size_bytes, 512);
    }

    #[test]
    fn parse_subtitle() {
        let kind = parse_artifact_kind("subtitle");
        assert_eq!(kind, ArtifactKind::Subtitle);
        assert_eq!(kind.to_string(), "subtitle");
    }

    #[test]
    fn subtitle_status_display_all_variants() {
        assert_eq!(SubtitleStatus::Pending.to_string(), "pending");
        assert_eq!(SubtitleStatus::InProgress.to_string(), "in_progress");
        assert_eq!(SubtitleStatus::Ready.to_string(), "ready");
        assert_eq!(SubtitleStatus::Failed.to_string(), "failed");
    }
}
