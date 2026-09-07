---
type: Playbook
title: "Behavioral Testing Contract"
status: active
---

# Behavioral Testing Contract

This focused playbook summarizes the TDD, ATDD-style acceptance-evidence, and
BDD traceability clauses defined authoritatively in
`AGENT_WORKFLOW_GUIDE.md`. The workflow guide remains the highest authority;
if this focused reference ever diverges from it, the guide wins. The
deterministic repository gates implement the `behavior-v2` rules summarized
below.

## One model, three responsibilities

- **BDD** defines stable, externally observable product/domain behavior. Not every task needs a `.feature`; use BDD for behavior that should survive implementation refactors or crosses layers.
- **Acceptance / ATDD discipline** defines when a development task is done: acceptance criteria plus stable `HP-#` / `EC-#` examples and executable evidence.
- **TDD** is an implementation technique. Use test-first where practical, require a failing regression test before the fix for reproducible defects, and strongly prefer test-first for deterministic critical logic such as authorization, fail-closed gates, parsers, state transitions, routing and invariants.

## `behavior-v2`

New development ledgers should declare:

```yaml
Behavioral coverage contract: behavior-v2
```

Completed development tasks under that marker must contain `Happy paths considered`, `Edge cases considered`, and a `Behavioral coverage certification` table:

```md
| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | ... | integration | `apps/api/tests/example.rs::test_name` | passed |
| EC-1 | Edge case | ... | e2e | `mobile/maestro/example.yaml` | passed |
```

Use the cheapest layer that actually proves the behavior: unit, component, integration, contract, or E2E. Evidence may be Rust, Python, Jest/TypeScript, Maestro YAML, or an executable runner artifact.

`unit-v1` remains a legacy opt-in contract and is not mass-migrated.

## BDD mapping

`docs/bdd/behavior-map-v2.json` is the machine-readable canonical inventory. Every `.feature` file must be declared there. Legacy specs may remain `mode: legacy`; newly authored specs default to `mode: strict`.

For strict specs:
- every `Scenario:` begins with a stable scenario ID;
- every scenario has at least one mapped task and one executable evidence path;
- one scenario may map to multiple tasks/tests;
- `.feature` files are specifications and cannot be cited as execution evidence.

`make qa-bdd-map` validates the manifest and strict mappings.

## Coverage semantics

The repository's 90% `cargo llvm-cov` threshold is a Rust-workspace line-coverage gate with configured filename exclusions. It is not a claim of universal cross-stack behavioral coverage. Mobile has a separate typecheck/lint/Jest gate. `behavior-v2` supplies the cross-stack completion contract by asking whether each approved HP/EC behavior has executable evidence.
