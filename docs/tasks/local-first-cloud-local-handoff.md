---
type: TaskList
title: "Tasks: Local-first / cloud-local handoff contract"
plan: docs/plan/local-first-cloud-local-handoff.md
status: active
slice: local-first-cloud-local-handoff
governed_by: [ADR-036, ADR-037]
---

# Tasks: Local-first / cloud-local handoff contract

## Objective

Turn the handoff contract described in
`docs/plan/local-first-cloud-local-handoff.md` into concrete, checkable
artifacts: a shared context-capsule/attempt-bundle schema, per-lane bundle
emission, a cloud conciliator gate, and a pilot that produces go/no-go
evidence — without introducing any new role, band, or authority.

## Slice RRI (this document contract, T0)

```
python3 scripts/rri.py --touches docs/plan/local-first-cloud-local-handoff.md \
  --touches docs/tasks/local-first-cloud-local-handoff.md \
  --C 0 --D 0 --K 0 --P 0 --T 0 --A 1 --X 3 --penalty arch_decision
```

**RRI 20 → Low (0–25) → Effort S.** Docs-only, so the full human-approval
presentation is not required; this ledger is posted for visibility, not
blocking approval.

- `Task-analysis review: n/a` — plan/task-ledger-only work.
- `Code-solution review: n/a` — no code changed by T0.

Per-task RRI values below (T1–T7) are **preliminary estimates**. Each must be
recomputed with `scripts/rri.py` at its own presentation time before
implementation starts, per `AGENT_WORKFLOW_GUIDE.md` § Mandatory workflow.
Development tasks (T1–T5) are expected to land in the 26–55 local-agent lane
themselves once implemented — this ledger's subject and its own execution
mechanism are the same lane, which is expected and not a conflict.

## Task order and dependencies

```text
T0 ──► T1 ──┬──► T2 ──┐
            ├──► T3 ──┼──► T4 ──► T6 ──► T7
            └──► T5 ──┘
```

T0 is this document pair. T1 must land before T2/T3/T5 (they all implement
against its schema). T2/T3/T5 can proceed in parallel. T4 needs all three.
T6 (pilot) needs T4. T7 (policy sync / go-no-go) needs T6.

---

## T0 — Document the handoff contract

- **Status:** `[x] Done`
- **Effort:** S
- **RRI:** 20 Low (see Slice RRI above)
- **Scope:** `docs/plan/local-first-cloud-local-handoff.md`, this file
- **Depends on:** none

### Objective

Record the existing local-first / cloud-local handoff lanes, role inventory,
and the context-capsule/attempt-bundle vocabulary in canonical plan/task
form, consolidating what ADR-036, ADR-037, `AGENT_WORKFLOW_GUIDE.md`, and
`RRI_POLICY.md` already define without changing any of it.

### Acceptance criteria

- Plan states objective, role inventory (no new actors), handoff-lane table,
  capsule/bundle definitions, a conciliator checklist summary, a Mermaid
  diagram, and a task dependency analysis.
- No band boundary, reviewer route, or repair budget is altered from what
  `RRI_POLICY.md` / ADR-036 / ADR-037 already state.
- `bash scripts/check-doc-consistency.sh`, `python3
  scripts/check_okf_frontmatter.py` pass.

### Status artifacts affected

- None outside this plan/task pair — T0 introduces no cross-references from
  other canonical docs; `docs/plan/roadmap.md` is not touched because this
  slice is process documentation, not a roadmap-tracked product slice.

### Completion evidence

- `docs/plan/local-first-cloud-local-handoff.md` created.
- This ledger created.
- Doc-consistency and OKF frontmatter checks run (see closure section below).

---

## T1 — Define the context-capsule / attempt-bundle schema

- **Status:** `[x] Done`
- **Effort:** M
- **RRI:** 27 Moderate (confirmed at presentation time via `scripts/rri.py
  --touches scripts/local-agent/handoff_schema.py --touches
  scripts/local-agent/handoff_schema_test.py --C 1 --D 2 --K 2 --P 1 --T 1
  --A 1 --X 2`; matches the preliminary estimate)
- **Scope:** `scripts/local-agent/handoff_schema.py`,
  `scripts/local-agent/handoff_schema_test.py`
- **Depends on:** T0

### Objective

Formalize the context capsule and attempt bundle described in the plan as a
validated schema (dataclass/JSON-schema, matching the existing
`scripts/local-agent/` module style) that `run_local_task.py`,
`delegate-low-rri.py`, and the ADR-037 wrapper (`run_analysis.py`) can each
construct or validate against, without forcing any of the three to change
their existing wire format immediately (T2/T3/T5 handle each adapter).

### Happy paths considered

- `HP-1`: a well-formed capsule dict (work item id, objective, allowed
  paths, acceptance criteria, revision, manifest hash) validates and its
  SHA-256 is computed deterministically.
- `HP-2`: an attempt bundle referencing a valid capsule hash, with
  implementer identity, timestamps, and an outcome enum value, validates.

### Edge cases considered

- `EC-1`: a capsule missing a required field fails validation with a
  specific field-name error, not a generic exception.
- `EC-2`: an attempt bundle whose `capsule_hash` does not match any known
  capsule is rejected, not silently accepted.

### Acceptance criteria

