// T1: S1 domain — audit event types per ADR-018
// S3-T1: generalized for recording events — ingest_token is now Option<Uuid>,
// recording_session_id added, and recording lifecycle AuditEventKind variants added (F1/F2).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventKind {
    // S1 ingestion events
    IngestionFinalized,
    IngestionRejectedMissingRights,
    IngestionRejectedMissingUploaderContext,
    // H1-T3: duplicate-token rejections now emit a durable audit row (ADR-018).
    IngestionRejectedDuplicateToken,
    // S3-T1: recording session lifecycle events (ADR-018, F2).
    // These occur before any ingest_token exists, so they use recording_session_id.
    RecordingSessionCreated,
    RecordingRejectedMissingRights,
    RecordingCaptureStarted,
    RecordingRecorded,
    RecordingFailed,
    RecordingBridgedToAsset,
}

impl std::fmt::Display for AuditEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::IngestionFinalized => "ingestion_finalized",
            Self::IngestionRejectedMissingRights => "ingestion_rejected_missing_rights",
            Self::IngestionRejectedMissingUploaderContext => {
                "ingestion_rejected_missing_uploader_context"
            }
            Self::IngestionRejectedDuplicateToken => "ingestion_rejected_duplicate_token",
            Self::RecordingSessionCreated => "recording_session_created",
            Self::RecordingRejectedMissingRights => "recording_rejected_missing_rights",
            Self::RecordingCaptureStarted => "recording_capture_started",
            Self::RecordingRecorded => "recording_recorded",
            Self::RecordingFailed => "recording_failed",
            Self::RecordingBridgedToAsset => "recording_bridged_to_asset",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub asset_id: Option<AssetId>,
    pub event_kind: AuditEventKind,
    /// Present for ingestion events; None for recording lifecycle events that
    /// occur before any ingest_token is allocated (F2, ADR-018).
    pub ingest_token: Option<Uuid>,
    /// Present for recording events; None for S1 ingestion events (F2, ADR-018).
    pub recording_session_id: Option<Uuid>,
    pub detail: Option<String>,
    pub happened_at: OffsetDateTime,
}

impl AuditEvent {
    /// Constructor for S1 ingestion events. Always sets `ingest_token`.
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
            ingest_token: Some(ingest_token),
            recording_session_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for S3 recording lifecycle events. Always sets `recording_session_id`.
    pub fn new_recording(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        recording_session_id: Uuid,
        ingest_token: Option<Uuid>,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            event_kind,
            ingest_token,
            recording_session_id: Some(recording_session_id),
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // S3-T1: AuditEvent round-trips with ingest_token=None and recording_session_id set
    #[test]
    fn audit_event_recording_round_trip_no_ingest_token() {
        let session_id = Uuid::new_v4();
        let event = AuditEvent::new_recording(
            None,
            AuditEventKind::RecordingSessionCreated,
            session_id,
            None,
            Some("session created".to_string()),
        );
        assert!(event.ingest_token.is_none());
        assert_eq!(event.recording_session_id, Some(session_id));
        assert_eq!(event.event_kind, AuditEventKind::RecordingSessionCreated);
    }

    #[test]
    fn audit_event_ingestion_sets_ingest_token_and_no_session_id() {
        let token = Uuid::new_v4();
        let event = AuditEvent::new(None, AuditEventKind::IngestionFinalized, token, None);
        assert_eq!(event.ingest_token, Some(token));
        assert!(event.recording_session_id.is_none());
    }

    #[test]
    fn audit_event_kind_display_all_variants() {
        assert_eq!(
            AuditEventKind::IngestionFinalized.to_string(),
            "ingestion_finalized"
        );
        assert_eq!(
            AuditEventKind::IngestionRejectedMissingRights.to_string(),
            "ingestion_rejected_missing_rights"
        );
        assert_eq!(
            AuditEventKind::IngestionRejectedMissingUploaderContext.to_string(),
            "ingestion_rejected_missing_uploader_context"
        );
        assert_eq!(
            AuditEventKind::IngestionRejectedDuplicateToken.to_string(),
            "ingestion_rejected_duplicate_token"
        );
        assert_eq!(
            AuditEventKind::RecordingSessionCreated.to_string(),
            "recording_session_created"
        );
        assert_eq!(
            AuditEventKind::RecordingRejectedMissingRights.to_string(),
            "recording_rejected_missing_rights"
        );
        assert_eq!(
            AuditEventKind::RecordingCaptureStarted.to_string(),
            "recording_capture_started"
        );
        assert_eq!(
            AuditEventKind::RecordingRecorded.to_string(),
            "recording_recorded"
        );
        assert_eq!(
            AuditEventKind::RecordingFailed.to_string(),
            "recording_failed"
        );
        assert_eq!(
            AuditEventKind::RecordingBridgedToAsset.to_string(),
            "recording_bridged_to_asset"
        );
    }
}
