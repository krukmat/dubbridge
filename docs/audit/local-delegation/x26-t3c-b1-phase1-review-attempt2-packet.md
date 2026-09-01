# X26-T3c-b1 — phase-1 review packet, retry after local resource recovery

This is a **task-analysis** review before any source file is changed. Review
only the proposed scope and semantic contract below. Return `PASS` when the
task is sufficiently bounded and the proposal satisfies every stated
acceptance criterion. Return `FINDINGS` only for a concrete correctness,
scope, or test-gap issue. Do not require persistence work or changes outside
`crates/domain/src/audit.rs`.

## Task and scope

- Task: `X26-T3c-b1`, RRI 20, Low.
- The only proposed source path is `crates/domain/src/audit.rs`.
- Add `AuditEvent::has_valid_ingestion_correlation(&self) -> bool`.
- It returns true only for these event kinds:
  `IngestionFinalized`, `IngestionRejectedMissingRights`,
  `IngestionRejectedMissingUploaderContext`, and
  `IngestionRejectedDuplicateToken`.
- For true, `ingest_token` must be `Some` and both
  `recording_session_id` and `platform_ingest_session_id` must be `None`.
- All non-ingestion kinds, including recording events, must return false.
- No constructor, persistence, emission, migration, or other event-family
  behavior may change.

## Acceptance examples

- HP-1: `AuditEvent::new(None, IngestionFinalized, token, None)` returns true.
- EC-1: an ingestion event whose `ingest_token` is cleared returns false.
- Scope guard: `RecordingSessionCreated` returns false.

## Proposed diff (not yet applied)

```diff
diff --git a/crates/domain/src/audit.rs b/crates/domain/src/audit.rs
--- a/crates/domain/src/audit.rs
+++ b/crates/domain/src/audit.rs
@@
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
@@
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
+        assert!(!event.has_valid_ingestion_correlation());
     }
@@
     fn audit_event_ingestion_sets_ingest_token_and_no_session_id() {
         let token = Uuid::new_v4();
         let event = AuditEvent::new(None, AuditEventKind::IngestionFinalized, token, None);
         assert_eq!(event.ingest_token, Some(token));
         assert!(event.recording_session_id.is_none());
         assert!(event.platform_ingest_session_id.is_none());
+        assert!(event.has_valid_ingestion_correlation());
     }
+
+    #[test]
+    fn ingestion_correlation_rejects_missing_token() {
+        let mut event = AuditEvent::new(
+            None,
+            AuditEventKind::IngestionRejectedMissingRights,
+            Uuid::new_v4(),
+            None,
+        );
+        event.ingest_token = None;
+
+        assert!(!event.has_valid_ingestion_correlation());
+    }
```

## Required reviewer output

Use the tagged `STATUS` / `SUMMARY` contract. Do not propose or write a patch.
