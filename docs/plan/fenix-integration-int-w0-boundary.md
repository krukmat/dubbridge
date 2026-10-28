---
type: Plan
title: "INT-W0: DubBridge-owned boundary for real Fenix integration"
status: draft
slice: INT-W0
governed_by: [ADR-008, ADR-030, ADR-031]
---

# INT-W0 — DubBridge-owned integration boundary

## Scope and evidence status

This document captures the existing DubBridge contracts and security work necessary for a **real** Fenix integration, solely on `feature/int-w0-fenix-integration-boundary`. It does not change runtime, credentials, production configuration, data, or deployment.

The approved first Fenix user case is read-only **project review queue inspection** with proposals returned to Fenix agents. **No mocks/stubs/fixture-only E2E evidence** may be substituted for a running DubBridge API using genuine JWT verification, a persistent PostgreSQL organization/project/review task and real Fenix invocation. Test data may be deliberately created in a local test database; the API and authentication must be real.

Companion Fenix-owned integration plan:
[INT-W0 Fenix ToolRegistry contract](https://github.com/krukmat/fenix/blob/feature/int-w0-dubbridge-integration-contract/docs/plans/fenix-dubbridge-int-w0-integration.md).

## Verified current code paths

| Existing behavior | Source in DubBridge |
|---|---|
| GET project review queue | `apps/api/src/routes/review.rs`: `/orgs/{org_id}/projects/{project_id}/review-tasks` |
| Gateway bearer relay | `apps/gateway/src/{lib,proxy}.rs`: `/api/*` -> backend |
| Existing response DTO | `apps/api/src/dto/review.rs`: `ReviewQueueResponse`, `ReviewTaskResponse` |
| App-issued JWT | `crates/auth/src/{issuer,service,principal}.rs` |
| Scope and reviewer membership | `routes/review.rs`, `apps/api/src/middleware/org_scope.rs` |
| Rights validation | `docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md` |
| Immutable decisions and publication gate | `docs/adr/ADR-030-review-decision-ledger-and-fail-closed-publication-gate.md` |
| JWT/mobile trust model | `docs/adr/ADR-031-mobile-jwt-credential-auth-fenix-parity.md` |

Actual endpoint on backend:
`GET /orgs/{org_id}/projects/{project_id}/review-tasks`.
Through gateway: `GET /api/orgs/{org_id}/projects/{project_id}/review-tasks`.

The existing endpoint returns `{org_id,project_id,tasks:[...]}`; each task includes `id`, `asset_id`, `target_language_id`, `assignee_subject_id`, `state` (`pending`, `approved`, `rejected`), and timestamps, plus org/project fields.

The route requires a **DubBridge-issued** verified bearer JWT, scope `workspaces:read`, organization membership and at least `Reviewer` role. The handler checks the project belongs to that organization. A Fenix caller's own JWT is not an equivalent credential.

## Critical identity gap

`crates/auth/src/service.rs` currently defines `DEFAULT_AUTH_SCOPES` including `assets:ingest` and `workspaces:write`, while `AuthService.login` issues the account's default scopes. Copying or storing that user's login token in Fenix would grant unnecessary mutation authority. Sharing `DUBBRIDGE_JWT_SECRET` with Fenix is prohibited.

Before any real INT-W1 invocation, design and implement a **separately authorized service identity** in DubBridge. Required properties:

1. Token minted or otherwise recognized **by DubBridge**, not crafted by Fenix.
2. Principal bound to an explicitly authorized organization/project and effective reviewer membership; no arbitrary tenant selection from untrusted request fields.
3. Least privilege: `workspaces:read` only for the chosen GET surface. No ingest, write, approve, publish, invite, or media-key access.
4. Bounded expiry, rotation/revocation plan, secret injection/redaction, auditable issuer/actor attribution and secure transport.
5. Compatibility with accepted ADR-031 for first-party mobile auth: service identity design must be reviewed as its own decision; do not silently rewrite or bypass the existing issuer.
6. Explicit mapping between Fenix workspace/user and DubBridge org/project, without treating matching UUIDs or email values as proof of authority.

**Status: design requirement, not implemented.** Until an acceptable service principal can be created and verified, the live cross-product integration verdict is BLOCKED.

## Correlation and audit (proposed gap closure)

Fenix proposes to send `X-Fenix-Trace-ID` and `X-Fenix-Execution-ID`. The gateway currently relays non-prohibited headers, but source inspection did **not** prove DubBridge handlers persist or emit these values in structured audit/response. A DubBridge-owned change must establish trace ingestion and safe audit propagation if the end-to-end correlation gate requires it.

Fenix controls its own agent/tool audit, while DubBridge controls its own domain decisions and per-request authorization. Neither system should treat the other's policy outcome as a bypass. No duplicated rights ledger, review decision ledger, or P2P publication ownership.

## No-mock real integration certification (required for INT-W1)

- Run the actual DubBridge `apps/api` and `apps/gateway`, with PostgreSQL/Redis and other needed local backing services; record commit IDs and exact command/output.
- Provision an authorized organization, project, review task and limited service principal through a real, auditable setup path. Seeded test data must reside in a real DB; never return canned API JSON.
- Run actual Fenix -> ToolRegistry -> authenticated DubBridge GET and compare IDs/states with DubBridge backend/DB.
- Negative checks: absent/expired JWT 401, missing scope 403, non-member/wrong role 403, wrong org-project association rejection; neither exposure of another tenant nor claimed success.
- Validate `pending`, `approved`, `rejected` interpretations and empty queue using persisted state. In this first slice, no POST decision/publish calls are allowed from Fenix.
- Interrupt and restore the real API to test bounded retry and explicit outage/error; no synthetic fallback, no write operation or replay.
- Verify actual trace identity appears in both runtime evidence streams; otherwise record failed/incomplete correlation certification.
- Preserve sanitized test evidence and security reviewer approval in each repo. No local-only mock test counts as end-to-end proof.

## Authority and non-goals

DubBridge retains all gates from ADR-008 (rights fail-closed), ADR-030 (review/publication), ADR-031 (JWT), org membership, and ADR-044 (P2P). P2P certification P7 and production hosting are **not** prerequisites to a local read-only review-queue E2E. The integration must never claim that a successful GET certifies P2P playback.

No new cloud subscription, hosting purchase, account change, rights mutation, media upload, production launch, or deployment is authorized here.

## Decision and dependencies

**Conditional GO for analysis**, **NO-GO for live integration until S2S identity is verifiably restricted**. Proposed protocol is HTTP/JSON, using existing routes and DTOs. Do not add MCP/A2A or a second approval system to solve a plain read-only API call.

Task ledger: [DubBridge INT-W0 boundary tasks](../tasks/fenix-integration-int-w0-boundary.md).
Implementation of auth, routing, audit or deployment is a **separately scored and approved** INT-W1 change; existing ADRs are not superseded by this draft plan.
