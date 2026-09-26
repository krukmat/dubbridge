---
type: TaskList
title: "Tasks: P3 Invitation, audience authorization, and K1 device envelope"
status: pass
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p3-invitation-envelope.md
behavioral_coverage_contract: behavior-v2
---

# P3 — planning task ledger

**Status:** **PASS 2026-09-22.** P3.T0/T1/T2 PASS; P3.T3a/T3b/T3c/T3d PASS. Owner verification for T3d/P3 closure is the explicit 2026-09-22 instruction to close the task. Evidence head `d5644b0b` completed 15/15 CI; implementation head `6c3a565c` completed 15/15 CI with the integrated T3 suite 3/3 PASS in test and coverage. Hosted-emulator CI remains hard-disabled.
**Phase gate:** P2 PASS; Accepted ADR-044.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P3.T0 | Contract and executable-path freeze | planning | M | P2 PASS | **PASS 2026-09-22** — owner-approved; CI 15/15 green |
| P3.T1 | Invitation persistence, claim, and inbox | development | decomposed | T0 PASS | **PASS 2026-09-22** — owner-verified; `4dede25d` 15/15 CI green |
| P3.T2 | O3 authorization and native K1 envelope delivery | development | decomposed | T1 PASS | **PASS 2026-09-22** — T2-A Done; T2c1/T2c2/T2c3 PASS; `56e9412a` 15/15 CI green; owner-verified |
| P3.T3 | P3 integration certification and closure | development/evidence | decomposed | T2 PASS | **PASS 2026-09-22** — T3a/T3b/T3c/T3d PASS; owner-verified |


## Shared activation and closure contract

Resolve exact writable paths, dependencies, parent and leaf RRI before executable
presentation. Source work packages may require further decomposition; do not
execute an L parent as one patch or reuse the documentation-update RRI. Preserve
accepted contracts and the full parent P3 HP/EC set. Future task-analysis and
code-solution review follow the then-current workflow; none is claimed here.

Evidence is stored under `docs/audit/` using the phase/task ID, with redacted
commands, exact artifact identities, actual results and behavioral mappings.
Status artifacts affected by every task: this ledger and `docs/plan/mvp0-p2p-p3-invitation-envelope.md`; phase
closure additionally updates `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-first.md`, and `docs/plan/roadmap.md`.
Release artifact/gate changes also synchronize the S-230 plan and ledger.

## P3.T0 — Contract and executable-path freeze

**Type:** planning

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** P2 PASS

**Status:** **PASS 2026-09-22.** Owner approved the next block, which serves as explicit T0 owner verification. The freeze commit `ffc7ffa` completed 15/15 CI checks successfully. The missing
task-scoped contract now exists at
`docs/audit/mvp0-p2p-p3-t0-contract-freeze-2026-09-22.md`. T0a-T0c reconcile
the current source and freeze invitation/claim/inbox plus O3/device boundaries;
T0d-T0g freeze secrets/audit, exact path ownership, evidence/RRI decomposition
and the handoff to T1/T2/T3. This planning closure changes no runtime behavior.

**Acceptance criteria:** Freeze invitation/claim/inbox and separate O3 authorization contracts, active-device binding, expiry/revocation predicates, audit map and exact path ownership; score the coherent implementation parent and independently meaningful leaves.

- **HP-P3.T0-1:** Ready descriptor and accepted D2 produce a complete API/schema/envelope contract with owned paths.
- **EC-P3.T0-1:** A missing readiness or Keystore boundary remains explicitly blocked; claim alone never grants envelope access.

### Frozen T0 decomposition

| Leaf | Result | Disposition |
|---|---|---|
| T0a | Current source/schema/API/native inventory reconciled | Done |
| T0b | Invitation / claim / inbox contract frozen | Done |
| T0c | O3 authorization + active-device + K1 release contract frozen | Done |
| T0d | Secret-deny and P3 audit-event contract frozen | Done |
| T0e | Exact writable-path ownership for T1/T2/T3 frozen | Done |
| T0f | Test/evidence map and RRI decomposition frozen | Done |
| T0g | Contract artifact + downstream handoff synchronized | **Done / owner-verified** |

The coherent P3 implementation parent is **RRI 100 / Very high** because it spans
authorization, persisted state, audit and native cryptographic custody. It must not
execute as one patch. The contract artifact therefore freezes independently
verifiable leaves and their planned RRI envelopes; every executable leaf must
rerun `scripts/rri.py` against its exact current paths immediately before work.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify P2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T0's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P3.T1 — Invitation persistence, claim, and inbox

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T0 PASS

