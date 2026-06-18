---
type: ADR
title: "ADR-019: Stream recording engine — FFmpeg subprocess orchestration"
status: Proposed
---

# ADR-019: Stream recording engine — FFmpeg subprocess orchestration

- **Status:** Proposed — **scope narrowed to the S3b live-recording sub-case
  (2026-05-31 replan)**
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

> **2026-05-31 scope note (S3 replan, see ADR-025).** The primary S3 intake path is
> now owner-authorized **platform download** (ADR-025), not RTMP/SRT live capture.
> The FFmpeg-subprocess decision in this ADR remains valid and unchanged, but it now
> governs the **deferred S3b live-recording sub-case** only. It is not on the
> primary S3 critical path. No technical decision here is reversed; only its slice
> placement changes.

## Context

A new requirement asks DubBridge to **record a live stream** (RTMP, SRT) into a
multimedia file that is then incorporated into the platform as an asset. We need a
capture engine that:

1. Fits the **Rust-first** architecture: Rust owns orchestration, governance, and
   quality gates; non-Rust media tooling is acceptable only behind a clear
   boundary (`docs/architecture.md`).
2. Reuses existing patterns. `crates/media` already treats media tooling as an
   **external process invoked via a command builder** (`ffprobe_command`), not as
   a linked native library.
3. Keeps the **rights fail-closed** invariant (ADR-008) and **explicit lineage**
   (ADR-006) under Rust's control.
4. Avoids introducing a new long-running runtime/language into the operational
   surface unless justified.

Open-source options were evaluated (see
`docs/proposals/stream-recording-module.md`):
FFmpeg (subprocess), GStreamer (`gstreamer-rs`, in-process), MediaMTX (Go sidecar,
MIT), SRS (C++ sidecar, MIT). MediaMTX has the cleanest recording subsystem and was reviewed as the primary
engineering reference, but it is a separate Go service.

## Decision

- **The v1 capture engine is FFmpeg, orchestrated by Rust as a supervised
  subprocess**, inside a new `crates/recorder` control-plane crate.
- The engine boundary mirrors `crates/media`: a **pure, deterministic command
  builder** (`recorder::ffmpeg::ffmpeg_record_command`) produces the argument
  vector; a separate supervisor (`recorder::session`) spawns and manages the
  process via `tokio::process`.
- The recorder uses **standard segmented-recording techniques** (segment/part model,
  path templating, retention, segment-complete hook — see ADR-020), **independently
  implemented (clean-room) in Rust**. Open-source media servers (primarily MediaMTX,
  MIT) were reviewed as engineering references only; **no third-party source code is
  used.** Competitive/reference material stays internal (not for publication).
- FFmpeg is consumed **only via subprocess** (no linked GPL libraries). Builds must
  remain within FFmpeg's LGPL configuration; GPL-only components are not used.
- Before recorder implementation, S3 Task T0c validates the concrete FFmpeg
  segmentation, graceful-stop, recovery, and output-artifact shape with a local
  synthetic-source spike. The subprocess decision is fixed; the v1 output contract
  is not.
- **GStreamer (`gstreamer-rs`)** is recorded as the sanctioned **future upgrade
  path** for in-process, low-latency pipelines, behind the same `crates/recorder`
  trait, if/when subprocess overhead or control granularity becomes a constraint.
- **MediaMTX/SRS as a sidecar** is the documented **fallback** if time-to-market
  ever outweighs Rust-native ownership; it is not adopted now because it adds a Go
  runtime and moves capture (and thus part of lineage control) outside Rust.

## Consequences

**Positive**
- Zero architectural friction: identical pattern to `ffprobe_command`.
- No new language/runtime in production; FFmpeg is a single, ubiquitous binary.
- Rust retains full control of lineage, rights gating, and audit at capture time.
- The command-builder/supervisor split keeps the engine unit-testable without
  spawning processes.

**Negative / trade-offs**
- We must build session lifecycle, segment management, health and restart logic
  ourselves (mitigated by following a proven, well-documented design — ADR-020).
- Subprocess management (graceful stop, stdin `q`, zombie/timeout handling) is
  inherently fiddly and needs careful, well-tested code.
- FFmpeg becomes a required runtime dependency of the worker-runner image.

## Alternatives considered

- **GStreamer in-process (`gstreamer-rs`)** — strong Rust-native fit but heavier
  system dependency (GStreamer + plugin set), steeper learning curve, and a
  plugin-licensing matrix. Deferred to a future upgrade, not rejected outright.
- **MediaMTX sidecar (Go, MIT)** — production-grade recording out of the box, but
  adds a Go runtime and pushes capture/lineage outside Rust. Kept as fallback and as
  a reviewed engineering reference.
- **SRS sidecar (C++, MIT)** — very featureful but introduces a C++ runtime and a
  larger operational surface; least aligned with Rust-first ownership.

## Related

- ADR-020 (recording lifecycle and segment model) — segment model (industry-standard pattern; reference reviewed).
- ADR-021 (recording-to-asset bridge) — how captures become assets under ADR-008.
- ADR-022 (source protocols and ingest auth) — RTMP/SRT scope and authorization.
- Proposal: `docs/proposals/stream-recording-module.md`.
