---
type: TaskList
title: "DubBridge consumes Fenix — canonical INT task ledger"
status: planned
slice: INT
plan: docs/plan/dubbridge-consumes-fenix-integration.md
---

# Canonical DubBridge → Fenix tasks (2026-10-29)

> Product-owner correction: **DubBridge is the caller; Fenix is the agent/workflow service provider**. No implementation/E2E PASS for this new direction. [Plan](../plan/dubbridge-consumes-fenix-integration.md).

## B0 — Rebaseline

- [x] **B0.T1** Correct canonical architecture direction and domain authority.
- [x] **B0.T2** Mark reverse-direction plans, proposed ADR-048 and SQL schema as superseded; preserve history, do not deploy old design.
- [x] **B0.T3** Source-inventory Fenix agent/workflow routes; existing end-user authentication found, not verified for S2S.

## B1 — Contract & authorization

- [ ] **B1.T1** **SOURCE_REVIEW_PASS, LIVE_PENDING:** existing `agents/trigger` persists a run only; `workflows/{id}/execute` runs DSL inline; `agents/runs/{id}` returns workspace-scoped persisted status. [Source assessment](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/plans/dubbridge-fenix-b1-t1-api-runtime-assessment.md). Actual model/provider runtime test pending.
- [ ] **B1.T2** **FENIX_SECURITY_ADR_PROPOSED:** service identity/allowlist/revocation/actor attribution in [draft ADR](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/decisions/ADR-DB-FENIX-S2S-PROPOSED.md); not accepted, no credentials issued.
- [ ] **B1.T3** **CONTRACT_CANDIDATE:** [wire v1 proposal](../plan/dubbridge-fenix-b1-contract-v1-proposal.md) includes safe backlog counts, durable request IDs, accepted/status/result and explicit retry semantics; not frozen. RRI/peer/security review pending.

## B2 — Fenix provider (dependency)

- [ ] **B2.T1** Restricted S2S authentication and authorization exist at Fenix boundary.
- [ ] **B2.T2** Only allowlisted workflow/agent invocations pass; arbitrary CRM/workflow access denied.
- [ ] **B2.T3** Persist run status/result/trace with negative security tests.

## B3 — DubBridge consumer

- [ ] **B3.T1** Develop Rust opt-in Fenix API adapter; strict TLS, timeout, bounded payload, injected service credentials and no hard-coded secret.
- [ ] **B3.T2** Gather user-authorized review queue statistics from real DubBridge domain/DB; apply minimal data contract and no approval/publish mutations.
- [ ] **B3.T3** Durable outbound operation/run correlation, explicit pending/terminal/error, idempotent retry/reconciliation and safe response projection.

## B4 — Real end-to-end

- [ ] **B4.T1** Launch actual DubBridge/DB and actual Fenix runtime with a permitted execution workflow/agent.
- [ ] **B4.T2** Execute DubBridge → Fenix request, observe real run ID and terminal analysis, compare persisted records in both systems.
- [ ] **B4.T3** Deny unauthorized tenants, broad tokens, forbidden operations, expired/revoked credentials, duplicate submissions and untrusted returned payloads; exercise real outage.

## B5 — Gate/closure

- [ ] **B5.T1** Local Rust/Go test suites, security/RRI/peer reviews and relevant local pre-push QA.
- [ ] **B5.T2** Auditable E2E evidence with exact commands/commits/run IDs and sanitized traces.
- [ ] **B5.T3** Publish GO/NO-GO, separately assess any later Fenix → DubBridge tool callback.

**GitHub Actions:** owner reports globally disabled in both repositories, until new instruction. Never re-enable or cite disabled remote CI as successful QA.

**Status:** B0 documentation/source audit completed; B1-B5 implementation, review and real E2E are NOT VERIFIED. Stop if security/RRI gates require explicit reviewer verdict.

## 2026-10-29 B1 checkpoint

**Selected design engine:** Fenix DSL workflow execution, accessed through a new narrow provider S2S surface, not its existing human-authenticated routes. The existing workflow handler is synchronous; a durable async accept/poll API with request deduplication must be implemented and verified in B2. No false 202 accepted claim based on current behavior.

**Security owner:** Fenix issues/verifies/revokes service identity; DubBridge stores the client credential and authorizes its own user/org/project input. No shared JWT key or human token.

**Current result:** documented source review and candidate design only; B1 remains OPEN pending live API verification, RRI, independent security/peer review and accepted contract.
