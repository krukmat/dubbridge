---
type: TaskList
title: "Tasks: P3 Invitation, audience authorization, and K1 device envelope"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-p3-invitation-envelope.md
behavioral_coverage_contract: behavior-v2
---

# P3 — planning task ledger

**Status:** In progress. P3.T0 PASS. P3.T1 implementation/evidence is closure-ready on 2026-09-22 at `4dede25d` with 15/15 CI checks green; final T1 PASS awaits owner verification of the implemented block.
**Phase gate:** P2 PASS; Accepted ADR-044.
**Effort:** provisional per work package below; executable RRI/effort pending activation.

## Task map

| Task | Outcome | Type | Provisional effort | Depends on | Status |
|---|---|---|---|---|---|
| P3.T0 | Contract and executable-path freeze | planning | M | P2 PASS | **PASS 2026-09-22** — owner-approved; CI 15/15 green |
| P3.T1 | Invitation persistence, claim, and inbox | development | decomposed | T0 PASS | **Closure-ready 2026-09-22** — T1a/T1b/T1c implemented; `4dede25d` 15/15 CI green; owner verification pending |
| P3.T2 | O3 authorization and native K1 envelope delivery | development | L | T1 PASS | Blocked on T1 PASS — release-context test exists; remaining predicate/API/native/audit certification frozen in T0 artifact |
| P3.T3 | P3 integration certification and closure | development/evidence | M | T2 PASS | Blocked — no work product exists, see verification note |


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

**Status:** **Closure-ready 2026-09-22; owner verification pending.** The approved
T1a/T1b/T1c block is implemented and certified at implementation head
`4dede25d`, which completed 15/15 CI checks successfully. Evidence:
`docs/audit/mvp0-p2p-p3-t1-implementation-2026-09-22.md`.

Delivered:
- **T1a:** real concurrent different-viewer claim race proves exactly one winner,
  exactly one authorization row and idempotent repeat by the winner.
- **T1b:** claim response now fails closed unless descriptor, invitation and
  authorization share the exact asset + publication + lineage identity.
- **T1c:** durable P3 device/invitation/claim/authorization/denial audit contract
  is wired through the governance audit boundary with bounded non-secret details.

The earlier "0 repo-layer tests" note is superseded. T2 remains blocked until
this T1 closure receives owner verification and the ledger records T1 PASS.

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

**Status:** Blocked on T1 PASS. **Re-verification 2026-09-22:** source is
present in `crates/p2p/src/device_envelope.rs`,
`crates/db/src/p2p_envelope_repo.rs`, `apps/api/src/routes/p2p_envelope.rs`,
`mobile/src/p2p/device/DeviceIdentity.ts` and the Android
`DubBridgeP2PKeyStoreModule.kt`. The DB integration suite already proves a
valid release context and denial after device revocation, so the earlier "all
0 tests" statement is stale. Remaining T2 work is the full fail-closed predicate
matrix, handler/binding behavior, explicit JS no-software-fallback evidence,
native opaque-key interop certification and P3 envelope audit coverage.

**Acceptance criteria:** Implement distinct backend audience authorization and all accepted D2 release predicates; prove HPKE Base P-256/HKDF-SHA256/AES-256-GCM with non-exportable Android Keystore private key, native unwrap, binding and expiry checks.

- **HP-P3.T2-1:** Eligible claimed viewer and active device receive a package-bound envelope and unwrap through the opaque native key.
- **EC-P3.T2-1:** Wrong device/package/viewer, expired or revoked authorization, non-ready publication, or missing Keystore capability produces no CK release; no software private-key fallback.

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

**Status:** Blocked. **Verification note (2026-09-18):** no work product
exists for this leaf. Full evidence:
`docs/audit/mvp0-p2p-p3-p4-p5-retrospective-closure-evidence-2026-09-18.md`.

**Acceptance criteria:** Map every parent P3 HP/EC to executable evidence, including claim races, O3 denial, native Keystore interop and log/storage secret inspection; publish descriptor/native-adapter handoff to P4/P5.

- **HP-P3.T3-1:** Owner create → viewer claim → authorized envelope → native unwrap passes with exact package binding.
- **EC-P3.T3-1:** Possession of ciphertext or a valid claim without current O3 authorization never permits envelope release.

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
