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
    /// Generic correlation identity for event families that cannot reuse an
    /// ingest token. P2 uses the publication id as the stable correlation id.
    pub correlation_id: Option<Uuid>,
    /// P2 publication identity. Always paired with `lineage_id` for P2 events.
    pub publication_id: Option<Uuid>,
    /// P2 K1 lineage identity. Never inferred from the current publication row.
    pub lineage_id: Option<Uuid>,
    pub detail: Option<String>,
    pub happened_at: OffsetDateTime,
}

impl AuditEvent {
    fn has_no_p2p_correlation(&self) -> bool {
        self.correlation_id.is_none() && self.publication_id.is_none() && self.lineage_id.is_none()
    }

    /// Returns whether an ingestion audit event has its required correlation
    /// token and no recording/platform/P2 correlation.
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
            && self.has_no_p2p_correlation()
    }

    /// Returns whether a recording audit event has its required recording
    /// session correlation and no platform/P2 correlation. An ingest token
    /// remains optional for this event family.
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
            && self.has_no_p2p_correlation()
    }

    /// Returns whether a platform-ingest audit event has its required
    /// platform-ingest session correlation and no competing correlations.
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
            && self.has_no_p2p_correlation()
    }

    /// Returns whether a workspace audit event has no correlation IDs.
    pub fn has_valid_workspace_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::OrgCreated
                | AuditEventKind::OrgMemberAdded
                | AuditEventKind::ProjectCreated
        ) && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
            && self.has_no_p2p_correlation()
    }

    /// Returns whether a consent audit event has no correlation IDs.
    pub fn has_valid_consent_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::ConsentGranted
                | AuditEventKind::ConsentRevoked
                | AuditEventKind::ConsentCheckDenied
        ) && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
            && self.has_no_p2p_correlation()
    }

    /// Returns whether a review audit event has no correlation IDs.
    pub fn has_valid_review_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::ReviewApproved
                | AuditEventKind::ReviewRejected
                | AuditEventKind::PublicationSucceeded
                | AuditEventKind::PublicationRefused
        ) && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
            && self.has_no_p2p_correlation()
    }

    /// Returns whether a playback audit event has no correlation IDs.
    pub fn has_valid_playback_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::PlaybackGrantIssued | AuditEventKind::PlaybackGrantRefused
        ) && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
            && self.has_no_p2p_correlation()
    }

    /// Returns whether an auth audit event has no correlation IDs.
    pub fn has_valid_auth_correlation(&self) -> bool {
        matches!(
            self.event_kind,
            AuditEventKind::AuthLoginSucceeded
                | AuditEventKind::AuthLoginFailed
                | AuditEventKind::AuthRegistered
        ) && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
            && self.has_no_p2p_correlation()
    }

    /// P2 events never fabricate an ingest token. They bind the exact durable
    /// publication and K1 lineage, while `correlation_id` equals publication id.
    pub fn has_valid_p2p_correlation(&self) -> bool {
        let is_p2p_kind = matches!(
            self.event_kind,
            AuditEventKind::P2pPublicationIntentCreated
                | AuditEventKind::P2pLineageSealed
                | AuditEventKind::P2pPublicationConfirmed
                | AuditEventKind::P2pPublicationReconciliationEntered
                | AuditEventKind::P2pPublicationReady
                | AuditEventKind::P2pPublicationFailed
        );
        let publication = self.publication_id.filter(|value| !value.is_nil());
        let lineage = self.lineage_id.filter(|value| !value.is_nil());

        is_p2p_kind
            && self.asset_id.is_some()
            && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
            && publication.is_some()
            && lineage.is_some()
            && self.correlation_id == publication
    }


    /// P3 audience/device events use an explicit non-nil correlation identity.
    /// Invitation/authorization/envelope success events also bind exact asset,
    /// publication and lineage. Device registration and pre-resolution denials
    /// may omit package identity, but never invent it.
    pub fn has_valid_p3_correlation(&self) -> bool {
        let is_p3_kind = matches!(
            self.event_kind,
            AuditEventKind::P2pDeviceRegistered
                | AuditEventKind::P2pInvitationCreated
                | AuditEventKind::P2pInvitationClaimed
                | AuditEventKind::P2pAudienceAuthorizationIssued
                | AuditEventKind::P2pDeviceEnvelopeReleased
                | AuditEventKind::P2pAudienceAccessDenied
        );
        let correlation = self.correlation_id.filter(|value| !value.is_nil());
        let publication = self.publication_id.filter(|value| !value.is_nil());
        let lineage = self.lineage_id.filter(|value| !value.is_nil());
        let has_exact_package = self.asset_id.is_some() && publication.is_some() && lineage.is_some();
        let has_no_package = publication.is_none() && lineage.is_none();

        let shape_valid = match self.event_kind {
            AuditEventKind::P2pInvitationCreated
            | AuditEventKind::P2pInvitationClaimed
            | AuditEventKind::P2pAudienceAuthorizationIssued
            | AuditEventKind::P2pDeviceEnvelopeReleased => has_exact_package,
            AuditEventKind::P2pDeviceRegistered | AuditEventKind::P2pAudienceAccessDenied => {
                has_exact_package || has_no_package
            }
            _ => false,
        };

        is_p3_kind
            && self.ingest_token.is_none()
            && self.recording_session_id.is_none()
            && self.platform_ingest_session_id.is_none()
            && correlation.is_some()
            && shape_valid
    }

    fn base_event(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            asset_id,
            event_kind,
            ingest_token: None,
            recording_session_id: None,
            platform_ingest_session_id: None,
            correlation_id: None,
            publication_id: None,
            lineage_id: None,
            detail,
            happened_at: OffsetDateTime::now_utc(),
        }
    }

    /// Constructor for S1 ingestion events. Always sets `ingest_token`.
    pub fn new(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        ingest_token: Uuid,
        detail: Option<String>,
    ) -> Self {
        let mut event = Self::base_event(asset_id, event_kind, detail);
        event.ingest_token = Some(ingest_token);
        event
    }

    /// Constructor for S3 recording lifecycle events. Always sets `recording_session_id`.
    pub fn new_recording(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        recording_session_id: Uuid,
        ingest_token: Option<Uuid>,
        detail: Option<String>,
    ) -> Self {
        let mut event = Self::base_event(asset_id, event_kind, detail);
        event.ingest_token = ingest_token;
        event.recording_session_id = Some(recording_session_id);
        event
    }

    /// Constructor for S3 platform-ingest lifecycle events. Always sets
    /// `platform_ingest_session_id`.
    pub fn new_platform_ingest(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        platform_ingest_session_id: Uuid,
        detail: Option<String>,
    ) -> Self {
        let mut event = Self::base_event(asset_id, event_kind, detail);
        event.platform_ingest_session_id = Some(platform_ingest_session_id);
        event
    }

    /// Constructor for workspace governance events.
    pub fn new_workspace_event(event_kind: AuditEventKind, detail: Option<String>) -> Self {
        Self::base_event(None, event_kind, detail)
    }

    /// Constructor for S-110 voice-consent governance events (ADR-018, ADR-028).
    pub fn new_consent(
        asset_id: AssetId,
        event_kind: AuditEventKind,
        detail: Option<String>,
    ) -> Self {
        Self::base_event(Some(asset_id), event_kind, detail)
    }

    /// Constructor for S-160 review/publication governance events.
    pub fn new_review_event(
        asset_id: AssetId,
        event_kind: AuditEventKind,
        detail: Option<String>,
    ) -> Self {
        Self::base_event(Some(asset_id), event_kind, detail)
    }

    /// Constructor for S-125 playback-grant governance events.
    pub fn new_playback_event(
        asset_id: AssetId,
        event_kind: AuditEventKind,
        detail: Option<String>,
    ) -> Self {
        Self::base_event(Some(asset_id), event_kind, detail)
    }

    /// Constructor for S-200 auth governance events.
    pub fn new_auth_event(event_kind: AuditEventKind, detail: Option<String>) -> Self {
        Self::base_event(None, event_kind, detail)
    }


    /// Constructor for MVP0-P2P P3 audience/device governance events.
    pub fn new_p3_event(
        asset_id: Option<AssetId>,
        event_kind: AuditEventKind,
        correlation_id: Uuid,
        publication_id: Option<Uuid>,
        lineage_id: Option<Uuid>,
        detail: Option<String>,
    ) -> Self {
        let mut event = Self::base_event(asset_id, event_kind, detail);
        event.correlation_id = Some(correlation_id);
        event.publication_id = publication_id;
        event.lineage_id = lineage_id;
        event
    }

    /// Constructor for MVP0-P2P publication lifecycle audit events.
    pub fn new_p2p_event(
        asset_id: AssetId,
        event_kind: AuditEventKind,
        publication_id: Uuid,
        lineage_id: Uuid,
        detail: Option<String>,
    ) -> Self {
        let mut event = Self::base_event(Some(asset_id), event_kind, detail);
        event.correlation_id = Some(publication_id);
        event.publication_id = Some(publication_id);
        event.lineage_id = Some(lineage_id);
        event
    }
}
