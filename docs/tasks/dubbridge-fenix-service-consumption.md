---
type: TaskList
title: "DubBridge consumer of Fenix — neutral integration tasks"
status: planned
slice: INT
rebaseline: 2026-10-31
plan: docs/plan/dubbridge-consumes-fenix-integration.md
---

# Canonical DubBridge → Fenix tasks

## B4.W1/W3.1 execution update — 2026-11-13

**W1.1/W1.2 Fenix:** standalone nonproduction S2S router candidate + explicit build-tag tests committed; **no runtime mount, no tagged Go test/reviewer certification**. Source is available only under `s2s_nonproduction` tag and exact `integration-test` opt-in.

**W3.1 DubBridge:** complete; [source-backed durable-ID inventory](../audit/dubbridge-fenix-b4w3-1-durable-intent-inventory-2026-11-13.md). **No safe neutral persisted ID exists**; translation, review and publication UUIDs cannot stand in as a cross-capability work item. **W3.2** needs a neutral immutable PostgreSQL dispatch-intent record scoped to trusted user/org/project before sending; **W3.3** then wires the Rust connector and records delivery/indeterminate outcomes. No migration added or applied in this block. W2 diagnostics and W5 reconciliation remain next; W4 activation still unapproved.

## CURRENT — GO CONDICIONAL, resumed B4 wiring (2026-11-13)

[Authoritative task dependencies and responsible parties](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/tasks/task_dubbridge_fenix_b4_operational_wiring.md). Owner has approved **progressing with technical source integration**, not real S2S activation. Existing Rust opt-in connector and PostgreSQL org/project authorizer are source candidates; **application caller/wiring and durable local work-item linkage are missing**.

Next DubBridge work **W3.1** identify the existing persisted work-item and derive stable UUIDv5 `operation_id`; **W3.2/W3.3** connect an explicitly disabled-by-default API-side caller, real trusted scope authorizer, credential-provider injection, sanitized response and durable operation correlation. Concurrent Fenix **W1** default-OFF dedicated S2S route then **W2** deterministic diagnostics. **W5** persists indeterminate/timeout and prevents retry without verified reconciliation, while **C4 GET/status stays deferred**. **W4 token/capability/route provisioning requires separate explicit owner approval**; B4.T1–T3 real nonproduction HTTPS follows, B5 certification later.

**Owner of planning/source:** ChatGPT. No local agent coding, no product workflow or LLM until B6, no CI, fresh branch, production credential, public route, deployment, paid hosting or main merge. Historical B4.Q QA failures remain recorded but are not a stop to dormant source development. Older "B2–B5 NOT IMPLEMENTED" text is historical.

## B4 Cloud offline interop checkpoint — 2026-11-11

