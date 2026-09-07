---
type: TaskList
title: "Tasks: Behavioral Testing Hardening"
status: complete
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

- `python3 scripts/check_behavioral_coverage_test.py` → deterministic fixture suite passed before integration; the suite was subsequently expanded to cover layer, Reflection and owner-verification failures.
- GitHub Actions run `33913058079`, job `qa-docs` → passed with the new deterministic gate in the `qa-docs` execution path.
- Existing `fmt`, `clippy`, `roadmap-drift`, and `maintainability` jobs also passed on commit `5f4c27cb` while this task was being closed.

### Owner final verification

- Owner: primary agent
- Date: 2026-09-04
- Statement: verified every BTH-T1 happy path and edge case has deterministic test evidence and that the integrated GitHub `qa-docs` job passes on `main`.
- Commands run: `python3 scripts/check_behavioral_coverage_test.py`; GitHub Actions run `33913058079` / `qa-docs`.

## BTH-T2 — BDD traceability gate and S-120 normalization

- **Status:** [x] Done — 2026-09-04
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

### Reflection log

Required passes: 2 (`30` → `Moderate`).

#### Pass 1

- **Draft verdict:** a machine-readable BDD manifest plus a strict/legacy mode provides deterministic traceability without forcing historical backfill or a Gherkin execution framework.
- **Critique findings:** validating only the manifest against `.feature` files would still allow the two human-facing inventories to drift; S-120 also cited its own specification as executable evidence.
- **Revisions applied:** added parity checks for `docs/bdd/README.md` and `DEVELOPMENT_REFERENCE.md`, put S-120 in strict mode with stable inline IDs, and replaced specification references with executable test evidence.

#### Pass 2

- **Draft verdict:** valid, many-to-many, missing-inventory, unknown-scenario and self-referential mappings were covered by deterministic fixtures and `qa-docs` passed on `main`.
- **Critique findings:** the validator implemented missing-evidence-path rejection but the fixture suite did not exercise that failure mode directly.
- **Revisions applied:** added `test_missing_evidence_file_fails` and re-ran the integrated `qa-docs` job through GitHub Actions.

### Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | canonical strict mapping with existing executable evidence passes | unit | `scripts/check_bdd_map_test.py::test_valid_strict_mapping_passes`; `scripts/check_bdd_map_test.py::test_human_inventory_drift_fails` | passed |
| HP-2 | Happy path | one scenario may map to multiple tasks and evidence items | unit | `scripts/check_bdd_map_test.py::test_many_to_many_tasks_and_evidence_passes` | passed |
| EC-1 | Edge case | missing inventory, unknown scenario or missing evidence fails closed | unit | `scripts/check_bdd_map_test.py::test_missing_feature_inventory_fails`; `scripts/check_bdd_map_test.py::test_unknown_scenario_fails`; `scripts/check_bdd_map_test.py::test_missing_evidence_file_fails` | passed |
| EC-2 | Edge case | a feature specification cannot certify itself as executable evidence | unit | `scripts/check_bdd_map_test.py::test_feature_cannot_be_its_own_evidence` | passed |

### Verification

- GitHub Actions run `33918043744`, job `qa-docs` → passed after the dedicated missing-evidence-path fixture landed.
- The same run also passed `fmt`, `clippy`, `cargo-check`, mobile, S3 integration, dependency policy, roadmap drift, Python complexity, peer-workflow review and maintainability gates before this closure record was written.
- `docs/bdd/behavior-map-v2.json` remains the machine-readable canonical inventory; strict S-120 mappings are checked against actual scenario IDs and evidence files.

### Owner final verification

- Owner: primary agent
- Date: 2026-09-04
- Statement: verified every BTH-T2 happy path and edge case has deterministic executable evidence and that the integrated BDD traceability gate passes in `qa-docs` on `main`.
- Commands run: `make qa-docs` via GitHub Actions run `33918043744` / job `qa-docs`.

## BTH-T3 — Workflow semantics and coverage clarification

- **Status:** [x] Done — 2026-09-04
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
- `docs/playbooks/BEHAVIORAL_TESTING_CONTRACT.md`;
- `DEVELOPMENT_REFERENCE.md`;
- this ledger.

### Completion evidence

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` is authoritative and now defines BDD, ATDD-style acceptance evidence and TDD as separate responsibilities; it defaults new development ledgers to `behavior-v2` while preserving legacy `unit-v1`.
- The guide requires RED → fix → GREEN for reproducible defects and strongly prefers test-first for critical deterministic logic, with pragmatic exceptions for wiring/config/generated/purely visual changes.
- The guide and `DEVELOPMENT_REFERENCE.md` describe the 90% threshold as the Rust workspace `cargo llvm-cov` gate with configured exclusions and keep mobile typecheck/lint/Jest as an independent gate.
- `docs/playbooks/BEHAVIORAL_TESTING_CONTRACT.md` is a focused summary explicitly subordinate to the authoritative workflow guide; it no longer acts as a competing forward-contract exception.
- No Cucumber, Behave, SpecFlow or equivalent BDD runner was introduced.
- GitHub Actions run `33917808846`, job `qa-docs` → passed on the authoritative guide version; run `33918043744` subsequently passed `qa-docs` with the final BDD fixture set.

## Closure

Behavioral Testing Hardening is complete. DubBridge now has one converged testing model:

`BDD behavior → ATDD-style HP/EC acceptance contract → implementation/TDD → tier-appropriate executable evidence → deterministic qa-docs/CI traceability`.

Historical `unit-v1` ledgers remain valid and are not mass-migrated. New development work defaults to `behavior-v2`; `.feature` files remain specifications rather than requiring a new runtime BDD framework.
