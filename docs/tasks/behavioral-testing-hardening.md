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

- **Status:** [~] In progress
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

## BTH-T2 — BDD traceability gate and S-120 normalization

- **Status:** [ ] Not started
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
- Mapping supports semicolon-separated many-to-many evidence references.
- S-120 scenarios receive stable inline IDs, concrete executable evidence, and architecture-neutral failure observability wording.
- `DEVELOPMENT_REFERENCE.md` and `docs/bdd/README.md` expose the same canonical feature inventory.
- `qa-bdd-map` is part of `qa-docs`.

### Evidence to emit

- deterministic validator tests for valid, missing, and self-referential evidence mappings.

### Status artifacts affected

- `docs/bdd/README.md`;
- `docs/bdd/s-120-media-preparation.feature`;
- `DEVELOPMENT_REFERENCE.md`;
- this ledger.

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