- Schema covers every field enumerated in the plan's "Context capsule" and
  "Attempt bundle" sections.
- Validation is pure/offline (no network, no filesystem beyond the passed
  object) so it can be unit-tested without Ollama.
- Unit tests cover HP-1, HP-2, EC-1, EC-2.

### Evidence to emit

- Unit-test output; a short usage example in the module docstring/README
  showing one capsule + one bundle constructed and validated.

### Completion evidence

- `scripts/local-agent/handoff_schema.py`: `Capsule`/`AttemptBundle` +
  `validate_capsule`/`validate_attempt_bundle`, `ValidationError(field_name,
  reason)`, SHA-256 manifest hash over canonical (sorted-key) JSON.
- `scripts/local-agent/handoff_schema_test.py`: 10 tests, all passing —
  `python3 scripts/local-agent/handoff_schema_test.py -v` → `Ran 10 tests ...
  OK`. Covers HP-1 (validation + deterministic, order-independent hash),
  HP-2 (bundle validates against known capsule hash), EC-1 (missing capsule
  field raises `ValidationError` with the specific field name; non-dict
  input rejected), EC-2 (unknown `capsule_hash` rejected; missing bundle
  field raises with field name; invalid `outcome` enum value rejected), and
  a combined usage-example test (one capsule + one bundle constructed and
  validated).
- Field coverage verified 1:1 against the plan's "Context capsule" (10
  fields) and "Attempt bundle" (9 fields, incl. `capsule_hash`) sections —
  no field added or omitted.
- Validation confirmed pure/offline: no imports beyond `hashlib`/`json`, no
  filesystem or network access inside either validator.

### Status artifacts affected

- This ledger (T1 marked done; T2/T3/T5 unblocked).

### Reflection strategy

RRI 26+ requires Reflection passes. At Moderate (26–40), 2 passes.

**Pass 1 (schema completeness + hash correctness) — result: clean.**
Verified `CAPSULE_REQUIRED_FIELDS`/`BUNDLE_REQUIRED_FIELDS` match the plan's
field lists exactly (set equality check), and that `compute_manifest_hash`
is deterministic and order-independent (canonical JSON via `sort_keys=True`
makes field insertion order irrelevant to the hash).

**Pass 2 (fail-closed validation) — result: clean.** Verified every single
required field in both `CapsuleSchema` and `AttemptBundle`, when omitted
individually, raises `ValidationError` naming exactly that field (looped
over all 10 capsule fields and all 9 bundle fields) — no silent coercion.
Also verified non-dict input, an unknown `capsule_hash`, and an invalid
`outcome` enum value are all rejected rather than accepted.

No findings from either pass required a code change.

### Review gate (Moderate band: Gemma phase-1 / Qwen27 phase-2)

- **Phase 1 — Gemma Reviewer (`gemma4:26b-a4b-it-qat`), 3 passes:** status
  `findings`, 3 minor location-inconsistent (non-consensus) findings, all
  the same underlying observation — `validate_capsule`/
  `validate_attempt_bundle` silently ignore unrecognized extra keys in the
  input dict instead of rejecting them (the schema only enforces *required*
  fields, per its acceptance criteria; it does not reject *unexpected*
  ones). No blocking or major findings.
- **Phase 2 — Qwen27 reviewer override (`qwen3.6:27b-q4_K_M`), 3 passes:**
  status `pass`, 0 findings across all passes. Summary: implementation
  correctly enforces required fields, outcome enum, capsule-hash
  cross-check, and deterministic hashing; unit tests cover HP-1/HP-2/EC-1/
  EC-2.
- **Disposition:** closed as-is, no code change. Gemma's finding describes
  accepted, in-scope behavior — the ledger's EC-1/EC-2 acceptance criteria
  require rejecting *missing* required fields and unknown `capsule_hash`
  values, not extra/unrecognized keys, and strict unknown-field rejection
  risks blocking legitimate metadata T2/T3/T5 may need to pass through
  their own adapters. Revisit only if a concrete need for strict-schema
  enforcement emerges in T2/T3/T5.

---

## T2 — Local-agent lane: attempt-bundle emission in `run_local_task.py`

- **Status:** `[x] Done`
- **Effort:** M
- **RRI:** 41 Med-high (confirmed at presentation time via `scripts/rri.py
  --touches scripts/local-agent/run_local_task.py --touches
  scripts/local-agent/run_local_task_test.py --C 2 --D 3 --K 3 --P 2 --T 2
  --A 1 --X 2 --F 1`; up from the preliminary ~30 Moderate estimate — D/K
  raised on agent judgment given `run_local_task.py`'s size and its coupling
  to the existing ADR-034 audit path)
- **Scope:** `scripts/local-agent/run_local_task.py`,
  `scripts/local-agent/run_local_task_test.py`
- **Depends on:** T1

### Objective

