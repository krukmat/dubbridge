# ADR-020: Recording session lifecycle and segment model

- **Status:** Proposed
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

## Context

Given FFmpeg subprocess orchestration (ADR-019), we need a well-defined model for
*how* a live stream is captured to disk reliably: how a session is started and
stopped, how the output is segmented to bound data loss and memory, how files are
named, and how completed segments are detected so they can be bridged into the
ingestion pipeline (ADR-021). These are standard segmented-recording techniques;
mature open-source media servers (primarily MediaMTX, MIT) were reviewed as
engineering references. This ADR fixes the concepts as an **original, clean-room
Rust design** — no third-party source code is used, and the reference material is
internal (not for publication).

## Decision

### Recording session state machine

A `RecordingSession` is the aggregate. Its status fails closed and never reaches a
capturing state without a validated rights basis (ADR-008):

```
Requested
  ├─(rights invalid)──────────────► RejectedMissingRights        (terminal)
  └─(rights valid)──► RightsValidated ──(start)──► Capturing
                                                     ├─(stop)──► Stopping ──► Recorded (terminal)
                                                     └─(error / max retries)─► Failed (terminal)
```

- `Recorded` means at least one complete, finalized output file exists and is
  ready to be bridged to an asset (ADR-021).
- `Failed` and `RejectedMissingRights` are terminal and audited (ADR-018).

### Segment / part model (standard technique)

- Output is written as **segments**; the default container is **fMP4** (fragmented
  MP4) for codec breadth and crash resilience; **MPEG-TS** is an alternative.
- An fMP4 segment is a concatenation of **parts**. **`record_part_duration`**
  (default ~1s) is the effective **Recovery Point Objective (RPO)** — the maximum
  data lost on an abrupt crash.
- **`record_max_part_size`** caps part size (default ~50 MB) to prevent memory
  exhaustion on high-bitrate inputs.
- **`record_segment_duration`** (default ~1h) bounds individual file size and
  enables incremental upload/bridging.
- **`record_delete_after`** (retention) governs local cleanup once a segment has
  been durably uploaded to object storage (ADR-006).

### Path templating (standard technique)

Output paths use a template with variables:
`%session/%Y-%m-%d/%H-%M-%S` → e.g. `recordings/{session_id}/2026-05-31/14-03-port.mp4`.
The template lives in `crates/recorder::segments` and resolves to a `storage_key`
under the `recordings/` prefix owned by `crates/storage`.

### Segment-complete signal (in-process event)

When the supervisor detects a completed segment (FFmpeg `segment` muxer rotation,
or graceful stop), it emits an internal **segment-complete event**. This is the
hook that triggers upload + the recording-to-asset bridge (ADR-021). We do not
shell out to external hook commands; the equivalent is an in-process Rust event.

### Process supervision

- Spawn via `tokio::process::Command` with piped `stdout`/`stderr`; parse progress
  lines for liveness and diagnostics.
- **Graceful stop** writes `q\n` to FFmpeg stdin so the current MP4 is finalized
  (not corrupted); a bounded timeout escalates to terminate.
- A **restart policy** (bounded retries with backoff) handles transient source
  drops; exceeding the budget transitions the session to `Failed`.

## Consequences

**Positive**
- Bounded data loss (RPO) and bounded memory via parts; bounded file size via
  segments.
- Crash-resilient finalization (fMP4) and clean graceful stop.
- A single, well-defined segment-complete event to drive ingestion and upload.
- Reuses a battle-tested design instead of inventing one.

**Negative / trade-offs**
- More moving parts than a single `ffmpeg -i ... out.mp4` invocation.
- Configuration surface grows (durations, sizes, retention) and needs sane
  defaults plus validation in `crates/config`.

## Alternatives considered

- **Single non-segmented output file** — rejected: unbounded file size, no RPO,
  high corruption risk on crash, no incremental upload.
- **External hook scripts (literal MediaMTX `runOnRecord*`)** — rejected: keeps
  orchestration in shell instead of Rust; we model the hook as an in-process event.

## Related

- ADR-019 (engine: FFmpeg subprocess) — the engine this lifecycle drives.
- ADR-021 (recording-to-asset bridge) — consumes the segment-complete event.
- ADR-018 (observability) — lifecycle transitions are audited.
