---
type: Plan
title: "Local agent strict task scope"
status: Active
---

# Local agent strict task scope

## Objective

Make the Moderate local implementer fail closed and deterministic: preload the
task card's authorized files, let the model edit only `allowed_paths`, and keep
formatting plus acceptance execution under runner control.

## Design decisions

- Keep `allowed_paths` as the single edit capability and preload those files as
  immutable initial context. The model has no read or process capability.
- Inject `allowed_paths` and parsed `acceptance_tests` into
  `LocalAgentBoundary`; reject every forbidden model tool or unlisted path
  before filesystem or process execution.
- On `finish`, format only edited authorized Rust files through temporary copies
  and execute the parsed acceptance commands in card order.
- Preserve the final git diff scope check as defense in depth for indirect
  writes produced by accepted build/test commands.
- Preserve immediate `boundary_violation` termination in the runner loop.
- End the DEV phase after in-scope acceptance passes. Organization, review,
  coverage, and closure are downstream orchestrator responsibilities.
- Amend ADR-036's earlier unrestricted-command productivity tradeoff for real
  task execution. Historical benchmark evidence remains historical.

## Execution sequence

1. `LASS-0` ratifies and propagates the strict-scope contract.
2. `LASS-1` enforces `allowed_paths` on every file tool.
3. `LASS-2` removes model-issued commands and makes formatting/acceptance
   runner-controlled.
4. Re-run `S-150-T2b-ii-c` locally with the unchanged two-file card.

## Dependencies

`ADR-036` -> `LocalAgentBoundary` -> `RunnerFileTools` / `run_local_task.py` ->
focused unit and integration tests -> live S-150 retry.
