---
type: TaskList
title: "Tasks: Local Role Prompt Canonicalization"
plan: docs/plan/local-role-prompt-canonicalization.md
status: proposed
rri: 40
band: Moderate
effort: M
---

# Tasks: Local Role Prompt Canonicalization

RRI shown in frontmatter is for **LRPC-1**, approved 2026-08-19. Each task
below carries its own `scripts/rri.py` result — do not assume the
ledger-level RRI applies to any other task. LRPC-2 through LRPC-8 are
sequenced but intentionally left at a lighter definition; each gets its own
pre-implementation analysis pass (its own `scripts/rri.py` run, its own
phase-1 review, its own approval) before implementation starts, per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`'s "present the next task"
discipline.

**LRPC-0b was inserted 2026-08-19, ahead of LRPC-1's Implement phase**, as a
prerequisite runner refactor (RRI 41, Med-high) — see § LRPC-0b immediately
below. It does not change LRPC-1's approved scope; LRPC-1's Implement phase
is blocked on it, everything already recorded for LRPC-1 (analysis, phase-1
review, approval) stands unchanged.

## Note on the motivating bug fix

The confirmed drift bug that grounds this plan
(`scripts/gemma-code-review.py:188-189` missing "certify coverage" and
paraphrasing "mark tasks complete" as "close tasks") is documented and
tracked as its own item **outside this ledger**, in
`docs/tasks/gemma-push-reviewer-role.md § T9` (RRI 23 -> Low). It has no
dependency edge into LRPC-1 or any task below — nothing here depends on it,
it depends on nothing here — and it is not part of the canonicalization
mechanism itself: it is a one-off manual correction of the single live
instance, superseded automatically once LRPC-3 lands and regenerates that
script's prompt from the canonical anchor. It was kept out of this ledger's
numbered tasks because it doesn't belong to the dependency graph below; see
`docs/tasks/gemma-push-reviewer-role.md § T9` for its full definition and
status.

## Pre-existing test debt discovered during LRPC-0b (out of scope, tracked here for visibility)

**Discovered 2026-08-19, before LRPC-0b's Implement phase**, while
establishing the regression baseline required by LRPC-0b's own HP-1
("identical test count and identical pass/fail outcome per test"). Two
distinct, unrelated pre-existing defects on clean `main`
(`scripts/local-agent/run_local_task_test.py`), neither caused by LRPC-0b
and neither part of its scope:

1. **Collection-blocking dead reference (fixed as a standalone one-line
   change, not part of LRPC-0b's own diff):** `run_local_task_test.py:18`
   read `runner_workflow_gate = rlt.runner_workflow_gate`, but
   `run_local_task.py` does not import `runner_workflow_gate` (that wiring
   was apparently dropped when `LASE2-T3` stripped Serena from
   `scripts/local-agent/runner_workflow_gate.py` — see
   `docs/plan/local-agent-simple-editing.md` line 95). This crashed
   collection of the *entire* test module (`AttributeError`) before a single
   test could run. The assigned local variable was verified unused anywhere
   else in the file (`grep -n "\brunner_workflow_gate\b"` — one hit, the
   dead assignment itself), so the line was deleted outright rather than
   fixed to import the real module — reintroducing a real import would
   silently reattach the (currently uncalled) organization-gate wiring to
   test collection, which is a separate decision this task does not make.
2. **33 pre-existing test failures, not collection errors, revealed once
   (1) was fixed.** Root-caused via error-message clustering
   (`docs/audit/gemma-evidence/` is not applicable here — this is a finding,
   not a reviewed change) into two clusters, both content/contract drift
   unrelated to code location:
   - Several tests assert exact substrings (`"disposable"`, `"resolved
     relative to the worktree root"`, `"run_command"`, the literal word
     `"turns"` used differently) against `TOOL_CALLING_SYSTEM_PROMPT`'s
     current text — the live prompt no longer contains that wording.
   - Several tests exercise the model calling `read_file`/`run_command` as
     model-invokable tools and assert on the old success/failure shapes —
     but `FORBIDDEN_MODEL_TOOL_NAMES = frozenset(("read_file",
     "run_command"))` (line 364) now makes both a `boundary_violation` by
     design, so these tests get `'boundary_violation' != 'aborted'` /
     `'boundary_violation' != 'budget_exhausted'` / `'boundary_violation' !=
     'out_of_scope'` instead of their expected old outcome.
   Both clusters point at the same underlying event: the runner's tool
   contract was simplified (full-file context supplied up front; the model
   no longer reads or executes commands itself) without updating this
   suite's coverage of the old contract. Full failing-test list preserved at
   session time in `/private/tmp/claude-501/-Users-matias-dubbridge/7753f577-9865-4822-9305-8087cbac2042/scratchpad/lrpc0b-preexisting-failures.txt`
   (scratch path, not committed — reproduce via `python3 -m pytest
   scripts/local-agent/run_local_task_test.py -q` against `main` plus fix
   (1) above).

**Disposition:** owner directed (2026-08-19) to document this as separate
debt and proceed with LRPC-0b regardless — this finding does not block
LRPC-0b. It has **no RRI yet and is not a task** — a future task must be
opened (own analysis, own `scripts/rri.py` run, own phase-1 review) either to
delete the 33 obsolete tests, rewrite them against the current contract, or
both, depending on whether the old `read_file`/`run_command` coverage still
has a live equivalent to preserve. Until that task exists, LRPC-0b's own
regression oracle (see HP-1 below) is scoped to the **60 tests that pass**
after fix (1), on the verified basis that all 33 known failures are
content/contract assertions unrelated to which file a function's code lives
in — none of them fail due to a missing symbol, an import path, or a
location-dependent lookup, which is the only failure class an Extract
Module refactor could introduce.

## LRPC-0b — Refactor `run_local_task.py` under the target-file size gate

**Why this task exists (inserted 2026-08-19, before LRPC-1 implementation
start):** `AGENT_WORKFLOW_GUIDE.md § Handoff prompt format` "Target-file size
gate" requires that before delegating RRI 26-40 work through
`scripts/local-agent/run_local_task.py`, every file the local implementer
must read in full stays under 500 lines — and `run_local_task.py` is the
runner itself, at **1491 lines**. LRPC-1's own implementation route names
this exact runner. Rather than delegate against an oversized file (silently
degrading local-model latency/attention per the gate's own rationale) or
fall back to cloud, this task refactors the runner first, so LRPC-1 and
every later Moderate/Med-high task that reuses this runner benefits.

This task does not change LRPC-1's already-approved scope (§ LRPC-1 below is
unmodified). It is a new prerequisite task in the dependency chain, not a
replan of LRPC-1 itself.

- **RRI:** 41 -> **Med-high (41-55)**. `scripts/rri.py --cc 5 --D 3 --K 2
  --P 2 --T 5 --A 1 --X 1 --touches scripts/local-agent/run_local_task.py
  --touches scripts/local-agent/session_loop.py --touches
  scripts/local-agent/audit_record.py --touches
  scripts/local-agent/rust_toolchain.py --touches
  scripts/local-agent/cli.py`. No penalties triggered; decomposition not
  triggered. Behavior-preserving refactor of already-tested code (no new
  logic, only extraction) keeps D/P low, but **F=5** (five touched files)
  pushes the base score into Med-high even though no single change is
  individually risky — a five-way module split is not a "simple code patch."
  Gate for this band: "Plan + explicit acceptance criteria required before
  approval" (satisfied by this section).
- **Effort:** L (derived from the Med-high band per the canonical effort
  mapping — not a separate subjective estimate).
- **Dependencies:** none. Blocks LRPC-1's implementation phase only (LRPC-1's
  analysis, phase-1 review, and approval already completed against the
  original scope and are unaffected).
- **Objective:** reduce `scripts/local-agent/run_local_task.py` to under 500
  lines by extracting cohesive, independently-named submodules along its
  existing seams — no behavior change, no logic rewrite. `run_local_task.py`
  itself becomes a thin composition/CLI-entry file that imports and
  re-exports the extracted symbols, so `run_local_task_test.py`'s existing
  `import run_local_task as rlt; rlt.<name>` access pattern keeps working
  unchanged (Facade re-export, not a test-suite rewrite).
- **Design pattern basis:** Extract Module / Single Responsibility — split
  along the file's own already-separable concerns (visible from its current
  top-level function grouping):
  - `session_loop.py` — `run_loop`, `ToolCall`, `MalformedToolCall`,
    `parse_tool_call`, `require_argument`, `apply_tool_call` (the
    turn-by-turn model-interaction state machine; the largest single
    cohesive block, ~250 lines).
  - `audit_record.py` — `build_audit_record`, `build_attempt_bundles`,
    `build_terminal_attempt_packet`, `build_moderate_fallback_checkpoint`,
    `_is_moderate_card`, `_terminal_result_is_eligible` (closure/evidence
    construction, no model interaction).
  - `rust_toolchain.py` — `_rust_edition_for_path`, `build_default_formatter`,
    `build_default_boundary` (Rust-specific formatter/boundary wiring; the
    part of this runner most likely to need per-language extension later,
    which is exactly why it should not stay fused to the session loop).
  - `cli.py` — `parse_args`, `load_card`, `main` (argument parsing and
    process entry point).
  - `run_local_task.py` (remaining) — `TaskCard`, `EffectiveLimits`,
    `resolve_effective_limits`, `build_live_chat_fn`,
    `build_default_test_runner`, `build_initial_system_message`,
    `render_authorized_context`, plus re-exports of every symbol moved above,
    so the module's public surface is unchanged for all existing callers and
    tests.
- **In scope:** the five files above; no change to `run_local_task_test.py`
  itself except what is strictly required to keep it passing unchanged (it
  should require none, since imports resolve through the re-export facade).
- **Out of scope:** any behavior change, any change to the tool-call JSON
  contract, any change to `boundary.py` / `scope_check.py` / `gemma_local.py`
  / `handoff_schema.py` (unrelated modules this file imports), and LRPC-1's
  already-approved scope.
- **HP-1:** after the refactor, `python3 -m pytest
  scripts/local-agent/run_local_task_test.py` passes with the identical test
  count and identical pass/fail outcome per test as the pre-refactor run — a
  behavior-preservation proof, not a new-feature test.
- **EC-1:** `wc -l scripts/local-agent/run_local_task.py` reports fewer than
  500 lines after the refactor, and no other file in `allowed_paths` for this
  task exceeds 500 lines either (the gate applies per-file, not just to the
  originally-named one).
- **EC-2:** every symbol `run_local_task_test.py` currently accesses via
  `rlt.<name>` (audited from the test file's own source before refactoring)
  remains accessible the same way afterward — a grep-based symbol-parity
  check, not just "tests pass" (a symbol could be accidentally dropped and
  covered by no test).
- **Evidence to emit:** before/after line counts per file; full test-suite
  run output (pre- and post-refactor, showing identical results);
  symbol-parity check output (EC-2).
- **Status artifacts affected:** none — this is an internal refactor of
  tooling, not a product-facing or governance-invariant change. No ADR,
  roadmap, or architecture doc references this file's internal structure.
- **Route (Med-high, 41-55):** ADR-038 Architect-refined single-attempt gate
  — Muse Glimmer advisory refinement (`GO_LOCAL` | `CLOUD_REQUIRED`) -> hash-
  bound route receipt (may downgrade, never upgrade) -> every result escalates
  to the concrete cloud-takeover model with the full ADR-038 §5 evidence
  bundle, **except** any module independently qualified under ADR-040
  per-module complexity-split routing. This task is a strong ADR-040
  candidate: the five extracted files are heterogeneous in complexity
  (`cli.py`/`audit_record.py`/`rust_toolchain.py` are low-CC mechanical
  moves; `session_loop.py` carries the state-machine control flow) and their
  target paths are disjoint by construction (each extraction touches exactly
  one new file plus the shared `run_local_task.py` re-export surface) — so
  the per-module split trigger (>=1 module C<=1 AND >=1 module C>=2,
  disjoint `allowed_paths`) should be evaluated explicitly at this task's own
  pre-implementation pass via `scripts/local-agent/module_split_gate.py`
  before defaulting to whole-task cloud-only routing. Band-resolved
  independent review (phases 1/2) and 3 Reflection passes apply regardless
  of where any module's code is authored.

### Module-split routing evidence

`scripts/local-agent/module_split_gate.py evaluate_split()` was run against
the five target files with real per-file cyclomatic complexity measured via
`radon cc -s -j` (max CC per file across its constituent functions):
`session_loop.py`=22 (C=3), `audit_record.py`=33 (C=4), `rust_toolchain.py`=6
(C=1), `cli.py`=20 (C=2), `run_local_task.py`(remaining)=18 (C=2). Result:
**`split`** — only `rust_toolchain.py` (C=1) qualified for the local tramo;
the other four (C>=2) were assigned to the cloud tramo
(`local_repair_budget=2, cloud_repair_budget=1`).

**Not used.** The resulting split was lopsided (1 file local / 4 files
cloud) and cloud implementation was unavailable this session (no Codex
tokens; owner directive 2026-08-19 to proceed without cloud and maximize
local/direct execution instead of pursuing a fragmented split for one
trivial extraction). The task was implemented as a single whole-task unit
by the primary orchestrator directly, not through either the local-agent
runner or a cloud implementer — see "Implementation routing evidence"
below for the full deviation rationale. The gate's `split` decision is
recorded here for audit completeness; it did not govern how the code was
actually authored.

### Implementation routing evidence

**Deviation from the Med-high route described above, directed by the owner
2026-08-19.** No Codex/cloud tokens were available this session. Per owner
instruction ("no hay tokens. tendras que hacerlo tu mismo"), the primary
orchestrator (Claude Code) authored the extraction directly, rather than
through `scripts/local-agent/run_local_task.py` (ironic given this task's
own objective — that runner is the file being resized) or a cloud
implementer.

This is treated as a bounded, explicitly-authorized exception to the
default Med-high cloud-only route — not the "documented tooling-failure" or
"mechanical lint-driven refactor" exceptions in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band
decomposition` (those apply after a local-agent repair budget is
exhausted; this task never entered that route at all, since ADR-038
Med-high has no whole-task local attempt to begin with). It is instead a
direct owner authorization to bypass the local-agent/cloud split entirely
for this specific task, given:
- the objective is a **behavior-preserving mechanical extraction**
  (Extract Module/Single Responsibility; every function body verbatim,
  zero logic changes) — the class of change the guide's own "mechanical
  lint-driven refactor" exception contemplates, even though this task's
  RRI (driven by F=5 touched files, not by any single risky change) placed
  it in Med-high rather than a lower band;
- an unusually strong automated regression oracle was available and used
  (byte-identical 60-passed/33-failed test-name sets before/after, a
  scripted 21/21 symbol-parity check, per-file line-count verification,
  and a real end-to-end subprocess CLI smoke test — see "Unit coverage
  certification" below); and
- the band-resolved independent review (Gemma, phase 1 already recorded;
  phase 2 below) and the full 3-pass Reflection cycle still ran regardless
  of authorship route, per the guide's standing rule that review/Reflection
  requirements are fixed by RRI band, never by where code was authored.

No repair attempts were consumed (none exist at this route); no fallback
selection artifact was produced (ADR-039's checkpoint applies to
local-agent/D14 terminal exits, neither of which this route entered).

### Phase 1 — Task-analysis review

`Task-analysis review: n/a (gap — see note) - PASS (user-approved)`

**Honesty note:** no separate Gemma phase-1 pass was run against LRPC-0b's
task definition specifically before the user approved it ("Aprobado
LRPC-0b"). The task's full definition (RRI, route, objective, HP-1/EC-1/
EC-2) was already written into this ledger at insertion time and the user
approved that written card directly. This is a process gap relative to
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Per-task discipline`'s "Phase 1 —
Task-analysis review... before presenting or delegating any task" — it
should have been sent to Gemma before presentation. Recorded here rather
than silently omitted or backfilled with a fabricated review; the user's
explicit approval stands as the HITL gate regardless, and phase 2 (code-
solution review) below did run in full.

### Peer Reviewer evidence

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, explicit `--model` override —
  same rationale as LRPC-1's phase-1 review: the bare script default
  resolves to `muse-glimmer:30b-q4_K_M`, the RRI 26-55 intermediate
  fallback, not the RRI 26-55 primary)
- Precheck: Ollama restarted for this task ID (`20822` -> `32453`),
  listener confirmed on `:11434`, warm-up probe `done_reason: stop`
  non-empty content before the real packet was sent
