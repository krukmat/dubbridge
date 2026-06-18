---
type: TaskList
title: "Tasks: P1-T5 — Authenticated proxy to `apps/api` with transparent refresh"
status: closed
slice: S-040
plan: docs/plan/s-040-session-gateway-bff.md
---
# Tasks: P1-T5 — Authenticated proxy to `apps/api` with transparent refresh

**Parent task:** T5 in `docs/tasks/s-040-session-gateway-bff.md`
**Plan:** `docs/plan/s-040-session-gateway-bff.md`
**Depends on:** P1-T4 (login/callback/logout routes)
**Unlocks:** P1-T6 (end-to-end lifecycle tests)

**Governing ADRs:** ADR-024 (tokens server-side only), ADR-023 (apps/api JWT boundary
unchanged), ADR-018 (token redaction), ADR-026 (fail-closed config).

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Sub-task dependency order

```text
T5.1 → T5.2 → T5.3
```

---

## T5.1 — Shared cookie extractor + session resolver

- **Status:** [x] Done — 2026-06-03
- **Effort:** S
- **Complexity:** Low
- **Parent:** T5
- **Depends on:** T4 (session store wired in GatewayState)
- **Objective:** Extract the `extract_session_id` helper (currently duplicated in
  `logout.rs`) into a shared module `src/cookie_ext.rs` (or `src/session_ext.rs`),
  and add a `resolve_session` helper that wraps the store call and returns
  `Option<(SessionId, StoredSession)>`. Both will be consumed by T5.2 and T5.3.
- **Inputs:**
  - `logout.rs::extract_session_id` (copy to move)
  - `SessionStore::resolve`
  - `GatewaySettings::session` (ttl fields + cookie_name)
- **Outputs:**
  - `apps/gateway/src/cookie_ext.rs` — `pub fn extract_session_id(headers, name) -> Option<SessionId>`
  - `apps/gateway/src/session_ext.rs` (or same file) — `pub async fn resolve_session(state, headers) -> Option<(SessionId, StoredSession)>`
  - `apps/gateway/src/lib.rs` — `pub mod cookie_ext;` (or `session_ext`)
  - `apps/gateway/src/auth/logout.rs` — replace local `extract_session_id` with the shared one
- **Acceptance criteria:**
  - `extract_session_id` with a valid cookie header returns `Some(SessionId)`.
  - `extract_session_id` with no matching cookie returns `None`.
  - `resolve_session` with a live session returns `Some((id, session))`.
  - `resolve_session` with unknown/expired session returns `None`.
  - `logout.rs` still compiles and all 70 existing tests pass.
- **Effort note:** Mechanical refactor + new helper with 4–5 unit tests.
- **Completion record (2026-06-03):**
  - Created `apps/gateway/src/cookie_ext.rs`: `extract_session_id` (moved from
    `auth/logout.rs`, now public), `resolve_session` (cookie extract + store
    resolve + touch in one fail-closed step).
  - Added `pub mod cookie_ext` to `apps/gateway/src/lib.rs`.
  - Updated `apps/gateway/src/auth/logout.rs`: replaced local `extract_session_id`
    definition with `use crate::cookie_ext::extract_session_id`.
  - 8 new tests: 4 for `extract_session_id` (present, absent, no header,
    multi-cookie), 4 for `resolve_session` (live session, unknown id, absent
    cookie, expired session).
  - Verification: `cargo test -p dubbridge-gateway` — 78/78 passed (8 new + 70 prior).

---

## T5.2 — Token expiry check + transparent refresh logic

- **Status:** [x] Done — 2026-06-03
- **Effort:** M
- **Complexity:** Medium
- **Parent:** T5
- **Depends on:** T5.1
- **Objective:** Implement the token-refresh decision and execution as a pure-ish
  function `ensure_fresh_token` in `src/proxy.rs`. Given a `StoredSession`, it
  decides whether the access token needs refreshing (based on `expires_in` and
  `created_at`), and if so calls `OAuthExecutor::send_token_request` with the
  refresh grant, creates a new session, deletes the old one, and returns both the
  fresh `access_token` string and the new `SessionId`. On refresh failure it returns
  a typed error so the caller can clear the session and respond 401.
