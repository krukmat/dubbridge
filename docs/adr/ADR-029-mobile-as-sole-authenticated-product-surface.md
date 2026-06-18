---
type: ADR
title: "ADR-029: Mobile as the sole authenticated product surface"
status: Accepted
---

# ADR-029: Mobile as the sole authenticated product surface

- **Status:** Accepted (transport amended by ADR-031)
- **Date:** 2026-06-13; 2026-06-17 (transport amended, S-200-T0)
- **Deciders:** DubBridge platform team
- **Closes:** S-105 surface-consolidation decision

> **Amended by [ADR-031](ADR-031-mobile-jwt-credential-auth-fenix-parity.md)
> (2026-06-17, S-200-T0).** This ADR's product-surface decision is unchanged: mobile
> remains the sole authenticated first-party UI, and UI visibility is never an
> authorization boundary. What changes is the **transport**: ADR-031 supersedes
> ADR-024, so mobile carries a backend-issued bearer JWT instead of an opaque
> gateway session. Read every reference to "ADR-024 session gateway / opaque
> session" below as superseded by ADR-031.

## Context

ADR-024 establishes the first-party session gateway and the opaque-session contract
used by interactive DubBridge clients. That ADR deliberately separates the
authentication/access pattern from any one concrete UI surface: the gateway can serve
multiple first-party transports without weakening the API trust boundary.

By S-100, DubBridge had both:

- a historical authenticated web prototype for workspace surfaces
- an operational React Native mobile client already carrying the main first-party
  session, asset, and project flows

The S-105 review found that keeping both authenticated surfaces active increased
maintenance without providing matching product value:

- the web console was incomplete and duplicated client/tooling overhead
- the mobile app already owned the operational authenticated workflows
- downstream slices (S-110 compliance and S-160 review/publication) would otherwise
  pay the cost of building and certifying two authenticated UIs

We need a durable architectural record for the product-surface decision itself, so
that it does not remain implicit in roadmap prose or get conflated with ADR-024's
gateway/session decision.

## Decision

### Mobile is the only operational authenticated first-party UI

DubBridge adopts `mobile/` as the **sole operational authenticated product surface**.
Authenticated workspace, compliance, consent, review, and publication user flows are
implemented and certified in mobile unless a later ADR explicitly broadens the
surface set again.

### The authenticated web console is retired

The authenticated `web/` console is not an operational surface after S-105. Its
historical implementation remains part of the project record, but the runtime app,
tests, and supporting frontend tooling are removed once mobile parity is certified.

### ADR-024 remains in force

This ADR does **not** change the session-gateway contract from ADR-024:

- first-party interactive auth still terminates at `apps/gateway`
- clients still carry only opaque session transports
- `apps/api` remains the JWT-validating protected-resource boundary

ADR-024 answers **how** first-party clients authenticate. This ADR answers **which**
authenticated product surface DubBridge operates.

### Future public web/player work is a separate decision

A future public website, playback surface, or distribution portal is outside the
scope of this ADR. Such work must be planned independently and must not implicitly
revive the retired authenticated web-console codepath.

### UI visibility is never an authorization boundary

Consolidating to mobile does not move authorization into the UI. Ownership, role
checks, consent gates, and publication gates remain backend-enforced and fail-closed.
Mobile may hide unavailable actions, but the API remains authoritative.

## Consequences

**Positive**
- One authenticated UI reduces maintenance, CI surface, documentation drift, and
  duplicate E2E work.
- Product slices S-110 and S-160 can concentrate verification on one executable UI
  surface plus backend authorization/gate evidence.
- The gateway/session architecture stays reusable without forcing the product to
  keep multiple first-party UIs alive.
- The historical web prototype is preserved as implementation history rather than as
  an accidental ongoing commitment.

**Negative / trade-offs**
- Reviewer/operator workflows are unavailable in a browser until a future decision
  intentionally restores a first-party web surface.
- Mobile becomes the only place where authenticated UX regressions are surfaced at
  the product layer, so its tests and Maestro coverage become more important.
- Documentation must clearly distinguish between "gateway can support multiple
  transports" and "the product currently operates only one authenticated UI."

## Alternatives considered

- **Keep web and mobile both operational** — rejected: duplicated maintenance and
  verification cost without enough product value for the current scope.
- **Amend ADR-024 instead of creating a new ADR** — rejected: ADR-024 governs the
  access/session pattern, not the product-surface portfolio. Combining both would
  blur two separate decisions.
- **Retain the web console as an unsupported fallback** — rejected: an "inactive but
  still present" authenticated UI still carries maintenance, drift, and ambiguity.

## Related

- ADR-024 (low-friction first-party API access via session gateway) — retained;
  this ADR narrows the active product surface, not the gateway contract.
- ADR-027 (organization membership authorization) — backend org/role checks remain
  authoritative regardless of UI surface.
- ADR-028 (voice-consent ledger) — compliance/consent UX is certified in mobile,
  while the underlying gate remains backend-enforced.
- Implemented by: `docs/plan/s-105-mobile-workspace-parity.md`,
  `docs/tasks/s-105-mobile-workspace-parity.md`, and the S-105-delivered mobile
  workspace/compliance surfaces plus authenticated `web/` removal.
