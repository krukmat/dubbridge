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
    assert!(event.has_valid_recording_correlation());
}

#[test]
fn recording_correlation_accepts_an_optional_ingest_token() {
    let event = AuditEvent::new_recording(
        None,
        AuditEventKind::RecordingRecorded,
        Uuid::new_v4(),
        Some(Uuid::new_v4()),
        None,
    );

    assert!(event.has_valid_recording_correlation());
}

#[test]
fn recording_correlation_requires_a_recording_id_and_rejects_other_families() {
    let session_id = Uuid::new_v4();
    let mut event = AuditEvent::new_recording(
        None,
        AuditEventKind::RecordingFailed,
        session_id,
        None,
        None,
    );

    event.recording_session_id = None;
    assert!(!event.has_valid_recording_correlation());

    event.recording_session_id = Some(session_id);
    event.event_kind = AuditEventKind::IngestionFinalized;
    assert!(!event.has_valid_recording_correlation());
}

#[test]
fn audit_event_ingestion_sets_ingest_token_and_no_session_id() {
    let token = Uuid::new_v4();
    let event = AuditEvent::new(None, AuditEventKind::IngestionFinalized, token, None);
    assert_eq!(event.ingest_token, Some(token));
    assert!(event.recording_session_id.is_none());
    assert!(event.platform_ingest_session_id.is_none());
    assert!(event.has_valid_ingestion_correlation());
}

#[test]
fn ingestion_correlation_requires_a_token_and_no_session_ids() {
    let token = Uuid::new_v4();
    let mut event = AuditEvent::new(None, AuditEventKind::IngestionFinalized, token, None);

    event.ingest_token = None;
    assert!(!event.has_valid_ingestion_correlation());

    event.ingest_token = Some(token);
    event.recording_session_id = Some(Uuid::new_v4());
    assert!(!event.has_valid_ingestion_correlation());
}