Emit one attempt-bundle record (per T1's schema) per Qwen35 execution/repair
attempt, referencing the task card's capsule hash, in addition to the
existing ADR-034 audit record — this is an additive, structurally-named view
of data the runner already captures (attempts, diffs, test results), not a
new data source.

### Happy paths considered

- `HP-1`: a successful single-attempt run emits one attempt bundle with
  `outcome: success` and a diff reference.
- `HP-2`: a two-attempt repair sequence emits two attempt bundles, both
  referencing the same capsule hash.

### Edge cases considered

- `EC-1`: escalation (repair budget exhausted) emits a bundle with
  `outcome: escalated` and no partial/malformed bundle for the aborted
  attempt.

### Acceptance criteria

- Bundle emission does not change existing ADR-034 audit-record behavior or
  schema — additive only.
- Bundles validate against T1's schema.
- Existing `run_local_task_test.py` suite remains green; new tests cover
  HP-1/HP-2/EC-1.

### Evidence to emit

- Unit-test output; one example bundle from a mocked run attached to the
  task closure record.

### Completion evidence

- `scripts/local-agent/run_local_task.py`: new `build_attempt_bundles(card,
  result, model, session_start, session_end)`, segmenting `run_loop`'s flat
  transcript on `test_result` events into one bundle per attempt; `TaskCard`
  gained a `capsule_hash` field (default `None`, so pre-T1 cards are
  unaffected); `main()` computes `session_end` once after `run_loop` returns
  and appends each bundle via the existing generic `append_audit_log`, after
  the pre-existing ADR-034 record append. Net diff: `run_local_task.py`
  +107/-3, `run_local_task_test.py` +190 (294 lines total, confirmed under
  the user's 500-line direct-implementation threshold — implemented
  directly rather than delegated to Qwen35).
- `scripts/local-agent/run_local_task_test.py`: new `AttemptBundleEmission`
  test class, 6 tests, all passing —
  `python3 -m unittest scripts.local-agent.run_local_task_test.AttemptBundleEmission -v`
  → `Ran 6 tests ... OK`. Covers HP-1 (`test_hp1_single_attempt_success_emits_one_bundle`,
  and `test_hp1_end_to_end_main_emits_bundle_via_append_audit_log` through
  the real `main()` path with a mocked `append_audit_log`), HP-2
  (`test_hp2_two_attempt_repair_emits_two_bundles_same_capsule_hash`), EC-1
  (`test_ec1_escalation_emits_escalated_bundle_no_partial_for_aborted_attempt`),
  plus two guard tests added during review (`test_no_capsule_hash_on_card_emits_no_bundles`,
  `test_boundary_violation_tail_with_no_test_result_emits_no_bundle_for_it`).
  Full module (`run_local_task_test.py`): `Ran 63 tests ... OK`, no
  regressions in the pre-existing suite.
- Example bundle from a mocked run (single successful attempt):
  ```json
  {
    "capsule_hash": "aaaa...aaaa",
    "implementer_id": "qwen35",
    "model_tag": "qwen35-coder",
    "start_ts": "2026-07-23T00:00:00Z",
    "end_ts": "2026-07-23T00:05:00Z",
    "diff_ref": [{"tool": "write_file", "path": "src/foo.rs"}],
    "test_results": {"passed": true, "status": "ok"},
    "review_verdict": "pending",
    "outcome": "success"
  }
  ```
  Validated against T1's schema via
  `handoff_schema.validate_attempt_bundle(...)` inside
  `test_hp1_end_to_end_main_emits_bundle_via_append_audit_log`.
- Non-interference with ADR-034 confirmed: `build_audit_record` and
  `run_loop`'s control flow/return shape received zero changes; the
  end-to-end test asserts the first `append_audit_log` call is still the
  standard audit record (`role`, `outcome`, `signature` present) and the
  bundle is strictly the second, additional call.

### Status artifacts affected

- This ledger (T2 marked done; T3/T5 remain independently unblocked by T1,
  T4 now has one of its three T2/T3/T5 dependencies satisfied).

### Reflection strategy

Med-high band (RRI 41, not the preliminary Moderate estimate) routes both
review phases to Qwen27 rather than Moderate's Gemma/Qwen27 split; 2
Reflection passes still apply.

**Pass 1 (emission correctness across success/repair/escalation paths) —
result: findings, fixed.** Verified segmentation against the three outcome
paths (HP-1 success, HP-2 repair, EC-1 escalation) and found two real
defects, both surfaced independently by the Qwen27 phase-1 review: (a)
`end_ts` was computed via `datetime.now()` inside the post-hoc
segmentation loop, so every bundle in a session got a near-identical
bundle-*generation* timestamp rather than a real per-attempt one — fixed by
bounding all bundles in a session to the caller-supplied
`session_start`/`session_end` window (computed once in `main()`
immediately after `run_loop` returns) and documenting the resulting
granularity limitation in the function's docstring rather than fabricating
false precision; (b) the escalation branch only fired for
`status == "budget_exhausted"`, so a failing final attempt under any other
terminal status (`boundary_violation`, `transport_error`, `aborted`) was
mislabeled `repair-needed`, incorrectly implying a further repair turn
would follow — fixed by escalating on any failing final attempt regardless
of terminal status.

**Pass 2 (non-interference with the existing ADR-034 audit record) —
result: clean.** Verified `build_audit_record` and `run_loop`'s control
flow/return shape are byte-for-byte unchanged (no diff to either); verified
bundle emission is strictly additive — it runs after, and consumes only the
already-returned `result`/`card`, the pre-existing `append_audit_log` call;
strengthened the end-to-end test to assert the first `append_audit_log`
call still carries the original audit record's `role`/`outcome`/`signature`
shape, not just that a second record appeared.

### Review gate (Med-high band: Qwen27 both phases)

- **Phase 1 — Qwen27 (`qwen3.6:27b-q4_K_M`):** status `findings` — 1 HIGH
  (fabricated `end_ts`, see Reflection pass 1), 1 MEDIUM (escalation
  mislabeling, see Reflection pass 1), 1 LOW (weak end-to-end assertion on
  `captured[0]`). All three fixed.
- **Phase 2 — Qwen27, re-review of the corrected diff:** status `findings`
  — 1 HIGH + 1 MEDIUM (defensive `.get()` guards proposed for
  `e["result"]`/`e["path"]` in the `tool_result`-derived `diff_ref`
  construction), 1 MEDIUM (missing test for a transcript ending on a
  non-`test_result` terminal event), 1 LOW (speculative concern about
  malformed/duplicate `test_result` events skewing segmentation). The
  HIGH and first MEDIUM were evaluated and **not applied**: `grep` confirms
  `tool_result` events are constructed in exactly one place in the file
  (`run_loop`, always setting both `result` and, for `write_file`/
  `apply_patch`, `path`) — internal, single-producer, trusted data, not
  external input — and the pre-existing sibling function
  `build_audit_record` already accesses the same fields unguarded; adding
  guards here would validate an unreachable scenario and diverge from
  established code style. The second MEDIUM was legitimate and cheap: added
  `test_boundary_violation_tail_with_no_test_result_emits_no_bundle_for_it`.
  The LOW was noted but not actionable (no concrete fix proposed, scenario
  outside T2's stated scope) — set aside.
- **Disposition:** closed after applying 3 of 4 phase-1 findings (weak
  assertion, timestamp bug, escalation-label bug) and 1 of 4 phase-2
  findings (missing boundary-tail test); 3 phase-2 findings rejected/set
  aside with evidence above. No escalation to D14 was needed in either
  phase.

---

## T3 — Low-band packet: metadata parity with the capsule schema

- **Status:** `[x] Done`
- **Effort:** S
- **RRI:** 23 Low (confirmed at presentation time via `scripts/rri.py
  --touches scripts/delegate-low-rri.py --C 1 --D 2 --K 2 --P 1 --T 1 --A 1
  --X 1 --F 1`; matches the preliminary ~22 Low estimate)
- **Scope:** `scripts/delegate-low-rri.py`, its test module
- **Depends on:** T1

### Objective

Ensure the existing Gemma Developer packet/result already carries the fields
T1's capsule/bundle schema expects (work item id, allowed paths, acceptance
criteria, capsule hash, outcome), adding a thin adapter rather than changing
the tagged-block wire contract Gemma itself must produce (that contract is
fixed by `RRI_POLICY.md` and out of scope here).

### Happy paths considered

- `HP-1`: an existing successful `delegate-low-rri.py` result, when passed
  through the new adapter, produces a schema-valid attempt bundle.

### Edge cases considered

- `EC-1`: a `BLOCKED` or timeout result still produces a valid bundle with
  the corresponding `outcome`, not a crash in the adapter.

### Acceptance criteria

- No change to the tagged-block contract Gemma must return.
- Adapter is additive and covered by unit tests for HP-1/EC-1.

### Evidence to emit

- Unit-test output.

### Completion evidence

- Diff: `scripts/delegate-low-rri.py` +42, `scripts/delegate_low_rri_test.py`
  +65, 107 lines total — under the 500-line direct-implementation threshold,
  so implemented directly rather than delegated to Qwen35.
- Added `build_attempt_bundle(delegation, capsule_hash, model, start_ts,
  end_ts)`: a pure, additive adapter mapping a `delegate-low-rri.py`
  `delegation` result dict (`status`, `files`, `test_commands`) onto one T1
  attempt bundle. `DELEGATION_STATUS_TO_OUTCOME` maps `patch`/`no_patch` to
  `success` and `blocked` to `blocked` (the one Low-band terminal status that
  maps directly onto a T1 `VALID_OUTCOMES` value — Low-band delegation has no
  `repair-needed`/`escalated` concept of its own, unlike the Moderate/
  Med-high session loop T2 adapts). Returns `None` when `capsule_hash` is
  falsy, mirroring T2's `build_attempt_bundles` convention for
  pre-T1/T3-adoption callers.
- Test results: `scripts.delegate_low_rri_test` full module, 87 tests, `OK`
  (was 83 pre-change; +4 net new). New `BuildAttemptBundle` class, 4 tests,
  `OK`:
  - `test_hp1_patch_result_produces_schema_valid_bundle` (HP-1) — asserts
    `outcome == "success"`, `implementer_id == "gemma"`, and validates the
    bundle through `handoff_schema.validate_attempt_bundle`.
  - `test_ec1_blocked_result_produces_valid_bundle_not_a_crash` (EC-1) —
    `BLOCKED` status maps to `outcome == "blocked"`, empty `diff_ref`, no
    crash, schema-valid.
  - `test_ec1_no_patch_result_produces_valid_bundle` (EC-1) — `no_patch`
    maps to `outcome == "success"` (nothing to do is not a failure),
    schema-valid.
  - `test_missing_capsule_hash_returns_none` — pre-adoption caller with no
    capsule hash gets `None`, not a malformed bundle.
- No change to `validate_delegation_payload`, `build_payload`,
  `build_replacement_payload`, or any tagged-block marker/rule — the
  contract Gemma must return is untouched, confirmed by the unchanged
  85-tests-still-passing pre-existing suite alongside the 4 new ones.

### Status artifacts affected

- This ledger (T3 marked done; T4 now has two of its three T2/T3/T5
  dependencies satisfied — only T5 remains before T4 can be presented).

### Reflection strategy

RRI recomputed at 23, Low band (≤25) — Reflection skipped per the workflow
guide's Low-band gate (delegate/implement, validate, review against
requirements, verify, report — no peer-model review pass). Implemented
directly (107-line diff, under the user's 500-line threshold) rather than
delegated to Gemma via Ollama, since this session was already the
orchestrator of record mid-plan; reviewed the adapter myself against
`handoff_schema.py`'s `BUNDLE_REQUIRED_FIELDS`/`VALID_OUTCOMES` and against
T2's `build_attempt_bundles` for convention parity (capsule-hash-gated
`None` return, `review_verdict: "pending"`), then verified via the full
test-module run above.

