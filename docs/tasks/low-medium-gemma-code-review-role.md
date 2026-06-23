---
type: TaskList
title: "Add Gemma Reviewer role for Low/Moderate code review"
plan: docs/plan/low-medium-gemma-code-review-role.md
status: closed
rri: 54
band: Med-high
effort: L
---
# Tasks: Low/Moderate Gemma Code Review Role

## Objective

Introduce a review-only Gemma role into the agent pipeline so code reviews for
Low and Moderate RRI development tasks include local Gemma Reviewer evidence,
while preserving the primary agent as final reviewer and owner.

## Governing Documents

- `docs/plan/low-medium-gemma-code-review-role.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/RRI_POLICY.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`
- `docs/gemma-local-improve.md`

## RRI

Overall pipeline-change estimate:

**Score: 54 -> Med-high (41-55) -> Effort L -> Balanced -> Premium -> thinking On**

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | review wrapper parser + transport + CLI path | High |
| F files | 2 | wrapper, tests, Makefile/docs; hooks optional | High |
| D domain | 3 | agent workflow and local Ollama integration | High |
| T coverage | 2 | existing delegation tests are relevant but no review-mode tests exist | High |
| A ambiguity | 0 | this task list defines scope and criteria | High |
| K coupling | 3 | workflow docs, local model transport, optional local gate | High |
| P impact | 2 | internal workflow behavior only; no product API/data change | High |
| X context | 3 | scripts, Makefile/hooks, workflow docs, RRI policy | High |

Penalties: `arch_decision` (+12) because this changes the agent pipeline role
model.

## Behavioral coverage contract: unit-v1

Tests for this task are Python (`scripts/gemma_code_review_test.py`,
`scripts/gemma_local_test.py`, and existing `scripts/delegate_low_rri_test.py`),
not Rust.
The Rust-only `.rs::test_name` certification enforced by
`scripts/check-task-unit-coverage.sh` does not apply; completion evidence is the
`python3 -m unittest` run plus the T4 `PASS`/`FINDINGS` fixtures. Sections below
therefore omit a `Type: Development` certification table by design.

## Task Order And Dependencies

```text
T0 -> T1 -> T2 -> T3 -> T4
```

T0 defines the role boundary. T1 implements the read-only wrapper contract. T2
documents workflow requirements. T3 adds the local pipeline entry point. T4
syncs evidence and final status.

---

## T0 - Define Gemma Reviewer semantics

- **Status:** [x] Done
- **Effort:** S
- **RRI:** 23 -> Low
- **Scope:** docs only

### Goal

Define the Gemma Reviewer role in governing docs: trigger, scope, authority
boundary, failure behavior, and relationship to Gemma Developer.

### Acceptance Criteria

- Gemma Reviewer is explicitly review-only.
- Low/Moderate development task code reviews require Gemma Reviewer evidence.
- Gemma Reviewer cannot approve, modify files, certify coverage, or close tasks.
- Gemma-authored Low-RRI patches require independent primary-agent review even
  when Gemma Reviewer also runs.
- Docs consistently use Low (0-25) and Moderate (26-40) RRI terminology.

### Handoff Prompt

T0 - define Gemma Reviewer semantics. Governing docs:
docs/plan/low-medium-gemma-code-review-role.md and this task file. Update only
the role-boundary prose in workflow/policy docs. Stop after docs build with
`make qa-docs`; do not implement scripts.

---

## T1 - Implement review-only Ollama wrapper

- **Status:** [x] Done
- **Effort:** M
- **RRI:** 33 -> Moderate
- **Scope:** `scripts/gemma-code-review.py`, `scripts/gemma_code_review_test.py`,
  `scripts/gemma_local.py`, `scripts/gemma_local_test.py`,
  `scripts/delegate-low-rri.py`, `scripts/delegate_low_rri_test.py`

### Goal

Add a read-only wrapper that sends a review packet and diff to the configured
local Gemma model and parses the tagged finding response. Reuse a shared local
Gemma/Ollama helper module for transport, timeouts, model defaults, generation
options, packet reading, tagged-content normalization, and result writing so
Gemma Developer and Gemma Reviewer do not drift.

### Acceptance Criteria

- CLI resolves `OLLAMA_HOST`, `DUBBRIDGE_REVIEW_MODEL` (review-specific override),
  falling back to `DUBBRIDGE_LOW_RRI_MODEL`, then the repo default.
- CLI accepts a pre-built packet file (assembled by the caller/orchestrator, not
  by the wrapper) and writes a structured result artifact.
- The model prompt forbids patches, file bodies, approvals, and task closure.
- Parser accepts `STATUS: PASS`, `STATUS: FINDINGS`, and `STATUS: BLOCKED`.
- Parser rejects missing markers, invalid severities, duplicate malformed blocks,
  extra text outside the contract, and patch-like output.
- Wrapper computes the changed-path set from the diff embedded in the packet and
  labels any finding whose `PATH` is not in that set as `out-of-scope`; it does
  not drop it. Semantic in-scope judgment remains with the primary agent.
