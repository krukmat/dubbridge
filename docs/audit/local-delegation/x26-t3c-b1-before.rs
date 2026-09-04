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
