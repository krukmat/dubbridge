---
type: ReviewPacket
task: P1.A2
phase: task-analysis
status: pending_review
date: 2026-08-30
---

# P1.A2 phase-1 task-analysis packet

## Goal

Implement proof-only transient synthetic seed storage: deterministically
generate and hash a synthetic fixture, write it through Hyperdrive/Corestore
within one validated Expo-cache run directory, close handles before deleting
that exact directory, verify absence, and run a bounded startup janitor for
abandoned marked proof runs.

## Frozen scope

- `mobile/src/p2p/runtime/worklet.ts`
- `mobile/src/p2p/runtime/protocol.ts`
- generated `mobile/src/p2p/runtime/worklet.bundle.js`
- `mobile/src/p2p/proof/P1ProofRuntimeFactory.ts`
- new `mobile/src/p2p/proof/transient-storage.ts`
- new `mobile/src/p2p/proof/P1SeedProofRunner.ts`
- new `mobile/__tests__/p2p/transient-seed.test.ts`
- P1.A2 audit/card evidence only

## Required behavior

- **HP-A2:** the seed receipt exposes only byte count and SHA-256; shutdown
  closes all handles, removes exactly the run directory, and proves it absent.
- **EC-A2:** traversal/foreign path, write/hash/close/delete failure, or an
  abandoned run is rejected or boundedly janitored without touching anything
  outside `Paths.cache/dubbridge-p2p/proofs`.
- The worklet receives the factory-derived `file:` URI via `Bare.argv[0];
  directory paths, fixture contents, keys, raw errors, discovery keys, and
  network activity are never logged or returned.
- The janitor requires the proof-root boundary, a valid run-id/marker, and
  the defined age bound. Cleanup failure is terminal, never PASS.

## Explicit exclusions

No Hyperswarm/discovery/replication/client worklet, `P2PService` change,
product startup, public API, durable identity/cache, user media, HTTP, keys,
or iOS work.

## Design constraints and verification

ADR-043 requires close-before-delete, post-delete nonexistence verification,
and bounded crash-residue cleanup. The prior P1.A1b.0 contract freezes
`file:` URI handoff and keeps the proof topology outside `P2PService`.

Tests must independently cover valid seed/receipt/cleanup, rejected foreign
or traversal paths, write/hash failure, close/delete failure, retained
unmarked or fresh directories, eligible stale marked residue removal, and no
discovery invocation. Required checks: focused Jest test, worklet build and
drift check, mobile typecheck, lint, and full Jest suite. Android evidence is
redacted and is not a claim of X28 resolution.

## RRI and route

RRI 46 Med-high, Effort L. Per ADR-038 it needs an explicit owner approval;
after approval it must produce Muse Glimmer refinement plus the hash-bound
receipt, then takes the cloud route regardless of `GO_LOCAL` (unless a later
ADR-040 module split qualifies). Phase-1 reviewer: Gemma; fallback Muse
Glimmer then D14.

## Reviewer request

Return structured JSON with `verdict` (`PASS` or `BLOCKED`) and findings by
severity. Assess scope completeness, cleanup ownership/order, janitor safety,
testable HP/EC coverage, and whether any missing constraint would make the
approval card unsafe.