- Exit code contract (per plan D6): exit `0` for both `PASS` and `FINDINGS`
  (result artifact written); exit non-zero only for operational failure (Ollama
  unavailable, invalid/truncated response, `STATUS: BLOCKED`).
- Transport timeout behavior mirrors `scripts/delegate-low-rri.py`, including
  idle-timeout, max-wall cap, and `done_reason == "length"` truncation detection.
- Unit tests cover parser, validation, dry-run payload, out-of-scope labeling,
  timeout/error mapping, and truncation detection.
- Shared helper tests cover role-neutral payload construction, timeout behavior,
  env boolean parsing, packet reading, tagged-content helpers, and atomic result
  writing.

### Happy Paths Considered

- **HP-1:** valid diff with no findings -> `STATUS: PASS` parsed and reported.
- **HP-2:** valid diff with one blocking finding -> finding is parsed with path,
  line, severity, detail, and suggestion.
- **HP-3:** Ollama returns `STATUS: BLOCKED` with a concise reason -> wrapper
  reports blocked without applying or modifying anything.

### Edge Cases Considered

- **EC-1:** model returns a unified diff or complete file contents -> parser
  rejects the response.
- **EC-2:** finding path is outside the changed-path set of the reviewed diff ->
  wrapper labels the finding `out-of-scope` (does not drop it); primary agent
  decides whether it represents a direct regression.
- **EC-3:** malformed response or missing end marker -> wrapper exits non-zero
  and records validation failure.
- **EC-4:** Ollama/model unavailable -> wrapper exits with a stable code and
  does not mask the missing review evidence.

### Handoff Prompt

T1 - implement the Gemma Reviewer wrapper. Governing docs:
docs/tasks/low-medium-gemma-code-review-role.md and
docs/plan/low-medium-gemma-code-review-role.md. Files:
scripts/gemma-code-review.py, scripts/gemma_code_review_test.py,
scripts/gemma_local.py, scripts/gemma_local_test.py, scripts/delegate-low-rri.py,
and scripts/delegate_low_rri_test.py. Acceptance: tagged review contract (D3 in
plan), read-only behavior, DUBBRIDGE_REVIEW_MODEL env var, pre-built packet
input, out-of-scope labeling, D6 exit-code contract, idle/wall/truncation
timeouts, parser validation, dry-run support, shared local Gemma helper reuse,
and unit tests. Stop after `python3 -m unittest scripts.gemma_local_test`,
`python3 -m unittest scripts/delegate_low_rri_test.py`, and
`python3 -m unittest scripts/gemma_code_review_test.py`.

---

## T2 - Wire Gemma Reviewer into workflow documentation

- **Status:** [x] Done
- **Effort:** S
- **RRI:** 23 -> Low
- **Scope:** workflow and policy docs

### Goal

Update the agent workflow so Low and Moderate development-task code reviews
include Gemma Reviewer evidence before completion reporting.

### Acceptance Criteria

- `AGENT_WORKFLOW_GUIDE.md` states when Gemma Reviewer runs and places it as an
  advisory input to the existing Reflection cycle (D7 in plan), not a separate step.
- `RRI_POLICY.md` states that the Low band still separates Gemma Developer from
  Gemma Reviewer.
- `HITL_AUTONOMY_POLICY.md` states Gemma Reviewer is required-when-available (D8):
  absence of Ollama never blocks task completion and does not open an extra gate.
- `LOW_RRI_LOCAL_MODEL_HANDOFF.md` explains the relationship between local patch
  delegation and local review.
- `docs/gemma-local-improve.md` summarizes the active review-only contract.
- Completion evidence includes the `Gemma Reviewer evidence` block.

### Handoff Prompt

T2 - wire Gemma Reviewer into workflow docs. Governing docs:
docs/tasks/low-medium-gemma-code-review-role.md and
docs/plan/low-medium-gemma-code-review-role.md. Update only the named docs.
Stop after `make qa-docs`; do not change scripts or hooks.

---

## T3 - Add local pipeline entry point

- **Status:** [x] Done
- **Effort:** M
- **RRI:** 40 -> Moderate
- **Scope:** `Makefile`; optional local hook only if T0 chooses hook enforcement

### Goal

Provide a consistent local command for agents and maintainers to run Gemma
Reviewer against the current task diff.

### Acceptance Criteria

- `make qa-gemma-review` invokes the review wrapper with a pre-built packet.
- The target is local-only by default and is not added to GitHub-hosted `qa-ci`
  until an Ollama-capable runner is available.
- The target exits non-zero (operational failure) when Ollama/model is unavailable,
  and exits 0 when the wrapper returns `PASS` or `FINDINGS` (per plan D6).
- The target can be skipped by setting `DUBBRIDGE_SKIP_GEMMA_REVIEW=1`; skipped
  review evidence must be reported by the primary agent.
- If hook integration is added, it is conditional on model availability and does
  not make unrelated docs-only pushes fail for lack of Ollama.

