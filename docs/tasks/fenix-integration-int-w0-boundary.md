---
type: TaskList
title: "Tasks: INT-W0 DubBridge-owned authentication and API boundary"
status: superseded
slice: INT-W0
plan: docs/plan/fenix-integration-int-w0-boundary.md
governed_by: [ADR-008, ADR-030, ADR-031]
---

> **SUPERSEDED (2026-10-29):** history of the incorrect Fenix → DubBridge direction. Use [active consumer-ledger](dubbridge-fenix-service-consumption.md) and [active plan](../plan/dubbridge-consumes-fenix-integration.md); historical checkboxes are not project progress.
# INT-W0 — DubBridge-owned task ledger

Scope: document the existing DubBridge API, issuer/scope and org/project authorization, and define exact blockers to a genuine cross-product integration. Branch: `feature/int-w0-fenix-integration-boundary`.

**Owner requirement:** no mock response, fake JWT, or simulated provider may justify a PASS of the real Fenix ↔ DubBridge end-to-end execution. A deliberately seeded **real PostgreSQL** test dataset is valid, provided all services and API/gates are authentic.

| Task | Deliverable | Dependency | Status |
|---|---|---|---|
| DB-INT-W0.T1 | Document actual GET review queue route/DTO | — | SOURCE-INSPECTED |
| DB-INT-W0.T2 | Identify JWT/scope/org/project checks | T1 | SOURCE-INSPECTED |
| DB-INT-W0.T3 | Specify least-privileged service principal; reject default broad token | T2 | DESIGN REQUIRED; not implemented |
| DB-INT-W0.T4 | Define trace propagation and auditable cross-product identity | T3 | DESIGN REQUIRED; not implemented |
| DB-INT-W0.T5 | Define real E2E security test matrix/exit gate | T4 | DEFINED; not executed |

## Findings

- Existing `GET /orgs/{org_id}/projects/{project_id}/review-tasks` is protected by bearer JWT, `workspaces:read`, org membership, minimum `Reviewer` role, project-in-org validation.
- Current `DEFAULT_AUTH_SCOPES` in `crates/auth/src/service.rs` includes write/ingest. A standard user login token is **not** an acceptable S2S read-only credential.
- No independent Fenix-to-DubBridge authorization mapping or verified structured Fenix correlation persistence is evidenced by the reviewed source. Both remain explicit open gaps.
- Auth architecture modifications require their own design/ADR review, RRI, test evidence, and owner checkpoint; this planning document does not amend accepted ADR-031.

## REAL INT-W1 dependencies

1. **DB-INT-W1.A:** scoped service principal and token lifecycle, with real issuer/verifier and organizational authorization. This must be approved and implemented in DubBridge before Fenix executes remotely.
2. **DB-INT-W1.B:** authenticated read endpoint contract/correlation propagation, only if the existing gateway/backend lacks necessary headers/audit trace behavior.
3. **FX-INT-W1.C:** Fenix read-only `dubbridge.review_queue.list` ToolRegistry adapter using the actual DubBridge provider.
4. **JOINT-INT-W1.D:** actual two-runtime test: seeded persisted review tasks, correct authorized results, cross-tenant denial, 401/403/404 behaviors, outage and correlation checks. Tests cannot be certified by mocks.

The work is **not ordered as a single unreviewed monolith**; individual executable leaves require current RRI, path inventory, peer review, local QA and applicable human approval.

## Exit

INT-W0 records a proposed security boundary, not production readiness. **Live integration remains BLOCKED** until restricted S2S credentials exist and real E2E verifies both runtimes. No push/merge to `main` is authorized by this record.

Verification: documentation/source inspection only; no live E2E, local QA, or peer-review PASS is claimed.
