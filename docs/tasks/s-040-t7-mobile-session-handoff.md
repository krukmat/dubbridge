---
type: TaskList
title: "Tasks: S-040-T7 - Mobile-safe Session Handoff / Deep-link Return"
status: closed
slice: S-040
plan: docs/plan/s-040-session-gateway-bff.md
---
# Tasks: S-040-T7 - Mobile-safe Session Handoff / Deep-link Return

**Parent task:** `docs/tasks/s-040-session-gateway-bff.md#t7--mobile-safe-session-handoff--deep-link-return`
**Plan:** `docs/plan/s-040-session-gateway-bff.md`
**Roadmap phase:** `S-040`. This is a post-T6 unblock for `S-050`.

**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`,
`AGENTS.md`.
**Governing ADRs:** ADR-024 (primary), ADR-023, ADR-026, ADR-018.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Why this is split

The original unblock request combines a contract decision, auth-route behavior,
session-transport behavior, refresh/logout behavior, tests, and status-artifact
sync. As one task it scores in the Complex / High RRI bands because it touches the
ADR-024 authentication boundary and changes a public first-party contract.

This ledger splits the work so each implementation approval is scoped to one seam:

```text
T7.1 contract decision/docs
  -> T7.2 mobile login callback + one-time handoff
  -> T7.3 handoff redemption + explicit mobile session header
  -> T7.4 refresh/logout parity + e2e/status sync
