// T1: S1 domain — audit event types per ADR-018
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventKind {
    IngestionFinalized,
    IngestionRejectedMissingRights,
    IngestionRejectedMissingUploaderContext,
}

impl std::fmt::Display for AuditEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::IngestionFinalized => "ingestion_finalized",
            Self::IngestionRejectedMissingRights => "ingestion_rejected_missing_rights",
            Self::IngestionRejectedMissingUploaderContext => {
                "ingestion_rejected_missing_uploader_context"
            }
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub asset_id: Option<AssetId>,
    pub event_kind: AuditEventKind,
    pub ingest_token: Uuid,
    pub detail: Option<String>,
    pub happened_at: OffsetDateTime,
}

impl AuditEvent {
    pub fn new(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        ingest_token: Uuid,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            event_kind,
            ingest_token,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }
}