---

## T4 — Cloud conciliator checklist / gate

- **Status:** `[x] Done`
- **Effort:** L
- **Preliminary RRI:** ~33 Moderate; `C2 F2 D3 T3 A1 K2 P2 X3`
- **Final RRI:** 42 Med-high; `C2 F1 D3 T3 A1 K2 P2 X3` (recomputed at
  presentation time; higher than the preliminary estimate)
- **Scope:** `scripts/local-agent/conciliator_checklist.py`,
  `scripts/local-agent/conciliator_checklist_test.py`; references but does
  not duplicate `scope_check.py`, `organization_gate.py`
- **Depends on:** T2, T3, T5

### Objective

Turn the plan's informal "Cloud conciliator checklist" into a callable gate
that reads a capsule + its attempt bundles and reports pass/fail per item
(scope, acceptance, review-verdict-recorded, repair-budget-respected,
reflection-log-present-if-required, status-artifacts-named) — advisory
output the primary agent consults before closing a task, not an
auto-closer.

### Happy paths considered

- `HP-1`: a complete, in-budget, reviewed bundle set reports all six items
  PASS.

### Edge cases considered

- `EC-1`: a bundle set missing a required Reflection log for an RRI 26+ task
  reports that specific item FAIL with a clear reason, not a generic
  failure.