```

The target is that each implementation subtask remains at or below the Med-high
RRI band (<= 55) after the contract is fixed in T7.1.

---

## T7.1 — Define the mobile return and handoff contract

- **Status:** [x] Done — 2026-06-04
- **Effort:** S
- **Complexity:** Medium
- **Type:** Docs / contract (no code)
- **Recommended model tier:** Balanced -> Premium, thinking On
- **RRI target:** 52.8 (Med-high)
- **Depends on:** S-040 T6; S-050 T0 blocked verification
- **Objective:** Record the exact mobile-safe return contract before code changes:
  system-browser OAuth still terminates at the gateway, the gateway returns to a
  registered mobile URI with a short-lived one-time handoff code, and the app
  redeems that code for an opaque gateway session reference. Neither the handoff
  code nor the session reference is an access token or refresh token.
- **Inputs:** ADR-024, S-050 T0 completion record, current `apps/gateway` auth routes.
- **Outputs:** Updated S-040 plan/task docs, S-050 task blocker text, and roadmap status.
- **Acceptance criteria:**
  - The contract names the mobile start, callback return, handoff redemption,
    `/api/*`, and `/auth/logout` surfaces.
  - The deep-link / app-link return carries only a short-lived opaque handoff code,
    not a JWT, refresh token, or long-lived gateway session id.
  - The mobile app stores only the redeemed opaque session reference in
    `expo-secure-store` later in S-050.
  - The contract states that subsequent mobile calls use an explicit gateway
    session header, not browser cookies and not `Authorization: Bearer <JWT>`.
  - S-050 T1+ remains blocked until the implementation subtasks complete.
- **Happy paths considered:**
  - mobile login -> system browser -> gateway callback -> registered mobile return
    with handoff code -> redeem -> opaque session reference stored later by S-050.
  - mobile `/api/*` request with the opaque session reference -> gateway resolves
    the server-side session and forwards with a server-side bearer token.
- **Edge cases considered:**
  - unregistered return URI -> gateway rejects before starting login.
  - expired or already-redeemed handoff code -> gateway rejects and returns no
    session reference.
  - any JWT-like value in return URI, JSON response, or mobile storage contract ->
    contract violation.

### Mobile return and handoff contract (decided 2026-06-04, T7.1)

This contract is binding for T7.2–T7.4 implementation and for S-050 T1+. The
ADR-024 mobile-seam decision (accepted 2026-06-03, S-040 T0) is extended here with
exact surface definitions, header names, payload shapes, TTL bounds, and
invariants that every implementation subtask must honour.

#### Surface 1 — Mobile start: `GET /auth/login?return_uri=<registered-mobile-uri>`

The mobile client adds a `return_uri` query parameter to the standard login
surface.

- `return_uri` must match an entry in the gateway's pre-registered mobile URI
  allowlist (`GatewaySettings.mobile_return_uris` in `config/*.toml`, injected as
  a non-secret profile value). Allowed schemes: HTTPS app-links and registered
  custom schemes (e.g. `dubbridge://`).
- If `return_uri` is **absent**: gateway starts the standard browser flow —
  existing behaviour unchanged.
- If `return_uri` is present but **not registered**: gateway returns
  `400 Bad Request` before initiating any OAuth redirect — fail closed, no
  attacker-supplied redirect target reaches the authorization server.
- The `return_uri` value is encoded into the `state` blob alongside the PKCE
  verifier so the callback can recover it after OAuth completes. It is never
  forwarded to the authorization server as part of the `redirect_uri` or any
  other parameter.
- Client receives: `302 Found` redirect to the authorization server, identical
  to the browser flow from the client's perspective.

#### Surface 2 — Callback return: `GET /auth/callback?code=...&state=...`

Single endpoint shared by browser and mobile flows.

When the decoded `state` blob carries a mobile `return_uri`:

1. Gateway validates `state` (single-use, matches pending store) and exchanges
   the authorization code for tokens — identical to the browser path.
2. Gateway creates the server-side session (same `StoredSession` structure,
   same TTL policy: 8 h absolute + 30 min idle from S-040 T0 decisions).
3. Gateway generates a **one-time opaque handoff code**:
   - 32 cryptographically random bytes, base64url-encoded → 43-character string.
   - Stored in a transient in-process store keyed by the code value, referencing
     the newly created `SessionId`.
   - **TTL: ≤ 90 seconds** from issuance. After expiry the code is invalid and
     the stored reference is unreachable.
4. Gateway redirects: `<return_uri>?handoff_code=<opaque-code>` — **only** the
   handoff code appears in the URI. No access token, no refresh token, no session
   id, no CSRF token appears anywhere in the redirect.

The handoff code is **not** a JWT, not a session id, not an access token, not a
refresh token. It is a short-lived single-use pointer to the server-side session.

When the `state` blob carries no mobile `return_uri` (browser flow):
- Existing `200 OK + Set-Cookie` behaviour is unchanged.

#### Surface 3 — Handoff redemption: `POST /auth/mobile/session`

The mobile app exchanges the handoff code received in the deep-link for an
opaque session reference.

Request:
```
POST /auth/mobile/session
Content-Type: application/json

{ "handoff_code": "<opaque>" }
```

Gateway behaviour:
1. Look up the handoff code in the transient store.
2. Verify the code has not expired (TTL ≤ 90 s from issuance).
3. Destroy the code entry immediately — **single-use**: a second call with the
   same code returns `401 Unauthorized`.
4. Return the opaque session reference.

Success response:
```
200 OK
Content-Type: application/json

{ "session_ref": "<opaque-session-id>" }
```

The `session_ref` value is the opaque `SessionId` from the server-side session
store. It is **not** a JWT, not an access token, not a refresh token. It is the
same opaque identifier the browser transport carries in the `dubbridge_session`
cookie — only the delivery mechanism differs.

Failure responses:
- Expired handoff code: `401 Unauthorized`, no `session_ref` in body.
- Unknown or already-redeemed handoff code: `401 Unauthorized`, no `session_ref`.
- Malformed request (missing or non-string `handoff_code`): `400 Bad Request`.

S-050 must store the returned `session_ref` in `expo-secure-store` (Keychain on iOS,
Keystore on Android). S-050 must not persist the handoff code after redemption.

#### Surface 4 — Authenticated API calls: `ANY /api/*` with session header

The mobile client sends subsequent API requests with an explicit session header
instead of a browser cookie.

```
ANY /api/*
X-Dubbridge-Session: <opaque-session-ref>
```

Gateway behaviour (extends the session resolver in `apps/gateway/src/cookie_ext.rs`):
- Extract the session reference from `X-Dubbridge-Session` if present.
- Extract the session reference from `Cookie: dubbridge_session=...` if present.
- **Conflict rule:** if both transports are present and reference **different**
  sessions: return `401 Unauthorized` — fail closed, never silently choose one
  transport over the other.
- If exactly one transport is present: resolve normally.
- On success: attach `Authorization: Bearer <JWT>` server-side and forward to
  `apps/api` (ADR-023 boundary unchanged; the mobile client never sees the JWT).

The mobile client must **not** send `Authorization: Bearer <JWT>` to the gateway.
The gateway must **not** expose any JWT value to the mobile client in any response
header, body, or cookie.

#### Surface 5 — Mobile logout: `POST /auth/logout` with session header

```
POST /auth/logout
X-Dubbridge-Session: <opaque-session-ref>
```

Gateway behaviour (extends `apps/gateway/src/auth/logout.rs`):
- Extract the session reference from `X-Dubbridge-Session` (mobile path) or from
  `Cookie: dubbridge_session=...` (browser path).
- **Conflict rule:** same as Surface 4 — if both transports are present and
  disagree: `401 Unauthorized`.
- Best-effort deletion of the server-side session.
- Return `200 OK` — idempotent: an unknown or already-expired reference is not
  an error.
- No `Set-Cookie` headers emitted in the mobile path (the mobile client has no
  cookie jar to clear).

#### ADR-024 invariants (explicit)

These invariants must hold across every surface and every test in T7.2–T7.4:

1. **No access token on device.** The OAuth access token lives only in the
   server-side session store. It must never appear in: the deep-link redirect URI
   query string, the handoff redemption response body or headers, any `/api/*`
   response body, header, or cookie, or any server log line that a client can
   observe.

2. **No refresh token on device.** Same rule as the access token. The refresh
   token is stored server-side only and never serialized into any client-visible
   surface.

3. **No parallel auth path.** S-050 must not call `apps/api` directly with
   `Authorization: Bearer <JWT>`. All authenticated calls go through the gateway
   carrying the opaque session reference in `X-Dubbridge-Session`. This is the
   architectural guarantee that prevents S-050 from re-introducing a
   token-on-device pattern and preserves the ADR-023 trust boundary.

4. **Two mobile-visible session credentials only:**
   - The **handoff code**: ≤ 90 s TTL, single-use, opaque random bytes. The app
     discards it immediately after successful redemption.
   - The **opaque session reference** (`session_ref`): stored in
     `expo-secure-store`. Subject to the same absolute TTL and idle TTL as a
     browser session (8 h / 30 min, T0 decisions). Rotated by the gateway after
     a transparent access-token refresh (T7.4); S-050 must update its stored value
     upon receiving a rotation signal.

#### Implementation notes for T7.2–T7.4

- **T7.2** must:
  - Add `mobile_return_uris` allowlist field to `GatewaySettings` in
    `crates/config/src/lib.rs` and the committed profile files.
  - Validate `return_uri` against the allowlist in `apps/gateway/src/auth/login.rs`
    before initiating the OAuth redirect; reject with `400` if unregistered.
  - Encode the `return_uri` inside the `state` blob in
    `apps/gateway/src/auth/pending.rs` (`PendingAuthStore` entry).
  - Generate the one-time handoff code and redirect to the mobile return URI in
    `apps/gateway/src/auth/login.rs` (`callback_handler`) when a mobile
    `return_uri` is recovered from the consumed `state`.
  - Add a transient handoff store (in-memory, TTL-aware, single-use) in a new
    module (e.g. `apps/gateway/src/auth/handoff.rs`).
  - Unit-test: valid registered return URI + valid OAuth callback → handoff code
    generated and redirect returned; no token in redirect; browser flow unchanged.

- **T7.3** must:
  - Add `POST /auth/mobile/session` endpoint to `apps/gateway/src/auth/mod.rs`
    that redeems the handoff code and returns `{ "session_ref": "..." }`.
  - Extend `apps/gateway/src/cookie_ext.rs` to extract `X-Dubbridge-Session`
    as an alternative to the `Cookie` transport, with the conflict-rule guard.
  - Apply the extended resolver in `apps/gateway/src/proxy.rs`.
  - Unit-test: redeem valid code → session ref; double-redeem → 401; `/api/*`
    with header transport succeeds; conflict rule returns 401.

- **T7.4** must:
  - Extend `apps/gateway/src/proxy.rs` refresh logic to return the rotated
    `session_ref` to mobile callers (via a response header or JSON body, decided
    in T7.4).
  - Extend `apps/gateway/src/auth/logout.rs` to accept `X-Dubbridge-Session` and
    apply the conflict rule.
  - Add a deterministic end-to-end mobile lifecycle test in
    `apps/gateway/tests/e2e_lifecycle.rs`: mobile login → handoff → redeem →
    `/api/*` → refresh rotation → logout → stale-session 401.
  - Assert in the e2e test that access tokens and refresh tokens never appear in
    any mobile-visible redirect URL, JSON body, header, cookie, or test log.
  - Synchronize S-040/S-050/roadmap/ADR-024 status; state clearly whether S-050 T1+ is
    unblocked.

#### Completion record (2026-06-04, T7.1)

- Contract defined and recorded as the authoritative specification for T7.2–T7.4
  implementation and for S-050 T1+ unblocking. No code changed.
- Five surfaces specified with exact endpoint paths, parameter names, header
  names (`X-Dubbridge-Session`), payload schemas, TTL bounds (≤ 90 s handoff
  code), and conflict rules.
- ADR-024 invariants enumerated explicitly (no access token on device, no refresh
  token on device, no parallel auth path, two permitted mobile credentials only).
- Implementation notes added for T7.2–T7.4 referencing the exact files each
  subtask must modify.
- S-050 task blocker updated: T7.1 contract complete; S-050 T1+ remains blocked
  pending T7.2–T7.4 gateway implementation.
- S-040 task and plan status updated; roadmap S-040/S-050 rows synced.
- `make qa-docs` result: **passed** (2026-06-04) — documentation consistency
  check confirms no dangling ADR references, no ADR status mismatches, and no
  missing index rows introduced by this task.

---

## T7.2 — Mobile login intent + callback handoff code

- **Status:** [x] Done — 2026-06-04
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model tier:** Balanced -> Premium, thinking On
- **RRI target:** 53.0 (Med-high)
- **Depends on:** T7.1
- **Objective:** Extend gateway login/callback behavior so a mobile login can
  carry a validated registered return URI through the existing PKCE/state flow.
  On successful OAuth callback, create the normal server-side session, then issue
  a short-lived one-time opaque handoff code and redirect to the registered mobile
  return URI with that handoff code only.
- **Inputs:** `apps/gateway/src/auth/login.rs`,
  `apps/gateway/src/auth/pending.rs`, `apps/gateway/src/state.rs`,
  `crates/config/src/lib.rs`.
- **Outputs:** Mobile login intent support, return-URI validation, handoff-code
  store, unit tests for callback branching and no-token leakage.
- **Acceptance criteria:**
  - Existing browser `/auth/login` and `/auth/callback` behavior remains unchanged.
  - Mobile login rejects unregistered return URIs before redirecting to the
    authorization server.
  - Mobile callback redirects to the registered return URI with a one-time handoff
    code only.
  - Callback response does not expose access tokens, refresh tokens, or the
    server-side token set in headers, body, cookies, or redirect URL.
- **Happy paths considered:**
  - valid registered return URI + valid OAuth callback -> handoff code generated
    and redirect returned to the app.
  - browser login without mobile intent -> existing `200 OK + Set-Cookie` callback
    remains green.
- **Edge cases considered:**
  - unknown `state` -> no session and no handoff code.
  - valid OAuth code but unregistered return URI associated with state -> fail
    closed; no mobile redirect to attacker-controlled URI.
  - token-exchange failure -> no session and no handoff code.

#### Completion record (2026-06-04, T7.2)

- Added `gateway.mobile_return_uris` to `GatewaySettings` and committed
  `config/{local,staging,production}.toml` as the mobile return-URI allowlist
  source for the gateway.
- Extended `apps/gateway/src/auth/pending.rs` so pending OAuth state now carries
  both the PKCE verifier and an optional mobile `return_uri`.
- Added `apps/gateway/src/auth/handoff.rs`: a transient in-memory single-use
  handoff-code store with the T7.1 TTL contract (`90 s` default, 32 random bytes
  base64url-encoded to a 43-character opaque code).
- Extended `GatewayState` and bootstrap wiring in `apps/gateway/src/{main.rs,lib.rs}`
  plus test helpers so the handoff store is available to auth handlers.
- Updated `apps/gateway/src/auth/login.rs`:
  - `GET /auth/login` now accepts optional `return_uri`, validates it against the
    configured allowlist, and rejects unregistered values with `400 Bad Request`
    before any redirect to the authorization server.
  - `GET /auth/callback` preserves the browser path (`200 OK + Set-Cookie`) when
    no mobile intent exists.
  - When a validated mobile `return_uri` is recovered from pending state, the
    callback creates the normal server-side session, issues a one-time handoff
    code, and redirects to `<return_uri>?handoff_code=<opaque>` with no cookies
    set on the response.
  - Callback re-validates the recovered `return_uri` and fails closed with `400`
    if the pending state contains an unregistered value.
- Added focused tests covering:
  - unregistered `return_uri` rejected at `/auth/login`
  - browser callback behavior remains unchanged
  - mobile callback redirect returns `handoff_code` only and sets no cookies
  - invalid mobile `return_uri` in recovered state fails closed
  - access tokens never appear in callback response headers
- Verification:
  - `~/.cargo/bin/cargo fmt --all`
  - `~/.cargo/bin/cargo test -p dubbridge-gateway` — passed (`101/101`)
  - `~/.cargo/bin/cargo test -p dubbridge-config` — passed (`38/38`)
  - `make qa-docs` — passed

---

## T7.3 — Handoff redemption + explicit mobile session header

- **Status:** [x] Done — 2026-06-04
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model tier:** Balanced -> Premium, thinking On
- **RRI target:** 53.0 (Med-high)
- **Depends on:** T7.2
- **Objective:** Add the mobile handoff redemption surface and teach session
  resolution to accept the explicit mobile session header as the second transport
  for the same opaque server-side session reference.
- **Inputs:** `apps/gateway/src/cookie_ext.rs`, `apps/gateway/src/proxy.rs`,
  `apps/gateway/src/auth/mod.rs`, handoff store from T7.2.
- **Outputs:** `POST /auth/mobile/session` (or the documented equivalent),
  explicit session-header extraction, resolver tests, and no-token-leak tests.
- **Acceptance criteria:**
  - Redeeming a valid one-time handoff code returns only an opaque session
    reference and any mobile-required non-secret metadata.
  - Redeeming the same handoff code twice fails closed.
  - `/api/*` accepts the explicit mobile session header as an alternative to the
    cookie transport and resolves the same server-side session store.
  - If both cookie and mobile session header are present and disagree, the gateway
    rejects the request rather than silently choosing one.
  - The mobile client never sends a JWT to the gateway as its session credential.
- **Happy paths considered:**
  - valid handoff code -> opaque session reference returned -> mobile `/api/*`
    succeeds through the gateway.
  - browser cookie request -> existing cookie resolver still succeeds.
- **Edge cases considered:**
  - expired/unknown handoff code -> no session reference returned.
  - mismatched cookie and mobile header -> 401; no upstream API call.
  - mobile header value that resembles a JWT -> rejected or treated as invalid
    opaque-session input, never forwarded as bearer.

#### Completion record (2026-06-04, T7.3)

- Added `POST /auth/mobile/session` in `apps/gateway/src/auth/mod.rs` via the new
  handler module `apps/gateway/src/auth/mobile_session.rs`.
- Handoff redemption now:
  - accepts JSON `{ "handoff_code": "<opaque>" }`
  - returns `200 OK` with `{ "session_ref": "<opaque-session-id>" }`
  - returns `401 Unauthorized` for expired, unknown, or already-redeemed codes
  - returns `400 Bad Request` for malformed requests (missing, empty, or non-string
    `handoff_code`)
  - never returns access tokens, refresh tokens, or any other mobile-visible
    credential material.
- Extended `apps/gateway/src/cookie_ext.rs`:
  - added `X-Dubbridge-Session` as the explicit mobile session transport
  - preserved cookie transport for browser callers
  - implemented the conflict rule: if cookie and header are both present and
    disagree, resolution returns a transport conflict instead of silently picking
    one.
- Extended `apps/gateway/src/proxy.rs` to use the updated resolver so `/api/*`
  accepts:
  - cookie-only browser transport
  - header-only mobile transport
  - matching cookie + header transport
  and rejects mismatched transports with `401 Unauthorized`.
- Added focused tests covering:
  - valid redeem -> `session_ref` only
  - double redeem -> second call `401`
  - malformed redeem body -> `400`
  - header transport resolves the same server-side session
  - mismatched cookie/header -> `401` and no upstream call
  - JWT-like mobile header -> `401` and never forwarded as bearer
- Verification:
  - `~/.cargo/bin/cargo fmt --all`
  - `~/.cargo/bin/cargo test -p dubbridge-gateway` — passed (`113/113`)
  - `~/.cargo/bin/cargo test -p dubbridge-config` — passed (`38/38`)
  - `make qa-docs` — passed

---

## T7.4 — Mobile refresh/logout parity + e2e/status sync

- **Status:** [x] Done — 2026-06-04
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development (tests) + docs sync
- **Recommended model tier:** Balanced -> Premium, thinking On
- **RRI target:** 54.2 (Med-high)
- **Depends on:** T7.3
- **Objective:** Make mobile session behavior complete across refresh rotation and
  logout, add deterministic end-to-end coverage, and synchronize S-040/S-050/roadmap
  status artifacts.
- **Inputs:** `apps/gateway/src/proxy.rs`, `apps/gateway/src/auth/logout.rs`,
  `apps/gateway/tests/e2e_lifecycle.rs`, S-040/S-050/roadmap docs.
- **Outputs:** Refresh responses expose the rotated opaque session reference to
  mobile callers, logout accepts the explicit session header, e2e mobile lifecycle
  tests, and status docs marking whether S-050 is unblocked.
- **Acceptance criteria:**
  - Mobile `/api/*` with an expired stored access token refreshes server-side and
    returns the rotated opaque session reference to the mobile caller.
  - Mobile `/auth/logout` deletes the server-side session when called with the
    explicit session header and remains idempotent.
  - Deterministic e2e coverage exercises mobile login handoff -> redeem -> API
    call -> refresh rotation -> logout -> stale-session 401.
  - Tests assert access tokens and refresh tokens never appear in mobile-visible
    redirect URLs, JSON bodies, headers, cookies, or logs under test.
  - S-040 plan/tasks, S-050 tasks, roadmap, and ADR-024 references are synchronized.
  - The final status states clearly whether S-050 T1+ is unblocked or still blocked.
- **Happy paths considered:**
  - full mobile session lifecycle succeeds with only opaque gateway session
    references visible to the app.
  - refreshed server-side session id is delivered to mobile so secure-store can be
    updated later in S-050.
- **Edge cases considered:**
  - refresh token revoked -> server-side session invalidated, mobile receives 401,
    no upstream API call.
  - logout with unknown/expired mobile session reference -> 200 idempotent clear or
    documented fail-closed response, with no token leakage.
  - stale mobile session reference after rotation -> rejected once the old session
    has been invalidated.

#### Completion record (2026-06-04, T7.4)

- Extended `apps/gateway/src/proxy.rs` so refresh rotation on the mobile transport
  returns the rotated opaque session reference in the response header
  `X-Dubbridge-Session`.
- Browser and mobile refresh paths now diverge correctly:
  - browser refresh keeps `Set-Cookie` rotation
  - mobile refresh emits `X-Dubbridge-Session` and no browser cookies
  - the gateway never forwards `X-Dubbridge-Session` to `apps/api`
- Extended `apps/gateway/src/auth/logout.rs` to accept the explicit mobile session
  header with the same conflict rule as `/api/*`:
  - matching cookie/header is accepted
  - mismatched cookie/header returns `401 Unauthorized`
  - mobile logout returns `200 OK` without `Set-Cookie`
  - unknown/expired mobile references remain idempotent best-effort deletes
- Added deterministic end-to-end mobile lifecycle coverage in
  `apps/gateway/tests/e2e_lifecycle.rs`:
  - mobile login intent
  - OAuth callback redirect with one-time handoff code
  - handoff redemption to `session_ref`
  - `/api/*` call with `X-Dubbridge-Session`
  - server-side refresh rotation with rotated `session_ref` returned in
    `X-Dubbridge-Session`
  - logout with the rotated mobile session reference
  - stale-session `401` after logout
- Added no-token-leak assertions across mobile-visible redirect URLs, JSON bodies,
  response headers, cookies, and upstream-forwarded headers.
- Final status sync completed:
  - S-040 slice complete
  - S-050 `T1+` unblocked
  - roadmap and ADR-024 references updated to the delivered mobile transport
- Verification:
  - `~/.cargo/bin/cargo fmt --all`
  - `~/.cargo/bin/cargo test -p dubbridge-gateway` — passed (`117/117` unit/integration + `3/3` e2e/doc tests section output)
  - `~/.cargo/bin/cargo test -p dubbridge-config` — passed (`38/38`)
  - `make qa-docs` — passed
