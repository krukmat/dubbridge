---
type: Plan
title: "Local Code Intelligence M4 — Operational Adoption"
status: proposed
branch: feature/local-code-intelligence-boundary
predecessor: M3 closed
---

# Local Code Intelligence M4 — Operational Adoption

## Objective

Move the Local Code Intelligence Boundary from an implemented/integrated capability into routine DubBridge agent use without reopening M1–M3 or expanding it into a separate platform.

M4 is a **use-and-adjust** milestone. It hardens only the trust-boundary gaps that matter for real operation, adds bounded context expansion, and then lets normal DubBridge tasks drive any further refinement.

## Starting assumptions

- M1–M3 are closed by project decision.
- Existing agent workflow integration is authoritative; M4 does not redesign model routing, RRI, or the Analyze phase.
- Source, tests, ADRs, and repository policy remain authoritative over graph-derived context.
- The CKG/backend remains local and replaceable.
- Cloud agents receive bounded artifacts, never unrestricted graph traversal.
- No formal benchmark/POC program, metrics dashboard, Neo4j migration, or GraphRAG platform is introduced.

### Documentation reconciliation note

The current boundary README/audit still describes a real backend adapter/integration as a future step. M4 begins by reconciling those statements with the project's declared M3-closed state. This is documentation/status synchronization only; it must not re-open or re-implement M3.

## M4 architecture focus

```text
M3 closed operational path
        |
        v
Local graph result
        |
        v
Freshness gate ---------------------> reject stale graph
        |
        v
Defense-in-depth export policy ----> minimize cloud metadata/content
        |
        v
Context Receipt + Capsule
        |
        +-----------------------------+
        |                             |
        v                             v
   normal agent use            bounded expansion request
                                      |
                                      v
                              local policy evaluation
                                      |
                                      v
                               revised receipt/capsule
```

## Architectural priorities

1. **Freshness becomes an invariant, not provenance only.** Operational usage must not silently consume a graph built from the wrong repository revision.
2. **Cloud minimum disclosure covers metadata as well as source fragments.** Filtering source while leaking unrelated file/symbol/boundary metadata defeats part of the boundary.
3. **Backend classification is not the only control.** The gateway adds deterministic defense-in-depth for obviously unsafe paths/data and only exports metadata justified by allowed task-local evidence.
4. **Context expansion is explicit and bounded.** Missing context must not push cloud agents back into repository exploration.
5. **Operational evidence stays lightweight.** Real tasks produce receipts/capsules and only record friction that causes an adjustment; no KPI harness or synthetic A/B program is required.
6. **Hardening is demand-driven.** Pair-level transaction machinery, richer policy DSLs, RRI automation, ADR graphs, dashboards, and similar complexity remain deferred unless real usage exposes a concrete need.

## Task dependency graph

```text
M3 CLOSED
   |
   v
M4-T0 Baseline reconciliation
   |
   +-------------------+
   |                   |
   v                   v
M4-T1 Freshness     M4-T2 Export-policy hardening
   |                   |
   +---------+---------+
             |
             v
      M4-T3 Bounded expansion
             |
             v
      M4-T4 Operational adoption
             |
      +------+------+
      |             |
      v             v
 M4-T5a         M4-T5b...
 conditional hardening only when real friction exists
      |             |
      +------+------+
             |
             v
        M4-T6 Closure
```

T1 and T2 are logically parallel but both touch the gateway contract. Execute them sequentially unless isolated worktrees/patches make conflict risk negligible.

## Task summary

| Task | Purpose | Type | Depends on |
|---|---|---|---|
| M4-T0 | Reconcile M3-closed state with branch docs and actual operational entry point | docs/status | M3 closed |
| M4-T1 | Enforce graph-to-repository freshness | development | T0 |
| M4-T2 | Harden cloud metadata/content minimum disclosure | development | T0; execute after T1 by default |
| M4-T3 | Implement bounded, reasoned context expansion | development | T1, T2 |
| M4-T4 | Use the path on ordinary DubBridge tasks and record only actionable friction | operational | T3 |
| M4-T5 | Apply evidence-backed hardening only for observed friction | conditional development | T4 finding |
| M4-T6 | Close milestone, sync docs, and decide whether an ADR is now warranted | docs/status | T4 and any triggered T5 |

## Explicitly deferred unless triggered by M4-T4

- pair-level filesystem transaction machinery; a completion manifest/ready marker is justified only if a real asynchronous consumer race/crash-recovery issue appears;
- a generic classification policy DSL;
- graph-derived automatic RRI changes;
- symbol-to-ADR graph expansion;
- metrics dashboards or benchmark harnesses;
- multiple resident heavy local models;
- a new graph database or hosted service.

## Resource behavior

Keep the sequential host model:

```text
index/query -> receipt/capsule -> graph mostly idle -> one heavy local model -> review
```

M4 must not make the CKG a permanently resident reasoning agent. Context reduction is the desired resource benefit; no context-window reduction is required merely to complete this milestone.

## Milestone closure condition

M4 is complete when:

1. stale graph consumption fails closed on the operational path;
2. cloud output no longer exposes unrelated/unjustified metadata simply because it was present in backend arrays;
3. missing context can be expanded through a bounded local request/decision path without granting graph traversal;
4. the mechanism has been used on normal DubBridge work and any material friction discovered has either been fixed or explicitly deferred with rationale;
5. repository plan/task/audit/operator docs describe the same operational contract.

No formal performance target or benchmark threshold is required for closure.
