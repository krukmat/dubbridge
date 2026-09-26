---
type: Plan
title: "P5: Local HLS playback through the existing player"
status: complete
slice: MVP0-P2P
---

# P5 — Local HLS playback through the existing player

Task ledger: `docs/tasks/mvp0-p2p-p5-local-playback.md`. **P5 is closed as
a feature-delivery phase on 2026-09-26:** T0-T2 are formally closed and the
former standalone P5.T3 certification record is transferred to P7.T2. This is
not a P5-CERT/P7 certification PASS. The exact-artifact physical gate remains
open under P7 and T9g. `e63209f5` completed 15/15 CI, mobile revalidation is
62/62 suites and 446/446 tests, and owner verification is complete. P4 PASS is
satisfied.

**P5.T3 closure and transfer 2026-09-26:** the standalone P5.T3 record is
closed. Its checklist is owned exclusively by the physical release tests:
S-230-T7p when it covers the exact RC, otherwise P7.T2. The 2026-09-26
emulator attempts gave no playback evidence and schedule no follow-up work.
They remain historical evidence in
`docs/audit/mvp0-p2p-p5-t3-postfix-diagnostic-2026-09-26.md`; the transfer
decision is recorded in
`docs/audit/mvp0-p2p-p5-t3-sequencing-replan-2026-09-22.md` § Amendment 2026-09-26.

The committed cold-drive repair's RRI and reproduced regression evidence are
recorded in `docs/audit/p5-t3-r1-discovery-repair-evidence-2026-09-26.md`;
independent review and physical validation remain pending. This does not
reactivate local-topology work or change the release evidence gate.

**Owner confirmation 2026-09-26:** discard the local-development diagnostic
lane for P5.T3. Local emulator/Colima connectivity is historical evidence,
not an active blocker or prerequisite for the physical release tests.

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
| P5.T3 | Playback and secret-boundary certification | transferred release checklist | M | T2 PASS | **Closed 2026-09-26 — transferred to T7p/P7.T2; no P5-CERT PASS claimed** |


The companion ledger defines acceptance, HP/EC, evidence and handoff per task.
Development rows are planning work packages, not pre-scored executable leaves.
At activation, score the coherent parent with `scripts/rri.py`, then perform
honest Low-band maximization using independently verifiable seams; retain the
parent approval/review envelope. Never assume a Low score for crypto, native
integration, authorization or distributed state based on this docs-only edit.

## Closure and downstream gate

**Closure 2026-09-26:** P5 delivery is complete. Final invited-playback
certification remains mandatory, but its sole owner is now T7p/P7.T2 rather
than an open P5 task.

All required parent/leaf HP/EC must map to passing executable evidence at the
appropriate layer. Apply the current band-routed review (or recorded applicable
exception), Reflection, behavioral certification and owner-verification gates.
Synchronize this plan, its ledger, the parent plan/ledger and roadmap. No phase
PASS is implied by automated/component evidence alone. The 2026-09-18 T1/T2
evidence remediation is recorded in
`docs/audit/mvp0-p2p-p5-t1-t2-evidence-remediation-2026-09-18.md`.

P5 is **CLOSED — release evidence transferred**. A failed or missing control in
T7p/P7.T2 forces a P7 `NOT_CERTIFIED` verdict and blocks T9g GO.

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
