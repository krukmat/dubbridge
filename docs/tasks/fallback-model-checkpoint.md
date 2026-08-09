---
type: TaskList
title: "Tasks: Human-selected fallback model checkpoint"
plan: docs/plan/fallback-model-checkpoint.md
status: done
slice: FMC
rri: 89
band: Very high
effort: XL
---

# Tasks: Human-selected Fallback Model Checkpoint

## Authorization and aggregate evidence

- **Owner authorization:** `autorizado` in the active session on 2026-08-09,
  covering the previously summarized checkpoint, receipt, D14/cloud separation,
  model matrix, scripts, tests, and documentation. This is a bounded approval;
  commit and push remain out of scope.
- **Aggregate RRI:** 89 (Very high), direct implementation prohibited.
- **Aggregate penalties:** `arch_decision` (+12), `many_files` (+8).
- **Owner implementation-route directive (2026-08-09):** FMC-4 and every
  remaining development task in this plan are cloud-primary until the plan is
  complete. This is a routing override only: runtime checkpoint behavior and all
  review, HITL, and evidence gates remain unchanged.
- **Decomposition:** FMC-1 through FMC-5 are development tasks at RRI 44-50;
  FMC-6a/FMC-6b are documentation/policy propagation tasks.
- **Evidence to emit:** `.agent/fallback-selection-*.json`, focused unit-test
  output, phase-1/phase-2 review artifacts, Reflection logs, and this ledger.
- **Status artifacts affected:** this ledger, linked plan, ADR-039 and index,
  workflow/HITL/RRI policies, task-card template, agent summaries.

## Ordered task list

| ID | Task | RRI / Effort | Status | Depends on |
|---|---|---|---|---|
| FMC-1 | Implement shared checkpoint and receipt contract | 44 / L | Done | ADR-039 |
| FMC-2 | Gate D14 reviewer fallback in phases 1 and 2 | 46 / L | Done | FMC-1 |
| FMC-3 | Gate Low-band Gemma-to-cloud implementation fallback | 49 / L | Done | FMC-1 |
| FMC-4 | Gate Moderate local-runner cloud fallback | 44 / L | Done | FMC-1 |
| FMC-5 | Gate Med-high `CLOUD_REQUIRED` and failed-attempt fallback | 50 / L | Done | FMC-1 |
| FMC-6a | Synchronize authoritative workflow/policy/template docs | docs/policy-only | Done | FMC-2..FMC-5 |
| FMC-6b | Synchronize agent summaries and generated override | docs/config-only | Done | FMC-6a |

## FMC-1 — Shared checkpoint and receipt contract

**Status:** [x] Done
**Effort:** L. **RRI:** 44 (Med-high). **Allowed paths:**
`scripts/fallback_selection.py`, `scripts/fallback_selection_test.py`.

**Objective:** Provide a reusable pure-Python contract that derives a recommendation,
hashes the fallback packet, validates human/preauthorized selection, and emits an
awaiting checkpoint or authorized receipt without invoking any model.

**Acceptance criteria:**

- `human-select` with no selection returns `awaiting_fallback_selection`.
- A complete selection returns `fallback_authorized` with packet and receipt SHA-256.
- `preauthorized` with any missing selection field fails closed.
- A changed packet cannot validate against the earlier receipt.
- For `role=d14`, the pure recommendation resolver returns
  `gpt-5.6-terra/medium` as the Balanced default; a human may select a different
  model, but downstream policy still enforces D14's Balanced capability floor.
- For `role=cloud-implementer`, the resolver returns the following task-frozen
  matrix (copied from `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` as reviewed on
  2026-08-09; the implementation must not read mutable policy text at runtime):
  Low -> `gpt-5.6-luna/low`; Moderate ->
  `gpt-5.6-terra/medium`; Med-high operational-only ->
  `gpt-5.6-terra/high`; Med-high capability/risk -> `gpt-5.6-sol/high`.
- Unit tests must assert every matrix row above plus the D14 recommendation.

- **HP-1:** `human-select` + explicit model/effort/selector -> authorized receipt
  bound to the exact packet.
- **HP-2:** complete `preauthorized` selection -> no interactive pause, same receipt
  contract.
- **EC-1:** missing model, effort, or selector -> awaiting/blocked; never authorized.
- **EC-2:** packet bytes change after selection -> receipt validation fails.
- **EC-3:** unsupported role, mode, reasoning effort, or incomplete
  `preauthorized` selection -> raise `FallbackSelectionError` (a `ValueError`
  subclass) with a stable field-specific message; never warn-and-continue.
- **EC-4:** packet input that is neither bytes, UTF-8 text, nor a JSON-serializable
  object -> deterministic validation error before hashing.
- **EC-5:** canonical structured inputs `{"a": 1, "b": 2}` and
  `{"b": 2, "a": 1}` -> identical packet SHA-256 despite insertion order.

**Implementation constraint:** Python standard-library primitives, including
`hashlib`, `json`, and `datetime`, are permitted; no third-party dependency or
hand-written cryptographic primitive may be added. Recommendation, hashing,
creation, and validation functions perform no network or subprocess calls. Only
the explicit artifact-writer helper may mutate the filesystem.

**Handoff prompt:** `FMC-1 — Implement only the shared fallback checkpoint contract
and its unit tests. Stop after focused tests pass; do not integrate callers.`

Task-analysis review: qwen3.6:27b-q4_K_M .agent/peer-task-review-fmc-1-r3.json - PASS
Code-solution review: d14 .agent/peer-code-review-fmc-1.json - PASS

### Implementation and review evidence

- Implementation route: `CLOUD_REQUIRED`; Codex authored the shared helper after
  the ADR-038 Qwen27 refinement attempt stalled and emitted
  `.agent/fmc-1-refinement.json`. The hash-bound primary route receipt is
  `.agent/fmc-1-primary-route-receipt.json`.