- `EC-2`: a bundle set whose diff touches a path outside the capsule's
  `allowed_paths` reports the scope item FAIL and does not mask it behind
  other passing items.

### Acceptance criteria

- Checklist is read-only (no filesystem writes beyond its own report
  artifact) and never itself closes or approves a task.
- Every one of the six checklist items from the plan is represented.
- Unit tests cover HP-1, EC-1, EC-2.

### Evidence to emit

- Unit-test output; one example checklist report from a mocked bundle set.

### Status artifacts affected

- This ledger.

### Reflection strategy

Med-high band (RRI 42): both review phases route to Qwen27
(`qwen3.6:27b-q4_K_M`), per the local-agent lane's Med-high routing.

### Execution summary

**Phase-1 task-analysis review (Qwen27), 4 rounds, converged:** wrote the
task packet, ran it through `scripts/peer-workflow-review.py --phase task`.
Round 1 found the UNKNOWN-vs-FAIL aggregation rule undefined and `overall`
missing from acceptance criteria; round 2 found EC-1 wording still
ambiguous, no mixed-status test coverage, and item-4 budget precedence
unclear; round 3 found two real gaps (empty `allowed_paths`/`diff_ref`
edge case, non-bool `passed` handling); round 4 found one more real gap
(non-dict `test_results` would crash `.get()`). Each round's real findings
were folded into the packet before re-review; iteration stopped once
remaining findings were self-resolving or reviewer re-confirmations, not
new defects.

**Implementation — Qwen3.5a (`qwen3.6:35b-a3b`) via `run_local_task.py`,**
disposable worktree `/private/tmp/dubbridge-t4-conciliator` (branch
`agent/t4-conciliator-checklist`), cut from commit `79bcd08` (T0–T3/T5
committed first so `handoff_schema.py` and its adapters were visible inside
the isolated worktree). Ran to `budget_exhausted` at the 30-turn cap, but
produced a complete, materially correct module: `conciliator_checklist.py`
(all six items, fail-closed aggregation matching the reviewed spec) +
`conciliator_checklist_test.py` (40 tests). Evaluation: 38/40 passed; the 2
failures were confirmed (by direct inspection of `_check_reflection` against
the approved spec) to be a test-authoring bug, not an implementation bug —
the implementation correctly treats a known RRI < 26 as PASS for the
reflection item, but two tests incorrectly asserted UNKNOWN for that case.

