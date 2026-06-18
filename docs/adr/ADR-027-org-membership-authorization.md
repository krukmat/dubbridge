---
type: ADR
title: "ADR-027: Organization membership authorization"
status: Accepted
---

# ADR-027: Organization membership authorization

- **Status:** Accepted
- **Date:** 2026-06-12
- **Deciders:** DubBridge platform team
- **Closes:** X22 / X-S-100-1

## Context

S-000 through S-060 treat every authenticated principal as a flat entity: an
`AuthenticatedPrincipal` carries only `subject_id` (a verified UUID from the JWT
`sub` claim) and a set of OAuth scopes. Assets are owned exclusively by their
uploader; there is no concept of an organization, a team member, or a role.

S-100 introduces the **collaborative product layer**: organizations group members
under role-based access control, and projects group uploader-owned assets within
an org. Routes that operate on org resources must enforce two distinct authorization
conditions:

1. The caller holds the required OAuth scope (existing ADR-023 check).
2. The caller is a **member** of the target organization with a **sufficient role**
   (new condition introduced here).

Neither condition alone is sufficient. Scope without membership would allow any
authenticated client to access any org. Membership without scope would bypass the
existing API authorization boundary.

This ADR records the decision for how those two conditions are combined and enforced
in a fail-closed, auditable way.

## Decision

### Organization as the tenancy boundary

An `organizations` table establishes orgs as the unit of multi-tenancy. Assets remain
uploader-owned (`assets.uploader_id` is set from `AuthenticatedPrincipal.subject_id`
and is never changed — ADR-023). Projects *link* assets through a `project_assets`
join table; they do not reassign ownership.

### Role enum — strict decode, fail-closed

Org membership is recorded in `org_members(org_id, subject_id, role)`. The `role`
column is a database enum with exactly five values:

```
Owner > Admin > Editor > Reviewer > Viewer
```

- **Owner**: full control including org deletion and member management.
- **Admin**: member management and all project/asset operations; cannot delete the org.
- **Editor**: create and edit projects; link/unlink assets; set target languages.
- **Reviewer**: read-only access to all org resources; cannot modify.
- **Viewer**: read-only access scoped to what the org owner has made visible; may be
  further restricted in later slices.

Unknown role strings are **rejected at decode time** — they must never produce a
permissive default. A failed decode is treated as a non-member (fail-closed, the same
posture as the rights gate in ADR-008).

### OrgMemberPrincipal — wraps AuthenticatedPrincipal

`crates/auth/src/membership.rs` introduces `OrgMemberPrincipal`:

```rust
pub struct OrgMemberPrincipal {
    pub principal: AuthenticatedPrincipal,  // ADR-023 verified identity
    pub org_id: Uuid,
    pub role: OrgRole,
}
```

`OrgMemberPrincipal` is not produced from the JWT. It is resolved from the database
at request time: `workspace_repo::get_membership(pool, subject_id, org_id)` returns
the role if a row exists, or `None` if the caller is not a member.

No org-scoped claim is added to the JWT or the access token. Membership is a
**runtime state** (it changes when members are added, removed, or promoted) and must
not be cached in a signed token with a longer TTL.

### Axum middleware: require_org_member(min_role)

`apps/api/src/middleware/org_scope.rs` provides an extractor that:

1. Extracts `org_id` from the request path.
2. Reads `AuthenticatedPrincipal` from the existing ADR-023 extractor (which has
   already verified the JWT and scope).
3. Calls `workspace_repo::get_membership` for `(subject_id, org_id)`.
4. If no membership row exists → `403 Forbidden`, fail-closed, no data leaked.
5. If the resolved role is below `min_role` → `403 Forbidden`, fail-closed.
6. Otherwise injects `OrgMemberPrincipal` into the Axum extension map for the handler.

The existing scope check from ADR-023 (`require_scope`) runs **before** the org guard.
The org guard is **additive**: it narrows access, never widens it. A caller who passes
the org guard has already passed the JWT + scope check.

### Audit obligation

Every governance event on org resources — member added, member removed, role changed,
project created, project deleted — must write a durable row to `audit_events` (ADR-018).
The audit row carries `subject_id`, `org_id`, `event_type`, and a structured payload.
Membership changes use the same append-only audit path already established by H1.

## Consequences

**Positive**
- Fail-closed by default: a principal who is not a member of an org gets `403` with
  no data leaked, matching the posture of the rights gate (ADR-008).
- Org routes gain a second, independent authorization layer without removing or
  weakening the existing ADR-023 scope check.
- No JWT change required: membership is resolved at runtime, so adding or revoking
  a member takes effect immediately without waiting for token expiry.
- Strict role decode prevents unknown future values from granting unintended access.
- Governance events remain auditable and append-only (ADR-018).

**Negative / trade-offs**
- Each org-scoped request pays one extra DB round-trip (membership lookup). This is
  acceptable at the current scale and avoids the complexity and stale-data risk of
  token-embedded claims.
- Role hierarchy is ordinal (`Owner > Admin > Editor > Reviewer > Viewer`). If a
  future slice needs non-hierarchical permissions (e.g. a role that can do A but not
  B), the model must be extended. Recorded as an open follow-up (X-S-100-3).
- `Editor` is introduced here for completeness but S-100 only enforces
  `owner/admin` write vs `reviewer/viewer` read at the API layer. The full
  `Editor` role distinction becomes load-bearing in S-160 when review/publication
  actions are added.

## Alternatives considered

- **Embed org membership in the JWT** — rejected: membership changes would not take
  effect until the current token expires; immediate revocation would be impossible
  without token invalidation infrastructure not present in the platform.
- **Single-level "is-member" flag without roles** — rejected: S-110 (consent) and
  S-160 (review/publication) require role distinctions; introducing roles now avoids
  a breaking schema change later.
- **Separate authorization service / policy engine (OPA, Casbin)** — deferred: adds
  significant operational complexity. The in-process DB lookup is sufficient at the
  current scale and team size.
- **Org claim in a custom JWT claim set** — rejected: requires coordination with the
  external authorization server and constrains the org model to what fits in a signed
  claim; runtime resolution is simpler and more correct.

## Related

- ADR-023 (API client authentication and principal propagation) — the ADR-023 scope
  check is the first authorization layer; this ADR adds the second.
- ADR-008 (rights ledger fail-closed precondition) — the fail-closed membership
  posture mirrors the rights gate pattern.
- ADR-018 (structured observability) — audit obligation for membership/governance events.
- ADR-006 (PostgreSQL metadata) — membership and audit rows require transactional storage.
- X-S-100-1 (this ADR closes the open follow-up recorded in plan §D2).
- X-S-100-3: future follow-up if non-hierarchical role permissions are needed.
- Implemented by: `docs/tasks/s-100-collaborative-workspace.md` S-100-T1 (schema),
  S-100-T2 (membership guard), S-100-T3 (workspace API routes).
