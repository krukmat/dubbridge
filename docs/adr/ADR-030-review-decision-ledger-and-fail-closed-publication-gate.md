# ADR-030: Review-decision ledger and fail-closed publication gate

- **Status:** Accepted
- **Date:** 2026-06-13
- **Deciders:** DubBridge platform team
- **Closes:** X23 / X-S-160-1

## Context

DubBridge's product promise is that no asset reaches an audience without passing
rights, quality, and human review. Earlier slices established adjacent governance
patterns but not the review/publication contract itself:

- ADR-008 requires fail-closed preconditions before governance-sensitive work proceeds.
- ADR-018 requires durable, traceable audit rows for governance decisions.
- ADR-027 establishes org-scoped roles and reviewer authorization.

S-160 introduces a human review and publication workspace, but before schema, API,
and mobile implementation work can proceed, the platform needs an explicit contract
for four questions:

1. What is the lifecycle of a reviewable unit?
2. Are review decisions mutable or append-only?
3. When is publication allowed?
4. How does this work before S-140/S-150 produce real derived artifacts?

This ADR records that contract so S-160-T1/T2 can implement it without inventing
policy locally.

## Decision

### Review tasks are the assignment anchor

A `review_tasks` row represents one reviewable unit for one `(asset, target)`
combination. It exists to anchor assignment, queue membership, and downstream
publication gating. It does not erase or replace the underlying artifact lineage.

Review tasks are scoped to the caller's organization/project context from S-100 and
S-105. A reviewer sees only tasks assigned to their org's projects.

### `review_decisions` is append-only

Reviewer verdicts are recorded in `review_decisions` as immutable rows:

- rows are never updated or deleted
- a new decision supersedes the prior one logically, but does not rewrite history
- the current review state is derived from the latest decision row

This mirrors the append-only posture already used for `rights_records` and
`voice_consents`. Unknown stored verdict values are rejected at decode time and do
not fall back to a permissive default.

### Review state is derived, fail-closed

The current state of a review task is derived from the latest decision row:

- no decision yet -> `pending`
- latest verdict `approved` -> `approved`
- latest verdict `rejected` -> `rejected`

If the system cannot derive a valid known state, it must reject the operation
fail-closed rather than infer approval.

### Publication is blocked unless the latest governing decision is approved

Publication is a fail-closed governance gate:

> A publication row may be created only when the latest governing review decision
> for the target review task is `approved`.

If the latest decision is missing, rejected, malformed, or otherwise not valid for
publication, the publish attempt is refused. UI visibility is not authoritative; the
backend gate owns the rule.

### Reviewer authorization is org-scoped and role-gated

Only org members with the `reviewer` role or higher, as defined by ADR-027, may act
on review tasks within their org scope. A caller must pass both:

- the existing API authentication/scope checks from ADR-023
- the org membership and minimum-role guard from ADR-027

This gate narrows access; it never widens it.

### Every review decision and publication attempt is auditable

Every governance-significant action in this domain emits a durable audit row:

- review approved
- review rejected
- publication succeeded
- publication refused because approval is absent

Audit rows carry the verified acting subject and the relevant task/asset/target
references. The audit obligation applies whether the operation succeeds or is
rejected by policy.

### The contract operates ahead of real subtitle/dub producers

Until S-140/S-150 produce real derived artifacts, S-160 operates against deterministic
fixtures and existing artifact lineage records. When those producer slices land, they
must enqueue review tasks and call the same publication gate rather than introduce a
parallel path.

## Consequences

**Positive**
- Human review becomes a durable, testable governance boundary rather than UI-only behavior.
- Publication is protected by the same fail-closed philosophy already used for rights and consent.
- Append-only decisions preserve a complete reviewer history.
- S-160 can be built now against fixtures without weakening the future runtime contract.

**Negative / trade-offs**
- Every publication attempt requires an additional review-state lookup before success.
- Review state is derived rather than stored as a single mutable source of truth, which adds query and repo complexity.
- Re-review after a rejection is modeled as another decision row, not a state overwrite; callers must understand that history remains visible.

## Alternatives considered

- **Mutable review state on the task row** — rejected: loses auditability and obscures reviewer history.
- **Allow publication based on UI state alone** — rejected: UI hiding is not an authorization or governance boundary.
- **Create publication first and reconcile review asynchronously** — rejected: creates a fail-open window where unreviewed content could be published.
- **Wait for S-140/S-150 before defining the contract** — rejected: would delay governance design and make downstream producer behavior the accidental source of truth.

## Related

- ADR-008 (rights ledger fail-closed precondition) — publication adopts the same fail-closed posture.
- ADR-018 (structured observability) — review and publication decisions are auditable events.
- ADR-023 (API client authentication and principal propagation) — verified identity remains the outer auth boundary.
- ADR-027 (organization membership authorization) — reviewer assignment and action scope are role-gated by org membership.
- Implemented by: `docs/tasks/s-160-review-publication-workspace.md` S-160-T0b; consumed by S-160-T1/T2/T3/T6/T7.
