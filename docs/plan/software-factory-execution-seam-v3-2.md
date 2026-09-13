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
9. Adopt additively first: observe/delegate to the existing runner instead of modifying its behavioral state machine in Wave 1.

## Implemented software-factory scope

Wave 1 is implemented as an additive facade so existing behavior remains byte/semantically authoritative:

- `scripts/local-agent/execution_contract.py` — normalized resolved-execution types;
- `scripts/local-agent/execution_evidence.py` — non-authoritative correlation/economics summary;
- `scripts/local-agent/run_normalized_task.py` — additive facade that delegates execution to `run_local_task` unchanged, observes runtime usage/audit outputs, and emits the normalized summary;
- `scripts/local-agent/execution_contract_test.py` — contract invariants;
- `scripts/local-agent/execution_evidence_test.py` — evidence/counter/handoff invariants;
- `scripts/local-agent/run_normalized_task_test.py` — observer transparency and error propagation.

Wave 1 deliberately does **not** modify:

- `run_local_task.py` routing behavior;
- `cli.py` state/control flow;
- `session_loop.py` repair/turn/test machine;
- `audit_record.py` authority/signature behavior;
- `fallback_selection.py` recommendation/authorization behavior.

A later refactor may move normalization closer to those modules only after the additive path demonstrates value and equivalent behavior.

## Documentation/status scope

- this plan;
- `docs/tasks/software-factory-execution-seam-v3-2.md`;
- `docs/adr/ADR-047-software-factory-execution-normalization-seam.md`;
- ADR index.

## Module dependency direction

```text
RRI / workflow / TaskCard / EffectiveLimits
                 |
                 v
        run_normalized_task.py
          /              \
         v                v
execution_contract   existing run_local_task
                           |
                           v
                    session_loop / fallback
                           |
                           v
                     existing audit
          \              /
           v            v
             execution_evidence
                    |
                    v
       execution-summary-v1 (non-authoritative)
```

The facade observes and references existing evidence; it does not replace it.

`execution_contract.py` and `execution_evidence.py` do not import RRI calculators, context providers, P2P code, reviewer policy, or fallback policy.

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
- no mandatory direct cloud adapter;
- no reusable cross-project package extraction in this wave.

## Delivery sequence

1. T1 — normalized resolved execution contract + binding/preset separation.
2. T2 — runtime usage observation + explicit evidence counters.
3. T3 — normalized cloud-handoff/evidence lineage integration and regression closure.

## Verification strategy

- pure contract/evidence modules must compile and pass targeted unit tests;
- the stream observer must return the exact underlying runtime result and re-raise underlying errors unchanged;
- final branch diff must contain no P2P/product-runtime modifications;
- existing runtime/reviewer behavior is not claimed recertified unless repository CI/reviewer gates actually run.