**Status:** **PASS 2026-09-22.** The owner verified the implemented T1a/T1b/T1c
block after review. Implementation head `4dede25d` completed 15/15 CI checks
successfully. Evidence:
`docs/audit/mvp0-p2p-p3-t1-implementation-2026-09-22.md`.

Delivered:
- **T1a:** real concurrent different-viewer claim race proves exactly one winner,
  exactly one authorization row and idempotent repeat by the winner.
- **T1b:** claim response now fails closed unless descriptor, invitation and
  authorization share the exact asset + publication + lineage identity.
- **T1c:** durable P3 device/invitation/claim/authorization/denial audit contract
  is wired through the governance audit boundary with bounded non-secret details.

The earlier "0 repo-layer tests" note is superseded. T1 is now owner-verified;
P3.T2 may activate under its own current workflow.

**Acceptance criteria:** Implement hash-only invitation storage, owner-only creation on P2P_READY content, atomic single-viewer claim and scoped inbox; preserve same-viewer idempotency and durable audit.

- **HP-P3.T1-1:** Owner creates an invite; token is returned once; eligible viewer claims and later uses their inbox without the raw token.
- **EC-P3.T1-1:** Concurrent different-viewer claims have one winner; expired/unknown/non-owner/non-ready requests fail closed without logging tokens.

### Implementation evidence

Implementation lineage:
`cbb00567` → `ec1ce7ea` → `e7e851a3` → `ea8b7d8c` →
`e8df1ae9` → `d7eb83fe` → `94ba73fc` → `4dede25d`.

Final implementation head `4dede25d`: **15/15 CI PASS**, including
`test`, `coverage`, `cargo-check`, `clippy`, `fmt`, `release-build`,
`mobile`, `s3-integration`, `deny`, `config-secrets`,
`peer-workflow-review`, `maintainability`, `python-complexity`,
`roadmap-drift` and `qa-docs`.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T0 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T1's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P3.T2 — O3 authorization and native K1 envelope delivery

**Type:** development

**Effort:** L (provisional; re-score/decompose at activation)

**Depends on:** T1 PASS

**Status:** **PASS 2026-09-22.** T2-A and T2-B are closed. Final certification head `56e9412a` completed **15/15 CI PASS** and the repository owner explicitly verified P3.T2. Backend block **T2-A** is complete at implementation
head `a228ddad` with **15/15 CI PASS**. Evidence:
`docs/audit/mvp0-p2p-p3-t2a-backend-evidence-2026-09-22.md`.

Delivered in T2-A:
- **T2a:** release predicate matrix now proves wrong viewer, dead authorization,
  dead invitation, device revocation/device drift, non-ready/reconciling
  publication, missing sealed-K1 evidence and undelivered outbox all fail closed.
  Impossible publication/lineage drift is additionally rejected by schema
  constraint/FK before release evaluation.
- **T2b:** envelope construction is isolated/testable, binds exact
  invitation/viewer/device/asset/publication/lineage/authorization/expiry, and
  maps KEK/nonce/wrapped-CK/device-key failures to stable fail-closed reasons.
- **T2d:** envelope success and denial now emit durable ADR-018 P3 audit before
  the response is released; audit failure returns 500. Audit detail contains
  bounded identifiers/reason codes only.

T2-B / T2c disposition:
- **T2c1 PASS:** existing JS native-only/no-software-fallback evidence retained; no JS source changes in this certification.
- **T2c2 PASS:** native binding JSON parsing + expiry enforcement before HPKE unwrap verified by the existing instrumentation.
- **T2c3 PASS:** local runner returned `P3_T2C3_RESULT=PASS` against `d14ff8b6646bde4e635d4c2d5bc6e4ff784615e8` on `sdk_gphone64_arm64`, API 34: **2 tests / 0 failures / 0 errors / 0 skipped**.

Evidence: [`docs/audit/mvp0-p2p-p3-t2c3-android-certification-2026-09-22.md`](../audit/mvp0-p2p-p3-t2c3-android-certification-2026-09-22.md). The binding case's exact existing name is `bindingValidationRejectsExpiryAndIdentityDriftBeforeUnwrap`; the harness now matches it exactly. Product crypto, test bodies and assertions were unchanged.

**P3.T2 PASS.** Owner verification is complete and the final documentation/evidence head `56e9412a` completed 15/15 CI checks successfully. P3.T3 is unblocked but remains unactivated; the hosted-emulator workflow remains hard-disabled.

