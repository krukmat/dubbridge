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

### Scope

- add normalized `ResolvedExecutionConstraints` representation;
- expose current logical binding through existing resolved limits;
- separate logical binding from runtime preset values;
- preserve current model-selection/rejection semantics;
- do not add priority/eligibility to the binding/preset schema.

### Acceptance criteria

- Low, Moderate, authorized Med-high and cloud-only paths keep current behavior;
- the normalized contract carries policy family/version, band/RRI, binding, runtime preset, local-execution permission and current limits;
- cloud-only resolution does not fabricate a local runtime preset;
- existing imports/tests that use `run_local_task` public constants/functions remain compatible.

**HP-1:** Moderate task + no explicit model -> existing Devstral route -> normalized contract reports the same local binding/runtime model.

**EC-1:** RRI 46–55 whole-task route -> local execution remains rejected -> normalized contract reports `cloud_handoff_required` and no executable local runtime preset.

**Evidence to emit:** targeted unit/regression test results; branch diff.

**Status artifacts affected:** this task ledger; ADR-047 if contract boundary changes.

**Agent handoff:** Refactor only existing resolved execution outputs into a typed normalization contract. Do not move policy into the new module.

## T2 — Runtime usage and execution-counter evidence

**Effort:** L

**Depends on:** T1

### Scope

- propagate Ollama usage already returned by `gemma_local.stream_chat`;
- preserve raw model transcript behavior;
- add explicit normalized counters for model invocations, repair attempts and test attempts;
- keep legacy audit fields for compatibility;
- record policy family/version in execution evidence.

### Acceptance criteria

- successful live model calls preserve prompt/output token counts and `done_reason` when available;
- usage absence is represented as unknown/null rather than fabricated;
- one generic `attempts` field is not used as the new canonical metric;
- existing audit validation/signature behavior remains unchanged.

**HP-1:** Ollama returns `prompt_eval_count` + `eval_count` -> execution evidence carries those measured counts.

**EC-1:** injected/test chat function has no runtime usage metadata -> execution succeeds and evidence keeps usage nullable without estimation.

**Evidence to emit:** usage-propagation tests; audit-record regression tests.

**Status artifacts affected:** this task ledger.

**Agent handoff:** Propagate existing runtime metadata; do not add a telemetry collector or alter the session-loop state machine.

## T3 — Cloud handoff and evidence-lineage normalization

**Effort:** M

**Depends on:** T1, T2

### Scope

- normalize fallback/cloud-handoff references after existing `fallback_selection` runs;
- preserve existing authorization receipt/checkpoint as source of truth;
- expose fallback handoff count/reference in final audit evidence;
- keep direct cloud execution optional and out of scope.

### Acceptance criteria

- an authorized/awaiting fallback checkpoint remains byte/semantic authority;
- normalized evidence references the existing selection artifact/receipt rather than copying authority-bearing semantics;
- initial cloud-only local rejection and terminal Moderate fallback remain distinguishable;
- no cloud model is invoked by the new seam.

**HP-1:** terminal Moderate local failure creates the existing fallback checkpoint -> normalized result references its artifact and receipt status.

**EC-1:** fallback is awaiting human selection -> normalized result remains `awaiting_fallback_selection`; no automatic cloud execution occurs.

**Evidence to emit:** fallback integration regression tests; final changed-file inventory.

**Status artifacts affected:** this task ledger; plan/ADR status if implementation differs from design.

**Agent handoff:** Normalize handoff evidence around `fallback_selection`; never replace or bypass it.

## Closure checks

- no changes under `crates/p2p`, `apps/availability-node`, `mobile/src/p2p`, or P2P persistence paths;
- no new network service or background process;
- targeted local-agent tests pass;
- broader available CI/checks show no regressions;
- architecture/status docs remain synchronized.
