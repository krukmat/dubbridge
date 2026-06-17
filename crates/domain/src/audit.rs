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
    // S3-P1: platform ingest lifecycle events (ADR-018, ADR-025).
    // These occur before any ingest_token exists, so they use platform_ingest_session_id.
    PlatformIngestSessionCreated,
    PlatformIngestRejectedMissingRights,
    PlatformIngestDownloadStarted,
    PlatformIngestDownloaded,
    PlatformIngestFailed,
    PlatformIngestBridgedToAsset,
    // S-100-T3: workspace governance events (ADR-027, ADR-018).
    OrgCreated,
    OrgMemberAdded,
    ProjectCreated,
    // S-110-T2b: voice-consent governance events (ADR-018, ADR-028).
    ConsentGranted,
    ConsentRevoked,
    ConsentCheckDenied,
    // S-160-T2b: review/publication governance events (ADR-018, ADR-030).
    ReviewApproved,
    ReviewRejected,
    PublicationSucceeded,
    PublicationRefused,
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
            Self::PlatformIngestSessionCreated => "platform_ingest_session_created",
            Self::PlatformIngestRejectedMissingRights => "platform_ingest_rejected_missing_rights",
            Self::PlatformIngestDownloadStarted => "platform_ingest_download_started",
            Self::PlatformIngestDownloaded => "platform_ingest_downloaded",
            Self::PlatformIngestFailed => "platform_ingest_failed",
            Self::PlatformIngestBridgedToAsset => "platform_ingest_bridged_to_asset",
            Self::OrgCreated => "org_created",
            Self::OrgMemberAdded => "org_member_added",
            Self::ProjectCreated => "project_created",
            Self::ConsentGranted => "consent_granted",
            Self::ConsentRevoked => "consent_revoked",
            Self::ConsentCheckDenied => "consent_check_denied",
            Self::ReviewApproved => "review_approved",
            Self::ReviewRejected => "review_rejected",
            Self::PublicationSucceeded => "publication_succeeded",
            Self::PublicationRefused => "publication_refused",
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
    /// Present for platform-ingest events; None for ingestion/recording events.
    pub platform_ingest_session_id: Option<Uuid>,
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
        assert!(event.platform_ingest_session_id.is_none());
        assert_eq!(event.event_kind, AuditEventKind::RecordingSessionCreated);
    }

    #[test]
    fn audit_event_ingestion_sets_ingest_token_and_no_session_id() {
        let token = Uuid::new_v4();
        let event = AuditEvent::new(None, AuditEventKind::IngestionFinalized, token, None);
        assert_eq!(event.ingest_token, Some(token));
        assert!(event.recording_session_id.is_none());
        assert!(event.platform_ingest_session_id.is_none());
    }

    #[test]
    fn audit_event_platform_ingest_round_trip() {
        let session_id = Uuid::new_v4();
        let event = AuditEvent::new_platform_ingest(
            None,
            AuditEventKind::PlatformIngestSessionCreated,
            session_id,
            None,
        );
        assert!(event.ingest_token.is_none());
        assert!(event.recording_session_id.is_none());
        assert_eq!(event.platform_ingest_session_id, Some(session_id));
        assert_eq!(
            event.event_kind,
            AuditEventKind::PlatformIngestSessionCreated
        );
    }

    #[test]
    fn audit_event_kind_display_platform_ingest_variants() {
        assert_eq!(
            AuditEventKind::PlatformIngestSessionCreated.to_string(),
            "platform_ingest_session_created"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestRejectedMissingRights.to_string(),
            "platform_ingest_rejected_missing_rights"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestDownloadStarted.to_string(),
            "platform_ingest_download_started"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestDownloaded.to_string(),
            "platform_ingest_downloaded"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestFailed.to_string(),
            "platform_ingest_failed"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestBridgedToAsset.to_string(),
            "platform_ingest_bridged_to_asset"
        );
    }

    #[test]
    fn audit_event_workspace_round_trip_has_no_correlation_ids() {
        let event = AuditEvent::new_workspace_event(
            AuditEventKind::OrgCreated,
            Some("{\"org_id\":\"demo\"}".to_string()),
        );
        assert!(event.asset_id.is_none());
        assert!(event.ingest_token.is_none());
        assert!(event.recording_session_id.is_none());
        assert!(event.platform_ingest_session_id.is_none());
        assert_eq!(event.event_kind, AuditEventKind::OrgCreated);
        assert_eq!(event.detail.as_deref(), Some("{\"org_id\":\"demo\"}"));
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
        assert_eq!(
            AuditEventKind::PlatformIngestSessionCreated.to_string(),
            "platform_ingest_session_created"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestRejectedMissingRights.to_string(),
            "platform_ingest_rejected_missing_rights"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestDownloadStarted.to_string(),
            "platform_ingest_download_started"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestDownloaded.to_string(),
            "platform_ingest_downloaded"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestFailed.to_string(),
            "platform_ingest_failed"
        );
        assert_eq!(
            AuditEventKind::PlatformIngestBridgedToAsset.to_string(),
            "platform_ingest_bridged_to_asset"
        );
        assert_eq!(AuditEventKind::OrgCreated.to_string(), "org_created");
        assert_eq!(
            AuditEventKind::OrgMemberAdded.to_string(),
            "org_member_added"
        );
        assert_eq!(
            AuditEventKind::ProjectCreated.to_string(),
            "project_created"
        );
        assert_eq!(
            AuditEventKind::ConsentGranted.to_string(),
            "consent_granted"
        );
        assert_eq!(
            AuditEventKind::ConsentRevoked.to_string(),
            "consent_revoked"
        );
        assert_eq!(
            AuditEventKind::ConsentCheckDenied.to_string(),
            "consent_check_denied"
        );
        assert_eq!(
            AuditEventKind::ReviewApproved.to_string(),
            "review_approved"
        );
        assert_eq!(
            AuditEventKind::ReviewRejected.to_string(),
            "review_rejected"
        );
        assert_eq!(
            AuditEventKind::PublicationSucceeded.to_string(),
            "publication_succeeded"
        );
        assert_eq!(
            AuditEventKind::PublicationRefused.to_string(),
            "publication_refused"
        );
    }

    #[test]
    fn new_consent_sets_asset_id_and_no_correlation_ids() {
        use crate::asset::AssetId;
        let asset_id = AssetId::new();
        let event = AuditEvent::new_consent(
            asset_id,
            AuditEventKind::ConsentGranted,
            Some("scope=voice_clone".to_string()),
        );
        assert_eq!(event.asset_id, Some(asset_id));
        assert!(event.ingest_token.is_none());
        assert!(event.recording_session_id.is_none());
        assert!(event.platform_ingest_session_id.is_none());
        assert_eq!(event.event_kind, AuditEventKind::ConsentGranted);
        assert_eq!(event.detail.as_deref(), Some("scope=voice_clone"));
    }

    #[test]
    fn new_review_event_sets_asset_id_and_no_correlation_ids() {
        use crate::asset::AssetId;
        let asset_id = AssetId::new();
        let event = AuditEvent::new_review_event(
            asset_id,
            AuditEventKind::ReviewApproved,
            Some("review_task_id=demo".to_string()),
        );
        assert_eq!(event.asset_id, Some(asset_id));
        assert!(event.ingest_token.is_none());
        assert!(event.recording_session_id.is_none());
        assert!(event.platform_ingest_session_id.is_none());
        assert_eq!(event.event_kind, AuditEventKind::ReviewApproved);
        assert_eq!(event.detail.as_deref(), Some("review_task_id=demo"));
    }
}