- Antares refinement/post-implementation: typed skip — FMC-1 carried no
  task-relevant T3a-watchlisted CWE hypothesis; a generic security sweep was not
  justified.
- Reviewer: `d14` using `gpt-5.6-terra` with `medium` reasoning.
- Command: context-isolated D14 review of the exact diff and FMC-1 acceptance
  criteria, followed by a same-reviewer re-review after repair.
- Artifact: `.agent/peer-code-review-fmc-1.json`.
- Verdict: `PASS`.
- Findings: the initial review found one missing EC-4 test for non-UTF-8-encodable
  text; the finding was accepted and repaired before the final PASS.
- Gemma fallback: not run — reason: the owner explicitly selected direct cloud
  fallback after the primary Qwen27 reviewer stalled.
- D14 fallback: triggered — reason: Qwen27 produced no usable verdict after 494
  seconds; the owner selected D14 through the frozen fallback receipt at
  `.agent/fmc-1-phase2-fallback-selection.json`.
- disposition_divergence: `none`.
- Primary-agent disposition: accepted and repaired the sole finding, then reran
  focused tests, coverage, compilation, diff validation, and D14 review.
- Review artifact: docs/audit/gemma-evidence/FMC-1.json

### Reflection log

Required passes: 3 (`44` -> `Med-high`)

#### Pass 1

- **Draft verdict:** The pure recommendation, packet hashing, checkpoint, receipt,
  validation, atomic writer, and CLI/env adapter contracts were implemented with
  focused unit tests.
- **Critique findings:** The first deterministic missing-field assertion expected
  the wrong stable field order.
- **Revisions applied:** Aligned the test with the contract's deterministic
  validation order and reran the focused suite.

#### Pass 2

- **Draft verdict:** Happy paths and core fail-closed behavior passed, but focused
  line coverage was 81% and receipt validation did not yet reject every top-level
  versus receipt mismatch.
- **Critique findings:** Missing negative coverage for malformed checkpoint fields,
  tampered receipts, and incomplete authorized artifacts left validation branches
  insufficiently exercised.
- **Revisions applied:** Tightened field/receipt consistency checks and added
  negative tests, raising focused line coverage above the 90% gate.

#### Pass 3

- **Draft verdict:** The helper was side-effect-free outside the explicit atomic
  writer, callers could consume a common CLI/env adapter, and stale packets failed
  validation.
- **Critique findings:** D14 identified one remaining EC-4 gap: an unpaired Unicode
  surrogate did not have a direct test proving deterministic rejection before
  hashing.
- **Revisions applied:** Added
  `PacketHashTest.test_non_utf8_encodable_text_fails`; the revised suite passed 20
  tests at 93% focused line coverage, and D14 returned PASS on re-review.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Human selection creates an authorized receipt bound to the exact packet | `scripts/fallback_selection_test.py::CheckpointTest::test_complete_human_selection_is_authorized_and_validates` | passed |
| HP-2 | Happy path | Complete preauthorization creates the same authorized receipt without an interactive pause | `scripts/fallback_selection_test.py::CheckpointTest::test_complete_preauthorization_is_authorized` | passed |
| EC-1 | Edge case | Missing selection fields await or fail closed and never authorize | `scripts/fallback_selection_test.py::CheckpointTest::test_human_select_without_selection_waits`; `scripts/fallback_selection_test.py::CheckpointTest::test_incomplete_preauthorization_fails_closed`; `scripts/fallback_selection_test.py::CheckpointTest::test_partial_human_selection_fails_closed` | passed |
| EC-2 | Edge case | Changing the packet invalidates its earlier receipt | `scripts/fallback_selection_test.py::CheckpointTest::test_changed_packet_invalidates_receipt` | passed |
| EC-3 | Edge case | Unsupported fields and invalid recommendation inputs raise stable errors | `scripts/fallback_selection_test.py::CheckpointTest::test_unsupported_values_raise_stable_errors`; `scripts/fallback_selection_test.py::RecommendationTest::test_invalid_trigger_and_rri_fail` | passed |
| EC-4 | Edge case | Non-serializable objects and non-UTF-8 text fail deterministically before hashing | `scripts/fallback_selection_test.py::PacketHashTest::test_non_serializable_packet_fails`; `scripts/fallback_selection_test.py::PacketHashTest::test_non_utf8_encodable_text_fails` | passed |
| EC-5 | Edge case | Equivalent mappings hash identically regardless of insertion order | `scripts/fallback_selection_test.py::PacketHashTest::test_mapping_order_does_not_change_hash` | passed |

### Owner final verification

