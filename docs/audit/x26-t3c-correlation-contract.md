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
identifier is created in the domain but is not persistently mapped, so an
always-on audit-boundary assertion claiming durable platform correlation is
blocked until that gap is resolved by separately scoped work.

## Source-backed matrix

| Event-kind family | Variants | Constructor and permitted correlation shape | Existing construction sites | `audit_events` persistence disposition |
|---|---|---|---|---|
| Ingestion | `IngestionFinalized`, `IngestionRejectedMissingRights`, `IngestionRejectedMissingUploaderContext`, `IngestionRejectedDuplicateToken` | `AuditEvent::new` (`crates/domain/src/audit.rs:115-132`): `ingest_token=Some`, `recording_session_id=None`, `platform_ingest_session_id=None`. | `apps/api/src/routes/ingestion.rs:376`, `crates/ingestion/src/lib.rs:208,271`; constructor unit test `crates/domain/src/audit.rs:280`. | Persisted and read: both DB insert variants bind `event.ingest_token`; select/row mapping reads `ingest_token` (`crates/db/src/audit_repo.rs:80-119,135-150`). |
| Recording | `RecordingSessionCreated`, `RecordingRejectedMissingRights`, `RecordingCaptureStarted`, `RecordingRecorded`, `RecordingFailed`, `RecordingBridgedToAsset` | `AuditEvent::new_recording` (`crates/domain/src/audit.rs:134-152`): `recording_session_id=Some`; `ingest_token` is deliberately `Option<Uuid>`; platform ID is absent. **Combined recording+ingest is valid** when the call site has allocated an ingest token. | The only current construction is the constructor unit test at `crates/domain/src/audit.rs:266`; repository search found no production `new_recording` caller. | Both nullable `ingest_token` and `recording_session_id` are persisted/read (`0009_alter_audit_events_for_recording.sql`; `crates/db/src/audit_repo.rs:80-150`). No database check constrains their combination. |
| Platform ingest | `PlatformIngestSessionCreated`, `PlatformIngestRejectedMissingRights`, `PlatformIngestDownloadStarted`, `PlatformIngestDownloaded`, `PlatformIngestFailed`, `PlatformIngestBridgedToAsset` | `AuditEvent::new_platform_ingest` (`crates/domain/src/audit.rs:154-171`): platform ID `Some`; ingest and recording IDs absent. | The only current construction is the constructor unit test at `crates/domain/src/audit.rs:291`; repository search found no production `new_platform_ingest` caller. | **Persistence blocker:** no migration adds `platform_ingest_session_id`; DB inserts/selects omit it and `row_to_event` deliberately reconstructs it as `None` (`crates/db/src/audit_repo.rs:61-72,80-150`). |
| Workspace | `OrgCreated`, `OrgMemberAdded`, `ProjectCreated` | `AuditEvent::new_workspace_event` (`crates/domain/src/audit.rs:174-187`): all three correlation IDs absent. | `apps/api/src/routes/workspace.rs:147,194,242`; unit test `crates/domain/src/audit.rs:336`. | Persisted as nullable ingestion/recording fields; no correlation ID is expected. |
| Consent | `ConsentGranted`, `ConsentRevoked`, `ConsentCheckDenied` | `AuditEvent::new_consent` (`crates/domain/src/audit.rs:189-205`): all three correlation IDs absent; `asset_id=Some`. | `apps/api/src/consent_gate.rs:79,126`, `apps/api/src/dto/compliance.rs:133`, `apps/api/src/routes/compliance_tests.rs:87`, `apps/api/tests/compliance_test.rs:133,143,185`; unit test `crates/domain/src/audit.rs:480`. | Persisted with no correlation ID; this is intentional governance-audit shape. |
| Review and publication | `ReviewApproved`, `ReviewRejected`, `PublicationSucceeded`, `PublicationRefused` | `AuditEvent::new_review_event` (`crates/domain/src/audit.rs:207-223`): all three correlation IDs absent; `asset_id=Some`. | `apps/api/src/review_gate.rs:142,162,186`; unit test `crates/domain/src/audit.rs:497`. | Persisted with no correlation ID; this is intentional governance-audit shape. |
| Playback | `PlaybackGrantIssued`, `PlaybackGrantRefused` | `AuditEvent::new_playback_event` (`crates/domain/src/audit.rs:225-241`): all three correlation IDs absent; `asset_id=Some`. | `apps/api/src/playback_service.rs:360`, `apps/api/src/playback_audit.rs:16`; unit test `crates/domain/src/audit.rs:529`. | Persisted with no correlation ID; this is intentional governance-audit shape. |
| Authentication | `AuthLoginSucceeded`, `AuthLoginFailed`, `AuthRegistered` | `AuditEvent::new_auth_event` (`crates/domain/src/audit.rs:243-255`): all three correlation IDs absent. | `apps/api/src/routes/auth.rs:105`; unit test `crates/domain/src/audit.rs:512`. | Persisted with no correlation ID; this is intentional governance-audit shape. |

## Reproduction searches

Executed from the repository root on 2026-08-31:

```text
rg -n "AuditEvent::new\\(" . --glob '*.rs' --glob '!target/**'
rg -n "AuditEvent::new_recording\\(" . --glob '*.rs' --glob '!target/**'
rg -n "AuditEvent::new_platform_ingest\\(" . --glob '*.rs' --glob '!target/**'
rg -n "AuditEvent::new_(workspace_event|consent|review_event|playback_event|auth_event)\\(" . --glob '*.rs' --glob '!target/**'
rg -n "platform_ingest_session_id" infra/migrations crates/db apps crates --glob '*.rs' --glob '*.sql'
```

The first four searches identify every direct constructor occurrence as of this
matrix. The last search returns the domain field and DB's explicit `None`, but
no migration, insert column/bind, or select column for the platform identifier.

## Downstream disposition

- `X26-T3c-b1` may enforce the ingestion-only shape.
- `X26-T3c-b2` may enforce a mandatory recording ID while preserving the
  optional ingest-token combination.
- `X26-T3c-c1` may enforce the in-memory platform-only shape, but must name
  its non-durability in test comments.
- `X26-T3c-c2` and `X26-T3c-c3` may enforce their intentional no-correlation
  shapes.
- `X26-T3c-d` is **blocked**: do not add an audit-boundary assertion until a
  separately authorized persistence decision either maps
  `platform_ingest_session_id` through schema/DB reads and writes or removes
  the unsupported platform event contract.
