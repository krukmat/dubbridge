---
type: Audit
title: "MVP0-P2P P5.T3 sequencing replan"
status: accepted
date: 2026-09-22
---

# P5.T3 sequencing replan

## Decision

P5.T3 is not waived and is not marked PASS. Its weight changes: it moves from
the development critical path to the release-certification lane.

P5 now exposes two milestones:

- **P5-DEV:** P5.T0-T2 formally closed. This is sufficient for P6 activation.
- **P5-CERT:** P5.T3 physical playback/secret-boundary evidence PASS. This is
  required before a passing P7 verdict and T9g GO.

P3 and P4 are not relaxed. P6 still requires **P3 PASS + P4 PASS + P5-DEV**.

## Release consolidation

The P5.T3 physical run should not be duplicated purely for sequencing. A T7p
run may close P5.T3 if it is against the exact RC and captures all P5.T3
criteria. Otherwise P7.T2 is the mandatory consolidation point.

```text
P3 PASS ─┐
P4 PASS ─┼─> P5-DEV ─> P6 PASS ─> DEV-HANDOFF ─> T6p-a..d ─> T7p
         │                                                   │
         └───────────────────────────────────────────────────┤
                                                             v
                                                   P7.T0 -> P7.T1 -> P7.T2
                                                                      │
                                                                      └─ resolves P5.T3
                                                                            │
                                                                            v
                                                                       P7.T3 -> T9g
```

## Invariants retained

- no P5.T3 PASS is inferred from CI;
- aggregate P5 may remain IN PROGRESS while development continues;
- no transient-drive residual is changed;
- no HTTP/S3 audience-media fallback is allowed;
- a failed or unresolved P5.T3 prevents a passing P7 verdict and T9g GO;
- JWT, CK, envelopes and invitation tokens remain redacted from evidence.

## Rationale

The previous sequence made one environment-sensitive Android run block dashboard
and deployment work even though the release lane already contains an
exact-artifact physical owner-to-viewer proof. Consolidating those proofs removes
duplicate device pressure without weakening the final go-live claim.

## Amendment 2026-09-26 — P5.T3 becomes a general test inside the physical tests

**Decision (owner, 2026-09-26):** P5.T3 is relativized. It is no longer a task
with its own Android run, harness loop or environment work. It remains as a
**general test**: the control checklist in
`docs/playbooks/P5_T3_ANDROID_CERTIFICATION.md` § Required evidence, executed
and recorded as part of the physical tests — S-230-T7p on the exact RC, or
P7.T2 when T7p does not cover it.

- **Not scheduled:** a dedicated P5.T3 device session, emulator diagnostic
  loops, or local-topology work (routable Colima VM, host-side seeder). Emulator
  runs are de-risking only and never count as P5-CERT evidence, which requires
  physical Android.
- **P5-CERT** is satisfied when the physical-test evidence covers every
  checklist control. It is no longer a separate milestone with its own owner
  action.
- **Invariants unchanged:** no PASS inferred from CI; no HTTP/S3 audience-media
  fallback; a failed checklist control in the physical tests forces P7
  NOT_CERTIFIED and blocks T9g GO; JWT, CK, envelopes and invitation tokens stay
  redacted.
- **Inputs for the physical run**, from the 2026-09-26 emulator attempts
  (`docs/audit/mvp0-p2p-p5-t3-postfix-diagnostic-2026-09-26.md`): provision a
  fresh viewer and do not clear app state afterwards (the Keystore key under
  alias `dubbridge-p2p-k1-v1` regenerates and conflicts with the active server
  device); the phone must reach the Availability Node over Hyperswarm, which the
  local Colima topology does not allow from the host/emulator; the client
  cold-drive discovery repair `P5.T3-r1` (commit `3f3ffbd`) is not
  device-validated, and the physical run is its first Android validation.
