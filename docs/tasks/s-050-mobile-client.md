---
type: TaskList
title: "Tasks: S-050 - First-party Mobile Client (React Native + Expo)"
status: closed
slice: S-050
plan: docs/plan/s-050-mobile-client.md
---
# Tasks: S-050 - First-party Mobile Client (React Native + Expo)

**Plan:** `docs/plan/s-050-mobile-client.md`
**Roadmap phase:** `S-050`. **Hard dependency:** `S-040`
(session gateway / BFF, ADR-024). Benefits from `S-070` (production identity hardening).

**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-024 (primary), ADR-023, ADR-026.
**Stack decision (2026-06-03):** React Native + Expo (managed workflow), TypeScript.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
S-040-T7 (mobile handoff built) -> T0 -> T1 -> T2 -> T3 -> T4 -> T5
```

> **Gate note:** S-040-T7 is now complete (2026-06-04). The mobile-safe session
> handoff / deep-link return contract is live in the gateway, and `T1+` is
> unblocked.

---

## T0 — Gate: confirm S-040 gateway session contract is available

- **Status:** [x] Done — 2026-06-04; gate outcome: initially blocked, now
  unblocked after S-040-T7.4 completion
- **Effort:** S
- **Complexity:** Low
- **Type:** Verification / docs (no app code)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** S-040 built
- **Objective:** Confirm the S-040 gateway exposes a stable, documented session
  contract usable by a native client: login start, callback, logout, authenticated
  proxy, and the transport-agnostic session mechanism defined in S-040 T0. Record the
  exact endpoints, redirect/callback scheme, and session-transport expectations the
  mobile app will rely on.
- **Inputs:** S-040 deliverables (`apps/gateway`, ADR-024 final), this plan.
- **Outputs:** A short "gateway contract for mobile" note appended to this task
  (endpoints, deep-link/redirect scheme, session transport); confirmation that no
  parallel auth path is required.
- **Acceptance criteria:**
  - The login/callback/logout/proxy endpoints and the session transport are
    documented and confirmed reachable.
  - The native redirect/deep-link scheme to return from the system browser to the
    app is decided and recorded.
  - If S-040 is not yet built/stable, this gate **blocks** and the slice does not start.
- **Completion record (2026-06-04):**
  - Verified the implemented S-040 gateway surface in `apps/gateway`:
    - `GET /auth/login` starts Authorization Code + PKCE and redirects to the
      authorization server.
    - `GET /auth/callback?code=...&state=...` validates single-use `state`,
      exchanges the code, creates the server-side session, and returns `200 OK`
      with `Set-Cookie` headers for `dubbridge_session` and
      `dubbridge_session_csrf`.
    - `POST /auth/logout` is idempotent and clears both cookies.
    - `ANY /api/*` resolves the session, refreshes tokens server-side when
      needed, forwards to `apps/api`, and returns `401` with cleared cookies when
      the session is missing, expired, or refresh fails.
  - Verified coverage evidence for the lifecycle above in:
    - `apps/gateway/src/auth/login.rs`
    - `apps/gateway/src/auth/logout.rs`
    - `apps/gateway/src/proxy.rs`
    - `apps/gateway/tests/e2e_lifecycle.rs`
  - Confirmed the currently implemented session transport:
    - ADR-024 defines the canonical session abstraction as an opaque session id.
    - The current gateway implementation exposes that session id only through the
      hardened `dubbridge_session` cookie; `apps/gateway/src/cookie_ext.rs`
      resolves sessions only from the `Cookie` request header.
    - No alternate mobile-readable transport exists today (no response body field,
      no dedicated header contract, no documented session-handoff endpoint).
  - Confirmed the current callback / return-path behavior:
    - The configured OAuth redirect URI is the gateway callback
      (`.../auth/callback`), not a native app deep link.
    - After successful callback, the gateway returns `200 OK` + `Set-Cookie`; it
      does not redirect or bridge back to a native deep-link scheme.
  - **Gateway contract for mobile (current state):**
    - `GET /auth/login`
    - `GET /auth/callback?code=...&state=...`
    - `POST /auth/logout`
    - `ANY /api/*`
    - Session transport currently required by the implementation: browser-style
      `Cookie: dubbridge_session=<opaque-session-id>`. The callback also emits a
      companion `dubbridge_session_csrf` cookie, but the mobile handoff contract
      for that browser-oriented state is not yet defined.
  - **Gate outcome (at T0 completion time):** blocked.
    - S-040 is built and stable for the browser/cookie transport.
    - The mobile-specific deep-link / return scheme and app-readable opaque-session
      handoff required by ADR-024 and this slice are not yet implemented or
      documented.
    - S-040 tracks the unblock as `T7 — Mobile-safe session handoff / deep-link
      return`, decomposed in `docs/tasks/s-040-t7-mobile-session-handoff.md`.
    - T7.1 contract is now defined (2026-06-04): five gateway surfaces named
      (`/auth/login?return_uri`, mobile callback redirect, `POST
      /auth/mobile/session`, `ANY /api/*` + `POST /auth/logout` with
      `X-Dubbridge-Session` header), ADR-024 invariants enumerated, and
      implementation notes for T7.2–T7.4 recorded.
    - T7.2 is now implemented: gateway login accepts validated mobile
      `return_uri`, callback preserves the browser path, and the mobile path
      returns only a one-time opaque `handoff_code` with no cookies set.
    - T7.3 is now implemented: `POST /auth/mobile/session` redeems the handoff
      code into `session_ref`, and `/api/*` accepts `X-Dubbridge-Session` with
      the documented cookie/header conflict rule.
    - T7.4 is now implemented: mobile refresh returns the rotated opaque session
      reference in `X-Dubbridge-Session`, mobile logout accepts the same header,
      and deterministic e2e coverage proves the full mobile lifecycle.
    - **Current gate status:** unblocked. `T1+` may proceed on the delivered S-040
      gateway contract.

---

## T1 — Expo app scaffold (TypeScript) + env-driven gateway config + navigation shell

- **Status:** [x] Done — 2026-06-07
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T0 with gate outcome unblocked after S-040-T7 completion
- **Objective:** Create the `mobile/` Expo React Native TypeScript app with an
  environment-driven gateway base URL (per ADR-026 — no hardcoded URL), a navigation
  shell (authed vs. unauthed trees), and a placeholder home screen.
- **Inputs:** Expo SDK, T0 gateway contract after S-040-T7 completion.
- **Outputs:** `mobile/{package.json, app.config.ts, tsconfig.json, babel.config.js}`,
  `src/config/env.ts`, `src/navigation/`, a placeholder `Home` screen.
- **Acceptance criteria:**
  - `npx expo start` runs; the app boots in a simulator/emulator to the placeholder.
  - The gateway base URL resolves from Expo config per environment; no URL is
    hardcoded in source.
  - Unauthed vs. authed navigation trees exist (authed shows placeholder for now).
- **Happy paths considered:**
  - app launches with a valid env config → renders the unauthed entry screen.
- **Edge cases considered:**
  - missing/empty gateway URL in config → app surfaces a clear configuration error
    rather than silently calling a wrong/default host.
- **Completion record (2026-06-07):**
  - Created the new `mobile/` Expo TypeScript application scaffold with
    `package.json`, `app.config.ts`, `tsconfig.json`, `babel.config.js`, `index.ts`,
    and `App.tsx`.
  - Added environment-driven config resolution in `mobile/src/config/env.ts` using
    Expo `extra` values sourced from `DUBBRIDGE_ENV` and
    `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL` / `DUBBRIDGE_GATEWAY_URL`, with explicit
    fail-clear validation for missing or invalid values.
  - Added the navigation shell in `mobile/src/navigation/RootNavigator.tsx` with
    distinct unauthenticated and authenticated stacks, plus placeholder
    `LoginScreen`, `HomeScreen`, and `ConfigErrorScreen` implementations.
  - Installed Expo SDK 56-compatible runtime/navigation dependencies and a minimal
    Jest/React Native Testing Library harness for task-level verification.

### Happy paths covered

- `HP-1`: valid runtime config renders the unauthenticated entry screen.
  Evidence:
  [RootNavigator](/Users/matiasleandrokruk/Documents/dubbridge/mobile/src/navigation/RootNavigator.tsx),
  [LoginScreen](/Users/matiasleandrokruk/Documents/dubbridge/mobile/src/screens/LoginScreen.tsx),
  [RootNavigator.test.tsx](/Users/matiasleandrokruk/Documents/dubbridge/mobile/__tests__/RootNavigator.test.tsx)
  prove that a valid `dubbridgeEnv` + `gatewayBaseUrl` path resolves config and
  mounts the unauthenticated tree.

### Edge cases covered

- `EC-1`: missing gateway URL surfaces a clear configuration error instead of using
  a default host.
  Evidence:
  [env.ts](/Users/matiasleandrokruk/Documents/dubbridge/mobile/src/config/env.ts),
  [ConfigErrorScreen](/Users/matiasleandrokruk/Documents/dubbridge/mobile/src/screens/ConfigErrorScreen.tsx),
  [RootNavigator.test.tsx](/Users/matiasleandrokruk/Documents/dubbridge/mobile/__tests__/RootNavigator.test.tsx)
  prove that missing config is rejected and rendered as an explicit startup error.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | app launches with valid env config and renders the unauthenticated entry screen | `mobile/__tests__/RootNavigator.test.tsx::renders the unauthenticated entry screen when runtime config is valid` | passed |
| EC-1 | Edge case | missing gateway URL in config surfaces a clear configuration error | `mobile/__tests__/RootNavigator.test.tsx::renders a clear configuration error when the gateway URL is missing` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `npm run typecheck`; `npm test`; `env CI=1 DUBBRIDGE_ENV=local EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://127.0.0.1:4000 npx expo start --offline`

---

## T2 — Gateway API client (typed) + error/session transport handling

- **Status:** [x] Done — 2026-06-07
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T1
- **Objective:** Implement a typed API client that calls the **S-040 gateway** `/api/*`
  proxy (never `apps/api` directly with a raw token), carrying the gateway session
  transport, capturing gateway-owned session rotations, and mapping responses/errors
  (401 → unauthenticated, 403 → forbidden, network/timeout). No raw access/refresh
  token is ever handled by the client.
- **Inputs:** T0 contract, T1 env config.
- **Outputs:** `src/api/client.ts`, typed request/response models, error mapping,
  and a typed session-ref update signal when the gateway returns a rotated
  `X-Dubbridge-Session`.
- **Acceptance criteria:**
  - All authenticated calls go through the gateway; the client holds no JWT.
  - The mobile app has no direct `apps/api` base URL and no direct protected API
    call path.
  - The client forwards the current opaque session reference in
    `X-Dubbridge-Session` and captures a rotated replacement from the same response
    header when the gateway returns one.
  - 401 from the gateway routes the app to re-authenticate; 403 surfaces a
    forbidden state; network errors surface a retryable error state.
  - Client logic is unit-tested against a stubbed gateway (no real network in tests).
- **Happy paths considered:**
  - authenticated GET asset list via gateway → typed list returned and rendered.
  - authenticated gateway response with rotated `X-Dubbridge-Session` → typed client
    result exposes the replacement session reference for secure-store persistence in
    T3.
- **Edge cases considered:**
  - gateway returns 401 (session expired) → client signals re-auth; no crash.
  - request timeout → typed error surfaced, UI can retry.
  - gateway omits rotation header → existing session reference remains unchanged.
  - malformed/empty rotation header → client ignores or rejects it explicitly; it
    never stores a JWT-like value as a session reference.
- **Completion record (2026-06-07):**
  - Created `mobile/src/api/types.ts` with `GatewayErrorKind` (discriminated union),
    `GatewayResponse<T>`, and `GatewayResult<T>` (consistent with the `RuntimeConfigResult`
    pattern from T1's `env.ts`).
  - Created `mobile/src/api/client.ts` with `createGatewayClient({ gatewayBaseUrl, timeoutMs? })`
    factory exposing `get<T>()` and `post<T>()`. Key behaviors: `X-Dubbridge-Session` header
    forwarding; `AbortController` + `setTimeout` for configurable timeout; `finally` block
    ensures timer is always cleared; `extractRotation()` with JWT-pattern guard
    (`/^[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+$/`) rejects JWT-like rotation values.
  - Created `mobile/__tests__/api.client.test.ts` with 10 test cases covering HP-1, HP-2,
    EC-1–EC-4, plus 4 additional cases (403, generic network error, null sessionRef, POST body).

### Happy paths covered

- `HP-1`: `sessionRef` present + 200 response → `{ ok: true, value: { data, sessionRotation: null } }`;
  `X-Dubbridge-Session` header verified in call args.
  Evidence: `mobile/__tests__/api.client.test.ts::HP-1 attaches X-Dubbridge-Session header and returns typed data`
- `HP-2`: response with `X-Dubbridge-Session: new-ref` → `{ ok: true, value: { ..., sessionRotation: 'new-ref' } }`.
  Evidence: `mobile/__tests__/api.client.test.ts::HP-2 captures rotated X-Dubbridge-Session from response headers`

### Edge cases covered

- `EC-1`: 401 → `{ ok: false, error: { kind: 'session_expired' } }`.
  Evidence: `mobile/__tests__/api.client.test.ts::EC-1 returns session_expired error kind on 401`
- `EC-2`: AbortError → `{ ok: false, error: { kind: 'network', message: 'timeout' } }`.
  Evidence: `mobile/__tests__/api.client.test.ts::EC-2 returns network error with "timeout" message on AbortError`
- `EC-3`: missing rotation header → `sessionRotation: null`.
  Evidence: `mobile/__tests__/api.client.test.ts::EC-3 leaves sessionRotation as null when response has no rotation header`
- `EC-4`: JWT-like rotation header → `sessionRotation: null` (guard in `extractRotation()`).
  Evidence: `mobile/__tests__/api.client.test.ts::EC-4 rejects a JWT-looking X-Dubbridge-Session`

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | GET with session ref → typed data + null rotation + header forwarded | `mobile/__tests__/api.client.test.ts::HP-1: authenticated GET returns typed data + null rotation > attaches X-Dubbridge-Session header and returns typed data` | passed |
| HP-2 | Happy path | response rotation header → exposed as sessionRotation | `mobile/__tests__/api.client.test.ts::HP-2: gateway response with rotation header exposes new session ref > captures rotated X-Dubbridge-Session from response headers` | passed |
| EC-1 | Edge case | 401 → session_expired kind | `mobile/__tests__/api.client.test.ts::EC-1: 401 → session_expired > returns session_expired error kind on 401` | passed |
| EC-2 | Edge case | AbortError → network 'timeout' message | `mobile/__tests__/api.client.test.ts::EC-2: AbortError → network timeout > returns network error with "timeout" message on AbortError` | passed |
| EC-3 | Edge case | absent rotation header → sessionRotation null | `mobile/__tests__/api.client.test.ts::EC-3: missing rotation header → sessionRotation null > leaves sessionRotation as null when response has no rotation header` | passed |
| EC-4 | Edge case | JWT-like rotation header rejected | `mobile/__tests__/api.client.test.ts::EC-4: JWT-like rotation header is rejected > rejects a JWT-looking X-Dubbridge-Session, sessionRotation stays null` | passed |

### Owner final verification

- Owner: `Claude Sonnet 4.6`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `npm run typecheck`; `npm test`

---

## T3 — Auth flow (system-browser OAuth via gateway) + secure session storage

- **Status:** [~] In progress (T3a done; T3b decomposed — see below)
- **Effort:** L
- **Complexity:** High
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
  (native OAuth redirect + secure-storage + session-lifecycle reasoning)
- **Depends on:** T2
- **Objective:** Implement login/logout through the device system browser
  (`expo-auth-session` / `expo-web-browser`) against the gateway, establishing the
  gateway session and storing only the opaque session reference in
  `expo-secure-store` (Keychain/Keystore). Persist rotated opaque session references
  emitted by the T2 gateway client. No access/refresh token is ever persisted on
  device (ADR-024).
- **Inputs:** T0 redirect scheme, T2 client, `expo-auth-session`,
  `expo-secure-store`.
- **Outputs:** `src/auth/session.ts`, `src/auth/AuthProvider.tsx`, login/logout
  wired to navigation gating.
- **Acceptance criteria:**
  - Login opens the system browser, completes the gateway OAuth flow, and returns to
    the app via the registered deep-link scheme; a session is established.
  - Only an opaque session reference is stored, in `expo-secure-store` — never a JWT
    or refresh token (asserted by a test).
  - When T2 reports a rotated opaque session reference, the auth/session layer
    atomically replaces the stored reference; stale references are not reused.
  - A 401 from the gateway clears local authenticated state and routes to
    re-authentication rather than trying to renew locally.
  - Logout clears the device session reference and calls the gateway logout.
- **Happy paths considered:**
  - login → system browser → callback deep link → session stored → authed tree shown.
  - gateway rotates session reference during an authenticated call → secure-store is
    updated with the replacement opaque reference.
  - logout → session cleared locally + at gateway → unauthed tree shown.
- **Edge cases considered:**
  - user cancels the system-browser login → app returns to unauthed state cleanly.
  - secure-store assertion: stored value is not a JWT/refresh token.
  - gateway returns 401 for a non-renewable/expired session → local session is
    cleared and the app returns to unauthenticated state.

---

### T3a — Session storage primitives (expo-secure-store, JWT guard, rotation)

- **Status:** [x] Done — 2026-06-07
- **Effort:** S
- **Complexity:** Low
- **Type:** Development
- **Depends on:** T2 (package scaffold and jest harness)
- **Objective:** Implement `mobile/src/auth/session.ts` — pure storage module with
  `saveSessionRef`, `loadSessionRef`, `clearSessionRef`, `updateSessionRef`, and
  `isJwtLike`; add `expo-secure-store` to `package.json`.
- **Files created/modified:**
  - `mobile/src/auth/session.ts` (new)
  - `mobile/__tests__/auth.session.test.ts` (new)
  - `mobile/package.json` — added `"expo-secure-store": "~56.0.4"` to
    `dependencies`; `"jest": "^29.7.0"` hoisted to `devDependencies` (was missing).
- **Completion record (2026-06-07):**
  - Implemented `src/auth/session.ts` with `SESSION_KEY = 'dubbridge_session_ref'`,
    `isJwtLike` (regex `/^[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+$/`),
    and the four async storage primitives. Module has no React or HTTP imports.
  - `updateSessionRef` is a no-op for `null` rotation and for any JWT-like value;
    delegates to `saveSessionRef` otherwise.
  - `expo-secure-store` mocked with `jest.mock(...)` in all tests — no real
    Keychain/Keystore access.
  - `npm test` (all 27 tests pass across 3 suites) and `npm run typecheck` (clean)
    confirmed.

### Happy paths covered

- `HP-1`: `saveSessionRef(ref)` → `setItemAsync('dubbridge_session_ref', ref)` called once.
  Evidence: `__tests__/auth.session.test.ts::saveSessionRef calls setItemAsync with the session key and ref`
- `HP-2`: `loadSessionRef()` returns stored opaque value and value is NOT JWT-like.
  Evidence: `__tests__/auth.session.test.ts::loadSessionRef returns the stored value when present`
  / `loadSessionRef returns a value that is NOT JWT-like`
- `HP-3`: `clearSessionRef()` → `deleteItemAsync('dubbridge_session_ref')` called once.
  Evidence: `__tests__/auth.session.test.ts::clearSessionRef calls deleteItemAsync with the session key`
- `HP-4`: `updateSessionRef(validOpaqueRef)` → `setItemAsync` called with the ref.
  Evidence: `__tests__/auth.session.test.ts::updateSessionRef saves the ref when rotation is a valid opaque reference`

### Edge cases covered

- `EC-1`: `updateSessionRef(null)` → `setItemAsync` NOT called.
  Evidence: `__tests__/auth.session.test.ts::updateSessionRef is a no-op when rotation is null`
- `EC-2`: `updateSessionRef(jwtLike)` → `setItemAsync` NOT called (JWT guard fires).
  Evidence: `__tests__/auth.session.test.ts::updateSessionRef is a no-op when rotation is JWT-like → setItemAsync NOT called`
- `EC-3`: `loadSessionRef()` when key absent → returns `null`.
  Evidence: `__tests__/auth.session.test.ts::loadSessionRef returns null when the key is absent`
- `EC-4`: `isJwtLike` correctly rejects two-segment, four-segment, and empty strings.
  Evidence: `__tests__/auth.session.test.ts::isJwtLike returns false for a two-segment string / four-segment string / empty string`

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `saveSessionRef(ref)` → `setItemAsync` with correct key+value | `auth.session.test.ts::saveSessionRef calls setItemAsync with the session key and ref` | passed |
| HP-2 | Happy path | `loadSessionRef()` returns present value, value not JWT-like | `auth.session.test.ts::loadSessionRef returns the stored value when present` / `returns a value that is NOT JWT-like` | passed |
| HP-3 | Happy path | `clearSessionRef()` → `deleteItemAsync` with correct key | `auth.session.test.ts::clearSessionRef calls deleteItemAsync with the session key` | passed |
| HP-4 | Happy path | `updateSessionRef(opaqueRef)` → saved via `setItemAsync` | `auth.session.test.ts::updateSessionRef saves the ref when rotation is a valid opaque reference` | passed |
| EC-1 | Edge case | `updateSessionRef(null)` → no-op, `setItemAsync` not called | `auth.session.test.ts::updateSessionRef is a no-op when rotation is null` | passed |
| EC-2 | Edge case | `updateSessionRef(jwtLike)` → JWT guard fires, `setItemAsync` not called | `auth.session.test.ts::updateSessionRef is a no-op when rotation is JWT-like → setItemAsync NOT called` | passed |
| EC-3 | Edge case | `loadSessionRef()` absent → returns `null` | `auth.session.test.ts::loadSessionRef returns null when the key is absent` | passed |
| EC-4 | Edge case | `isJwtLike` rejects non-three-segment strings and empty string | `auth.session.test.ts::isJwtLike returns false for ...` (3 cases) | passed |

### Owner final verification

- Owner: `Claude Sonnet 4.6`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior. `npm test` (27 tests, 3 suites — all passed) and `npm run typecheck` (no errors) both ran clean.
- Commands run: `npm test`; `npm run typecheck`

---

### T3b — AuthProvider + OAuth system-browser login/logout wired to navigation

> **Decomposition note (2026-06-07):** T3b was derived from the original T3 scope
> (everything not covered by T3a). Its RRI scored **79 (High, 71–85)**, triggering
> the mandatory decomposition rule (RRI > 70). T3b was split into three subtasks
> before any implementation started.
>
> **Relationship to original T3:** T3 defined the full auth flow output as
> `src/auth/AuthProvider.tsx` + login/logout wired to navigation. T3b-i through
> T3b-iii together deliver that output in three ordered increments:
> T3b-i (AuthProvider state machine) → T3b-ii (login() OAuth flow) →
> T3b-iii (navigation + screen wiring). Every acceptance criterion from the
> original T3 is covered by one of these subtasks.
>
> **Original T3 acceptance criteria mapping:**
>
> | Original T3 criterion | Covered by |
> |---|---|
> | Login opens system browser, returns via deep-link, session established | T3b-ii |
> | Only opaque session ref stored — never JWT/refresh token (test asserted) | T3b-ii |
> | Rotated session ref atomically replaces stored ref | T3b-i |
> | 401 clears local state → re-auth | T3b-i |
> | Logout clears device ref + calls gateway | T3b-i |
> | Login → system browser → deep-link → session stored → authed tree | T3b-ii + T3b-iii |
> | User cancels browser → clean unauthed state | T3b-ii |
> | Stored value not JWT-like (secure-store assertion) | T3b-ii |

#### T3b decomposition RRI summary

| Subtask | Scope | RRI | Band | Gate |
|---|---|---|---|---|
| **T3b-i** | AuthProvider: context + state machine (mount, onSessionRotation, logout) | 70 | Complex | Plan approved + diff review |
| **T3b-ii** | `login()`: expo-web-browser + handoff redemption via T2 | 57 | Complex | Diff review |
| **T3b-iii** | Navigation + screen wiring (RootNavigator, LoginScreen, HomeScreen) | 28 | Moderate | Existing T1 tests pass |

> **Note on RRI floor for auth tasks:** Due to the DubBridge anchor rubric
> (D ≥ 4, P = 5 for auth/credential boundary) plus the mandatory auth penalty
> (+10), any new auth-domain implementation task in this repository will score
> ≥ 56 (Complex). This is by design — auth tasks require human diff review.
> Further subdivision reduces F and C but cannot lower D, P, or the auth penalty.

---

### T3b-i — AuthProvider: context, state machine, mount, onSessionRotation, logout

- **Status:** [~] In progress — decomposed into T3b-i-α (not started) + T3b-i-β (not started)
- **Effort:** M
- **Complexity:** Medium-High
- **RRI:** 70 (Complex, 56–70) → decomposed 2026-06-07
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
- **Depends on:** T3a (done), T2 (done)

> **Decomposition note (2026-06-07):** T3b-i RRI=70 sits at the Complex/High
> boundary. Decomposed to isolate the zero-logic scaffold (T3b-i-α, RRI 9) from
> the state-machine implementation (T3b-i-β, RRI 67). T3b-i-β's RRI 67 is the
> structural minimum for new auth-domain logic in this project (D=4, P=5, +10 auth
> penalty, +10 no-tests penalty). Further subdivision cannot lower it; TDD
> eliminates the no-tests penalty during the implementation phase.
>
> **T3b-i coverage mapping:**
>
> | T3b-i acceptance criterion | Covered by |
> |---|---|
> | `AuthContextValue` type, `AuthContext`, `useAuth()` hook exist | T3b-i-α |
> | Mount with valid ref → `'authed'` | T3b-i-β |
> | Mount with absent ref → `'unauthed'` | T3b-i-β |
> | Mount with JWT-like value → `clearSessionRef()` + `'unauthed'` | T3b-i-β |
> | `onSessionRotation(ref)` → `updateSessionRef` + in-memory update | T3b-i-β |
> | `onSessionRotation(null)` → no-op | T3b-i-β |
> | `logout()` → `clearSessionRef()` + `'unauthed'` + best-effort gateway | T3b-i-β |
> | `logout()` gateway error → local clear regardless (fail-safe) | T3b-i-β |

---

#### T3b-i-α — AuthContext foundation: types + scaffold + package.json

- **Status:** [x] Done — 2026-06-07
- **Effort:** S
- **Complexity:** Low
- **RRI:** 9 (Low, 0–25)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6`
- **Depends on:** T3a, T2 (installed)
- **Objective:** Install packages and create the `AuthProvider.tsx` shell —
  types, context creation, `useAuth()` hook, and a `children`-accepting
  `AuthProvider` component with a stub value. Zero state logic, zero async,
  zero side effects. This is the compile target for T3b-i-β.
- **Files:**
  - `mobile/package.json` (modified — add `expo-web-browser ~14.1.0`,
    `expo-auth-session ~6.1.0`)
  - `mobile/src/auth/AuthProvider.tsx` (new — types + scaffold only)
- **Acceptance criteria:**
  - `AuthContextValue` type exported: `{ sessionRef, status, loginError, login, logout, onSessionRotation }`.
  - `AuthContext` created with `createContext`.
  - `useAuth()` hook exported — throws if used outside provider.
  - `AuthProvider` component accepts `children` and renders `<AuthContext.Provider>`.
  - `npm run typecheck` passes clean.
  - No state logic, no `useEffect`, no calls to `session.ts` or `client.ts` in this file at this stage.

**RRI — T3b-i-α:**

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | Zero branches — types and context creation only | High |
| F files | 1 | 2 files: `package.json`, `AuthProvider.tsx` | High |
| D domain | 1 | Types + React infrastructure — no domain logic | High |
| T coverage | 0 | No testable logic in this phase | High |
| A ambiguity | 0 | Mechanical — shapes fully defined | High |
| K coupling | 0 | No side effects — pure declaration | High |
| P impact | 1 | Defines the contract but does not implement the security boundary | High |
| X context | 1 | Only `AuthProvider.tsx` in scope | High |

```
Base = 100 × ((0+0.12×1+0.15×1+0+0+0+0.10×1+0.06×1) / 5)
     = 100 × (0.43 / 5) = 8.6
Penalties: none — no auth logic implemented yet
Final RRI = 9 → Low (0–25)
```

Gate: auto-execute (Low band). Present RRI + one-line intent, then implement.

- **Completion record (2026-06-07):**
  - Added `expo-auth-session ~6.1.0` and `expo-web-browser ~14.1.0` to
    `mobile/package.json` and refreshed `mobile/package-lock.json` with
    `npm install`.
  - Created `mobile/src/auth/AuthProvider.tsx` as a zero-logic scaffold exporting
    `AuthStatus`, `AuthContextValue`, `AuthContext`, `useAuth()`, and
    `AuthProvider`.
  - `useAuth()` throws when used outside `AuthProvider`; the provider currently
    exposes only the stubbed `loading` value required by this phase.
  - Verified `AuthProvider.tsx` has no state logic, no `useEffect`, and no imports
    from `session.ts` or `client.ts`.
  - `npm run typecheck` passes clean.

### Happy paths covered

- `HP-1`: The auth scaffold exports the required context contract and provider shell.
  Evidence: `mobile/src/auth/AuthProvider.tsx` exports `AuthStatus`,
  `AuthContextValue`, `AuthContext`, `useAuth()`, and `AuthProvider`.

### Edge cases covered

- `EC-1`: `useAuth()` fails closed outside the provider boundary.
  Evidence: `mobile/src/auth/AuthProvider.tsx` throws
  `"useAuth must be used within an AuthProvider"` when the context value is
  `undefined`.
- `EC-2`: This phase does not couple to session persistence or gateway client logic.
  Evidence: `mobile/src/auth/AuthProvider.tsx` imports only React context types and
  helpers; there are no imports from `session.ts`, `client.ts`, or any effect/state
  hooks.

### Owner final verification

- Owner: `Codex GPT-5`
- Date: `2026-06-07`
- Statement: I verified this scaffold task remains compile-only and logic-free, with
  the required provider contract exported and no coupling to session or gateway
  logic.
- Commands run: `npm install`; `npm run typecheck`

---

#### T3b-i-β — State machine: mount + onSessionRotation + logout + TDD tests

- **Status:** [x] Done — 2026-06-07
- **Effort:** M
- **Complexity:** Medium-High
- **RRI:** 67 (Complex, 56–70)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
- **Depends on:** T3b-i-α
- **Objective:** Replace the stub body from T3b-i-α with the real state machine.
  TDD: write `auth.provider.test.tsx` first, then implement mount (`loadSessionRef`
  + `isJwtLike` guard), `onSessionRotation` (`updateSessionRef`), and `logout()`
  (local `clearSessionRef` + best-effort `client.post('/auth/logout', ...)`).
  `login()` remains a stub (implemented in T3b-ii).
- **Files:**
  - `mobile/__tests__/auth.provider.test.tsx` (new — TDD tests written first)
  - `mobile/src/auth/AuthProvider.tsx` (modified — add state machine logic)
- **Acceptance criteria:** (inherited from T3b-i parent, state-machine cases only)
  - Mount with valid `session_ref` → status `'authed'`, `sessionRef` in state.
  - Mount with absent ref → status `'unauthed'`.
  - Mount with JWT-like value in secure-store → `clearSessionRef()` + status `'unauthed'`.
  - `onSessionRotation(ref)` → `updateSessionRef(ref)` + `state.sessionRef = ref`.
  - `onSessionRotation(null)` → no-op.
  - `logout()` → `clearSessionRef()` + status `'unauthed'` + best-effort gateway call.
  - `logout()` gateway error → local session cleared regardless.
  - `npm test` and `npm run typecheck` pass clean.

**RRI — T3b-i-β:**

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | mount: 2 branches, `onSessionRotation`: 1, `logout`: 1 → CC=4 → band 1–5 | High |
| F files | 1 | 2 files: `AuthProvider.tsx` (mod), `auth.provider.test.tsx` (new) | High |
| D domain | 4 | Anchor: credential storage + auth/authorization system boundary | High |
| T coverage | 4 | No tests for state machine yet — T3b-i-α only creates scaffold | High |
| A ambiguity | 0 | Acceptance criteria exact + TDD approach defined | High |
| K coupling | 3 | T3a storage (Keychain) + T2 HTTP client + React async effects | High |
| P impact | 5 | Anchor: authentication/authorization system boundary | High |
| X context | 3 | `session.ts`, `client.ts`, `AuthContext` (T3b-i-α), ADR-024 | Medium |

```
Base = 100 × ((0.18×0+0.12×1+0.15×4+0.15×4+0.12×0+0.12×3+0.10×5+0.06×3) / 5)
     = 100 × (2.36 / 5) = 47.2
Penalties: +10 (auth boundary) + +10 (no tests + P=5) = +20
Final RRI = 67 → Complex (56–70)
```

Note: the +10 "no tests" penalty is eliminated mid-task once TDD tests are
written (before implementing the logic). Effective implementation-phase RRI ≈ 57.

Gate: human reviews diff after implementation.

- **Completion record (2026-06-07):**
  - Added `mobile/__tests__/auth.provider.test.tsx` first and used it as the TDD
    driver for mount hydration, session rotation, and logout behavior.
  - Replaced the `AuthProvider` stub with stateful auth context logic in
    `mobile/src/auth/AuthProvider.tsx`: mount-time secure-store hydration,
    JWT-like session rejection + local clear, persisted session rotation, and
    fail-safe logout that clears local auth state before a best-effort gateway
    logout call.
  - `login()` remains a stub exactly as scoped for this task; OAuth browser flow
    is still deferred to `T3b-ii`.
  - Verified with `npm test` and `npm run typecheck` — both pass clean.

### Happy paths covered

- `HP-1`: valid stored opaque session boots into `authed` with `sessionRef` in memory.
  Evidence: `mobile/src/auth/AuthProvider.tsx` hydrates from `loadSessionRef()` and
  sets `status='authed'` for non-JWT values; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-i-β HP-1: valid stored opaque session boots into authed state > loads a stored ref and exposes authed state`.
- `HP-2`: `onSessionRotation(ref)` persists the rotated opaque ref and updates in-memory state.
  Evidence: `mobile/src/auth/AuthProvider.tsx::onSessionRotation` calls
  `updateSessionRef(rotation)` and then sets `sessionRef/status`; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-i-β HP-2: onSessionRotation persists and updates in-memory state > stores the rotated ref and updates the current sessionRef`.
- `HP-3`: `logout()` clears local auth state and makes a best-effort gateway logout call.
  Evidence: `mobile/src/auth/AuthProvider.tsx::logout` clears local state, then
  calls `client.post('/auth/logout', previousSessionRef, {})`; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-i-β HP-3: logout clears local auth state and calls gateway logout > clears the session ref and makes a best-effort gateway logout call`.

### Edge cases covered

- `EC-1`: absent stored session resolves to `unauthed`.
  Evidence: `mobile/src/auth/AuthProvider.tsx` sets `status='unauthed'` when
  `loadSessionRef()` returns `null`; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-1: absent stored session boots into unauthed state > becomes unauthed when no session ref is stored`.
- `EC-2`: JWT-like stored value is cleared and rejected.
  Evidence: `mobile/src/auth/AuthProvider.tsx` calls `clearSessionRef()` and stays
  `unauthed` when `isJwtLike(storedSessionRef)` is true; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-2: JWT-like stored value is cleared and rejected > clears the stored value and becomes unauthed`.
- `EC-3`: `onSessionRotation(null)` is a no-op.
  Evidence: `mobile/src/auth/AuthProvider.tsx::onSessionRotation` returns early on
  `null`; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-3: onSessionRotation(null) is a no-op > does not persist or alter the current session`.
- `EC-4`: gateway logout failure still leaves local state cleared.
  Evidence: `mobile/src/auth/AuthProvider.tsx::logout` clears local state before
  awaiting the gateway result; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-4: gateway logout failure still leaves local auth state cleared > fails safe to unauthed state even when the gateway logout result is not ok`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid stored opaque session → `authed` + `sessionRef` in state | `mobile/__tests__/auth.provider.test.tsx::T3b-i-β HP-1: valid stored opaque session boots into authed state > loads a stored ref and exposes authed state` | passed |
| HP-2 | Happy path | `onSessionRotation(ref)` persists rotation and updates in-memory state | `mobile/__tests__/auth.provider.test.tsx::T3b-i-β HP-2: onSessionRotation persists and updates in-memory state > stores the rotated ref and updates the current sessionRef` | passed |
| HP-3 | Happy path | `logout()` clears local state and calls gateway logout | `mobile/__tests__/auth.provider.test.tsx::T3b-i-β HP-3: logout clears local auth state and calls gateway logout > clears the session ref and makes a best-effort gateway logout call` | passed |
| EC-1 | Edge case | absent stored session → `unauthed` | `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-1: absent stored session boots into unauthed state > becomes unauthed when no session ref is stored` | passed |
| EC-2 | Edge case | JWT-like stored value is cleared and rejected | `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-2: JWT-like stored value is cleared and rejected > clears the stored value and becomes unauthed` | passed |
| EC-3 | Edge case | `onSessionRotation(null)` is a no-op | `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-3: onSessionRotation(null) is a no-op > does not persist or alter the current session` | passed |
| EC-4 | Edge case | gateway logout failure still leaves local state cleared | `mobile/__tests__/auth.provider.test.tsx::T3b-i-β EC-4: gateway logout failure still leaves local auth state cleared > fails safe to unauthed state even when the gateway logout result is not ok` | passed |

### Owner final verification

- Owner: `Codex GPT-5`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior, and the full mobile
  test suite plus TypeScript compilation both ran clean after the implementation.
- Commands run: `npm test`; `npm run typecheck`

---

### T3b-ii — login(): expo-web-browser + handoff code redemption

- **Status:** [x] Done — 2026-06-07
- **Effort:** M
- **Complexity:** Medium-High
- **RRI:** 57 (Complex, 56–70)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
- **Depends on:** T3b-i
- **Objective:** Replace the `login()` stub from T3b-i with the real OAuth flow:
  `makeRedirectUri` → `openAuthSessionAsync` → extract `handoff_code` from
  deep-link URL → `POST /auth/mobile/session` → `saveSessionRef` → status
  `'authed'`. Tests added to existing `auth.provider.test.tsx`.
- **Files:**
  - `mobile/src/auth/AuthProvider.tsx` (modified — replace login stub)
  - `mobile/__tests__/auth.provider.test.tsx` (modified — add login tests)
- **Gateway contracts (verified in Rust source):**
  - `redirectUri` = `makeRedirectUri({ scheme: 'dubbridge', path: 'auth/callback' })` → `"dubbridge://auth/callback"`
  - `loginUrl` = `${gatewayBaseUrl}/auth/login?return_uri=dubbridge%3A%2F%2Fauth%2Fcallback`
  - Callback returns `dubbridge://auth/callback?handoff_code=<43-char-base64url>` — single param, TTL 90s, single-use
  - `POST /auth/mobile/session { "handoff_code": "..." }` → `200 { "session_ref": "..." }` (only field)
- **Acceptance criteria:**
  - `login()` opens system browser to correct URL with `return_uri`.
  - Browser cancel (`type !== 'success'`) → no-op, status stays `'unauthed'`.
  - `handoff_code` absent in callback URL → `loginError = 'missing_handoff_code'`.
  - `POST /auth/mobile/session` returns 401 → `loginError = 'session_expired'`.
  - Full success → `saveSessionRef` called + status `'authed'`.
  - Stored value is NOT JWT-like (explicit test assertion).

#### RRI — T3b-ii

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | `type!='success'`=1, `!handoffCode`=1, `!response.ok`=1 + URL parse → CC≈6 → band 6–10 | High |
| F files | 1 | 2 files: `AuthProvider.tsx` (mod), `auth.provider.test.tsx` (mod) | High |
| D domain | 4 | OAuth redirect + platform async browser + auth domain | High |
| T coverage | 1 | Tests from T3b-i cover the area; login tests added incrementally | High |
| A ambiguity | 0 | Gateway contracts verified in Rust, exact shapes documented | High |
| K coupling | 4 | `expo-web-browser` (platform async), T2 HTTP client (external), `expo-auth-session` | High |
| P impact | 5 | Auth boundary — login controls entry to authenticated state | High |
| X context | 3 | `AuthProvider.tsx`, `session.ts`, `client.ts`, gateway Rust contracts | Medium |

```
Base = 100 × ((0.18×1 + 0.12×1 + 0.15×4 + 0.15×1 + 0.12×0 + 0.12×4 + 0.10×5 + 0.06×3) / 5)
     = 100 × (2.33 / 5) = 46.6
Penalties: +10 (auth boundary)
No "no tests + P≥4": T=1 (tests from T3b-i exist in area)
Final RRI = 57 → Complex (56–70)
```

Gate: human reviews diff.

- **Completion record (2026-06-07):**
  - Extended `mobile/__tests__/auth.provider.test.tsx` first with login-flow TDD
    for success, browser cancel, missing `handoff_code`, `401` redemption, and
    JWT-like `session_ref` rejection.
  - Replaced the `login()` stub in `mobile/src/auth/AuthProvider.tsx` with the
    real mobile handoff flow: `makeRedirectUri({ scheme: 'dubbridge', path:
    'auth/callback' })`, `openAuthSessionAsync(loginUrl, redirectUri)`,
    `POST /auth/mobile/session`, `saveSessionRef(session_ref)`, and transition to
    `status='authed'` on success.
  - Added a fail-closed guard that rejects any gateway-returned `session_ref`
    matching JWT structure before persistence.
  - Verified with `npm test` and `npm run typecheck` — both pass clean.

### Happy paths covered

- `HP-1`: successful browser login returns a `handoff_code`, redeems it, persists
  the opaque `session_ref`, and transitions to `authed`.
  Evidence: `mobile/src/auth/AuthProvider.tsx::login` builds the redirect URI,
  opens the auth session, redeems `/auth/mobile/session`, persists via
  `saveSessionRef`, and updates `sessionRef/status`; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii HP-1 + HP-2: login redeems a handoff code and persists an opaque session > opens the system browser, redeems the code, and becomes authed`.
- `HP-2`: the persisted mobile credential remains opaque and not JWT-like.
  Evidence: `mobile/src/auth/AuthProvider.tsx::login` rejects JWT-like
  `session_ref` values before persistence; explicit stored-value assertion in
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii HP-1 + HP-2: login redeems a handoff code and persists an opaque session > opens the system browser, redeems the code, and becomes authed`
  (`saveSessionRef` argument does not contain JWT segment separators).

### Edge cases covered

- `EC-1`: browser cancel leaves the state `unauthed` and does not redeem or persist anything.
  Evidence: `mobile/src/auth/AuthProvider.tsx::login` returns early when
  `result.type !== 'success'`; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-1: browser cancel leaves auth state unchanged > stays unauthed when the auth session result is not success`.
- `EC-2`: callback without `handoff_code` fails clearly and does not authenticate.
  Evidence: `mobile/src/auth/AuthProvider.tsx::login` sets
  `loginError='missing_handoff_code'` when the callback URL lacks the query param;
  unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-2: missing handoff code surfaces a clear login error > sets loginError to missing_handoff_code and does not authenticate`.
- `EC-3`: `401` from `/auth/mobile/session` maps to `loginError='session_expired'`.
  Evidence: `mobile/src/auth/AuthProvider.tsx::login` maps `redeemResult.error.kind`
  to `session_expired`; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-3: 401 from handoff redemption maps to session_expired > does not authenticate and stores a session_expired loginError`.
- `EC-4`: gateway returning a JWT-like `session_ref` is rejected and not persisted.
  Evidence: `mobile/src/auth/AuthProvider.tsx::login` sets
  `loginError='invalid_session_ref'` and skips `saveSessionRef` for JWT-like
  values; unit proof in
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-4: JWT-like session_ref from gateway is rejected > does not persist a JWT-looking session ref`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | browser success + valid handoff code + gateway redemption → persisted session + `authed` | `mobile/__tests__/auth.provider.test.tsx::T3b-ii HP-1 + HP-2: login redeems a handoff code and persists an opaque session > opens the system browser, redeems the code, and becomes authed` | passed |
| HP-2 | Happy path | persisted login credential is opaque and not JWT-like | `mobile/__tests__/auth.provider.test.tsx::T3b-ii HP-1 + HP-2: login redeems a handoff code and persists an opaque session > opens the system browser, redeems the code, and becomes authed` | passed |
| EC-1 | Edge case | browser cancel leaves auth state unchanged | `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-1: browser cancel leaves auth state unchanged > stays unauthed when the auth session result is not success` | passed |
| EC-2 | Edge case | callback missing `handoff_code` → explicit login error | `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-2: missing handoff code surfaces a clear login error > sets loginError to missing_handoff_code and does not authenticate` | passed |
| EC-3 | Edge case | gateway `401` on handoff redemption → `session_expired` | `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-3: 401 from handoff redemption maps to session_expired > does not authenticate and stores a session_expired loginError` | passed |
| EC-4 | Edge case | JWT-like `session_ref` from gateway is rejected | `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-4: JWT-like session_ref from gateway is rejected > does not persist a JWT-looking session ref` | passed |

### Owner final verification

- Owner: `Codex GPT-5`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior, and the full mobile
  test suite plus TypeScript compilation both ran clean after the implementation.
- Commands run: `npm test`; `npm run typecheck`

---

### T3b-iii — Navigation + screen wiring

- **Status:** [x] Done — 2026-06-07
- **Effort:** S
- **Complexity:** Low
- **RRI:** 28 (Moderate, 26–40)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4.6`
- **Depends on:** T3b-ii
- **Objective:** Wire `AuthProvider` into the navigation tree and replace the
  placeholder callbacks from T1. `RootNavigator.tsx` wraps with `<AuthProvider>`
  and gates on `AuthContext.status`. `LoginScreen.tsx` calls `auth.login()`.
  `HomeScreen.tsx` calls `auth.logout()`. Existing T1 tests updated for new API.
- **Files:**
  - `mobile/src/navigation/RootNavigator.tsx` (modified)
  - `mobile/src/screens/LoginScreen.tsx` (modified)
  - `mobile/src/screens/HomeScreen.tsx` (modified)
- **Acceptance criteria:**
  - `RootNavigator` shows unauthed tree when `status === 'unauthed'` or `'loading'`.
  - `RootNavigator` shows authed tree when `status === 'authed'`.
  - `LoginScreen` calls `auth.login()` on button press — no `onContinue` placeholder.
  - `HomeScreen` calls `auth.logout()` on sign-out — no `onSignOut` placeholder.
  - Existing `RootNavigator.test.tsx` tests updated and passing.
  - `npm test` and `npm run typecheck` pass clean.

#### RRI — T3b-iii

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | `status==='authed'` branch=1, config error check=1 → CC=2 → band 1–5 | High |
| F files | 2 | 3 files: `RootNavigator.tsx`, `LoginScreen.tsx`, `HomeScreen.tsx` | High |
| D domain | 3 | Navigation + state management; no auth logic in this layer | High |
| T coverage | 1 | `RootNavigator.test.tsx` exists from T1 — updated, not replaced | High |
| A ambiguity | 0 | Mechanical wiring — inputs and outputs fully defined | High |
| K coupling | 2 | React context consumption + navigation; no external side effects | High |
| P impact | 2 | UI routing only; access control lives in `AuthProvider` (T3b-i/ii) | High |
| X context | 2 | `AuthProvider.tsx`, `RootNavigator.tsx`, screens | High |

```
Base = 100 × ((0.18×0 + 0.12×2 + 0.15×3 + 0.15×1 + 0.12×0 + 0.12×2 + 0.10×2 + 0.06×2) / 5)
     = 100 × (1.40 / 5) = 28.0
Penalties: none
Final RRI = 28 → Moderate (26–40)
```

Gate: confirm existing T1 tests pass with new context API.

- **Completion record (2026-06-07):**
  - Wrapped `RootNavigator` with `AuthProvider` and replaced the local
    `useState(false)` placeholder gate with `useAuth().status`.
  - Removed the placeholder `onContinue` / `onSignOut` props from
    `LoginScreen.tsx` and `HomeScreen.tsx`; both screens now invoke
    `auth.login()` / `auth.logout()` directly through the auth context.
  - Updated `mobile/__tests__/RootNavigator.test.tsx` to cover unauthenticated,
    loading, authenticated, and invalid-config rendering, plus button wiring for
    `auth.login()` and `auth.logout()`.
  - Verified with `npm test` and `npm run typecheck` — both pass clean.

### Happy paths covered

- `HP-1`: `status='authed'` renders the authenticated tree.
  Evidence: `mobile/src/navigation/RootNavigator.tsx` branches to
  `AuthedNavigator` when `auth.status === 'authed'`; unit proof in
  `mobile/__tests__/RootNavigator.test.tsx::renders the authenticated home screen when auth status is authed`.
- `HP-2`: `status='unauthed'` renders the login entry tree.
  Evidence: `mobile/src/navigation/RootNavigator.tsx` falls back to
  `UnauthedNavigator` for non-`authed` status; unit proof in
  `mobile/__tests__/RootNavigator.test.tsx::renders the unauthenticated entry screen when runtime config is valid`.
- `HP-3`: UI button actions call the auth context methods directly.
  Evidence: `mobile/src/screens/LoginScreen.tsx` calls `auth.login()` and
  `mobile/src/screens/HomeScreen.tsx` calls `auth.logout()`; unit proof in
  `mobile/__tests__/RootNavigator.test.tsx::wires the login screen button to auth.login()` and
  `mobile/__tests__/RootNavigator.test.tsx::wires the home screen button to auth.logout()`.

### Edge cases covered

- `EC-1`: `status='loading'` does not expose the authenticated tree.
  Evidence: `mobile/src/navigation/RootNavigator.tsx` renders the unauthenticated
  branch for any non-`authed` status; unit proof in
  `mobile/__tests__/RootNavigator.test.tsx::renders the unauthenticated entry screen while auth is loading`.
- `EC-2`: invalid runtime config still renders `ConfigErrorScreen`.
  Evidence: `mobile/src/navigation/RootNavigator.tsx` checks `readRuntimeConfig()`
  before navigation rendering; unit proof in
  `mobile/__tests__/RootNavigator.test.tsx::renders a clear configuration error when the gateway URL is missing`.
- `EC-3`: the UI no longer depends on placeholder callback props parallel to the context.
  Evidence: `mobile/src/screens/LoginScreen.tsx` and `mobile/src/screens/HomeScreen.tsx`
  no longer accept placeholder action props and use `useAuth()` instead; call
  wiring is asserted in
  `mobile/__tests__/RootNavigator.test.tsx::wires the login screen button to auth.login()` and
  `mobile/__tests__/RootNavigator.test.tsx::wires the home screen button to auth.logout()`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `status='authed'` renders the authenticated tree | `mobile/__tests__/RootNavigator.test.tsx::renders the authenticated home screen when auth status is authed` | passed |
| HP-2 | Happy path | `status='unauthed'` renders the login entry tree | `mobile/__tests__/RootNavigator.test.tsx::renders the unauthenticated entry screen when runtime config is valid` | passed |
| HP-3 | Happy path | UI actions call `auth.login()` / `auth.logout()` | `mobile/__tests__/RootNavigator.test.tsx::wires the login screen button to auth.login()` / `mobile/__tests__/RootNavigator.test.tsx::wires the home screen button to auth.logout()` | passed |
| EC-1 | Edge case | `status='loading'` keeps the app on the unauthenticated tree | `mobile/__tests__/RootNavigator.test.tsx::renders the unauthenticated entry screen while auth is loading` | passed |
| EC-2 | Edge case | invalid runtime config renders `ConfigErrorScreen` | `mobile/__tests__/RootNavigator.test.tsx::renders a clear configuration error when the gateway URL is missing` | passed |
| EC-3 | Edge case | no placeholder callback path remains parallel to the context wiring | `mobile/__tests__/RootNavigator.test.tsx::wires the login screen button to auth.login()` / `mobile/__tests__/RootNavigator.test.tsx::wires the home screen button to auth.logout()` | passed |

### Owner final verification

- Owner: `Codex GPT-5`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior, and the full mobile
  test suite plus TypeScript compilation both ran clean after the implementation.
- Commands run: `npm test`; `npm run typecheck`

---

## T4 — Core screens (Login, Home, AssetList, AssetDetail)

- **Status:** [x] Done — 2026-06-07
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T3
- **Objective:** Build the core authenticated screens against the gateway client:
  Login entry, authenticated Home, AssetList (from S-010 asset/ingestion state), and
  AssetDetail / ingestion status. Handle loading/empty/error states and degrade
  gracefully where S-120–S-180 backend surfaces do not yet exist.
- **Inputs:** T2 client, T3 auth context.
- **Outputs:** `src/screens/{Login,Home,AssetList,AssetDetail}.tsx`.
- **Acceptance criteria:**
  - Authenticated screens render real data fetched through the gateway (no mocked
    backend data; only test doubles in tests).
  - Loading, empty, and error states are handled on each data screen.
  - Screens that would depend on unbuilt slices (S-120–S-180) show a clear
    "not available yet" state instead of failing.
- **Happy paths considered:**
  - `HP-1`: authed user opens `AssetList` → assets load via gateway → tapping one
    opens `AssetDetail`.
  - `HP-2`: authed user opens `AssetDetail` → available S-010 asset summary and
    ingestion status render successfully.
- **Edge cases considered:**
  - `EC-1`: empty asset list → friendly empty state.
  - `EC-2`: gateway/network failure on asset loading → clear error state.
  - `EC-3`: unavailable mobile asset surfaces or downstream S-120–S-180 product data →
    explicit `not available yet` state.

- **Completion record (2026-06-07):**
  - Added `mobile/src/screens/AssetListScreen.tsx` with gateway-backed asset-list
    loading, empty/error handling, and a fail-clear `404 => not available yet`
    branch for the currently unshipped mobile list endpoint.
  - Added `mobile/src/screens/AssetDetailScreen.tsx` with gateway-backed S-010 asset
    detail rendering and an explicit downstream `not available yet` panel for
    S-120–S-180 product surfaces that do not exist yet.
  - Updated `mobile/src/screens/HomeScreen.tsx` to expose the authenticated asset
    entry point and aligned `mobile/src/screens/LoginScreen.tsx` copy with the
    real gateway login flow.
  - Updated `mobile/src/navigation/RootNavigator.tsx` so the authenticated stack
    now flows `Home -> AssetList -> AssetDetail`.
  - Added `mobile/__tests__/asset.screens.test.tsx` to cover T4 happy/edge cases.
  - Verified with `npm test` and `npm run typecheck` — both pass clean.

### Happy paths covered

- `HP-1`: authenticated user opens `AssetList`, real gateway data loads, and a
  selected asset navigates to detail.
  Evidence: `mobile/src/screens/AssetListScreen.tsx` calls
  `GET /api/assets?view=mobile`, renders cards from gateway data, and emits the
  selected asset through `onOpenAsset`; unit proof in
  `mobile/__tests__/asset.screens.test.tsx::T4 HP-1: authenticated user opens AssetList and assets render > loads asset list data through the gateway and opens an asset`.
- `HP-2`: authenticated user opens `AssetDetail` and sees the available S-010 asset
  summary and ingestion status.
  Evidence: `mobile/src/screens/AssetDetailScreen.tsx` calls
  `GET /api/assets/{id}` and renders title/status/uploader metadata; unit proof
  in
  `mobile/__tests__/asset.screens.test.tsx::T4 HP-2: authenticated user opens an asset and sees detail/status > loads asset detail and shows the available S-010 summary`.

### Edge cases covered

- `EC-1`: empty asset list renders a friendly empty state.
  Evidence: `mobile/src/screens/AssetListScreen.tsx` detects `data.length === 0`
  and renders `No assets yet`; unit proof in
  `mobile/__tests__/asset.screens.test.tsx::T4 EC-1: empty asset list renders a friendly empty state > shows an empty state when the gateway returns no assets`.
- `EC-2`: gateway/network failure renders a clear error state.
  Evidence: `mobile/src/screens/AssetListScreen.tsx` maps gateway errors into a
  dedicated error panel; unit proof in
  `mobile/__tests__/asset.screens.test.tsx::T4 EC-2: gateway or network failure renders a clear error state > shows an error state when the gateway request fails`.
- `EC-3`: unavailable mobile surfaces or downstream S-120–S-180 data render explicit
  `not available yet` messaging.
  Evidence: `mobile/src/screens/AssetListScreen.tsx` maps `404` to `Asset list not available yet`,
  and `mobile/src/screens/AssetDetailScreen.tsx` renders a downstream
  `Not available yet` panel even on successful S-010 detail load; unit proof in
  `mobile/__tests__/asset.screens.test.tsx::T4 EC-3: unavailable surfaces render not available yet > shows a not-available state when the mobile asset list endpoint is not live`
  and
  `mobile/__tests__/asset.screens.test.tsx::T4 HP-2: authenticated user opens an asset and sees detail/status > loads asset detail and shows the available S-010 summary`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | `AssetList` loads gateway data and selected asset opens detail | `mobile/__tests__/asset.screens.test.tsx::T4 HP-1: authenticated user opens AssetList and assets render > loads asset list data through the gateway and opens an asset` | passed |
| HP-2 | Happy path | `AssetDetail` renders the available S-010 asset summary and status | `mobile/__tests__/asset.screens.test.tsx::T4 HP-2: authenticated user opens an asset and sees detail/status > loads asset detail and shows the available S-010 summary` | passed |
| EC-1 | Edge case | empty asset list → friendly empty state | `mobile/__tests__/asset.screens.test.tsx::T4 EC-1: empty asset list renders a friendly empty state > shows an empty state when the gateway returns no assets` | passed |
| EC-2 | Edge case | gateway/network failure → clear error state | `mobile/__tests__/asset.screens.test.tsx::T4 EC-2: gateway or network failure renders a clear error state > shows an error state when the gateway request fails` | passed |
| EC-3 | Edge case | unavailable mobile/downstream surfaces → explicit `not available yet` state | `mobile/__tests__/asset.screens.test.tsx::T4 EC-3: unavailable surfaces render not available yet > shows a not-available state when the mobile asset list endpoint is not live` / `mobile/__tests__/asset.screens.test.tsx::T4 HP-2: authenticated user opens an asset and sees detail/status > loads asset detail and shows the available S-010 summary` | passed |

### Owner final verification

- Owner: `Codex GPT-5`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior, and the full mobile
  test suite plus TypeScript compilation both ran clean after the implementation.
- Commands run: `npm test`; `npm run typecheck`

---

## T5 — Tests (unit + component + auth-flow) + docs/roadmap sync

- **Status:** [x] Done — 2026-06-07
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development (tests) + docs sync
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T4
- **Objective:** Add unit tests (API client, session logic), component tests
  (core screens via React Native Testing Library), and an auth-flow integration test
  against a stubbed gateway. Synchronize status artifacts.
- **Inputs:** all prior tasks; a stubbed gateway harness.
- **Outputs:** `mobile/__tests__/*`; updated `docs/architecture.md` (mobile in
  first-party client surfaces), `docs/plan/roadmap.md` (S-050 status), ADR-024 mobile
  reference confirmed.
- **Acceptance criteria:**
  - Unit + component + auth-flow tests pass deterministically (no real network).
  - A test asserts no JWT/refresh token is ever stored on device or exposed to UI.
  - `docs/architecture.md` and `docs/plan/roadmap.md` reflect S-050 as built.
- **Happy paths considered:**
  - `HP-1`: full mobile login → asset list → detail flow stays green against the
    stubbed gateway.
  - `HP-2`: token-safety guards remain green while the authenticated mobile flow
    runs; only opaque session references are accepted for persistence.
- **Edge cases considered:**
  - `EC-1`: JWT-like or raw token values are never persisted to device storage.
  - `EC-2`: JWT-like or raw token values are never exposed in the UI during the
    mobile auth flow.

- **Completion record (2026-06-07):**
  - Added `mobile/__tests__/mobile.auth-flow.test.tsx`, an auth-flow integration
    test that renders `RootNavigator` with the real `AuthProvider`, stubs the
    system browser and gateway client, and proves the full mobile flow from login
    through asset list and asset detail.
  - Verified the token-safety invariants remain covered in the auth/session suite:
    JWT-like values are rejected before secure-store persistence, and the UI never
    renders raw session values during the integrated auth flow.
  - Updated `docs/architecture.md` to mark the mobile app as an operational
    first-party client surface and `docs/plan/roadmap.md` to mark `S-050` complete.
  - Verified with `npm test` and `npm run typecheck` — both pass clean.

### Happy paths covered

- `HP-1`: full mobile login → asset list → detail flow stays green against the
  stubbed gateway.
  Evidence: `mobile/__tests__/mobile.auth-flow.test.tsx` renders `RootNavigator`,
  drives real `auth.login()` through the mocked system-browser return, opens the
  asset list, and navigates to asset detail; unit proof in
  `mobile/__tests__/mobile.auth-flow.test.tsx::T5 HP-1: full mobile login to asset detail flow stays green against the stub > signs in through the gateway, opens the asset list, and renders asset detail`.
- `HP-2`: token-safety guards remain green while the authenticated mobile flow runs.
  Evidence: `mobile/src/auth/AuthProvider.tsx::login` rejects JWT-like
  `session_ref` values before persistence, `mobile/src/auth/session.ts::updateSessionRef`
  rejects JWT-like rotations, and the integrated flow persists only opaque values;
  unit proof in
  `mobile/__tests__/mobile.auth-flow.test.tsx::T5 HP-1: full mobile login to asset detail flow stays green against the stub > signs in through the gateway, opens the asset list, and renders asset detail`
  plus
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-4: JWT-like session_ref from gateway is rejected > does not persist a JWT-looking session ref`
  and
  `mobile/__tests__/auth.session.test.ts::is a no-op when rotation is JWT-like → setItemAsync NOT called`.

### Edge cases covered

- `EC-1`: JWT-like or raw token values are never persisted to device storage.
  Evidence: `mobile/src/auth/session.ts::updateSessionRef` no-ops on JWT-like
  values and `mobile/src/auth/AuthProvider.tsx::login` rejects JWT-like
  `session_ref` values before `saveSessionRef`; unit proof in
  `mobile/__tests__/auth.session.test.ts::is a no-op when rotation is JWT-like → setItemAsync NOT called`
  and
  `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-4: JWT-like session_ref from gateway is rejected > does not persist a JWT-looking session ref`.
- `EC-2`: JWT-like or raw token values are never exposed in the UI during the
  mobile auth flow.
  Evidence: `mobile/__tests__/mobile.auth-flow.test.tsx` asserts the integrated
  flow does not render the opaque session or JWT-like sentinel values in the UI;
  unit proof in
  `mobile/__tests__/mobile.auth-flow.test.tsx::T5 HP-1: full mobile login to asset detail flow stays green against the stub > signs in through the gateway, opens the asset list, and renders asset detail`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | full mobile login → asset list → detail flow stays green against the stubbed gateway | `mobile/__tests__/mobile.auth-flow.test.tsx::T5 HP-1: full mobile login to asset detail flow stays green against the stub > signs in through the gateway, opens the asset list, and renders asset detail` | passed |
| HP-2 | Happy path | token-safety guards remain green while the authenticated flow runs | `mobile/__tests__/mobile.auth-flow.test.tsx::T5 HP-1: full mobile login to asset detail flow stays green against the stub > signs in through the gateway, opens the asset list, and renders asset detail` / `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-4: JWT-like session_ref from gateway is rejected > does not persist a JWT-looking session ref` / `mobile/__tests__/auth.session.test.ts::is a no-op when rotation is JWT-like → setItemAsync NOT called` | passed |
| EC-1 | Edge case | JWT-like or raw token values are never persisted to device storage | `mobile/__tests__/auth.session.test.ts::is a no-op when rotation is JWT-like → setItemAsync NOT called` / `mobile/__tests__/auth.provider.test.tsx::T3b-ii EC-4: JWT-like session_ref from gateway is rejected > does not persist a JWT-looking session ref` | passed |
| EC-2 | Edge case | JWT-like or raw token values are never exposed in the UI during the auth flow | `mobile/__tests__/mobile.auth-flow.test.tsx::T5 HP-1: full mobile login to asset detail flow stays green against the stub > signs in through the gateway, opens the asset list, and renders asset detail` | passed |

### Owner final verification

- Owner: `Codex GPT-5`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior, and the full mobile
  test suite plus TypeScript compilation both ran clean after the implementation.
- Commands run: `npm test`; `npm run typecheck`

---

## Related sub-slices

- **Retrospective BDD source of truth**
  ([docs/bdd/s-050-mobile-client.feature](/Users/matias/Documents/projects/dubbridge/docs/bdd/s-050-mobile-client.feature:1),
  [docs/bdd/README.md](/Users/matias/Documents/projects/dubbridge/docs/bdd/README.md:16)).
  This slice predated the repo-wide BDD-first convention; the shipped mobile-client
  behaviors were backfilled into a dedicated retrospective `.feature` spec on
  2026-06-12 without changing runtime scope or historical task order. The
  retrospective mapping is backed by shipped mobile-client test evidence and does
  not imply a standalone Maestro flow where none exists.
- **S-055 — Maestro screenshot / visual-audit suite**
  (`docs/plan/s-055-maestro-screenshot-suite.md`,
  `docs/tasks/s-055-maestro-screenshot-suite.md`). Mobile-hardening backlog capability
  that auto-captures screenshots of every mobile screen on an Android emulator via a
  two-phase Maestro flow. The historical gate for starting it, **this slice's T4**
  (core screens + T3b-ii/iii auth), is already satisfied; `T5` also closed on
  2026-06-07. Approved 2026-06-07 with **Option A** (ADR-024-clean handoff-code
  bootstrap — no JWT on device) and sequencing **S2** (defer the entire suite until
  after T4). Uses a `V`-prefixed task namespace (`V1`–`V8`) distinct from this
  file's `T` prefix; `V4 ≠ T4`. S-055 is **complete as of 2026-06-12**: V1–V8 all
  done. Both Maestro phases produce screenshots (`01_auth_login.png`, `02_home.png`);
  `cd mobile && npm run screenshots` runs the full suite end-to-end.

---

## Agent handoff prompt (delegation-ready)

> Implement slice **S-050 — first-party mobile client (React Native + Expo, TypeScript)**
> in the `dubbridge` repo, one task at a time in order T0→T5, per
> `docs/tasks/s-050-mobile-client.md` and `docs/plan/s-050-mobile-client.md`. Read the
> canonical guides first (`README_AGENT_ORDER.md`,
> `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`,
> `AGENTS.md`) and ADR-024/023/026. **Hard prerequisite: slice S-040 (session gateway /
> BFF) must already be built and stable** — T0 is a gate that blocks if it is not.
> The device authenticates only through the S-040 gateway and stores only an opaque
> session reference in `expo-secure-store`; it must **never** hold or persist an
> access/refresh JWT (ADR-024). Gateway-owned renewal/rotation is the only session
> extension path: mobile sends the current opaque reference, persists a rotated
> `X-Dubbridge-Session` value when returned by the gateway, and treats `401` as
> requiring re-authentication. Use the system browser for OAuth
> (`expo-auth-session`), not an embedded webview. The gateway base URL is
> environment-driven, never hardcoded (ADR-026). Connect to the real gateway/backend
> for behavior; stub only external boundaries (system browser, gateway) in tests.
> Present each task for explicit approval before implementing it; mark progress in
> this file after each task; do not commit with broken tests.
