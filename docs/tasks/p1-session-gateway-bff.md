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

- **Status:** [x] Done — 2026-06-03
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
- **Completion record (2026-06-03):**
  - Added the new `apps/gateway` Axum crate with `src/main.rs`, `src/lib.rs`,
    `src/state.rs`, public `/health/live` and `/health/ready`, and a minimal
    `GatewayState` carrying the shared config plus a `reqwest::Client`.
  - Added `GatewaySettings`, `GatewayOAuthSettings`, and
    `GatewaySessionSettings` to `crates/config/src/lib.rs`, plus
    `AppConfig::gateway_settings()` for fail-closed gateway bootstrap.
  - Extended `AppConfig::validate()` so gateway config rejects:
    missing `gateway.upstream_api_base_url`, empty OAuth endpoint/client values,
    zero TTLs, missing `gateway.oauth.client_secret` in production-like envs,
    and localhost authorization/token/upstream URLs in production-like envs.
  - Added committed non-secret gateway profiles to `config/local.toml`,
    `config/staging.toml`, and `config/production.toml`; documented the injected
    secret `DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET` in `.env.example`.
  - Verification:
    - `~/.cargo/bin/cargo fmt --all`
    - `~/.cargo/bin/cargo test -p dubbridge-config`
    - `~/.cargo/bin/cargo build -p dubbridge-gateway`
  - `dubbridge-config` tests passed `37/37`, including new gateway load/validation
    coverage and local/public health-route coverage in the gateway crate.

---

## T2 — OAuth client: PKCE token exchange + refresh (pure builder + IO executor)

- **Status:** [x] Done — 2026-06-03
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
- **Completion record (2026-06-03):**
  - Created `apps/gateway/src/auth/mod.rs` and `apps/gateway/src/auth/oauth_client.rs`.
  - Added `pub mod auth;` to `apps/gateway/src/lib.rs`.
  - Implemented: `PkceVerifier` (32 random bytes, base64url, 43 chars), `PkceChallenge`
    (SHA-256 S256), `OAuthState` (24 random bytes, base64url), `TokenSet`,
    `OAuthError` (ServerError / InvalidResponse / Http / UrlParse),
    `build_authorization_url` (pure URL builder), `build_token_exchange_params` (pure),
    `build_token_refresh_params` (pure), `parse_token_response` (pure JSON parser),
    `OAuthExecutor` (IO executor — injects secret only at send time per ADR-018).
  - Added deps to `apps/gateway/Cargo.toml`: `sha2 = "0.10"`, `base64 = "0.22"`,
    `rand = "0.8"`, `url = "2.5"`, `thiserror = "2.0"`, `serde_json.workspace`,
    `reqwest` feature `json` added.
  - 22 new unit tests cover PKCE invariants, builder output, and token response parsing.
  - Verification:
    - `cargo test -p dubbridge-gateway` — 23/23 passed (22 new + 1 existing health test).
    - `cargo test -p dubbridge-config` — 37/37 passed (no regressions).

---

## T3 — Session store (trait + in-memory + Redis) + hardened cookie + CSRF

- **Status:** [x] Done — 2026-06-03
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
- **Completion record (2026-06-03):**
  - Created `apps/gateway/src/session/mod.rs`: `SessionId` (32-byte base64url, 43 chars),
    `CsrfToken` (24-byte base64url, constant-time `verify` via `subtle`), `StoredSession`
    (subject + TokenSet + CsrfToken + unix timestamps), `is_expired()` (absolute + idle
    TTL check), `SessionError`, `SessionStore` async trait (via `async_trait` for
    `dyn`-safety).
  - Created `apps/gateway/src/session/store.rs`:
    - `InMemorySessionStore` — `Arc<Mutex<HashMap>>`, lazy expiry on resolve, deterministic
      for tests.
    - `RedisSessionStore` — `redis::aio::ConnectionManager`, key `dubbridge:session:{id}`,
      JSON value, Redis TTL = absolute_ttl, idle TTL enforced in Rust on resolve, KEEPTTL
      on touch to preserve absolute deadline.
  - Created `apps/gateway/src/cookie.rs`: `build_session_cookie` (HttpOnly + Secure +
    SameSite=Lax + Path=/), `clear_session_cookie` (Max-Age=0), `build_csrf_cookie`
    (non-HttpOnly, for double-submit JS read), `clear_csrf_cookie`.
  - Extended `apps/gateway/src/state.rs`: `GatewayState` now carries
    `Arc<dyn SessionStore>` (injected at boot).
  - Updated `apps/gateway/src/main.rs`: boots `RedisSessionStore` from `config.redis_url`
    and injects into `GatewayState`.
  - Updated `apps/gateway/src/lib.rs`: `pub mod cookie; pub mod session;` + health test
    updated to inject `InMemorySessionStore`.
  - Added deps to `apps/gateway/Cargo.toml`: `async-trait = "0.1"`, `subtle = "2"`,
    `redis = { version = "0.27", features = ["tokio-comp", "connection-manager"] }`.
  - 29 new unit tests: SessionId invariants, CsrfToken verify, StoredSession TTL logic,
    InMemorySessionStore (create/resolve/touch/delete/expired), cookie attributes, and the
    ADR-024 invariant (cookie value must not contain the access token).
  - Verification:
    - `cargo test -p dubbridge-gateway` — 52/52 passed (29 new + 23 prior), 0 warnings.

---

## T4 — Login / callback / logout routes

