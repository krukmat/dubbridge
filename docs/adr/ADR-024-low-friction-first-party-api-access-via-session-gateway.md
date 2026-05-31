# ADR-024: Low-friction first-party API access via session gateway

- **Status:** Proposed
- **Date:** 2026-05-31
- **Deciders:** DubBridge platform team

## Context

ADR-023 establishes DubBridge as an OAuth 2.0 resource server that validates JWT
access tokens on every protected API request. That remains a sound security
boundary for machine-to-machine integrations and internal services, but it creates
avoidable friction for **first-party interactive clients** such as a DubBridge web
frontend or internal operator console.

The core problem is not JWT validation itself. The friction comes from pushing the
full token lifecycle to every browser/client:

- obtain access tokens directly from the authorization server
- store and refresh them safely
- attach them on every API request
- handle expiry/renewal behavior in UI code

For first-party UX, that shifts auth complexity to the least trusted and least
operationally controlled client surface. At the same time, removing request-level
authentication from the API would weaken the fail-closed identity boundary that
protects `assets.uploader_id` and other governance-sensitive actions (ADR-008,
ADR-023).

We need a model that reduces UX friction **without** downgrading the security
posture of the API itself.

## Decision

- DubBridge will support **two access patterns** for protected API usage:
  - **First-party interactive clients** use a **session gateway / BFF**
    (backend-for-frontend) boundary.
  - **Programmatic clients** (internal services, CLI automation, partners) continue
    to call the API directly with `Authorization: Bearer <JWT>`.
- The **core protected API remains a JWT-validating resource server** per ADR-023.
  This ADR does not remove per-request identity verification from `apps/api`.
- For first-party UX, the browser/client authenticates once with the external
  authorization server, and the session gateway maintains a **server-side session**
  represented to the browser by a hardened cookie:
  - `HttpOnly`
  - `Secure`
  - appropriate `SameSite`
- The session gateway is responsible for:
  - token exchange / refresh with the external authorization server
  - storing tokens away from browser JavaScript
  - calling the protected DubBridge API or translating the session into internal
    API credentials
- The browser/client must **not** be the canonical holder of long-lived access or
  refresh tokens for first-party UX. Avoid storing such tokens in `localStorage`
  or similar script-accessible storage.
- API authorization semantics remain unchanged:
  - mutable upload-ingestion endpoints still require `assets:ingest`
  - asset reads still require `assets:read`
  - uploader identity is still derived from the verified principal, never from
    request payload
- This ADR applies only to **API client access**. It does not change ADR-022's
  separate source-authentication boundary for RTMP/SRT recording ingest.

## Consequences

**Positive**
- Lower friction for first-party browser/UI clients.
- Sensitive token lifecycle logic moves from browser code into a controlled server
  boundary.
- The protected API keeps the fail-closed, per-request verification model from
  ADR-023.
- Programmatic clients keep a standards-based direct API path without being forced
  through a browser-oriented session model.

**Negative / trade-offs**
- Introduces an additional architectural surface: the session gateway / BFF.
- Session management, cookie policy, CSRF protection, and logout semantics now need
  explicit design and operational ownership.
- The team must maintain two client-access patterns instead of exactly one, even
  though they terminate in the same protected API boundary.

## Alternatives considered

- **Keep JWT direct for every client, including first-party browsers** — rejected:
  secure but unnecessarily high-friction for UX; shifts token handling to the
  browser.
- **Replace the protected API with cookie-only server sessions everywhere** —
  rejected: degrades the direct integration story for machine-to-machine and
  partner clients; couples the core API too tightly to browser/session semantics.
- **Shared API keys for first-party clients** — rejected: too coarse for the
  principal and scope model required by ADR-023.
- **Drop per-request API auth after login** — rejected: weakens the
  governance-critical fail-closed identity boundary.

## Related

- ADR-023 (API client authentication and principal propagation) — retained as the
  core protected API boundary.
- ADR-008 (rights ledger fail-closed precondition) — uploader identity remains
  trustworthy only if request-level identity is preserved.
- ADR-018 (traceable governance events) — the verified subject remains the
  auditable actor.
- Follow-up expected: a concrete design/task slice for session gateway/BFF
  implementation, cookie policy, CSRF posture, and first-party client routing.
