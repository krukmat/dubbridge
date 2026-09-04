---
type: Plan
title: "Behavioral Testing Hardening"
status: complete
description: "Consolidate DubBridge TDD/ATDD/BDD practice by aligning workflow documentation, cross-stack behavioral evidence, BDD traceability, and CI enforcement without adding a new BDD framework."
---

# Behavioral Testing Hardening

## Objective

Harden the testing workflow that DubBridge already uses. The change does not introduce a new methodology or BDD runner. It aligns the documented process with deterministic repository gates so that acceptance behavior, executable evidence, and BDD mappings stay consistent across Rust, Python, React Native/Jest, and Maestro.

## Current-state findings

- TDD is documented as `test first where practical` and is already used for regression-first bug fixes, but enforcement is intentionally pragmatic.
- ATDD-like task discipline already exists through acceptance criteria plus stable `HP-#` / `EC-#` examples and closure evidence.
- `Behavioral coverage contract: unit-v1` is opt-in and its deterministic parser is limited to Rust/Python unit-test references.
- BDD specs are canonical under `docs/bdd/`, but inventory and evidence mappings can drift because `qa-docs` had no BDD-specific consistency gate before this hardening.
- `S-120` was the clearest historical drift example: scenario IDs were external to the feature text, evidence pointed back to the feature itself, and one scenario referenced a management console despite ADR-029 making mobile the sole first-party authenticated UI.

## Design decisions

1. Preserve `unit-v1` as a legacy contract; do not mass-migrate historical ledgers.
2. Add `behavior-v2` as the default contract for new development ledgers.
3. `behavior-v2` accepts executable evidence at the cheapest appropriate layer: unit, component, integration, contract, or E2E.
4. Evidence validation is cross-stack: Rust/Python named tests, Jest/TypeScript test files/selectors, and Maestro YAML/runner artifacts.
5. Add a deterministic `qa-bdd-map` gate rather than Cucumber/Behave or another BDD execution framework.
6. BDD traceability is many-to-many: one scenario may map to multiple tasks/evidence items and one task may support multiple scenarios.
7. Keep TDD wording pragmatic, but explicitly require regression-test-first for reproducible defects and strongly prefer test-first for critical deterministic logic.
8. Clarify that the existing 90% gate is Rust-workspace coverage with exclusions; mobile/Jest is a separate correctness gate unless a dedicated threshold is introduced later.

## Delivered files and surfaces

- `scripts/check-behavioral-coverage.py` and deterministic tests.
- `scripts/check-bdd-map.py` and deterministic tests.
- `Makefile` integration for `qa-behavioral-coverage`, `qa-bdd-map`, and `qa-docs`.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` as the authoritative `behavior-v2` workflow contract.
- `docs/playbooks/BEHAVIORAL_TESTING_CONTRACT.md` as the focused subordinate reference.
- `docs/bdd/behavior-map-v2.json` as the machine-readable BDD inventory/mapping contract.
- `docs/bdd/README.md` and `docs/bdd/s-120-media-preparation.feature` normalization.
- `DEVELOPMENT_REFERENCE.md` BDD inventory and QA semantics alignment.
- `docs/tasks/behavioral-testing-hardening.md` closure ledger.

## Implementation order — completed

1. [x] Add cross-stack `behavior-v2` validation and deterministic tests while retaining `unit-v1`.
2. [x] Add BDD inventory/mapping consistency validation and deterministic tests.
3. [x] Wire both into `qa-docs`.
4. [x] Normalize S-120 scenario IDs/evidence and remove stale UI wording.
5. [x] Synchronize workflow/reference documentation and close the ledger.

## Verification

- GitHub Actions run `33917808846`: `qa-docs` passed after the authoritative workflow guide was reconciled with `behavior-v2`; supporting CI jobs including mobile, clippy, fmt, cargo-check, S3 integration, deny, roadmap drift and maintainability also passed or were green during verification.
- GitHub Actions run `33918043744`: `qa-docs` passed after adding the explicit missing-evidence-path BDD fixture; the same run had the principal deterministic and cross-stack gates green before plan closure.
- The completed task ledger contains case-level certification and owner verification for BTH-T1 and BTH-T2.

## Non-goals preserved

- No product/runtime behavior changes.
- No Cucumber, Behave, SpecFlow, or equivalent framework.
- No blanket historical migration to `behavior-v2`.
- No universal requirement that every task has a `.feature` file.
- No claim that line coverage alone proves behavioral coverage.

## Outcome

The repository now uses a converged testing contract:

`BDD behavior → ATDD-style acceptance criteria + HP/EC → implementation/TDD → tier-appropriate executable evidence → qa-docs/CI traceability`.

`unit-v1` remains valid for grandfathered ledgers; new development ledgers default to `behavior-v2`.
