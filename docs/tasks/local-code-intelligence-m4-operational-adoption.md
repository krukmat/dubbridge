---
type: TaskList
title: "Local Code Intelligence M4 — Operational Adoption"
status: proposed
plan: docs/plan/local-code-intelligence-m4-operational-adoption.md
behavioral_coverage_contract: unit-v1
---

# Local Code Intelligence M4 — Operational Adoption Tasks

## Execution order

```text
M4-T0
  |
  +--> M4-T1 --+
  |            |
  +--> M4-T2 --+
               |
               v
            M4-T3
               |
               v
            M4-T4
               |
        findings only
               |
               v
            M4-T5
               |
               v
            M4-T6
```

T1/T2 are logically independent but should normally be executed sequentially because both change the gateway/policy surface.

---

## M4-T0 — Reconcile operational baseline

**Status:** pending  
**Effort:** S  
**Type:** docs/status only  
**Depends on:** M3 closed

Synchronize the branch documentation with the project's declared M3-closed state and identify the actual existing operational entry point used by Analyze/agent orchestration.

### Acceptance criteria
- No document still implies that M3 itself must be reimplemented.
- The actual Analyze/agent entry point consuming the boundary is named explicitly.
- Any backend/adapter already selected by M3 is documented by interface/location, without making the core contract vendor-specific.
- If the branch does not contain the M3 implementation itself, documentation says where the authoritative integration lives rather than inventing one.

**Evidence to emit:** documentation diff and exact integration entry point.  
**Status artifacts affected:** M4 plan/ledger, existing code-intelligence README/audit if stale.

**Agent handoff:** Reconcile status only. Do not redesign or reopen M3.

---

## M4-T1 — Enforce graph freshness

**Status:** pending  
**Effort:** S/M  
**Type:** development  
**Depends on:** T0

Turn graph/repository revision binding from recorded provenance into an enforced operational invariant.

### Design direction
- Prefer an explicit expected repository revision supplied by the orchestrator/Analyze integration over hidden assumptions.
- The gateway must compare the graph result revision against the expected operational baseline before producing consumable artifacts.
- Fixture/audit usage may have an explicit test-only/manual escape path if necessary, but the normal agent path must be verified.

### Acceptance criteria
- Matching graph and expected repository revisions produce normal artifacts.
- A stale/mismatched graph fails closed before a successful capsule/receipt pair is published.
- The failure identifies expected versus received revision without leaking unrelated repository context.
- Existing deterministic hashing remains stable for equivalent verified inputs.

### Behavioral examples
- **HP-41:** Analyze supplies revision `A`, graph was built from `A` -> gateway produces receipt/capsule.
- **EC-41:** Analyze supplies revision `A`, backend returns graph revision bound to git revision `B` -> gateway rejects the result and produces no consumable success artifacts.

**Evidence to emit:** unit tests for HP-41/EC-41 plus CLI smoke with a deliberate stale fixture.  
**Status artifacts affected:** this ledger, M4 plan, code-intelligence README/audit, receipt schema only if contract fields change.

**Agent handoff:** Make freshness deterministic and testable. Do not make the graph authoritative over Git.

---

## M4-T2 — Harden minimum disclosure beyond backend labels

**Status:** pending  
**Effort:** M  
**Type:** development  
**Depends on:** T0; execute after T1 by default to minimize conflicts

Add defense-in-depth so cloud export does not rely solely on backend-provided classification and does not export unrelated metadata arrays wholesale.

### Design direction
- Preserve the backend-neutral graph contract.
- Add a small deterministic gateway-side policy layer rather than a generic policy DSL.
- Cloud metadata should be justified by allowed task-local evidence instead of copied wholesale from `anchors`, `files`, `symbols`, `tests`, `boundaries`, or governance arrays.
- Add hard-deny handling for clearly unsafe repository/runtime/secret paths or values where deterministic detection is reliable.
- Do not block legitimate cloud work merely because it touches a protected product boundary; the goal is minimum disclosure, not a blanket path ban.

### Acceptance criteria
- Cloud capsule excludes unrelated metadata that has no allowed task-local evidence.
- Explicit secret/runtime records remain denied even if another backend field attempts to reference them.
- A deliberately mislabeled obviously unsafe fixture is rejected or reduced by gateway-side defense-in-depth.
- Local target remains richer than cloud while still excluding explicit secret/runtime data.
- Policy behavior remains small, inspectable, deterministic, and unit-tested.

### Behavioral examples
- **HP-42:** task-local fragment/relationship references `crates/media/...` -> only the justified media file/symbol metadata is exported to cloud.
- **EC-42:** backend includes unrelated auth/storage topology metadata while no allowed task-local evidence requires it -> cloud capsule omits it.
- **EC-43:** a fixture deliberately labels an obviously secret/runtime path or value as `task_local` -> gateway defense-in-depth prevents export.

**Evidence to emit:** unit tests for HP-42/EC-42/EC-43 and before/after cloud capsule fixture comparison.  
**Status artifacts affected:** this ledger, M4 plan, code-intelligence README/audit, receipt/capsule schema if metadata representation changes.

**Agent handoff:** Solve P2/P3 with the smallest deterministic policy surface that protects cloud export. Do not create a generic security-classification framework.

---

## M4-T3 — Bounded context expansion

