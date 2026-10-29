---
type: Plan
title: "DubBridge consumes Fenix — Agentic services integration"
status: proposed
date: 2026-10-29
scope: INT
---

# Canonical INT plan — DubBridge → Fenix

> **Direction confirmed by product owner (2026-10-29):** DubBridge is the CLIENT/PRODUCT; Fenix is the PROVIDER of agentic planning, agents and workflows. This replaces the prior Fenix → DubBridge review-queue initiative. This is a planning decision, not an implementation or E2E PASS.

## 1. Mission and boundaries

DubBridge requests analytical/agentic execution from Fenix and receives a verifiable outcome. DubBridge continues to own project state, multimedia, review decisions, rights, approval, publishing and P2P. Fenix owns agent/workflow runtime, planning, Blackboard, ToolRegistry, capability governance and its own execution audit. The two systems run independently.

**Canonical direction:**
```text
DubBridge UI/domain event
   -> DubBridge integration client (validated command + operator policy)
   -> HTTPS S2S -> Fenix execution API (authz for DubBridge service identity)
   -> Fenix agent/workflow runtime -> tools/Blackboard as authorized
   -> Fenix run ID / status / minimized result / trace
   -> DubBridge correlation + persisted result or human-visible proposal
```

An eventual Fenix → DubBridge **tool callback**, for Fenix to query permitted domain facts, is a separate opt-in capability and must not be smuggled into this first slice. Fenix never becomes DubBridge's domain authority.

## 2. First vertical use case

An authorized DubBridge operator requests an **analysis of a project review backlog**. DubBridge gathers authorized, minimal review-task facts from its real PostgreSQL-backed services, then submits a bounded, versioned analysis request to Fenix. Fenix executes an allowlisted agent or workflow and returns an identified outcome. DubBridge persists execution linkage and displays an advisory summary; **no approve/reject/publish/rights/P2P mutation** occurs automatically.

The first E2E uses **real DubBridge API/domain+DB and real Fenix agent/workflow execution**. Unit tests may mock network boundaries but no mocked service, JWT or canned response qualifies as E2E proof.

## 3. As-is reuse opportunities / gaps

- **Fenix** has `POST /api/v1/agents/trigger`, `GET /api/v1/agents/runs/{id}`, `POST /api/v1/workflows/{id}/execute` and a governed runtime, per `internal/api/routes.go` and `internal/api/handlers/{agent,workflow}.go`. These routes presently run behind Fenix's generic JWT middleware; **a service identity restricted to DubBridge has not been validated**. Do not reuse a broad human login token.
- **DubBridge** already has review queue state and its own authorization, `apps/api/src/routes/review.rs` + `crates/db/src/review_repo.rs`. Reuse these for gathering facts; no reverse public review endpoint is required for the first slice.
- A run's precise response/async behavior, scope enforcement, model/provider readiness and persistence must be measured against real Fenix runtime. Do not assume an existing endpoint is suitable for external S2S merely because it exists.
- The old `fenix-readonly` principal schema and ADR-048 proposed a **Fenix → DubBridge** identity: not applicable to the canonical direction. Neither has been applied/accepted.

## 4. Integration contract (to freeze in B1)

Proposed semantic request (schema is *candidate*, not an existing API):
```json
{
  "contract_version": "1",
  "operation": "dubbridge.review_backlog.analyze",
  "workflow_ref": "allowlisted-fenix-workflow",
  "idempotency_key": "dubbridge-operation-id",
  "source": {"product":"dubbridge","org_id":"uuid","project_id":"uuid"},
  "trace_id": "bounded-correlation-id",
  "input": {"review_counts": {"pending":0,"approved":0,"rejected":0}}
}
```
Full task lists, personal identifiers, secrets, private media, rights evidence, credentials and raw project artifacts do not leave DubBridge unless a separately authorized, minimized contract requires it. Use Fenix-owned run IDs, terminal states and validation of returned data. DubBridge controls **which** Fenix workflow may run; callers cannot supply arbitrary Fenix workflow IDs or prompt/config URLs.

**Trust:** DubBridge authenticates a caller locally, enforces org/project authorization before gathering facts, and connects to Fenix via TLS + a *Fenix-issued/recognized* restricted service credential bound to the allowed Fenix workspace and operations. Fenix validates the service credential, audience/issuer/expiry and scope on every request, independently of DubBridge's claims. No shared signing secrets or database connection. Design token lifecycle, revocation, rotation and tenant mapping in Fenix, not DubBridge.

