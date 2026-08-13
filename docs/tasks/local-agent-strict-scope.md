---
type: TaskList
title: "Local agent strict task scope"
status: Active
---

# Local agent strict task scope

Behavioral coverage contract: unit-v1

## LASS-0: Ratify strict task-scope execution

**Type:** ADR/policy/docs-only
**Effort:** M (`RRI 29`, Moderate)
**Status:** [x] Done

**Objective:** Amend the local-first contract so production task runs fail
closed on model-issued file access and commands outside the task card.

**Acceptance criteria:** ADR-036, workflow, HITL, and RRI policy consistently
state the strict tool boundary while retaining the final diff scope gate.

**Evidence to emit:** `docs/audit/lass-0-rri.md`; `make qa-docs`.

**Status artifacts affected:** This ledger, the linked plan, ADR-036, workflow,
HITL policy, and RRI policy.

**Approval:** Matias explicitly authorized the bounded runner correction and
local S-150 retry on 2026-08-13; no additional checkpoint requested.

Task-analysis review: n/a - ADR/policy/docs-only exemption.

Code-solution review: n/a - ADR/policy/docs-only exemption.

- Verification: `make qa-docs` passed after regenerating the canonical
  `AGENTS.override.md` projection.
- REVIEW-OVERRIDE: not-applicable — ADR/policy/docs-only task.
- Scope-note: No executable code changed in LASS-0.

## LASS-1: Enforce allowed paths at file-tool call time

**Type:** development
**Effort:** L (`RRI 55`, Med-high)
**Depends on:** LASS-0
**Status:** [ ] Planned

**Objective:** Preload authorized files and reject every model read while
restricting writes and patches to `allowed_paths` before mutation occurs.

**Happy paths considered:**

- **HP-1:** Writing and patching either an exact allowed file or a descendant
  of an allowed directory succeeds; initial authorized contents are preloaded.

**Edge cases considered:**

- **EC-1:** Any model-issued `read_file`, including for an allowed path, returns
  `boundary_violation` and terminates the attempt on that turn.
- **EC-2:** Absolute paths, `..` traversal, and symlink escapes remain rejected.
- **EC-3:** An out-of-scope write or patch is rejected before mutation, while
  the final diff gate remains active.

**Acceptance criteria:** `LocalAgentBoundary` receives the card's allowed paths;
the model schema exposes only editing and finish tools; focused boundary,
file-tool, runner, and integration tests pass.

**Evidence to emit:** `docs/audit/lass-1-rri.md`, phase reviews, three
Reflection passes, test output, unit coverage certification, owner verification.

**Status artifacts affected:** This ledger and the linked plan.

**Approval:** Explicitly waived for this bounded task by Matias on 2026-08-13.

## LASS-2: Make finish validation runner-controlled

**Type:** development
**Effort:** L (`RRI 52`, Med-high)
**Depends on:** LASS-1
**Status:** [ ] Planned

**Objective:** Remove model-issued commands; on `finish`, format edited
authorized Rust files and run operator-authored acceptance commands in order.

**Happy paths considered:**

- **HP-1:** `finish` formats only edited authorized Rust files and then executes
  every acceptance command in card order with the stripped environment.
- **HP-2:** A formatter or acceptance failure returns its output and refreshed
  authorized contents for a bounded repair.

**Edge cases considered:**

- **EC-1:** Any model-issued command, including an acceptance command, returns
  `boundary_violation` before process creation.
- **EC-2:** Empty, malformed, or unparsable acceptance commands grant no command
  capability.
- **EC-3:** Formatting uses isolated copies and can write back only edited
  `allowed_paths`.

**Acceptance criteria:** The prompt/schema expose no read or command tool; the
boundary retains canonical acceptance argv for runner-only execution; functional
checks prove scoped formatting, automatic acceptance, bounded repair feedback,
immediate rejection of model reads/commands, and DEV success after in-scope
acceptance without applying downstream organization gates.

**Evidence to emit:** `docs/audit/lass-2-rri.md`, phase reviews, three
Reflection passes, test output, unit coverage certification, owner verification,
and the live `S-150-T2b-ii-c` retry artifact.

**Status artifacts affected:** This ledger, the linked plan, and S-150 status
artifacts only after the live retry completes.

**Approval:** Explicitly waived for this bounded task by Matias on 2026-08-13.