**Status:** pending  
**Effort:** M  
**Type:** development  
**Depends on:** T1, T2

Implement the explicit expansion path anticipated by the architecture and existing `expansions` receipt field.

### Design direction
- An agent requests additional context with a reason and reference to the prior receipt/capsule.
- The request is evaluated locally against the current graph revision and the same export policy as the initial capsule.
- The result is a new bounded artifact generation, not arbitrary graph traversal from the cloud side.
- Preserve a verifiable link to the prior receipt (hash/reference) and record what was requested, allowed, reduced, or denied.

### Acceptance criteria
- Valid expansion request against a current receipt can add newly allowed task-relevant context.
- Expansion cannot bypass cloud deny/minimum-disclosure policy.
- Expansion against a stale/mismatched base receipt is rejected.
- Receipt history records the reason and decision without exposing denied content.
- New artifacts are deterministic for equivalent inputs.

### Behavioral examples
- **HP-43:** cloud agent lacks one adjacent task-local symbol and requests it with a concrete reason -> local gateway returns a new capsule containing only the approved expansion and records the decision.
- **EC-44:** expansion requests global architecture/cross-boundary topology for convenience -> request is denied/reduced for cloud target.
- **EC-45:** expansion references a receipt generated from a different repository/graph revision -> fail closed.

**Evidence to emit:** expansion unit tests, CLI/API smoke, receipt hash-chain/reference evidence.  
**Status artifacts affected:** this ledger, M4 plan, receipt schema, code-intelligence README/audit.

**Agent handoff:** Implement bounded expansion as a local decision. Never expose a generic graph query surface to cloud agents.

---

## M4-T4 — Operational use-and-adjust loop

**Status:** pending  
**Effort:** ongoing/S per adjustment  
**Type:** operational  
**Depends on:** T3

Use the M4 path during ordinary DubBridge work. Do not create synthetic benchmark tasks or an A/B evaluation program.

### Acceptance criteria
- Normal tasks consume the existing Analyze/CKG path rather than a special benchmark harness.
- When initial context is sufficient, no expansion is requested merely for completeness.
- When context is insufficient, bounded expansion is used instead of unrestricted repository exploration.
- Only actionable friction is recorded: stale graph, missing relevant context, irrelevant context, policy over-blocking, policy under-blocking, host resource regression, or consumer artifact race.
- Each recurring/material issue becomes a narrowly scoped T5 hardening task or is explicitly deferred with rationale.

### Behavioral examples
- **HP-44:** ordinary DubBridge task receives sufficient bounded context and proceeds through existing routing/review without additional repository discovery.
- **HP-45:** ordinary task needs one additional adjacent symbol -> bounded expansion supplies it and work continues.
- **EC-46:** agent attempts to treat the receipt/graph as authoritative over source/tests -> workflow requires source/test verification.
- **EC-47:** a cloud consumer asks for broad architecture traversal -> request remains bounded/denied; no direct graph access is added.

**Evidence to emit:** existing receipts/capsules plus concise friction note only when an adjustment is warranted.  
**Status artifacts affected:** this ledger and any narrowly scoped T5 task created from real evidence.

**Agent handoff:** Use the mechanism on real work. Do not optimize numbers; adjust only when actual friction is reproducible.

---

## M4-T5 — Evidence-backed hardening (conditional)

**Status:** conditional  
**Effort:** variable  
**Type:** development only when triggered  
**Depends on:** a concrete T4 finding

Create one narrowly scoped task per reproducible operational issue. Do not pre-schedule speculative hardening.

### Allowed trigger examples
- pair-level artifact race/crash recovery -> consider completion manifest/ready marker;
- repeated false-positive/false-negative export classification -> refine deterministic policy;
- recurring backend freshness race -> refine revision lifecycle;
- repeated unnecessary context expansion -> adjust retrieval/selection heuristic;
- measurable host pressure attributable to code-intelligence lifecycle -> adjust index/query unload sequencing.

### Not valid triggers by themselves
- desire for a dashboard;
- desire for a generic policy DSL;
- theoretical elegance;
- adding Neo4j/GraphRAG;
- automatic RRI changes without repeated real-task evidence.

Each triggered T5 task must receive its own HP/EC examples, RRI computation, review route, acceptance criteria, and evidence before implementation.

---

## M4-T6 — Milestone closure and architecture decision

**Status:** pending  
**Effort:** S  
**Type:** docs/status  
**Depends on:** T4 and every triggered blocking T5 task

Close M4 after the operational contract is internally consistent and material findings are resolved or explicitly deferred.

### Acceptance criteria
- Plan, task ledger, audit entry point, schemas, and operator README agree on the operational contract.
- M4 closure explicitly records deferred issues and why they are safe to defer.
- Decide whether accumulated durability/cross-cutting impact now warrants a formal ADR.
- RRI/model-routing remains unchanged unless a separate approved task changed it.
- No product-runtime coupling was introduced.

**Evidence to emit:** final M4 closure note and ADR recommendation (`required`, `amend existing`, or `not required yet`).  
**Status artifacts affected:** this ledger, M4 plan, relevant audit/status docs.

**Agent handoff:** Close the milestone without expanding scope. Any new architectural capability becomes a successor milestone/task, not hidden M4 cleanup.
