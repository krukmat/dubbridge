// T1: S1 domain — asset aggregate and ingestion status state machine
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetId(pub Uuid);

impl AssetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ingestion status — fails closed. No processing-ready variant exists in S1.
/// Downstream slices (S2+) add transitions explicitly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IngestionStatus {
    Pending,
    Finalized,
    RejectedMissingRights,
    RejectedMissingUploaderContext,
}

impl std::fmt::Display for IngestionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Finalized => "finalized",
            Self::RejectedMissingRights => "rejected_missing_rights",
            Self::RejectedMissingUploaderContext => "rejected_missing_uploader_context",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub title: String,
    pub uploader_id: Uuid,
    pub status: IngestionStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Asset {
    pub fn new_pending(title: String, uploader_id: Uuid) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: AssetId::new(),
            title,
            uploader_id,
            status: IngestionStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T1-T3: Display variants used in DB serialization and audit logs
    #[test]
    fn ingestion_status_display_all_variants() {
        assert_eq!(IngestionStatus::Pending.to_string(), "pending");
        assert_eq!(IngestionStatus::Finalized.to_string(), "finalized");
        assert_eq!(
            IngestionStatus::RejectedMissingRights.to_string(),
            "rejected_missing_rights"
        );
        assert_eq!(
            IngestionStatus::RejectedMissingUploaderContext.to_string(),
            "rejected_missing_uploader_context"
        );
    }

    #[test]
    fn asset_id_display_matches_inner_uuid() {
        let id = AssetId::new();
        assert_eq!(id.to_string(), id.0.to_string());
    }

    #[test]
    fn asset_new_pending_sets_pending_status() {
        let uploader = Uuid::new_v4();
        let asset = Asset::new_pending("My Title".to_string(), uploader);
        assert_eq!(asset.status, IngestionStatus::Pending);
        assert_eq!(asset.title, "My Title");
        assert_eq!(asset.uploader_id, uploader);
    }
}