#[test]
fn ingestion_correlation_rejects_non_ingestion_events() {
    let event = AuditEvent::new_recording(
        None,
        AuditEventKind::RecordingSessionCreated,
        Uuid::new_v4(),
        Some(Uuid::new_v4()),
        None,
    );

    assert!(!event.has_valid_ingestion_correlation());
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

// HP-1: valid platform-ingest event with session id set and no
// competing ingest_token/recording_session_id correlates correctly.
#[test]
fn platform_ingest_correlation_accepts_a_platform_session_event() {
    let event = AuditEvent::new_platform_ingest(
        None,
        AuditEventKind::PlatformIngestDownloaded,
        Uuid::new_v4(),
        None,
    );

    assert!(event.has_valid_platform_ingest_correlation());
}

// EC-1: missing platform_ingest_session_id, a competing ingest_token, a
// competing recording_session_id, or a non-platform-ingest event kind
// must each independently reject the correlation.
#[test]
fn platform_ingest_correlation_requires_a_session_id_and_rejects_other_families() {
    let session_id = Uuid::new_v4();
    let mut event = AuditEvent::new_platform_ingest(
        None,
        AuditEventKind::PlatformIngestFailed,
        session_id,
        None,
    );
    assert!(event.has_valid_platform_ingest_correlation());

    event.platform_ingest_session_id = None;
    assert!(!event.has_valid_platform_ingest_correlation());

    event.platform_ingest_session_id = Some(session_id);
    event.ingest_token = Some(Uuid::new_v4());
    assert!(!event.has_valid_platform_ingest_correlation());

    event.ingest_token = None;
    event.recording_session_id = Some(Uuid::new_v4());
    assert!(!event.has_valid_platform_ingest_correlation());

    event.recording_session_id = None;
    event.event_kind = AuditEventKind::IngestionFinalized;
    assert!(!event.has_valid_platform_ingest_correlation());
}

// HP-1: existing workspace constructors satisfy the no-correlation predicate.
#[test]
fn workspace_correlation_accepts_workspace_events_with_no_correlation_ids() {
    for kind in [
        AuditEventKind::OrgCreated,
        AuditEventKind::OrgMemberAdded,
        AuditEventKind::ProjectCreated,
    ] {
        let event = AuditEvent::new_workspace_event(kind, None);
        assert!(event.has_valid_workspace_correlation());
    }
}

// EC-1: adding any correlation identifier to a workspace event, or using a
// non-workspace event kind, must independently reject the correlation.
#[test]
fn workspace_correlation_rejects_any_correlation_id_and_other_families() {
    let mut event = AuditEvent::new_workspace_event(AuditEventKind::OrgCreated, None);
    assert!(event.has_valid_workspace_correlation());

    event.ingest_token = Some(Uuid::new_v4());
    assert!(!event.has_valid_workspace_correlation());

    event.ingest_token = None;
    event.recording_session_id = Some(Uuid::new_v4());
    assert!(!event.has_valid_workspace_correlation());

    event.recording_session_id = None;
    event.platform_ingest_session_id = Some(Uuid::new_v4());
    assert!(!event.has_valid_workspace_correlation());

    event.platform_ingest_session_id = None;
    event.event_kind = AuditEventKind::ConsentGranted;
    assert!(!event.has_valid_workspace_correlation());
}

// HP-1: existing consent constructors satisfy the no-correlation predicate.
#[test]
fn consent_correlation_accepts_consent_events_with_no_correlation_ids() {
    use crate::asset::AssetId;
    let asset_id = AssetId::new();
    for kind in [
        AuditEventKind::ConsentGranted,
        AuditEventKind::ConsentRevoked,
        AuditEventKind::ConsentCheckDenied,
    ] {
        let event = AuditEvent::new_consent(asset_id, kind, None);
        assert!(event.has_valid_consent_correlation());
    }
}

// EC-1: adding any correlation identifier to a consent event, or using a
// non-consent event kind, must independently reject the correlation.
#[test]
fn consent_correlation_rejects_any_correlation_id_and_other_families() {
    use crate::asset::AssetId;
    let asset_id = AssetId::new();
    let mut event = AuditEvent::new_consent(asset_id, AuditEventKind::ConsentGranted, None);
    assert!(event.has_valid_consent_correlation());

    event.ingest_token = Some(Uuid::new_v4());
    assert!(!event.has_valid_consent_correlation());

    event.ingest_token = None;
    event.recording_session_id = Some(Uuid::new_v4());
    assert!(!event.has_valid_consent_correlation());

    event.recording_session_id = None;
    event.platform_ingest_session_id = Some(Uuid::new_v4());
    assert!(!event.has_valid_consent_correlation());

    event.platform_ingest_session_id = None;
    event.event_kind = AuditEventKind::OrgCreated;
    assert!(!event.has_valid_consent_correlation());
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
// Exhaustive one-assert-per-variant Display check; splitting it would only
// obscure the intent.
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
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
    assert_eq!(
        AuditEventKind::PlaybackGrantIssued.to_string(),
        "playback_grant_issued"
    );
    assert_eq!(
        AuditEventKind::PlaybackGrantRefused.to_string(),
        "playback_grant_refused"
    );
    assert_eq!(
        AuditEventKind::AuthLoginSucceeded.to_string(),
        "auth_login_succeeded"
    );
    assert_eq!(
        AuditEventKind::AuthLoginFailed.to_string(),
        "auth_login_failed"
    );
    assert_eq!(
        AuditEventKind::AuthRegistered.to_string(),
        "auth_registered"
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

#[test]
fn new_auth_event_sets_no_correlation_ids() {
    let event = AuditEvent::new_auth_event(
        AuditEventKind::AuthLoginFailed,
        Some("email=owner@example.com".to_string()),
    );

    assert!(event.asset_id.is_none());
    assert!(event.ingest_token.is_none());
    assert!(event.recording_session_id.is_none());
    assert!(event.platform_ingest_session_id.is_none());
    assert_eq!(event.event_kind, AuditEventKind::AuthLoginFailed);
    assert_eq!(event.detail.as_deref(), Some("email=owner@example.com"));
}

#[test]
fn new_playback_event_sets_asset_id_and_no_correlation_ids() {
    use crate::asset::AssetId;
    let asset_id = AssetId::new();
    let event = AuditEvent::new_playback_event(
        asset_id,
        AuditEventKind::PlaybackGrantIssued,
        Some("grant_id=demo".to_string()),
    );

    assert_eq!(event.asset_id, Some(asset_id));
    assert!(event.ingest_token.is_none());
    assert!(event.recording_session_id.is_none());
    assert!(event.platform_ingest_session_id.is_none());
    assert_eq!(event.event_kind, AuditEventKind::PlaybackGrantIssued);
    assert_eq!(event.detail.as_deref(), Some("grant_id=demo"));
}
