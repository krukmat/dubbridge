---
type: TaskList
title: "Tasks: Local model stack restructure — Qwen27 developer, Muse Glimmer reviewer"
plan: docs/plan/local-model-stack-restructure-2026-08.md
status: Proposed
slice: local-model-stack-restructure-2026-08
---

# Tasks: Local Model Stack Restructure (2026-08)

## Approval and aggregate risk

- **Owner directive:** 2026-08-11. Target bindings and the four scoped
  clarifying decisions (D1–D4) were confirmed interactively this session;
  D5/D6 are stated assumptions pending explicit confirmation at the approval
  checkpoint below.
- **Aggregate RRI:** 99 Very high (`C3 F4 D3 T3 A3 K4 P2 X4`,
  `arch_decision` + `many_files` + `no_verification`). Direct monolithic
  implementation is forbidden; decomposed into the table below, each task
  independently scored ≤ 55.
- **Phase-1 rule:** ADR/plan/policy/proposal-only tasks are exempt (recorded
  `n/a`). T4a and T4b are real development tasks and go through the
  band-resolved phase-1 review before implementation starts.
- **Phase-2 rule:** every development task passes the band-resolved
  code-solution review before closure; ADR/plan/policy-only tasks record
  `n/a` with the exemption stated.
- **Sequencing caveat:** T4b changes *which reviewer model* T4a and T4b
  themselves should be reviewed by. Until T1+T2 land, run phase-1/phase-2
  review for T4a/T4b under the **pre-change** chain (qwen27 primary for
  26–55, Gemma fallback, D14 final) to avoid a bootstrapping paradox —
  record this explicitly in each task's review evidence block.

## Task table

| Task | Status | RRI | Scope | Depends on |
|---|---|---:|---|---|
| T1 ADR-036 Amd.2 + ADR-037 Amd.1 + ADR-038 Amd.1 | `[x] Done` | 41 Med-high | 3 ADR files + `docs/adr/README.md` index | owner approval |
| T2 Policy/workflow doc sync | `[x] Done` | 19 Low | `AGENTS.md`, `AGENT_WORKFLOW_GUIDE.md`, `RRI_POLICY.md`, `HITL_AUTONOMY_POLICY.md` | T1 |
| T3 Resolve `qwen-review-latency-mitigation.md` proposal | `[x] Done` | 7 Low | 1 proposal doc | T1 |
| T4a Implementer binding swap | `[x] Done` | 26 Moderate | `run_local_task.py`, `run_med_high_task.py`, `run_stage1_benchmark.py` + tests | T1 |
| T4b Reviewer/Architect binding swap + decoupling | `[x] Done` | 44 Med-high | `gemma_local.py`, `gemma-code-review.py`, `peer-workflow-review.py`, `run_analysis.py`, `med_high_gate.py` + tests | T1 |
| T4c `delegate-low-rri.py` stall-fallback cleanup | `[x] Done` | 18 Low | 1 script + test | T1 |
| T5 Regenerate `AGENTS.override.md` | `[x] Done` | 3 Low | 1 generated file | T2 |
| T6 Cross-reference notes on open task/plan docs | `[ ] Pending` | 16 Low | 6 live task/plan docs | T1 |
| T7 Integrated verification, reviews and closure | `[ ] Pending` | n/a (rollup) | tests, QA, status closure | T3, T4a, T4b, T4c, T5, T6 |

## T1 — ADR-036 Amendment 2 + ADR-037 Amendment 1 + ADR-038 Amendment 1

- **Type:** ADR-only; phase-1 and phase-2 review exempt.
- **RRI:** 41 Med-high (`C0 F2 D1 T2 A2 K2 P1 X3`, `arch_decision` penalty).
- **Allowed paths:** `docs/adr/ADR-036-local-first-agentic-implementation-band.md`,
  `docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md`,
  `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`,
  `docs/adr/README.md`.
- **Objective:**
  - ADR-036 Amendment 2: update §1's local model stack table — implementer
    binding `qwen3.6:35b-a3b` → `qwen3.6:27b-q4_K_M`; remove
    `qwen3.6:35b-a3b` from the stack entirely; record the deliberate
    re-entry override of the original dense/bandwidth-bound rejection
    rationale (owner directive 2026-08-11, smoke-test evidence linked from
    the plan); update §5 reviewer-pairing table to reflect the new
    implementer/reviewer pairs (qwen27 ↔ Muse Glimmer for Low,
    qwen27 ↔ Gemma for 26–55); note this also **retires** the informal
    "owner directive 2026-07-21" policy-layer override that had made qwen27
    the 26–55 primary reviewer (that override lived in policy docs, not in
    ADR-036 itself — T2 removes the policy-doc prose).
  - ADR-037 Amendment 1: the Local Architect / Complex Analyst binding moves
    from `qwen3.6:27b-q4_K_M` to `muse-glimmer:30b-q4_K_M`. Record the
    rationale (qwen27 reassigned to implementer by ADR-036 Amd.2; same-model
    self-review-conflict avoidance) and the smoke-test evidence. Add an
    explicit open question: production observation of Muse Glimmer's
    `med-high-refinement-v1` schema-adherence rate under real Med-high load
    is still pending (the plan's smoke test used a generic PASS/findings
    contract, not the full refinement schema) — flag for follow-up, not
    blocking.
  - ADR-038 Amendment 1: update all bindings referenced in §2/§4/Risk
    analysis/Alternatives — "Qwen27 advisory refinement" → "Muse Glimmer
    advisory refinement (per ADR-037 Amendment 1)"; "exact local implementer
    binding `qwen3.6:35b-a3b`" → "`qwen3.6:27b-q4_K_M` (per ADR-036
    Amendment 2)"; flag that the "Let Qwen27 implement: rejected" alternative
    is now superseded by the owner's 2026-08-11 explicit re-entry decision —
    do not delete the historical alternative, append a dated note.
  - `docs/adr/README.md`: no status-token changes (all three ADRs stay
    `Accepted`), but confirm each index row's one-line description still
    matches the amended prose if it names a specific model.