**Cross-delegated repair — 5 attempts, all failed:** per the owner's
2026-07-23 instruction to minimize direct involvement and cross-delegate
between Gemma and Qwen3.5a on failure, the 2-test fix was sent to: Gemma
(prose packet) — rewrote the entire file from scratch with a wrong import
path and a wrong `build_report` call signature, deleting 34 of 40 passing
tests; Qwen3.5a (same packet) — first attempt truncated by token limit
(no-op), second attempt (raised `--num-predict`) also rewrote the whole
file with a different wrong import (`from local_agent...`, missing the
`scripts.` prefix and using an invalid underscore package name) and
imported nonexistent helper functions; Gemma again with a restrictive
`--mode before-after` packet (exact find/replace blocks, explicit
"surgical two-line edit only" instruction) — produced a syntactically
broken 8-line file (only the two replacement blocks, no class/import
wrapper); Qwen27 as a last-resort implementer — returned `status: blocked`,
misreading its own sandboxed repo-root access as "file not present."

**Escalation:** per `HITL_AUTONOMY_POLICY.md`'s repair-budget/escalation
conditions (same defect, repeated cross-model failure) and
`feedback_local_pipeline_roles.md`'s documented exception for exactly this
case, reported the failure pattern to the owner; owner confirmed (after one
more failed last-resort attempt) direct application. Applied the two-line
fix myself: `test_reflection_rri_low_unknown`'s expected status corrected
from `"UNKNOWN"` to `"PASS"` (matches the spec: RRI known and <26 is a
valid PASS, not missing context), and `test_ec4_all_unknown_overall_unknown`
changed `_capsule(rri=10)` to `_capsule()` (EC-4 requires RRI genuinely
absent, not a known low value). Verified: `python3 -m unittest
scripts.local-agent.conciliator_checklist_test -v` → `Ran 40 tests ... OK`.

**Phase-2 code review (Qwen27):** verdict `findings`, 8 findings on the
747-line combined content. Disposition: the four "HIGH" findings on budget
precedence (`build_report`/`_check_budget`/`_get_band_from_capsule`) were
each the reviewer talking itself through the logic and concluding "this is
correct" / "this works" / "by design" in its own text — verified directly
against the code and the task spec's own "Item 4 precedence rule" (capsule
fields take precedence over caller-supplied `band`/`rri`, exactly as
implemented); no action needed. One real MEDIUM (root path `"/"` in
`allowed_paths` normalizes to an empty string, a theoretical edge case) was
evaluated and left as-is: this repo's path convention (matching
`scope_check.py`'s own `_normalise_allowed_path`, which rejects paths
starting with `/`) never uses `/` as an allowed-path entry, so this is
unreachable in practice, not a live defect. Two LOW/MEDIUM findings were
genuine missing-test suggestions (mixed trailing-slash normalization,
`passed`+`status` key conflict, `rri=26` boundary) — noted but not added,
since they're coverage nice-to-haves outside the approved acceptance
criteria (HP-1/EC-1..4), not defects; can be added in a follow-up if T6's
pilot surfaces a related gap.

**Final evidence:** `scripts/local-agent/conciliator_checklist.py` (364
lines) + `scripts/local-agent/conciliator_checklist_test.py` (380 lines),
copied from the worktree into the main tree; `python3 -m unittest
scripts.local-agent.conciliator_checklist_test -v` → `Ran 40 tests ... OK`
in both the worktree and the main repo tree. All six checklist items
(scope, acceptance, review, budget, reflection, status_sync) implemented
with fail-closed aggregation (FAIL dominates UNKNOWN dominates PASS,
matching the approved spec). Read-only: no filesystem writes beyond an
optional caller-requested report path; never closes or approves a task.

### Follow-up round — root-path fail-closed guard (2026-07-23)

The owner asked to fix, rather than leave as a note, the phase-2 review's
MEDIUM finding above (root path `"/"` normalising to `""`). Initial
cross-delegation attempts (Gemma, then Qwen3.5a) on `--mode before-after`
produced byte-identical broken output — a 373-line diff, `NameError: name
'Any' is not defined`, file truncated to 42 lines — from both models on
the same call. Per the owner's explicit instruction not to assume model
fault when two independent models fail identically, this was investigated
rather than retried: inspecting the actual applied diff's git hunks showed
the corruption spanned from line 1 to line 364 of the file, proving the
`--before-file` supplied to the CLI did not match the small span described
in the accompanying prose packet. `apply_before_after()` in
`scripts/delegate-low-rri.py` performs a blind literal
`original.replace(before_block, after_block)` — the model never sees or
echoes the BEFORE text — so an oversized/mis-scoped before-file silently
replaces far more of the file than intended, regardless of which model
supplies the (correctly small) AFTER text. Root cause was a caller-side
packet-construction defect, not a defect in either model or in the tool's
existing replace/validate logic.

Long-term fix (out of T4's file scope, landed in
`scripts/delegate-low-rri.py` and `scripts/delegate_low_rri_test.py`,
RRI 22 Low via `scripts/rri.py`, delegated to Gemma in 4 small before-after
calls, all applied clean on the first attempt): `apply_before_after` now
rejects any BEFORE block over `MAX_BEFORE_LINES` (40) before calling the
model, and rejects (post-substitution) any `.py` result that fails
`ast.parse`, both fail-closed with a clear `RuntimeError`. Two regression
tests added. This is now reusable by any future before-after delegation,
not just T4.