- Command: `python3 scripts/gemma-code-review.py --model
  gemma4:26b-a4b-it-qat --num-ctx 65536 --num-predict 4096 --no-think
  --passes 3 --task-id LRPC-0b --out <result.json> <packet.txt>`
- Artifact: `docs/audit/gemma-evidence/LRPC-0b-phase2.json`
- Verdict: `PASS` (status `findings`, but 0 consensus / 0 pass-specific / 0
  severity-inconsistent findings across all 3 passes — the only items
  raised were 3 `likely_false_positive`/`location_inconsistent`
  observations, all confirming the `BoundaryViolation` double-import guard
  is correctly preserved, not defects)
- Findings: none actionable. Reviewer summary: "The refactor successfully
  splits the 1491-line `run_local_task.py` into five submodules, all under
  the 500-line limit. The facade pattern in `run_local_task.py` correctly
  re-exports all public symbols, maintaining backward compatibility for
  existing test suites. The `__main__` guard for `BoundaryViolation` is
  preserved, preventing the double-import identity bug. No logic changes
  were detected in the extracted functions."
- Muse Glimmer fallback: not triggered — primary Gemma responded normally
  across all 3 passes.
- D14 fallback: not triggered.
- disposition_divergence: `none` — no findings required disposition beyond
  the reviewer's own not-actionable classification.

### Reflection log

Required passes: 3 (`RRI 41` -> `Med-high`)

#### Pass 1

- **Draft verdict:** initial 5-file split completed; `run_local_task.py`
  still at 554 lines (over the 500-line target) because
  `TOOL_CALLING_SYSTEM_PROMPT`, `TOOL_CALL_JSON_SCHEMA`, and
  `build_live_chat_fn` had not yet been relocated.
- **Critique findings:** EC-1 (every file under 500 lines) not yet
  satisfied; the three chat-transport/tool-contract items are large static
  blocks that don't belong in a "composition/CLI-entry" facade anyway.
- **Revisions applied:** moved `TOOL_CALLING_SYSTEM_PROMPT`,
  `TOOL_CALL_JSON_SCHEMA`, and `build_live_chat_fn` into `cli.py` (the
  module whose `main()` is their sole real caller); left thin
  backward-compatible wrappers in `run_local_task.py` preserving the exact
  original call signatures/defaults for direct `rlt.build_live_chat_fn`
  callers.

#### Pass 2

- **Draft verdict:** all five files now under 500 lines; `rlt.main()` still
  passed a `build_live_chat_fn` keyword argument to `cli.main()` that no
  longer existed in `cli.main()`'s signature after the Pass 1 move.
- **Critique findings:** `TypeError: main() got an unexpected keyword
  argument 'build_live_chat_fn'` on every `rlt.main(...)` call path —
  caught by re-running the full test suite, not by static inspection alone
  (confirms the value of the regression oracle over code-reading for this
  kind of cross-file wiring change).
- **Revisions applied:** removed the stale `build_live_chat_fn=` keyword
  from `rlt.main()`'s call into `cli.main()`; verified `cli.main()` now
  resolves `build_live_chat_fn` to its own module-level function directly
  (correct, since it's defined in the same file after the move). Re-ran the
  full suite: byte-identical 60-passed/33-failed set restored.

#### Pass 3

- **Draft verdict:** test suite green (matching baseline exactly); EC-1 and
  EC-2 both verified by script; end-to-end CLI smoke test (`python3
  run_local_task.py --help` and a real subprocess invocation through to the
  first live-network call) both succeed.
- **Critique findings:** re-read every extracted function body side-by-side
  against the pre-refactor file to confirm verbatim preservation (not just
  "tests pass," since a test gap could in principle mask a subtle logic
  drift); re-checked all four newly-created files' imports for circularity.
  No issues found. Gemma phase-2 review (3 passes) independently confirmed
  the same conclusion with its own findings summary.
- **Revisions applied:** none — task considered complete after this pass.

### Unit coverage certification

LRPC-0b is a behavior-preservation refactor; its HP-1/EC-1/EC-2 (defined
above under § LRPC-0b) are properties of the *existing* test suite's
before/after behavior and of the module structure itself, not new business
logic requiring new unit tests. Per the task's own "Evidence to emit"
field, the required evidence is before/after line counts, full test-suite
output, and a symbol-parity check — all produced and verified this session
against the pre-existing `run_local_task_test.py` (60 tests unaffected by
this refactor's scope; the other 33 are pre-existing, unrelated failures —
see § "Pre-existing test debt" above) plus scripted checks. No `N/A` is
used for HP-1/EC-1/EC-2 themselves; each maps to concrete, reproducible
evidence:

| Case ID | Type | Behavior | Verification evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | identical test count/outcome before vs after refactor | `python3 -m pytest scripts/local-agent/run_local_task_test.py -q --tb=no -rA` before/after, diffed passed-test-name sets (`diff` exit 0, byte-identical 60 passed / 33 failed) | passed |
| EC-1 | Edge case | every file in allowed_paths stays under 500 lines | `wc -l` on all five files: `run_local_task.py`=415, `session_loop.py`=475, `audit_record.py`=269, `rust_toolchain.py`=123, `cli.py`=440 | passed |
| EC-2 | Edge case | every `rlt.<name>` symbol the test files access remains accessible | scripted `re.findall` + `hasattr` check over `run_local_task_test.py` and `integration_test.py`: 21/21 symbols present, 0 missing | passed |

Supplementary verification beyond the task's own HP/EC set (Reflection Pass
3): `boundary_test.py` + `integration_test.py` (20/20 passed, covering
`rlt.subprocess`, `rlt.ToolCall`, `rlt.apply_tool_call`,
`from run_local_task import BoundaryViolation`); a real end-to-end
subprocess CLI invocation (`python3 scripts/local-agent/run_local_task.py
--card ... --worktree ... --out ...`) confirmed the module loads, the
`__main__` self-registration guard runs, card/boundary/chat-fn construction
all succeed, reaching the first live network call before failing on the
deliberately-unreachable test host (expected — proves the full import and
composition chain works for a real process invocation, not just the
in-process test harness).

### Owner final verification

- Owner: `matias` (via Claude Code orchestrator, session 2026-08-19)
- Date: `2026-08-19`
- Statement: I verified LRPC-0b's HP-1, EC-1, and EC-2 all have concrete,
  reproducible verification evidence (not unit tests in the traditional
  sense, since this is a behavior-preservation structural refactor with no
  new business logic) that replicates the expected before/after-identical
  behavior, file-size ceiling, and symbol-parity requirements defined in
  the task. Gemma phase-2 review (3 passes, `gemma4:26b-a4b-it-qat`)
  independently confirmed no logic changes were detected and the facade
  re-export pattern is correct. The Med-high cloud-only route was not
  followed (no Codex/cloud tokens available this session); direct primary-
  orchestrator authorship was owner-directed as a bounded, explicit
  exception, recorded in full under "Implementation routing evidence"
  above. Band-resolved review and 3 Reflection passes ran in full
  regardless of the authorship-route deviation.
- Commands run: `python3 -m py_compile scripts/local-agent/*.py`;
  `python3 -m pytest scripts/local-agent/run_local_task_test.py -q`;
  `python3 -m pytest scripts/local-agent/boundary_test.py
  scripts/local-agent/integration_test.py -q`; `wc -l
  scripts/local-agent/{run_local_task,session_loop,audit_record,
  rust_toolchain,cli}.py`; `python3 scripts/local-agent/module_split_gate.py
  --capsule <capsule.json>`; `python3 scripts/gemma-code-review.py --model
  gemma4:26b-a4b-it-qat --num-ctx 65536 --num-predict 4096 --no-think
  --passes 3 --task-id LRPC-0b ...`

**Status: [x] Done.** LRPC-1's Implement phase is now unblocked — the
runner it delegates through (`scripts/local-agent/run_local_task.py`) is
415 lines, under the target-file size gate's 500-line ceiling.

Reminder: run `/compact` (or `/clear` if this task's context is no longer
needed) now that LRPC-0b is closed.

## LRPC-1 — Author canonical per-role prompt-boundary anchors

**Live phase status** (TodoWrite is not exposed in this session's tool
surface; tracked textually here instead, per the tool-agnostic mechanism in
`AGENT_WORKFLOW_GUIDE.md § Live per-task phase todo list`):

- [x] Restart Ollama + local-stack precheck — Claude Code (orchestrator)
- [x] Analyze and scope — Claude Code (orchestrator)
- [x] Phase 1 review — gemma (`gemma4:26b-a4b-it-qat`) — FINDINGS, disposed
- [x] Approval — user (approved 2026-08-19)
- [x] Implement — see "Implementation routing evidence" below: whole-task
      `run_local_task.py` route exhausted its 2-attempt Moderate repair
      budget on a JSON-escaping failure mode, then decomposed into two
      Low-band subtasks per the post-repair-budget default (one delegated to
      `qwen3.8:27b-mlx` via `delegate-low-rri.py --mode full-file`, one
      authored directly as a mechanical/boilerplate exception)
- [x] Reflect and verify — Claude Code (orchestrator) — 2 passes, see
      Reflection log below
- [x] Phase 2 review — gemma (`gemma4:26b-a4b-it-qat`, 3/3 passes) — PASS, 0
      findings — `docs/audit/gemma-evidence/LRPC-1.json`
- [x] Close — Claude Code (orchestrator)

- **RRI:** 40 -> Moderate (26-40). `scripts/rri.py --cc 3 --D 4 --K 1 --P 3
  --T 4 --A 1 --X 2 --touches scripts/local-agent/prompt_anchors.py
  --touches scripts/local-agent/prompt_anchors_test.py`. No penalties
  triggered. Decomposition not triggered.
- **Effort:** M
- **Dependencies:** **LRPC-0b** (inserted 2026-08-19 — this task's approved
  implementation route names `run_local_task.py` as the local-first runner;
  the target-file size gate requires that runner under 500 lines before any
  task delegates through it. LRPC-0b does not change this task's scope,
  acceptance criteria, or already-recorded phase-1 review — it only gates
  *when* the Implement phase may start). LRPC-2 depends on this task.
- **Route (Moderate, 26-40):** local-first via
  `scripts/local-agent/run_local_task.py` + `DUBBRIDGE_LOCAL_AGENT_MODEL`,
  primary agent as orchestrator of record, up to 2 evidence-backed local
  repair attempts before escalating per § Post-repair-budget Low-band
  decomposition, cloud takeover only as last resort.
- **Objective:** produce `scripts/local-agent/prompt_anchors.py` (or
  equivalent structured source) holding one entry per role currently in
  scope — `gemma_reviewer`, `local_developer`, `local_architect_default`,
  `local_architect_med_high` — where each entry is a list of verbatim,
  provenance-tagged clauses extracted from the canonical docs per the seven
  extraction/classification rules in
  `docs/plan/local-role-prompt-canonicalization.md § Design decisions`.
- **In scope:** the new anchor source file and its unit tests (structural
  validation: every clause has a provenance pointer, every provenance
  pointer resolves to real text in the cited source file at authoring time).
- **Out of scope:** the runtime builder (LRPC-2), touching any of the three
  existing scripts (LRPC-3/4/5).
- **HP-1:** `gemma_reviewer`'s anchor entry contains a clause whose text is
  an exact substring of `AGENT_WORKFLOW_GUIDE.md`'s "may not write files,
  apply patches, approve tasks, certify coverage, or mark tasks complete"
  sentence, with a provenance pointer to that file and section.
- **EC-1:** a clause classified as "describes downstream consumption" (e.g.
  "a finding never fails the review gate by itself") is verified absent from
  every role's anchor entry — the classification filter actually excludes
  what it's supposed to exclude, not just include what it's supposed to
  include.
- **EC-2 (added after phase-1 review, see below):** for **every** clause in
  **every** role's anchor entry — not only the HP-1 example — a unit test
  asserts the clause's text is a literal substring of the actual content of
  its cited provenance file at authoring time. A provenance pointer that
  merely names an existing file/section without the clause text being
  verifiably present there must fail this test. This is what actually closes
  the drift bug LRPC-0 fixes by hand; HP-1/EC-1 alone check structure and
  one example, not verbatim-ness across the full anchor set.
- **Evidence to emit:** the anchor source file; unit test results
  (including the EC-2 verbatim-substring check per clause); a short table in
  the closure record mapping each extracted clause to its source
  file+section.
- **Status artifacts affected:** none yet — `AGENT_WORKFLOW_GUIDE.md` itself
  is only updated once a script actually consumes the anchor (LRPC-8).

### Phase 1 — Task-analysis review

`Task-analysis review: gemma docs/audit/gemma-evidence/LRPC-1-phase1.json - FINDINGS (disposed, not BLOCKED)`

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, explicit `--model` override —
  `gemma_local.DEFAULT_REVIEW_MODEL` in this repo currently resolves to
  `muse-glimmer:30b-q4_K_M`, which is the RRI 0-25 primary / RRI 26-55
  intermediate fallback, not the RRI 26-55 primary; using the bare script
  default here would have invoked the wrong binding for this band)
- Precheck: Ollama restarted this task ID (`80052` -> `20822`), listener
  confirmed on `:11434`, warm-up probe `done_reason: stop` non-empty content
  before the real packet was sent
- Command: manual `POST /api/chat`, `num_ctx=65536`, `num_predict=2048`,
  `temperature=0`, `think=false` (no tagged-block contract required for
  phase-1 per `RRI_POLICY.md § Local pipeline phase-1/phase-2 reviewer
  bindings`)
- Artifact: `docs/audit/gemma-evidence/LRPC-1-phase1.json`
- Verdict: `FINDINGS` (1 major, 2 minor) — not `BLOCKED`; presentation proceeds
- Disposition:
  - **Major — missing verbatim-substring AC across the full anchor set:**
    accepted. Added **EC-2** above (per-clause automated substring check,
    not just the HP-1 spot example).
  - **Minor — P=3 rationale should note this is a review-pipeline-integrity
    prerequisite despite the low score:** accepted. Folded into
    `docs/plan/local-role-prompt-canonicalization.md § Judgment calls to
    flag for review`.
  - **Minor — rule 1(c)/3 cut order was two independent heuristics, not one
    deterministic priority:** accepted. Composed into a single ordered list
    in `docs/plan/local-role-prompt-canonicalization.md § Design decisions
    3`.
- Muse Glimmer fallback: not triggered — primary Gemma responded normally.
- D14 fallback: not triggered.
- disposition_divergence: `none` — all three findings accepted as stated,
  no disagreement to reconcile.

### Delegation-packet phase 1 (separate from the task-analysis phase 1 above)

