---
type: Plan
title: "P7Local: Local end-to-end P2P pre-certification"
status: planned
slice: MVP0-P2P
---

# P7Local — Local end-to-end P2P pre-certification

Task ledger: `docs/tasks/mvp0-p2p-p7-local-certification.md`.

## Objective

Provide a reproducible local Android pre-certification of the complete MVP0-P2P product flow before exact-artifact P7. P7Local proves that the implemented owner/viewer use cases compose correctly on a developer-controlled environment; it does **not** replace phase PASS criteria, release gates, or P7 exact-artifact certification.

## Position in the delivery sequence

The intended sequence is:

`P5 device certification -> P6 product use cases -> P7Local local E2E -> P7 exact-artifact certification`.

P7Local may be prepared earlier as documentation and harness planning, but behavioral execution of the full owner-to-viewer flow remains blocked until the required P5/P6 capabilities are PASS at their authoritative gates.

## Scope

One owner, one invited viewer/device, one short video, local/control-plane services required by the existing MVP0-P2P architecture, and the Android application built from the active branch/revision under test.

The required product path is:

`owner content -> invite -> viewer claim -> sync -> package verification -> Available -> local loopback playback`.

Negative coverage includes expired/denied invitation, incomplete or tampered package, playback failure/teardown, retry behavior, logout/account switch and cross-account isolation.

P7Local does not add analytics, payments, offline entitlement, multi-device management, performance certification, DRM, Studio Web, community features or a new audience-media fallback.

## Boundary and non-goals

- P7Local is a **pre-certification** environment, not a release artifact certification.
- P7Local never upgrades P5 or P6 to PASS by documentation alone; their own required evidence still applies.
- P7Local does not waive P7's requirement to certify the exact deployed/RC artifact and prove legacy HTTP/S3 audience-media fallback is disabled.
- P7Local does not redefine O3/K1/O4, ADR-032 review delivery, backend authorization, P4 verification, P5 transient-key handling or loopback-only playback.
- Existing review playback remains a separate product path and must not regress.

## Canonical use-case contract

### CU-P2P-01 — Owner can invite from eligible content

An authenticated owner sees their own P2P publication state. Invite is available only when canonical backend/runtime facts make the package eligible. Non-ready, failed or foreign-owner content cannot expose an effective invite action.

### CU-P2P-02 — Viewer can claim the intended invitation

The intended viewer can enter/open and claim a valid invitation through P3 authorization. Expired, denied, already-invalid or another-viewer invitations fail closed and do not leak package access.

### CU-P2P-03 — Viewer can start and observe sync

A claimed authorized invitation can start P4 sync. The UI projects progress/status from authoritative runtime facts rather than optimistic local state. Cancellation/retry never creates a second effective ownership path for the same session.

### CU-P2P-04 — Available requires verified package state

`Available` and `can_play` are enabled only after the downloaded ciphertext package has passed the P4 verification contract. Partial, altered, missing or unverified package state never enables playback.

### CU-P2P-05 — Viewer plays from the local encrypted copy

Playback starts through the existing P5 session capability, unwraps/decrypts only for the authorized session, serves HLS only over the scoped `127.0.0.1` loopback gateway and renders through the existing VideoPlayer seam. No HTTP/S3 audience-media fallback is permitted.

### CU-P2P-06 — Failures remain fail-closed and recoverable where allowed

Expired invitation, unavailable key material, tamper, decrypt/authentication failure, interrupted sync and playback error produce an explicit non-playable/error state. Retry is offered only where the underlying contract permits it and must not bypass authorization, verification or teardown.

### CU-P2P-07 — Account/session lifecycle is isolated

Logout, account switch, view unmount and playback stop/error deterministically release session ownership and transient key/gateway state. Content/action state from the previous account is not reused or displayed as authorized for the new account.

## Evidence profile

P7Local should emit a compact evidence bundle tied to a single branch revision/build identity. At minimum record:

- branch and commit SHA under test;
- Android build identity/device or emulator profile;
- backend/runtime profile used by the run;
- owner/viewer identities represented by redacted test aliases;
- ordered CU results with PASS/FAIL and timestamps;
- relevant application/runtime logs for sync, verification and playback lifecycle;
- visual evidence only where it adds product-state value;
- negative-control results for tamper, expiry/denial and account isolation.

Screenshots or Maestro flows are supporting evidence, not substitutes for crypto/runtime/network/lifecycle assertions.

## Planned work

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P7L.T0 | Freeze local certification profile, CU matrix and evidence manifest | planning | S | P5 implementation complete; P6 contract available | Planned; documentation may be prepared early |
| P7L.T1 | Certify owner -> invite -> viewer -> claim -> sync -> verify -> Available -> local play | integration/e2e evidence | M | P5 PASS; P6.T2 PASS | Planned; blocked |
| P7L.T2 | Certify failure, tamper, retry, teardown and account isolation controls | integration/e2e evidence | M | P7L.T1 PASS | Planned; blocked |
| P7L.T3 | Produce local verdict and handoff deltas for P6.T3/P7 | evidence/handoff | S | P7L.T2 evidence complete | Planned; blocked |

## Relationship with P6 and Maestro

P6 owns the product UI and use cases. P7Local consumes those use cases as an end-to-end local product flow. P6.T3 may use Maestro or equivalent UI automation for deterministic visual/interaction evidence, but P7Local additionally requires runtime assertions for authorization, verified sync, local playback, teardown, tamper and account isolation.

The existing Maestro suite does not need broad pre-emptive expansion before P6. P2P-specific flows should be added when the P6 screens/actions exist, then reused as one layer of P7Local evidence.

## Closure and downstream handoff

P7Local PASS means the complete local owner-to-viewer MVP0-P2P path and required negative controls have reproducible evidence on the recorded local profile. It does **not** imply P7 PASS.

Any failure that exposes a defect in P3-P6 reopens or blocks the owning phase/task until corrected and re-evidenced. P7 consumes the stabilized behavior but repeats certification against its exact-artifact/deployment profile and no-fallback controls.

## References

- `docs/plan/mvp0-p2p-p5-local-playback.md` — local playback capability and lifecycle.
- `docs/plan/mvp0-p2p-p6-dashboard.md` — owner/viewer product UI and actions.
- `docs/plan/mvp0-p2p-p7-certification.md` — exact-artifact end-to-end certification.
- `docs/tasks/mvp0-p2p-p6-dashboard.md` — P6 phase gates and acceptance.
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md` — runtime ownership/proof isolation.
- `docs/adr/ADR-044-p2p-audience-delivery-boundary.md` — audience delivery and secrets boundary.
