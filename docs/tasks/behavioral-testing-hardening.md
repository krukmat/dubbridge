---
type: TaskList
title: "Tasks: Behavioral Testing Hardening"
status: active
plan: docs/plan/behavioral-testing-hardening.md
Behavioral coverage contract: behavior-v2
---

# Tasks: Behavioral Testing Hardening

**Plan:** `docs/plan/behavioral-testing-hardening.md`

Owner approval: explicit in-session approval on 2026-09-04 to execute the reviewed TDD/ATDD/BDD hardening plan directly on `main`.

## BTH-T1 — Cross-stack behavioral evidence contract

- **Status:** [x] Done — 2026-09-04
- **Type:** development
- **Effort:** M
- **RRI:** 30 → Moderate
- **Depends on:** —

### Objective

Introduce `behavior-v2` as a backward-compatible behavioral evidence contract and wire it into deterministic documentation QA.

### Happy paths considered

- HP-1: a completed development task under `behavior-v2` with HP/EC cases and valid Rust/Python/Jest/Maestro evidence passes the gate.
- HP-2: a legacy `unit-v1` ledger keeps its existing Rust/Python validation behavior unchanged.

### Edge cases considered

- EC-1: missing evidence files, missing named Rust/Python tests, or malformed evidence references fail closed.
- EC-2: a completed development task under `behavior-v2` without both HP and EC cases fails closed.

### Acceptance criteria

- `behavior-v2` is recognized separately from legacy `unit-v1`.
- Evidence can reference `.rs`, `.py`, `.ts`, `.tsx`, `.js`, `.jsx`, `.yaml`, `.yml`, and executable shell runner artifacts.
- Rust/Python named-test references retain exact function validation.
- JS/TS evidence validates the referenced file and, when a selector is provided, that selector text exists in the test file.
- YAML/shell evidence validates file existence and may be used for E2E/runner evidence without pretending to be a unit test.
- Existing `unit-v1` semantics continue to pass their existing test suite.
- `make qa-docs` invokes the new behavior gate.

### Evidence to emit

- deterministic script tests covering the happy/edge paths above;
- exact `make qa-docs` composition in the Makefile.

### Status artifacts affected