### Happy Paths Considered

- **HP-1:** developer runs the target with Ollama available -> review result
  artifact is produced.
- **HP-2:** no development diff is present -> target reports no reviewable code
  changes and exits cleanly.

### Edge Cases Considered

- **EC-1:** Ollama unavailable -> target fails with a clear message and does not
  pretend review passed.
- **EC-2:** diff includes docs-only changes -> target skips because Gemma
  Reviewer applies to code review, not docs-only review.
- **EC-3:** GitHub-hosted CI lacks Ollama -> CI remains green because this gate is
  not mandatory remotely until runner support exists.

### Handoff Prompt

T3 - add local Gemma Reviewer pipeline command. Governing docs:
docs/tasks/low-medium-gemma-code-review-role.md and
docs/plan/low-medium-gemma-code-review-role.md. Files: Makefile and optionally
local hook files if T0 approved hook enforcement. Stop after the target is
tested in dry-run mode and existing `make qa-docs` still passes.

---

## T4 - Status sync and end-to-end review evidence

- **Status:** [x] Done
- **Effort:** S
- **RRI:** 24 -> Low
- **Scope:** task ledger, plan, local evidence docs

### Goal

Record final evidence that the new role is implemented, tested, and integrated
without changing approval ownership.

### Acceptance Criteria

- This task ledger records completion evidence for T0-T3.
- At least one synthetic review fixture demonstrates `PASS`.
- At least one synthetic review fixture demonstrates `FINDINGS`.
- Final docs state how agents report skipped or blocked Gemma Reviewer runs.
- `make qa-docs` passes.
- Relevant script unit tests pass.

### Handoff Prompt

T4 - close out Gemma Reviewer role integration. Governing docs:
docs/tasks/low-medium-gemma-code-review-role.md and
docs/plan/low-medium-gemma-code-review-role.md. Record evidence for every
accepted task and run the documented checks. Stop after status docs are synced;
do not start unrelated workflow changes.

### Completion Evidence

**T0** — Role boundary prose added to `AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer`,
`RRI_POLICY.md § Gemma Developer vs. Gemma Reviewer`, `HITL_AUTONOMY_POLICY.md §
Gemma Reviewer availability`, `LOW_RRI_LOCAL_MODEL_HANDOFF.md § Patch delegation
vs. code review`, and `docs/gemma-local-improve.md`. All docs consistently use
Low (0–25) / Moderate (26–40) RRI band terminology. Gemma Reviewer is defined as
review-only with no file-write or approval authority.

**T1** — `scripts/gemma-code-review.py` (review-only wrapper, review contract
parser, out-of-scope labeling, D6 exit-code contract, dry-run, idle/wall/truncation
timeouts) and `scripts/gemma_local.py` (shared transport, timeouts, bool_from_env,
read_packet, build_chat_payload, stream_chat, normalize_tagged_content,
write_result) implemented and tested. `scripts/gemma_code_review_test.py` and
`scripts/gemma_local_test.py` cover parser, validation, dry-run payload,
out-of-scope labeling, timeout/error mapping, truncation detection, and shared
helpers.

**T2** — `AGENT_WORKFLOW_GUIDE.md` states Gemma Reviewer runs after implementation,
feeds the Reflection cycle as advisory input, and defines the completion evidence
block. `RRI_POLICY.md` distinguishes Gemma Developer from Gemma Reviewer.
`HITL_AUTONOMY_POLICY.md` states absence of Ollama never opens a human approval
gate. `LOW_RRI_LOCAL_MODEL_HANDOFF.md` explains the patch-delegation vs. review
split. `docs/gemma-local-improve.md` documents both local contracts.

**T3** — `make qa-gemma-review` target already present in `Makefile`. Skippable
via `DUBBRIDGE_SKIP_GEMMA_REVIEW=1`. Exits 0 for `PASS`/`FINDINGS`, non-zero for
operational failure. Docs-only diffs are skipped automatically. Not wired into
`qa-ci` (no Ollama on GitHub-hosted runners).

**Synthetic fixtures via unit tests** (`scripts/gemma_code_review_test.py`):

- `PASS` fixture: `test_parse_response_pass` — valid `STATUS: PASS` with summary,
  no findings; asserts `status == "pass"` and empty findings list.
- `FINDINGS` fixture: `test_parse_response_findings` — `STATUS: FINDINGS` with one
  complete finding block; asserts path, line, severity, detail, suggestion, and
  `scope` label resolved correctly.

**Skipped/blocked Gemma evidence reporting** — documented in
`AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer` and `HITL_AUTONOMY_POLICY.md §
Gemma Reviewer availability`. When Ollama is unavailable the agent records
`Status: BLOCKED` in the `### Gemma Reviewer evidence` block and proceeds with
normal primary-agent Reflection; no extra gate is opened.

**Verification:**

```
make qa-docs                        # passed
python3 -m unittest scripts.gemma_local_test scripts.gemma_code_review_test
# 33 tests, OK
```
