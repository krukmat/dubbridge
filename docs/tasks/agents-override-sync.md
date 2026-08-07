---
type: TaskList
title: "Tasks: AGENTS.override.md generation and drift guard"
status: proposed
plan: docs/plan/agents-override-sync.md
---
# Tasks: AGENTS.override.md generation and drift guard

Governing plan: `docs/plan/agents-override-sync.md`
Governing guides: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/RRI_POLICY.md`,
`docs/policies/HITL_AUTONOMY_POLICY.md`
Related: `docs/tasks/antares-security-specialist-advisor.md` § T6,
`docs/tasks/doc-consistency-guardrails.md` (precedent pattern)

## Status Legend
- [ ] Not started · [x] Done · [~] In progress · [!] Blocked

Build order: **AOS1** (single task; no decomposition required at RRI 51).

---

## Task AOS1 — Generate AGENTS.override.md + wire drift check

**Status:** `[x] Done` — 2026-08-07

**Effort:** L (RRI band-derived, Med-high) · **Depends on:** nothing

### Continuation analysis (2026-08-07)

Written to let a future session resume this task without re-deriving context.
Nothing below changes scope, acceptance criteria, or the RRI — it sharpens the
evidence the original plan/task ledger was written against, using a direct
read of the current repository state.

**1. Why the hash-attestation gate does not already catch this drift.**
`scripts/agent-preflight.py::hash_source_file` (line 184) computes a SHA-256
of `AGENTS.override.md` *as it exists at receipt-generation time* and records
it into the v2 receipt's `native_instruction.sha256` field (line 219-220).
`_reverify_document_hashes` (line 301) later re-hashes the file and compares
against that recorded value — but only to detect whether the file changed
**since the receipt was issued**, i.e. mid-session tampering or staleness of
the receipt itself. It has no notion of, and never checks, whether
`AGENTS.override.md`'s content actually matches `AGENTS.md` +
`AGENT_WORKFLOW_GUIDE.md` + `HITL_AUTONOMY_POLICY.md` at that moment. In other
words: the hash gate certifies "Codex read the same bytes this receipt
claims it read," not "those bytes are current relative to their sources."
This is why the T6 drift (stale pre-T4/T5 Antares text sitting in
`AGENTS.override.md` while `AGENT_WORKFLOW_GUIDE.md` had already moved past
it) passed every existing gate silently — `make qa-docs` has zero notion of
`AGENTS.override.md` (confirmed: `scripts/check-doc-consistency.sh` defines
only `check_status_parity_and_completeness`, `check_dangling_refs`, and
`check_superseded_successors` — no function references
`AGENTS.override.md` at all), and the hash gate is structurally incapable of
catching it. AOS1's drift check is the first mechanism that would.

**2. Current file state differs slightly from the plan's original framing —
verified 2026-08-07.** The plan (`docs/plan/agents-override-sync.md`) states
`AGENTS.override.md` contains "zero occurrences" of HITL Autonomy Policy
text. A fresh `grep -n "HITL_AUTONOMY_POLICY"` against the current file
(1710 lines) finds **three matches** (lines 13, 212, 1388) plus one more in
the closing `## Related` list (line 1707) — but all four are inline
*citations* to `docs/policies/HITL_AUTONOMY_POLICY.md` inherited verbatim
from `AGENTS.md`'s and `AGENT_WORKFLOW_GUIDE.md`'s own prose (e.g. "see
`docs/policies/HITL_AUTONOMY_POLICY.md`"). There is still no standalone `#
Human-in-the-Loop (HITL) Autonomy Policy` section — the file's content is
confirmed to end at `AGENT_WORKFLOW_GUIDE.md`'s own closing `## Related`
block (verified by reading the last 40 lines of the file: it ends on
"`docs/gemma-local-improve.md` — active local Gemma contract summary", which
is `AGENT_WORKFLOW_GUIDE.md`'s last line). **The gap HP-2/the plan's
"missing source" problem is real and unchanged** — Codex has never received
the actual policy text, only other documents' references to its filename.
This distinction (references-to vs. content-of) is worth restating precisely
if this task is picked up later, since a shallow re-check ("grep finds HITL,
so it's fine") would wrongly conclude the gap is already closed.

**3. Seam format is confirmed reusable.** Read `AGENTS.override.md:215-226`
directly: the seam between `AGENTS.md` content and `AGENT_WORKFLOW_GUIDE.md`
content is a bare `---` line immediately followed by the next document's own
YAML frontmatter (`type: Playbook`, `title: "Agent Workflow Guide"`, ...) and
then its `# Agent Workflow Guide` heading. The generator should reproduce
exactly this pattern for the `HITL_AUTONOMY_POLICY.md` seam: `---` + that
file's own frontmatter + its `# Human-in-the-Loop (HITL) Autonomy Policy`
heading, appended after the current EOF. This keeps the one-time
regeneration diff a pure tail-addition, matching HP-2's "no unrelated
reformatting" acceptance bar and making the eventual code review trivial to
verify by eye (diff should show only new lines at the end of the file, zero
changed lines above the current EOF).

**4. Scale for the implementer.** Source files: `AGENTS.md` (219 lines),
`AGENT_WORKFLOW_GUIDE.md` (1491 lines), `HITL_AUTONOMY_POLICY.md` (328
lines) — combined ~2038 lines, consistent with why `AGENTS.override.md` will
grow from 1710 to roughly 2040 lines after regeneration.
`scripts/check-doc-consistency.sh` itself is 239 lines with three existing
check functions in a flat dispatch pattern (each check function is called
unconditionally near the bottom of the file, violations accumulate in a
shared `violations` variable via `add_violation`, and the script exits 1 if
any accumulated) — the new drift-check function should follow that exact
dispatch pattern (define function, call it alongside the other three, let it
call `add_violation` on mismatch) rather than introducing a second
violation-reporting mechanism.

**5. Nothing here changes the routing decision.** This remains RRI 51 /
Med-high, ungated for implementation until the Compact Approval Task Card is
presented and explicitly approved by the user, after which it follows the
ADR-038 gate (Qwen27 advisory refinement → hash-bound route receipt → bounded
local session or cloud escalation) — see `docs/policies/HITL_AUTONOMY_POLICY.md
§ Med-high Architect-refined single-attempt gate (RRI 41–55)`. No
implementation has started as of this analysis.

### Objective
Turn `AGENTS.override.md` from a manually-mirrored duplicate into a generated
artifact with an enforced freshness gate, and close its existing content gap
(missing `HITL_AUTONOMY_POLICY.md` section) in the same change.

### Happy paths considered
- **HP-1:** `AGENT_WORKFLOW_GUIDE.md` (or `AGENTS.md`, or `HITL_AUTONOMY_POLICY.md`)
  is edited, `scripts/generate-agents-override.py` is run, and the regenerated
  `AGENTS.override.md` is byte-identical to the fixed concatenation of the three
  current source files with the `---` seam — `make qa-docs` passes afterward.
- **HP-2:** After the one-time regeneration, `AGENTS.override.md` contains the
  `HITL_AUTONOMY_POLICY.md` content (verifiable via
  `grep -c "Human-in-the-Loop (HITL) Autonomy Policy" AGENTS.override.md` -> `1`),
  closing the gap found during investigation.
- **HP-3:** Running the generator twice in a row with no source changes produces
  byte-identical output both times (idempotent) — no spurious drift-check failure
  from running it more than once.

### Edge cases considered
- **EC-1:** `AGENTS.override.md` is hand-edited (or a source file changes) without
  re-running the generator — the new drift check in
  `scripts/check-doc-consistency.sh` fails closed with a message naming the exact
  command to fix it (`scripts/generate-agents-override.py`), and `make qa-docs`
  exits non-zero.
- **EC-2:** One of the three source files (`AGENTS.md`, `AGENT_WORKFLOW_GUIDE.md`,
  `HITL_AUTONOMY_POLICY.md`) is missing or empty at generation time — the generator
  exits non-zero with a clear error and does **not** write a partial or empty
  `AGENTS.override.md`.
- **EC-3:** The drift check runs in a repo state where `AGENTS.override.md` does
  not exist yet (e.g. a fresh checkout that somehow lost the file) — it fails
  closed with the same "stale, regenerate" message rather than silently passing or
  crashing with an unhandled exception.

### Acceptance criteria
- `scripts/generate-agents-override.py` exists, is deterministic (no LLM calls),
  and implements HP-1/HP-3 and EC-2.
- `scripts/check-doc-consistency.sh` gains a drift-check function reusing the
  generator's output (not a re-implementation of the concatenation logic) and
  implements EC-1/EC-3; it is exercised by the existing `qa-docs` Makefile target
  with no new Makefile target added.
- `AGENTS.override.md` is regenerated once as part of this task; the only content
  delta versus the pre-task file is the addition of the
  `HITL_AUTONOMY_POLICY.md` section (HP-2) — no unrelated reformatting.
- `make qa-docs` passes after the regeneration.

### Evidence to emit
- `git diff --stat` for the regenerated `AGENTS.override.md` (expected: pure
  addition, no reordering of existing lines).
- `make qa-docs` full output (showing the new check running and passing).
- A deliberately-introduced drift (e.g. a throwaway edit to
  `AGENT_WORKFLOW_GUIDE.md` without regenerating) demonstrating the check fails
  closed, then reverted/regenerated to show it passes again.

### Status artifacts affected
- `docs/tasks/antares-security-specialist-advisor.md` § T6 — add a forward
  reference noting the manual-mirror gap T6 hit is now closed by this task's
  drift check (informational only; does not reopen T6).
- No ADR change; no roadmap slice change (cross-cutting tooling, not on the
  media-pipeline sequence).

### Files affected
- `scripts/generate-agents-override.py` (new)
- `scripts/check-doc-consistency.sh` (new drift-check function)
- `AGENTS.override.md` (regenerated)

---

### RRI

```
python3 scripts/rri.py \
  --touches scripts/generate-agents-override.py \
  --touches scripts/check-doc-consistency.sh \
  --touches AGENTS.override.md \
  --touches docs/plan/agents-override-sync.md \
  --touches docs/tasks/agents-override-sync.md \
  --cc 8 --D 2 --K 3 --P 2 --T 3 --A 1 --X 2 \
  --penalty arch_decision
```

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 8 (estimated: generator ~3-4 branches, drift-check ~4-5 branches) -> score 1 | High |
| F files | 2 | 5 touched paths | High |
| D domain | 2 | agent-supplied — no anchor-rubric match for `scripts/`; judged as normal business logic (concatenation + diff), not pure formatting, because it feeds a governance hash-attestation gate | High |
| T coverage | 3 | agent-supplied — new mechanism, no existing tests, but sits inside an already-exercised script family (`check-doc-consistency.sh` has prior test precedent via G2) | High |
| A ambiguity | 1 | agent-supplied — explicit acceptance criteria, HP/EC set, and design decisions recorded above | High |
| K coupling | 3 | agent-supplied — filesystem I/O + integration into an existing CI-facing gate (`make qa-docs`) | High |
| P impact | 2 | agent-supplied — changes internal dev-tooling behavior only; not public API, auth, or persisted user data | High |
| X context | 2 | agent-supplied — must hold ~5 docs/scripts in mind (3 sources + 2 scripts) | High |

**Base value:** 100 x (weighted / 5) = 39
**Penalties applied:** `arch_decision` (+12) — this changes the maintenance model of
Codex's hash-attested native-instruction source from hand-maintained to generated,
a process/governance-tooling decision, not just a code change.
**Final RRI:** 51 -> band **Med-high (41-55)** -> Effort L
**Gates for this band:** Plan + explicit acceptance criteria required before
approval (satisfied above); ADR-038 Architect-refined single-attempt implementation
gate; band-routed peer review (phases 1 and 2); 3 Reflection passes.
**Decomposition:** not triggered.

---

## Agent handoff prompt (for delegation)

```
Task: AOS1 — docs/tasks/agents-override-sync.md
Plan: docs/plan/agents-override-sync.md

File + line range: AGENTS.override.md:1-1710 (read-only reference for seam format
at line 225); new files scripts/generate-agents-override.py,
scripts/check-doc-consistency.sh (add function, do not restructure existing checks).

Acceptance criteria: see "Acceptance criteria" above (bullets only).

Stop condition: after `make qa-docs` passes and the regenerated
AGENTS.override.md's diff is pure-addition (HITL section only), stop and report.
Do not touch AGENT_WORKFLOW_GUIDE.md, HITL_AUTONOMY_POLICY.md, or AGENTS.md content
— this task only changes how AGENTS.override.md is produced/verified.
```

---

## Closure record

### ADR-038 routing evidence

- Qwen27 advisory refinement (`med-high-refinement-v1`): `route_recommendation: GO_LOCAL`.
  Artifact: `docs/audit/med-high/aos1/refinement_artifact.json` (packet SHA-256
  `c9c06859ce300640d3441c6e6a3becb4c44db063a253c3d9e2b15c5e57df3ad0`, model
  `qwen3.6:27b-q4_K_M`, resolved digest matches expected digest).
- Primary hash-bound route receipt: `decision: GO_LOCAL` (concurred with Qwen27;
  no auth/security/rights/schema/unresolved-ADR exclusion applies per ADR-038 §6).
- `med_high_gate.py` evaluation: `route: GO_LOCAL`, reason "Qwen27 and primary
  both recommend GO_LOCAL."
- Bounded local session (`run_med_high_task.py`, `qwen3.6:35b-a3b`, own process
  group, ≤8 turns/≤300s/0 repairs): stopped at turn 6/8 with
  `status: wall_clock_exceeded` after 300.02s. No diff had been written to the
  worktree at that point (still in the read/context-gathering phase) — nothing
  to salvage. Per ADR-038, Med-high carries **zero** repair attempts; this
  routed directly to cloud escalation, exactly as designed, not as a deviation.
  Escalation bundle: `docs/audit/aos1-med-high-escalation.md`.
- Cloud implementation: Claude Code (Sonnet 5), this session, using the same
  approved task card, `allowed_paths`, and acceptance criteria the local
  session would have used.

### Reflection log

Required passes: 3 (`51` → `Med-high`)

#### Pass 1 — contract fidelity

- **Draft verdict:** Generator implements plain concatenation (no separator),
  fail-closed on missing/empty source, deterministic, `--write`/stdout-default
  split matches the approved packet exactly.
- **Critique findings:** `--check` flag is a no-op alias of the default
  (intentional, matches plan's "default (no flags, or `--check`)" phrasing) —
  not a defect.
- **Revisions applied:** none.

#### Pass 2 — drift-check correctness

- **Draft verdict:** Drift-check function reuses generator output via
  subprocess (no re-implementation), integrates into the existing flat
  dispatch pattern, handles `set -euo pipefail` correctly via `if ! ...`.
- **Critique findings:** the original `$(...)`-based string comparison strips
  trailing newlines from both sides symmetrically, so pure trailing-blank-line
  drift at EOF would be invisible to the check.
- **Revisions applied:** initially accepted as an out-of-threat-model
  limitation; the phase-2 reviewer disagreed given `AGENTS.override.md` is a
  hashed attestation source, and this was accepted as correct on review — see
  Peer Reviewer evidence below. Fixed by switching to a `cmp -s` byte-exact
  comparison against a temp file instead of bash string comparison.

#### Pass 3 — idempotence/coverage

- **Draft verdict:** HP-1/HP-2/HP-3 covered by `generate_agents_override_test.py`;
  EC-1/EC-3 covered by `check_doc_consistency_agents_override_test.py`; EC-2
  covered at the unit level with mocks.
- **Critique findings:** EC-2 coverage for `--write` mode relied only on
  mocked `Path.write_text`, not a real filesystem write attempt — a real
  emptied/missing source file was never exercised end-to-end for `--write`.
- **Revisions applied:** added `GenerateAgentsOverrideRealFilesystemTest`
  (`test_ec2_write_mode_does_not_modify_output_on_empty_source`,
  `test_ec2_write_mode_does_not_create_output_on_missing_source`) exercising
  real subprocess invocations against a disposable git worktree.

### Peer Reviewer evidence

- Reviewer: `qwen3.6:27b-q4_K_M`
- Command: manual `/api/chat` invocation (Ollama, `think: false`,
  `num_ctx: 32768`), packet = full task diff
- Artifact (round 1): `docs/audit/med-high/aos1/phase2_review_round1.json` —
  **BLOCKED**, 4 findings (1 disputed as likely-false-positive and later
  vindicated by fix; 1 accepted and fixed — trailing-newline drift blind spot;
  1 rejected as out-of-scope filesystem corruption; 1 factually incorrect —
  reviewer claimed `check-doc-consistency.sh` had no `set -e`, but line 2 is
  `set -euo pipefail`, unchanged by this diff)
- Artifact (round 2, after fix): `docs/audit/med-high/aos1/phase2_review_round2.json`
  — **PASS**, 0 findings
- Verdict: `PASS` (round 2, after the trailing-newline byte-exactness fix)
- Findings: trailing-newline-only drift was invisible to a `$(...)`-based
  string comparison; fixed by switching to `cmp -s` byte-exact comparison;
  confirmed fixed by a new regression test
  (`test_ec1_check_catches_trailing_newline_only_drift`) and by the round-2
  re-review
- Gemma fallback: not triggered — `qwen3.6:27b-q4_K_M` available both rounds
- D14 fallback: not triggered
- disposition_divergence: `partial` — 1 of 4 round-1 findings accepted and
  fixed, 1 rejected (scope), 1 disputed-then-vindicated (real tests added), 1
  corrected as a reviewer factual error
- Primary-agent disposition: 1 finding fixed (byte-exact `cmp` comparison), 1
  finding's coverage gap closed with real-filesystem tests (already-correct
  behavior, now with stronger evidence), 1 finding rejected as out of the
  task's threat model (directory/permission corruption on the output path), 1
  finding corrected (reviewer misread the diff hunk in isolation from the
  full file header)

Task-analysis review: `qwen3.6:27b-q4_K_M` `docs/audit/med-high/aos1/phase1_review.json` - PASS
Code-solution review: `qwen3.6:27b-q4_K_M` `docs/audit/med-high/aos1/phase2_review_round2.json` - PASS

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | generator output byte-identical to fixed concatenation; `make qa-docs` passes | `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest::test_hp1_generate_matches_fixed_concatenation`, `scripts/check_doc_consistency_agents_override_test.py::AgentsOverrideDriftCheckTest::test_hp1_check_passes_after_regeneration` | passed |
| HP-2 | Happy path | regenerated AGENTS.override.md contains HITL_AUTONOMY_POLICY.md content | `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest::test_hp2_generate_includes_hitl_source` | passed |
| HP-3 | Happy path | generator is idempotent; no separator inserted between sources | `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest::test_hp3_generate_is_idempotent`, `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest::test_hp3_no_separator_inserted_between_sources` | passed |
| EC-1 | Edge case | hand-edited/stale AGENTS.override.md fails closed, names fix command; catches byte-level (not just content) drift | `scripts/check_doc_consistency_agents_override_test.py::AgentsOverrideDriftCheckTest::test_ec1_check_fails_closed_on_hand_edited_drift`, `::test_ec1_check_catches_trailing_newline_only_drift`, `::test_ec1_fix_then_pass_round_trip` | passed |
| EC-2 | Edge case | missing or empty source file exits non-zero, no partial/empty write | `scripts/generate_agents_override_test.py::GenerateAgentsOverrideTest::test_ec2_missing_source_file_exits_nonzero`, `::test_ec2_empty_source_file_exits_nonzero`, `::test_ec2_missing_source_does_not_call_generate_further`, `scripts/generate_agents_override_test.py::GenerateAgentsOverrideRealFilesystemTest::test_ec2_write_mode_does_not_modify_output_on_empty_source`, `::test_ec2_write_mode_does_not_create_output_on_missing_source` | passed |
| EC-3 | Edge case | drift check fails closed when AGENTS.override.md does not exist | `scripts/check_doc_consistency_agents_override_test.py::AgentsOverrideDriftCheckTest::test_ec3_check_fails_closed_when_file_missing` | passed |

### Owner final verification

- Owner: `claude-code-sonnet-5` (orchestrator of record for this session; RRI
  51 Med-high task with explicit user approval — see approval in session
  transcript, "aprobado")
- Date: `2026-08-07`
- Statement: I verified every happy path and edge case defined for this task
  has unit test evidence that replicates the expected behavior, that
  `make qa-docs` passes including the new drift check, that the regenerated
  `AGENTS.override.md` is a byte-exact match of
  `cat AGENTS.md docs/playbooks/AGENT_WORKFLOW_GUIDE.md docs/policies/HITL_AUTONOMY_POLICY.md`
  (verified via `diff`, zero output), and that the diff against the pre-task
  file is a pure 328-line addition with zero removed or reordered lines.
- Commands run:
  `python3 -m unittest scripts.generate_agents_override_test scripts.check_doc_consistency_agents_override_test`
  (16 tests, all passed); `make qa-docs` (passed, including
  `check_agents_override_drift`); `python3 scripts/generate-agents-override.py --write`;
  `diff <(cat AGENTS.md docs/playbooks/AGENT_WORKFLOW_GUIDE.md docs/policies/HITL_AUTONOMY_POLICY.md) AGENTS.override.md`
  (empty); `git diff --stat AGENTS.override.md` (328 insertions, 0 deletions).

Reviewability budget: n/a — this band (26–55) routes to `qwen3.6:27b-q4_K_M`
peer review, not the RRI 0–25 Gemma reviewability-budget gate.
