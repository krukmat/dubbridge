---
type: ADR
title: "ADR-022: Source protocol support (RTMP + SRT) and ingest authentication"
status: Proposed
---

# ADR-022: Source protocol support (RTMP + SRT) and ingest authentication

- **Status:** Proposed — **scope narrowed to the S3b live-recording sub-case
  (2026-05-31 replan)**
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

> **2026-05-31 scope note (S3 replan, see ADR-025).** RTMP/SRT source protocols and
> capture-edge authentication govern the **deferred S3b live-recording sub-case**.
> The primary S3 intake is owner-authorized platform download; its connector
> authentication and owner-credential model are governed by **ADR-025**, not this
> ADR. Both share the same fail-closed, credential-redaction posture.

## Context

The recording module must capture from live sources. The protocol surface
directly drives complexity, the FFmpeg input configuration, and the security
posture. We must scope v1 deliberately and define how authorized-only ingestion
(ADR-008) is enforced at the protocol edge, since "we only process authorized
content" must also mean "we only *capture* from authorized sources".

This source-authentication layer is separate from API client authentication
(ADR-023). ADR-023 authenticates the caller invoking Axum recording endpoints;
this ADR authenticates the RTMP/SRT source before the recorder connects.

## Decision

- **v1 supports two source protocols: RTMP and SRT.**
  - **RTMP** — ubiquitous for OBS and software/hardware encoders (`rtmp://.../app/{stream_key}`).
  - **SRT** — low-latency professional contribution (`srt://host:port?streamid=...&passphrase=...`).
  - These cover the two dominant live-contribution cases with a bounded test matrix.
- **RTSP, HLS pull, and WebRTC are explicitly out of scope for v1** and recorded as
  follow-ups; they are not blocked architecturally — the source is modeled as an
  enum (`SourceProtocol`) so adding a variant is additive.
- **Ingest authentication is mandatory and fails closed:**
  - **RTMP** captures require a **stream key** that is validated against the
    `RecordingSession` before the supervisor connects; unknown/expired keys are
    refused.
  - **SRT** captures require a **passphrase** (encryption) and a `streamid` bound
    to the session; unencrypted SRT is rejected by default.
  - Credentials are **never logged**; they are redacted in traces (ADR-018) and
    stored as references/secrets, not in plaintext audit detail.
- The protocol/source is part of the rights provenance: a session records its
  `SourceProtocol` and the resolved rights `source_type`
  (`InternalFeed` / `LicensedSource`, ADR-021), so "where did this come from" is
  auditable.
- Source URLs are **validated and normalized** before being passed to the FFmpeg
  command builder, to prevent argument injection and to enforce the allowed scheme
  set (`rtmp`, `srt` only in v1).

## Consequences

**Positive**
- Bounded, realistic v1 scope with a small protocol/test matrix.
- Authorized-only capture is enforced at the edge, consistent with ADR-008.
- The enum-based source model keeps future protocols additive.

**Negative / trade-offs**
- Encoders/contributors must be provisioned with stream keys / SRT passphrases;
  there is operational setup before a capture can start.
- WebRTC (browser capture) — a likely future ask — needs a signaling/SFU component
  and is deliberately deferred; expectations must be set.

## Alternatives considered

- **RTMP only** — rejected: excludes low-latency professional SRT contribution that
  is a common real-world need.
- **All protocols (incl. WebRTC) in v1** — rejected: WebRTC requires significant
  additional infrastructure (signaling, ICE/SFU) and would balloon v1 scope and
  risk.
- **Open ingest (no auth)** — rejected outright: incompatible with the
  authorized-only, fail-closed posture (ADR-008).

## Related

- ADR-008 (authorized-only, fail-closed).
- ADR-023 (API client authentication) — separate caller identity boundary.
- ADR-019 (FFmpeg engine) — protocols map to FFmpeg input options.
- ADR-021 (recording-to-asset bridge) — `source_type` provenance.