Certification checklist: harness self-check completed; local API 34 instrumentation completed; redacted evidence completed; final CI 15/15 PASS; owner verification complete.
Resource plan: existing deterministic Android runner; bounded harness correction and local independent review only; no new Android infrastructure.

**Acceptance criteria:** Implement distinct backend audience authorization and all accepted D2 release predicates; prove HPKE Base P-256/HKDF-SHA256/AES-256-GCM with non-exportable Android Keystore private key, native unwrap, binding and expiry checks.

- **HP-P3.T2-1:** Eligible claimed viewer and active device receive a package-bound envelope and unwrap through the opaque native key.
- **EC-P3.T2-1:** Wrong device/package/viewer, expired or revoked authorization, non-ready publication, or missing Keystore capability produces no CK release; no software private-key fallback.

### T2-A implementation evidence

Implementation lineage:
`35f04a2e` → `44976e05` → `22c18fea` → `a0ddc408` →
`767c2393` → `a228ddad`.

Final backend implementation head `a228ddad`: **15/15 CI PASS**, including
`test`, `coverage`, `cargo-check`, `clippy`, `fmt`, `release-build`,
`mobile`, `s3-integration`, `deny`, `config-secrets`,
`peer-workflow-review`, `maintainability`, `python-complexity`,
`roadmap-drift` and `qa-docs`.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T1 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T2's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.

## P3.T3 — P3 integration certification and closure

**Type:** development/evidence

**Effort:** M (provisional; re-score/decompose at activation)

**Depends on:** T2 PASS

**Status:** **PASS 2026-09-22.** T3a/T3b integrated certification PASS at `eb1e8abe`; T3c secret-boundary PASS at `6c3a565c`; **T3d closure/handoff PASS and owner-verified. Aggregate P3 = PASS.** **Historical verification note (2026-09-18):** no work product
exists for this leaf. Full evidence:
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`.

**Acceptance criteria:** Map every parent P3 HP/EC to executable evidence, including claim races, O3 denial, native Keystore interop and log/storage secret inspection; publish descriptor/native-adapter handoff to P4/P5.

- **HP-P3.T3-1:** Owner create → viewer claim → authorized envelope → native unwrap passes with exact package binding.
- **EC-P3.T3-1:** Possession of ciphertext or a valid claim without current O3 authorization never permits envelope release.


### T3 activation breakdown

| Leaf | Outcome | Status |
|---|---|---|
| T3a | Integrated owner → invite → claim → O3 → envelope happy path | **PASS 2026-09-22** |
| T3b | Integrated fail-closed viewer/O3/device/package matrix | **PASS 2026-09-22** |
| T3c | API/audit/DB/mobile/native secret-boundary inspection | **PASS 2026-09-22** |
| T3d | Evidence map, downstream handoff, final closure | **PASS 2026-09-22** |

T3a/T3b implementation certification head `eb1e8abe` completed 15/15 CI.
During that certification, the DB audit CHECK was found to lag the already
frozen P3 correlation contract; migration 0039 repaired the durable constraint
without relaxing P2 invariants.

T3c evidence:
`docs/audit/mvp0-p2p-p3-t3c-secret-boundary-evidence-2026-09-22.md`.
Exact T3c head `6c3a565c` completed 15/15 CI; the integrated suite executed
T3a/T3b/T3c 3/3 PASS under both normal tests and coverage, workspace line
coverage remained 90.43%, and the static guard returned
`P3_T3C_SECRET_BOUNDARY=PASS`.

T3c closes the frozen secret-deny boundary without re-enabling the hosted
Android HPKE emulator. **T3d is now closed; P3.T3 PASS and aggregate P3 PASS.**

T3d evidence: `docs/audit/mvp0-p2p-p3-t3d-closure-2026-09-22.md`.

**Evidence to emit:** task-scoped contract/decision record for planning; actual
command/test/device/network results as relevant to the acceptance criteria for
implementation or operational work. Map the examples above and inherited parent
examples to appropriate `unit`, `component`, `integration`, `contract`, or `e2e`
evidence when behavior is delivered. Record failures rather than inferring PASS.

**Status artifacts affected:** shared status set above; propagate any changed
downstream input to its consuming phase before claiming closure.

**Agent handoff:** Read this phase plan and governing references. Verify T2 PASS;
freeze and score exact paths, preserve the accepted boundary, and deliver only
P3.T3's acceptance criteria through the current workflow. Stop on a
contract conflict or unmet dependency; do not silently advance the next phase.