- **Inputs:**
  - `StoredSession` (subject, token_set, csrf_token, timestamps)
  - `build_token_refresh_params` (T2)
  - `OAuthExecutor::send_token_request` (T2)
  - `SessionStore::create` + `SessionStore::delete` (T3)
  - `GatewaySettings::oauth` (token_url, client_id, client_secret)
  - `GatewaySettings::session::absolute_ttl_seconds`
- **Outputs:**
  - `apps/gateway/src/proxy.rs` — `pub(crate) async fn ensure_fresh_token(...) -> Result<(String, SessionId), RefreshError>`
  - `RefreshError` enum: `RefreshFailed(OAuthError)`, `NoRefreshToken`, `StoreFailed`
- **Acceptance criteria:**
  - Token with remaining lifetime > refresh window (60 s) → returned as-is, no
    network call made (asserted by test with no mock registered).
  - Token at/past expiry with valid refresh → exactly one POST to token endpoint,
    new `SessionId` returned, old session deleted from store.
  - Token at/past expiry with no refresh token → `Err(NoRefreshToken)`.
  - Authorization server returns `invalid_grant` → `Err(RefreshFailed(...))`, old
    session deleted.
  - All tests use `wiremock` to stub the token endpoint; no real network calls.
- **Pseudocode:**
  ```
  fn token_expires_at(session) -> Option<u64>:
      session.token_set.expires_in.map(|e| session.created_at_unix_secs + e)

  fn needs_refresh(session) -> bool:
      match token_expires_at(session):
          None    → false          // unknown expiry, let apps/api reject
          Some(t) → unix_now() + REFRESH_WINDOW_SECS >= t

  async fn ensure_fresh_token(state, old_id, session) -> Result<(String, SessionId), RefreshError>:
      if !needs_refresh(&session):
          return Ok((session.token_set.access_token, old_id))

      refresh_token = session.token_set.refresh_token.ok_or(NoRefreshToken)?
      params = build_token_refresh_params(client_id, &refresh_token)
      new_tokens = executor.send_token_request(token_url, params, secret).await
          .map_err(RefreshFailed)?

      new_session = StoredSession::new(session.subject, new_tokens, session.csrf_token)
      new_id = store.create(new_session, abs_ttl).await.map_err(StoreFailed)?
      let _ = store.delete(&old_id).await    // best-effort

      Ok((new_tokens.access_token, new_id))
  ```

- **Completion record (2026-06-03):**
  - Created `apps/gateway/src/proxy.rs`: `RefreshError` (NoRefreshToken /
    RefreshFailed / StoreFailed), `token_expires_at`, `needs_refresh`
    (REFRESH_WINDOW_SECS = 60), `unix_now`, `ensure_fresh_token`.
  - Added `pub mod proxy` to `apps/gateway/src/lib.rs`.
  - 9 tests: `needs_refresh` (absent expiry, plenty of life, expired, within window),
    `ensure_fresh_token` (no refresh needed, no refresh token, successful refresh +
    old session deleted, refresh failure + old session deleted, csrf preserved).
  - Verification: `cargo test -p dubbridge-gateway` — 87/87 passed (9 new + 78 prior).

---

## T5.3 — HTTP proxy handler + route mount

- **Status:** [x] Done — 2026-06-04
- **Effort:** M
- **Complexity:** Medium
- **Parent:** T5
- **Depends on:** T5.2
- **Objective:** Implement the catch-all `proxy_handler` in `src/proxy.rs` that
  wires T5.1 (session resolver) and T5.2 (refresh logic) into a full HTTP forwarding
  path: extract session, ensure fresh token, forward the request to `apps/api`,
  relay the upstream response. Mount it at `/api` in `lib.rs`.
- **Inputs:**
  - `resolve_session` (T5.1)
  - `ensure_fresh_token` (T5.2)
  - `GatewayState.http_client` for upstream call
  - `GatewayState.gateway.upstream_api_base_url`
  - `build_session_cookie` + `clear_session_cookie` + `clear_csrf_cookie` (T3 cookie.rs)
- **Outputs:**
  - `apps/gateway/src/proxy.rs` — `pub async fn proxy_handler(...)` + `pub fn proxy_router()`
  - `apps/gateway/src/lib.rs` — `.nest("/api", proxy_router())` added to `build_app`