- Owner: `Codex agent`
- Date: 2026-08-09
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/fallback_selection_test.py`;
  `cd scripts && COVERAGE_FILE=/tmp/dubbridge-fmc1.coverage python3 -m coverage run --source=fallback_selection fallback_selection_test.py`;
  `cd scripts && COVERAGE_FILE=/tmp/dubbridge-fmc1.coverage python3 -m coverage report -m fallback_selection.py`;
  `python3 -m py_compile scripts/fallback_selection.py scripts/fallback_selection_test.py`;
  `git diff --check -- scripts/fallback_selection.py scripts/fallback_selection_test.py docs/tasks/fallback-model-checkpoint.md docs/audit/gemma-evidence/FMC-1.json`.

## FMC-2 — D14 reviewer checkpoint

**Status:** [x] Done
**Effort:** L. **RRI:** 46 (Med-high). **Allowed paths:**
`scripts/peer-workflow-review.py`, `scripts/peer_workflow_review_test.py`.

**Approval:** Covered by the bounded owner authorization recorded in this ledger;
no additional approval is inferred for scope, commit, or push changes.

**Resolved implementation route:** `CLOUD_REQUIRED` (Codex). The 656-line
`scripts/peer-workflow-review.py` exceeds the 500-line local-delegation read gate,
and this task changes the governance-critical authorization boundary before D14.
Splitting out a behavior-neutral refactor would enlarge the approved scope without
isolating the checkpoint's atomic reviewer-control-flow change.

**Objective:** Route Low, RRI 26-55, and RRI 56+ unusable reviewer chains to the
shared selection checkpoint before external D14 execution.

- **HP-1:** at any D14 trigger — Low after Gemma is unusable, RRI 26-55 after
  Qwen27 and Gemma are unusable, or RRI 56+ after the cross-vendor peer is
  unusable — complete preauthorization -> review artifact includes an authorized
  D14 receipt and the wrapper signals that the orchestrator may spawn that exact
  model. The review artifact retains `reviewer`, `phase`, `summary`, `findings`,
  and `d14_packet`, adds `fallback_selection_artifact`, and embeds the complete
  `fallback_selection-v1` checkpoint under `fallback_selection`. An authorized
  result uses `verdict=d14_required`; the selected model/effort/selector and
  `authorization_receipt` are read from the embedded checkpoint. Tests assert
  these exact keys, the authorized receipt, and the existing D14-required exit
  code for both phases and representative Low, RRI 26-55, and RRI 56+ routes.
- **EC-1:** at the same D14 triggers, no selection -> artifact says
  `awaiting_fallback_selection`, uses dedicated pause exit code 3, and does not
  say to spawn D14. Its embedded checkpoint has no `authorization_receipt`.
  Tests assert the selection artifact and embedded checkpoint are present, the
  receipt is absent, and exit code 3 is returned for both phases and every RRI
  band.
- **EC-2:** Low Gemma unusable -> same D14 checkpoint instead of the current
  generic blocked artifact (`reviewer=gemma`, `verdict=blocked`, `blocked=true`,
  exit 2). That legacy tuple is the behavior being replaced, not the expected
  result: Low follows HP-1 when preauthorized, EC-1 when awaiting selection, and
  EC-3 only when integrity validation fails. Tests assert all three Low outcomes.
- **EC-3:** a malformed D14 relay packet, or a preauthorization whose checkpoint
  or receipt hash does not match the exact relayed `d14_packet`, fails closed:
  write a review artifact with `verdict=blocked`, `blocked=true`, and an integrity
  error summary; return exit code 2; do not emit a spawn instruction. Tests cover
  malformed packet input and checkpoint/receipt hash mismatch.

**Acceptance criteria:** both review phases and all RRI bands use the same
checkpoint; existing usable reviewer behavior is unchanged. When no fallback is
triggered, the existing artifact keys/values and exit codes remain unchanged; the
new fallback keys are absent. The checkpoint packet hash covers the exact
`d14_packet` object relayed to the orchestrator, not a reconstructed source packet.
Focused tests must assert the exact authorized, awaiting, and integrity-failure
artifact contracts and exit codes described by HP-1 and EC-1 through EC-3.

| Runtime state | Review verdict | Exit | New fallback keys | Spawn instruction |
|---|---|---:|---|---|
| usable reviewer; fallback not triggered | baseline verdict | baseline | absent | absent |
| D14 trigger; selection pending | `awaiting_fallback_selection` | 3 | artifact path + complete checkpoint, no receipt | absent |
| D14 trigger; preauthorization valid | `d14_required` | 1 | artifact path + complete checkpoint + receipt | exact selected model/effort only |
| malformed relay or integrity mismatch | `blocked` with `blocked=true` | 2 | no authorized receipt | absent |

The first row is a strict no-op contract: focused regression tests compare its
review artifact and exit code with the pre-change baseline for both phases. The
two middle rows assert the presence and schema of `fallback_selection_artifact`
and embedded `fallback_selection`; those keys are absent from the first row.

### FMC-2 RRI evidence

Command:
`python3 scripts/rri.py --auto-cc --touches scripts/peer-workflow-review.py --touches scripts/peer_workflow_review_test.py --D 4 --K 3 --P 3 --T 2 --A 1 --X 2 --platform python`

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 2 | radon max CC 11 across the two allowed Python files | High |
| F files | 1 | two touched files | High |
| D domain | 4 | agent orchestration and review workflow | High |
| T coverage | 2 | existing focused tests require new fallback-state cases | High |
| A ambiguity | 1 | bounded acceptance with one integration choice to resolve | High |
| K coupling | 3 | filesystem artifact and external reviewer handoff | High |
| P impact | 3 | internal workflow/CLI contract changes | High |
| X context | 2 | task, plan, ADR, helper, caller, and tests | High |

Final RRI: `46` -> `Med-high` -> Effort `L`; no penalties; decomposition not
triggered. Antares refinement: typed skip — no task-relevant T3a-watchlisted CWE
hypothesis exists for this workflow change.

**Handoff prompt:** `FMC-2 — Integrate the shared checkpoint only into reviewer
fallback paths and tests. Stop after focused tests pass.`

Task-analysis review: qwen3.6:27b-q4_K_M .agent/peer-task-review-fmc-2-r3.json - BLOCKED

- **Phase-1 owner waiver (2026-08-09):** the owner explicitly waived the
  residual task-wording findings after three completed review rounds and directed
  execution to continue without further phase-1 retries. The runtime-state matrix
  above incorporates and bounds the findings; the verdict is retained faithfully
  as `BLOCKED`, not relabeled `PASS`.
Code-solution review: gemma .agent/peer-code-review-fmc-2.json - PASS

### Peer Reviewer evidence

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`)
- Command: `{ awk '/^## FMC-2 /{emit=1} /^## FMC-3 /{emit=0} emit' docs/tasks/fallback-model-checkpoint.md; git diff -- scripts/peer-workflow-review.py scripts/peer_workflow_review_test.py; } | python3 scripts/peer-workflow-review.py --phase code --rri 46 --caller codex --task-id FMC-2 --artifact .agent/peer-code-review-fmc-2.json --no-think --num-predict 512 --max-wall 180`
- Artifact: `.agent/peer-code-review-fmc-2.json`
- Verdict: `PASS`
- Findings: none
- Gemma fallback: `triggered` — Qwen27's first attempt hit the 180-second wall
  limit and its required retry returned `BLOCKED`.