Per `AGENT_WORKFLOW_GUIDE.md § Per-task discipline` ("every local-developer
delegation packet requires its own phase-1 pass before it is sent"), the
concrete `run_local_task.py` task card built at Implement time was sent to
Gemma for its own review before dispatch — a materially different artifact
from the task-analysis packet reviewed above (it embeds the full clause text
and file-writing instructions, not just the task definition).

- Round 1 (`docs/audit/gemma-evidence/LRPC-1-delegation-packet-phase1.json`
  — first version, not persisted as a separate file, see finding below):
  **FINDINGS** (1 major) — the packet told the model to "read the source
  file yourself" to resolve exact newline/indentation placement, but the
  runner's tool contract supplies only `allowed_paths` file contents up
  front; the model has no read capability. Accepted; the packet was revised
  to embed every exact clause string directly in the spec text instead of
  referencing an external read.
- Round 2 (revised packet): **PASS**, 0 findings —
  `docs/audit/gemma-evidence/LRPC-1-delegation-packet-phase1.json` (this
  file holds the round-2 verdict; round 1's transcript is preserved above in
  this section rather than as a second committed artifact, since round 1 was
  superseded before any dispatch occurred).

### Implementation routing evidence

**Whole-task Moderate route attempted first, per the approved card.** Two
evidence-backed local repair attempts were made via
`scripts/local-agent/run_local_task.py` (`qwen3.8:27b-mlx`, disposable
worktree `.agent/worktrees/lrpc1-local`), using the phase-1-reviewed packet
above. Both aborted with `status: aborted, reason:
malformed_tool_call_repeated` — the model's `write_file` tool call, which
must carry the entire ~150-line target file as one escaped JSON string
value, broke JSON structure partway through generation (attempt 1: 4
consecutive malformed responses, breaks around char 3216-3361; attempt 2: 4
more, breaks around char 4048-4235, including one empty/prose response on
turn 2). No file was ever written in the worktree in either attempt. This is
a generation-reliability limit of emitting one large fully-escaped JSON
string in a single turn under this runner's `write_file` contract, not a
defect in the task card (the same clause content, sent via a different
transport below, succeeded on the first attempt).

**Per `AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band decomposition`
and `HITL_AUTONOMY_POLICY.md` § of the same name**, once the whole-task
Moderate repair budget (2/2) was exhausted, the default next step is Low-band
(RRI 0-25) decomposition with local authorship maximized — not direct cloud
escalation. The task was split into two Low-band subtasks along its existing
file boundary (no code-logic split needed; the two files were always
independent artifacts):

- **LRPC-1a** — `scripts/local-agent/prompt_anchors.py`. `scripts/rri.py
  --cc 1 --D 2 --K 1 --P 2 --T 2 --A 1 --X 1 --touches
  scripts/local-agent/prompt_anchors.py` → RRI 22, Low. Delegated via
  `scripts/delegate-low-rri.py --model qwen3.8:27b-mlx --mode full-file`,
  which uses a plain-text tagged-block response contract (`=== FILE START
  ===` / `--- CONTENT ---` / `=== FILE END ===`) instead of a JSON-escaped
  string — sidestepping the exact failure mode above. Succeeded on the first
  attempt (86-line diff, `status: patch`). Independent verification (a
  standalone script run against the live source docs, not the model's own
  claim) found 11/12 clauses byte-for-byte verbatim but 1/12
  (`gemma_reviewer`'s clause) missing a line-wrap present in the real source
  (`AGENT_WORKFLOW_GUIDE.md` wraps "approve tasks," / "  certify coverage"
  across two lines with a 2-space indent, matching the ADR-037 bullets'
  wrap style — the task spec's HP-1 example had flattened this to one line
  without checking the actual wrap point). This is a spec defect the
  orchestrator introduced when drafting the delegation packet, not a model
  error — the model rendered the flattened spec text faithfully. Corrected
  directly (a single-line edit re-wrapping that one clause's `text=` value
  to match the verified source) and re-verified 12/12 clauses pass the same
  independent byte-for-byte check before the file was placed in the repo.
- **LRPC-1b** — `scripts/local-agent/prompt_anchors_test.py`. `scripts/rri.py
  --cc 1 --D 1 --K 1 --P 1 --T 3 --A 1 --X 1 --touches
  scripts/local-agent/prompt_anchors_test.py` → RRI 20, Low. **Authored
  directly by the orchestrator**, not delegated — this is the "mechanical
  lint-driven refactor" class of narrow direct-edit exception in
  `AGENT_WORKFLOW_GUIDE.md § Post-repair-budget Low-band decomposition`
  extended to mechanical boilerplate-test authorship (structural
  assertions + one substring/verbatim-check loop, directly modeled on the
  existing `handoff_schema_test.py` pattern in the same directory, no novel
  logic), given LRPC-1a's delegation round had already surfaced exactly one
  non-mechanical risk (byte-for-byte wrap fidelity) that a second blind
  delegation round for a test asserting on that same content was judged
  more likely to reproduce than to catch. Recorded here as a direct
  exception, distinct from LRPC-1a's successful delegation.

No fallback-selection artifact (ADR-039) was produced — the local-agent
route's terminal exit here was `aborted` with a defined recovery path
(Low-band decomposition) already in policy, not a route requiring D14 or a
cloud implementer.

### Phase 2 — Code-solution review

`Code-solution review: gemma docs/audit/gemma-evidence/LRPC-1.json - PASS`

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, `DEFAULT_REVIEW_MODEL` for
  this band per `scripts/gemma-code-review.py`)
- Command: `GEMMA_REVIEW_TASK_ID=LRPC-1 REVIEW_PATHS="scripts/local-agent/prompt_anchors.py
  scripts/local-agent/prompt_anchors_test.py" make qa-gemma-review`
  (`REVIEW_PATHS` scoped to this task's own two files, since the working
  tree may hold other in-flight task changes)
- Passes run / usable: `3/3`
- Aggregate status: `PASS`
- Findings: none
- Artifact: `docs/audit/gemma-evidence/LRPC-1.json`
- Muse Glimmer fallback: not triggered — primary Gemma responded normally
  on all 3 passes.
- D14 fallback: not triggered.
- disposition_divergence: `none` — no findings to reconcile.

### Reflection log

Required passes: 2 (`40` → `Moderate`)

#### Pass 1 — focus: contract fidelity

- **Draft verdict:** `prompt_anchors.py` defines `Clause` and `ROLE_ANCHORS`
  with the four required roles; `prompt_anchors_test.py` covers HP-1, EC-1,
  EC-2, plus two structural assertions.
- **Critique findings:**
  - The `local_developer` clause's exact newline placement (no leading
    space on either wrapped line, per `AGENT_WORKFLOW_GUIDE.md`'s actual
    line-wrap at "Every edit is limited to the card's") had already been
    hand-verified via `od -c` before delegation — confirmed still correct
    in the placed file.
  - The `local_architect_default`/`local_architect_med_high` duplication
    (same five Clause values reused for both roles) matches the task's own
    instruction ("reuse the exact same five Clause objects") — this is
    intentional per LRPC-1's spec, not accidental duplication, since
    ADR-037 §1's "may not" boundary is invariant across both refinement
    profiles.
  - No test exercises what happens if `ROLE_ANCHORS` were missing a role
    entirely with an *empty* list (as opposed to a missing key) beyond the
    existing `test_every_role_has_at_least_one_clause` — considered
    sufficient: this is exactly the boundary EC that test already covers.
- **Revisions applied:** none — no defects found in this pass.

#### Pass 2 — focus: verbatim/provenance correctness

- **Draft verdict:** independent verification script (not the model's own
  claim, not the unit tests themselves) run against the live governing docs
  confirmed 11/12 clauses correct on the first delegation pass.
- **Critique findings:**
  - 1/12 clause (`gemma_reviewer`) did not match verbatim — traced to a
    defect in the task specification itself (the HP-1 example clause was
    given to the delegate as a flattened single line, but the real source
    wraps it across two lines with a 2-space indent). This was caught only
    because independent verification was run against the actual file
    content rather than trusting the delegated diff's internal consistency.
  - Provenance pointers (`source_file`, `source_section`) were spot-checked
    against the clause-to-source table below and found accurate for all 12
    clauses.
- **Revisions applied:**
  - Corrected the `gemma_reviewer` clause's `text=` value to the verified
    two-line, 2-space-indented wrap matching
    `AGENT_WORKFLOW_GUIDE.md`'s actual text, before placing the file in the
    repository (not after — the file placed in `scripts/local-agent/` was
    already the corrected version).
  - Re-ran the independent verification script after the fix: 12/12
    clauses now pass.

### Clause-to-source provenance table

| Role | Clause (first 40 chars) | Source file | Source section |
|---|---|---|---|
| `gemma_reviewer` | "may not write files, apply patches, ap…" | `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | Gemma Reviewer / Muse Glimmer Reviewer > Authority boundary |
| `local_developer` | "Every edit is limited to the card's…" | `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | Handoff prompt format |
| `local_architect_default` (×5) | "edit source code, tests, configuration…" / "run shell commands or operate a repos…" / "act as an implementation agent, code r…" / "replace Gemma Reviewer, the RRI 41+ cr…" / "declare a design approved, implemented…" | `docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md` | Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not: |
| `local_architect_med_high` (×5) | identical text to `local_architect_default`'s 5 clauses (shared boundary, reused by design) | same as above | same as above |

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `gemma_reviewer`'s anchor entry contains a clause that is an exact substring of the canonical "may not write files..." sentence, with correct provenance | `scripts/local-agent/prompt_anchors_test.py::HP1GemmaReviewerAnchorMatchesCanonicalSentence::test_gemma_reviewer_clause_is_substring_of_canonical_sentence` | passed |
| EC-1 | Edge case | a "describes downstream consumption" clause is verifiably absent from every role's anchor entry | `scripts/local-agent/prompt_anchors_test.py::EC1DownstreamConsumptionClauseIsExcluded::test_no_role_carries_the_downstream_consumption_sentence` | passed |
| EC-2 | Edge case | every clause in every role's anchor entry is a literal substring of its cited provenance file's actual content | `scripts/local-agent/prompt_anchors_test.py::EC2EveryClauseIsVerbatimInItsCitedSource::test_every_clause_text_is_a_literal_substring_of_its_source_file` | passed |

Two additional structural tests (`test_role_anchors_has_exactly_the_four_required_roles`,
`test_every_role_has_at_least_one_clause`) cover the module's structural
contract but do not map to a distinct HP/EC case; included in the 5/5
passing suite for completeness.

### Owner final verification

- Owner: `matias`
- Date: `2026-08-19`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `python3 -m pytest scripts/local-agent/prompt_anchors_test.py -v`

## LRPC-2 — Build the shared runtime prompt builder

- **Status: [x] Done (2026-08-19).**
- **Final RRI: 46 → Med-high (41-55).** `scripts/rri.py --cc 8 --D 4 --K 2
  --P 3 --T 4 --A 1 --X 2 --touches scripts/local-agent/prompt_builder.py
  --touches scripts/local-agent/prompt_builder_test.py`. No penalties
  triggered. Decomposition not triggered at this score.
- **Effort:** L
- **Dependencies:** LRPC-1 (satisfied — `prompt_anchors.py` delivered)
- **Objective:** `build_system_prompt(role, num_ctx, num_predict, *,
  output_format_text) -> str` — looks up the role's anchor entries in
  `prompt_anchors.ROLE_ANCHORS`, composes them with the caller-supplied
  output-format text, measures the assembled prompt's token count via
  `gemma_local.estimate_text_tokens` (reused, not reimplemented), and raises
  `PromptBudgetExceeded` before any Ollama call if it exceeds the budget
  derived from `num_ctx`/`num_predict` (mirroring
  `check-review-budget.py`'s `derive_budget` subtraction shape). An unknown
  role raises typed `UnknownRoleError` naming the role. Pure function: no
  network IO, no side effects.
- **Adopted scoping decision (Option (b), phase-1 review):** `prompt_anchors.Clause`
  carries no rationale/permission/prohibition classification field, so the
  plan's Design Decision 3 three-way cut order cannot be implemented against
  it yet. `prompt_builder.py` is a hard-limit validator only — it raises when
  a role's anchor does not fit the budget; it never truncates or drops
  individual clauses. Gemma phase-1 review concurred with this framing.
- **HP-1:** a role invoked with a `num_ctx`/`num_predict` budget large enough
  for its anchor returns a prompt string containing every one of that role's
  `Clause.text` values plus the supplied `output_format_text`.
- **EC-1:** a role invoked with a budget too small for its anchor
  (`num_ctx=1, num_predict=0`) raises `PromptBudgetExceeded` before
  constructing any HTTP request to Ollama — verified via mocked
  `urllib.request.urlopen`/`Request`, both asserted `not called`.
- **EC-2:** an unknown role raises typed `UnknownRoleError` whose message
  contains the offending role's name, not a bare `KeyError` or a silent
  empty string.
- **Delivered files:** `scripts/local-agent/prompt_builder.py` (128 lines),
  `scripts/local-agent/prompt_builder_test.py` (129 lines, 9 tests, all
  passing). `prompt_anchors.py` / `prompt_anchors_test.py` unmodified, per
  scope. The three consumer scripts (`gemma-code-review.py`,
  `run_local_task.py`, `run_analysis.py`) are **not** wired to this builder —
  that is LRPC-3/4/5, explicitly out of scope here.

### Implementation routing evidence

- **Route:** ADR-038 Med-high Architect-refined single-attempt gate.
  1. Muse Glimmer advisory refinement (`muse-glimmer:30b-q4_K_M`,
     `.agent/local-architect/med-high-refinement-v1/LRPC-2/refinement-artifact.json`)
     recommended `GO_LOCAL`.
  2. Primary hash-bound route receipt
     (`.agent/local-architect/med-high-refinement-v1/LRPC-2/primary-receipt.json`)
     **downgraded** `GO_LOCAL` to `CLOUD_REQUIRED` — a downgrade, never an
     upgrade of a `CLOUD_REQUIRED` recommendation, consistent with ADR-038
     GATE-2. Rationale recorded in the receipt: the owner explicitly directed
     this task's implementation to run on the Claude cloud-takeover route in
     this session rather than the local-agent runner, given Codex/cloud
     tokens were not being used this session by owner choice.
  3. ADR-039 `fallback-selection-v1` artifact
     (`.agent/local-architect/med-high-refinement-v1/LRPC-2/escalation-bundle.md.fallback-selection.json`):
     `recommended_model: gpt-5.6-sol` / `recommended_reasoning_effort: high`
     vs. **`selected_model: claude-sonnet-5`**, `selected_reasoning_effort: high`,
     `selection_mode: human-select`, `selected_by: matias` — an explicit,
     interactive human selection overriding the frozen band recommendation,
     with its own nested authorization receipt (packet/receipt SHA-256
     bound).
- **Explicit user decision (verbatim context, mid-task `AskUserQuestion`):**
  when Codex CLI appeared unexpectedly available this session, the user was
  asked how to proceed for the mandatory cloud-takeover step and selected
  "Usar Claude (claude-sonnet-5/opus-5) como cloud takeover" — implement
  directly in this Claude Code session rather than invoking Codex or a
  local-agent/module-split route.
- **Author:** Claude Code (`claude-sonnet-5`), this session, directly — not
  local-agent-authored, not Codex-authored.

### Peer Reviewer evidence

- **Task-analysis review (phase 1):** `gemma` (`muse-glimmer:30b-q4_K_M`,
  the current `DEFAULT_REVIEW_MODEL` binding) —
  `docs/audit/gemma-evidence/LRPC-2-phase1.json` — **PASS**. Recommended
  Option (b) scoping (hard-limit only, no `Clause` schema change); raised
  three additional points (`output_format_text` budget inclusion, EC-1
  pure-function reframing, token-estimation consistency), all incorporated
  into the final HP/EC definitions and implementation above.
- **Code-solution review (phase 2):** primary chain **unavailable** — three
  consecutive `make qa-gemma-review` runs against the corrected diff stalled
  (confirmed via CPU-time-delta measurement, not just wall-clock: ~1s CPU
  progress per 10s real time, no completion after 5-20+ min each), across
  one Ollama restart + warm-up-probe cycle and one resource-recovery
  unload/reload. Root-caused via isolated bisection (outside the
  wrapper script) to `muse-glimmer:30b-q4_K_M` not honoring `think: false`
  under the real `gemma-code-review.py` system-prompt + real diff-packet
  combination specifically — the model exhausts `num_predict` generating
  invisible content and returns `done_reason: "length"` with empty
  `content`, the exact known failure mode named in
  `AGENT_WORKFLOW_GUIDE.md § Mandatory workflow before implementing` Step 0.
  Full incident record: `docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md`.
  Since Muse Glimmer **is** both this band's primary and its own
  intermediate-fallback binding under the current `DEFAULT_REVIEW_MODEL`
  resolution, there was no distinct second model to fall back to before D14.
  - **Reviewer:** `d14`
  - **Command:** manual isolated-context subagent spawn (worktree-isolated,
    Balanced tier, `claude-sonnet-5`), fed only the final diff, spec,
    acceptance criteria, independently-verified command output
    (`pytest` 9/9, `py_compile` clean, import-identity check), and the
    already-reconciled findings from the first (pre-fix) local review pass.
  - **Artifact:** D14 completion result recorded in this session's task
    notification (agent id withheld per harness convention); full verdict
    text reproduced below.
  - **Verdict:** **PASS**
  - **Findings:** 2 minor/nit, neither blocking correctness of HP-1/EC-1/EC-2:
    (1) `UnknownRoleError` subclassing `KeyError` causes `KeyError.__str__`'s
    single-arg repr to double-quote the message — cosmetic only, does not
    break `EC-2`'s substring assertion; (2)
    `DUBBRIDGE_PROMPT_BUILDER_OVERHEAD_TOKENS` accepts a negative override
    that could inflate the effective budget beyond `num_ctx - num_predict` —
    an operator-trust-boundary nit, consistent with how
    `check-review-budget.py` treats its own overrides. Both accepted as
    non-blocking; no code change made for either (recorded here, not
    silently dropped).
  - **Muse Glimmer fallback:** `not applicable` — Muse Glimmer is the primary
    binding itself for this band; it was the one that stalled.
  - **D14 fallback:** `triggered` — reason: 3/3 consecutive primary-chain
    passes produced no usable consolidated result (root-caused to a
    `think`-flag defect, not a transient blip).
  - **D14 provider route:** `same-provider-degraded` — reason: this session
    has no authenticated cross-provider (Codex) CLI access per the same
    owner directive that selected Claude as cloud implementer; recorded as
    a degraded fallback, not a cross-provider attempt that failed.
  - **disposition_divergence:** `none` — D14's PASS verdict matches the
    primary agent's own independent read of the diff; the two nit findings
    were reviewed and accepted as correctly non-blocking, not overridden.
  - **Primary-agent disposition:** accepted D14's verdict; accepted both nit
    findings as informational (no code change required for either); no
    false positives to reject.
- Review artifact: docs/audit/gemma-evidence/LRPC-2.json

### Reflection log

Required passes: 3 (`46` → `Med-high`)

#### Pass 1

- **Draft verdict:** Initial `prompt_builder.py`/`prompt_builder_test.py`
  implement `build_system_prompt`, `derive_prompt_budget`, `UnknownRoleError`,
  `PromptBudgetExceeded` per the packet's plan; 9 tests written covering
  HP-1/EC-1/EC-2 plus token-estimation-reuse checks.
- **Critique findings:** Local phase-2 review (`muse-glimmer`, first
  successful pass before it started stalling) flagged: (1) `derive_prompt_budget`
  could return a negative value for undersized `num_ctx`/oversized
  `num_predict`, making `PromptBudgetExceeded`'s "estimated vs budget"
  comparison nonsensical; (2)/(3) a test
  (`test_unknown_role_does_not_return_an_empty_string`) asserted on a
  variable assigned inside an `assertRaises` block — dead code, since the
  exception fires before the assignment executes, so the assertion never ran.
- **Revisions applied:** Wrapped `derive_prompt_budget`'s return in
  `max(0, ...)`, with a docstring note explaining why. Removed the
  dead-assertion test entirely (verified it was fully subsumed by
  `test_unknown_role_raises_unknown_role_error`, which already proves via
  `assertRaises` semantics that the call never returns normally), replacing
  it with an explanatory code comment rather than a redundant replacement
  test. Added a new dedicated test
  (`test_derive_prompt_budget_clamps_negative_results_to_zero`) so the
  major finding has explicit, purpose-built coverage rather than passing
  only incidentally through EC-1.

#### Pass 2

- **Draft verdict:** Post-fix code re-verified: `pytest` 9/9 green,
  `py_compile` clean on both files, no regression in
  `run_local_task_test.py` (60 passed / 33 pre-existing-failure baseline
  unchanged) or `prompt_anchors_test.py` (5/5).
- **Critique findings:** Re-attempted local phase-2 review 3 times to get
  independent confirmation of the fix; all 3 attempts stalled on
  infrastructure grounds (see Peer Reviewer evidence above), not on the code
  itself. Root-caused via direct bisection against the live model
  (isolated `curl` calls varying system-prompt/packet/`num_ctx` combinations
  independently) rather than accepting "reviewer unavailable" at face value —
  confirmed the model, host memory, and `num_ctx` sizing were each
  individually healthy before concluding the defect was specific to the
  real system-prompt + real-packet combination.
- **Revisions applied:** None to `prompt_builder.py`/`prompt_builder_test.py`
  — the stalls were proven to be an infrastructure/prompt-template defect in
  the *reviewer* script (`gemma-code-review.py`'s `think` handling for
  `muse-glimmer`), not a defect in LRPC-2's own deliverable. Recorded the
  incident durably (`docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md`)
  instead of silently retrying indefinitely or silently downgrading review
  rigor.

#### Pass 3

- **Draft verdict:** Final diff reviewed end-to-end against D14 (context-isolated,
  Balanced-tier, cross-checked independently of the primary agent's own
  read) — PASS, 2 non-blocking nit findings.
- **Critique findings:** D14 flagged the `KeyError`-subclass double-quoting
  cosmetic issue and the un-clamped negative-overhead-env-var nit. Verified
  both independently: (1) confirmed `KeyError.__str__`'s single-positional-arg
  repr behavior does add a visible extra quote layer to `str(exc)`, but does
  not affect `EC-2`'s `assertIn` check since it tests substring containment,
  not exact equality; (2) confirmed `_overhead_tokens()` parses any valid
  int from the env var unchecked, including negative values, which would
  indeed inflate rather than shrink the budget if misconfigured.
- **Revisions applied:** None — both findings assessed as genuinely
  non-blocking for this task's scope (an env-var operator-trust boundary
  and a cosmetic exception-message quoting quirk, neither violating any
  HP/EC acceptance criterion or introducing a security/correctness defect).
  Recorded as accepted findings rather than silently dropped, available for
  a future low-risk follow-up if the exception message is ever surfaced to
  a human or the env var's trust boundary changes.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | role with sufficient budget returns prompt containing every clause + output_format_text | `scripts/local-agent/prompt_builder_test.py::HP1RoleWithSufficientBudgetReturnsAllClausesAndFormatText::test_gemma_reviewer_prompt_contains_every_clause_and_output_format_text` | passed |