- **Acceptance criteria:**
  - Request with valid session + non-expired token → forwarded with `Authorization: Bearer <token>`, upstream response relayed (status + body).
  - Request with valid session + expired token + valid refresh → one refresh, new session cookie set in response, upstream response relayed.
  - Request with no session cookie → 401, no upstream call.
  - Request with expired/unknown session id → 401, no upstream call.
  - Refresh failure → session deleted, both cookies cleared, 401.
  - `apps/api` 403 → relayed unchanged (no gateway interference).
  - Client `Cookie` header is NOT forwarded upstream (only `Authorization: Bearer`).
  - Access token never appears in any client-visible response header or body.
  - All cases covered with `wiremock` stubs; no real network calls.
- **Forwarding rules:**
  - Forward: method, path+query (`/api/X` → `upstream/X`), request body, safe
    headers (Content-Type, Accept, X-Request-Id, etc.).
  - Strip inbound: `Cookie`, `Authorization` (client must not inject a bearer).
  - Strip outbound: upstream `Set-Cookie` (prevent session-id collision),
    `Transfer-Encoding` (reqwest decodes chunked).
- **Pseudocode:**
  ```
  async fn proxy_handler(State(state), headers, method, OriginalUri(uri), body):
      (session_id, session) = resolve_session(&state, &headers).await
          else → return 401

      (access_token, new_session_id) = ensure_fresh_token(&state, session_id, session).await
          on Err(NoRefreshToken | RefreshFailed) →
              delete old session, clear both cookies → return 401
          on Err(StoreFailed) → return 500

      upstream_path = strip_api_prefix(uri)   // /api/tracks → /tracks
      upstream_url  = format!("{}{}", upstream_base, upstream_path)

      upstream_resp = http_client
          .request(method, upstream_url)
          .header("Authorization", format!("Bearer {}", access_token))
          .headers(forward_safe_headers(headers))   // strips Cookie + Authorization
          .body(body)
          .send().await
          else → return 502

      relay_response(upstream_resp, new_session_id_if_refreshed)
      // relay: status, safe headers (strip Set-Cookie + Transfer-Encoding), body stream
  ```

- **Completion record (2026-06-04):**
  - Extended `apps/gateway/src/proxy.rs` with `proxy_router()` and `proxy_handler`,
    plus small relay helpers for unauthorized responses, upstream URL building,
    and outbound header sanitization.
  - Mounted the proxy router in `apps/gateway/src/lib.rs` under `.nest("/api", proxy_router())`.
  - Forwarding behavior implemented:
    - resolves the session with `resolve_session`
    - refreshes access tokens via `ensure_fresh_token`
    - strips inbound `Cookie`, `Authorization`, and `Host`
    - strips outbound `Set-Cookie` and `Transfer-Encoding`
    - sets a fresh session cookie when refresh rotates the session id
    - clears both session and CSRF cookies on no-session and refresh-failure `401`s
  - Added 6 wiremock-backed tests covering:
    - no session → `401` + cleared cookies + no upstream call
    - unknown session → `401` + no upstream call
    - live token forwarding with sanitized headers and relayed body/status
    - expired token + successful refresh → new session cookie + forwarded request
    - refresh failure → `401` + cleared cookies + no upstream call
    - upstream `403` relay with outbound `Set-Cookie` / `Transfer-Encoding` stripped
  - Verification:
    - `~/.cargo/bin/cargo test -p dubbridge-gateway` — 93/93 passed

---

## Agent handoff prompt (delegation-ready)

> Implement sub-tasks **T5.1 → T5.2 → T5.3** from
> `docs/tasks/s-040-t5-proxy.md` in the `dubbridge` repo.
> Parent context: `docs/tasks/s-040-session-gateway-bff.md` (T5),
> `docs/plan/s-040-session-gateway-bff.md`.
> T4 is complete — `GatewayState` carries `session_store` and `pending_store`;
> cookie/session helpers live in `src/cookie.rs` and `src/session/`.
> Key types: `extract_session_id` in `src/auth/logout.rs` (move to shared module
> in T5.1), `build_token_refresh_params` + `OAuthExecutor` in
> `src/auth/oauth_client.rs`, `SessionStore` in `src/session/mod.rs`.
> TDD: write tests first using `wiremock` (already in dev-deps) to stub the token
> endpoint and `apps/api`. Do not commit with failing tests. Present each sub-task
> for explicit approval before implementing. Mark progress in this file after each
> sub-task.