- D14 fallback: `not triggered` — Gemma produced a usable `PASS` result.
- disposition_divergence: `null`
- Primary-agent disposition: accepted the no-finding review.
- Review artifact: docs/audit/gemma-evidence/FMC-2.json

### Reflection log

Required passes: 3 (`46` -> `Med-high`)

#### Pass 1

- **Draft verdict:** the red tests demonstrated that the reviewer CLI had no
  selection options and all D14 routes bypassed a human checkpoint.
- **Critique findings:** Low-band unavailability still terminated in its legacy
  generic blocked artifact instead of joining the common D14 route.
- **Revisions applied:** registered the shared CLI options, added one common D14
  handler, and routed Low, RRI 26-55, and RRI 56+ through it in both phases.

#### Pass 2

- **Draft verdict:** awaiting and preauthorized paths emitted the intended
  artifacts and exit codes.
- **Critique findings:** the relay needed an explicit structural check before
  hashing, plus receipt validation against the exact `d14_packet`.
- **Revisions applied:** added fail-closed packet validation and authorized
  checkpoint validation; malformed or mismatched data now emits `blocked=true`,
  exit 2, and no spawn instruction.

#### Pass 3

- **Draft verdict:** focused tests passed across both phases and all three bands.
- **Critique findings:** the non-fallback regression assertion initially checked
  only absence of new keys rather than full baseline parity.
- **Revisions applied:** strengthened the test to compare the complete baseline
  artifact (excluding its timestamp). Coverage of newly added executable scope is
  approximately 98%; the inherited whole-file result remains 72%.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | complete selection authorizes the exact D14 handoff in both phases and every band | `scripts/peer_workflow_review_test.py::TestD14FallbackSelection.test_all_bands_and_phases_relay_exact_preauthorized_selection` | passed |
| EC-1 | Edge case | absent selection pauses with checkpoint and exit 3 | `scripts/peer_workflow_review_test.py::TestD14FallbackSelection.test_all_bands_and_phases_pause_before_d14_without_selection` | passed |
| EC-2 | Edge case | Low Gemma unavailability joins the same awaiting/authorized D14 checkpoint | `scripts/peer_workflow_review_test.py::TestD14FallbackSelection.test_all_bands_and_phases_pause_before_d14_without_selection` | passed |
| EC-3 | Edge case | malformed relay or packet-hash mismatch fails closed | `scripts/peer_workflow_review_test.py::TestD14FallbackSelection.test_malformed_d14_packet_fails_closed_without_spawn_authorization`; `scripts/peer_workflow_review_test.py::TestD14FallbackSelection.test_checkpoint_hash_mismatch_fails_closed` | passed |

### Owner final verification

- Owner: `Codex` (orchestrator of record under bounded owner authorization)
- Date: `2026-08-09`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m py_compile scripts/peer-workflow-review.py scripts/peer_workflow_review_test.py`; `python3 -m coverage run --include='scripts/peer-workflow-review.py' scripts/peer_workflow_review_test.py`; `python3 -m coverage report --include='scripts/peer-workflow-review.py' -m`; `git diff --check -- scripts/peer-workflow-review.py scripts/peer_workflow_review_test.py`

## FMC-3 — Low implementation fallback checkpoint

**Status:** [x] Done
**Effort:** L. **RRI:** 49 (Med-high). **Allowed paths:**
`scripts/delegate-low-rri.py`, `scripts/delegate_low_rri_test.py`.

- **HP-1:** exhausted Low repair path emits an authorized Luna/low (or explicit
  human override) cloud-implementer receipt when preauthorized.
- **EC-1:** unavailable local model without a selection emits an awaiting checkpoint
  and stops before cloud implementation.

**Acceptance criteria:** successful Gemma patches are unchanged; only terminal
cloud-escalation exits receive the checkpoint.

**Handoff prompt:** `FMC-3 — Gate terminal Low implementation escalation with the
shared contract. Stop after focused tests pass.`

**Resolved implementation route:** `CLOUD_REQUIRED` (Codex). Both target files
exceed the 500-line local-delegation read gate. A behavior-neutral decomposition
would not isolate the terminal-control-flow change and would expand the approved
scope, so cloud implementation was required.

**RRI evidence:** `python3 scripts/rri.py --auto-cc --touches
scripts/delegate-low-rri.py --touches scripts/delegate_low_rri_test.py --D 4
--K 3 --P 3 --T 2 --A 1 --X 2 --platform python` -> `49` (Med-high, Effort L;
no penalties). Dominant drivers: measured max CC 27, agent-workflow domain 4,
and filesystem/external-handoff coupling 3.

**Terminal-route decision:** Only `--terminal-cloud-escalation` declares that
the caller exhausted the Low-band repair path. The wrapper therefore checkpoints
only local model/request unavailability, terminal timeout after its one internal
stall retry, or a terminal `STATUS: BLOCKED`. `PATCH`, `NO_PATCH`, validation,
scope/apply failures, and non-terminal `BLOCKED` preserve their prior behavior.

Task-analysis review: qwen3.6:27b-q4_K_M .agent/peer-task-review-fmc-3.json - PASS
Code-solution review: gemma .agent/peer-code-review-fmc-3.json - PASS

### Peer Reviewer evidence

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`)
- Command: direct Ollama `/api/chat`, `think=false`, `num_predict=4096`,
  `num_ctx=131072`
- Artifact: `.agent/peer-code-review-fmc-3.json`
- Verdict: `PASS`
- Findings: none
- Gemma fallback: triggered — reason: Qwen27 phase-2 full-packet attempt and
  required compact retry produced no usable response before their bounded timeout.
