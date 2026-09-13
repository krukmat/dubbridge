---
type: Plan
status: Active
target_branch: feature/software-factory-execution-seam-v3-2
baseline: feature/p2p-mvp-core
---

# Software-Factory Execution Seam v3.2

## Objective

Implement the v3.2 architecture as a minimal in-process normalization seam around the existing DubBridge software-factory path.

The implementation centralizes contracts and evidence only. Existing behavior authorities remain unchanged:

- RRI/workflow owns scoring and route eligibility;
- CKG/context providers own authorized context;
- `session_loop` owns local turn/repair/test control;
- `fallback_selection` owns fallback recommendation/authorization;
- reviewer semantics remain external;
- P2P/product runtime remains untouched.

## Design decisions

1. Normalize already-resolved execution state rather than adding a policy-projection engine.
2. Separate portable Logical Binding identity from runtime/vendor tuning.
3. Do not put priority or eligibility in bindings/presets.
4. Direct cloud execution is optional; signed/authorized handoff is a first-class result.
5. Preserve runtime usage already exposed by Ollama and propagate it into execution evidence.
6. Distinguish execution session, model invocation, repair attempt, test attempt, and fallback handoff.
7. Persist policy family/version so RRI v1/v2 analytics cannot be mixed silently.
8. Reference existing authority-bearing receipts/artifacts instead of duplicating their payloads.

## Affected software-factory files

Expected implementation scope:

- `scripts/local-agent/execution_contract.py` — new normalized contract types only;
- `scripts/local-agent/run_local_task.py` — expose logical binding through existing resolved limits and preserve public compatibility;
- `scripts/local-agent/cli.py` — build normalized constraints, preserve runtime usage, normalize cloud handoff result references;
- `scripts/local-agent/audit_record.py` — evidence lineage, explicit counters, policy version and usage aggregation;
- targeted tests under `scripts/local-agent/*_test.py`.

Documentation/status scope:

- this plan;
- `docs/tasks/software-factory-execution-seam-v3-2.md`;
- `docs/adr/ADR-047-software-factory-execution-normalization-seam.md`;
- ADR index.

## Module dependency direction

```text
RRI / workflow / TaskCard / EffectiveLimits
                 |
                 v
      execution_contract.py
                 |
                 v
        cli.py / local runner
                 |
        +--------+---------+
        |                  |
        v                  v
  local execution     cloud handoff
        |                  |
        +--------+---------+
                 v
          audit_record.py
                 |
                 v
      existing audit/review evidence
```

`execution_contract.py` does not import RRI calculators, context providers, P2P code, reviewer policy, or fallback policy.

## Non-goals

- no new network service;
- no model marketplace;
- no semantic router;
- no learned optimizer;
- no generic retry controller;
- no replacement for `session_loop`;
- no replacement for `fallback_selection`;
- no new telemetry platform;
- no product-runtime/P2P changes;
- no reusable cross-project package extraction in this wave.

## Delivery sequence

1. T1 — normalized resolved execution contract + binding/preset separation.
2. T2 — runtime usage propagation + explicit evidence counters.
3. T3 — normalized cloud-handoff/evidence lineage integration and regression closure.

Each task must preserve current behavior before the next task proceeds.
