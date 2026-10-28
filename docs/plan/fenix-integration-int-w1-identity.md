---
type: Plan
title: "INT-W1 — DubBridge service identity for real Fenix review queue integration"
status: proposed
slice: INT-W1
governed_by: [ADR-008, ADR-030, ADR-031]
---

# DB-INT-W1.T2 — Dedicated Fenix read-only service identity

**Owner:** DubBridge. **Branch:** `feature/int-w1-fenix-service-identity`. **Date:** 2026-10-28. **State:** design only, not security-approved or implemented.

## Outcome and user constraints

Permit the genuine Fenix ToolRegistry to read a real DubBridge project review queue, without granting write/ingest/publish rights, without sharing JWT signing keys, and without mocking either application or database. Keep existing mobile JWT behavior, protected review mutations, rights records and P2P unchanged.

GitHub Actions suspension is **reported completed by the owner**; repository actions-configuration status is not readable through this connection. The last API run inventory showed no new runs, but previously queued DubBridge `push-review` jobs remain. Do not re-enable Actions until instructed.

## Current code findings

- `crates/auth/src/service.rs` issues default user JWT with `assets:read assets:ingest workspaces:read workspaces:write`; a user login token is unsuitable as a least-privileged integration secret.
- `crates/auth/src/issuer.rs` issues HS256 JWT with `sub,workspace_id,iat,nbf,exp,scope`.
- `crates/auth/src/verifier.rs` projects JWT subject and scopes to `AuthenticatedPrincipal`; `workspace_id` is not preserved in the generic principal.
- `apps/api/src/routes/review.rs` already implements `GET /orgs/{org_id}/projects/{project_id}/review-tasks`, guarded by `workspaces:read`, organization Reviewer membership and project-in-org checks.
- `infra/migrations/0010_create_organizations.sql` stores `org_members(org_id,subject_id,role)`, without an account FK; `0018_create_user_account.sql` stores human login accounts.
- These checks do not, by themselves, provide a dedicated, revocable **service account with a single allowed org/project**.
- Fenix owns the ToolRegistry, its audit and agent-safe projection; DubBridge owns the actual queue, issuer and authorization.

## Security design to freeze before code

### 1. Dedicated service entrypoint

Propose `GET /integrations/fenix/orgs/{org_id}/projects/{project_id}/review-tasks`. The path is **not yet implemented**. It calls the existing production review repository without introducing another queue implementation; it never exposes POST review/publish/rights operations.

Use a **separate verifier/key domain** for integration JWTs so that a mobile user token cannot be exchanged for an integration one. Token design: audience `dubbridge:fenix-review`, type `service`, scope `integrations:review_queue:read`, bound principal ID, token ID/version and bounded expiry. These are **proposed** claims, not fields of the existing mobile JWT or `AuthenticatedPrincipal`. The service scope replaces broad `workspaces:read` for this *dedicated* route; existing GET keeps its current authorization unchanged. Do not reuse the mobile signing secret or distribute any DubBridge signing key to Fenix.

### 2. Durable least-privileged identity

Add a DubBridge-owned `integration_principals` table (exact migration requires review) with immutable principal UUID, identity kind `fenix-review-reader`, allowed org and project UUIDs, active/revoked state, credential version or JWT ID, issue/expiry metadata and audit attribution. Validate org-project relationship at provisioning and at every read. Keep no plaintext bearer in DB.

Only an authenticated DubBridge operator may provision/revoke/rotate through an explicitly approved maintenance interface. **No unauthenticated service-registration, user-login delegation, or general-purpose self-issuance endpoint.** Actual operator interface and approval trail require security review.

### 3. Every-request verification

In the S2S-only route: validate signature/algorithm, dedicated issuer/audience/type, `exp`/`nbf`, exact scope, active DB principal, token version/revocation and exact allowed org/project. Validate project still belongs to org. Reject default mobile tokens, caller-selected arbitrary tenant IDs, and any token with write/ingest/publication authority. This authorization is independent of the user-facing org-member roles; whether a specific `org_members` Reviewer row is additionally required needs a documented decision, not a silent bypass.

Error semantics: missing/bad/expired bearer → 401; insufficient authorization or mismatched project → 403/404 without revealing another tenant; database error → fail closed. Do not retry authorization failures. No token, password, secret, personal identifier or raw review payload in logs.

### 4. Trace and evidence

Treat `X-Fenix-Trace-ID` / `X-Fenix-Execution-ID` as untrusted correlation strings only. Validate length and syntax, emit redacted audit data at both ends, never infer actor authority from headers. Existing DubBridge JWT verifier does not prove that these headers are durably captured, so trace parity remains an explicit test gate.

### 5. Runtime restrictions

No shared JWT secret, no cross-product DB access, no URL injection, no media/R2/WordPress provisioning, no paid cloud deployment, no service fallback with synthetic responses, and no mutation rights.

## Implementation leaves (separately reviewable)

| Task | Work | Depends on | Status |
|---|---|---|---|
| DB-INT-W1.T2a | Security ADR, RRI, contract and reviewer readiness | W1.T1 | Proposed |
| DB-INT-W1.T2b | SQL migration + data access: service principal/rotation/revocation | T2a | Not started |
| DB-INT-W1.T2c | Operator-only provisioning/short-lived service token | T2b | Not started |
| DB-INT-W1.T2d | Restricted JWT verifier + dedicated read-only endpoint | T2c | Not started |
| DB-INT-W1.T2e | Real PostgreSQL/API negative security checks + independent review | T2d | Not started |

## Real verification requirements

- Launch actual API/gateway with actual PostgreSQL migrations and bounded test credentials; preserve exact commit/commands.
- Issue real DubBridge service credential; query a persisted review task from Fenix with the same org/project and compare returned IDs/status to DB.
- Verify expired/revoked/wrong-audience/default-mobile tokens are rejected.
- Verify cross-project/cross-org token reuse rejected and every existing write, ingest, review decision and publish endpoint is inaccessible with the service token.
- Capture a genuine trace/execution correlation and negative outage/timeout evidence, with no mock server and no synthetic queue data.
- Do not label standalone compilation/unit tests or an unauthenticated GET as integration PASS.

## Review gate

The repo's RRI calculation, independent task-analysis/security review, owner checkpoint and local Rust/DB QA must run before runtime modifications are pushed or declared done. No such review or real E2E has yet run in this connector-only session.

Related documents: [DubBridge INT-W0](fenix-integration-int-w0-boundary.md) and [Fenix INT-W0](https://github.com/krukmat/fenix/blob/feature/int-w1-dubbridge-readonly/docs/plans/fenix-dubbridge-int-w0-integration.md).
