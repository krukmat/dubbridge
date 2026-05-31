// T1: S1 domain — original media artifact record
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

/// Artifact kind — S1 only tracks the original upload.
/// Probe, rendition, HLS, transcript etc. are added in later slices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    OriginalMedia,
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "original_media")
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
