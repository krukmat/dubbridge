---
type: TaskList
title: "Add before-after delegation mode for files over 400 lines"
plan: docs/evaluations/large-file-delegation-2026-06-21.md
status: approved
rri: 65
band: Complex
effort: L
---

# Tasks: large-file-delegation-before-after

## Objective

Make local Gemma delegation safe for large files by replacing the full-file
regeneration contract with an exact-block replacement protocol for files > 400 lines.
Fix the silent-truncation corruption root cause in `stream_chat` independently of
the new mode.

## Governing documents

- `docs/evaluations/large-file-delegation-2026-06-21.md` — diagnosis, proposed CLI
  shape, algorithm, acceptance criteria
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — mandatory workflow
- `docs/policies/RRI_POLICY.md` — bands, gates, decomposition
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md` — mode-selection rule (orchestrator)

## RRI

**Score: 65 → Complex (56–70) → Effort L → Premium → thinking On**

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | raw CC 18 → score 2 | High |
| F files | 1 | 2 files touched | High |
| D domain | 3 | delegation contract + token-limit semantics | High |
| T coverage | 2 | no existing gate for the new mode | High |
| A ambiguity | 2 | new response contract + silent edge cases | High |
| K coupling | 3 | Ollama stream + done_reason + parser + git apply | High |
| P impact | 3 | current silent failure destroys whole files | High |
| X context | 2 | Gemma state, token limits, Ollama behaviour | High |

Penalties: arch_decision (+12), refactor_and_behavior (+8)

## Behavioral coverage contract: unit-v1

## Task order and dependencies

```
T1 → T2 → T3 → T4
```

T1 is a prerequisite for T3 (done_reason guard must exist before the replacement
algorithm references it). T2 is a prerequisite for T3 (parser must exist before
main() wires it). T4 can only run after T3 is complete.

---

## T1 — `done_reason` fail-closed guard in `stream_chat`

- **Status:** [x] Done
- **Effort:** S
- **Scope:** `stream_chat` function in `scripts/delegate-low-rri.py` (~3–5 lines)

### Goal

Capture `done_reason` from the final NDJSON stream chunk. Raise `RuntimeError`
before building any diff when `done_reason == "length"`. This is the root-cause fix
for the silent truncation corruption recorded in `result_workspace_dead_if.json`
and applies to both `full-file` and `before-after` modes.

### Handoff prompt

Task T1 of large-file-delegation-before-after.
Governing docs: docs/tasks/large-file-delegation-before-after.md,
docs/evaluations/large-file-delegation-2026-06-21.md.
File + line range: scripts/delegate-low-rri.py — `stream_chat` function (~L278–337).
Acceptance criteria:
- record `done_reason` from the final stream chunk where `chunk.get("done") == True`;
- if `done_reason == "length"`, raise RuntimeError("response cut by token limit; output may be truncated") before returning content;
- return content normally for any other done_reason value;
- stream_chat must also return done_reason so callers can inspect it;
- existing behaviour for idle/wall timeout and URLError is unchanged.
Stop: commit T1 only. Do not start T2.

### Happy paths considered

- **HP-T1-1:** Stream ends with `done_reason == "stop"` and valid content → `stream_chat` returns content normally; caller proceeds to parse.
- **HP-T1-2:** `done_reason` is absent in the final chunk (old Ollama) → treated as non-length termination; content returned normally.

### Edge cases considered

- **EC-T1-1:** `done_reason == "length"` → `RuntimeError` raised before any content is returned; no diff built, no file written.
- **EC-T1-2:** `done_reason == "length"` in `full-file` mode → same guard fires; existing `full-file` callers are protected without code changes to their paths.
- **EC-T1-3:** Multiple chunks received before `done`; `done_reason` only present on the final `done: true` chunk → only that chunk is inspected.

---

## T2 — `before-after` response parser and validation

- **Status:** [x] Done
- **Effort:** M
- **Scope:** new functions in `scripts/delegate-low-rri.py`; new system-prompt branch

### Goal

Implement `parse_replacement_response()` and `validate_replacement_payload()` for
the `before-after` contract:

```
STATUS: PATCH|NO_PATCH|BLOCKED
SUMMARY: …
=== REPLACEMENT START ===
PATH: <repo-relative path>
--- AFTER ---
<replacement block only>
=== REPLACEMENT END ===
```

Validation must reject: missing closing marker, extra text, multiple blocks,
mismatched PATH, PATH outside `--allow-path`, `STATUS: PATCH` without AFTER block.

### Handoff prompt

Task T2 of large-file-delegation-before-after.
Governing docs: docs/tasks/large-file-delegation-before-after.md,
docs/evaluations/large-file-delegation-2026-06-21.md §Proposed before-after response contract.
File + line range: scripts/delegate-low-rri.py — add after existing parse_tagged_response (~L351).
Acceptance criteria (see §T2 edge cases below).
Stop: commit T2 only. Do not start T3.

### Happy paths considered

- **HP-T2-1:** Well-formed response with `STATUS: PATCH`, single `=== REPLACEMENT START/END ===` block, valid PATH, and AFTER block → parsed payload returned; no error.
- **HP-T2-2:** `STATUS: NO_PATCH` with no replacement block → parsed as a no-op; no error.

### Edge cases considered

- **EC-T2-1:** `=== REPLACEMENT END ===` absent (truncated) → error raised before payload returned.
- **EC-T2-2:** PATH value does not match `--allow-path` → scope error before diff.
- **EC-T2-3:** `STATUS: PATCH` with no AFTER block → error.
- **EC-T2-4:** Multiple `=== REPLACEMENT START ===` blocks → error.
- **EC-T2-5:** Extra text outside permitted tagged sections → error.

---

## T3 — Replacement algorithm and CLI integration

- **Status:** [x] Done
- **Effort:** M
- **Scope:** new `apply_before_after()`, CLI flags `--mode`, `--target-path`,
  `--before-file`; `main()` dispatch

### Goal

Implement the literal find-and-replace algorithm and wire it into `main()`:

1. Read `--before-file`; verify it occurs exactly once in the current target file.
2. In deletion mode: derive AFTER from BEFORE by removing marked lines; verify
   AFTER is a line-subset of BEFORE.
3. In replacement mode: use model-returned AFTER block.
4. `final = original.replace(before, after, 1)` — pass to existing `build_diff` +
   `apply_diff`.
5. `--mode before-after` without `--before-file` or `--target-path` → fail closed immediately.

### Handoff prompt

Task T3 of large-file-delegation-before-after.
Governing docs: docs/tasks/large-file-delegation-before-after.md,
docs/evaluations/large-file-delegation-2026-06-21.md §Replacement algorithm.
File + line range: scripts/delegate-low-rri.py — new apply_before_after(), parse_args() additions, main() dispatch.
Requires T1 (done_reason guard) and T2 (parser) to be complete first.
Acceptance criteria (see §T3 edge cases below).
Stop: commit T3 only. Do not start T4.

### Happy paths considered

- **HP-T3-1:** BEFORE block found exactly once in target file; AFTER returned by model; `final` constructed; diff touches only the changed lines; `git apply --check` passes.
- **HP-T3-2:** `workspace.rs` dead-code deletion: caller provides BEFORE (3-line block) and derives AFTER (empty); diff is exactly 3 lines removed; not `−1188/+2`.
- **HP-T3-3:** Small file continues to use `--mode full-file`; existing path unchanged; no regression.

### Edge cases considered

- **EC-T3-1:** BEFORE block not found in target file → error before diff construction.
- **EC-T3-2:** BEFORE block found more than once → error before diff construction.
- **EC-T3-3:** `--mode before-after` invoked without `--before-file` → argument error, exit 1.
- **EC-T3-4:** `done_reason == "length"` (T1 guard) fires before algorithm runs → error propagated; no diff built.
- **EC-T3-5:** Deletion mode AFTER contains lines not present in BEFORE → error before diff.

---

## T4 — Documentation updates and workspace.rs regression

- **Status:** [x] Done (unit tests excluded per user instruction; regression verified inline)
- **Effort:** M
- **Scope:** `scripts/delegate_low_rri_test.py`; docs listed below

### Goal

Add unit tests covering all T1–T3 HP/EC cases. Add workspace.rs end-to-end
regression test. Update all governing docs so the active contract documents
`before-after` mode for large files.

Docs to update:
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md` — mode-selection rule
- `docs/policies/RRI_POLICY.md` — before-after mode reference
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — delegation packet update
- `docs/policies/HITL_AUTONOMY_POLICY.md` — large-file delegation autonomy note
- `docs/gemma-local-improve.md` — active contract summary

### Handoff prompt

Task T4 of large-file-delegation-before-after.
Governing docs: docs/tasks/large-file-delegation-before-after.md.
Files: scripts/delegate_low_rri_test.py; docs listed in §T4 goal.
Acceptance criteria:
- all HP/EC cases from T1–T3 have at least one named unit test;
- workspace.rs regression test asserts diff is exactly 3 lines (not −1188/+2);
- python3 -m unittest scripts/delegate_low_rri_test.py passes;
- make qa-docs passes after doc updates.
Stop: mark T4 done. Close task ledger.

### Happy paths considered

- **HP-T4-1:** All T1–T3 happy path cases have a named, passing unit test.
- **HP-T4-2:** `make qa-docs` passes after doc updates; no dangling references.

### Edge cases considered

- **EC-T4-1:** workspace.rs regression test fails if diff is not exactly 3 lines → test blocks CI; no suppression allowed.
- **EC-T4-2:** A doc update introduces a dangling ADR reference → `make qa-docs` catches it.
