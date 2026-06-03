# Tasks: P1 — First-party session gateway / BFF

**Plan:** `docs/plan/p1-session-gateway-bff.md`
**Roadmap slice:** P1 (supporting platform). Depends on S0 (ADR-023) + external
authorization-server contract. Prerequisite for the web frontend and for **P3**
(mobile, React Native + Expo).

**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-024 (primary), ADR-023, ADR-026, ADR-018.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
T0 -> T1 -> T2 -> T3 -> T4 -> T5 -> T6
```

---

## T0 — Finalize ADR-024 decisions (cookie policy, CSRF, session store, mobile seam)

- **Status:** [x] Done — 2026-06-03
- **Effort:** S
- **Complexity:** Low
- **Type:** Docs / ADR (no code)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** —
- **Objective:** Move ADR-024 from *Proposed* to *Accepted* by recording the
  concrete decisions P1 needs: cookie attributes (`HttpOnly`, `Secure`, `SameSite`
  value), CSRF strategy (double-submit token vs. `SameSite=Strict` + origin check),
  session-store choice (Redis-backed, in-memory for tests), session TTL/idle
  policy, and the **transport-agnostic session seam** that lets P3 (mobile) reuse
  the gateway without a parallel auth path.
- **Inputs:** ADR-024 (current Proposed text), ADR-023, ADR-026, this plan.
- **Outputs:** Updated `docs/adr/ADR-024-...md` (Status → Accepted; decisions added;
  `Implemented by: docs/tasks/p1-session-gateway-bff.md`); no code.
- **Acceptance criteria:**
  - ADR-024 Status is `Accepted` with date.
  - Cookie policy, CSRF posture, session store, and TTL are stated unambiguously.
  - A short subsection states how a native mobile client (P3) reuses the same
    session contract (opaque session id; cookie is one transport).
  - No code is changed in this task.
- **Completion record (2026-06-03):**
  - ADR-024 moved `Proposed -> Accepted` (header dated 2026-06-03).
  - Added a "Concrete decisions (accepted 2026-06-03, P1 T0)" section to
    `docs/adr/ADR-024-...md` recording:
    - Login: Authorization Code + PKCE (`S256`) + single-use `state`.
    - Cookie: opaque session id only; `HttpOnly` + `Secure` + `SameSite=Lax` +
      host-scoped `Path=/`.
    - CSRF: `Origin`/`Referer` check + double-submit token; `state` covers callback.
    - Session store: trait + in-memory (tests) + Redis (runtime); stores
      `TokenSet`, subject, CSRF token, timestamps.
    - TTL: 8h absolute + 30m idle, env-configurable (ADR-026); transparent access-
      token refresh while the session is valid.
    - Mobile seam: opaque session id is canonical; cookie is one transport; P3
      (RN/Expo) reuses the same gateway via system-browser OAuth + secure-store,
      never holding access/refresh tokens.
  - Added `Implemented by (planned): P1` and `Consumed by (planned): P3` references
    to the ADR's Related section.
  - No code changed (docs/ADR-only).

---

## T1 — `apps/gateway` scaffold + state + fail-closed config + health

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T0
- **Objective:** Create the `apps/gateway` Axum binary crate, its `GatewayState`,
  and a typed `GatewaySettings` wired through the layered fail-closed config
  (ADR-026). Expose `/health/live` and `/health/ready`. No auth logic yet.
- **Inputs:** `crates/config` layered loader, `crates/observability`, `apps/api`
  bootstrap pattern (`apps/api/src/main.rs`) as the structural reference.
- **Outputs:** `apps/gateway/{Cargo.toml, src/main.rs, src/state.rs, src/lib.rs}`,
  `GatewaySettings` in `crates/config`, workspace member added to root `Cargo.toml`,
  non-secret profile keys in `config/*.toml`, secrets documented in `.env.example`.
- **Acceptance criteria:**
  - `cargo build -p dubbridge-gateway` succeeds; the binary boots with valid config.
  - Missing required gateway config (e.g., authorization-server endpoints, upstream
    api url, client secret in a production-like env) aborts startup (fail-closed).
  - `/health/live` and `/health/ready` return 200 and are public.
- **Happy paths considered:**
  - valid `DUBBRIDGE_ENV=local` + complete gateway profile → gateway boots; health
    endpoints return 200.
- **Edge cases considered:**
  - missing upstream api base url → startup aborts with a clear error, no bind.
  - production-like env with a localhost authorization-server endpoint → rejected
    by `validate()` (consistent with ADR-026 fail-closed posture).

---

## T2 — OAuth client: PKCE token exchange + refresh (pure builder + IO executor)

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T1
- **Objective:** Implement the OAuth 2.0 client against the external authorization
  server: build the authorization-code request (with PKCE `code_challenge` + state),
  exchange the code for tokens, and refresh an access token using a refresh token.
  Split a **pure request builder** from the **IO executor** (mirrors the ADR-025
  connectors seam) so the builder is unit-testable without network.
- **Inputs:** authorization-server endpoint config from `GatewaySettings`, `reqwest`.
- **Outputs:** `apps/gateway/src/auth/oauth_client.rs` (builder + executor),
  PKCE verifier/challenge helpers.
- **Acceptance criteria:**
  - PKCE `code_verifier`/`code_challenge` (S256) generated correctly; `state` is
    random and single-use.
  - Token-exchange and refresh request builders are pure and unit-tested against
    expected URLs/params/headers without performing IO.
  - Executor parses a token response into a typed `TokenSet` (access, refresh,
    expiry); error responses map to typed errors, never panics.
  - The client secret is read from config (injected env), never logged (ADR-018
    redaction).
- **Happy paths considered:**
  - valid code + matching verifier → builder produces correct token-exchange
    request → executor returns a `TokenSet`.
  - expired access token + valid refresh token → refresh builder produces correct
    request → new `TokenSet`.
- **Edge cases considered:**
  - authorization server returns `invalid_grant` → typed error surfaced; no token
    stored; secret not echoed in logs.
  - malformed token response (missing `access_token`) → typed parse error, no panic.

---

## T3 — Session store (trait + in-memory + Redis) + hardened cookie + CSRF

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T2
- **Objective:** Define the server-side session contract: an opaque session id maps
  to a stored `TokenSet` + subject + CSRF token + timestamps. Provide an in-memory
  store (tests) and a Redis store (runtime) behind a trait. Build the hardened
  session cookie (`HttpOnly`, `Secure`, `SameSite` per T0) and CSRF helper.
- **Inputs:** T0 decisions, T2 `TokenSet`, Redis (already reserved), cookie deps.
- **Outputs:** `apps/gateway/src/session/{mod.rs, store.rs}`,
  `apps/gateway/src/cookie.rs`.
- **Acceptance criteria:**
  - Tokens are stored server-side only; the cookie carries only an opaque,
    high-entropy session id — never an access/refresh token (core ADR-024 property,
    asserted by a test).
  - Cookie is `HttpOnly` + `Secure` + the `SameSite` value chosen in T0.
  - Session TTL/idle expiry honored; an expired session id resolves to "no session".
  - CSRF token issued and verifiable per the T0 strategy.
- **Happy paths considered:**
  - store a session → resolve by id → returns the stored subject + token set.
- **Edge cases considered:**
  - tampered/unknown session id → resolves to no session (fail-closed; no 500).
  - test assertion: serialized cookie value does **not** contain the access token.

---

## T4 — Login / callback / logout routes

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T3
- **Objective:** Wire the OAuth client (T2) and session store (T3) into HTTP routes:
  `/auth/login` (start PKCE + redirect), `/auth/callback` (validate state, exchange
  code, create session, set cookie), `/auth/logout` (invalidate session + clear
  cookie).
- **Inputs:** T2 oauth client, T3 store + cookie.
- **Outputs:** `apps/gateway/src/auth/{login.rs, logout.rs}`, routes mounted.
- **Acceptance criteria:**
  - `/auth/login` redirects to the authorization server with PKCE + state and stores
    the pending state/verifier.
  - `/auth/callback` rejects a mismatched/expired `state` (CSRF on login leg), and on
    success creates a session and sets the hardened cookie.
  - `/auth/logout` invalidates the server-side session and clears the cookie.
- **Happy paths considered:**
  - login → callback with valid code+state → session created, hardened cookie set.
  - logout → session removed; subsequent proxy call is unauthenticated.
- **Edge cases considered:**
  - callback with mismatched `state` → rejected, no session created.
  - callback with a code the authorization server rejects → 401/redirect, no
    session; secret not leaked.

---

## T5 — Authenticated proxy to `apps/api` with transparent refresh

- **Status:** [ ] Not started
- **Effort:** L
- **Complexity:** High
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
  (cross-service flow + refresh-on-expiry concurrency reasoning)
- **Depends on:** T4
- **Objective:** Implement the authenticated forwarding layer: gateway `/api/*`
  routes resolve the session, attach a valid `Bearer` access token (refreshing
  transparently when expired), and forward the request/response to `apps/api`. The
  upstream API contract (ADR-023) is unchanged — the gateway only supplies the
  bearer token.
- **Inputs:** T3 store, T2 refresh, upstream `apps/api` base url from config.
- **Outputs:** `apps/gateway/src/proxy.rs`, routes mounted.
- **Acceptance criteria:**
  - A request with a valid session is forwarded to `apps/api` with a valid
    `Authorization: Bearer` header derived from the stored token.
  - When the stored access token is expired but the refresh token is valid, the
    gateway refreshes once, updates the session, and forwards — transparent to the
    client.
  - A request without a valid session is rejected at the gateway (401) and never
    forwarded.
  - The gateway never forwards the client's cookie or session id upstream as a
    bearer; it never exposes tokens to the client.
- **Happy paths considered:**
  - valid session + non-expired token → forwarded with bearer → upstream 200 relayed.
  - valid session + expired token + valid refresh → refresh → forwarded → 200.
- **Edge cases considered:**
  - no/invalid session → 401 at gateway, no upstream call.
  - refresh fails (refresh token revoked) → session invalidated, client gets 401,
    cookie cleared; no token leaked.
  - upstream `apps/api` returns 403 (scope) → relayed unchanged (authorization stays
    an `apps/api` concern, ADR-023).

---

## T6 — End-to-end lifecycle tests + docs/ADR/roadmap sync

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development (tests) + docs sync
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T5
- **Objective:** Add deterministic end-to-end tests over a stubbed authorization
  server and a stubbed upstream API covering login → callback → authenticated proxy
  → refresh → logout, plus the failure branches. Synchronize status artifacts.
- **Inputs:** all prior tasks; stub harness pattern from `crates/auth` tests.
- **Outputs:** `apps/gateway` integration tests; updated `docs/architecture.md`
  (gateway promoted to operational), `docs/plan/roadmap.md` (P1 status + P3 note),
  ADR-024 `Implemented by` reference confirmed.
- **Acceptance criteria:**
  - Full lifecycle test passes deterministically (no real network).
  - A test asserts the access token never appears in any client-visible response or
    cookie.
  - `docs/architecture.md` and `docs/plan/roadmap.md` reflect P1 as built; ADR-024
    references this slice.
  - Coverage meets the repository gate (≥90%, per T1/X6).
- **Happy paths considered:**
  - full login→proxy→logout lifecycle green end to end.
- **Edge cases considered:**
  - mid-session token expiry triggers exactly one transparent refresh in the e2e
    flow.
  - logout then proxy → 401 (session gone).

---

## Agent handoff prompt (delegation-ready)

> Implement slice **P1 — first-party session gateway / BFF** in the `dubbridge`
> repo, one task at a time in order T0→T6, per `docs/tasks/p1-session-gateway-bff.md`
> and `docs/plan/p1-session-gateway-bff.md`. Read the canonical guides first
> (`README_AGENT_ORDER.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
> `docs/policies/HITL_AUTONOMY_POLICY.md`, `AGENTS.md`) and ADR-024/023/026/018.
> Build a new `apps/gateway` Axum service; **do not modify the `apps/api` JWT
> resource-server trust boundary** (ADR-023). Tokens must live server-side only;
> the client sees only a hardened opaque session cookie (ADR-024). Keep the session
> contract transport-agnostic so slice **P3 (mobile, React Native + Expo)** can
> reuse the same gateway. TDD: write tests first, then implement, then run all
> tests. Do not commit with broken tests. Present each task for explicit approval
> before implementing it; mark progress in this file after each task.