- **Status:** [x] Done — 2026-06-03
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
- **Completion record (2026-06-03):**
  - Created `apps/gateway/src/auth/pending.rs`: `PendingAuthStore` (Mutex<HashMap>,
    TTL 10 min, single-use consume via `HashMap::remove`), `PendingError`
    (NotFound / Expired). 6 unit tests covering happy path, single-use invariant,
    missing state, expired state, verifier integrity, and multi-entry isolation.
  - Created `apps/gateway/src/auth/login.rs`: `login_handler` (GET /auth/login —
    PKCE + state generate, pending insert, 302 redirect to AS), `callback_handler`
    (GET /auth/callback — consume state, token exchange via `OAuthExecutor`, session
    create, set hardened cookies), `extract_jwt_subject` (base64-decode JWT payload,
    no sig verify per ADR-023). 7 tests: login redirect, required params, unknown
    state → 400, valid callback → 200 + hardened cookie, access token never in
    headers (ADR-018/ADR-024), single-use state, invalid_grant → 401.
  - Created `apps/gateway/src/auth/logout.rs`: `logout_handler` (POST /auth/logout —
    extract session cookie, best-effort delete, always clear both cookies),
    `extract_session_id`. 5 tests: noop on missing cookie, cookie clearing on noop,
    valid session deleted, both cookies cleared, idempotent on already-deleted session.
  - Updated `apps/gateway/src/auth/mod.rs`: added `pub mod login; pub mod logout;
    pub mod pending;` + `pub fn auth_router() -> Router<Arc<GatewayState>>` (state
    inherited from parent via nest(), not fixed in sub-router).
  - Updated `apps/gateway/src/lib.rs`: `build_app` nests `auth_router()` at `/auth`;
    updated health test to use new 5-arg `GatewayState::new`.
  - Updated `apps/gateway/src/state.rs`: added `pending_store: Arc<PendingAuthStore>`
    field to `GatewayState`; updated `new()` signature.
  - Updated `apps/gateway/src/main.rs`: boot `PendingAuthStore::with_default_ttl()`
    and pass to `GatewayState::new`.
  - Added dev-dependencies: `wiremock = "0.6"`, `axum-test = "16"`.
  - Verification:
    - `cargo test -p dubbridge-gateway` — 70/70 passed (18 new T4 + 52 prior), 0 warnings.
  - Files affected:
    - `apps/gateway/src/auth/pending.rs` (new)
    - `apps/gateway/src/auth/login.rs` (new)
    - `apps/gateway/src/auth/logout.rs` (new)
    - `apps/gateway/src/auth/mod.rs` (extended)
    - `apps/gateway/src/state.rs` (extended)
    - `apps/gateway/src/lib.rs` (extended)
    - `apps/gateway/src/main.rs` (extended)
    - `apps/gateway/Cargo.toml` (dev-deps added)

---

## T5 — Authenticated proxy to `apps/api` with transparent refresh

- **Status:** [x] Done — 2026-06-04
- **Effort:** L → split into 3 sub-tasks (see `docs/tasks/p1-t5-proxy.md`)
- **Complexity:** High
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T4
- **Sub-tasks:** `docs/tasks/p1-t5-proxy.md`
  - T5.1 — Shared cookie extractor + session resolver (Effort: S)
  - T5.2 — Token expiry check + transparent refresh logic (Effort: M)
  - T5.3 — HTTP proxy handler + route mount (Effort: M)
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
- **Completion record (2026-06-04):**
  - T5.1 complete — shared cookie extractor + session resolver in
    `apps/gateway/src/cookie_ext.rs`.
  - T5.2 complete — refresh-window evaluation and `ensure_fresh_token` in
    `apps/gateway/src/proxy.rs`.
  - T5.3 complete — authenticated `/api/*` proxy mounted in `apps/gateway/src/lib.rs`,
    forwarding with bearer injection, refresh-on-demand, inbound/outbound header
    stripping, session-cookie rotation on refresh, and fail-closed `401` cookie clearing.
  - Verification:
    - `~/.cargo/bin/cargo test -p dubbridge-gateway` — 93/93 passed

---

## T6 — End-to-end lifecycle tests + docs/ADR/roadmap sync

- **Status:** [x] Done — 2026-06-04
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
- **Completion record (2026-06-04):**
  - Added `apps/gateway/tests/e2e_lifecycle.rs` with 2 deterministic end-to-end tests:
    - `e2e_login_refresh_logout_lifecycle_is_deterministic` — full
      `/auth/login -> /auth/callback -> /api/* (refresh) -> /auth/logout -> stale /api/*`
      flow against stubbed authorization-server and upstream API surfaces.
    - `e2e_access_tokens_never_appear_in_client_visible_responses_or_cookies` —
      asserts callback/proxy responses and browser-visible cookies never contain the
      access token.
  - E2E coverage added in the actual crate integration-test surface rather than
    extending route-local unit tests, so lifecycle behavior is exercised through
    `build_app(...)` with real router wiring.
  - Updated `docs/architecture.md`: promoted the first-party session gateway / BFF
    to an operational supporting surface and documented `apps/gateway` under
    operational runtime surfaces.
  - Updated `docs/plan/roadmap.md`: marked P1 done and recorded that the hard P3
    dependency is now satisfied.
  - Updated `docs/plan/p1-session-gateway-bff.md`: implementation status now shows
    T0-T6 complete and P1 implemented.
  - Updated `docs/adr/ADR-024-low-friction-first-party-api-access-via-session-gateway.md`:
    replaced `Implemented by (planned)` with the delivered P1 implementation references.
  - Verification:
    - `~/.cargo/bin/cargo test -p dubbridge-gateway` — 95/95 passed
    - `make qa-docs` — passed
    - `make qa-coverage` — passed, total line coverage `94.54%` (`>= 90%` gate)

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
