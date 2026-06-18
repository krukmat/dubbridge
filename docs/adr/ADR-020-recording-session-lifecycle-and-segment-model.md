---
type: ADR
title: "ADR-020: Recording session lifecycle and segment model"
status: Accepted
---

# ADR-020: Recording session lifecycle and segment model

- **Status:** Accepted — **applies to the S3b live-recording sub-case
  (2026-05-31 replan)**
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

> **2026-05-31 scope note (S3 replan, see ADR-025).** This segment/capture lifecycle
> governs **live recording (S3b)**. The primary S3 platform-download path uses the
> simpler download lifecycle (`Requested → RightsValidated → Resolving →
> Downloading → Downloaded`) defined in ADR-025, not the `Capturing/Stopping`
> segment model below. The fail-closed rights invariant is identical in both.

## Context

Given FFmpeg subprocess orchestration (ADR-019), we need a well-defined model for
*how* a live stream is captured to disk reliably: how a session is started and
stopped, how the output is segmented to bound data loss, how files are named, and
which finalized output becomes the asset boundary (ADR-021). These are standard
segmented-recording techniques; mature open-source media servers (primarily
MediaMTX, MIT) were reviewed as engineering references. This ADR fixes the
concepts as an **original, clean-room Rust design** — no third-party source code is
used, and the reference material is internal (not for publication).

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

### Validated v1 output contract

S3 Task T0c validated the v1 recording shape with a local synthetic-source spike:

- FFmpeg can write a local **HLS fMP4 package** consisting of:
  - `init.mp4`
  - `session.m3u8`
  - `seg_000.m4s`, `seg_001.m4s`, ...
- A **graceful stop** via `q\n` finalizes the last segment and writes
  `#EXT-X-ENDLIST`.
- After a graceful stop, FFmpeg can remux `session.m3u8` into one
  **assembled whole-session MP4** using `ffmpeg -i session.m3u8 -c copy assembled.mp4`.
- After a hard kill, completed segments remain on disk but the manifest is left
  open (no `ENDLIST`), and direct remux of the open manifest may block waiting for
  more input.

V1 therefore chooses:

- **Local segmented staging** during capture: HLS fMP4 (`init.mp4` + manifest +
  `.m4s` segments) under the session-local recording directory.
- **One whole-session assembled multimedia artifact** as the asset boundary.
- **Bridge only after a clean stop or explicit recovery step**, never from an
  in-progress or crash-open manifest.

This is a deliberate separation:

- segments are an internal recorder staging format
- the asset boundary is one finalized MP4 per recording session

### Segment model

- **`record_segment_duration`** is the operator-facing control for local staging.
  The T0c spike used `2s` segments to make rotation observable. V1 should ship with
  a modest default (for example `10s`) rather than the previous placeholder `~1h`,
  so crash loss is bounded without creating excessive file churn.
- `record_segment_duration` bounds local loss on crash to roughly the current open
  segment plus encoder buffering.
- The previous speculative **part** controls are **not part of v1**. T0c did not
  validate an FFmpeg-native part-level contract that this design can rely on, so
  `record_part_duration` and `record_max_part_size` are deferred rather than
  treated as guaranteed invariants.
- **`record_delete_after`** governs local cleanup of staging files after either:
  - successful assembly + upload + bridge, or
  - failed sessions that age out without recovery.

### Path templating (standard technique)

Output paths use a template with variables:
`%session/%Y-%m-%d/%H-%M-%S` → e.g. `recordings/{session_id}/2026-05-31/14-03-00/`.
The local staging directory contains `init.mp4`, `session.m3u8`, and rotated
segment files. The template lives in `crates/recorder::segments`.

The final object-storage key is separate from the staging template and points to
the assembled MP4 (for example under `recordings/{session_id}/final/session.mp4`).

### Segment-complete signal (in-process event)

When the supervisor detects a completed segment rotation, it emits an internal
**segment-complete event**. In v1, this event is for local bookkeeping and liveness,
not for immediate asset creation. The bridge runs once per session after a clean
stop and successful whole-session assembly.

### Crash recovery boundary

- A clean stop is the normal path to `Recorded`.
- A hard crash may still leave multiple complete `.m4s` segments plus a manifest
  without `ENDLIST`.
- V1 does **not** auto-bridge crashed sessions. They transition to `Failed` and
  keep their local staging files for bounded retention or operator-assisted future
  recovery.

### Process supervision

- Spawn via `tokio::process::Command` with piped `stdout`/`stderr`; parse progress
  lines for liveness and diagnostics.
- **Graceful stop** writes `q\n` to FFmpeg stdin so the current MP4 is finalized
  (not corrupted); a bounded timeout escalates to terminate.
- A **restart policy** (bounded retries with backoff) handles transient source
  drops; exceeding the budget transitions the session to `Failed`.

## Consequences

**Positive**
- Preserves bounded local loss on crash through rotated fMP4 segments while keeping
  the downstream asset contract simple: one finalized MP4 per session.
- Graceful stop cleanly finalizes the manifest and last segment, making remux to a
  single MP4 deterministic.
- Keeps downstream S1/S4-style consumers on a conventional single-file artifact
  instead of introducing per-segment or manifest-native assets in v1.
- Avoids a `recording_segments` schema in v1; segment lineage remains an internal
  staging concern.

**Negative / trade-offs**
- More moving parts than a single `ffmpeg -i ... out.mp4` invocation.
- V1 does not automatically recover crashed sessions into assets; the session fails
  closed and requires bounded local retention or a future recovery workflow.
- Configuration surface still grows (segment duration, retention, assembly path)
  and needs sane defaults plus validation in `crates/config`.

## Alternatives considered

- **Single non-segmented output file** — rejected: unbounded file size, no RPO,
  high corruption risk on crash, no incremental upload.
- **Manifest-backed session artifact** — rejected for v1: the HLS package is easy
  to produce, but making a manifest the first-class asset would force downstream
  consumers to learn segment resolution before S4 is even built.
- **Per-segment assets** — rejected for v1: it explodes one recording session into
  many assets and makes rights/audit/idempotency behavior harder than necessary.
- **External hook scripts (literal MediaMTX `runOnRecord*`)** — rejected: keeps
  orchestration in shell instead of Rust; we model the hook as an in-process event.

## Related

- ADR-019 (engine: FFmpeg subprocess) — the engine this lifecycle drives.
- ADR-021 (recording-to-asset bridge) — consumes the assembled whole-session output.
- ADR-018 (observability) — lifecycle transitions are audited.