- D14 fallback: not triggered — reason: Gemma returned a usable PASS verdict.
- disposition_divergence: `none`
- Primary-agent disposition: accepted PASS; no revision was required after review.
- Review artifact: `docs/audit/gemma-evidence/FMC-3.json`

### Reflection log

Required passes: 3 (`49` -> `Med-high`)

#### Pass 1

- **Draft verdict:** The new CLI terminal marker and RRI bind a common
  `cloud-implementer` checkpoint to exact Low delegation evidence.
- **Critique findings:** A model-authored `BLOCKED` does not prove that the
  permitted repair path was exhausted.
- **Revisions applied:** Kept terminality as an explicit orchestrator flag;
  non-terminal `BLOCKED` retains the existing success exit and emits no receipt.

#### Pass 2

- **Draft verdict:** Authorized, awaiting, and invalid-selection checkpoints
  returned distinct exits for terminal unavailable, timeout, and blocked paths.
- **Critique findings:** A non-Low RRI could otherwise consume the Low wrapper
  and obtain a non-Low recommendation.
- **Revisions applied:** Added the `0-25` RRI validation and focused regression
  test; invalid/missing RRI now fails closed with exit 2.

#### Pass 3

- **Draft verdict:** Focused tests cover the selected Luna default, explicit
  override, awaiting pause, local failures, and retained non-terminal behavior.
- **Critique findings:** Changed executable scope initially lacked direct
  coverage of the legacy non-terminal transport rethrow.
- **Revisions applied:** Added that regression test; changed-scope statement
  coverage reached `49/54 (90.7%)` and the phase-2 fallback review returned PASS.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Terminal preauthorized Low escalation emits a receipt with Luna/low or the explicit human override | `scripts/delegate_low_rri_test.py::TerminalCloudFallbackSelection::test_hp1_terminal_blocked_preauthorization_emits_luna_receipt`; `scripts/delegate_low_rri_test.py::TerminalCloudFallbackSelection::test_hp1_explicit_human_model_override_is_preserved` | passed |
| EC-1 | Edge case | Terminal local unavailability without a selection writes awaiting checkpoint and exits 3 before cloud execution | `scripts/delegate_low_rri_test.py::TerminalCloudFallbackSelection::test_ec1_terminal_unavailable_model_awaits_selection_and_exits_3` | passed |
| Regression | Existing behavior | Successful `PATCH` / `NO_PATCH` and non-terminal `BLOCKED` remain unchanged | `scripts/delegate_low_rri_test.py::AuditEmission::test_patch_emits_one_record_with_developer_role`; `scripts/delegate_low_rri_test.py::AuditEmission::test_no_patch_emits_skipped_apply_result`; `scripts/delegate_low_rri_test.py::TerminalCloudFallbackSelection::test_nonterminal_blocked_preserves_existing_success_exit_and_no_checkpoint` | passed |
| Integrity | Edge case | Incomplete selection and missing/non-Low RRI fail closed | `scripts/delegate_low_rri_test.py::TerminalCloudFallbackSelection::test_incomplete_preauthorization_fails_closed`; `scripts/delegate_low_rri_test.py::TerminalCloudFallbackSelection::test_terminal_path_requires_rri_and_fails_closed`; `scripts/delegate_low_rri_test.py::TerminalCloudFallbackSelection::test_terminal_path_rejects_non_low_rri` | passed |

### Owner final verification

