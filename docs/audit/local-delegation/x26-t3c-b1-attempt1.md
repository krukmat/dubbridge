# X26-T3c-b1 — Qwen Developer packet, attempt 1

## Goal

Modify only `crates/domain/src/audit.rs`. Apply the exact replacement below.
Do not modify any other file, do not change any event family other than
ingestion, and do not edit persistence or audit emission code.

## Required result

1. Add `pub fn has_valid_ingestion_correlation(&self) -> bool` after
   `new_auth_event`.
2. It must return true only when the event kind is one of the four ingestion
   variants, `ingest_token` is present, and both session IDs are absent.
3. Extend the existing ingestion constructor unit test to assert true.
4. Add one malformed-ingestion test that clears `ingest_token` and asserts
   false, and one non-ingestion scope-guard test that asserts false.
5. Return exactly one before-after replacement for
   `crates/domain/src/audit.rs`; do not add prose or a diff.

## Exact BEFORE block

```rust
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
```

## Exact AFTER block

```rust
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

    /// Returns whether an ingestion event has its required and exclusive correlation shape.
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
        assert!(!event.has_valid_ingestion_correlation());
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
    fn ingestion_correlation_rejects_missing_token() {
        let mut event = AuditEvent::new(
            None,
            AuditEventKind::IngestionRejectedMissingRights,
            Uuid::new_v4(),
            None,
        );
        event.ingest_token = None;

        assert!(!event.has_valid_ingestion_correlation());
    }
```

## Output contract

Use the `before-after` tagged response contract from
`scripts/delegate-low-rri.py`; emit only the replacement for the exact BEFORE
block above at path `crates/domain/src/audit.rs`.
