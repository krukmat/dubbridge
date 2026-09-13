---
type: TaskLedger
status: Active
plan: docs/plan/software-factory-execution-seam-v3-2.md
---

# Software-Factory Execution Seam v3.2 — Tasks

## Parent outcome

Implement the minimal v3.2 normalization seam without creating a new policy, retry, fallback, reviewer, telemetry, or product-runtime authority.

Owner direction to execute this plan was given for a new branch based on `feature/p2p-mvp-core`.

## T1 — Resolved execution contract and logical bindings

**Effort:** L  
**Depends on:** none  
**Status:** Implemented; targeted local verification passed; repository CI/reviewer verification pending.

### Implemented scope

- added `scripts/local-agent/execution_contract.py`;
- added normalized `ResolvedExecutionConstraints`;
- separated `LogicalBinding` from `RuntimePreset`;
- prohibited priority/eligibility in the binding schema;
- represented cloud-only execution as `cloud_handoff_required` with no local runtime preset;
- preserved current routing/model-selection semantics by deriving inputs from the existing runner rather than recomputing policy.

### Acceptance evidence

- local execution carries the resolved runtime preset;
- cloud-only execution has no local preset;
- policy family/version is explicit (`rri` / `rri-v2` for current executions);
- pure contract module compiled successfully in independent local verification;
- targeted contract cases passed in independent local verification.

### Remaining verification

- repository-native test execution / CI;
- repository reviewer gate when available.

## T2 — Runtime usage and execution-counter evidence

**Effort:** L  
**Depends on:** T1  
**Status:** Implemented; targeted local verification passed; live Ollama/repository CI verification pending.

### Implemented scope

- added `scripts/local-agent/execution_evidence.py`;
- added additive `scripts/local-agent/run_normalized_task.py` facade;
- facade delegates execution to `run_local_task` unchanged;
- facade observes `gemma_local.stream_chat` in-process and preserves the exact returned result/error behavior;
- measured prompt/output tokens and `done_reason` are propagated when the runtime provides them;
- missing usage remains `null` and is not estimated;
- explicit counters distinguish model invocations, repair attempts, test attempts and materialized fallback handoffs;
- total elapsed time and verification status are referenced from the existing authoritative audit record when available;
- existing audit writes still execute unchanged.

### Acceptance evidence

- pure evidence module compiled and targeted counter/usage cases passed in independent local verification;
- observer wrapper compiled and targeted transparency/error-propagation cases passed in independent local verification;
- no modifications were made to `session_loop.py` or existing audit/signature behavior.

### Remaining verification

- live Ollama invocation through the facade on the target host;
- repository-native test execution / CI;
- repository reviewer gate when available.

## T3 — Cloud handoff and evidence-lineage normalization

**Effort:** M  
**Depends on:** T1, T2  
**Status:** Implemented; targeted local verification passed; repository integration verification pending.

### Implemented scope

- existing `fallback_selection` remains the sole fallback recommendation/authorization authority;
- normalized evidence references existing fallback artifact path and authorization receipt SHA only;
- authority-bearing fallback payload is not duplicated into the normalized summary;
- initial cloud-only local rejection is represented as `cloud_handoff_required` but is not counted as a materialized fallback handoff until a selection/checkpoint exists;
- no cloud model is invoked by the new seam;
- output schema is `execution-summary-v1` and is explicitly non-authoritative.

### Acceptance evidence

- targeted evidence tests cover authorized fallback reference behavior;
- targeted evidence tests distinguish required versus materialized handoff;
- direct cloud execution remains out of scope.

### Remaining verification

- repository-native Moderate terminal fallback integration run;
- repository CI/reviewer gates when available.

## Actual implementation topology

```text
existing TaskCard / EffectiveLimits / RRI decision
                    |
                    v
          run_normalized_task.py
             /             \
            v               v
 execution_contract     run_local_task (unchanged)
                            |
                       existing controls
                            |
                   fallback/audit artifacts
             \             /
              v           v
              execution_evidence
                    |
                    v
          execution-summary-v1
```

The additive facade was chosen over direct edits to current runner/control modules because it delivers the contract/evidence business value with lower blast radius and leaves current authorities intact.

## Closure checks

- [ ] final branch diff contains no changes under `crates/p2p`, `apps/availability-node`, `mobile/src/p2p`, or P2P persistence paths;
- [x] no new network service or background process;
- [x] new pure modules compile in independent local verification;
- [x] targeted pure contract/evidence/observer cases pass in independent local verification;
- [ ] repository-native CI/checks reviewed;
- [ ] repository reviewer gate reviewed if available;
- [ ] ADR index synchronized;
- [ ] final changed-file inventory recorded.

No reviewer PASS is claimed in this ledger until the repository's reviewer path actually runs.
