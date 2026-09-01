---
type: Audit
title: "X26-T6 Python complexity gate implementation"
status: complete
slice: tiger-style-adaptation
related_task: X26-T6
---

# X26-T6 Python complexity gate implementation

## Scope

X26-T6 introduces a Python complexity/length gate for worker code only. The gate intentionally does not scan repository-root `scripts/*.py` tooling.

## Implementation

- Added root `ruff.toml` pinned to Ruff `0.16.5` and Python 3.12 semantics.
- Ruff discovery is restricted to `workers/*-py/**/*.py`.
- Enforced rules:
  - `C901` McCabe complexity, maximum 15.
  - `PLR0912` branch count, maximum 15.
  - `PLR0915` statement count, maximum 50.
  - `E501` line length, maximum 120.
- Added fail-closed `make qa-python-complexity`; it reports the pinned Ruff install command when Ruff is unavailable.
- Added `qa-python-complexity` to the aggregate `qa-ci` target.
- Added a dedicated GitHub Actions `python-complexity` job using Python 3.12 and pinned Ruff 0.16.5, then invoking the Make target.

## Scope guard

The combination of `ruff.toml`'s `include = ["workers/*-py/**/*.py"]` and the Make target's explicit `ruff check workers` invocation prevents the gate from expanding into `scripts/*.py`.

## CI verification

GitHub Actions run `33432812287`, job `python-complexity` (`99622019314`), passed on the consolidated T6 commit. The job installed Ruff `0.16.5`, executed `make qa-python-complexity PYTHON=python`, and Ruff reported `All checks passed!` across the worker Python surface.

This confirms the T6 gate accepts the current `workers/asr-worker-py/main.py` and the currently implemented translation worker while remaining scoped away from repo-root scripts.

## Non-blocking control incident

The same CI run's `qa-docs` job failed in `scripts/check-task-unit-coverage.sh` because `docs/tasks/s-150-translation-dubbing.md` contains historical review-artifact `commit_sha` values that no longer resolve to valid commit objects. The failure lists S-150 tasks from `T1a` through `T3c`; it does not reference `ruff.toml`, the Make target, the Python worker surface, or X26-T6.

Disposition: documented only, per owner direction. This pre-existing S-150 documentation-control issue does not reopen T6.

## Verification policy

Per the owner direction for X26 tasks, implementation completion is not blocked by unrelated CI/control failures. The dedicated `python-complexity` job is the authoritative runtime verification for this task. Any remote control failure is recorded here rather than folded into unrelated implementation changes.

## Tool version provenance

Ruff 0.16.5 was the latest PyPI release verified on 2026-08-31 (released 2026-08-27) and is pinned so future releases cannot silently change the gate.