- Owner: `Codex` (orchestrator of record under bounded owner authorization)
- Date: `2026-08-09`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/delegate_low_rri_test.py`; `cd scripts &&
  COVERAGE_FILE=/tmp/dubbridge-fmc3.coverage python3 -m coverage run
  --include='*/delegate-low-rri.py' delegate_low_rri_test.py`; `python3 -m
  py_compile scripts/delegate-low-rri.py scripts/delegate_low_rri_test.py`;
  `git diff --check -- scripts/delegate-low-rri.py scripts/delegate_low_rri_test.py
  docs/tasks/fallback-model-checkpoint.md docs/audit/gemma-evidence/FMC-3.json`.

## FMC-4 — Moderate implementation fallback checkpoint

**Status:** [x] Done
**Effort:** L. **RRI:** 44 (Med-high). **Allowed paths:**
`scripts/local-agent/run_local_task.py`, `scripts/local-agent/run_local_task_test.py`.

- **Routing boundary:** the delivery task is Med-high because of its implementation
  risk, but it changes only the runner behavior for a *Moderate task card*
  (`26 <= card.rri <= 40`). FMC-5 exclusively owns Med-high/ADR-038 routes.
- **HP-1:** a terminal Moderate local failure with a complete preauthorization emits
  a `cloud-implementer` receipt recommending `gpt-5.6-terra` at `medium`, bound to
  the canonical terminal-attempt packet.
- **EC-1:** a terminal Moderate local failure without a complete selection writes an
  `awaiting_fallback_selection` checkpoint and pauses before any cloud execution.
- **EC-2:** a successful local implementation emits no fallback checkpoint,
  authorization, or cloud-model invocation.
- **EC-3:** an intermediate `repair-needed` result emits neither checkpoint nor
  receipt; the existing Moderate repair budget and audit semantics remain intact.

**Acceptance criteria:** selection is added only after an eligible terminal
Moderate non-success result: `budget_exhausted/repair_attempts_exhausted`,
`budget_exhausted/total_turns_exhausted`, `transport_error`,
`boundary_violation`, or `out_of_scope`. A repair-needed result is not terminal;
nor are `success` and malformed-tool recovery that stays inside the running
session. The implementation must construct one canonical `terminal_attempt_packet`
for every eligible terminal result, whether or not that session contains a
`test_result` event. Its hash input contains the task ID, `phase=implementation`,
terminal status and reason, trigger and trigger kind, card RRI, implementer model,
effective turn and repair limits plus counters used, the full terminal transcript,
and the existing final test-result bundle when one exists (otherwise an explicit
`null`). The SHA-256 of this packet—not a test-result-only attempt bundle—binds the
emitted checkpoint or receipt. An incomplete `human-select` selection remains
awaiting; a complete `preauthorized` selection produces the receipt. The runner
must only emit this shared-contract artifact; it must not start a cloud model.

**Verification:** add focused unit tests in
`scripts/local-agent/run_local_task_test.py` for HP-1, EC-1, EC-2, and EC-3,
including the receipt/checkpoint hash binding for both a terminal final-test
failure and a terminal path without `test_result` (boundary, transport, scope, or
turn budget). Prove that repair-budget exhaustion and total-turn exhaustion stay
distinct, and that the existing repair/audit flow is unchanged before the terminal
result.

**Evidence to emit:** canonical terminal-attempt packet, awaiting checkpoint or
authorized receipt, focused test output, and phase-1/phase-2 review artifacts.

**Status artifacts affected:** this ledger and the runner audit evidence.

**Resolved implementation route:** owner-directed `CLOUD_PRIMARY`. Codex starts
with `gpt-5.6-terra` at `high`; evidence of capability/risk not covered by the
approved task routes to `gpt-5.6-sol` at `high`. No Qwen35 local implementation
attempt is permitted for this task.

### FMC-4 RRI evidence

Command:
`python3 scripts/rri.py --C 2 --touches scripts/local-agent/run_local_task.py --touches scripts/local-agent/run_local_task_test.py --D 4 --K 4 --P 3 --T 1 --A 0 --X 3 --platform dubbridge`

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 2 | material runner branches are in the 11–20 CC range | High |
| F files | 1 | two allowed paths | High |
| D domain | 4 | runner escalation and authorization contract | High |
| T coverage | 1 | focused unit-test surface exists and requires cases | High |
| A ambiguity | 0 | terminal states, packet contract, and runtime-band boundary are explicit | High |
| K coupling | 4 | runner, shared checkpoint, and attempt-evidence contract | High |
| P impact | 3 | internal implementation-routing/audit behavior | High |
| X context | 3 | runner, tests, task, plan, and ADR-039 | High |

Final RRI: `44` -> `Med-high` -> Effort `L`; no penalties; decomposition not
triggered. Antares refinement: typed skip — no task-relevant T3a-watchlisted CWE
hypothesis exists for this workflow change.

**Handoff prompt:** `FMC-4 — Gate terminal Moderate runner escalation with the shared
contract. Stop after focused tests pass.`

Task-analysis review history: d14 .agent/peer-task-review-fmc-4-r3-cloud-route.json - BLOCKED (resolved by canonical terminal-attempt packet contract)
Task-analysis review: d14 .agent/peer-task-review-fmc-4-r4-terminal-packet.json - PASS
Code-solution review: gemma .agent/peer-code-review-fmc-4.json - PASS

### Peer Reviewer evidence

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`)
- Command: direct Ollama `/api/chat`, `think=false`, `num_predict=1024`,
  `num_ctx=32768`, reviewing the FMC-4 acceptance criteria and scoped diff.
- Artifact: `.agent/peer-code-review-fmc-4.json`
- Verdict: `PASS`
- Findings: none
- Gemma fallback: triggered — Qwen27 did not produce a usable review artifact
  after its bounded review attempts and was stopped.
- D14 fallback: not triggered — Gemma returned a usable PASS verdict; therefore
  no D14 provider route was required.
- disposition_divergence: `none`
- Primary-agent disposition: accepted PASS; no revision was required after review.
- Review artifact: `docs/audit/gemma-evidence/FMC-4.json`

Antares post-implementation: typed skip — no task-relevant T3a-watchlisted CWE
hypothesis exists for this workflow change.

### Reflection log

Required passes: 3 (`44` -> `Med-high`)

#### Pass 1

- **Draft verdict:** Terminal Moderate exits now produce one packet that includes
  status, reason, limits, counters, full transcript, and final test result/null.
- **Critique findings:** The result artifact also needed the canonical packet so a
  later operator can reproduce the checkpoint hash without reconstructing it.
- **Revisions applied:** Persisted `terminal_attempt_packet` alongside the shared
  checkpoint and its artifact path before writing the final result.

#### Pass 2

- **Draft verdict:** Eligible terminal classification is limited to the five
  specified Moderate exit families.
- **Critique findings:** Repair-budget and total-turn exhaustion must remain
  distinguishable even when both have `budget_exhausted` status.
- **Revisions applied:** Added separate reason assertions and hash-distinct packet
  coverage for `repair_attempts_exhausted` and `total_turns_exhausted`.

#### Pass 3

- **Draft verdict:** Focused and full runner tests preserve success and
  repair-needed behavior while exercising receipt, awaiting, and no-test-result
  terminal paths.
- **Critique findings:** Independent review found no remaining issue.
- **Revisions applied:** None; `87` runner tests passed and changed-file coverage
  was `93%`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Terminal Moderate failure with preauthorization emits a Terra/medium cloud-implementer receipt bound to the canonical packet | `scripts/local-agent/run_local_task_test.py::ModerateTerminalFallbackSelection::test_hp1_preauthorized_terminal_test_failure_emits_receipt_bound_to_packet` | passed |
| EC-1 | Edge case | Terminal Moderate failure without selection writes an awaiting checkpoint and pauses before cloud execution | `scripts/local-agent/run_local_task_test.py::ModerateTerminalFallbackSelection::test_main_persists_awaiting_checkpoint_for_terminal_boundary_without_test_result` | passed |
| EC-2 | Edge case | Successful local implementation emits no checkpoint or receipt | `scripts/local-agent/run_local_task_test.py::ModerateTerminalFallbackSelection::test_ec2_success_and_ec3_repair_needed_do_not_create_terminal_checkpoint` | passed |
| EC-3 | Edge case | Repair-needed behavior emits no checkpoint and retains the existing repair flow | `scripts/local-agent/run_local_task_test.py::ModerateTerminalFallbackSelection::test_ec2_success_and_ec3_repair_needed_do_not_create_terminal_checkpoint`; `scripts/local-agent/run_local_task_test.py::HP2RepairThenSuccess::test_exactly_two_repair_turns_then_success` | passed |

