---
type: Plan
title: "P5: Local HLS playback through the existing player"
status: in_progress
slice: MVP0-P2P
---

# P5 — Local HLS playback through the existing player

Task ledger: `docs/tasks/mvp0-p2p-p5-local-playback.md`. P5 remains **in progress** because P5.T3/P5-CERT is still open. **P5-DEV is SATISFIED 2026-09-22:** T0/T1/T2 are formally closed, `e63209f5` completed 15/15 CI, mobile revalidation is 62/62 suites and 446/446 tests, and owner verification is complete. P4 PASS is satisfied.

## Objective

Deliver G5 and the playback half of CU-04 from verified local ciphertext through a loopback HLS gateway and existing VideoPlayer.

## Scope and accepted decisions

Authorized foreground playback, transient session CK and gateway teardown. No ADR-032 review-path replacement, DRM, offline certification, remote key erasure claim or HTTP/S3 audience-media fallback.

No change to accepted O3/K1/O4, ADR-032 review delivery, or C0 contracts.
The parent P5 HP/EC set in `docs/tasks/mvp0-p2p-first.md` remains required;
leaf examples below supplement it rather than narrowing acceptance.

## Affected areas and module dependencies

P4 verified package/state, P3 native Keystore unwrap and current O3 authorization, accepted K1/AAD fixtures, ADR-043 runtime owner, existing VideoPlayer/expo-video. Candidate areas: `mobile/src/p2p`, existing player integration, native adapter seams and tests.

These are candidate areas, not an executable writable-path grant. The phase
activation task must inspect the then-current source, reserve exact paths and
freeze interfaces before any development leaf is presented or delegated.

## Phase-specific decisions to freeze

Freeze loopback-only listener and local session access boundary, HLS path/URI validation, authorization-to-session binding, lifecycle teardown, CK transfer/lifetime, and player ownership. Reject paths outside the verified package. Bind decryption to the accepted manifest/AAD rather than defining another crypto profile.

## Planned work and verification

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P5.T0 | Gateway/session contract freeze | planning | M | P4 PASS | Done 2026-09-18 |
| P5.T1 | Loopback ciphertext decryption gateway | development | L | T0 PASS | **PASS / Done 2026-09-22** |
| P5.T2 | Existing VideoPlayer and deterministic teardown | development | M | T1 PASS | **PASS / Done 2026-09-22** |
| P5.T3 | Playback and secret-boundary certification | release-certification evidence | M | T2 PASS | Deferred release obligation; not a P6 activation gate after P5-DEV closes; may close from compatible T7p or P7.T2 evidence |


The companion ledger defines acceptance, HP/EC, evidence and handoff per task.
Development rows are planning work packages, not pre-scored executable leaves.
At activation, score the coherent parent with `scripts/rri.py`, then perform
honest Low-band maximization using independently verifiable seams; retain the
parent approval/review envelope. Never assume a Low score for crypto, native
integration, authorization or distributed state based on this docs-only edit.

## Closure and downstream gate

**Replan 2026-09-22:** downstream development consumes **P5-DEV = T0-T2
formally closed**. P5.T3 remains mandatory for final invited-playback
certification, but is carried to the release lane instead of blocking P6.

All required parent/leaf HP/EC must map to passing executable evidence at the
appropriate layer. Apply the current band-routed review (or recorded applicable
exception), Reflection, behavioral certification and owner-verification gates.
Synchronize this plan, its ledger, the parent plan/ledger and roadmap. No phase
PASS is implied by automated/component evidence alone. The 2026-09-18 T1/T2
evidence remediation is recorded in
`docs/audit/mvp0-p2p-p5-t1-t2-evidence-remediation-2026-09-18.md`.

Aggregate P5 remains **IN PROGRESS** until T3 Android device evidence. **P5-DEV is satisfied** because T0-T2 are formally closed, so P6 may activate while T3 stays pending. P5.T3 is carried to T7p/P7.T2 and remains
a hard prerequisite for a passing P7 verdict and T9g GO.

## Calendar and estimation limits

Target: development PASS by 2026-10-15.
Effort labels are provisional work-package sizing, not elapsed-day estimates.
Activation must record owner, actual leaf scope, estimates and contingency;
this document does not establish capacity or guarantee the October date.

## References

- `docs/tasks/mvp0-p2p-first.md` — parent acceptance examples and phase dependencies.
- `docs/plan/mvp0-p2p-first.md` — release profile and calendar targets.
- `docs/plan/mvp0-p2p-design-inputs.md` — product inputs; suggested APIs/RPC are non-binding.
- `docs/adr/ADR-044-p2p-audience-delivery-boundary.md` — O3/K1/O4 authority and secrets.
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md` — product runtime ownership.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` and `docs/policies/HITL_AUTONOMY_POLICY.md` — activation and closure.
- `docs/audit/mvp0-p2p-adr044-d2-key-envelope.md` — exact K1 predicates and opaque native-key proof.
- `DESIGN.md` — existing mobile visual intent; shipped theme tokens resolve visual drift.
