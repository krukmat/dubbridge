---
type: Plan
title: "DubBridge consumes Fenix — capability-neutral integration foundation"
status: active_design
date: 2026-10-31
scope: INT
---

# Canonical integration plan — DubBridge → Fenix

**Owner decision (2026-10-31):** This program **does not define or validate a real business workflow now**. Workflows, AI agents, prompts and business schemas are introduced incrementally **when usage establishes the need**. The original `review_backlog.analyze` example was wrongly made a prerequisite; it is **deferred and nonbinding**. It is not part of B1/B2/B3/B4 acceptance.

## Intent and domain boundary

DubBridge is the consuming product: owns authorized users, organizations/projects, multimedia, review/rights, approval, publication and P2P. Fenix provides a platform-side identity boundary, governed capability registry, agent/workflow orchestration engine, Blackboard, ToolRegistry and persisted execution status. Integration does not transfer domain decision-making to Fenix.

```text
DubBridge user/service -> DubBridge API authorization
 -> DubBridge S2S client -> Fenix S2S API
 -> check service principal + mapped tenant/workspace + capability status
 -> [registered and enabled capability?] yes -> future workflow/agent dispatch
                                 no  -> explicit rejection, no run
 -> status/trace/result correlated by operation_id -> DubBridge
```

`capability_key` is an abstract **configured logical capability**, not a user-selected workflow ID. The initial foundation ships with **no domain capability enabled**. The registry is deny-by-default; unknown capability requests do not launch any model or fabricated operation.

## Technical baseline and contract

- Fenix's existing routes use human JWTs; `workflows/{id}/execute` invokes DSL inline and `agents/trigger` by itself creates a run, not actual execution. Design a **separate, restricted S2S boundary**.
- Fenix persists in its own **SQLite**, DubBridge in PostgreSQL. No shared token signing secret, login credential or DB.
- Versioned generic envelope with `contract_version`, `operation_id`, `capability_key`, `input_schema_version`, `trace_id`, authorized context, bounded `input`; no embedded DSL/prompt/arbitrary workflow ID.
- On registered capability, input/output schema is required **per capability**; until then unknown capabilities are rejected before execution.
- Fenix controls operator provisioning, credential revocation/rotation, allowed workspace, permitted capabilities and tenant mapping; DubBridge authorizes local user/org/project before sending a request.
- Durable operation reservation and idempotency are backend capabilities. `operation_id` is the stable GET key; `run_id: null` is expected until a real run is allocated; no duplicate execution retries after uncertain acceptance.
- Responses are sanitized, status-oriented. Generic envelope cannot return unvalidated model internals or allow direct mutation of DubBridge.

Canonical [generic wire contract](dubbridge-fenix-b1-contract-v1-proposal.md), [Fenix source/provider plan](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/plans/dubbridge-consumes-fenix-integration.md).

## Blocks, tasks, dependencies (two-level)

| Block | Tasks | Dependency / acceptance |
|---|---|---|
| **B0 — Orientation** | B0.T1 caller/provider direction; B0.T2 archive inverse artifacts; B0.T3 source inventory | DONE |
| **B1 — Foundation contract** | B1.T1 source API inventory only; B1.T2 S2S principal and tenant/workspace policy; B1.T3 generic request/status/idempotency schema; B1.T4 security/RRI review | B0; **no live workflow required** |
| **B2 — Fenix foundation** | B2.A separate S2S auth and revocation; B2.B registry/permission denial; B2.C SQLite operation dedupe; B2.D status/trace and recovery; B2.E real provider-side negative tests without registered domain capability | B1 security and task-level approval |
| **B3 — DubBridge client** | B3.T1 Rust client + TLS/secret; B3.T2 local tenancy authorization and neutral capability envelope; B3.T3 operation polling/timeout/idempotency/error | B1 schema + B2 API |
| **B4 — Infrastructure E2E** | B4.T1 authenticated HTTP between real apps; B4.T2 deny unknown/disabled capability, wrong tenant/revoked token; B4.T3 status/idempotency/retry/outage evidence | B2+B3; **no LLM/workflow needed** |
| **B5 — Foundation closure** | B5.T1 local Go/Rust QA, security review; B5.T2 evidence/runbook; B5.T3 GO/NO-GO | B4 |
| **B6 — Capabilities on demand** | B6.T1 choose use case from actual needs; B6.T2 register capability/schema/policy; B6.T3 implement and certify real workflow/agent + business E2E | B5 + separate use-case decision |

## Corrected sequencing and gates

The prior [Fenix B1.C historical NO-GO](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1c-gate-2026-10-31.md) conflated foundation with a **specific workflow**. It is superseded as a roadmap gate: B1 is **not blocked** by a missing model, active workflow or human-user test JWT. The older RRI 122 is a **hypothetical monolithic forecast** requiring task decomposition, not a measured security approval.

Security is **not** deferred: isolated service identities, tenant binding, capability allowlisting, negative auth tests, durable operation handling and independent review remain mandatory before deployment. No arbitrary capability can be executed before registration.

Historical review-count JSON schemas and the local live-workflow probe are **archived concept artifacts** for possible B6 usage; previous offline schema receipts do not certify this neutral contract.

No paid hosting, main merge, GitHub Actions workflows, S2S tokens or application runtime have been changed by the planning correction.

## Current checkpoint

**B0 DONE. B1 design/source audit PARTIAL (workflows no longer a blocker). B2–B5 NOT IMPLEMENTED. B6 DEFERRED until usage.** Next work: neutral envelope security/schema review → task-scoped B2.A identity layer.
