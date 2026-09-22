---
type: Plan
title: "P3: Invitation, audience authorization, and K1 device envelope"
status: in_progress
slice: MVP0-P2P
---

# P3 — Invitation, audience authorization, and K1 device envelope

Task ledger: `docs/tasks/mvp0-p2p-p3-invitation-envelope.md`. P2 is PASS and ADR-044 is accepted. P3.T0 PASS. P3.T1 PASS on 2026-09-22. **P3.T2 PASS on 2026-09-22** after local T2c3 certification, 15/15 CI PASS at `56e9412a`, and explicit owner verification. P3.T3 is unblocked but not activated.

## Objective

Deliver CU-02 and the claim half of CU-04 through backend-owned O3 authorization and the accepted K1 device envelope.

## Scope and accepted decisions

One owner, one invited viewer, one active device. No email/bulk invites, public revoke endpoint, multi-device, trusted-time, or replacement of ADR-032.

No change to accepted O3/K1/O4, ADR-032 review delivery, or C0 contracts.
The parent P3 HP/EC set in `docs/tasks/mvp0-p2p-first.md` remains required;
leaf examples below supplement it rather than narrowing acceptance.

## Affected areas and module dependencies

Existing API authentication/ownership, PostgreSQL, P2 ready descriptor and server-wrapped CK; Android native Keystore adapter. Candidate areas: `apps/api`, `crates/domain`, `crates/db`, `crates/p2p`, `mobile` native adapter and tests. Reserve migration numbers against the current ledger at activation.

These areas were reconciled by P3.T0 on 2026-09-22. Exact writable paths and leaf boundaries are now frozen in `docs/audit/mvp0-p2p-p3-t0-contract-freeze-2026-09-22.md`; executable leaves must still re-fetch current source and rerun RRI before work.

## Phase-specific decisions to freeze

Freeze concrete API/schema names, token-expiry policy, O3 authorization lifecycle, audience/device audit events, envelope binding serialization, and native adapter boundary. Preserve all D2 predicates. The Keystore capability proof must precede integrated envelope closure; inability to use an opaque P-256 private key requires STOP/reopening D2, never a K2 fallback.

## Planned work and verification

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P3.T0 | Contract and executable-path freeze | planning | M | P2 PASS | **PASS 2026-09-22** |
| P3.T1 | Invitation persistence, claim, and inbox | development | decomposed | T0 PASS | **PASS 2026-09-22**; owner-verified; 15/15 CI green |
| P3.T2 | O3 authorization and native K1 envelope delivery | development | decomposed | T1 PASS | **PASS 2026-09-22**; T2c1/T2c2/T2c3 PASS; `56e9412a` 15/15 CI green; owner-verified |
| P3.T3 | P3 integration certification and closure | development/evidence | M | T2 PASS | **Unblocked; not activated** |


The companion ledger defines acceptance, HP/EC, evidence and handoff per task.
Development rows are planning work packages, not pre-scored executable leaves.
At activation, score the coherent parent with `scripts/rri.py`, then perform
honest Low-band maximization using independently verifiable seams; retain the
parent approval/review envelope. Never assume a Low score for crypto, native
integration, authorization or distributed state based on this docs-only edit.

## Closure and downstream gate

All required parent/leaf HP/EC must map to passing executable evidence at the
appropriate layer. Apply the current band-routed review (or recorded applicable
exception), Reflection, behavioral certification and owner-verification gates.
Synchronize this plan, its ledger, the parent plan/ledger and roadmap. No phase
PASS is implied by plan availability or provisional effort.

Current sequencing amendment: P6 activates only after P3 PASS + P4 PASS + P5-DEV. S-230 P2P deployment preparation consumes DEV-HANDOFF = P3 PASS + P4 PASS + P5-DEV + P6 PASS; deferred P5.T3 is not part of that development gate.

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
- `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md` — ready descriptor and sealed-lineage input.
- `docs/audit/mvp0-p2p-p3-t0-contract-freeze-2026-09-22.md` — current P3 source reconciliation, contract freeze, gaps, path ownership and evidence/RRI decomposition.
- `docs/audit/mvp0-p2p-p3-t1-implementation-2026-09-22.md` — T1 claim-race, exact-lineage handoff, durable audience audit and 15/15 CI evidence.
- `docs/audit/mvp0-p2p-p3-t2a-backend-evidence-2026-09-22.md` — T2-A DB release matrix, envelope API/binding, durable envelope audit and 15/15 CI evidence.


### 2026-09-22 local T2c3 certification

The local runner `scripts/p3-t2c3-certify-android.sh` returned `P3_T2C3_RESULT=PASS` against `d14ff8b6646bde4e635d4c2d5bc6e4ff784615e8` on `sdk_gphone64_arm64`, Android API 34, with **2 tests / 0 failures / 0 errors / 0 skipped**. Evidence: [`mvp0-p2p-p3-t2c3-android-certification-2026-09-22.md`](../audit/mvp0-p2p-p3-t2c3-android-certification-2026-09-22.md). The harness matches the existing binding test's full `BeforeUnwrap` name; no crypto implementation or assertion changes were needed.

**T2c1 PASS; T2c2 PASS; T2c3 PASS; P3.T2 PASS.** Final head `56e9412a` completed 15/15 CI checks and the owner explicitly verified the task. P3.T3 is unblocked but not activated. The GitHub-hosted emulator workflow remains hard-disabled and contributed no certification evidence.