- this ledger;
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`;
- `DEVELOPMENT_REFERENCE.md`.

### Reflection log

Required passes: 2 (`30` → `Moderate`).

#### Pass 1

- **Draft verdict:** a separate `behavior-v2` validator avoids changing the legacy `unit-v1` parser and supports Rust, Python, Jest/TypeScript, Maestro YAML and shell evidence.
- **Critique findings:** applying the new semantics inside `check-task-unit-coverage.sh` would risk silently changing grandfathered ledgers; evidence syntax also needed fail-closed file/type validation.
- **Revisions applied:** implemented a separate validator, retained exact named-test checks for Rust/Python, and limited the contract to ledgers explicitly declaring `behavior-v2`.

#### Pass 2

- **Draft verdict:** deterministic fixture tests passed and the repository `qa-docs` GitHub Actions job passed with the new wrapper in the execution path.
- **Critique findings:** in-progress tasks must not require closure certification and legacy `unit-v1` must continue through the pre-existing gate unchanged.
- **Revisions applied:** the validator gates only completed development sections; the old script/test suite remains byte-for-byte available through the existing `qa-docs` path.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | valid cross-stack behavior-v2 evidence passes | unit | `scripts/check_behavioral_coverage_test.py::test_valid_cross_stack_evidence_passes` | passed |
| HP-2 | Happy path | legacy unit-v1 validation remains operational | unit | `scripts/check_task_unit_coverage_test.py::test_valid_review_artifact_passes` | passed |
| EC-1 | Edge case | missing file or named test fails closed | unit | `scripts/check_behavioral_coverage_test.py::test_missing_evidence_file_fails_closed`; `scripts/check_behavioral_coverage_test.py::test_missing_python_function_fails_closed` | passed |
| EC-2 | Edge case | completed task without an edge case fails closed | unit | `scripts/check_behavioral_coverage_test.py::test_missing_edge_case_fails_closed` | passed |

### Verification

- `python3 scripts/check_behavioral_coverage_test.py` → 5/5 passed before integration; the suite is expanded by the subsequent closure-hardening change to cover layer, Reflection and owner-verification failures as well.
- GitHub Actions run `33913058079`, job `qa-docs` → passed with the new deterministic gate in the `qa-docs` execution path.
- Existing `fmt`, `clippy`, `roadmap-drift`, and `maintainability` jobs also passed on commit `5f4c27cb` while this task was being closed.

### Owner final verification

- Owner: primary agent
- Date: 2026-09-04
- Statement: verified every BTH-T1 happy path and edge case has deterministic test evidence and that the integrated GitHub `qa-docs` job passes on `main`.
- Commands run: `python3 scripts/check_behavioral_coverage_test.py`; GitHub Actions run `33913058079` / `qa-docs`.

## BTH-T2 — BDD traceability gate and S-120 normalization

- **Status:** [~] In progress
- **Type:** development
- **Effort:** M
- **RRI:** 30 → Moderate
- **Depends on:** BTH-T1

### Objective

Add `qa-bdd-map` to detect BDD drift and repair the known S-120 inconsistencies without mass-rewriting historical behavior specs.

### Happy paths considered

- HP-1: every canonical `.feature` is represented by the BDD index and all mapped evidence paths exist.
- HP-2: a scenario may map to multiple tasks/evidence references without being rejected.

### Edge cases considered

- EC-1: a feature missing from the canonical inventory, an unknown scenario ID, or a missing evidence path fails closed.
- EC-2: a mapping that cites its own `.feature` as `Executable Evidence` fails because specification is not execution evidence.

### Acceptance criteria

- `make qa-bdd-map` validates canonical feature inventory against `docs/bdd/*.feature`.
- Scenario IDs in mapping rows resolve to scenario IDs present in their feature file.
- `Executable Evidence` paths exist and do not point back to the `.feature` specification itself.
- Mapping supports many-to-many task/evidence references.
- S-120 scenarios receive stable inline IDs, concrete executable evidence, and architecture-neutral failure observability wording.
- `DEVELOPMENT_REFERENCE.md` and `docs/bdd/README.md` expose the same canonical feature inventory.
- `qa-bdd-map` is part of `qa-docs`.

### Evidence to emit

- deterministic validator tests for valid, missing, many-to-many, and self-referential evidence mappings.

### Status artifacts affected

- `docs/bdd/README.md`;
- `docs/bdd/s-120-media-preparation.feature`;
- `DEVELOPMENT_REFERENCE.md`;
- this ledger.

### Progress record

- Added machine-readable `docs/bdd/behavior-map-v2.json` covering every current `.feature` and placing S-120 in strict mode while grandfathering the remaining historical specs.
- Added `scripts/check-bdd-map.py` plus deterministic tests and integrated it into the `qa-docs` execution path.
- Normalized S-120 with inline scenario IDs and executable many-to-many mappings; removed the stale management-console wording.
- Synchronized the human inventories in `docs/bdd/README.md` and `DEVELOPMENT_REFERENCE.md` and extended the deterministic gate to reject future drift across either surface.
- Remaining before closure: final `qa-docs` evidence on the inventory-parity version and the authoritative workflow wording alignment handled by BTH-T3.

## BTH-T3 — Workflow semantics and coverage clarification

- **Status:** [ ] Not started
- **Type:** docs / policy alignment
- **Effort:** S
- **RRI:** 18 → Low
- **Depends on:** BTH-T1, BTH-T2

### Acceptance criteria

- The workflow explicitly distinguishes BDD behavior, ATDD acceptance/evidence, and TDD implementation technique.
- New development ledgers default to `behavior-v2`; legacy `unit-v1` remains grandfathered.
- Reproducible defects use regression-test-first; critical deterministic logic strongly prefers test-first.
- Documentation states accurately that the 90% coverage threshold is the Rust workspace gate with configured exclusions, while mobile/Jest is independently gated by typecheck/lint/tests.
- No new BDD framework is introduced.

### Status artifacts affected

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`;
- `DEVELOPMENT_REFERENCE.md`;
- this ledger.