**Execution:** choose synchronous/async run contract only after examining actual endpoint semantics and run status handling. Initial safest design is async with bounded poll/backoff, idempotency mapping, explicit timeout, no duplicate submission, terminal-state persistence, and trace propagation. Never retry an unknown acceptance state without deduplication semantics.

## 5. Two-level tasks and dependencies

| Block | Task leaves (max two levels) | Depends on | Owner | Status |
|---|---|---|---|---|
| **B0 — Rebaseline** | **B0.T1** fix canonical direction; **B0.T2** quarantine reverse-direction branches/ADR; **B0.T3** snapshot actual Fenix endpoints | — | Both | Planning in progress |
| **B1 — Contract & S2S** | **B1.T1** validate agent/workflow execution API behavior; **B1.T2** define restricted Fenix client credential and DubBridge-to-Fenix tenancy; **B1.T3** freeze schema, async/idempotency and security ADR + RRI/reviews | B0 | Fenix + DubBridge | Pending |
| **B2 — Fenix provider** | **B2.T1** implement Fenix service authorization; **B2.T2** expose/guard allowlisted workflow invocation; **B2.T3** run/status/result/audit contract and negative tests | B1 | Fenix | Pending |
| **B3 — DubBridge consumer** | **B3.T1** implement opt-in Fenix HTTP adapter and bounded secret config; **B3.T2** authorized review summary construction; **B3.T3** persist run linkage, status/retry and safe response | B1, B2 contract | DubBridge | Pending |
| **B4 — Real E2E** | **B4.T1** real DB/agent/workflow bootstrap; **B4.T2** execute request and verify run/result; **B4.T3** wrong workspace/org/project, expired/revoked token, replay, outage and mutation denial | B2, B3 | Both | Not run |
| **B5 — Closure** | **B5.T1** full repo QA and peer/security review; **B5.T2** publish sanitized trace/DB evidence; **B5.T3** GO/NO-GO for next functional slice | B4 | Both | Not started |

### B1 dependencies

B1.T1 is a **source-and-live-contract validation** task, not permission to broaden Fenix's existing JWT. B1.T2 must resolve *service* vs *human* actor attribution, allowed Fenix workspace, secret owner, provisioning/rotation/revocation, and isolation from Fenix CRM users. B1.T3 must decide the minimum payload needed for a useful recommendation without exposing reviewer data. Do not implement B2/B3 S2S code until governance/readiness gates pass.

## 6. Acceptance and operational constraints

1. DubBridge API/service submits an authorized real request to the real Fenix API.
2. Fenix accepts only the restricted DubBridge service identity and allowlisted operation/workspace.
3. Fenix runs a genuine workflow/agent (not a placeholder output) and persists a queryable run; DubBridge sees matching terminal result and trace.
4. Replaying the same logical request never accidentally executes twice; uncertain network outcomes fail closed.
5. Unauthorized org/project, service identity, excessive scopes and unrelated Fenix operations fail; neither side can authorize the other's domain mutations.
6. No forced cloud hosting, no paid service, no GitHub Actions activation. Owner reported Actions disabled; verify local QA independently.
7. Full local Rust/Go QA and independent reviews before merge; no claim of production readiness from docs or unit tests.

## 7. Prior work disposition

- **Fenix branch** `feature/int-w1-dubbridge-readonly`: outgoing `dubbridge.review_queue.list` adapter and startup wiring exist but are **WRONG DIRECTION** for this initiative. Do **not merge/deploy/use** as B1/B2/B3 deliverable. Keep for archival/reference until an independently approved separate use case is defined; remove startup wiring from any future integration base.
- **DubBridge branch** `feature/int-w1-fenix-service-identity`: the former Fenix-read-only identity plan, candidate SQL, and proposed ADR-048 are **WITHDRAWN** as a design direction. No database migration applied; no service issuer implemented.
- Historical W0/W1 texts remain as source-audit history, explicitly superseded by this plan.

## 8. Status / next checkpoint

**GO for corrected planning only. NO-GO for operational E2E.** Next executable leaf: **B1.T1 Fenix execution API behavior review**; then B1.T2 service authority and B1.T3 contract/RRI/independent peer-readiness. No implied prior security approval.
