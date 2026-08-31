use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::asset::AssetId;

use super::kind::AuditEventKind;

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
    /// Present for platform-ingest events; None for ingestion/recording events.
    pub platform_ingest_session_id: Option<Uuid>,
    pub detail: Option<String>,
    pub happened_at: OffsetDateTime,
}

impl AuditEvent {
    /// Returns whether an ingestion audit event has its required correlation
    /// token and no recording or platform-ingest session correlation.
    pub fn has_valid_ingestion_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::IngestionFinalized
                | AuditEventKind::IngestionRejectedMissingRights
                | AuditEventKind::IngestionRejectedMissingUploaderContext
                | AuditEventKind::IngestionRejectedDuplicateToken
        ) && self.ingest_token.is_some()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
    }

    /// Returns whether a recording audit event has its required recording
    /// session correlation and no platform-ingest session correlation.
    /// An ingest token remains optional for this event family.
    pub fn has_valid_recording_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::RecordingSessionCreated
                | AuditEventKind::RecordingRejectedMissingRights
                | AuditEventKind::RecordingCaptureStarted
                | AuditEventKind::RecordingRecorded
                | AuditEventKind::RecordingFailed
                | AuditEventKind::RecordingBridgedToAsset
        ) && self.recording_session_id.is_some()
            && self.platform_ingest_session_id.is_none()
    }

    /// Returns whether a platform-ingest audit event has its required
    /// platform-ingest session correlation and no recording session correlation.
    /// An ingest token remains optional for this event family.
    /// This checks only the in-memory event shape and does not by itself
    /// guarantee `platform_ingest_session_id` is persisted to the database.
    pub fn has_valid_platform_ingest_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::PlatformIngestSessionCreated
                | AuditEventKind::PlatformIngestRejectedMissingRights
                | AuditEventKind::PlatformIngestDownloadStarted
                | AuditEventKind::PlatformIngestDownloaded
                | AuditEventKind::PlatformIngestFailed
                | AuditEventKind::PlatformIngestBridgedToAsset
        ) && self.platform_ingest_session_id.is_some()
            && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
    }

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
            platform_ingest_session_id: None,
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
            platform_ingest_session_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for S3 platform-ingest lifecycle events. Always sets
    /// `platform_ingest_session_id`.
    pub fn new_platform_ingest(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        platform_ingest_session_id: Uuid,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            event_kind,
            ingest_token: None,
            recording_session_id: None,
            platform_ingest_session_id: Some(platform_ingest_session_id),
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for workspace governance events. These events are not tied to
    /// asset ingestion or recording correlation identifiers.
    pub fn new_workspace_event(event_kind: AuditEventKind, detail: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id: None,
            event_kind,
            ingest_token: None,
            recording_session_id: None,
            platform_ingest_session_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for S-110 voice-consent governance events (ADR-018, ADR-028).
    pub fn new_consent(
        asset_id: AssetId,
        event_kind: AuditEventKind,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id: Some(asset_id),
            event_kind,
            ingest_token: None,
            recording_session_id: None,
            platform_ingest_session_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for S-160 review/publication governance events.
    pub fn new_review_event(
        asset_id: AssetId,
        event_kind: AuditEventKind,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id: Some(asset_id),
            event_kind,
            ingest_token: None,
            recording_session_id: None,
            platform_ingest_session_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for S-125 playback-grant governance events.
    pub fn new_playback_event(
        asset_id: AssetId,
        event_kind: AuditEventKind,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id: Some(asset_id),
            event_kind,
            ingest_token: None,
            recording_session_id: None,
            platform_ingest_session_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for S-200 auth governance events.
    pub fn new_auth_event(event_kind: AuditEventKind, detail: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id: None,
            event_kind,
            ingest_token: None,
            recording_session_id: None,
            platform_ingest_session_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }
}
