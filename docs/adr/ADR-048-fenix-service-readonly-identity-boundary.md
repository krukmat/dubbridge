---
type: ADR
title: "ADR-048 — Fenix restricted service identity for DubBridge review queue reads"
status: withdrawn
date: 2026-10-28
governed_by: [ADR-008, ADR-030, ADR-031]
---

> **WITHDRAWN BEFORE ACCEPTANCE (2026-10-29):** Product owner confirmed DubBridge is the consumer and Fenix the agent/workflow service provider. This proposal describes the opposite trust direction; **do not implement or accept this ADR**. A new Fenix-owned S2S identity/security ADR must be designed for DubBridge → Fenix. See [canonical plan](../plan/dubbridge-consumes-fenix-integration.md).
# ADR-048 — Fenix restricted service identity for review queue reads

- **Status:** Withdrawn before acceptance (2026-10-29); never implemented.
- **Date:** 2026-10-28
- **Related:** ADR-008 (rights/attribution), ADR-027 (org membership), ADR-030 (review decisions/publication), ADR-031 (mobile JWT), INT-W1 T2/T3.

## Context

Fenix needs to read persisted DubBridge review tasks from its governed ToolRegistry. DubBridge's current endpoint `GET /orgs/{org_id}/projects/{project_id}/review-tasks` accepts a DubBridge-issued JWT with `workspaces:read`, reviewer membership and project-to-org checks. The default human JWT includes `assets:ingest` and `workspaces:write`. Reusing it as a service credential would permit unintended operations. A Fenix JWT must never be treated as a DubBridge credential.

## Proposed decision (not yet accepted)

1. Add **dedicated read-only S2S boundary**: `GET /integrations/fenix/orgs/{org_id}/projects/{project_id}/review-tasks`. Only it accepts the new integration credential. Gateway relay path is `/api/integrations/fenix/... `. It calls the **existing** PostgreSQL review repository, never a parallel queue.
2. A dedicated DubBridge identity is stored durably with single allowed organization/project, active/revoked state, credential version/JTI, and operator attribution. Validate project-in-org on enrollment and every read.
3. Use a separately keyed JWT verification domain for service tokens. Require bounded `exp`/`nbf`, verified `iss`/`aud`, service-only `typ` or equivalent type claim, `sub`, `jti`/version, and exactly `integrations:review_queue:read` capability. A normal mobile JWT and a broad `workspaces:read` token do not authorize the S2S endpoint. **No signing keys or HMAC secrets are exported to Fenix**.
4. Service identity may be provisioned, rotated and revoked only by an authenticated operator process or protected management API (separate design/approval). No public self-registration, no direct Fenix access to DubBridge DB, and no mutable review endpoints exposed through this integration.
5. Continue to enforce DubBridge's existing rights, decision and publication gates. A successful Fenix governance check never substitutes for domain authorization. The existing human JWT/mobile routes keep their current behavior.
6. Treat `X-Fenix-Trace-ID` and `X-Fenix-Execution-ID` as untrusted audit correlation hints; validate and sanitize them, preserving actor identity from authenticated service credentials, not request headers.
7. Scope is limited to one explicitly configured org/project per Fenix deployment in INT-W1. Multi-project tenancy must be designed separately rather than implicitly admitting caller-selected projects.

## Rejected alternatives

- **Reusing user login JWT:** unnecessarily broad privileges, human/service identity conflation.
- **Reusing the normal read endpoint with only `workspaces:read`:** the same scope also opens other read surfaces, so cannot establish the required exact-capability privilege boundary.
- **Sharing DubBridge's HS256 signing secret with Fenix:** Fenix would be able to impersonate any DubBridge principal.
- **Fake/mock HTTP provider for E2E:** does not demonstrate integration or database authorization.
- **Direct shared database or adding MCP/A2A now:** enlarges the trust and operational surface without benefit for one GET.

## Consequences and required gates

Adds a migration, service provisioning, verifier, dedicated route and audit correlation in DubBridge. Fenix owns a bounded HTTP adapter, ToolRegistry governance and Blackboard projection only. Before accepting this ADR, security reviewer must explicitly assess key separation, revocation consistency, subject/org/project binding, normal mobile isolation, and detailed deny matrix.

**Real E2E acceptance:** actual Fenix ToolRegistry/Blackboard -> DubBridge gateway/API -> PostgreSQL with a genuinely issued restricted credential; wrong-org/project, expired/revoked, human token and mutation attempts rejected, trace captured in both systems. No mocked issuer, response or provider can constitute PASS.

## Status

Proposed

This ADR is intentionally **not accepted**. T2 implementation and closure cannot claim security certification without the repository's RRI/peer gates, local Rust QA and real E2E evidence.