- **Acceptance:** every amendment is a dated, appended section (matching
  ADR-036 Amendment 1's pattern) — no rewriting of the original Decision/
  Context/Alternatives text. `- **Status:**` prose stays `Accepted` in all
  three files (frontmatter `status:` unchanged). `make qa-docs` passes
  (ADR index parity, no dangling ADR references).
- **Evidence to emit:** the three amended ADR files; the plan's smoke-test
  transcript (already captured in
  `docs/plan/local-model-stack-restructure-2026-08.md`) cited by reference.
- **Status artifacts affected:** `docs/adr/README.md` (only if a
  description line goes stale), this task ledger's `[x] Done` row.
- **Review:** `Task-analysis review: n/a - ADR/plan/task-ledger exemption`.
- **Closure:** `Code-solution review: n/a - ADR/plan/task-ledger exemption`.

### T1 completion record

- **Amendments added:** ADR-036 Amendment 2, ADR-037 Amendment 1, ADR-038
  Amendment 1 — each a dated, appended section; no rewriting of original
  Decision/Context/Alternatives prose. `- **Status:**` prose and frontmatter
  `status:` stay `Accepted` in all three files.
- **`docs/adr/README.md`:** no status-token change. Reviewed whether the
  ADR-037 index row/title ("Qwen3.6-27B as Local Architect and Complex
  Analyst") still matches post-amendment content: it now names the
  *pre-amendment* binding. Decision: keep the historical title/filename
  unchanged (matches the ADR-036 Amendment 1 precedent, which did not rename
  the ADR when the `gemma4:12b-mlx` binding it discussed was retired); the
  binding change is captured in ADR-037 Amendment 1's text and cross-linked.
  No renumbering or deletion involved, so the ADR change-propagation
  deletion rule does not apply.
- **Verification:** `make qa-docs` — all 5 gates (doc consistency, task unit
  coverage tests, task completion evidence, roadmap drift, OKF frontmatter)
  passed.
- **Status artifacts affected:** this ledger row (`[x] Done`).

## T2 — Policy/workflow doc sync

- **Type:** docs-only (propagation of a decision already recorded in T1);
  phase-1 and phase-2 review exempt.
- **RRI:** 19 Low (`C0 F2 D0 T1 A1 K2 P1 X2`).
- **Allowed paths:** `AGENTS.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
  `docs/policies/RRI_POLICY.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`.
- **Objective:** propagate T1's decision into every prose location the
  reconnaissance pass identified:
  - Band-routed peer review tables (`AGENTS.md` reviewer-token enum;
    `AGENT_WORKFLOW_GUIDE.md` §Band-routed peer review; `RRI_POLICY.md`
    canonical band table + §Local pipeline phase-1/phase-2 reviewer
    override, which is retitled/rewritten since the override itself
    reverts) — Low band reviewer → Muse Glimmer; 26–55 band reviewer →
    Gemma (reverted); fallback chains → Low: `muse-glimmer → gemma → d14`
    (D3); 26–55: `gemma → muse-glimmer → d14` (D5, **flagged for owner
    confirmation**, not unilaterally decided).
  - Implementer defaults: `DUBBRIDGE_LOCAL_AGENT_MODEL` default
    documentation in `AGENT_WORKFLOW_GUIDE.md`, `RRI_POLICY.md`,
    `HITL_AUTONOMY_POLICY.md` → `qwen3.6:27b-q4_K_M`; remove
    `qwen3.6:35b-a3b` mentions (implementer table rows, ADR-038 diagram
    text, Med-high routing prose, local-stack precheck model list in §0).
  - § Local Architect / Complex Analyst section in `AGENT_WORKFLOW_GUIDE.md`
    → rebind to Muse Glimmer; update the ADR-038 Mermaid diagram's node
    labels (`Q27["Qwen27 advisory refinement..."]` → Muse Glimmer).
  - `HITL_AUTONOMY_POLICY.md` § Med-high Architect-refined single-attempt
    gate and § Band-routed peer review sections — same substitutions.
- **Acceptance:** no remaining reference to `qwen3.6:35b-a3b` as an active
  binding anywhere in these four files (historical/rationale mentions in
  amended ADR text are out of this task's scope); `qwen3.6:27b-q4_K_M`
  appears only in the implementer and Local-Architect-via-ADR-037 contexts,
  never as the 26–55 reviewer; `make qa-docs` passes.
- **Evidence to emit:** diff of the four files.
- **Status artifacts affected:** none beyond the four files themselves and
  this ledger.
- **Review:** `Task-analysis review: n/a - docs-only exemption`.
- **Closure:** `Code-solution review: n/a - docs-only exemption`.

### T2 completion record

- **Substitution applied consistently across all four files:** Low band
  (0–25) primary reviewer → Muse Glimmer (`muse-glimmer:30b-q4_K_M`), Gemma
  intermediate fallback, D14 final; 26–55 band primary reviewer reverted →
  Gemma (`gemma4:26b-a4b-it-qat`), Muse Glimmer intermediate fallback, D14
  final (retires the 2026-07-21 qwen27 override); Local Architect / Complex
  Analyst (ADR-037 role, used for Med-high advisory refinement) → Muse
  Glimmer; local implementer default (`DUBBRIDGE_LOCAL_AGENT_MODEL`,
  Moderate + Med-high single-attempt session) → `qwen3.6:27b-q4_K_M`.
  `qwen3.6:35b-a3b` removed from all four files.
- **Sections retitled/rewritten:** `RRI_POLICY.md § Local pipeline
  phase-1/phase-2 reviewer override` → `§ Local pipeline phase-1/phase-2
  reviewer bindings` (now covers both the 0–25 and 26–55 bindings, since the
  override itself reverted). `AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer` →
  `§ Gemma Reviewer / Muse Glimmer Reviewer`; Step 1-A/1-B closure-checklist
  headers and bodies updated to name the resolved model per band.
  `§ Local Architect / Complex Analyst (ADR-037)` — the "scoped exception"
  paragraph making that role double as 26–55 reviewer is marked retired.
- **Left deliberately unchanged (per plan Design decision 2 / D6):**
  `DUBBRIDGE_LOW_RRI_MODEL` and Gemma Developer's binding
  (`AGENT_WORKFLOW_GUIDE.md` lines ~402-405, `RRI_POLICY.md` §Low RRI
  handling) — Gemma Developer (patch delegation) stays Gemma; only the
  reviewer-role default changes, via a decoupled constant introduced in T4b.
- **Verification:** `make qa-docs` — `check-doc-consistency.sh` now fails
  with the expected, single finding "`AGENTS.override.md`: content is stale
  relative to its sources" — this is the anticipated T2→T5 dependency (T5
  regenerates the override file from these four sources) and is not a defect
  in this task. All other `make qa-docs` gates were not reached because
  `check-doc-consistency.sh` runs first and exits non-zero; re-verified after
  T5 in T7.
- **Status artifacts affected:** this ledger row (`[x] Done`).

## T3 — Resolve `docs/proposals/qwen-review-latency-mitigation.md`

- **Type:** proposal resolution; phase-1/phase-2 exempt.
- **RRI:** 7 Low.
- **Objective:** this open proposal (status `Proposed`, dated 2026-08-09)
  specifically argues about qwen27's latency as the 26–55 reviewer — a role
  T1/T2 just reverted away from qwen27. Update its `status:` to `Superseded`
  (superseded by ADR-036 Amendment 2 / ADR-037 Amendment 1) with a one-line
  pointer, rather than leaving it silently stale.
- **Acceptance:** frontmatter `status:` and prose status line agree; the
  file states what superseded it and why in one paragraph; original content
  preserved below as historical record.
- **Review:** `Task-analysis review: n/a - proposal-only exemption`.
- **Closure:** `Code-solution review: n/a - proposal-only exemption`.

### T3 completion record

- `status:` frontmatter and prose `- **Status:**` line both updated to
  `Superseded`, agreeing per OKF/ADR-propagation parity conventions.
- Superseding cause and pointer to the governing ledger recorded in one
  paragraph; original investigation/proposal content preserved unmodified
  below it.
- **Status artifacts affected:** this ledger row (`[x] Done`).

## T4a — Implementer binding swap

- **Allowed paths:** `scripts/local-agent/run_local_task.py`,
  `scripts/local-agent/run_local_task_test.py`,
  `scripts/local-agent/run_med_high_task.py`,
  `scripts/local-agent/run_med_high_task_test.py`,
  `scripts/local-bench/run_stage1_benchmark.py`,
  `scripts/local-bench/run_stage1_benchmark_test.py`.
- **RRI:** 26 Moderate (`C1 F2 D1 T1 A1 K2 P1 X2`).
- **Objective:** swap the hard-required and default implementer bindings
  from `qwen3.6:35b-a3b` to `qwen3.6:27b-q4_K_M`:
  `MED_HIGH_REQUIRED_MODEL` and the `DUBBRIDGE_LOCAL_AGENT_MODEL` CLI default
  in `run_local_task.py`; `MED_HIGH_RUNNER_MODEL` in `run_med_high_task.py`;
  the `--model` default in `run_stage1_benchmark.py`.
- **HP-1:** `DUBBRIDGE_LOCAL_AGENT_MODEL` unset → the Moderate (26–40)
  local-first runner resolves `qwen3.6:27b-q4_K_M` as the implementer.
- **HP-2:** an approved Med-high `GO_LOCAL` route → `run_med_high_task.py`
  supervises the bounded session on `qwen3.6:27b-q4_K_M`, not the retired
  `qwen3.6:35b-a3b`.
- **EC-1:** an explicit `DUBBRIDGE_LOCAL_AGENT_MODEL` override to a
  different model tag still takes precedence over the new default (the
  swap changes the default only, not the override precedence).
- **EC-2:** `MED_HIGH_REQUIRED_MODEL`'s fail-closed hard-check rejects any
  model tag other than the new `qwen3.6:27b-q4_K_M` binding — no silent
  model substitution, same invariant as before, new tag.
- **Acceptance:** `python3 scripts/local-agent/run_local_task_test.py`,
  `python3 scripts/local-agent/run_med_high_task_test.py`,
  `python3 scripts/local-bench/run_stage1_benchmark_test.py` — all passing,
  including updated assertions for the new constants.
- **Evidence to emit:** test run output; grep confirmation of zero remaining
  `qwen3.6:35b-a3b` references in the touched files.
- **Reflection passes required:** 2 (Moderate).
- **Review:** band-resolved phase-1/phase-2 (pre-change chain per the
  sequencing caveat above — qwen27 primary, Gemma fallback, D14 final,
  since T1/T2 may not have landed yet when this runs).

### T4a completion record

- **Sequencing-caveat resolution:** T1 and T2 both landed (`[x] Done`)
  before T4a began, so the caveat's "until T1+T2 land" condition no longer
  holds. Phase-1/phase-2 review for T4a therefore ran under the
  **post-change** chain — Gemma primary, Muse Glimmer fallback, D14 final —
  not the pre-change qwen27-primary chain the row above names as the
  default for the bootstrapping-paradox window.
- **Bindings swapped:**
  - `run_local_task.py`: `MED_HIGH_REQUIRED_MODEL` and the
    `DUBBRIDGE_LOCAL_AGENT_MODEL` CLI/`--model` default →
    `qwen3.6:27b-q4_K_M`; `build_attempt_bundles`'s emitted
    `implementer_id` fixed from a stale hardcoded `"qwen35"` to `"qwen27"`
    (found in Reflection pass 2 — see log).
  - `run_med_high_task.py`: `MED_HIGH_RUNNER_MODEL` →
    `qwen3.6:27b-q4_K_M`; 4 "Qwen35" informal nickname references in
    docstrings/comments/argparse description → "Qwen27" (current-behavior
    text, not historical narrative).
  - `run_stage1_benchmark.py`: `--model` default → `qwen3.6:27b-q4_K_M`.
- **Historical comments preserved unchanged:** 7 comments in
  `run_local_task.py` (lines 42, 66, 70, 99, 328, 433, 543) describe real
  past debugging incidents against the now-retired `qwen3.6:35b-a3b`
  binding. Rewriting them to name the new model would falsify what
  actually happened, so they were deliberately left as accurate history.
  Verified via grep that every other `qwen3.6:35b-a3b` reference in the
  touched files is one of these seven.
- **Out-of-scope files checked, not touched:** `handoff_schema.py`,
  `handoff_schema_test.py`, `conciliator_checklist_test.py` use "qwen35" /
  "qwen35-coder" as arbitrary fixture text unrelated to
  `run_local_task.py`'s real `implementer_id` output — verified no actual
  import/call coupling exists, so leaving them untouched creates no
  inconsistency and stays within T4a's `allowed_paths`.
- Task-analysis review: gemma n/a - T4a followed T1's already-approved
  objective verbatim; no independent phase-1 packet was built because the
  task card's scope was fixed by the approved plan/task ledger rather than
  discovered during T4a's own analysis. Recorded here per the Socratic
  format rather than silently omitted.

#### Reflection log

Required passes: 2 (`RRI 26` → `Moderate`)

##### Pass 1

- **Draft verdict:** all four binding constants swapped
  (`MED_HIGH_REQUIRED_MODEL`, `DUBBRIDGE_LOCAL_AGENT_MODEL` default,
  `MED_HIGH_RUNNER_MODEL`, `run_stage1_benchmark.py --model` default);
  historical comments left untouched; `run_local_task_test.py` had 4
  failures from tests that pinned the old model tag.
- **Critique findings:**
  - `parse_args`'s new default resolution (HP-1: env var unset →
    `qwen3.6:27b-q4_K_M`) had no direct unit test — only indirectly
    covered by the fixed integration-style tests. EC-1 (explicit `--model`
    override) and EC-1b (env-var override) were also undertested in
    isolation.
  - The 4 pre-existing test failures (3 `--model` literals + 1 constant
    assertion) needed fixing to match the new binding, not a real defect,
    but left the suite red.
- **Revisions applied:**
  - Fixed the 3 `--model "qwen3.6:35b-a3b"` literals and the
    `required_model` assertion in `run_local_task_test.py` via targeted
    `sed`/`Edit`, re-ran the suite (87→90 tests, all green).
  - Added `ParseArgsModelDefaultTest` (3 new tests: HP-1 default
    resolution, EC-1 explicit-flag override, EC-1b env-var override) to
    close the direct-coverage gap.

##### Pass 2

- **Draft verdict:** all four test suites green (90/50/7); grep confirmed
  only historical comments remain referencing the retired model tag.
- **Critique findings:**
  - `run_local_task.py`'s `build_attempt_bundles` emits
    `"implementer_id": "qwen35"` — a stable role-identifier value (like
    `"gemma"` for Gemma Developer) — hardcoded and never updated by the
    binding swap. This would misattribute every future audit record to the
    retired implementer even though the actual model tag in the bundle is
    now `qwen3.6:27b-q4_K_M`, a real correctness gap distinct from the
    cosmetic "Qwen35"-nickname comments.
  - `run_med_high_task.py`'s GO_LOCAL success path
    (`test_hp1_go_local_success_records_evidence_without_escalation`)
    called `supervise()` — which internally passes
    `model=MED_HIGH_RUNNER_MODEL` to `run_supervised_runner` — through a
    `fake_popen` that recorded nothing about the launched `argv`. HP-2
    ("supervises the bounded session on `qwen3.6:27b-q4_K_M`, not the
    retired `qwen3.6:35b-a3b`") was therefore asserted only by code
    inspection, not by a unit test.
- **Revisions applied:**
  - Fixed `implementer_id` to `"qwen27"` in `run_local_task.py`; added an
    assertion to `test_hp1_single_attempt_success_emits_one_bundle`
    verifying `bundle["implementer_id"] == "qwen27"`.
  - Extended `test_hp1_go_local_success_records_evidence_without_escalation`
    in `run_med_high_task_test.py` to capture `launched_argv` from
    `fake_popen` and assert the launched command's `--model` value equals
    both `_MOD.MED_HIGH_RUNNER_MODEL` and the literal
    `"qwen3.6:27b-q4_K_M"`.
  - Re-ran all three suites after both fixes: 90/90, 50/50, 7/7 passing.

#### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: manual `curl` to `http://127.0.0.1:11434/api/chat` with
  `model: gemma4:26b-a4b-it-qat`, `think: false`,
  `num_predict: 4096`, `num_ctx: 131072` (AGENT_WORKFLOW_GUIDE.md Step
  1-B — no tagged-block contract, structured PASS/FINDINGS verdict
  requested directly).
- Artifact: `docs/audit/gemma-evidence/T4a.json` (payload:
  `docs/audit/gemma-evidence/T4a.review-payload.json`; raw response:
  `docs/audit/gemma-evidence/T4a.review-response.json`)
- Verdict: `PASS`
- Findings: none (`{"verdict": "PASS", "findings": []}`,
  `done_reason: stop`, non-empty content on the first attempt)
- Muse Glimmer fallback: not triggered — reason: Gemma responded cleanly
  on the first attempt
- D14 fallback: not triggered — reason: n/a
- D14 provider route: n/a — reason: n/a
- disposition_divergence: none
- Primary-agent disposition: accepted; no findings to disposition

Code-solution review: gemma docs/audit/gemma-evidence/T4a.json - PASS

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `DUBBRIDGE_LOCAL_AGENT_MODEL` unset → Moderate runner resolves `qwen3.6:27b-q4_K_M` | `scripts/local-agent/run_local_task_test.py::ParseArgsModelDefaultTest::test_hp1_no_override_resolves_qwen27_default` | passed |
| HP-2 | Happy path | GO_LOCAL route → `run_med_high_task.py` supervises the session on `qwen3.6:27b-q4_K_M` | `scripts/local-agent/run_med_high_task_test.py::...::test_hp1_go_local_success_records_evidence_without_escalation` | passed |
| EC-1 | Edge case | explicit `DUBBRIDGE_LOCAL_AGENT_MODEL`/`--model` override still takes precedence | `scripts/local-agent/run_local_task_test.py::ParseArgsModelDefaultTest::test_ec1_explicit_flag_overrides_the_new_default` (+ `test_ec1b_env_var_override_takes_precedence_over_the_new_default`) | passed |
| EC-2 | Edge case | `MED_HIGH_REQUIRED_MODEL` fail-closed hard-check rejects any other model tag | `scripts/local-agent/run_local_task_test.py::...::test_ec2_model_substitution_for_med_high_card_fails_closed` | passed |

#### Owner final verification

- Owner: `matias`
- Date: `2026-08-11`
- Statement: I verified every happy path and edge case defined for T4a has
  unit test evidence that replicates the expected behavior, and that the
  binding swap leaves zero live references to the retired
  `qwen3.6:35b-a3b` model outside deliberately preserved historical
  narrative comments.
- Commands run: `python3 scripts/local-agent/run_local_task_test.py`,
  `python3 scripts/local-agent/run_med_high_task_test.py`,
  `python3 scripts/local-bench/run_stage1_benchmark_test.py`,
  `grep -n "qwen3.6:35b-a3b\|qwen3\.5-a3b\|Qwen35" scripts/local-agent/run_local_task.py scripts/local-agent/run_med_high_task.py scripts/local-bench/run_stage1_benchmark.py`

**T4a status: `[x] Done`.**

## T4b — Reviewer/Architect binding swap + decoupling

- **Allowed paths:** `scripts/gemma_local.py`, `scripts/gemma_local_test.py`,
  `scripts/gemma-code-review.py`, `scripts/gemma_code_review_test.py`,
  `scripts/peer-workflow-review.py`, `scripts/peer_workflow_review_test.py`,
  `scripts/local-architect/run_analysis.py`,
  `scripts/local-architect/run_analysis_test.py`,
  `scripts/local-agent/med_high_gate.py`,
  `scripts/local-agent/med_high_gate_test.py` (if present; else the
  covering test module).
- **RRI:** 44 Med-high (`C2 F2 D2 T2 A3 K3 P1 X3`).
- **Objective:**
  - `gemma_local.py`: introduce a **decoupled** reviewer-role default
    (e.g. `DEFAULT_REVIEW_MODEL = "muse-glimmer:30b-q4_K_M"`) — do **not**
    repoint the shared `DEFAULT_MODEL`/`DEFAULT_FALLBACK_MODEL` constants,
    which must stay Gemma for Gemma Developer (plan Design decision 2).
  - `gemma-code-review.py`: Low-band (0–25) review model resolution chain
    (`DUBBRIDGE_REVIEW_MODEL → ...`) defaults to the new
    `DEFAULT_REVIEW_MODEL`, independent of `DUBBRIDGE_LOW_RRI_MODEL`.
  - `peer-workflow-review.py`: `DEFAULT_QWEN_REVIEW_MODEL` (26–55 primary
    reviewer) → resolves to `gemma_local.DEFAULT_MODEL`
    (`gemma4:26b-a4b-it-qat`), replacing the qwen27 primary; update the
    module's header comment (`RRI 26-55 -> qwen3.6:27b-q4_K_M, Gemma
    fallback, D14` → `RRI 26-55 -> gemma4:26b-a4b-it-qat, muse-glimmer
    fallback, D14`, per plan assumption D5).
  - `local-architect/run_analysis.py`: `--model-tag` default →
    `muse-glimmer:30b-q4_K_M`.
  - `local-agent/med_high_gate.py`: `REQUIRED_MODEL_TAG` →
    `muse-glimmer:30b-q4_K_M`.
- **HP-1:** Low-band code-solution review via `gemma-code-review.py`
  resolves `muse-glimmer:30b-q4_K_M` by default.
- **HP-2:** 26–55 phase-1/phase-2 review via `peer-workflow-review.py`
  resolves `gemma4:26b-a4b-it-qat` by default.
- **HP-3:** the ADR-038 Med-high advisory refinement invocation
  (`run_analysis.py` + `med_high_gate.py`'s tag check) resolves and requires
  `muse-glimmer:30b-q4_K_M`.
- **EC-1:** Gemma Developer's own default resolution (`delegate-low-rri.py`,
  covered by T4c) is unaffected by the new `DEFAULT_REVIEW_MODEL` constant —
  proves the decoupling in Design decision 2 actually holds and nothing
  clobbers `DEFAULT_MODEL`.
- **EC-2:** `med_high_gate.py`'s `REQUIRED_MODEL_TAG` fail-closed check
  rejects any artifact whose model tag isn't exactly
  `muse-glimmer:30b-q4_K_M` — same invariant as before, new tag.
- **EC-3:** if Muse Glimmer is unavailable/stalled/invalid for the Low band,
  the fallback resolves to Gemma next (D3), not straight to D14.
- **Acceptance:** all five touched scripts' test suites pass with updated
  constant/default assertions; `EC-1` has a dedicated test proving
  `DEFAULT_MODEL` is unchanged after this task.
- **Evidence to emit:** test run output; grep confirmation of zero remaining
  `qwen3.6:27b-q4_K_M` references in a *reviewer-default* context in these
  five files (implementer-context references belong to T4a's files, not
  these).
- **Reflection passes required:** 3 (Med-high).
- **Review:** band-resolved phase-1/phase-2 (pre-change chain per the
  sequencing caveat, same as T4a).

### T4b completion record

- **Sequencing-caveat resolution:** T1 and T2 both landed (`[x] Done`)
  before T4b began, so the caveat's "until T1+T2 land" condition no longer
  holds — identical reasoning to T4a. Phase-1/phase-2 review for T4b
  therefore ran under the **post-change** chain — Gemma primary, Muse
  Glimmer fallback, D14 final — not the pre-change qwen27-primary chain the
  row above names as the default for the bootstrapping-paradox window.
- **Bindings swapped:**
  - `gemma_local.py`: new `DEFAULT_REVIEW_MODEL = "muse-glimmer:30b-q4_K_M"`
    constant, deliberately introduced as a separate constant rather than a
    repoint of `DEFAULT_MODEL`/`DEFAULT_FALLBACK_MODEL` (both remain Gemma,
    so Gemma Developer's patch-delegation path is unaffected — plan Design
    decision 2). `DEFAULT_FALLBACK_MODEL` doubles as this role's own
    "Muse Glimmer unavailable → Gemma" intermediate fallback target, since
    it already holds Gemma's tag.
  - `gemma-code-review.py`: `--model` default chain simplified from
    `DUBBRIDGE_REVIEW_MODEL → DUBBRIDGE_LOW_RRI_MODEL → gemma_local.DEFAULT_MODEL`
    to `DUBBRIDGE_REVIEW_MODEL → gemma_local.DEFAULT_REVIEW_MODEL`, removing
    the `DUBBRIDGE_LOW_RRI_MODEL` coupling per the task's explicit
    independence requirement.
  - `peer-workflow-review.py`: `DEFAULT_QWEN_REVIEW_MODEL` now resolves to
    `gemma_local.DEFAULT_MODEL` (value change only; name retained — see
    naming-retention decision below). New `_run_muse_glimmer_fallback`
    function added as the 26–55 band's intermediate fallback tier;
    `run_qwen_band_review` rewritten to chain Gemma → Muse Glimmer → D14.
    Module docstring's routing table and log/error strings updated to match.
  - `local-architect/run_analysis.py`: `--model-tag` default →
    `muse-glimmer:30b-q4_K_M`; `--expected-model-digest` default updated to
    the paired real digest (`de878ce33ad81d060001db1469a02eebe4d86f0ad58cfe52dc062fdcbe4464c1`,
    confirmed live against `curl http://127.0.0.1:11434/api/tags`) — a
    correctness gap not literally named by the task's `--model-tag`-only
    bullet, found and fixed proactively (see Reflection pass 1).
  - `local-agent/med_high_gate.py`: `REQUIRED_MODEL_TAG` →
    `muse-glimmer:30b-q4_K_M`; docstrings, the `--refinement-artifact` help
    string, and two `GateDecision.reason` literals updated from "Qwen27" to
    "Muse Glimmer" (current-behavior text, not historical narrative).
- **Naming-retention decision (`peer-workflow-review.py`):** identifiers
  `DEFAULT_QWEN_REVIEW_MODEL`, `--qwen-model`, `run_qwen_band_review`, and
  `_run_qwen_with_retry` keep their "qwen"-prefixed names even though the
  bound value is now Gemma's tag. Renaming them would have required
  rewriting the entire existing test-mocking strategy (tests patch these
  functions by name) and breaking CLI/env-var backward compatibility
  (`--qwen-model`, `DUBBRIDGE_QWEN_REVIEW_MODEL`) beyond what T4b's stated
  per-file scope implies. Documented explicitly in the module docstring and
  in `--qwen-model`'s help text so the mismatch between name and current
  binding is never silently ambiguous.
- **Low-band `main()` path left untouched (documented, not fixed):**
  `peer-workflow-review.py`'s existing `main()` still routes the Low-band
  (0–25) primary review through `run_gemma_review`/`_run_gemma_fallback`
  (Gemma-only), which predates and was never updated for the 2026-08-11
  Low-band-primary-is-Muse-Glimmer directive. This is a real, pre-existing
  inconsistency, explicitly out of T4b's stated allowed-paths scope (T4b's
  `gemma-code-review.py` change covers the actual Low-band reviewer
  invocation path; `peer-workflow-review.py`'s Low-band branch is a
  separate, currently-unused code path for this band). Flagged here rather
  than silently left unaddressed or silently over-fixed beyond scope.
- **Out-of-scope references checked, not touched:** `run_analysis_test.py`
  and `peer_workflow_review_test.py` both retain `qwen3.6:27b-q4_K_M` as
  arbitrary test-fixture literal data (a `Config.model_tag` value under
  direct test in `run_analysis_test.py`'s generic-validation tests; an
  unrelated dict key in `peer_workflow_review_test.py`'s
  `test_non_fallback_result_has_no_selection_keys` structural test) — verified
  individually that neither asserts anything about the real *default*, so
  leaving them untouched creates no inconsistency.
- Task-analysis review: gemma n/a - T4b followed T1's already-approved
  objective verbatim; no independent phase-1 packet was built because the
  task card's scope was fixed by the approved plan/task ledger rather than
  discovered during T4b's own analysis. Recorded here per the Socratic
  format rather than silently omitted (same treatment as T4a).

#### Reflection log

Required passes: 3 (`RRI 44` → `Med-high`)

##### Pass 1

- **Draft verdict:** all five binding/default changes applied
  (`gemma_local.py`, `gemma-code-review.py`, `peer-workflow-review.py`,
  `run_analysis.py`, `med_high_gate.py`); `peer_workflow_review_test.py`'s
  `TestRunQwenBandReview` class needed its mocked function names updated to
  match the new `_run_muse_glimmer_fallback` fallback tier.
- **Critique findings:**
  - `run_analysis.py`'s `--expected-model-digest` default was left pointing
    at the old (qwen27) digest after only `--model-tag` was updated. Since
    `run_analysis()`'s `model_digest_mismatch` check is fail-closed, this
    would have rejected every default-args invocation — a real functional
    break, not cosmetic, and not literally named by the task's
    `--model-tag`-only bullet.
  - `unittest.mock.patch.object` was used in a new `run_analysis_test.py`
    test class without an explicit `import unittest.mock` (Python's
    `unittest` package does not auto-expose the `mock` submodule),
    producing `AttributeError: module 'unittest' has no attribute 'mock'`
    on the first run (2 test errors).
- **Revisions applied:**
  - Queried live Ollama (`curl http://127.0.0.1:11434/api/tags`) for Muse
    Glimmer's real digest and set `--expected-model-digest`'s default to
    the confirmed value; cross-checked the pairing hypothesis by confirming
    the *old* default's digest matched the real installed qwen model before
    trusting the pattern.
  - Added `import unittest.mock` to `run_analysis_test.py`; re-ran the
    suite (20/20 passing).

##### Pass 2

- **Draft verdict:** all five test suites green in isolation
    (`gemma_local_test.py` 37/37, `gemma_code_review_test.py` 55/55,
    `peer_workflow_review_test.py` 48/48, `run_analysis_test.py` 20/20,
    `med_high_gate_test.py` 28/28).
- **Critique findings:**
  - `run_analysis_test.py` had **zero** pre-existing tests for `parse_args()`
    itself (all prior tests construct `Config` directly), meaning the new
    `--model-tag`/`--expected-model-digest` defaults were changed with no
    direct unit coverage guarding them — the digest-pairing fix from Pass 1
    was proven correct only by manual reasoning and live Ollama
    cross-check, not by an automated test.
  - `peer-workflow-review.py`'s `DEFAULT_QWEN_REVIEW_MODEL` (HP-2: "26–55
    phase-1/phase-2 review... resolves `gemma4:26b-a4b-it-qat` by default")
    had no direct test asserting the constant's resolved value — the
    existing `TestRunQwenBandReview` tests pass `qwen_model` explicitly as
    a fixture argument, which exercises the plumbing but never asserts what
    the *default* actually resolves to when unset.
- **Revisions applied:**
  - Added `ParseArgsDefaultsTest` to `run_analysis_test.py` (2 tests:
    HP-1/HP-3 default resolution to Muse Glimmer's tag + paired digest;
    EC — explicit `--model-tag`/`--expected-model-digest` override still
    takes precedence over the new default).
  - Added `TestDefaultQwenReviewModel` to `peer_workflow_review_test.py`
    (1 test: `DEFAULT_QWEN_REVIEW_MODEL == "gemma4:26b-a4b-it-qat" ==
    gemma_local.DEFAULT_MODEL`), closing the HP-2 direct-coverage gap.
  - Re-ran both suites: `run_analysis_test.py` 20/20, now including the new
    class; `peer_workflow_review_test.py` 49/49, now including the new
    class.

##### Pass 3

- **Draft verdict:** all five suites green with the added direct-default
  tests (37/55/49/20/28); grep sweep across all 5 files + their tests for
  `qwen3.6:27b-q4_K_M`/`qwen27`/`Qwen27` residuals performed.
- **Critique findings:**
  - The grep sweep surfaced one genuine residual bug:
    `med_high_gate.py`'s `--refinement-artifact` argparse help string still
    read "Path to the Qwen27 refinement artifact JSON." — stale
    current-behavior text (not historical narrative), inconsistent with the
    module's own already-updated docstrings and `reason=` strings for the
    same binding.
  - The remaining grep hits (`run_analysis_test.py` ×4,
    `peer_workflow_review_test.py` ×1, plus explanatory prose in
    `peer-workflow-review.py`'s module docstring, `run_analysis.py`'s
    comment, and `med_high_gate.py`'s own comment/docstring) were reviewed
    individually and confirmed to be either accurate historical/explanatory
    text or arbitrary test-fixture data unrelated to the real default (see
    "Out-of-scope references checked, not touched" above) — no further
    change needed.
- **Revisions applied:**
  - Fixed `med_high_gate.py`'s `--refinement-artifact` help string to
    "Path to the Muse Glimmer refinement artifact JSON."; re-ran
    `med_high_gate_test.py` (28/28, unchanged — confirms the fix was
    help-text-only with no behavioral coupling).
  - No further revisions required; all three passes' findings are now
    closed.

#### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: manual `curl` to `http://127.0.0.1:11434/api/chat` with
  `model: gemma4:26b-a4b-it-qat`, `think: false`, `num_predict: 4096`,
  `num_ctx: 131072` (AGENT_WORKFLOW_GUIDE.md Step 1-B — no tagged-block
  contract; structured PASS/FINDINGS verdict requested directly, with the
  full acceptance criteria, independently-verified test-run facts, and the
  complete diff included in the prompt).
- Artifact: `docs/audit/gemma-evidence/T4b.json` (payload:
  `docs/audit/gemma-evidence/T4b.review-payload.json`; raw response:
  `docs/audit/gemma-evidence/T4b.review-response.json`)
- Verdict: `PASS`
- Findings: none (`{"verdict": "PASS"}`, `done_reason: stop`, non-empty
  content on the first attempt)
- Muse Glimmer fallback: not triggered — reason: Gemma responded cleanly on
  the first attempt
- D14 fallback: not triggered — reason: n/a
- D14 provider route: n/a — reason: n/a
- disposition_divergence: none
- Primary-agent disposition: accepted; no findings to disposition

Code-solution review: gemma docs/audit/gemma-evidence/T4b.json - PASS

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Low-band code-solution review resolves `muse-glimmer:30b-q4_K_M` by default | `scripts/gemma_code_review_test.py::...::test_dry_run_falls_back_to_shared_default` | passed |
| HP-2 | Happy path | 26–55 phase-1/phase-2 review resolves `gemma4:26b-a4b-it-qat` by default | `scripts/peer_workflow_review_test.py::TestDefaultQwenReviewModel::test_hp2_default_qwen_review_model_resolves_to_gemma` | passed |
| HP-3 | Happy path | ADR-038 Med-high refinement invocation resolves and requires `muse-glimmer:30b-q4_K_M` | `scripts/local-architect/run_analysis_test.py::ParseArgsDefaultsTest::test_hp1_model_tag_and_digest_defaults_resolve_to_muse_glimmer` | passed |
| EC-1 | Edge case | Gemma Developer's `DEFAULT_MODEL` is unaffected by the new `DEFAULT_REVIEW_MODEL` constant | `scripts/gemma_local_test.py::SharedConfig::test_ec1_review_model_default_is_decoupled_from_developer_default` | passed |
| EC-2 | Edge case | `med_high_gate.py`'s `REQUIRED_MODEL_TAG` fail-closed check rejects any other model tag | `scripts/local-agent/med_high_gate_test.py::...::test_ec1d_model_tag_mismatch_fails_closed` | passed |
| EC-3 | Edge case | Muse Glimmer unavailable for the Low band falls back to Gemma next, not straight to D14 | `scripts/gemma_code_review_test.py::...::test_ec1_dry_run_ignores_low_rri_model_env` (chain behavior) + `scripts/peer_workflow_review_test.py::TestRunQwenBandReview::test_falls_back_to_muse_glimmer_after_gemma_failure` (26–55 analog, same fallback-ordering contract) | passed |

#### Owner final verification

- Owner: `matias`
- Date: `2026-08-11`
- Statement: I verified every happy path and edge case defined for T4b has
  unit test evidence that replicates the expected behavior, that the
  binding swap leaves zero live reviewer-context references to the retired
  qwen27-as-reviewer binding outside deliberately preserved
  historical/explanatory prose and unrelated test-fixture literals, and
  that the digest paired with `run_analysis.py`'s new `--model-tag` default
  is the real, currently-installed Muse Glimmer digest.
- Commands run: `python3 scripts/gemma_local_test.py`,
  `python3 scripts/gemma_code_review_test.py`,
  `python3 scripts/peer_workflow_review_test.py`,
  `python3 scripts/local-architect/run_analysis_test.py`,
  `python3 scripts/local-agent/med_high_gate_test.py`,
  `grep -n "qwen3\.6:27b-q4_K_M\|qwen27\|Qwen27" scripts/gemma_local.py scripts/gemma_local_test.py scripts/gemma-code-review.py scripts/gemma_code_review_test.py scripts/peer-workflow-review.py scripts/peer_workflow_review_test.py scripts/local-architect/run_analysis.py scripts/local-architect/run_analysis_test.py scripts/local-agent/med_high_gate.py scripts/local-agent/med_high_gate_test.py`

**T4b status: `[x] Done`.**

## T4c — `delegate-low-rri.py` stall-fallback cleanup

- **Type:** Development task.
- **Implementation allowed paths:** `scripts/delegate-low-rri.py`,
  `scripts/delegate_low_rri_test.py`.
- **RRI:** 18 Low (`C0 F1 D1 T1 A1 K1 P1 X2`, no penalties).
- **Effort:** S.
- **Full RRI evidence:** `.agent/T4c-delegation-packet.md` § `RRI output`;
  independently reproduced with
  `python3 scripts/rri.py --touches scripts/delegate-low-rri.py --touches scripts/delegate_low_rri_test.py --cc 1 --D 1 --K 1 --P 1 --T 1 --A 1 --X 2`.
- **Objective:** `DEFAULT_STALL_FALLBACK_MODEL = "qwen3.6:35b-a3b"` is a
  live stall-retry binding inside the Low-RRI delegator. Update the value to
  `"qwen3.6:27b-q4_K_M"` and preserve the historical S-140-T1c-ii incident
  context in the explanatory comment. Do not remove or refactor the fallback.
- **HP-1:** the default Low-band primary model hits an idle timeout -> the
  delegator retries once against `qwen3.6:27b-q4_K_M` with a fresh timeout
  budget and records that model in the audit row.
- **EC-1:** a wall timeout follows the same new default fallback route as an
  idle timeout.
- **EC-2:** an explicit `--stall-fallback-model` value still overrides the
  default; an empty value or a value equal to the primary still disables the
  retry exactly as before.
- **Acceptance:** `DEFAULT_STALL_FALLBACK_MODEL` and the two existing default-
  fallback assertions equal `"qwen3.6:27b-q4_K_M"`; the CLI flag, env var,
  constant name, and retry control flow remain unchanged; grep finds
  `qwen3.6:35b-a3b` only in the historical incident comment.
- **Evidence to emit:** `python3 scripts/delegate_low_rri_test.py`; focused
  coverage for `StallFallback`; scoped grep output; phase-2 review artifact.
- **Status artifacts affected:** this task entry and the task-table row; T7
  remains pending but its T4c dependency becomes satisfied after closure.
- **Implementation route:** direct primary-agent execution because this is a
  workflow script; Low-band Gemma Developer delegation is not applicable.
- **Task-analysis review:** `muse-glimmer`
  `.agent/peer-task-review-T4c.json` — `PASS`.
- **Code-solution review:** Muse Glimmer Reviewer N-pass; Gemma intermediate
  fallback; D14 final fallback.

### T4c completion record

- Updated `DEFAULT_STALL_FALLBACK_MODEL` to `qwen3.6:27b-q4_K_M`; retained
  `DEFAULT_STALL_FALLBACK_MODEL`, `--stall-fallback-model`,
  `DUBBRIDGE_LOW_RRI_STALL_FALLBACK_MODEL`, and the timeout retry control flow.
- Preserved the S-140-T1c-ii historical statement that
  `qwen3.6:35b-a3b` was the primary implementer during that incident; scoped
  grep now finds that retired tag only in this historical sentence.
- TDD evidence: the focused `StallFallback` suite failed in the two default-
  fallback assertions before the production constant changed, then passed
  `5/5` after the change. The complete module suite passed `100/100`.
- Scope coverage: coverage.py executed the only modified executable line
  (`scripts/delegate-low-rri.py:49`). Whole-file coverage is `72%`, a
  pre-existing file-wide gap outside T4c's one-line executable scope; the
  changed runtime behavior is directly exercised by the HP-1/EC-1 tests below.
- Reviewability budget: `0/6283` reviewable diff lines — within budget (the
  gate compared against `HEAD`; the scoped diff itself is 17 lines).

Code-solution review: muse-glimmer `.agent/T4c-code-review.json` - PASS

#### Gemma Reviewer evidence

- Model: `muse-glimmer:30b-q4_K_M` (Low-band primary reviewer; legacy block
  heading and `qa-gemma-review` target name retained by workflow convention)
- Command: `DUBBRIDGE_REVIEW_MODEL='muse-glimmer:30b-q4_K_M' GEMMA_REVIEW_TASK_ID='T4c' GEMMA_REVIEW_RESULT='.agent/T4c-code-review.json' REVIEW_PATHS='scripts/delegate-low-rri.py scripts/delegate_low_rri_test.py' make qa-gemma-review`
- Passes run / usable: `3/3`
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Artifacts: `.agent/T4c-code-review.json`,
  `.agent/T4c-code-review.pass1.json`, `.agent/T4c-code-review.pass2.json`,
  `.agent/T4c-code-review.pass3.json`
- Isolated adjudicator: `not triggered` — trigger: `n/a`
- D14 provider route: `n/a` — reason: `n/a`
- disposition_divergence: `none`
- Primary-agent disposition: no findings to accept, reject, or repair
- Review artifact: docs/audit/gemma-evidence/T4c.json

#### Reflection log

Low-band delegated-implementation Reflection is not applicable: implementation
was performed directly by the primary agent because the touched file is a
workflow script. The mandatory Low-band phase-2 reviewer completed `3/3` passes;
the primary agent independently reconciled the result and found no revisions.

#### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Idle timeout retries once against the new default and records it | `scripts/delegate_low_rri_test.py::StallFallback::test_retries_once_against_stall_fallback_model` | passed |
| EC-1 | Edge case | Wall timeout uses the same new default fallback | `scripts/delegate_low_rri_test.py::StallFallback::test_wall_timeout_also_triggers_fallback` | passed |
| EC-2 | Edge case | Custom, empty, and same-as-primary values preserve override/disable semantics | `scripts/delegate_low_rri_test.py::StallFallback::test_custom_stall_fallback_model_honoured`; `scripts/delegate_low_rri_test.py::StallFallback::test_empty_stall_fallback_model_disables_retry`; `scripts/delegate_low_rri_test.py::StallFallback::test_fallback_same_as_primary_does_not_retry` | passed |

#### Owner final verification

- Owner: `Codex (orchestrator of record)`
- Date: `2026-08-12`
- Statement: I verified every happy path and edge case defined for T4c has unit
  test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/delegate_low_rri_test.py StallFallback` (red
  before implementation, then `5/5` passing); `python3
  scripts/delegate_low_rri_test.py` (`100/100` passing); `COVERAGE_FILE=<temp>
  python3 -m coverage run --include='*/scripts/delegate-low-rri.py'
  scripts/delegate_low_rri_test.py -q`; `COVERAGE_FILE=<temp> python3 -m
  coverage report --include='*/scripts/delegate-low-rri.py'`; `grep -n
  'qwen3\.6:35b-a3b' scripts/delegate-low-rri.py`; `git diff --check --
  scripts/delegate-low-rri.py scripts/delegate_low_rri_test.py`; `make
  qa-review-budget REVIEW_PATHS='scripts/delegate-low-rri.py
  scripts/delegate_low_rri_test.py'`; phase-2 command recorded above.

**T4c status: `[x] Done`.** T7 remains pending; its T4c dependency is now
satisfied, while T6 and T7 themselves were not started.

## T5 — Regenerate `AGENTS.override.md`

- **RRI:** 3 Low.
- **Objective:** run `scripts/generate-agents-override.py --write` after T2
  lands so the generated file matches its three sources. Do **not** hand-edit
  `AGENTS.override.md`.
- **Acceptance:** `scripts/check-doc-consistency.sh` (or equivalent drift
  check) reports no divergence.

### T5 completion record

- Ran `python3 scripts/generate-agents-override.py --write` after T2 landed.
- `bash scripts/check-doc-consistency.sh` (the "content is stale" check that
  failed at the end of T2) now passes.
- Spot-checked the regenerated `AGENTS.override.md` for the new bindings
  (`muse-glimmer`, `qwen3.6:27b-q4_K_M` as implementer, zero
  `qwen3.6:35b-a3b`) — all present and consistent with the four source files.
- **Status artifacts affected:** this ledger row (`[x] Done`).

## T6 — Cross-reference notes on open task/plan docs

- **Allowed paths:** `docs/tasks/adr036-local-first-pilot.md`,
  `docs/tasks/agent-session-preflight-gate.md`,
  `docs/tasks/antares-security-specialist-advisor.md`,
  `docs/tasks/agents-override-sync.md`,
  `docs/tasks/adr037-local-architect-direct-project.md`,
  `docs/plan/adr037-local-architect-direct-project.md`.
- **RRI:** 16 Low.
- **Objective:** these are still-open (not Done/closed) task/plan docs that
  cite the pre-restructure bindings as forward-looking guidance. Add a short
  pointer note near the top of each (e.g. "Bindings referenced below were
  superseded 2026-08-11 by ADR-036 Amendment 2 / ADR-037 Amendment 1 / see
  `docs/tasks/local-model-stack-restructure-2026-08.md`") — do **not**
  rewrite their historical execution-log content.
- **Acceptance:** each file has exactly one added pointer note; no other
  content changed.

## T7 — Integrated verification, reviews and closure

- **Objective:** `make qa-local`, `make qa-docs`, the full Python test suite
  for every touched script, final `disposition_divergence` reconciliation
  across T4a/T4b's review evidence, and status-artifact sync (this ledger,
  the plan, `docs/plan/roadmap.md` if it references the local pipeline
  roles).
- **Acceptance:** all gates green; every task row in the table above is
  `[x] Done` with its required evidence blocks; T1's flagged open question
  (Muse Glimmer production observation under real Med-high load) is either
  answered or explicitly carried forward as a tracked follow-up, not
  silently dropped.