With the tool hardened, the root-path guard itself was re-delegated as two
small single-anchor before-after calls (`_normalise_path`,
`_check_scope`) plus one for the new test — all against
`scripts/local-agent/conciliator_checklist.py` /
`conciliator_checklist_test.py`. The first `_check_scope`-adjacent test
edit was attempted with Gemma; the new `ast.parse` gate caught a
syntactically broken response and refused to write it (no corruption, no
manual revert needed); per
[[feedback_cross_delegate_on_failure]] the same edit was re-sent to
Qwen3.5a (`qwen3.6:35b-a3b`), which produced a clean result on the first
try. All other calls succeeded on the first attempt with either model.

`_normalise_path` now raises `ValueError` when a non-empty path normalises
to an empty string; `_check_scope` catches this per-path, collects
offending raw paths, and returns a scope `FAIL` naming them — before the
prior "empty diff_ref → PASS" branch. New test:
`test_scope_root_path_touched_fails_closed`.

**Follow-up phase-2 review (Qwen27, delta-scoped — Med-high band routing
still applies):** verdict `findings`, none HIGH. Summary: "The fix
correctly implements fail-closed behavior for root paths." Three findings,
all coverage/robustness nice-to-haves, not defects: (1) LOW — test uses
the same `next(...)` lookup pattern as every other test in this file,
flagged as slightly fragile; left as-is for consistency with the existing
suite. (2) MEDIUM — suggests additional test coverage for other inputs
that also normalise to empty (`"./"`, `"///"`); the guard itself is
generic (fires on any non-empty path normalising to empty, not a
root-specific check) so these cases are already covered by the
implementation, only the test enumeration is incomplete. (3) LOW — the
FAIL reason string could grow long if many bad paths are present in one
bundle; a real but minor robustness edge case. Disposition matches the
precedent set earlier in this same task: noted, not added — coverage
nice-to-haves outside the approved acceptance criteria, not defects;
revisit if T6's pilot surfaces a related gap.

**Final evidence (this round):** `python3 -m unittest
scripts.local-agent.conciliator_checklist_test -v` → `Ran 41 tests ... OK`.
`python3 -m pytest scripts/delegate_low_rri_test.py -q` → `89 passed`
(includes the 2 new guard-regression tests). Files remain uncommitted
pending the owner's commit approval for this round, same as the rest of
T4/T0–T3/T5's local state.

---

## T5 — Local Architect preplanning capsule path (ADR-037)

- **Status:** `[x] Done`
- **Effort:** S
- **Preliminary RRI:** ~24 Low; `C1 F1 D2 T1 A1 K1 P1 X2`
- **Final RRI:** 22 Low; `C1 F0 D2 T1 A1 K1 P1 X2`
- **Scope (actual):** new `scripts/local-architect/adr037_handoff_mapping.py`
  + `adr037_handoff_mapping_test.py` (additive; `run_analysis.py` and
  `handoff_schema.py` unchanged, per acceptance criteria)
- **Depends on:** T1

### Objective

Express the ADR-037 project packet (already produced by T3 of
`docs/tasks/adr037-local-architect-direct-project.md`) as a context capsule
per T1's schema, and its resulting advisory artifact as a (non-implementing,
advisory-only) attempt bundle, so the Local Architect's preplanning role is
visible in the same handoff vocabulary as the implementer/reviewer lanes —
without granting it any implementation or approval authority it does not
already have under ADR-037.

### Happy paths considered

- `HP-1`: an existing ADR-037 packet (e.g. the frozen `S-140` packet) maps
  onto T1's capsule schema without information loss.

### Edge cases considered

- `EC-1`: the resulting "attempt bundle" for an advisory run is tagged
  `advisory-only` and the conciliator checklist (T4) must never treat it as
  satisfying the local-agent lane's implementer or reviewer requirements.

### Acceptance criteria

- No change to ADR-037's authority boundary (§1 may/may-not list) or to
  `run_analysis.py`'s existing wrapper behavior beyond the capsule/bundle
  mapping.
- Unit tests cover HP-1/EC-1.

### Evidence to emit

- Unit-test output; one mapped example from the existing `S-140` packet.

### Status artifacts affected

- This ledger; note in
  `docs/evaluations/adr037-direct-project-report.md` if the mapping surfaces
  a schema gap the ADR-037 packet didn't previously need to name explicitly.

### Reflection strategy

Likely Low band; if recomputation lands 26+, apply the Moderate 2-pass
cycle with pass 1 on mapping fidelity and pass 2 on the `advisory-only`
tagging never being droppable by the conciliator gate.

### Execution summary

RRI recomputed at 22 (Low, unchanged band) — delegated directly to local
Gemma (`gemma4:26b-a4b-it-qat`) via `delegate-low-rri.py`, scoped to the two
new files via `--allow-path`. Gemma returned `STATUS: PATCH`; diff applied
cleanly (174 lines, 2 files).

Mechanical repair (in-scope, no re-delegation): fixed three import/path bugs
in Gemma's output — a non-existent `local_agent` package import (the
directory is `local-agent`, not a valid Python package name), a duplicated
path segment, and a module-identity bug from `handoff_schema` being imported
twice under two distinct module objects (same class of bug already
documented in `run_local_task.py`), fixed by registering the loaded module in
`sys.modules` before a second load site imports it.

Evidence:
- Unit tests: 3/3 pass (`HP-1`, `EC-1`, plus a third case: failed analysis →
  `outcome: "blocked"`).
