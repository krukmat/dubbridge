---
type: ADR
title: "ADR-031: Mobile credential login with backend-issued JWT (FenixCRM parity)"
status: Accepted
supersedes: [ADR-023, ADR-024]
---

# ADR-031: Mobile credential login with backend-issued JWT (FenixCRM parity)

- **Status:** Accepted
- **Date:** 2026-06-17 (proposed); 2026-06-17 (accepted, S-200-T0)
- **Deciders:** DubBridge platform team
- **Supersedes:** ADR-023, ADR-024
- **Amends:** ADR-029 (transport only; mobile remains the sole surface)
- **Opens:** S-200 (mobile credential JWT auth)
- **Implemented by:** S-200-T0 (ADR acceptance, 2026-06-17), T1a/T1b-i/T1b-ii/T1c-i/T1c-ii (HS256 issuer + alg pinning, 2026-06-17), T2a/T2b/T2c (user_account migration + repo, 2026-06-18), T3a/T3b/T3c (bcrypt + AuthService, 2026-06-18), T4a/T4b/T4c/T4d/T4e/T4f (apps/api auth handlers, 2026-06-18), T5a/T5b/T5c/T5d/T5e (gateway relay, 2026-06-18), T6a/T6b (mobile bearer auth, 2026-06-18), T7 (BDD + Maestro + docs sync, 2026-06-18)

> **Accepted (S-200-T0, 2026-06-17).** This ADR **inverts** the authentication
> architecture established by ADR-023 and ADR-024 and amends ADR-029's transport.
> ADR-023 and ADR-024 are now `Superseded by ADR-031`. Implementation of the
> inversion proceeds through the S-200 task ledger; each code task remains
> independently gated (RRI > 25 → explicit approval). Acceptance is the point at
> which the platform takes on the security regressions recorded in §Risk analysis.

## Context

`/Users/matias/fenix/docs/mobile-auth-flow-reference.md` documents the FenixCRM
mobile authentication flow as a portable reference. Its shape is:

- React Native + Expo mobile → Express BFF (**transparent relay**) → Go backend.
- The **backend** validates `email`/`password` (bcrypt cost 12) and **issues its own
  HS256 JWT** carrying `user_id` and `workspace_id` (default 24 h expiry).
- The **device stores the JWT** in `expo-secure-store` (`fenixcrm_token`); an Axios
  request interceptor injects `Authorization: Bearer <jwt>`; a `401` response
  interceptor forces logout. No refresh tokens in the MVP.

The platform directive (2026-06-17) is to adapt DubBridge's mobile authentication so
it works **like FenixCRM, at full fidelity** — including the long-lived bearer token
held on the device and the backend issuing its own tokens.

This is the deliberate opposite of DubBridge's current, triple-anchored design:

- **ADR-023** makes `apps/api` an OAuth 2.0 *resource server only*. It pins
  verification to **RS256** (asymmetric; private signing key never in DubBridge) and
  explicitly **rejected** "implement login and token issuance inside DubBridge".
  Evidence: `crates/auth/src/verifier.rs` pins `Algorithm::RS256` and a unit test
  `verify_rejects_algorithm_substitution` rejects an HS256-signed token.
- **ADR-024** makes first-party clients terminate at `apps/gateway`, which holds a
  **server-side session** and exposes only an **opaque `session_ref`** to clients;
  the device must **never** hold an access or refresh token. Gateway owns session
  renewal/rotation via `X-Dubbridge-Session`.
- **ADR-029** consolidates the product onto `mobile/` as the sole authenticated UI
  but keeps the ADR-024 gateway/session transport.

The current mobile code actively *enforces* the no-token-on-device invariant: the
`isJwtLike()` guard in `mobile/src/auth/session.ts`, `mobile/src/api/client.ts`, and
`mobile/src/auth/AuthProvider.tsx` rejects persisting anything shaped like a JWT.

Adopting FenixCRM parity therefore is not an incremental change; it is an
authentication re-architecture that removes those guards, replaces the OAuth/PKCE
gateway session with backend-issued bearer tokens, and reduces the gateway to a
relay. The initiative-level RRI is **109 (Excessive, > 100)**, whose gate is
"architecture/design work must happen first" — hence this ADR plus the S-200 risk
analysis and decomposition precede any implementation.

## Decision

DubBridge adopts a **first-party credential login with a backend-issued HS256 JWT**,
matching the FenixCRM reference flow.

### 1. The backend owns login, registration, and token issuance

`apps/api` (via `crates/auth`) gains a credential-auth boundary:

- `POST /auth/login` (public): validate `email` + `password` against a stored
  account; on success issue a signed JWT and return `{ token, userId, workspaceId }`.
- `POST /auth/register` (public): create the workspace + account atomically, hash the
  password with **bcrypt cost 12**, then issue a token. `201 Created`; `409 Conflict`
  on duplicate email. Minimum password length 12.
- Passwords are stored only as bcrypt hashes; plaintext is never persisted or logged.
- Login failure returns a **generic** `ErrInvalidCredentials`-equivalent (`401`) for
  both unknown email and wrong password — no email enumeration.

This **reverses ADR-023's** "DubBridge does not implement login, password storage,
token issuance, or refresh-token flows" decision.

### 2. Token format: HS256, backend-signed

- Algorithm **HS256** (symmetric), signed with `DUBBRIDGE_JWT_SECRET`.
- The secret is an **injected env secret** (ADR-026); the process **fails closed at
  startup** if it is unset in a non-local environment.
- Claims: `sub` (account UUID, the principal / `assets.uploader_id` source),
  `workspace_id`, `iat`, `nbf`, `exp` (`iat + DUBBRIDGE_JWT_EXPIRY_HOURS`, default
  24 h).
- **Algorithm pinning at parse time**: any token whose header `alg` is not `HS256` is
  rejected before signature checks (blocks algorithm-substitution attacks).
- No refresh tokens in v1. A `401` forces re-login.

This **replaces ADR-023's** RS256 asymmetric verification with symmetric
issue-and-verify inside DubBridge.

### 3. The gateway becomes a transparent relay (BFF)

`apps/gateway` stops being a stateful session boundary for the mobile transport:

- It forwards `POST /auth/login` and `POST /auth/register` bodies to `apps/api` and
  relays the status + JSON unchanged.
- It forwards authenticated `/api/*` requests, passing the client's
  `Authorization: Bearer <jwt>` through to `apps/api`.
- It no longer mints, stores, rotates, or owns sessions for the mobile transport, and
  no longer issues `X-Dubbridge-Session`.

This **reverses ADR-024's** server-side session / opaque-reference model for the
first-party transport.

### 4. The device holds the JWT

`mobile/`:

- Stores the JWT in `expo-secure-store` (Keychain / Keystore), replacing the opaque
  `dubbridge_session_ref` with the token payload `{ token, userId, workspaceId }`.
- A request interceptor injects `Authorization: Bearer <jwt>` on every authenticated
  call; a response interceptor calls `logout()` on `401`.
- On cold start, restores the stored token before first render (splash pattern) and
  routes straight to home when present.
- The `isJwtLike()` anti-JWT guards are **removed** (their invariant no longer holds).
- Login moves from the system-browser OAuth/PKCE handoff to an in-app
  **email/password form**.

This **reverses ADR-024's** "never an access or refresh token on the device".

### 5. Authorization semantics that must survive the inversion

- `assets.uploader_id` and every governance actor identity is still derived from the
  **verified token subject**, never from a request body (ADR-008 spirit preserved).
- Scope checks (`assets:read`, `assets:ingest`, `workspaces:*`, review scopes) remain
  enforced at `apps/api`; the issued token must carry the caller's scopes.
- Org-membership and role gates (ADR-027) and the publication gate (ADR-030) remain
  backend-enforced and fail-closed. UI visibility is never an authorization boundary
  (ADR-029 §"UI visibility is never an authorization boundary" is retained).

## Risk analysis (required: RRI Excessive)

This decision is a **deliberate, directive-driven security downgrade**. It is
recorded explicitly so acceptance is informed.

| # | Risk | Severity | Why it appears | Mitigation in S-200 |
|---|------|----------|----------------|---------------------|
| R1 | Long-lived bearer token persisted on the device | High | Device is the least-controlled surface; theft/extraction yields a usable token until `exp` | Secure-store only (Keychain/Keystore); short `DUBBRIDGE_JWT_EXPIRY_HOURS`; never log the token |
| R2 | Symmetric secret signs **and** verifies | High | A `DUBBRIDGE_JWT_SECRET` leak lets an attacker **mint** valid tokens for any subject | Secret via injected env (ADR-026), fail-closed if unset; rotate on suspicion; X-S-200-1 tracks RS256 hardening |
| R3 | No pre-expiry revocation | Medium | No server-side session means logout cannot invalidate an outstanding token | Short expiry; optional `jti` deny-list as a follow-up (X-S-200-2) |
| R4 | Loss of gateway-owned rotation / refresh indirection | Medium | ADR-024 isolated token lifecycle from the client; relay removes that isolation | Document the trade-off; keep relay minimal and auditable |
| R5 | `apps/api` now an issuer, broadening its blast radius | Medium | Credential validation + signing live next to protected resources | Isolate issuance in `crates/auth`; audit login success/failure (ADR-018) |
| R6 | Algorithm-substitution / `alg:none` forgery | High | Symmetric verification is the classic substitution target | Pin `alg=HS256` at parse time; reject `none`; characterization test mirrors `verify_rejects_algorithm_substitution` |
| R7 | Email enumeration via differential errors | Low | Distinct errors for unknown-email vs wrong-password leak account existence | Generic `401` for both; constant-ish comparison via bcrypt on a dummy hash when account is absent |

