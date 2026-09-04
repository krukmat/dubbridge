---
type: Plan
title: "Behavioral Testing Hardening"
status: active
description: "Consolidate DubBridge TDD/ATDD/BDD practice by aligning workflow documentation, cross-stack behavioral evidence, BDD traceability, and CI enforcement without adding a new BDD framework."
---

# Behavioral Testing Hardening

## Objective

Harden the testing workflow that DubBridge already uses. The change does not introduce a new methodology or BDD runner. It aligns the documented process with deterministic repository gates so that acceptance behavior, executable evidence, and BDD mappings stay consistent across Rust, Python, React Native/Jest, and Maestro.

## Current-state findings

- TDD is documented as `test first where practical` and is already used for regression-first bug fixes, but enforcement is intentionally pragmatic.
- ATDD-like task discipline already exists through acceptance criteria plus stable `HP-#` / `EC-#` examples and closure evidence.
- `Behavioral coverage contract: unit-v1` is opt-in and its deterministic parser is limited to Rust/Python unit-test references.
- BDD specs are canonical under `docs/bdd/`, but inventory and evidence mappings can drift because `qa-docs` has no BDD-specific consistency gate.
- `S-120` is the clearest historical drift example: scenario IDs are external to the feature text, evidence points back to the feature itself, and one scenario references a management console despite ADR-029 making mobile the sole first-party authenticated UI.

## Design decisions

1. Preserve `unit-v1` as a legacy contract; do not mass-migrate historical ledgers.
2. Add `behavior-v2` as the default contract for new development ledgers.
3. `behavior-v2` accepts executable evidence at the cheapest appropriate layer: unit, component, integration, contract, or E2E.
4. Evidence validation is cross-stack: Rust/Python named tests, Jest/TypeScript test files/selectors, and Maestro YAML/runner artifacts.
5. Add a deterministic `qa-bdd-map` gate rather than Cucumber/Behave or another BDD execution framework.
6. BDD traceability is many-to-many: one scenario may map to multiple tasks/evidence items and one task may support multiple scenarios.
7. Keep TDD wording pragmatic, but explicitly require regression-test-first for reproducible defects and strongly prefer test-first for critical deterministic logic.
8. Clarify that the existing 90% gate is Rust-workspace coverage with exclusions; mobile/Jest is a separate correctness gate unless a dedicated threshold is introduced later.

## Expected files

- `scripts/check-behavioral-coverage.py` and tests.
- `scripts/check-bdd-map.py` and tests.
- `Makefile` (`qa-behavioral-coverage`, `qa-bdd-map`, `qa-docs` composition).
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.
- `docs/bdd/README.md`.
- `docs/bdd/s-120-media-preparation.feature`.
- `DEVELOPMENT_REFERENCE.md`.
- this plan and `docs/tasks/behavioral-testing-hardening.md`.

## Implementation order

1. Add cross-stack `behavior-v2` validation and deterministic tests while retaining `unit-v1`.
2. Add BDD inventory/mapping consistency validation and deterministic tests.
3. Wire both into `qa-docs`.
4. Normalize S-120 scenario IDs/evidence and remove stale UI wording.
5. Synchronize workflow/reference documentation and close the ledger.

## Non-goals

- No product/runtime behavior changes.
- No Cucumber, Behave, SpecFlow, or equivalent framework.
- No blanket historical migration to `behavior-v2`.
- No universal requirement that every task has a `.feature` file.
- No claim that line coverage alone proves behavioral coverage.
