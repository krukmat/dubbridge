// T1: S1 domain — audit event types per ADR-018
// S3-T1: generalized for recording events — ingest_token is now Option<Uuid>,
// recording_session_id added, and recording lifecycle AuditEventKind variants added (F1/F2).
use serde::{Deserialize, Serialize};

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
    // S-125-T4b-i: playback-grant governance events (ADR-018, ADR-032).
    PlaybackGrantIssued,
    PlaybackGrantRefused,
    // S-200-T4c: auth governance events for API-issued credential login (ADR-018, ADR-031).
    AuthLoginSucceeded,
    AuthLoginFailed,
    AuthRegistered,
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
            Self::PlaybackGrantIssued => "playback_grant_issued",
            Self::PlaybackGrantRefused => "playback_grant_refused",
            Self::AuthLoginSucceeded => "auth_login_succeeded",
            Self::AuthLoginFailed => "auth_login_failed",
            Self::AuthRegistered => "auth_registered",
        };
        write!(f, "{s}")
    }
}
