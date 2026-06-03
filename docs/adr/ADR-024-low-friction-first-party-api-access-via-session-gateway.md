# ADR-024: Low-friction first-party API access via session gateway

- **Status:** Accepted
- **Date:** 2026-05-31 (proposed); 2026-06-03 (accepted)
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

## Concrete decisions (accepted 2026-06-03, P1 T0)

These decisions resolve the implementation choices the proposed text left open and
are binding for slice **P1** (`docs/plan/p1-session-gateway-bff.md`) and its
first-party consumers — the web app and slice **P3** (mobile, React Native + Expo).

### Login flow
- The gateway is a **confidential OAuth 2.0 client** using **Authorization Code with
  PKCE** (`S256` `code_challenge`) plus an unguessable, single-use `state`. PKCE +
  `state` defend the login leg against code interception and login-CSRF.

### Session cookie
- The browser session is represented by a single cookie carrying an **opaque,
  high-entropy session id** — never an access or refresh token.
- Cookie attributes:
  - `HttpOnly` — not readable by client JavaScript.
  - `Secure` — only sent over TLS.
  - `SameSite=Lax` — the default posture. It permits top-level navigations (needed
    for the OAuth redirect return) while blocking cross-site subrequests. A stricter
    `SameSite=Strict` may be adopted later if the redirect-return UX allows it; this
    is an operational tightening, not a contract change.
  - `Path=/` and a host-scoped cookie (no broad `Domain` widening).

### CSRF posture
- Because `SameSite=Lax` still allows top-level GET navigations, mutating gateway
  routes use **defense in depth**: an **`Origin`/`Referer` check** plus a
  **double-submit CSRF token** (a non-`HttpOnly` token cookie echoed in a request
  header by first-party client code, compared server-side). The login `state`
  parameter covers CSRF on the OAuth callback specifically.

### Session store
- Sessions are stored **server-side only**. The store is defined behind a trait with
  two implementations: an **in-memory** store for deterministic tests and a
  **Redis-backed** store for runtime (Redis is already reserved for coordination in
  the architecture). Stored material: the OAuth `TokenSet` (access + refresh +
  expiry), the verified subject, the CSRF token, and timestamps.

### Session TTL / idle policy
- Each session has an **absolute lifetime** (default 8 hours) and an **idle timeout**
  (default 30 minutes of inactivity), both env-configurable per ADR-026. Reaching
  either bound invalidates the session; the next request resolves to "no session"
  and the client must re-authenticate. The access token inside the session is
  refreshed transparently against the authorization server while the session itself
  remains valid.

### Transport-agnostic session seam (mobile / P3)
- A **native mobile client does not share the browser cookie jar**, so the canonical
  session abstraction is an **opaque session id**, of which the hardened cookie is
  one transport. The mobile client (P3) authenticates through the **same gateway**
  using the device system browser (`expo-auth-session`) and carries the same opaque
  session reference, stored on-device in secure storage
  (`expo-secure-store` / Keychain / Keystore) — **never** an access or refresh token.
- This means web and mobile are two transports of one session contract terminating
  in the **same** gateway trust boundary. P3 must not introduce a parallel auth path
  or hold long-lived tokens on the device.

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
- Implemented by (planned): slice **P1** — `docs/plan/p1-session-gateway-bff.md`,
  `docs/tasks/p1-session-gateway-bff.md` (cookie policy, CSRF posture, session store,
  TTL, and first-party client routing decided here in P1 T0).
- Consumed by (planned): slice **P3** — first-party mobile client (React Native +
  Expo), `docs/plan/p3-mobile-client.md`, `docs/tasks/p3-mobile-client.md`. Mobile
  is the second transport of the same opaque-session contract.
