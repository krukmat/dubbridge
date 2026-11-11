---
type: Plan
title: "DubBridge consumes Fenix — capability-neutral integration foundation"
status: active_design
date: 2026-10-31
scope: INT
---

# Canonical integration plan — DubBridge → Fenix

> **Cloud development owner decision (2026-10-31):** RRI is **not applicable** to this integration when built with cloud coding agents. It cannot block B2.A or determine model selection. Retain independent peer/security review, Go/Rust QA and owner approval for auth/migrations/deployment. Historical RRI forecasts are informational only.



**Owner decision (2026-10-31):** This program **does not define or validate a real business workflow now**. Workflows, AI agents, prompts and business schemas are introduced incrementally **when usage establishes the need**. The original `review_backlog.analyze` example was wrongly made a prerequisite; it is **deferred and nonbinding**. It is not part of B1/B2/B3/B4 acceptance.

## B4 offline interop checkpoint — 2026-11-11

**Actual status:** [Cloud B4 audit](../audit/dubbridge-fenix-b4-offline-interop-2026-11-11.md) and [issue #13](https://github.com/krukmat/dubbridge/issues/13). **B4.T1 live authenticated HTTP remains blocked** by unmounted Fenix endpoint, operator capability/token provisioning, security T4 and Go/Rust gates. **B4.T2/T3 have source tests and an executed 28/28 Cloud contract simulation**, not real two-service E2E.

Concrete cross-repo changes: Fenix proposed `admission-ack`/`admission-error` schemas and a Go `S2SAdmissionReceipt` JSON projector; DubBridge pinned those exact schemas with blob SHA, strengthened Rust success/denial parsing, and added an offline dependency-free Node contract checker. 24/24 source assertions also passed, but Node CLI, Go/Rust tests and independent review remain NOT_RUN. `202` is new reservation, `200` is replay; both require the same operation and trace IDs. `503` and unknown delivery states are ambiguous, not automatic retries. No C4 GET/status is implemented or required for this offline preparation.

**No production route, token, capability activation, hosted E2E, database migration, CI, worker, agent or main merge.** B4 is not closed; B5 release readiness cannot be claimed. B6 remains dependent on real future usage.

## Current B3 Cloud implementation checkpoint — 2026-11-11

**Source-only B3.T1–T4 implementation** is now published on `feature/int-v2-dubbridge-consumes-fenix` in the existing connectors and API crates. [Task ledger](../tasks/dubbridge-fenix-service-consumption.md) and [issue #12](https://github.com/krukmat/dubbridge/issues/12) hold paths and verification limits.

- **T1:** opt-in HTTPS client in `crates/connectors/src/fenix_s2s.rs`, with Rustls, redirects disabled, bounded 64-KiB envelopes/8-KiB responses, 30-second timeout ceiling, a short-lived Fenix-issued bearer (never human JWT), and redacted Debug.
- **T2:** async `FenixScopeAuthorizer` invoked *before* constructing the neutral B1 request; dormant `apps/api/src/fenix_scope.rs` queries live PostgreSQL org-membership role and project ownership, rejecting errors or mismatches. Actual caller wiring and capability-specific role decision remain later.
- **T3:** caller must already hold a durable local work-item ID; key/trace UUIDv5 derivation is stable, conflicts don't rewrite the key, and ambiguous sends are never auto-retried. **No polling:** B2.C4 GET/status remains deferred, so there is no fake resolver. Actual durable local linkage/reconciliation and the public Fenix acknowledgement remain later contracts.
- **T4:** static fake-transport/unit tests authored in Rust; [Cloud audit](../audit/dubbridge-fenix-b3-cloud-source-audit-2026-11-11.md) records **33/33 source predicates PASS**, not executed Rust QA. The operation request's fields are private so production callers must use a stable work-item factory.

**Source snapshot:** `72c356801b630a8d5d02619ed904b521bd549c84`. **Gate status:** no `rustc`, `cargo`, local tokens, independently authorized review or actual Fenix HTTP route available in this Cloud session. B3 is therefore **published source candidate / QA and review pending**, not closed or deployable. Current Fenix B2.E admission remains dormant, without production URL or approved input schemas; B2.B4/T4 and owner's activation/security approvals are still distinct gates. No token, paid hosting, production route, CI, migration, real workflow, deployment or main merge was authorized or used here. **RRI N/A for cloud DB-FENIX-INT**.

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

**Testability without an actual workflow:** B4 may enable a strictly isolated nonproduction `integration.diagnostics` capability backed by a bounded deterministic Fenix provider diagnostic handler. It exercises **real authentication, durable SQLite operations, HTTP status, idempotency and recovery**, without impersonating an agent or providing business/model results. Production registry remains default-deny and has no domain capabilities registered. Label such tests **infrastructure E2E**, not functional agent E2E.

## Blocks, tasks, dependencies (two-level)

| Block | Tasks | Dependency / acceptance |
|---|---|---|
| **B0 — Orientation** | B0.T1 caller/provider direction; B0.T2 archive inverse artifacts; B0.T3 source inventory | DONE |
| **B1 — Foundation contract** | B1.T1 source API inventory only; B1.T2 S2S principal and tenant/workspace policy; B1.T3 generic request/status/idempotency schema; B1.T4 security/RRI review | B0; **no live workflow required** |
| **B2 — Fenix foundation** | B2.A separate S2S auth and revocation; B2.B registry/permission denial; B2.C SQLite operation dedupe; B2.D status/trace and recovery; B2.E real provider-side negative tests without registered domain capability | B1 security and task-level approval |
| **B3 — DubBridge client** | B3.T1 opt-in Rust TLS/secret client; B3.T2 authorized local org/project mapping and neutral request; B3.T3 stable idempotency/timeout/ambiguous-outcome policy (**GET/status deferred**); B3.T4 fake transport/tests and Cloud audit | B1 schema + inert B2 source; actual API activation remains gated |
| **B4 — Infrastructure E2E** | B4.T1 authenticated HTTP between real apps (**blocked, endpoint not mounted**); B4.T2 denied identity/capability/tenant (**offline source + simulation**); B4.T3 idempotency/trace/ambiguous outcomes (**offline source + simulation; GET/status deferred**) | B2+B3; **real E2E pending**, no LLM/workflow needed |
| **B5 — Foundation closure** | B5.T1 local Go/Rust QA, security review; B5.T2 evidence/runbook; B5.T3 GO/NO-GO | B4 |
| **B6 — Capabilities on demand** | B6.T1 choose use case from actual needs; B6.T2 register capability/schema/policy; B6.T3 implement and certify real workflow/agent + business E2E | B5 + separate use-case decision |

## Corrected sequencing and gates

The prior [Fenix B1.C historical NO-GO](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1c-gate-2026-10-31.md) conflated foundation with a **specific workflow**. It is superseded as a roadmap gate: B1 is **not blocked** by a missing model, active workflow or human-user test JWT. The older RRI 122 is a **hypothetical monolithic forecast** requiring task decomposition, not a measured security approval.

Security is **not** deferred: isolated service identities, tenant binding, capability allowlisting, negative auth tests, durable operation handling and independent review remain mandatory before deployment. No arbitrary capability can be executed before registration.

Historical review-count JSON schemas and the local live-workflow probe are **archived concept artifacts** for possible B6 usage; previous offline schema receipts do not certify this neutral contract.

No paid hosting, main merge, GitHub Actions workflows, S2S tokens or application runtime have been changed by the planning correction.

## Current checkpoint

**B0 DONE. B1 design/source audit PARTIAL (workflows no longer a blocker). B2–B5 NOT IMPLEMENTED. B6 DEFERRED until usage.** Next work: neutral envelope security/schema review → task-scoped B2.A identity layer.


## B1 contract validation checkpoint — 2026-10-31

Provider evidence: [Fenix generic envelope audit](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1-neutral-checkpoint-2026-10-31.md) reports **17 directed PASS + 120 matrix PASS** against the *current* generic request/response/error schemas using matching Git blob SHA. This verifies **schema shape only**, not S2S authorization or live transport. Fenix security ADR remains proposed; independent security/official RRI gates are still open. [Decomposed B2.A](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/tasks/dubbridge-fenix-b2a-s2s-identity.md) is the next task-readiness scope. B6 alone defines real business workflows when needed; no B1–B5 business-workflow dependency.