- HP-1 verified against the real frozen `S-140` packet (not only the test's
  synthetic fixture): `map_packet_to_capsule` reproduces all
  `CAPSULE_REQUIRED_FIELDS` with no information loss, `manifest_hash` is a
  valid 64-char SHA-256.
- EC-1 verified against the real `S-140` analysis artifact
  (`t4-analysis-artifact.json`): the resulting bundle carries
  `outcome: "advisory-only"`, and `handoff_schema.validate_attempt_bundle`
  correctly raises `ValidationError` on `field_name == "outcome"` when given
  that bundle, since `"advisory-only"` is intentionally not in
  `VALID_OUTCOMES` — this is what keeps T4's conciliator checklist from ever
  being able to treat an advisory-only bundle as satisfying an
  implementer/reviewer requirement through the standard validator.
- No change to ADR-037's authority boundary or to `run_analysis.py`'s
  existing behavior — neither file was touched.

Review gate: RRI 22 is Low band (0-25); the mandatory Gemma Reviewer / D14
gate (`docs/policies/HITL_AUTONOMY_POLICY.md`) applies only to Moderate+
(RRI 26+) and does not apply here. No reflection-log artifact required for
this band.

---

## T6 — Pilot and metrics

- **Status:** `[ ] Open`
- **Effort:** M
- **Preliminary RRI:** ~32 Moderate; `C0 F3 D3 T2 A2 K3 P2 X3`
- **Scope:** `docs/evaluations/local-first-cloud-local-handoff-pilot.md`
  (new); no product code
- **Depends on:** T4

### Objective

Run a small number (target: 3–5) of real RRI 26–55 tasks through the full
capsule → attempt-bundle → conciliator-checklist path end-to-end, and record
whether the checklist's six items correctly predicted the primary agent's
actual closure decision on each.

### Happy paths considered

- `HP-1`: a task where the checklist reports all-PASS and the primary agent
  independently reaches the same closure decision.

### Edge cases considered

- `EC-1`: a task where the checklist flags a FAIL item and the primary agent
  must decide to escalate, repair, or override with a documented reason —
  recorded as a disagreement case, not hidden.

### Acceptance criteria

- At least 3 real tasks run through the path.
- Each run's checklist output vs. primary-agent decision is recorded in the
  evaluation doc.
- No task is closed on the checklist's authority alone — the primary agent
  remains the decision-maker per the plan's role inventory.

### Evidence to emit

- Per-task checklist report + primary-agent disposition, in the evaluation
  doc.

### Status artifacts affected

- `docs/evaluations/local-first-cloud-local-handoff-pilot.md`; this ledger.

### Task-analysis review

`n/a` — evaluation/reporting task; no code authored by T6 itself (it
consumes T1–T5's already-reviewed code). Standard band-routed review applies
to any code fix T6 itself needs to make to those modules, tracked as a
separate task if material.

---

## T7 — Policy sync / go-no-go

- **Status:** `[ ] Open`
- **Effort:** S
- **Preliminary RRI:** ~28 Moderate; `C0 F2 D2 T1 A1 K2 P2 X2`
- **Scope:** this plan/ledger; cross-references added to
  `AGENT_WORKFLOW_GUIDE.md` / `RRI_POLICY.md` only if T6 recommends adoption
- **Depends on:** T6

### Objective

Decide, from T6's evidence, whether the capsule/bundle vocabulary and
conciliator checklist become a referenced convention in
`AGENT_WORKFLOW_GUIDE.md` (a documentation cross-reference, not a new gate),
or remain a standalone plan/ledger without further propagation.

### Acceptance criteria

- Go/no-go decision recorded with the T6 evidence it rests on.
- If GO: `AGENT_WORKFLOW_GUIDE.md` § Per-task discipline gains a short
  cross-reference to this plan's conciliator checklist as an optional aid —
  explicitly not a new mandatory gate, since introducing one would itself be
  an RRI 56+-class policy decision requiring its own presentation.
- If NO-GO: this plan/ledger is marked `status: superseded-by-evaluation` (or
  similar) with the reason, per the same discipline used for retired
  approaches (e.g. `local-agent-semantic-editing.md`).
- Human approval required before any edit to `AGENT_WORKFLOW_GUIDE.md` or
  `RRI_POLICY.md`, per those files' own authority and the RRI 26+ gate this
  task's recomputed score is expected to land in.

### Evidence to emit

- The go/no-go statement and, if GO, the exact cross-reference text added.

### Status artifacts affected

- This plan/ledger; `AGENT_WORKFLOW_GUIDE.md` only on GO.

---

## Closure note for T0

T0 is docs-only (plan + this ledger), exempt from the Gemma Reviewer/D14
phase-2 gate and from unit coverage certification per
`AGENT_WORKFLOW_GUIDE.md` § Development Closure Rule and the RRI 0–25 Low
band handling in `RRI_POLICY.md`. Its closure requirements are:

1. Doc-consistency and OKF frontmatter checks pass (see command output
   recorded below).
2. `docs/plan/roadmap.md` was evaluated and found **not** materially
   affected — this slice is process/tooling documentation, not a
   roadmap-tracked product slice, so no roadmap row references it.
3. No `[x] Done` unit-coverage certification or owner-verification block is
   required or presented for T0, consistent with its docs-only exemption.
