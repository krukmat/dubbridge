# ADR-025: Platform connector ingest and owner-authorized credential model

- **Status:** Proposed
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team
- **Supersedes (as primary S3 path):** the RTMP/SRT-first framing in ADR-019/020/022

## Context

The S3 intake use case was re-scoped on 2026-05-31. The primary intake mode is no
longer "a client points their live encoder at DubBridge" (RTMP/SRT capture).
The primary mode is:

> The **content owner provides credentials to their own platform account**
> (YouTube, Vimeo, etc.) and DubBridge **downloads the owner's content on their
> behalf**.

This is legal and authorized because the client owns the content and grants
explicit, scoped access to their own account. RTMP/SRT live capture remains valid
but only for the minority of clients who produce live broadcasts; it is demoted to
a deferred sub-slice (S3b) governed by ADR-019/020/022.

This ADR fixes the architecture and the credential/governance model for the
platform-download primary path. It does not change the asset boundary: every
downloaded file still becomes an asset through the single fail-closed finalize gate
(ADR-008, ADR-021).

## Decision

### 1. Per-connector architecture behind a trait boundary

A new control-plane crate (`crates/connectors`) owns platform integrations. Each
platform is a `PlatformConnector` implementation. The boundary mirrors the
established `crates/media` / `crates/recorder` split between a **pure, deterministic
request builder** and an **IO executor**:

```text
crates/connectors
  src/lib.rs        -> PlatformConnector trait, Platform enum, shared types
  src/youtube.rs    -> YouTube connector (v1)
  src/vimeo.rs      -> Vimeo connector (later)
```

Trait shape (illustrative; final signatures decided in the implementing task):

```rust
pub enum Platform { YouTube, Vimeo /* extensible */ }

/// What the caller asks for: an owner-owned item on a platform.
pub struct SourceRef { pub platform: Platform, pub external_id: String }

/// Owner-authorized credential, resolved from the secrets store by reference.
pub struct ConnectorCredential { /* opaque; never logged */ }

pub struct RemoteMediaMetadata { /* title, duration, mime, size hint, ownership */ }
pub struct DownloadedMedia { pub local_path: PathBuf, pub bytes: u64, pub content_type: String }

#[async_trait]
pub trait PlatformConnector {
    fn platform(&self) -> Platform;
    /// Verify the item exists and is owned/accessible by the credential. Pure-ish:
    /// one authenticated metadata call, no large transfer.
    async fn resolve(&self, src: &SourceRef, cred: &ConnectorCredential)
        -> Result<RemoteMediaMetadata, ConnectorError>;
    /// Download the owner's media to local staging.
    async fn download(&self, src: &SourceRef, cred: &ConnectorCredential, dest: &Path)
        -> Result<DownloadedMedia, ConnectorError>;
}
```

- The **request/URL builder** portion of each connector is a pure function,
  unit-testable without network (the `ffmpeg_record_command` discipline of ADR-019).
- The **executor** is the only part that performs authenticated network IO.
- `crates/connectors` has **no DB dependency**; it depends on `crates/domain` and
  `crates/config`, exactly like `crates/recorder` was specified to.

### 2. YouTube connector is v1; retrieval mechanism validated by a spike

YouTube does not expose a single Data-API endpoint that returns the original media
bytes for arbitrary videos. The legitimate owner-download mechanism (YouTube Data
API for metadata + ownership verification, plus the owner-authorized retrieval of
their own media) must be validated by a throwaway internal spike **before** the
connector is implemented — the same gate discipline T0c applied to FFmpeg.

The spike fixes, for v1:
- the OAuth scope(s) the owner must grant,
- how ownership is verified (`resolve`),
- the concrete retrieval mechanism for owner-authorized media,
- output container/quality contract for the downloaded file.

The trait boundary keeps the chosen mechanism swappable without touching callers.

### 3. Owner-authorized credential model (fail-closed, redacted)

- Credentials are **owner-provided, scoped to the owner's own account**, and stored
  **by reference** in the secrets store — never in plaintext columns. (No dedicated
  secrets ADR exists yet; the secrets-store mechanism is an open decision tracked as
  a follow-up — see X-new in `docs/plan/roadmap.md`.)
- The DB persists only a credential reference/handle and non-secret connection
  metadata. Tokens, refresh tokens, and API keys are redacted from all logs and
  traces (ADR-018), identical posture to RTMP keys / SRT passphrases (ADR-022).
- Authorization is captured **before download starts**. A session without a valid
  `RightsBasis` **and** a valid owner credential is rejected before any bytes are
  transferred — the platform-download twin of the capture-edge gate in ADR-022.

### 4. Intake session lifecycle (download, not capture)

Platform ingest uses a download lifecycle, not the segment/capture lifecycle of
ADR-020:

```text
Requested
  ├─(rights or credential invalid)──► RejectedMissingRights   (terminal, audited)
  └─(valid)──► RightsValidated ──► Resolving ──► Downloading ──► Downloaded
                                                     └─(error/max retries)─► Failed
```

- `Resolving` performs `resolve()` (ownership + metadata).
- `Downloaded` means a complete local file exists and is ready for the bridge.
- `Failed` and `RejectedMissingRights` are terminal and audited (ADR-018).

This reuses the S3-T1 fail-closed validation and audit posture but is a distinct
state set from `RecordingStatus` (no `Capturing`/`Stopping`).

### 5. Same asset boundary (ADR-021, generalized)

A downloaded file is bridged into the **existing S1 finalize path** via the now
generalized ADR-021 bridge: SHA-256 over the downloaded file, one artifact row, one
`ingest_token` per ingest session, fail-closed rights. The artifact kind is a new
`ArtifactKind::DownloadedPlatformMedia` (additive, like `RecordedStreamMedia`), so
lineage distinguishes platform downloads from uploads and live recordings.

## Consequences

**Positive**
- The primary intake path now matches the real, legal use case.
- One ingestion gate, one rights invariant, one audit story — no bypass (ADR-021).
- Connector trait makes new platforms (Vimeo, …) additive without touching the
  finalize path or the API contract shape.
- Pure-builder/executor split keeps connectors unit-testable without network.

**Negative / trade-offs**
- Owner-credential handling adds a secrets-store integration surface that must be
  audited for redaction and scope minimization.
- The exact YouTube retrieval mechanism is not yet fixed and needs a spike before
  implementation (risk mitigated by the trait boundary).
- The domain grows a second intake aggregate/state set alongside recording.

## Alternatives considered

- **Keep RTMP/SRT as the primary path** — rejected: it does not match the actual
  use case; most clients are not live broadcasters.
- **Reuse `RecordingStatus` for downloads** — rejected: `Capturing`/`Stopping` are
  meaningless for a download; a distinct lifecycle is clearer and still fail-closed.
- **A separate finalize path for downloads** — rejected for the same reason ADR-021
  rejected it for recordings: it would create a weaker parallel ingestion gate.
- **A single mega-connector with platform `if/else`** — rejected: a trait per
  connector keeps platform quirks isolated and testable.

## Related

- ADR-008 (rights ledger, fail-closed) — the gate reused here.
- ADR-021 (intake-to-asset bridge) — generalized to cover platform downloads.
- ADR-006 (metadata + object storage) — artifact rows and checksums.
- ADR-018 (observability) — lifecycle transitions and credential redaction.
- ADR-019/020/022 — RTMP/SRT live recording, now the deferred S3b sub-case.
- Plan: `docs/plan/stream-recording-ingest.md`; Tasks:
  `docs/tasks/stream-recording-ingest.md`.