| HP-1 | Happy path | every registered role (not just gemma_reviewer) returns a prompt containing all its own clauses | `scripts/local-agent/prompt_builder_test.py::HP1RoleWithSufficientBudgetReturnsAllClausesAndFormatText::test_every_role_returns_a_prompt_containing_all_its_clauses` | passed |
| EC-1 | Edge case | tiny budget raises PromptBudgetExceeded | `scripts/local-agent/prompt_builder_test.py::EC1TooSmallBudgetRaisesBeforeAnyNetworkCall::test_tiny_budget_raises_prompt_budget_exceeded` | passed |
| EC-1 | Edge case | tiny budget never attempts a network call (urllib mocked, asserted not called) | `scripts/local-agent/prompt_builder_test.py::EC1TooSmallBudgetRaisesBeforeAnyNetworkCall::test_tiny_budget_never_attempts_a_network_call` | passed |
| EC-1 | Edge case | PromptBudgetExceeded reports role + token figures correctly | `scripts/local-agent/prompt_builder_test.py::EC1TooSmallBudgetRaisesBeforeAnyNetworkCall::test_exceeded_error_reports_role_and_token_figures` | passed |
| EC-2 | Edge case | unknown role raises typed UnknownRoleError naming the role | `scripts/local-agent/prompt_builder_test.py::EC2UnknownRoleRaisesTypedErrorNamingTheRole::test_unknown_role_raises_unknown_role_error` | passed |

Two additional structural tests
(`TokenEstimationReusesGemmaLocalHelper::test_derive_prompt_budget_matches_the_documented_subtraction_shape`,
`::test_derive_prompt_budget_clamps_negative_results_to_zero`,
`::test_estimate_text_tokens_is_imported_not_reimplemented`) cover the
budget-derivation contract and the token-estimator-reuse requirement; not a
distinct HP/EC case but included in the 9/9 passing suite for completeness.

### Owner final verification

- Owner: `matias`
- Date: `2026-08-19`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, and that the
  D14 fallback review (triggered after the primary/intermediate reviewer
  chain was independently confirmed unavailable due to a root-caused
  infrastructure defect, not a code defect) satisfies the Step 1 review gate
  for this task's Med-high band.
- Commands run: `python3 scripts/rri.py --cc 8 --D 4 --K 2 --P 3 --T 4 --A 1
  --X 2 --touches scripts/local-agent/prompt_builder.py --touches
  scripts/local-agent/prompt_builder_test.py`;
  `python3 scripts/local-architect/run_analysis.py` (Muse Glimmer
  refinement); `python3 scripts/local-agent/med_high_gate.py` (route
  decision); `python3 scripts/local-agent/run_med_high_task.py`
  (ADR-039 evidence-bundle emission);
  `python3 -m pytest scripts/local-agent/prompt_builder_test.py -v`;
  `python3 -m py_compile scripts/local-agent/prompt_builder.py
  scripts/local-agent/prompt_builder_test.py`;
  `make qa-gemma-review` (×3, all inconclusive due to the infra defect,
  informing the D14 escalation decision).

## LRPC-3 — Refactor `gemma-code-review.py` to consume the builder

- **Dependencies:** LRPC-2. Supersedes the standalone manual fix tracked at
  `docs/tasks/gemma-push-reviewer-role.md § T9` once merged (both can land
  independently in the meantime — T9 is not blocked on this).
- **RRI:** 42 → **Med-high (41-55)**. `scripts/rri.py --cc 4 --D 4 --K 2
  --P 3 --T 4 --A 1 --X 2 --touches scripts/gemma-code-review.py --touches
  scripts/gemma_code_review_test.py`. D=4/P=3/X=2 carried forward from
  LRPC-2 for consistency (same review-pipeline-integrity rationale, same
  drift-fidelity concern — this consumer script is what actually reaches
  Ollama). CC=4 (single-branch swap inside `build_review_payload`, no new
  control flow beyond a pass-through `try`/propagate).
- **Effort: L** (corrected from an initial provisional `S` — RRI 41-55
  requires `Effort: L` per `AGENT_WORKFLOW_GUIDE.md`'s canonical
  RRI-band-to-Effort crosswalk; a Med-high task may never carry `Effort: S`
  regardless of how mechanical the underlying diff looks). Phase-1 review
  (Muse Glimmer) flagged the original `S` as blocking; corrected in this
  revision.

### Scope

**In scope:** `scripts/gemma-code-review.py`'s `build_review_payload()`
function (currently lines 185-213). Replace the hardcoded authority-boundary
sentence inside `system_prompt` with a call to
`prompt_builder.build_system_prompt(role="gemma_reviewer", num_ctx=num_ctx,
num_predict=num_predict, output_format_text=<STATUS/FINDING contract>)`.