**Tracker:** [#13](https://github.com/krukmat/dubbridge/issues/13); **evidence:** [B4 offline audit](../audit/dubbridge-fenix-b4-offline-interop-2026-11-11.md). **Owner:** ChatGPT Cloud, RRI N/A; no local tokens available.

| Task | Current outcome | Status |
|---|---|---|
| **B4.T1 — real two-service HTTPS** | Proposed dedicated Fenix ack/error schemas; inert Go wire projector and stricter Rust parser published; client endpoint **not** enabled | **BLOCKED_LIVE_ENDPOINT**, not executed |
| **B4.T2 — tenant/capability/credential rejection** | Go provider-side in-process negative test source for wrong org/project/workspace, denied/revoked principal, missing/disabled capability; strict Rust error-code/status/body; offline schema negative cases | **MOCK_SOURCE_DELIVERED**, not real E2E |
| **B4.T3 — idempotency/correlation/outage** | Go proposed ack adapter + tests for 202 new, 200 replay, 409 conflicting trace, zero agent runs; Rust no-auto-retry on ambiguity; exact-contract offline checker | **MOCK_SOURCE_DELIVERED**, not real E2E |

**Executed Cloud evidence:** 28/28 JS contract checks PASS against exact pinned Fenix schemas; 24/24 static cross-language source predicates PASS. The repeatable Node checker was authored but **not run with Node** here. **Go/Rust project tests, peer/security review and authenticated HTTP E2E NOT_RUN**. Do not close B4 or upgrade source candidates to production security approval; B2.B4/T4 operator gates and Fenix route remain missing.

**Contract nuance:** the historic B1 error proposal does not match the newer B2.E projector; a *separate proposed admission-error schema* now aligns Go/Rust safe classifications. This is not C4 lifecycle GET/status (still deferred). No new user/agent workflow or token is required for the offline portion.

## B3 Cloud source checkpoint — 2026-11-11

**Owner approval:** B3 block approved in the live conversation. **Branch:** `feature/int-v2-dubbridge-consumes-fenix`. **Implementation owner:** ChatGPT Cloud; local agent tokens unavailable. **RRI:** N/A for DB-FENIX-INT cloud. **Issue:** [#12](https://github.com/krukmat/dubbridge/issues/12).

| B3 task | Source delivery | Cloud state | QA/peer |
|---|---|---|---|
| **B3.T1** | `crates/connectors/src/fenix_s2s.rs`: opt-in HTTPS-only Rustls/reqwest client, service-token prefix, redacted Debug, 30s timeout cap, redirect denial, bounded request/response | **Candidate published** | NOT_RUN |
| **B3.T2** | Async Fenix local policy seam and `apps/api/src/fenix_scope.rs` live PostgreSQL `org_members` role + `projects` parent check; B1 org/project/capability-neutral envelope builder | **Candidate published** | NOT_RUN |
| **B3.T3** | Stable operation/trace UUIDv5 from a *pre-existing persisted local work item*, one-shot send, strict proposed acknowledgement, no automatic retry after ambiguous send, explicit reconciliation advice | **Candidate published** | NOT_RUN |
| **B3.T4** | Rust tests authored for policy rejection/cross-tenant mismatch, bounded body, human JWT rejection, HTTPS config, invalid/mismatched ack, fake transport and no auto retries; [Cloud source audit: 33/33 static PASS](../audit/dubbridge-fenix-b3-cloud-source-audit-2026-11-11.md) | **Cloud static audit complete** | NOT_RUN |

**Important task limitation:** T2 uses actual PostgreSQL queries in a dormant implementation, but no real application caller or selected capability role has been connected. T3 reconstructs an idempotency key from a caller-supplied **already persisted** local work-item UUID; the caller-side durable work record and an approval/reconciliation policy are **not** implemented by this block. Thus neither production authorization nor persistent end-to-end linkage is claimed.

**Contract limits:** No GET/status polling (C4 deferred); no live Fenix HTTP endpoint, registered business capability, agent execution, credentials, CI, migrations, deployment or main merge. The proposed admission-ack wire shape still requires agreement with Fenix before real activation. The Fenix B2.E source is an inactive candidate and its Go QA is separate.

**Final Cloud snapshot:** last source commit `72c356801b630a8d5d02619ed904b521bd549c84`, with request operation key construction restricted to the deterministic persisted-work-item factory. **33/33 read-only static predicates PASS**; neither compilation nor runtime tests are implied.

**Verification boundary:** The remote Cloud execution environment lacks `cargo`, `rustc` and access to the user's local checkout. No Rustfmt, cargo test/check/clippy, peer review or security acceptance was run; no PASS or waiver is implied. Follow-on changes can remain dormant source candidates without spending local tokens.

## Current authoritative provider checkpoint — 2026-11-05

Fenix branch `feature/int-v2-dubbridge-consumes-fenix` has progressed: **A1 opaque tokens**, **A2 Go digest-only principal store (migration proposal only)**, **A3 isolated S2S middleware (not routed)** and **B2.B empty/default-deny scoped capability registry**, all with exact-blob isolated local Go/race/vet tests PASS. **B2.C idempotency has only a SQLite proposal and offline tests**; no runtime operation API. [Provider audit](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b2-cloud-checkpoint-2026-11-05.md). **No Fenix production endpoint, accepted real workflow, secret or DubBridge Rust client yet.** Independent security review/full Fenix QA/actual SQLite migration and transport E2E remain pending; RRI N/A.


## Cloud integration checkpoint — 2026-11-05

Fenix `feature/int-v2-dubbridge-consumes-fenix` now contains [B2.A1 isolated Go S2S token source and tests](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/internal/domain/auth/service_token.go), with repeat/race/vet/format PASS and **95.7% isolated coverage**. The [A2 SQLite principal schema](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/contracts/dubbridge-fenix-b2a2-principal-sqlite-proposal.up.sql) is a non-runtime proposal under `docs/contracts` (negative/revocation/rollback offline checks PASS), not yet integrated into Fenix migrations. [Exact scope and remaining gates](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/audit/dubbridge-fenix-b2a1-cloud-2026-11-05.md).

**Pending:** independent security review and full Fenix QA; A2 Go persistence, A3 middleware, A4 acceptance, actual Rust S2S client. No service credential or real workflow is active. This is not an E2E PASS. Owner's RRI N/A policy applies to cloud work.



> **Cloud development owner decision (2026-10-31):** `RRI: N/A` for this DubBridge→Fenix integration executed by cloud coding agents. RRI forecasts and outstanding RRI receipts in prior checkpoints are **not gates**. Continue requiring independent security/peer review, Go/Rust tests and approval for sensitive changes. The missing peer reviewer has **not** been waived.



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


## Fenix B2.A1 task-ready checkpoint — 2026-10-31

The owner accepted the next B2.A block. [Fenix B2.A1 bounded token task and risk packet](https://github.com/krukmat/fenix/blob/feature/int-v2-dubbridge-consumes-fenix/docs/tasks/task_dubbridge_fenix_b2a1_credential_readiness.md) was committed. Fenix computed a **policy-based forecast RRI 67**, not a signed script result. **Mandatory independent peer task-readiness is blocked** in the execution container (Claude/Codex not installed). Therefore no service-token Go implementation or live service credential exists yet; DubBridge Rust remains unchanged. This security gate is unrelated to workflow definition (deferred to B6).