**Recommended hardening (not part of full-fidelity parity, tracked as follow-ups):**

- **X-S-200-1** — issue with **RS256** (asymmetric) so the verification key can be
  public and the signing key is not co-located with verification. This keeps the
  FenixCRM *flow* while restoring ADR-023's key-separation property. Deferred because
  the directive selected full HS256 fidelity.
- **X-S-200-2** — `jti` + deny-list (or short-lived access + rotating refresh) to
  enable pre-expiry revocation.

## Consequences

**Positive**
- One self-contained auth system; no dependency on an external authorization server
  or its deployment (removes ADR-023's "needs issuer/audience/RSA key" operational
  cost).
- Login UX matches FenixCRM: in-app email/password, immediate token, no system-browser
  redirect.
- The gateway shrinks to a relay; session store, cookie policy, CSRF, and rotation
  logic are removed from the mobile path.
- The flow is portable and matches a documented reference the team already maintains.

**Negative / trade-offs**
- The security regressions R1–R7 above, accepted by directive.
- Removes the ADR-024 property that the client never holds tokens — the single most
  important security property of the prior design.
- DubBridge now operates an identity provider's responsibilities (password storage,
  reset, lockout, secret rotation) that ADR-023 deliberately avoided. Account
  lifecycle (reset, lockout, MFA) becomes DubBridge's burden and is out of scope for
  v1.
- Two accepted ADRs are superseded; downstream docs that cite them must be updated on
  acceptance.

## Alternatives considered

- **Keep the opaque-session gateway (status quo, ADR-024)** — rejected by directive;
  does not match FenixCRM.
- **In-house IdP behind the gateway, no token on device** — DubBridge issues its own
  tokens but the gateway keeps the opaque-session seam so the device never holds a
  JWT. Rejected by directive (selected "full FenixCRM fidelity"); retained here as the
  lower-risk design the team explicitly declined.
- **In-house issuance with RS256 instead of HS256** — rejected for v1 fidelity;
  preserved as hardening **X-S-200-1**.
- **Amend ADR-023/ADR-024 instead of a new ADR** — rejected: the inversion is large
  enough and reverses enough accepted text that a dedicated superseding ADR is the
  auditable record.

## Related

- ADR-023 (API client authentication and principal propagation) — **superseded on
  acceptance**; its RS256 resource-server-only boundary is replaced by in-house HS256
  issuance.
- ADR-024 (low-friction first-party API access via session gateway) — **superseded on
  acceptance**; the server-side session / opaque-reference transport is replaced by a
  relay + token-on-device.
- ADR-029 (mobile as the sole authenticated product surface) — **amended on
  acceptance**: mobile stays the sole surface, but its transport changes from
  opaque-session to bearer-token; the "UI is not an authorization boundary" clause is
  retained.
- ADR-008 (rights ledger fail-closed precondition) — uploader identity must remain
  derived from the verified token subject after the change.
- ADR-018 (structured observability) — login success/failure and registration emit
  durable audit rows.
- ADR-026 (layered fail-closed configuration) — `DUBBRIDGE_JWT_SECRET` and
  `DUBBRIDGE_JWT_EXPIRY_HOURS` are injected secrets/profile values; production fails
  closed without the secret.
- ADR-027 (org membership authorization), ADR-030 (review/publication gate) —
  backend authorization is unchanged and remains fail-closed.
- Opened by: `docs/plan/s-200-mobile-jwt-credential-auth.md`,
  `docs/tasks/s-200-mobile-jwt-credential-auth.md`,
  `docs/bdd/s-200-mobile-auth.feature`.
- Source reference: `/Users/matias/fenix/docs/mobile-auth-flow-reference.md`.
