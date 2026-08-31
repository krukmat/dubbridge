---
type: Audit
title: "X26-T3c correlation contract matrix"
task_id: X26-T3c-a
status: complete
---

# X26-T3c correlation contract matrix

## Decision

The audit-event contract is **family-specific**, not “exactly one identifier”
for every row. A recording event always has a `recording_session_id` and may
also have an `ingest_token`; all other correlation families are exclusive, and
several governance families intentionally have none. The platform-ingest
identifier is now persisted and rehydrated by the audit DB adapter as part of
`X26-T3c-d`, resolving the blocker originally recorded by this matrix before
the always-on audit-boundary assertion is enabled.

## Source-backed matrix

| Event-kind family | Variants | Constructor and permitted correlation shape | Existing construction sites | `audit_events` persistence disposition |
|---|---|---|---|---|
| Ingestion | `IngestionFinalized`, `IngestionRejectedMissingRights`, `IngestionRejectedMissingUploaderContext`, `IngestionRejectedDuplicateToken` | `AuditEvent::new`: `ingest_token=Some`, `recording_session_id=None`, `platform_ingest_session_id=None`. | `apps/api/src/routes/ingestion.rs`, `crates/ingestion/src/lib.rs`; constructor unit tests under `crates/domain/src/audit/tests.rs`. | Persisted and read: both DB insert variants bind `event.ingest_token`; select/row mapping reads `ingest_token`. |
| Recording | `RecordingSessionCreated`, `RecordingRejectedMissingRights`, `RecordingCaptureStarted`, `RecordingRecorded`, `RecordingFailed`, `RecordingBridgedToAsset` | `AuditEvent::new_recording`: `recording_session_id=Some`; `ingest_token` is deliberately `Option<Uuid>`; platform ID is absent. **Combined recording+ingest is valid** when the call site has allocated an ingest token. | Current construction sites are covered by the constructor/unit-test surface. | Both nullable `ingest_token` and `recording_session_id` are persisted/read (`0009_alter_audit_events_for_recording.sql`; `crates/db/src/audit_repo.rs`). No database check constrains their combination. |
| Platform ingest | `PlatformIngestSessionCreated`, `PlatformIngestRejectedMissingRights`, `PlatformIngestDownloadStarted`, `PlatformIngestDownloaded`, `PlatformIngestFailed`, `PlatformIngestBridgedToAsset` | `AuditEvent::new_platform_ingest`: platform ID `Some`; ingest and recording IDs absent. | Current construction is covered by the constructor/unit-test surface. | **Resolved in X26-T3c-d:** `0030_add_platform_ingest_correlation_to_audit_events.sql` adds the nullable UUID column; both audit insert variants bind it and row mapping rehydrates it. |
| Workspace | `OrgCreated`, `OrgMemberAdded`, `ProjectCreated` | `AuditEvent::new_workspace_event`: all three correlation IDs absent. | `apps/api/src/routes/workspace.rs`; unit tests under `crates/domain/src/audit/tests.rs`. | Persisted with nullable correlation columns; no correlation ID is expected. |
| Consent | `ConsentGranted`, `ConsentRevoked`, `ConsentCheckDenied` | `AuditEvent::new_consent`: all three correlation IDs absent; `asset_id=Some`. | Consent gate/compliance call sites plus unit tests. | Persisted with no correlation ID; this is intentional governance-audit shape. |
| Review and publication | `ReviewApproved`, `ReviewRejected`, `PublicationSucceeded`, `PublicationRefused` | `AuditEvent::new_review_event`: all three correlation IDs absent; `asset_id=Some`. | `apps/api/src/review_gate.rs`; unit tests. | Persisted with no correlation ID; this is intentional governance-audit shape. |
| Playback | `PlaybackGrantIssued`, `PlaybackGrantRefused` | `AuditEvent::new_playback_event`: all three correlation IDs absent; `asset_id=Some`. | Playback service/audit call sites plus unit tests. | Persisted with no correlation ID; this is intentional governance-audit shape. |
| Authentication | `AuthLoginSucceeded`, `AuthLoginFailed`, `AuthRegistered` | `AuditEvent::new_auth_event`: all three correlation IDs absent. | `apps/api/src/routes/auth.rs`; unit tests. | Persisted with no correlation ID; this is intentional governance-audit shape. |

## Reproduction searches

Executed from the repository root on 2026-08-31:

```text
rg -n "AuditEvent::new\\(" . --glob '*.rs' --glob '!target/**'
rg -n "AuditEvent::new_recording\\(" . --glob '*.rs' --glob '!target/**'
rg -n "AuditEvent::new_platform_ingest\\(" . --glob '*.rs' --glob '!target/**'
rg -n "AuditEvent::new_(workspace_event|consent|review_event|playback_event|auth_event)\\(" . --glob '*.rs' --glob '!target/**'
rg -n "platform_ingest_session_id" infra/migrations crates/db apps crates --glob '*.rs' --glob '*.sql'
```

The first four searches identify the constructor families. After X26-T3c-d,
the final search also returns migration `0030`, both audit insert bindings,
the select projection, and `row_to_event` rehydration, so the former platform
persistence blocker is no longer present.

## Downstream disposition

- `X26-T3c-b1` enforces the ingestion-only shape.
- `X26-T3c-b2` enforces a mandatory recording ID while preserving the optional
  ingest-token combination.
- `X26-T3c-c1` enforces the platform-only in-memory shape; X26-T3c-d now makes
  that correlation durable as well.
- `X26-T3c-c2` and `X26-T3c-c3` enforce their intentional no-correlation
  shapes.
- `X26-T3c-d` may enforce the family-specific audit-boundary assertion because
  the only persistence blocker identified by this matrix has been resolved.