### Owner final verification

- Owner: `Codex` (orchestrator of record under owner approval)
- Date: `2026-08-09`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/local-agent/run_local_task_test.py`;
  `cd scripts/local-agent && COVERAGE_FILE=/tmp/dubbridge-fmc4.coverage python3
  -m coverage run --source=run_local_task run_local_task_test.py`;
  `cd scripts/local-agent && COVERAGE_FILE=/tmp/dubbridge-fmc4.coverage python3
  -m coverage report -m --include='*/run_local_task.py'`;
  `python3 -m py_compile scripts/local-agent/run_local_task.py
  scripts/local-agent/run_local_task_test.py`;
  `git diff --check -- scripts/local-agent/run_local_task.py
  scripts/local-agent/run_local_task_test.py docs/tasks/fallback-model-checkpoint.md
  docs/audit/gemma-evidence/FMC-4.json`.

## FMC-5 — Med-high implementation fallback checkpoint

**Status:** [x] Done
**Effort:** L. **RRI:** 50 (Med-high). **Allowed paths:**
`scripts/local-agent/run_med_high_task.py`,
`scripts/local-agent/run_med_high_task_test.py`.

- **HP-1:** operational-only failure recommends Terra/high and can consume a valid
  preauthorization receipt. Operational-only means the exact Qwen35 supervisor
  process cannot be launched or its local binding is unavailable, without evidence
  of a task acceptance, scope, organization, boundary, or risk failure. The
  checkpoint is `role=cloud-implementer`, `mode=preauthorized`, and accepts only
  a complete `fallback-selection-v1` selection whose receipt validates against
  the canonical packet for that exact handoff bundle.
- **HP-2:** hard exclusion, `CLOUD_REQUIRED`, or acceptance/scope/organization
  failure recommends Sol/high. A timeout, nonzero runner exit, malformed runner
  transcript, or a failed local acceptance/scope/organization/boundary gate is a
  capability-or-risk takeover and follows this row.
- **EC-1:** missing selection in `human-select` mode writes
  `awaiting_fallback_selection`, preserves the ADR-038 handoff bundle, returns the
  dedicated pause exit code `3`, and pauses before any cloud consumption. A partial
  `preauthorized` selection fails closed with `verdict=blocked`, a stable
  field-specific integrity/selection error, exit code `2`, no receipt, and no cloud
  instruction.
- **EC-2:** the canonical fallback packet includes the exact emitted handoff bundle
  (or its byte-for-byte SHA-256 plus immutable artifact location); mutating that
  bundle after selection makes receipt validation fail. The supervisor writes a
  `verdict=blocked` checkpoint artifact with a stable `bundle_receipt_mismatch`
  summary, returns exit code `2`, retains the unmodified bundle for diagnosis, and
  emits no cloud instruction. The tests must prove this state transition,
  telemetry, and absence of a cloud instruction.

**Acceptance criteria:** every non-success Med-high route emits the checkpoint;
GO_LOCAL success remains unchanged and emits neither bundle checkpoint nor receipt.
`human-select` and `preauthorized` use the shared contract's explicit modes: the
former awaits a complete human selection, the latter requires model, effort, and
selector before authorization. Recommendations are pure static resolver metadata
from the task-frozen matrix, never an inference or a model invocation; the
supervisor only returns the selected model/effort after validating the matching
receipt. Focused tests cover each takeover class, pause exit, no-invocation
boundary, and tampered-bundle rejection.

### FMC-5 RRI evidence

- Full evidence: `.agent/fmc-5-rri.md`.
- Recomputed RRI: `50` -> `Med-high` -> Effort `L`.
- Antares refinement: typed skip — no task-relevant T3a-watchlisted CWE
  hypothesis is present for this agent-workflow change.

**Handoff prompt:** `FMC-5 — Gate all ADR-038 cloud routes with the shared contract.
Stop after focused tests pass.`

Task-analysis review: gemma .agent/peer-task-review-fmc-5-r3.json - PASS
Code-solution review: d14 .agent/peer-code-review-fmc-5.json - PASS

### Peer Reviewer evidence

- Reviewer: `d14` (same-provider degraded adjudicator)
- Command: `python3 scripts/peer-workflow-review.py --phase code --rri 50
  --caller codex --task-id FMC-5 --artifact .agent/peer-code-review-fmc-5.json
  --max-wall 60 --idle-timeout 30 --no-think`, followed by a context-isolated
  D14 review of only the final diff summary and FMC-5 acceptance criteria.
- Artifact: `.agent/peer-code-review-fmc-5.json`
- Verdict: `PASS`
- Findings: none
- Gemma fallback: triggered — Qwen27 retried through the band route but idled
  for 30 seconds without a token; Gemma also idled for 30 seconds without a
  token and returned no usable verdict.
- D14 fallback: triggered — the Qwen27 and Gemma review chain was unusable.
- D14 provider route: `same-provider-degraded` — the required cross-provider
  `claude -p --bare --model claude-sonnet-5 --effort medium` attempt was
  unauthenticated (`Not logged in`), so a context-isolated Codex D14 reviewer
  was used.
- disposition_divergence: `none`
- Primary-agent disposition: accepted the context-isolated D14 PASS; no
  post-review revision was needed.
- Review artifact: `docs/audit/gemma-evidence/FMC-5.json`

Antares post-implementation: typed skip — FMC-5 carries no task-relevant
T3a-watchlisted CWE hypothesis, so a generic security sweep was not justified.

### Reflection log

Required passes: 3 (`50` -> `Med-high`)

#### Pass 1

- **Draft verdict:** Every supervisor non-success now carries a checkpoint bound
  to the emitted ADR-038 bundle's absolute path and byte SHA-256.
- **Critique findings:** A direct `CLOUD_REQUIRED` path and a runner launch
  failure require different recommendation metadata without invoking either
  model.
- **Revisions applied:** Classified only `transport_error` as operational-only;
  all other non-success routes use the static capability-risk recommendation.

#### Pass 2

- **Draft verdict:** The shared selection modes produce awaiting and authorized
  artifacts while the supervisor exposes an instruction only after validation.
- **Critique findings:** The bundle could change after a receipt is created and
  before a consumer uses it.
- **Revisions applied:** Re-read the bundle after receipt creation and convert a
  digest mismatch into receipt-free `bundle_receipt_mismatch` blocking state.

#### Pass 3

- **Draft verdict:** Focused tests cover successful local execution, operational
  and capability-risk paths, pause/blocked exits, and bundle tampering.
- **Critique findings:** Context-isolated D14 review found no remaining issue
  after the Qwen27 and Gemma reviewer chain was unusable.
- **Revisions applied:** None; the focused suite passed 50 tests with 97% line
  coverage for the changed supervisor.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Operational startup failure authorizes Terra/high against the exact bundle | `scripts/local-agent/run_med_high_task_test.py::SuperviseIntegrationTest::test_hp1_operational_start_failure_authorizes_terra_against_bundle` | passed |
| HP-2 | Happy path | `CLOUD_REQUIRED` authorizes Sol/high without launching or invoking a model | `scripts/local-agent/run_med_high_task_test.py::SuperviseIntegrationTest::test_hp2_cloud_required_authorizes_sol_without_model_invocation` | passed |
| EC-1 | Edge case | Missing selection pauses with exit 3; incomplete preauthorization blocks with exit 2 | `scripts/local-agent/run_med_high_task_test.py::SuperviseIntegrationTest::test_ec1_human_selection_pause_preserves_bundle_without_instruction`; `scripts/local-agent/run_med_high_task_test.py::MainCliTest::test_ec1_cli_partial_preauthorization_blocks` | passed |
| EC-2 | Edge case | Mutated bundle invalidates the receipt, blocks with `bundle_receipt_mismatch`, and emits no instruction | `scripts/local-agent/run_med_high_task_test.py::SuperviseIntegrationTest::test_ec2_bundle_mutation_after_receipt_is_blocked` | passed |

### Owner final verification

- Owner: `Codex` (orchestrator of record under approved FMC-5 scope)
- Date: `2026-08-09`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior.
- Commands run: `python3 scripts/local-agent/run_med_high_task_test.py`;
  `cd scripts/local-agent && COVERAGE_FILE=/tmp/dubbridge-fmc5.coverage python3
  -m coverage run --source=run_med_high_task run_med_high_task_test.py`;
  `cd scripts/local-agent && COVERAGE_FILE=/tmp/dubbridge-fmc5.coverage python3
  -m coverage report -m --include='*/run_med_high_task.py'`;
  `python3 -m py_compile scripts/local-agent/run_med_high_task.py
  scripts/local-agent/run_med_high_task_test.py`;
  `git diff --check -- scripts/local-agent/run_med_high_task.py
  scripts/local-agent/run_med_high_task_test.py docs/tasks/fallback-model-checkpoint.md
  docs/audit/gemma-evidence/FMC-5.json`.

## FMC-6a — Authoritative workflow, policy, and template propagation

**Status:** [x] Done — 2026-08-09

**Effort:** S (policy-only; development review/Reflection exempt).

Synchronize ADR-039, the workflow guide, RRI/HITL policies, Low handoff playbook,
and Compact Approval Task Card. Document `human-select` as the interactive default,
`preauthorized` as the automation mode, the D14 Balanced floor, receipt validation,
the pause exit, and exact phase-specific behavior. Do not update agent summaries or
the generated override in this subtask.

Task-analysis review: n/a — docs/policy/config-only exemption.
Code-solution review: n/a — docs/policy/config-only exemption.

### Completion evidence

- Updated the workflow guide, RRI/HITL policies, Low handoff playbook, and
  Compact Approval Task Card with ADR-039's packet-bound selection contract.
- Verified: `git diff --check -- docs/templates/compact-approval-task-card.md
  docs/policies/RRI_POLICY.md docs/policies/HITL_AUTONOMY_POLICY.md
  docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md
  docs/playbooks/AGENT_WORKFLOW_GUIDE.md docs/tasks/fallback-model-checkpoint.md`;
  `make qa-docs`.
- At this subtask's completion, FMC-6b remained pending; agent summaries and
  `AGENTS.override.md` were intentionally not changed here.

## FMC-6b — Agent-summary and generated-override propagation

**Status:** [x] Done — 2026-08-09

**Effort:** S (config-only; development review/Reflection exempt).

Synchronize `AGENTS.md` and `CLAUDE.md`, then regenerate `AGENTS.override.md` with
`python3 scripts/generate-agents-override.py --write`; do not edit the generated
file manually. Verify the byte-exact drift gate before marking this subtask done.

Task-analysis review: n/a — docs/policy/config-only exemption.
Code-solution review: n/a — docs/policy/config-only exemption.

### Completion evidence

- Added the ADR-039 fallback-selection summary to `AGENTS.md` and `CLAUDE.md`.
- Regenerated `AGENTS.override.md` exclusively with
  `python3 scripts/generate-agents-override.py --write`; it was not hand-edited.
- Verified: `git diff --check -- AGENTS.md CLAUDE.md AGENTS.override.md
  docs/tasks/fallback-model-checkpoint.md docs/plan/fallback-model-checkpoint.md`;
  `make qa-docs`.