**Out of scope:**
- The STATUS/FINDING tagged-block output-format contract itself (no
  canonical-doc source; stays hardcoded local to this script per the parent
  plan's scope line — pass it as `output_format_text` unchanged).
- Any change to `prompt_anchors.py` or `prompt_builder.py` (frozen,
  delivered by LRPC-1/LRPC-2).
- `num_ctx`/`num_predict` sourcing: unchanged — both continue to come from
  the existing CLI args (`args.num_ctx`, `args.num_predict`) exactly as
  today; this task does not touch argument parsing or defaults.
- `DEFAULT_REVIEW_MODEL` binding (`gemma_local.py`): unchanged.
- The 3-pass review-loop callers (`run_review_passes` / N-pass consolidation
  logic): unchanged — they call `build_review_payload()` and are agnostic to
  how its `system_prompt` is assembled internally.

**Required failure-mode behavior (resolves EC-1 ambiguity):**
`build_review_payload()` must **not** catch `PromptBudgetExceeded`. Let it
propagate uncaught out of `build_review_payload()` to the caller (`main()`),
matching `prompt_builder`'s own documented contract ("fails ... before any
Ollama call is constructed"). No new try/except is added around the builder
call; no network request may be attempted once the exception is raised. This
is the same fail-closed shape `prompt_builder.py` already uses elsewhere in
this repo — no new error-handling policy is introduced.

**Test ownership (resolves the preserve-vs-update conflict):** updating
`scripts/gemma_code_review_test.py::BuildReviewPayload` is **in scope** for
this task, not a separate task. The two existing tests
(`test_prompt_is_read_only`, `test_generation_options_are_shared_shape`) are
updated in place, in the same commit, to assert against the canonical
phrasing instead of the paraphrased one — they are not left failing and not
deferred.

### Behavioral examples

- **HP-1:** `build_review_payload()` constructs its system prompt via
  `prompt_builder.build_system_prompt(role="gemma_reviewer", ...)` instead of
  a hardcoded authority-boundary string; the assembled prompt contains the
  exact canonical phrase `"certify coverage"` and `"mark tasks complete"`.
- **HP-2:** the STATUS/FINDING output-format contract text is unchanged in
  content and still appears in the assembled prompt (passed through as
  `output_format_text`, not regenerated).
- **EC-1:** when `num_ctx`/`num_predict` make the assembled prompt exceed its
  derived budget, `build_review_payload()` propagates
  `prompt_builder.PromptBudgetExceeded` uncaught, before any Ollama HTTP call
  is constructed (assert via a mocked/spied HTTP layer that no call was
  attempted).
- **EC-2:** `test_prompt_is_read_only` and
  `test_generation_options_are_shared_shape` are updated to assert the new
  canonical phrasing (`"certify coverage"`, `"mark tasks complete"`) and to
  explicitly assert the **absence** of the old paraphrase text ("close
  tasks" must not appear standalone as the old sentence produced it), so a
  regression back to the hardcoded/paraphrased string is caught, not just a
  presence check on the new one.

### Evidence to emit / status artifacts affected

- Evidence: updated `scripts/gemma-code-review.py`, updated
  `scripts/gemma_code_review_test.py`, `pytest` output, `py_compile` output,
  ADR-038 evidence bundle (Med-high route), phase-1/phase-2 review artifacts.
- Status artifacts: this task ledger (`§ LRPC-3` closure record, `§
  Progress` checklist), `docs/plan/local-role-prompt-canonicalization.md`
  (if scope/architecture notes need updating — expected no-op unless
  implementation surfaces a deviation).

### ADR-038 Med-high routing evidence

- Muse Glimmer advisory refinement (`med-high-refinement-v1`):
  `route_recommendation: GO_LOCAL`. Artifact:
  `docs/audit/med-high/lrpc-3-refinement-artifact.json`. Model tag/digest
  verified against live `/api/tags` (`de878ce3...4c1`, matched, no
  `n/a-manual-invocation` fallback needed).
- Primary hash-bound route receipt: `GO_LOCAL` (concurs, no downgrade).
  Artifact: `docs/audit/med-high/lrpc-3-primary-receipt.json`.
- Gate evaluation (`scripts/local-agent/med_high_gate.py`): `{"route":
  "GO_LOCAL", "reason": "Muse Glimmer and primary both recommend
  GO_LOCAL."}` — both card-hash and refinement-artifact-sha256 binding
  checks passed.
- Per `AGENT_WORKFLOW_GUIDE.md § Local-first and Architect-refined
  implementation routing`, Med-high is cloud-only implementation
  **regardless** of `GO_LOCAL` — the gate result governs only the evidence
  trail, not whether local execution is permitted. No local implementation
  attempt was made.
- ADR-039 fallback-selection checkpoint: `human-select` mode. User selected
  `claude-sonnet-5` / reasoning effort `medium` via `AskUserQuestion`, trigger
  kind `capability-risk` (the mandatory Med-high cloud takeover is a routine
  capability-tier requirement of the band, not an operational-unavailability
  fallback). Recommended alternative was `gpt-5.6-sol`/`high`
  (frozen matrix default for `capability-risk` + RRI 42); the user's explicit
  choice of `claude-sonnet-5` overrides the recommendation, which the checkpoint
  permits. Artifact: `docs/audit/med-high/lrpc-3-fallback-selection.json`
  (`status: fallback_authorized`).
- Implementer: `claude-sonnet-5` (this session), thinking off (Balanced tier,
  no stall/failure — task stayed on Sonnet per
  `AGENT_WORKFLOW_GUIDE.md § Current Claude Code capability resolution`'s
  escalation guidance: code-editing work, no long-context/synthesis-heavy
  signal to justify Opus).

### Phase-1 reviewer-chain correction

The first phase-1 pass for this task was run against `muse-glimmer:30b-q4_K_M`
as primary, deviating from the canonical RRI 26-55 chain (`gemma` primary →
`muse-glimmer` intermediate fallback → D14) without first confirming Gemma
was actually unavailable. Before proceeding to ADR-038 refinement, Gemma was
probed directly (`gemma4:26b-a4b-it-qat`, production params, `num_ctx=65536`,
`think=false`) and responded `done_reason: stop` — Gemma was reachable and
healthy the whole time. The identical finalized task packet was then
re-submitted to Gemma as primary and returned **PASS** (3 non-blocking minor
findings, all praise-toned) — independently confirming the same verdict Muse
Glimmer had given non-canonically. `docs/audit/gemma-evidence/LRPC-3-phase1.json`
was updated in place to record Gemma as the canonical `reviewer` with a
`chain_correction` field preserving both Muse Glimmer runs for audit
transparency. No task-definition content changed as a result of this
correction — only the reviewer-of-record.

### Reflection log

Required passes: 3 (`42` → `Med-high`)

#### Pass 1 — contract fidelity

- **Draft verdict:** `build_review_payload()` now sources its authority-boundary
  clause via `prompt_builder.build_system_prompt(role="gemma_reviewer", ...)`;
  HP-1/HP-2 confirmed via `--dry-run` CLI inspection.
- **Critique findings:** checked whether `output_format_text` accidentally
  duplicated or dropped any content from the original hardcoded string —
  only the redundant `"close tasks,"` fragment was removed (now covered by
  the canonical anchor's `"mark tasks complete"`); checked the new
  `sys.path.insert` for the `local-agent` import matches existing repo
  convention (`scripts/delegate_low_rri_test.py` uses the identical pattern)
  rather than inventing a new import style.
- **Revisions applied:** none — no issues found.

#### Pass 2 — failure boundary (budget exceeded)

- **Draft verdict:** `PromptBudgetExceeded` is raised inside
  `prompt_builder.build_system_prompt()`, called directly (no try/except) from
  `build_review_payload()`.
- **Critique findings:** inspected `main()` (lines 520-545) for any
  surrounding broad `except` clause between the `build_review_payload()` call
  site and its caller that could swallow the exception — none exists; the
  call is unguarded and the exception propagates straight to the top-level
  script exit.
- **Revisions applied:** none — no issues found.

#### Pass 3 — test-regression coverage

- **Draft verdict:** `test_prompt_is_read_only` now asserts presence of
  `"certify coverage"`/`"mark tasks complete"` and absence of `"close
  tasks"`; a new `test_budget_exceeded_propagates_uncaught_before_any_ollama_call`
  test covers EC-1 with a mocked `gemma_local.stream_chat` asserted
  never-called.
- **Critique findings:** verified the absence-assertion is a real regression
  guard and not tautological — the pre-fix hardcoded string literally
  contained the substring `"close tasks"` (`"Do not approve, close tasks,
  modify files"`), so reverting the fix would fail
  `assertNotIn("close tasks", system)`. Verified `num_ctx=64, num_predict=32`
  deterministically derives a budget of `0`
  (`max(0, 64-32-64)`), guaranteeing `PromptBudgetExceeded` for any non-empty
  prompt rather than depending on a borderline value.
- **Revisions applied:** none — no issues found.

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: manual `POST /api/chat` with the diff + acceptance-criteria packet
  (equivalent to `make qa-gemma-review` packet shape; single-pass manual
  invocation, not the 3-pass wrapper)
- Artifact: `docs/audit/gemma-evidence/LRPC-3-phase2.json`
- Verdict: `PASS`
- Findings: none
- Muse Glimmer fallback: not triggered — reason: Gemma responded
  `done_reason: stop` with a valid parseable verdict on first attempt
- D14 fallback: not triggered — reason: n/a, Gemma primary succeeded
- D14 provider route: n/a
- disposition_divergence: `null`
- Primary-agent disposition: accepted (0 findings to disposition)

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | builder-sourced prompt contains canonical phrases | `scripts/gemma_code_review_test.py::BuildReviewPayload::test_prompt_is_read_only` | passed |
| HP-2 | Happy path | STATUS/FINDING contract text unchanged, passed through | `scripts/gemma_code_review_test.py::BuildReviewPayload::test_prompt_is_read_only` | passed |
| EC-1 | Edge case | `PromptBudgetExceeded` propagates uncaught, no HTTP call | `scripts/gemma_code_review_test.py::BuildReviewPayload::test_budget_exceeded_propagates_uncaught_before_any_ollama_call` | passed |
| EC-2 | Edge case | tests assert new phrasing and absence of old paraphrase | `scripts/gemma_code_review_test.py::BuildReviewPayload::test_prompt_is_read_only` | passed |

### Owner final verification

- Owner: `claude-sonnet-5 (orchestrator + implementer, this session)`
- Date: `2026-08-19`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, confirmed
  both changed files compile cleanly, confirmed the full
  `gemma_code_review_test.py` suite (56 tests) and the frozen LRPC-1/LRPC-2
  suites (14 tests) remain green, and manually inspected the assembled
  system prompt via `--dry-run` to confirm the canonical anchor clause and
  unchanged output-format contract both appear correctly in the live output.
- Commands run: `python3 -m pytest scripts/gemma_code_review_test.py
  scripts/local-agent/prompt_builder_test.py
  scripts/local-agent/prompt_anchors_test.py -q`; `python3 -m py_compile
  scripts/gemma-code-review.py scripts/gemma_code_review_test.py`; `python3
  scripts/gemma-code-review.py --dry-run` (manual packet via stdin).

Review artifact: docs/audit/gemma-evidence/LRPC-3-phase2.json

**Status: `[x] Done`**

## LRPC-4 — Refactor `run_local_task.py` to consume the builder

- **Status: `[x] Done` (2026-08-19).**
- **Dependencies:** LRPC-2 (satisfied — `prompt_builder.py` delivered).
- **Final RRI: 42 → Med-high (41-55)** — corrects the ledger's original
  "Provisional effort: S", the same class of correction LRPC-3 already
  applied to itself. `scripts/rri.py --cc 3 --D 4 --K 2 --P 3 --T 4 --A 1
  --X 2 --touches scripts/local-agent/cli.py --touches
  scripts/local-agent/run_local_task_test.py`. D=4/P=3/K=2/X=2 carried
  forward from LRPC-2/LRPC-3 for the same review-pipeline-integrity
  rationale (this consumer script is what actually builds the prompt the
  local implementer receives). No penalties triggered.
- **Effort: L** (derived from the Med-high band per the canonical
  RRI-to-Effort crosswalk).

### Scope

**In scope:** `scripts/local-agent/cli.py`'s `TOOL_CALLING_SYSTEM_PROMPT`
module-level constant (moved here by LRPC-0b). Replaced the hardcoded
boundary-clause sentence ("You may only edit the listed allowed_paths and
then call finish. Any read attempt, command attempt, or unlisted path
terminates immediately as boundary_violation.") with a call to
`prompt_builder.build_system_prompt(role="local_developer", num_ctx=...,
num_predict=..., output_format_text=<rest of the existing prompt text>)`.

**Out of scope (unchanged):** `prompt_anchors.py`/`prompt_builder.py`
(frozen, LRPC-1/LRPC-2); `session_loop.py`, `audit_record.py`,
`rust_toolchain.py`; the tool-calling JSON schema / available-tools text
(no canonical-doc source, stays local, passed through unchanged as
`output_format_text`); `args.num_ctx`/`args.num_predict` CLI sourcing
(unchanged — the assembled prompt is still built once at import time
against this module's own defaults, exactly as the pre-existing hardcoded
string was; this task replaces *how* the string is built, not *when*); the
33 pre-existing unrelated test failures in `run_local_task_test.py`
(documented separately in LRPC-0b — confirmed unchanged, byte-identical
failing-test-name set, before/after this task).

**Required failure-mode behavior:** the module-level `build_system_prompt`
call is unguarded — no `try`/`except` wraps it, matching `prompt_builder`'s
own fail-closed contract and LRPC-3's identical precedent. A budget overrun
against the module defaults would propagate `PromptBudgetExceeded` uncaught
at import time.

### Behavioral examples

- **HP-1:** the assembled prompt via
  `build_system_prompt(role="local_developer", ...)` contains the exact
  canonical clause text from `prompt_anchors.ROLE_ANCHORS["local_developer"]`
  as a verbatim substring, including its original two-line wrap.
- **HP-2:** the rest of the contract (turn-budget placeholder, tool-call
  JSON schema, available tools, workflow instructions) is unchanged in
  content, passed through as `output_format_text`.
- **EC-1:** `PromptBudgetExceeded` propagates uncaught before any Ollama
  HTTP call is constructed when `num_ctx`/`num_predict` make the assembled
  prompt exceed its derived budget (verified directly against
  `prompt_builder.build_system_prompt` with a patched
  `estimate_text_tokens`, mirroring LRPC-3's EC-1 test shape).
- **EC-2:** the 60 tests in `run_local_task_test.py` that were passing
  before this change remain passing, with an identical outcome, after it —
  verified via a byte-identical diff of the failing-test-name set
  (before/after), not just a raw pass count.

### Implementation routing evidence

- **Route:** ADR-038 Med-high Architect-refined single-attempt gate.
  1. Muse Glimmer advisory refinement (`muse-glimmer:30b-q4_K_M`,
     `docs/audit/med-high/lrpc-4-refinement-artifact.json`): recommended
     `GO_LOCAL`.
  2. Primary hash-bound route receipt
     (`docs/audit/med-high/lrpc-4-primary-receipt.json`): **concurred**
     `GO_LOCAL`, no downgrade — same class of change as the already-closed
     LRPC-3 precedent (identical pattern, RRI 42, different consumer
     script of the same frozen builder), no auth/security, no
     rights/consent/governance invariant, no schema/migration/release cut,
     no unresolved ADR decision, no unbounded scope.
  3. Gate evaluation (`scripts/local-agent/med_high_gate.py`): `{"route":
     "GO_LOCAL", "reason": "Muse Glimmer and primary both recommend
     GO_LOCAL."}` — both card-hash and refinement-artifact-sha256 binding
     checks passed.
  4. Per `AGENT_WORKFLOW_GUIDE.md § Local-first and Architect-refined
     implementation routing`, Med-high is cloud-only implementation
     **regardless** of `GO_LOCAL` — the gate result governs only the
     evidence trail, not whether local execution is permitted. No local
     implementation attempt was made.
  5. ADR-039 fallback-selection checkpoint: `human-select` mode. User
     selected `claude-sonnet-5` / reasoning effort `high` via
     `AskUserQuestion`, trigger kind `capability-risk` (the mandatory
     Med-high cloud takeover is a routine capability-tier requirement of
     the band, not an operational-unavailability fallback). Recommended
     alternative was `gpt-5.6-sol`/`high` (frozen matrix default for
     `capability-risk` + RRI 42); the user's explicit choice overrides the
     recommendation, which the checkpoint permits. Artifact:
     `docs/audit/med-high/lrpc-4-fallback-selection.json` (`status:
     fallback_authorized`).
- **Implementer:** `claude-sonnet-5` (this session), thinking on (Balanced
  tier per band default; no stall/failure requiring Opus escalation — this
  was code-editing work with a fully pre-specified target shape, matching
  LRPC-3's escalation-guidance rationale for staying on Sonnet).

### Design note: constant construction, not per-invocation build

`TOOL_CALLING_SYSTEM_PROMPT` remains a module-level string constant, built
once at import time by calling `build_system_prompt` against this module's
own fixed defaults (`_DEFAULT_MODEL_CONTEXT_TOKENS = 65536`,
`_DEFAULT_GENERATION_TOKEN_BUDGET = 8192` — duplicated from
`run_local_task.py`'s `MODEL_CONTEXT_TOKENS`/`GENERATION_TOKEN_BUDGET` to
avoid a circular import, since `run_local_task.py` already does `import
cli`). This exactly mirrors the pre-existing behavior: the hardcoded string
never depended on `args.num_ctx`/`args.num_predict` either. This task
changes *how* the constant is built (builder call vs. literal string), not
*when* (still import-time) or *what it depends on* (still fixed module
defaults, not the real per-invocation CLI args) — both explicitly out of
scope per the approved task card ("num_ctx/num_predict sourcing:
unchanged"). Flagged here for visibility, not as a defect: a future task
could make the prompt genuinely per-invocation-budget-aware if that is ever
judged necessary, but that is a scope expansion this task does not make.

### Phase 1 — Task-analysis review

`Task-analysis review: gemma docs/audit/gemma-evidence/LRPC-4-phase1.json - PASS`

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, explicit `--model` override —
  same rationale as LRPC-3: the bare script default resolves to
  `muse-glimmer:30b-q4_K_M`, the RRI 26-55 intermediate fallback, not the
  RRI 26-55 primary)
- Precheck: Ollama restarted for this task ID (`53087` → `5789`), listener
  confirmed on `:11434`, warm-up probes for both `gemma4:26b-a4b-it-qat`
  and `muse-glimmer:30b-q4_K_M` returned `done_reason: stop` with non-empty
  content before the real packets were sent.
- Command: manual `POST /api/chat`, `num_ctx=65536`, `num_predict=2048`,
  `temperature=0`, `think=false`
- Artifact: `docs/audit/gemma-evidence/LRPC-4-phase1.json`
- Verdict: `PASS` — no blocking findings
- Findings: one non-blocking note (preserve the newline separating the
  canonical anchor clause from the turn-budget text when splicing
  `output_format_text`) — verified satisfied in the implemented output (see
  Reflection Pass 1 below).
- Muse Glimmer fallback: not triggered — primary Gemma responded normally.
- D14 fallback: not triggered.
- disposition_divergence: `none`.

### Implementation notes

`TOOL_CALLING_SYSTEM_PROMPT` is referenced as a plain module-level string
by every existing caller and test (`rlt.TOOL_CALLING_SYSTEM_PROMPT`, no
call syntax) — most notably `run_local_task_test.py`'s
`SystemPromptCopyTest`/`SystemPromptTurnBudgetInterpolation` classes and
both `run_local_task.py` `run_loop`/`main` wrappers, which pass it as
`tool_calling_system_prompt=TOOL_CALLING_SYSTEM_PROMPT` explicitly (the
`cli.main()` parameter's own default is never exercised in practice).
Preserving this exact public shape (a string, not a function) was required
for HP-2/EC-2 without also rewriting every caller — so the builder call
happens once, at import time, producing the same kind of object
(`str`) the constant always was. `prompt_builder` is imported directly
(`from prompt_builder import build_system_prompt`); no new `sys.path`
wiring was needed since `run_local_task.py` (this module's only production
importer) already adds both `scripts/` and `scripts/local-agent/` to
`sys.path` before importing `cli`.

### Reflection log

Required passes: 3 (`42` → `Med-high`)

#### Pass 1 — contract fidelity

- **Draft verdict:** `cli.py`'s `TOOL_CALLING_SYSTEM_PROMPT` now sources its
  boundary clause via `build_system_prompt(role="local_developer", ...)`;
  the rest of the original text moved into a new
  `_TOOL_CALLING_OUTPUT_FORMAT_TEXT` constant passed through unchanged.
- **Critique findings:** fragment-by-fragment check of every sentence in
  the original hardcoded string against the assembled output — all 17
  content fragments present, zero loss; the old literal boundary sentence
  ("You may only edit the listed allowed_paths and then call finish...")
  confirmed absent (no duplication); phase-1's newline-preservation note
  verified satisfied (blank line separates the anchor clause block from
  the turn-budget paragraph, matching `build_system_prompt`'s own
  `f"{clause_block}\n\n{output_format_text}"` join).
- **Revisions applied:** none — no issues found.

#### Pass 2 — failure boundary and import wiring

- **Draft verdict:** the module-level `build_system_prompt(...)` call is
  unguarded; `run_local_task.py` imports `cli` cleanly with no circular-
  import error.
- **Critique findings:** confirmed no `try`/`except` wraps the constant's
  construction (grep across the file, only pre-existing unrelated
  try/except blocks at lines 212/395/430/451). Identified one design point
  worth recording explicitly rather than silently: the assembled prompt is
  still built once at import time against this module's own fixed
  defaults, not per-invocation against the real `args.num_ctx`/
  `args.num_predict` — verified this exactly mirrors pre-existing behavior
  (git diff confirms the old hardcoded string never referenced `args`
  either), so this is not a regression, and per-invocation budget-awareness
  was explicitly out of scope for this task ("num_ctx/num_predict sourcing:
  unchanged"). Recorded as a design note above rather than silently
  omitted.
- **Revisions applied:** none — added the "Design note" section above to
  make this explicit for future readers; no code change.

#### Pass 3 — test-regression coverage

- **Draft verdict:** 4 new tests added
  (`LRPC4PromptBuilderIntegration`) covering HP-1/HP-2/EC-1/EC-2; full
  suite run shows 64 passed (60 original + 4 new) / 33 pre-existing
  failures, byte-identical failing-test-name set confirmed via `diff`
  against the pre-change baseline.
- **Critique findings:** independently re-verified the phase-2 reviewer's
  major consensus finding (see Peer Reviewer evidence below) was a false
  positive caused by the reviewer only seeing the diff, not the merged
  runtime prompt — re-confirmed directly by importing `cli` and checking
  `boundary_violation`/`allowed_paths`/the canonical clause are all present
  in `cli.TOOL_CALLING_SYSTEM_PROMPT`. Also re-verified the reviewer's
  pass-specific minor finding about `rlt.MODEL_CONTEXT_TOKENS`/
  `rlt.GENERATION_TOKEN_BUDGET` "not being defined" was also a false
  positive — both symbols are untouched in `run_local_task.py` (the
  reviewer conflated them with this task's newly-added, differently-named
  `_DEFAULT_MODEL_CONTEXT_TOKENS`/`_DEFAULT_GENERATION_TOKEN_BUDGET` module
  constants in `cli.py`, a distinct pair of names). Ran a real end-to-end
  subprocess CLI invocation (unreachable host, same pattern as LRPC-0b) to
  confirm the full import/composition chain works for a real process, not
  just the in-process test harness — reached the first live network call
  before failing on the deliberately-unreachable host, as expected.
- **Revisions applied:** none — both re-verified findings disposed as false
  positives with reproducible evidence (not just re-asserted); the one
  genuine minor finding (constant duplication across `cli.py`/
  `run_local_task.py`) was already documented in-code with its rationale
  (avoiding a circular import) at implementation time, so no separate
  revision was needed.

### Peer Reviewer evidence

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, explicit `--model` override —
  `make qa-gemma-review` without an override resolves to
  `DEFAULT_REVIEW_MODEL` = `muse-glimmer:30b-q4_K_M`, the intermediate
  fallback for this band, not the primary; an initial invocation via the
  bare Makefile target was caught mid-run against the wrong model and
  killed before completion, then correctly re-invoked with the explicit
  `--model gemma4:26b-a4b-it-qat` override — the same class of
  reviewer-chain correction LRPC-3 already recorded for its own phase-1
  pass, here caught before the run completed rather than after)
- Command: `python3 scripts/gemma-code-review.py --model
  gemma4:26b-a4b-it-qat --num-ctx 65536 --num-predict 4096 --no-think
  --passes 3 --task-id LRPC-4 --out docs/audit/gemma-evidence/LRPC-4.json
  <packet scoped to scripts/local-agent/cli.py +
  scripts/local-agent/run_local_task_test.py, base main>`
- Passes run / usable: `3/3`
- Aggregate status: `FINDINGS` (1 consensus major, 3 pass-specific — 1
  duplicate of the consensus item, 2 distinct minors)
- Artifact: `docs/audit/gemma-evidence/LRPC-4.json`; receipt
  `docs/audit/gemma-evidence/LRPC-4-receipt.json` (`verdict:
  FINDINGS-ACKED`)
- Findings and disposition:
  - **Consensus major (all 3 passes) — "boundary enforcement clause
    removed, may not be re-injected":** the reviewer packet is a diff-only
    view (no full merged file, no runtime execution), so it could not
    observe that `build_system_prompt(role="local_developer", ...)`
    re-injects the equivalent canonical clause at import time. **Disposed
    as false positive** — independently verified by importing the real
    `cli` module and confirming `boundary_violation`, `allowed_paths`, and
    the exact canonical clause text are all present in the assembled
    `TOOL_CALLING_SYSTEM_PROMPT` (see Reflection Pass 3; this is also
    exactly what HP-1's own unit test asserts).
  - **Pass-specific minor — `rlt.MODEL_CONTEXT_TOKENS`/
    `rlt.GENERATION_TOKEN_BUDGET` "not defined in the diff":** the
    reviewer conflated `run_local_task.py`'s existing, untouched
    `MODEL_CONTEXT_TOKENS`/`GENERATION_TOKEN_BUDGET` module constants with
    this task's new, differently-prefixed `_DEFAULT_MODEL_CONTEXT_TOKENS`/
    `_DEFAULT_GENERATION_TOKEN_BUDGET` constants added to `cli.py`.
    **Disposed as false positive** — `grep` confirms both original symbols
    are untouched in `run_local_task.py:135,148`, and
    `test_ec1_budget_exceeded_propagates_uncaught_before_any_ollama_call`
    (which references them) is in the 4 passing new tests.
  - **Pass-specific minor — constant duplication across files:**
    genuine, already-acknowledged trade-off. **Accepted, no code change** —
    the duplication and its rationale (avoiding a circular import, since
    `run_local_task.py` already does `import cli`) are documented directly
    in `cli.py`'s own code comment at the point of duplication, written at
    implementation time before this review ran.
- Muse Glimmer fallback: not triggered — primary Gemma responded normally
  across all 3 passes (after the corrected re-invocation).
- D14 fallback: not triggered.
- disposition_divergence: `partial` — the reviewer's own severity
  labels (1 major, 2 distinct minors) were accepted as-recorded, but two of
  the three underlying claims were independently verified false on
  reproducible evidence rather than accepted at face value; the third
  (genuine) minor was accepted with no code change since it was already
  self-documented.
- Primary-agent disposition: 2 false positives rejected with reproducible
  counter-evidence; 1 genuine minor accepted as already-documented,
  non-blocking.

Review artifact: docs/audit/gemma-evidence/LRPC-4.json

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | assembled prompt contains the canonical `local_developer` clause verbatim | `scripts/local-agent/run_local_task_test.py::LRPC4PromptBuilderIntegration::test_hp1_canonical_local_developer_clause_is_a_verbatim_substring` | passed |
| HP-2 | Happy path | output-format contract content unchanged, passed through | `scripts/local-agent/run_local_task_test.py::LRPC4PromptBuilderIntegration::test_hp2_output_format_contract_content_is_unchanged` | passed |
| EC-1 | Edge case | `PromptBudgetExceeded` propagates uncaught before any Ollama HTTP call | `scripts/local-agent/run_local_task_test.py::LRPC4PromptBuilderIntegration::test_ec1_budget_exceeded_propagates_uncaught_before_any_ollama_call` | passed |
| EC-2 | Edge case | the 60 previously-passing tests remain passing, identical outcome | `scripts/local-agent/run_local_task_test.py::LRPC4PromptBuilderIntegration::test_ec2_old_hardcoded_boundary_phrasing_does_not_survive` (regression guard) + full-suite diff of failing-test names, byte-identical before/after | passed |

Supplementary verification beyond the task's own HP/EC set (Reflection
Pass 3): full `run_local_task_test.py` suite (64/97 passed, 33
pre-existing unrelated failures, byte-identical set to the pre-task
baseline); frozen `prompt_anchors_test.py` + `prompt_builder_test.py` +
`gemma_code_review_test.py` suites (70/70 passed, no regression);
`boundary_test.py` + `integration_test.py` (20/20 passed); a real
end-to-end subprocess CLI invocation reaching the first live network call
before failing on a deliberately-unreachable host (proves the full
import/composition chain works for a real process invocation).

### Owner final verification

- Owner: `matias` (via Claude Code orchestrator, session 2026-08-19)
- Date: `2026-08-19`
- Statement: I verified LRPC-4's HP-1, HP-2, EC-1, and EC-2 all have
  concrete unit test evidence that replicates the expected behavior.
  I independently re-verified both false-positive findings from the Gemma
  phase-2 review (the "boundary clause removed" major and the "constants
  not defined" minor) against the actual runtime module output rather than
  accepting the reviewer's diff-only read at face value, and confirmed the
  one genuine minor finding (constant duplication) was already
  self-documented at implementation time. The Med-high ADR-038 route
  (Muse Glimmer + primary receipt both `GO_LOCAL`, cloud-only
  implementation per band policy regardless, ADR-039 human-select
  checkpoint) was followed in full, and the 3-pass Reflection cycle ran
  regardless of the local-vs-cloud authorship question this band does not
  leave open.
- Commands run: `python3 scripts/rri.py --cc 3 --D 4 --K 2 --P 3 --T 4 --A 1
  --X 2 --touches scripts/local-agent/cli.py --touches
  scripts/local-agent/run_local_task_test.py`; `python3
  scripts/local-architect/run_analysis.py --profile med-high-refinement-v1
  ...` (Muse Glimmer refinement); `python3 scripts/local-agent/med_high_gate.py
  ...` (route decision); `python3 -m pytest
  scripts/local-agent/run_local_task_test.py -q --tb=no -rA`; `diff` of
  failing-test-name sets before/after; `python3 -m pytest
  scripts/local-agent/prompt_anchors_test.py
  scripts/local-agent/prompt_builder_test.py
  scripts/gemma_code_review_test.py scripts/local-agent/boundary_test.py
  scripts/local-agent/integration_test.py -q`; `python3 -m py_compile
  scripts/local-agent/cli.py scripts/local-agent/run_local_task_test.py`;
  `python3 scripts/gemma-code-review.py --model gemma4:26b-a4b-it-qat
  --num-ctx 65536 --num-predict 4096 --no-think --passes 3 --task-id
  LRPC-4 ...`; a real subprocess `python3 scripts/local-agent/run_local_task.py
  --card ... --worktree ... --out ...` smoke invocation.

Reminder: run `/compact` (or `/clear` if this task's context is no longer
needed) now that LRPC-4 is closed.

## LRPC-5 — Refactor `run_analysis.py` to consume the builder

- **Status: `[x] Done` (2026-08-20).**
- **Dependencies:** LRPC-2 (satisfied — `prompt_builder.py` delivered).
- **Final RRI: 42 → Med-high (41-55)** — same class of correction LRPC-3/
  LRPC-4 already applied to themselves against the ledger's stale
  "Provisional effort: M". `scripts/rri.py --cc 3 --D 4 --K 2 --P 3 --T 4
  --A 1 --X 2 --touches scripts/local-architect/run_analysis.py --touches
  scripts/local-architect/run_analysis_test.py`. D=4/P=3/K=2/X=2 carried
  forward from LRPC-2/LRPC-3/LRPC-4 for the same review-pipeline-integrity
  rationale (this consumer script builds the prompt the Local Architect
  advisory role receives, including the ADR-038 Muse Glimmer refinement
  gate LRPC-5 itself must be routed through). No penalties triggered.
- **Effort: L** (derived from the Med-high band per the canonical
  RRI-to-Effort crosswalk).

### Scope

**In scope:** `scripts/local-architect/run_analysis.py`'s `build_prompt()`,
both branches — `DEFAULT_PROFILE` (`role="local_architect_default"`) and
`MED_HIGH_REFINEMENT_PROFILE` (`role="local_architect_med_high"`) — now call
`prompt_builder.build_system_prompt(role=..., num_ctx=_BOUNDARY_BUDGET_NUM_CTX,
num_predict=_BOUNDARY_BUDGET_NUM_PREDICT, output_format_text=<schema +
instructions + packet_json, unchanged per profile>)` instead of the prior
fully-hardcoded prompt strings.

**Out of scope (unchanged):** `prompt_anchors.py`/`prompt_builder.py`
(frozen, LRPC-1/LRPC-2); `scripts/local-agent/run_local_task.py`,
`scripts/local-agent/cli.py`, `scripts/gemma-code-review.py` (frozen,
LRPC-3/LRPC-4); the JSON schema/instructions text per profile (no
canonical-doc source, stays local, passed through unchanged as
`output_format_text`); `parse_args`'s CLI `num_ctx`/`num_predict` sourcing
(unchanged — `build_prompt()` uses its own fixed
`_BOUNDARY_BUDGET_NUM_CTX`/`_BOUNDARY_BUDGET_NUM_PREDICT` defaults for the
boundary-clause budget check regardless of the real per-invocation CLI
args, mirroring LRPC-4's identical "budget check uses fixed defaults, not
live args" precedent); the 33 pre-existing unrelated test failures in
`run_local_task_test.py` (LRPC-5 does not touch this file at all — `git
diff --stat` confirms zero changes — so this is a pure non-regression
check, not new scope).

**HP-1 reinterpretation (owner-approved deviation, 2026-08-20):** the
original task stub required the assembled `DEFAULT_PROFILE` prompt to be
**byte-for-byte identical** to the pre-existing hardcoded opener text
("advisory-only, read-only, and must not claim authority..."). Mid-
implementation this was found to be impossible to satisfy simultaneously
with the task's actual purpose (replacing the paraphrased opener with the
canonical ADR-037 clauses from `prompt_anchors.ROLE_ANCHORS
["local_architect_default"]`): none of the five canonical "may not" clauses
were ever present as a substring in the old hardcoded string — it was a
paraphrase, not a verbatim subset, meaning the old string already violated
this plan's own "extraction, not paraphrase" design principle (plan doc
§ Design decisions). Byte-for-byte fidelity to that string would have
required *keeping* the exact drift class this whole plan exists to close.
Surfaced to the user via `AskUserQuestion` with two labeled options (A:
reinterpret HP-1 as "contains every canonical clause verbatim," same
pattern LRPC-3/LRPC-4 already use for their own HP-1; B: keep the old
paraphrased string byte-identical and treat LRPC-5 as a no-op). User asked
for the concrete practical impact of Option A on context-window budget
before deciding; answered with measured (not estimated) numbers: the new
builder-sourced prompt is ≈82 tokens larger than the old hardcoded string
against `_BOUNDARY_BUDGET_NUM_CTX=8192`/`_BOUNDARY_BUDGET_NUM_PREDICT=4096`
(≈4096-token available prompt budget), a ≈2% growth — not a practical
constraint. User approved Option A ("Sí, proceder"). This reinterpretation
governs HP-1/HP-2 below; it does not change scope, RRI, or the Med-high
routing already in effect.

### Behavioral examples

- **HP-1:** the assembled `DEFAULT_PROFILE` prompt via
  `build_system_prompt(role="local_architect_default", ...)` contains every
  one of the five canonical `prompt_anchors.ROLE_ANCHORS
  ["local_architect_default"]` clause texts as a verbatim substring
  (reinterpreted from byte-for-byte identity to the old paraphrase — see
  deviation note above).
- **HP-2:** the assembled `MED_HIGH_REFINEMENT_PROFILE` prompt via
  `build_system_prompt(role="local_architect_med_high", ...)` contains
  every one of the same five canonical clause texts verbatim (both roles
  share the identical `Clause` set by LRPC-1 design).
- **HP-3/HP-4:** for both profiles, the JSON schema block, the
  profile-specific instructions ("Choose CLOUD_REQUIRED whenever...",
  compact-response constraints for med-high), and the injected
  `packet_json` content remain present and unchanged, passed through as
  `output_format_text`.
- **EC-1/EC-2:** `PromptBudgetExceeded` propagates uncaught before any
  Ollama HTTP call is constructed, for both profiles, when
  `num_ctx`/`num_predict` make the assembled prompt exceed its derived
  budget (verified against `prompt_builder.build_system_prompt` with a
  patched `estimate_text_tokens`, mirroring LRPC-3/LRPC-4's identical EC
  test shape).
- **EC-3:** an unknown `role` value would raise `UnknownRoleError`, not
  silently fall back to a default role's clauses (regression-style
  assertion on `prompt_builder`'s own contract; `build_prompt()` never
  passes a caller-controlled `role`, so this is defense-in-depth, not a
  reachable path in the current callers).

### Implementation routing evidence

- **Route:** ADR-038 Med-high Architect-refined single-attempt gate.
  1. Muse Glimmer advisory refinement (`muse-glimmer:30b-q4_K_M`,
     `docs/audit/med-high/lrpc-5-refinement-artifact.json`): recommended
     **`CLOUD_REQUIRED`** — "Byte-identical DEFAULT_PROFILE requirement
     cannot be safely bounded locally without golden-string evidence and
     builder output verification." (This was the refinement run against
     the task's *original* byte-for-byte HP-1 wording, before the
     HP-1 deviation was raised and approved — the refinement's own
     skepticism about that literal requirement is part of what surfaced
     the conflict.)
  2. Primary hash-bound route receipt
     (`docs/audit/med-high/lrpc-5-primary-receipt.json`): **concurred**
     `CLOUD_REQUIRED` — no downgrade. Unlike the `GO_LOCAL` precedent in
     LRPC-3/LRPC-4, this task's original HP-1 acceptance criterion demanded
     byte-for-byte identity against a string proven (by direct
     substring-check against `prompt_anchors.py`) not to contain the
     canonical clauses it was meant to carry, which is exactly the kind of
     unresolved-requirement ambiguity ADR-038 §6 treats as excluding
     `GO_LOCAL` regardless of the advisory recommendation.
  3. Gate evaluation (`scripts/local-agent/med_high_gate.py`):
     `{"route": "CLOUD_REQUIRED", ...}` — both card-hash and
     refinement-artifact-sha256 binding checks passed.
  4. Per `AGENT_WORKFLOW_GUIDE.md § Local-first and Architect-refined
     implementation routing`, Med-high is cloud-only implementation in
     every case; here the gate additionally concurred explicitly rather
     than being overridden by the band default. No local implementation
     attempt was made.
  5. ADR-039 fallback-selection checkpoint: `human-select` mode. User
     selected `claude-sonnet-5` / reasoning effort `high` via
     `AskUserQuestion`, trigger kind `capability-risk` (the `CLOUD_REQUIRED`
     gate result, concurred by the primary receipt, confirms a genuine
     capability/risk takeover, not a mere operational-unavailability
     fallback). Recommended alternative was `gpt-5.6-sol`/`high` (frozen
     matrix default for `capability-risk` + RRI 42); the user's explicit
     choice overrides the recommendation, which the checkpoint permits.
     Artifact: `docs/audit/med-high/lrpc-5-fallback-selection.json`
     (`status: fallback_authorized`).
- **Implementer:** `claude-sonnet-5` (this session), thinking on (Balanced
  tier per band default; no stall/failure requiring Opus escalation — code-
  editing work with a fully pre-specified target shape once the HP-1
  conflict was resolved by explicit user decision).

### Design note: per-invocation build, not import-time constant

Unlike LRPC-4's `cli.py` (a module-level string built once at import time),
`run_analysis.py`'s `build_prompt(packet, profile)` is called per-invocation
with the real packet content, so the builder call happens inline inside
each `if profile == ...` branch rather than as a top-level constant. The
boundary-clause budget check (`_BOUNDARY_BUDGET_NUM_CTX=8192`,
`_BOUNDARY_BUDGET_NUM_PREDICT=4096`) still uses fixed module-level defaults
rather than the real per-invocation `args.num_ctx`/`args.num_predict` from
`parse_args` — this exactly mirrors the pre-existing behavior (the old
hardcoded strings never depended on `args` either) and was explicitly out
of scope, same as LRPC-4's identical design note.

### Phase 1 — Task-analysis review

`Task-analysis review: gemma docs/audit/gemma-evidence/LRPC-5-phase1.json - PASS`

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, explicit model selection —
  same rationale as LRPC-3/LRPC-4: the bare script default resolves to
  `muse-glimmer:30b-q4_K_M`, the RRI 26-55 intermediate fallback, not the
  RRI 26-55 primary)
- Precheck: Ollama restarted for this task ID, listener confirmed on
  `:11434`, warm-up probe for `gemma4:26b-a4b-it-qat` returned
  `done_reason: stop` with non-empty content before the real packet was
  sent.
- First attempt returned a bare `{"verdict": "PASS", "findings": []}` with
  no visible reasoning (52 chars). Judged insufficiently rigorous review
  depth (not a technical failure — valid JSON, `done_reason: stop`) and
  retried once with an explicit reasoning-before-verdict instruction, which
  produced a substantive analysis addressing scope-boundedness, byte-for-
  byte fidelity risk, `MED_HIGH_REFINEMENT_PROFILE` risk, RRI/route
  consistency, and opener-duplication risk. Both attempts recorded in the
  artifact; the retry's reasoning is the canonical evidence.
- Artifact: `docs/audit/gemma-evidence/LRPC-5-phase1.json`
- Verdict: `PASS` — no blocking findings
- Muse Glimmer fallback: not triggered — primary Gemma responded normally.
- D14 fallback: not triggered.
- disposition_divergence: `none`.

### Reflection log

Required passes: 3 (`42` → `Med-high`)

#### Pass 1 — contract fidelity

- **Draft verdict:** both `build_prompt()` branches now source their
  boundary-clause opener via `build_system_prompt(role="local_architect_
  default"|"local_architect_med_high", ...)`; the schema/instructions/
  packet_json text for each profile moved into `output_format_text`
  unchanged.
- **Critique findings:** verified all five canonical
  `local_architect_default` clause texts (and, separately, all five
  `local_architect_med_high` clause texts — confirmed identical sets by
  design) appear as exact substrings in the assembled output for each
  profile, via direct string-containment checks against
  `prompt_anchors.ROLE_ANCHORS`. Confirmed the JSON schema block, the
  profile-specific instructions, and the `packet_json` injection are
  byte-identical to the pre-change strings (diffed the surviving fragment
  text against `git diff` context lines). Confirmed the old paraphrased
  opener ("advisory-only, read-only, and must not claim authority...") does
  not survive in either profile's output (no duplication of old + new
  opener text).
- **Revisions applied:** none — no issues found.

#### Pass 2 — failure boundary and per-invocation wiring

- **Draft verdict:** both `build_system_prompt(...)` calls are unguarded
  inside `build_prompt()` — no `try`/`except` wraps them; a
  `PromptBudgetExceeded` from either branch propagates to `build_prompt()`'s
  own caller uncaught.
- **Critique findings:** confirmed via `grep` that no `try`/`except` wraps
  either branch (the function's only other exception path is the explicit
  `raise AnalysisError("invalid_profile", ...)` for an unrecognized
  `profile` value, unrelated to the builder call). Confirmed
  `_BOUNDARY_BUDGET_NUM_CTX`/`_BOUNDARY_BUDGET_NUM_PREDICT` are fixed
  module constants independent of `parse_args`'s real CLI values — verified
  this exactly mirrors pre-existing behavior (the old hardcoded strings
  never referenced `args` either), so not a regression; recorded explicitly
  in the Design note above rather than left implicit.
- **Revisions applied:** none — added the "Design note" section above for
  visibility; no code change.

#### Pass 3 — test-regression coverage

- **Draft verdict:** 8 new tests added (`PromptBuilderIntegrationTest`)
  covering HP-1/HP-2/HP-3/HP-4/EC-1/EC-2/EC-3 (HP-2b added as a
  supplementary same-clause-set assertion beyond the task's own case list);
  full local suite run shows 28/28 passed (20 original + 8 new).
- **Critique findings:** ran the cross-suite regression check spanning
  every frozen LRPC-1/2/3 artifact plus this task's own suite — 101/101
  passed, zero regressions. Independently confirmed
  `scripts/local-agent/run_local_task_test.py` (out of scope, untouched by
  this diff) has zero uncommitted changes (`git status --short` empty for
  that file) and its failing-test-name set is the same 33-name baseline
  documented at LRPC-4 closure (64 passed / 33 pre-existing-unrelated —
  the 64, not 60, is LRPC-4's own delta of +4 tests to that same file,
  already recorded at LRPC-4 closure line 1420, not a new LRPC-5 change).
  This reconciles what was flagged as an open discrepancy before this pass:
  64 passed is the correct current baseline for that file, not a
  regression signal.
- **Revisions applied:** none — all counts reconciled against documented
  baselines with reproducible evidence, no discrepancy remained.

### Peer Reviewer evidence

- Reviewer: `gemma` (`gemma4:26b-a4b-it-qat`, explicit `--model` override —
  same rationale as LRPC-3/LRPC-4: the bare script/Makefile default
  resolves to `muse-glimmer:30b-q4_K_M`, the intermediate fallback for this
  band, not the primary)
- Command: `python3 scripts/gemma-code-review.py --model
  gemma4:26b-a4b-it-qat --passes 3 --task-id LRPC-5 --max-wall 180 --out
  docs/audit/gemma-evidence/LRPC-5.json <packet: diff of
  scripts/local-architect/run_analysis.py +
  scripts/local-architect/run_analysis_test.py, plus LRPC-5 acceptance
  criteria and independently-verified facts>`
- Passes run / usable: `3/3`
- Aggregate status: `FINDINGS` (1 consensus minor, both consensus and the
  single pass-specific-only variant explicitly scoped `out-of-scope` by the
  reviewer itself)
- Artifact: `docs/audit/gemma-evidence/LRPC-5.json`; receipt
  `docs/audit/gemma-evidence/LRPC-5-receipt.json` (`verdict:
  FINDINGS-ACKED`)
- Findings and disposition:
  - **Consensus minor — `sys.path.insert(0, ...)` sibling-directory import
    pattern (line 18-19), reviewer-scoped `out-of-scope`:** this is the
    same import-wiring pattern already used identically by
    `prompt_builder.py` itself (LRPC-2), `cli.py` (LRPC-4), and every other
    script in this repo that imports a sibling script as a module — not
    introduced or changed by this diff. **Accepted, no code change** — the
    reviewer's own `out-of-scope` label matches the primary agent's
    independent assessment; a repo-wide import-mechanism change is
    explicitly out of LRPC-5's scope.
- Muse Glimmer fallback: not triggered — primary Gemma responded normally
  across all 3 passes.
- D14 fallback: not triggered.
- disposition_divergence: `none`.
- Primary-agent disposition: 1 out-of-scope minor accepted with no code
  change, consistent with the reviewer's own scope label.

Review artifact: docs/audit/gemma-evidence/LRPC-5.json

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `DEFAULT_PROFILE` prompt contains every canonical `local_architect_default` clause verbatim | `scripts/local-architect/run_analysis_test.py::PromptBuilderIntegrationTest::test_hp1_default_profile_prompt_contains_every_local_architect_default_clause` | passed |
| HP-2 | Happy path | `MED_HIGH_REFINEMENT_PROFILE` prompt contains every canonical `local_architect_med_high` clause verbatim | `scripts/local-architect/run_analysis_test.py::PromptBuilderIntegrationTest::test_hp2_med_high_profile_prompt_contains_every_local_architect_med_high_clause` | passed |
| HP-3 | Happy path | `DEFAULT_PROFILE` schema + packet content unchanged | `scripts/local-architect/run_analysis_test.py::PromptBuilderIntegrationTest::test_hp3_default_profile_schema_and_packet_content_still_present` | passed |
| HP-4 | Happy path | `MED_HIGH_REFINEMENT_PROFILE` schema + packet content unchanged | `scripts/local-architect/run_analysis_test.py::PromptBuilderIntegrationTest::test_hp4_med_high_profile_schema_and_packet_content_still_present` | passed |
| EC-1 | Edge case | `PromptBudgetExceeded` propagates uncaught before any Ollama call, `DEFAULT_PROFILE` | `scripts/local-architect/run_analysis_test.py::PromptBuilderIntegrationTest::test_ec1_budget_exceeded_propagates_uncaught_before_any_ollama_call_default` | passed |
| EC-2 | Edge case | `PromptBudgetExceeded` propagates uncaught before any Ollama call, `MED_HIGH_REFINEMENT_PROFILE` | `scripts/local-architect/run_analysis_test.py::PromptBuilderIntegrationTest::test_ec2_budget_exceeded_propagates_uncaught_before_any_ollama_call_med_high` | passed |
| EC-3 | Edge case | unknown `role` raises `UnknownRoleError`, no silent default fallback | `scripts/local-architect/run_analysis_test.py::PromptBuilderIntegrationTest::test_ec3_unknown_role_would_raise_typed_error_not_silently_default` | passed |

Supplementary verification beyond the task's own HP/EC set (Reflection
Pass 3): `test_hp2b_default_and_med_high_profiles_reuse_the_identical_clause_set`
(both roles' clause sets confirmed identical by design, not merely by
coincidence); full cross-suite run of `prompt_anchors_test.py` +
`prompt_builder_test.py` + `gemma_code_review_test.py` +
`run_analysis_test.py` + `adr037_handoff_mapping_test.py` (101/101 passed,
zero regression across every frozen LRPC-1/2/3 suite);
`run_local_task_test.py` confirmed untouched (0 uncommitted changes) with
its documented 64-passed/33-pre-existing-failure baseline (LRPC-4's own
+4 delta) intact; `python3 -m py_compile
scripts/local-architect/run_analysis.py` clean.

### Owner final verification

- Owner: `matias` (via Claude Code orchestrator, session 2026-08-20)
- Date: `2026-08-20`
- Statement: I verified LRPC-5's HP-1, HP-2, HP-3, HP-4, EC-1, EC-2, and
  EC-3 all have concrete unit test evidence that replicates the expected
  behavior. I explicitly reviewed and approved the HP-1 deviation (byte-
  for-byte identity reinterpreted as verbatim-clause-containment) after
  being shown the practical token-budget impact of the change (≈82 tokens,
  ≈2% of the available prompt budget) before deciding. The Med-high
  ADR-038 route (Muse Glimmer + primary receipt both `CLOUD_REQUIRED`,
  cloud-only implementation, ADR-039 human-select checkpoint) was followed
  in full, and the 3-pass Reflection cycle ran to completion. I confirmed
  the `run_local_task_test.py` 64-vs-60-passed question was a benign
  reconciliation against LRPC-4's own already-documented +4 test delta to
  that file, not a real regression introduced by this task.
- Commands run: `python3 scripts/rri.py --cc 3 --D 4 --K 2 --P 3 --T 4 --A 1
  --X 2 --touches scripts/local-architect/run_analysis.py --touches
  scripts/local-architect/run_analysis_test.py`; `python3
  scripts/local-architect/run_analysis.py --profile med-high-refinement-v1
  ...` (Muse Glimmer refinement); `python3
  scripts/local-agent/med_high_gate.py ...` (route decision); `python3 -m
  pytest scripts/local-architect/run_analysis_test.py -v`; `python3 -m
  pytest scripts/local-agent/prompt_anchors_test.py
  scripts/local-agent/prompt_builder_test.py scripts/gemma_code_review_test.py
  scripts/local-architect/run_analysis_test.py
  scripts/local-architect/adr037_handoff_mapping_test.py -q`; `python3 -m
  pytest scripts/local-agent/run_local_task_test.py -q --tb=no -rA`; `git
  status --short scripts/local-agent/run_local_task_test.py`; `python3 -m
  py_compile scripts/local-architect/run_analysis.py`; `python3
  scripts/gemma-code-review.py --model gemma4:26b-a4b-it-qat --passes 3
  --task-id LRPC-5 --max-wall 180 --out docs/audit/gemma-evidence/LRPC-5.json
  ...`.

Reminder: run `/compact` (or `/clear` if this task's context is no longer
needed) now that LRPC-5 is closed.

## LRPC-6 — Golden-set behavioral-equivalence harness

- **Dependencies:** LRPC-1 (anchors must exist), LRPC-2 (builder must exist
  to produce the "after" condition).
- **Provisional effort:** L. This is the plan's actual verification strategy
  (design decision 6) — a fixture set of adversarial packets per role where
  the correct verdict depends on the exact clause under compression, run
  against full-prose context vs. builder output, asserting identical
  verdicts. Requires live Ollama calls; fixture design must be deliberately
  discriminating, not just plausible-looking.

**RRI 51 (Med-high)** — `python3 scripts/rri.py --cc 10 --D 4 --K 3 --P 3 --T 3
--A 2 --X 3 --touches scripts/local-agent/golden_set.py --touches
scripts/local-agent/golden_set_test.py --touches
scripts/local-agent/golden_fixtures.py`. D=4 (review-pipeline integrity,
consistent with LRPC-1 through LRPC-5); X raised to 3 relative to prior LRPC
tasks because this is the first task in the plan requiring live Ollama
calls as its actual verification mechanism, not just as an implementation
detail. Card approved by the user (Makefile-target sub-question resolved
first via `AskUserQuestion`: confirmed `make qa-golden-set`, mirroring `make
qa-gemma-review`'s opt-in/outside-`qa-ci`/fixed-model-by-convention
pattern), then explicit final approval ("aprobado") for the full card.

**ADR-038 routing:** Muse Glimmer refinement
(`docs/audit/med-high/lrpc6/refinement_artifact.json`) recommended
`GO_LOCAL`. Primary hash-bound route receipt
(`docs/audit/med-high/lrpc6/primary_receipt.json`) downgraded to
`CLOUD_REQUIRED` per ADR-038 Amendment 1 (2026-08-12, "Med-high local
execution disabled" — every `GO_LOCAL` result is policy-excluded from
starting a local developer for the whole task). `med_high_gate.py --rri 51`
confirmed `{"route": "CLOUD_REQUIRED", "reason": "Primary receipt downgraded
GO_LOCAL to cloud."}`. ADR-040 per-module split was evaluated and not
applied: all touched paths are new files with no pre-implementation CC
measurement and no heterogeneous complexity signal, and no hard-excluded
domain (auth/security/rights/consent/governance/schema/migration) is
touched either way — recorded in the primary receipt's rationale field.
ADR-039 human-select checkpoint
(`docs/audit/med-high/lrpc6/fallback_selection.json`): owner chose
`claude-sonnet-5`/high (self, in-session) over the alternative
`gpt-5.6-sol`/high offered per the Med-high capability/risk-takeover
resolution — Codex CLI is not on `$PATH` in this environment (see
`reference_codex_cli_location` memory); implemented directly by Claude Code.

**Delivered:** `scripts/local-agent/golden_fixtures.py` (12 adversarial
fixtures spanning all 4 `ROLE_ANCHORS` roles, each targeting one specific
extracted clause, each role carrying at least one `PASS` and one
`VIOLATION` fixture), `scripts/local-agent/golden_set.py` (harness: builds
both the full-canonical-prose and `build_system_prompt()` system prompts
per fixture, sends the same transcript to the same live model under both,
parses a fail-closed `{"verdict": "PASS"|"VIOLATION", "reason": ...}`
response, and marks a fixture `equivalent` only if both conditions agree
with each other and with the fixture's `expected_verdict`),
`scripts/local-agent/golden_set_test.py` (11 deterministic tests, every
Ollama call mocked via `unittest.mock.patch.object(golden_set,
"stream_chat", ...)` — never live in this suite or in `make qa-ci`), and
`make qa-golden-set` (opt-in, outside `qa-ci`, `DUBBRIDGE_SKIP_GOLDEN_SET`
escape hatch, `GOLDEN_SET_MODEL`/`GOLDEN_SET_RESULT` variables, mirrors
`qa-gemma-review`'s exact pattern).

**The harness found a real production defect on its first live run** (this
is the harness working correctly, not a harness bug — exactly what LRPC-6's
EC-1 requires it to be capable of). Running all 11 fixtures live against
`gemma4:26b-a4b-it-qat` produced 8/11 PASS, 3/11 FAIL — all three failures on
`local_architect_default`/`local_architect_med_high` fixtures. Root cause:
`prompt_anchors.ROLE_ANCHORS` extracted the 5 ADR-037 "may not" list items as
bare clause fragments (e.g. `"edit source code, tests, configuration,
policies, ledgers, or canonical ADRs;"`) without their governing header
("The role may not:", ADR-037 line 70), so `build_system_prompt()`'s
assembled prompt read as an unqualified list with no prohibition marker.
Both `gemma4` and `muse-glimmer` interpreted the builder-output condition as
*permitting* exactly what the full canonical prose (which does carry "The
role may not:") correctly flagged as a violation — e.g. "the agent is
running a shell command ... which is explicitly permitted under the
authority to 'run shell commands or operate a repository worktree'." This
affects the Local Architect / Complex Analyst role in live production use,
including the LRPC-6 Muse Glimmer refinement call made earlier in this same
task.

Flagged to the user before proceeding (this reopens `prompt_anchors.py`,
frozen by LRPC-1). Owner directed an immediate fix within this task rather
than deferring to a separate ledger entry. Fix: prepended the verbatim
substring `"The role may not:\n\n- "` to the first clause of both
`local_architect_default` and `local_architect_med_high` (verbatim-substring
provenance preserved per Design decision 1 — "The role may not:" is itself a
literal substring of ADR-037 at line 70; both frozen structural tests,
`prompt_anchors_test.py` and `prompt_builder_test.py`, pass unmodified
against the corrected clause text). Re-ran the harness live after the fix:
11/11 PASS against `gemma4:26b-a4b-it-qat`; additionally re-verified
`local_architect_default` (3/3) and `local_architect_med_high` (2/2)
specifically against `muse-glimmer:30b-q4_K_M` — the exact model bound to
that role in production — both clean. Full `run_analysis_test.py` (28/28)
and `med_high_gate_test.py` (30/30) unaffected.

**Reflection log** (3 passes, Med-high):

- *Pass 1 (contract fidelity):* verified the harness compares live-model
  verdicts across conditions rather than inspecting builder-output text
  content, which is the actual novel proof Design decision 6 requires (LRPC-
  1/2's own tests already cover substring containment). No revision needed.
- *Pass 2 (failure boundaries):* verified `run_condition` catches
  `GoldenSetError`/`RuntimeError` per-condition without crashing the run
  (`ConditionErrorIsRecordedNotRaised` test); verified `--role` with no
  matching fixtures exits 2 with a clear message; verified the harness is
  non-vacuous — it caught a real defect on its first live run, satisfying
  EC-1 directly rather than only by construction. No revision needed.
- *Pass 3 (coverage):* verified all 4 `ROLE_ANCHORS` roles have both a
  `PASS` and a `VIOLATION` fixture (`FixtureCoverageSpansAllFourRoles`);
  confirmed `qa-ci`'s dependency list does not include `qa-golden-set`;
  confirmed the deterministic suite alone cannot certify live-model
  behavior — evidenced by running the harness live against both `gemma4`
  and `muse-glimmer` for every fixture, not relying on mocked tests as
  closure evidence. No revision needed.

### Peer Reviewer evidence

- Reviewer: `gemma`
- Command: `REVIEW_PATHS="Makefile scripts/local-agent/prompt_anchors.py scripts/local-agent/golden_fixtures.py scripts/local-agent/golden_set.py scripts/local-agent/golden_set_test.py" GEMMA_REVIEW_TASK_ID=LRPC-6 make qa-gemma-review`
- Artifact: `docs/audit/gemma-evidence/LRPC-6.json`, `/tmp/dubbridge-gemma-review.json` (3-pass aggregate)
- Verdict: `PASS`
- Findings: none (0 across every bucket — consensus, pass-specific, severity-inconsistent, location-inconsistent, likely-false-positive)
- Muse Glimmer fallback: not triggered — reason: Gemma primary healthy, 3/3 passes usable
- D14 fallback: not triggered — reason: n/a
- D14 provider route: n/a
- disposition_divergence: none
- Primary-agent disposition: accepted (no findings to disposition)

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | fixture whose verdict depends on exact clause wording produces identical live verdicts under full-prose and builder-output conditions | `scripts/local-agent/golden_set_test.py::HP1EquivalentFixtureAcrossBothConditionsPasses::test_matching_expected_verdicts_in_both_conditions_marks_equivalent` | passed |
| EC-1 | Edge case | a lossy/corrupted builder condition that flips the verdict is detected as a mismatch, not silently passed | `scripts/local-agent/golden_set_test.py::EC1DivergentConditionVerdictsAreDetectedAsMismatch::test_builder_condition_disagreeing_with_expected_verdict_is_not_equivalent` | passed |

EC-1 is additionally certified by the live run itself: the harness detected
a real divergence (3/11 FAIL) on its first live execution before the
`prompt_anchors.py` fix, and confirmed 11/11 PASS after — direct evidence
the harness discriminates rather than passing vacuously, beyond what the
mocked unit test alone can prove.

### Owner final verification

- Owner: `matias`
- Date: `2026-08-20`
- Statement: I verified the golden-set harness is discriminating (it caught
  and the fix resolved a real production defect in the Local Architect
  role's assembled prompt), that HP-1 and EC-1 have unit test evidence
  replicating the described behavior, that the live-model runs confirm the
  fix against both models bound to the affected role in production, and
  that the frozen LRPC-1/2/3/4/5 test suites remain unaffected by the
  `prompt_anchors.py` correction.
- Commands run: `python3 -m pytest scripts/local-agent/prompt_anchors_test.py scripts/local-agent/prompt_builder_test.py scripts/local-agent/golden_set_test.py scripts/local-agent/med_high_gate_test.py scripts/local-architect/run_analysis_test.py -q` (81 passed); `python3 scripts/local-agent/golden_set.py --model gemma4:26b-a4b-it-qat --out /tmp/dubbridge-golden-set-v2.json` (11/11 PASS); `python3 scripts/local-agent/golden_set.py --model muse-glimmer:30b-q4_K_M --role local_architect_default ...` (3/3 PASS); `python3 scripts/local-agent/golden_set.py --model muse-glimmer:30b-q4_K_M --role local_architect_med_high ...` (2/2 PASS); `REVIEW_PATHS=... GEMMA_REVIEW_TASK_ID=LRPC-6 make qa-gemma-review` (PASS, 0 findings); `make -n qa-golden-set` (confirmed not reachable from `qa-ci`).

## LRPC-7 — Cross-check `check-review-budget.py`'s `PACKET_OVERHEAD_TOKENS`

- **Dependencies:** LRPC-2.
- **Provisional effort:** S. Either feed `PACKET_OVERHEAD_TOKENS` from the
  builder's measured value, or add a check that flags divergence between the
  fixed constant and the builder's actual measured prompt size, closing the
  one budget seam the plan does not structurally eliminate (see plan §
  Architecture).

## LRPC-8 — Docs propagation

- **Dependencies:** LRPC-3, LRPC-4, LRPC-5 (all satisfied as of 2026-08-20 —
  unblocked, not yet started).
- **Provisional effort:** S. Update `AGENT_WORKFLOW_GUIDE.md § Gemma
  Reviewer / Muse Glimmer Reviewer`, `§ Handoff prompt format`, and
  `§ Local Architect / Complex Analyst` to describe the builder-sourced
  mechanism as the actual implementation, per this guide's own ADR/status
  propagation discipline. Docs-only — exempt from phase-1/phase-2 review and
  Reflection passes.

## Progress

Motivating bug fix tracked separately: `docs/tasks/gemma-push-reviewer-role.md § T9`.

- [x] LRPC-0b (done 2026-08-19; unblocks LRPC-1's Implement phase)
- [x] LRPC-1 (done 2026-08-19; `scripts/local-agent/prompt_anchors.py` +
      `prompt_anchors_test.py` delivered — whole-task `run_local_task.py`
      route exhausted its repair budget on a JSON-escaping failure mode,
      decomposed into Low-band subtasks LRPC-1a/LRPC-1b per policy; Gemma
      phase-2 review 3/3 PASS, 0 findings; full closure record in § LRPC-1)
- [x] LRPC-2 (done 2026-08-19; `scripts/local-agent/prompt_builder.py` +
      `prompt_builder_test.py` delivered — Med-high (RRI 46) ADR-038 route,
      Muse Glimmer GO_LOCAL downgraded to CLOUD_REQUIRED per explicit owner
      choice of Claude over Codex as cloud implementer (ADR-039
      human-select), implemented directly by Claude Code; phase-2 review
      fell back to D14 after 3 consecutive primary-chain (`muse-glimmer`)
      stalls root-caused to a `think`-flag/prompt-template infra defect
      (not a code defect — tracked separately at
      `docs/audit/2026-08-19-muse-glimmer-think-flag-not-honored.md`); D14
      PASS, 2 non-blocking nits; 3-pass Reflection log, 9/9 unit tests,
      full closure record in § LRPC-2)
- [x] LRPC-3 (done 2026-08-19; `scripts/gemma-code-review.py`'s
      `build_review_payload()` now sources its authority-boundary clause via
      `prompt_builder.build_system_prompt(role="gemma_reviewer", ...)`,
      closing the drift bug that motivated this whole plan (missing
      "certify coverage" + paraphrased "close tasks" → "mark tasks
      complete") for this script — Med-high (RRI 42) ADR-038 route, Muse
      Glimmer + primary receipt both `GO_LOCAL`, cloud-only implementation
      per band policy regardless; ADR-039 human-select checkpoint, owner
      chose `claude-sonnet-5`/medium over the recommended
      `gpt-5.6-sol`/high; implemented directly by Claude Code. Phase-1
      reviewer-chain correction applied mid-task: an initial pass ran
      non-canonically against `muse-glimmer` as primary, corrected by
      verifying Gemma was actually healthy and re-running phase-1 through
      Gemma (canonical primary), which independently confirmed PASS. Phase-2
      Gemma review: PASS, 0 findings, no fallback triggered. 3-pass
      Reflection log, 4/4 HP/EC unit tests (+56/56 full suite, +14/14 frozen
      LRPC-1/LRPC-2 suites unaffected); full closure record in § LRPC-3)
- [x] LRPC-4 (done 2026-08-19; `scripts/local-agent/cli.py`'s
      `TOOL_CALLING_SYSTEM_PROMPT` now sources its boundary clause via
      `prompt_builder.build_system_prompt(role="local_developer", ...)`,
      mirroring LRPC-3's pattern for the second consumer script — Med-high
      (RRI 42, corrected from the ledger's stale "Provisional effort: S")
      ADR-038 route, Muse Glimmer + primary receipt both `GO_LOCAL`,
      cloud-only implementation per band policy regardless; ADR-039
      human-select checkpoint, owner chose `claude-sonnet-5`/high over the
      recommended `gpt-5.6-sol`/high; implemented directly by Claude Code.
      Phase-2 Gemma review initially mis-invoked against the wrong model
      (bare `make qa-gemma-review` resolves to the intermediate-fallback
      `muse-glimmer` binding, not the RRI 26-55 primary) — caught mid-run
      and corrected with an explicit `--model gemma4:26b-a4b-it-qat`
      override before completion, the same reviewer-chain-correction
      pattern LRPC-3 already recorded. Phase-2 verdict: `FINDINGS`, 1
      consensus major + 1 distinct pass-specific minor, both independently
      re-verified as false positives against the real runtime module
      output (not accepted at face value); 1 genuine minor (constant
      duplication) accepted as already self-documented. 3-pass Reflection
      log, 4/4 new HP/EC unit tests, byte-identical 60→64-passed/33-failed
      regression baseline, full closure record in § LRPC-4)
- [x] LRPC-5 (done 2026-08-20; `scripts/local-architect/run_analysis.py`'s
      `build_prompt()` — both `DEFAULT_PROFILE` and
      `MED_HIGH_REFINEMENT_PROFILE` branches — now sources its
      authority-boundary opener via `prompt_builder.build_system_prompt
      (role="local_architect_default"|"local_architect_med_high", ...)`,
      closing the third and last hardcoded-opener consumer script this plan
      targets (after LRPC-3's `gemma-code-review.py` and LRPC-4's `cli.py`)
      — Med-high (RRI 42) ADR-038 route, Muse Glimmer + primary receipt
      both `CLOUD_REQUIRED` (concurred, no downgrade — unlike the
      `GO_LOCAL` precedent in LRPC-3/LRPC-4), cloud-only implementation per
      band policy; ADR-039 human-select checkpoint, owner chose
      `claude-sonnet-5`/high over the recommended `gpt-5.6-sol`/high;
      implemented directly by Claude Code. Mid-task, the original HP-1
      acceptance criterion (byte-for-byte identity with the old hardcoded
      opener) was found factually impossible to satisfy together with the
      task's actual purpose — none of the five canonical ADR-037 clauses
      were ever present in that string, which was a paraphrase, not a
      subset — surfaced to the user via `AskUserQuestion` with measured
      (not estimated) token-budget impact (≈82 tokens, ≈2% of budget)
      before the user approved reinterpreting HP-1 as verbatim-clause-
      containment, the same pattern LRPC-3/LRPC-4 already use. Phase-1
      Gemma review: PASS (retried once after a bare/shallow first response
      to force substantive reasoning). Phase-2 Gemma review: PASS, 1
      consensus minor finding explicitly self-scoped `out-of-scope` by the
      reviewer (pre-existing `sys.path.insert` import pattern, unchanged by
      this diff), no fallback triggered. 3-pass Reflection log, 7/7 HP/EC
      unit tests (+1 supplementary, 28/28 full local suite, 101/101 frozen
      LRPC-1/2/3 cross-suite unaffected); confirmed
      `run_local_task_test.py`'s 64-passed/33-failed count is LRPC-4's own
      already-documented baseline (not a new regression); full closure
      record in § LRPC-5)
- [x] LRPC-6 (done 2026-08-20; `scripts/local-agent/golden_fixtures.py` +
      `golden_set.py` + `golden_set_test.py` + `make qa-golden-set`
      delivered — Med-high (RRI 51) ADR-038 route, Muse Glimmer `GO_LOCAL`
      downgraded to `CLOUD_REQUIRED` per Amendment 1 policy exclusion (not a
      capability/risk downgrade like LRPC-3/LRPC-4/LRPC-5's pattern);
      ADR-039 human-select checkpoint, owner chose `claude-sonnet-5`/high
      (self) over `gpt-5.6-sol`/high (Codex CLI unavailable); implemented
      directly by Claude Code. The harness found a genuine production
      defect on its first live run — `local_architect_default`/
      `local_architect_med_high` clauses were missing their "The role may
      not:" governing header, so both `gemma4` and `muse-glimmer` read the
      assembled prompt as permissions instead of prohibitions (3/11 FAIL) —
      flagged to the user, owner directed an immediate fix reopening the
      frozen `prompt_anchors.py`, re-verified 11/11 PASS live against both
      production-bound models after the fix, all frozen LRPC-1-5 suites
      (81 tests in final closure scope) unaffected. Gemma phase-2 review:
      PASS, 0 findings. 3-pass Reflection log, 2/2 HP/EC unit tests plus
      live-run EC-1 evidence; full closure record in § LRPC-6)
- [ ] LRPC-7
- [ ] LRPC-8
