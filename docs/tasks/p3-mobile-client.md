# Tasks: P3 — First-party mobile client (React Native + Expo)

**Plan:** `docs/plan/p3-mobile-client.md`
**Roadmap slice:** P3 (supporting platform). **Hard dependency: P1** (session
gateway / BFF, ADR-024). Benefits from **P2** (production identity hardening).

**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-024 (primary), ADR-023, ADR-026.
**Stack decision (2026-06-03):** React Native + Expo (managed workflow), TypeScript.

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
P1 (built) -> T0 -> T1 -> T2 -> T3 -> T4 -> T5
```

> **Blocking note:** T1+ must not start until P1 is built and its session contract
> (login/callback/logout + authenticated `/api/*` proxy) is stable. T0 is the gate.

---

## T0 — Gate: confirm P1 gateway session contract is available

- **Status:** [ ] Not started
- **Effort:** S
- **Complexity:** Low
- **Type:** Verification / docs (no app code)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** P1 built
- **Objective:** Confirm the P1 gateway exposes a stable, documented session
  contract usable by a native client: login start, callback, logout, authenticated
  proxy, and the transport-agnostic session mechanism defined in P1 T0. Record the
  exact endpoints, redirect/callback scheme, and session-transport expectations the
  mobile app will rely on.
- **Inputs:** P1 deliverables (`apps/gateway`, ADR-024 final), this plan.
- **Outputs:** A short "gateway contract for mobile" note appended to this task
  (endpoints, deep-link/redirect scheme, session transport); confirmation that no
  parallel auth path is required.
- **Acceptance criteria:**
  - The login/callback/logout/proxy endpoints and the session transport are
    documented and confirmed reachable.
  - The native redirect/deep-link scheme to return from the system browser to the
    app is decided and recorded.
  - If P1 is not yet built/stable, this gate **blocks** and the slice does not start.

---

## T1 — Expo app scaffold (TypeScript) + env-driven gateway config + navigation shell

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T0
- **Objective:** Create the `mobile/` Expo React Native TypeScript app with an
  environment-driven gateway base URL (per ADR-026 — no hardcoded URL), a navigation
  shell (authed vs. unauthed trees), and a placeholder home screen.
- **Inputs:** Expo SDK, T0 gateway contract.
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

---

## T2 — Gateway API client (typed) + error/session transport handling

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T1
- **Objective:** Implement a typed API client that calls the **P1 gateway** `/api/*`
  proxy (never `apps/api` directly with a raw token), carrying the gateway session
  transport, and maps responses/errors (401 → unauthenticated, 403 → forbidden,
  network/timeout). No raw access/refresh token is ever handled by the client.
- **Inputs:** T0 contract, T1 env config.
- **Outputs:** `src/api/client.ts`, typed request/response models, error mapping.
- **Acceptance criteria:**
  - All authenticated calls go through the gateway; the client holds no JWT.
  - 401 from the gateway routes the app to re-authenticate; 403 surfaces a
    forbidden state; network errors surface a retryable error state.
  - Client logic is unit-tested against a stubbed gateway (no real network in tests).
- **Happy paths considered:**
  - authenticated GET asset list via gateway → typed list returned and rendered.
- **Edge cases considered:**
  - gateway returns 401 (session expired) → client signals re-auth; no crash.
  - request timeout → typed error surfaced, UI can retry.

---

## T3 — Auth flow (system-browser OAuth via gateway) + secure session storage

- **Status:** [ ] Not started
- **Effort:** L
- **Complexity:** High
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
  (native OAuth redirect + secure-storage + session-lifecycle reasoning)
- **Depends on:** T2
- **Objective:** Implement login/logout through the device system browser
  (`expo-auth-session` / `expo-web-browser`) against the gateway, establishing the
  gateway session and storing only the opaque session reference in
  `expo-secure-store` (Keychain/Keystore). No access/refresh token is ever persisted
  on device (ADR-024).
- **Inputs:** T0 redirect scheme, T2 client, `expo-auth-session`,
  `expo-secure-store`.
- **Outputs:** `src/auth/session.ts`, `src/auth/AuthProvider.tsx`, login/logout
  wired to navigation gating.
- **Acceptance criteria:**
  - Login opens the system browser, completes the gateway OAuth flow, and returns to
    the app via the registered deep-link scheme; a session is established.
  - Only an opaque session reference is stored, in `expo-secure-store` — never a JWT
    or refresh token (asserted by a test).
  - Logout clears the device session reference and calls the gateway logout.
- **Happy paths considered:**
  - login → system browser → callback deep link → session stored → authed tree shown.
  - logout → session cleared locally + at gateway → unauthed tree shown.
- **Edge cases considered:**
  - user cancels the system-browser login → app returns to unauthed state cleanly.
  - secure-store assertion: stored value is not a JWT/refresh token.

---

## T4 — Core screens (Login, Home, AssetList, AssetDetail)

- **Status:** [ ] Not started
- **Effort:** M
- **Complexity:** Medium
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** T3
- **Objective:** Build the core authenticated screens against the gateway client:
  Login entry, authenticated Home, AssetList (from S1 asset/ingestion state), and
  AssetDetail / ingestion status. Handle loading/empty/error states and degrade
  gracefully where S4–S9 backend surfaces do not yet exist.
- **Inputs:** T2 client, T3 auth context.
- **Outputs:** `src/screens/{Login,Home,AssetList,AssetDetail}.tsx`.
- **Acceptance criteria:**
  - Authenticated screens render real data fetched through the gateway (no mocked
    backend data; only test doubles in tests).
  - Loading, empty, and error states are handled on each data screen.
  - Screens that would depend on unbuilt slices (S4–S9) show a clear
    "not available yet" state instead of failing.
- **Happy paths considered:**
  - authed user opens AssetList → assets load via gateway → tapping one opens
    AssetDetail with ingestion status.
- **Edge cases considered:**
  - empty asset list → friendly empty state.
  - detail for an asset whose downstream (transcode/ASR) is not built → graceful
    "pending / not available" state.

---

## T5 — Tests (unit + component + auth-flow) + docs/roadmap sync

- **Status:** [ ] Not started
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
  first-party client surfaces), `docs/plan/roadmap.md` (P3 status), ADR-024 mobile
  reference confirmed.
- **Acceptance criteria:**
  - Unit + component + auth-flow tests pass deterministically (no real network).
  - A test asserts no JWT/refresh token is ever stored on device or exposed to UI.
  - `docs/architecture.md` and `docs/plan/roadmap.md` reflect P3 as built.
- **Happy paths considered:**
  - full mobile login → asset list → detail → logout flow green against the stub.
- **Edge cases considered:**
  - expired gateway session mid-use → app routes to re-auth (covered by a test).
  - secure-store test asserts the absence of any raw token.

---

## Agent handoff prompt (delegation-ready)

> Implement slice **P3 — first-party mobile client (React Native + Expo, TypeScript)**
> in the `dubbridge` repo, one task at a time in order T0→T5, per
> `docs/tasks/p3-mobile-client.md` and `docs/plan/p3-mobile-client.md`. Read the
> canonical guides first (`README_AGENT_ORDER.md`,
> `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`,
> `AGENTS.md`) and ADR-024/023/026. **Hard prerequisite: slice P1 (session gateway /
> BFF) must already be built and stable** — T0 is a gate that blocks if it is not.
> The device authenticates only through the P1 gateway and stores only an opaque
> session reference in `expo-secure-store`; it must **never** hold or persist an
> access/refresh JWT (ADR-024). Use the system browser for OAuth
> (`expo-auth-session`), not an embedded webview. The gateway base URL is
> environment-driven, never hardcoded (ADR-026). Connect to the real gateway/backend
> for behavior; stub only external boundaries (system browser, gateway) in tests.
> Present each task for explicit approval before implementing it; mark progress in
> this file after each task; do not commit with broken tests.
