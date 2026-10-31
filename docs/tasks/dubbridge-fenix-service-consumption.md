---
type: TaskList
title: "DubBridge consumer of Fenix — neutral integration tasks"
status: planned
slice: INT
rebaseline: 2026-10-31
plan: docs/plan/dubbridge-consumes-fenix-integration.md
---

# Canonical DubBridge → Fenix tasks

**Owner decision:** specific agents/workflows are **defined later from real usage** (B6); no first hardcoded `review_backlog.analyze` requirement. See [canonical plan](../plan/dubbridge-consumes-fenix-integration.md) and [generic wire proposal](../plan/dubbridge-fenix-b1-contract-v1-proposal.md).

## B0 — Architecture and source inventory

- [x] **B0.T1** Caller direction corrected: DubBridge uses Fenix.
- [x] **B0.T2** Historical inverse security and SQL proposals withdrawn.
- [x] **B0.T3** Fenix API/user-JWT source inventory completed.

## B1 — Capability-neutral integration contract

- [x] **B1.T1** Source-level Fenix endpoint inventory: API shape and failure semantics; **no real business workflow required**.
- [ ] **B1.T2** Consumer security scope: user authorization in DubBridge, Fenix-owned service principal, tenant/workspace/capability mapping, revocation and operator credentials. Independent review pending.
- [ ] **B1.T3** Generic envelope + result/error/idempotency and registry-not-configured semantics; no hardcoded review metrics. Candidate contract/schema requires validation.
- [ ] **B1.T4** Task-level RRI/ADR/readiness gate. **Business-agent run is not a gate**.

## B2 — Fenix provider dependency

- [ ] **B2.A** Separate S2S identity and revocation at Fenix boundary.
- [ ] **B2.B** Default-deny capability registry, authorized workspace and permitted capability mapping; no workflow selected.
- [ ] **B2.C** Durable operation record and idempotent replay.
- [ ] **B2.D** Poll/status, trace, error and projection.
- [ ] **B2.E** Provider API tests for auth, capability absence, isolation, crash/retry.

## B3 — DubBridge client implementation

- [ ] **B3.T1** Rust HTTPS client with bounded JSON, timeouts, injected Fenix credential, no human JWT reuse.
- [ ] **B3.T2** DubBridge org/project authorization and neutral `capability_key` request builder; local mapping, not a review-specific business handler.
- [ ] **B3.T3** Persist `operation_id` linkage, status polling/error mapping, bounded retry and cancellation policy.

## B4 — Infrastructure integration E2E

- [ ] **B4.T1** Real DubBridge → Fenix HTTP connection using service identity (not mocks).
- [ ] **B4.T2** Prove denial for unknown/disabled capability, wrong org/workspace, revoked credential and forbidden operations. **No real workflow/LLM required**.
- [ ] **B4.T3** Verify idempotency, correlation and outage recovery across the two real services without fabricating a run.

**B4 test seam:** Use a locked-down nonproduction `integration.diagnostics` provider capability for real accepted-operation/idempotency/polling checks; no agent/LLM or business result. Production capability registry stays empty/default-deny. Record only infrastructure E2E evidence.

## B5 — Closure

- [ ] **B5.T1** Local Rust/Go QA, security/peer review and sanitized test evidence.
- [ ] **B5.T2** Operational handoff, disabled-by-default rollout and GO/NO-GO for foundation.

## B6 — Business capability lifecycle (later)

- [ ] **B6.T1** Capture and prioritize a concrete usage-driven workflow.
- [ ] **B6.T2** Register allowed Fenix capability, versioned schema, tools and model budgets.
- [ ] **B6.T3** Implement real agent/workflow, quality assurance, domain/E2E execution and controlled release.

## Historical checkpoint

[Prior B1.C NO-GO](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1c-gate-2026-10-31.md) was tied to a specific agent/workflow: **superseded as a foundation gate**. [Earlier example contract](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-v1-request.schema.json) and its test receipts are retained as historical experiments only. Real workflow validation happens at B6 and requires genuine model/runtime evidence then.

**Current:** B0 DONE, B1.T1 SOURCE PASS; B1.T2–T4 OPEN, B2–B5 NOT IMPLEMENTED, B6 DEFERRED. The fact that a workflow is undefined **does not block B1/B2 foundation work**. Never reenable GitHub Actions, merge to main or deploy credentials based on documentation alone.


## 2026-10-31 neutral foundation B1 validation

- [x] **B1.T1:** source inspection of the Fenix provider APIs is sufficient; real workflow definition/testing **only at B6**.
- [x] **B1.T3 candidate JSON verification:** exact committed provider schemas and scripts locally **PASS 17/17 directed and 120/120 state-matrix checks**. [Audit with blobs and scope](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b1-neutral-checkpoint-2026-10-31.md). This is wire-shape validation, NOT deployed interface or S2S authorization.
- [ ] **B1.T2/T4:** independent security/readiness and official RRI receipt pending. [Proposed S2S ADR](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/decisions/ADR-DB-FENIX-S2S-PROPOSED.md) now identifies separate service context, token revocation, tenant scopes, empty capability registry, request-size and policy enforcement needs.
- [x] **Next-block preparation:** [Fenix B2.A credential foundation tasks](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/tasks/dubbridge-fenix-b2a-s2s-identity.md) split by crypto, SQLite, isolated gateway and QA. **No Rust or Go runtime modifications made.**

**Decision:** B1 candidate contract tested; security gate open. Allowed next work is B2.A1 review/RRI. Do not implement or deploy auth changes, issue tokens, run a functional workflow or activate GitHub Actions based on schema tests.
