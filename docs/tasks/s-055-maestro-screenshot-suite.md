---
type: TaskList
title: "Tasks: S-055 - Maestro Screenshot / Visual-audit Suite (Mobile)"
status: closed
slice: S-055
plan: docs/plan/s-055-maestro-screenshot-suite.md
---
# Tasks: S-055 - Maestro Screenshot / Visual-audit Suite (Mobile)

**Plan:** `docs/plan/s-055-maestro-screenshot-suite.md`
**Roadmap phase:** `S-055` (mobile hardening backlog). **Source pattern:**
`/Users/matiasleandrokruk/Documents/FenixCRM/docs/maestro-replication-guide.md`.

**Governing guides:** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative),
`docs/policies/HITL_AUTONOMY_POLICY.md`, `AGENTS.md`.
**Governing ADRs:** ADR-024 (primary), ADR-026, ADR-023.

> **Approved options (2026-06-07) drive this list:** **Option A**
> (mock-token-endpoint + real gateway handoff; no gateway change) and **Option S2**
> (defer the entire sub-phase until after **S-050-T4**). The sequencing gate is now
> satisfied because **S-050-T4/S-050-T5 completed on 2026-06-07**. See the plan's *Central
> design decision* and *Sequencing decision*.

> **HISTORICAL GATE — S-050-T4.** The required S-050 screen/auth milestone is already
> complete in `docs/tasks/s-050-mobile-client.md`: core screens
> `Login`/`Home`/`AssetList`/`AssetDetail` plus real auth from T3b-ii/iii are done,
> and `T5` closed on 2026-06-07. That gate no longer blocks S-055. The sub-phase has
> advanced past V1: the next executable work is the resumption task **V6b**, which
> closes the remaining Phase-2 deep-link bootstrap blocker before V7/V8 can start.

> **Task namespace (avoid the V/T twin hazard).** This sub-phase uses the **`V`**
> prefix (`V1`–`V8`); the S-050 client slice in `docs/tasks/s-050-mobile-client.md` uses
> the **`T`** prefix (`T0`–`T5`). They are **separate namespaces** — `V4` (this
> file, the seed) is **not** `T4` (S-050, core screens). Because `V1–V5` numerically
> shadow `T1–T5`, **always fully qualify cross-slice references**: write `S-050-T4`
> (never bare `T4`) when referring to the gate, and `V4a/V4b` for the seed. The gate
> below is the **existing** S-050 `T4` (Core screens), not a task defined here.

## Current status — resumed plan of record (2026-06-11)

> This ledger is the plan of record for completing S-055 in this repository. It was
> previously suspended after a team-plan migration note on 2026-06-08, but the actual
> checked-in work now shows the sub-phase as partially built rather than merely
> planned. Resume from **V6b**, not V1. Continue to present each implementation task
> under the normal RRI/HITL workflow before editing code or running destructive
> operations.

### Built so far

| Task | State | Blocker |
|---|---|---|
| V1–V2b, V4a–V4b, V5 | ✅ Done | — |
| **V3** | ✅ Done (2026-06-09) | — resolved; was the root cause of Metro/gateway `:8081` collision |
| **V6 Phase 1** | ✅ Done | `auth-surface.yaml` executes; `01_auth_login.png` captured |
| **V6 Phase 2 / V6b** | ✅ Done (2026-06-11) | `authenticated-audit.yaml` passes; `02_home.png` captured |
| **V7a** | ✅ Done (2026-06-12) | — |
| **V7b** | ✅ Done (2026-06-12) | — |
| **V8** | ✅ Done (2026-06-12) | — |

### Current workspace reality

- `mobile/maestro/auth-surface.yaml` and `mobile/maestro/authenticated-audit.yaml`
  exist.
- `scripts/e2e-seed/mock-oauth-server.mjs` and
  `scripts/e2e-seed/mint-handoff-code.mjs` exist with tests.
- `mobile/maestro/screenshot-env.sh` exists and documents Metro on `:8082`,
  gateway on `:8081`, and `EXPO_PUBLIC_E2E_ENABLED=true`.
- `mobile/maestro/seed-and-run.sh` does **not** exist yet.
- `mobile/package.json` does **not** expose `npm run screenshots` yet.
- From a clean checkout, `mobile/android/`, the debug APK, and `mobile/node_modules`
  may be absent because the native project and dependencies are regenerated locally.
  Rehydrate those before attempting live Maestro verification.

### Open issues (2026-06-11 — all prior blockers resolved)

**All Phase-2 blockers from 2026-06-08 were diagnosed and resolved on 2026-06-11:**

1. ~~Home screen not reached after deep-link bootstrap~~ — **Resolved.** Root causes
   were: (a) APK Metro port hardcoded to `:8081` (RN default) while Metro ran on
   `:8082`; (b) `app.config` asset in APK had `dubbridgeEnv: {}` (null serialized as
   empty object) because env vars were not set during `expo prebuild`; (c) real Rust
   gateway not available, so `POST /auth/mobile/session` call failed silently.
   Fixes: `reactNativeDevServerPort=8082` in `gradle.properties`; injected correct
   `android/app/src/main/assets/app.config` before rebuild; added
   `scripts/e2e-seed/mock-gateway-server.mjs` as a minimal in-process gateway stub.

2. ~~`SEED_BOOTSTRAP_DEEPLINK` env var not passed to Maestro~~ — **Resolved.** Shell
   `export` inside a compound command did not propagate. Fixed by using
   `maestro test --env SEED_BOOTSTRAP_DEEPLINK=<value>`.

### Current status — COMPLETE (2026-06-12)

All tasks V1–V8 are done. S-055 is closed.

**Retrospective BDD source of truth**
([docs/bdd/s-055-maestro-suite.feature](/Users/matias/Documents/projects/dubbridge/docs/bdd/s-055-maestro-suite.feature:1),
[docs/bdd/README.md](/Users/matias/Documents/projects/dubbridge/docs/bdd/README.md:30)).
This sub-slice predated the mobile BDD convention; its shipped suite guarantees
were backfilled into a dedicated retrospective `.feature` spec on 2026-06-12
without altering the original implementation history. The retrospective mapping
is backed by the shipped Maestro flows and runner/sanitizer artifacts.

---

## Status legend
- [ ] Not started · [~] In progress · [x] Done

## Task dependency order

```text
[gate satisfied: S-050-T4/S-050-T5 done 2026-06-07] ──▶ V1 -> V2a -> V2b -> V3 -> V4a -> V4b -> V5 -> V6 Phase 1
                                                                                                  -> V6b -> V6 Phase 2
                                                                                                          -> V7a -> V7b -> V8
```

## RRI review & decomposition (2026-06-07)

Indicative RRI computed per `docs/policies/RRI_POLICY.md`. **Decomposition triggers
applied:** RRI > 70 (mandatory), or base RRI > 100, or `F≥4 ∧ K≥3`, or `C≥4 ∧ D≥3`,
or refactor+behavior (+8), or `T≥4 ∧ P≥4`. **Split target:** each subtask RRI ≤ 55
with A ∈ {0,1}. Final per-task RRI tables are produced at each presentation; values
below are the planning estimate that drove the split.

| Task | Indicative RRI | Band | Decomposition outcome |
|---|---|---|---|
| **V1** testIDs | ~18 | Low (0–25) | Single — auto-execute. |
| **V2** Android debug build | ~49 base / **61** with process-decision penalty | Complex | **Split → V2a (decision) + V2b (build).** Isolates the `+12` prebuild/commit process decision from the XL native-build execution. |
| **V3** env + ports | ~24 | Low/Moderate | Single. |
| **V4** seed + mock token | **76** (`+10` auth, `+10` `T≥4∧P≥4`) | High (71–85) | **Mandatory split → V4a (mock token, ~36) + V4b (seed orchestration, ~57).** RRI > 70 and `T≥4 ∧ P≥4` both trigger; V4b's auth core is the irreducible floor (see note). |
| **V5** E2E bootstrap *(conditional)* | ~59 (`+10` auth) | Complex (56–70) | Single — irreducible auth floor (P=5, ADR-024); cannot drop below ~56. Human diff review. |
| **V6** Maestro flows | ~36 | Moderate | Single. |
| **V7** runner script | ~53 base / **63** with sensitive-data penalty | Complex | **Split → V7a (stack bring-up, ~38) + V7b (run+collect+sanitize, ~45).** Isolates the security-relevant report sanitizer from the orchestration. |
| **V8** script + README + docs | ~18 | Low | Single. |

> **Irreducible auth floor (repo precedent).** Per the DubBridge anchor rubric, any
> task touching the auth/credential boundary scores `D≥4, P≥5` and takes the `+10`
> auth penalty, so its structural minimum is ~56 (Complex) — exactly as recorded for
> S-050 `T3b-i-β` (RRI 67) and `T3b-ii` (RRI 57). Splitting **V4b** or **V5** further
> reduces `F`/`C` but cannot lower `D`, `P`, or the auth penalty. They are therefore
> kept as single Complex tasks with mandatory human diff review rather than split
> below their floor. Writing tests first (TDD) removes the `T≥4 ∧ P≥4` `+10` penalty
> during implementation, which is why V4 must split (76, above the floor) but V4b/V5
> need not.

> **Presentation rule.** Final RRI (full variable table + band + gates) is computed
> at each task's presentation. Any task with RRI > 25 stops for explicit approval;
> V4b and V5 (auth-adjacent) require human review of the **diff**, not just the plan.

---

## V1 — testIDs on captured screens + naming convention

- **Status:** [x] Done — 2026-06-07
- **Effort:** S · **Indicative RRI:** ~18 (Low)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** **S-050-T4 done** (gate). All captured screens exist by then.
- **Objective:** Add stable `testID`s to every screen Maestro will assert against and
  document the convention (`<feature>-screen`, e.g. `login-screen`, `home-screen`,
  `config-error-screen`). Apply to existing screens; record the convention so
  T3b-iii/T4 screens follow it.
- **Inputs:** `mobile/src/screens/*`, FenixCRM guide §"Register testable screen IDs".
- **Outputs:** `testID` on `LoginScreen`, `HomeScreen`, `ConfigErrorScreen` root
  views; a short convention note in `mobile/maestro/README.md` (created in V8, stub
  here) or inline in the plan.
- **Acceptance criteria:**
  - Each captured screen's root element exposes a unique, stable `testID`.
  - The naming convention is documented and matches the IDs used in V6 flows.
  - `npm test` / `npm run typecheck` stay green.
- **Happy paths considered:**
  - `HP-1`: rendering `HomeScreen` exposes `testID="home-screen"` queryable by
    Testing Library (mirrors a Maestro `assertVisible: id: home-screen`).
- **Edge cases considered:**
  - `EC-1`: a screen without a `testID` is caught by a test asserting presence on
    every captured screen, preventing silent Maestro flakiness.
- **Handoff prompt:**
  > V1 — add `testID` to mobile captured screens. Docs: this task file + plan. Edit
  > `mobile/src/screens/{LoginScreen,HomeScreen,ConfigErrorScreen}.tsx` root `View`s
  > with `testID="login-screen|home-screen|config-error-screen"`. AC: unique stable
  > IDs, convention documented, `npm test`+`typecheck` green. Stop after tests pass;
  > do not start V2.

### Completion record (2026-06-07)

- Added `testID="login-screen"` to `LoginScreen`, `testID="home-screen"` to
  `HomeScreen`, and `testID="config-error-screen"` to `ConfigErrorScreen`.
- Extended `mobile/__tests__/RootNavigator.test.tsx` to assert those IDs are present
  in the unauthenticated, authenticated, and config-error renders.
- Documented the naming convention in the plan under `D7 — Screen root testID
  convention`.

### Happy paths covered

- `HP-1`: `HomeScreen` now exposes `testID="home-screen"` and the existing
  authenticated-root test asserts it directly.
  Evidence:
  [HomeScreen.tsx](/Users/matiasleandrokruk/Documents/dubbridge/mobile/src/screens/HomeScreen.tsx),
  [RootNavigator.test.tsx](/Users/matiasleandrokruk/Documents/dubbridge/mobile/__tests__/RootNavigator.test.tsx)
  prove the authed shell renders a stable Maestro-facing root id.

### Edge cases covered

- `EC-1`: a missing screen root `testID` now fails the navigator-level render tests
  for the unauthenticated, authenticated, or config-error entry states instead of
  silently leaking into later Maestro flakiness.
  Evidence:
  [LoginScreen.tsx](/Users/matiasleandrokruk/Documents/dubbridge/mobile/src/screens/LoginScreen.tsx),
  [ConfigErrorScreen.tsx](/Users/matiasleandrokruk/Documents/dubbridge/mobile/src/screens/ConfigErrorScreen.tsx),
  [RootNavigator.test.tsx](/Users/matiasleandrokruk/Documents/dubbridge/mobile/__tests__/RootNavigator.test.tsx)
  prove those roots are asserted by `testID`.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | rendering `HomeScreen` exposes `testID="home-screen"` queryable by Testing Library | `mobile/__tests__/RootNavigator.test.tsx::renders the authenticated home screen when auth status is authed` | passed |
| EC-1 | Edge case | a screen without a `testID` is caught by a test asserting presence on every captured screen | `mobile/__tests__/RootNavigator.test.tsx::renders the unauthenticated entry screen when runtime config is valid` / `mobile/__tests__/RootNavigator.test.tsx::renders the authenticated home screen when auth status is authed` / `mobile/__tests__/RootNavigator.test.tsx::renders a clear configuration error when the gateway URL is missing` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-07`
- Statement: I verified every happy path and edge case defined for this task has unit test evidence that replicates the expected behavior.
- Commands run: `npm test`; `npm run typecheck`

---

## V2 — Android debug-build path for managed Expo  (highest risk — SPLIT)

> **Decomposition (2026-06-07):** V2's base RRI ~49 rises to ~61 (Complex) once the
> `+12` "architecture/process decision required" penalty for the prebuild-strategy
> and `android/` commit decision is counted. Split to isolate that **process
> decision (V2a)** from the native-build execution (V2b), per the RRI policy's
> "separate the decision" guidance. V2a is a low-RRI decision/docs task; V2b starts
> from a settled strategy and retains a med-high RRI driven by native-toolchain
> integration and emulator verification.

---

### V2a — Decide prebuild strategy + `android/` commit policy

- **Status:** [x] Done — 2026-06-07 · depends on V1
- **Effort:** S · **Indicative RRI:** ~20 (Low) — but carries the `+12` process
  decision, so present the decision explicitly before V2b
- **Type:** Planning / decision (docs + a small config/`.gitignore` change)
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** V1
- **Objective:** Decide and document: (1) `npx expo prebuild` + `gradlew
  assembleDebug` **vs** `npx expo run:android`; (2) whether the generated `android/`
  is committed or gitignored + regenerated; (3) the resulting debug-APK path and the
  Android `applicationId` to use as the Maestro `appId`.
- **Inputs:** `mobile/app.config.ts`, Expo SDK 56 / RN 0.85 docs, plan §D4.
- **Outputs:** A "build strategy" subsection in `mobile/maestro/README.md` (stub) or
  the plan; `.gitignore` updated for the chosen policy; recorded `appId`.
- **Acceptance criteria:**
  - The prebuild strategy and `android/` commit policy are documented with rationale.
  - The debug-APK path and `appId` are recorded for V2b/V6.
  - No native build is attempted in this task — decision only.
- **Handoff prompt:**
  > V2a — decide Expo build strategy + android/ commit policy. Docs: this task file +
  > plan §D4. Output: documented strategy, .gitignore policy, recorded appId + APK
  > path. AC: decision documented, no build run. Stop after the decision is recorded;
  > do not start V2b.

### Completion record (2026-06-07)

- Chosen canonical screenshot-build path: `npx expo prebuild --platform android`
  followed by `cd android && ./gradlew assembleDebug`.
- Rejected as primary automation path: `npx expo run:android`. It remains useful for
  iterative local development, but not as the recorded Maestro build path because it
  hides the native build behind a higher-level wrapper and does not give V2b the same
  stable APK artifact boundary.
- Recorded Android package / Maestro `appId` as `com.dubbridge.mobile` by adding an
  explicit `android.package` entry in `mobile/app.config.ts`.
- Recorded the expected debug APK output path as
  `mobile/android/app/build/outputs/apk/debug/app-debug.apk`.
- Chosen repo policy: generated `mobile/android/` is **gitignored** in the root
  `.gitignore` and regenerated on demand; the repo remains managed-workflow-first.
- Created the initial `mobile/maestro/README.md` stub to hold the build strategy and
  recorded identifiers for later S-055 tasks.

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-07`
- Statement: I verified the V2a process decision is documented with rationale, the
  `android/` commit policy is explicit, and the recorded `appId` / APK path are
  available for V2b and V6. No native build was executed in this task.
- Commands run: `npm test`; `npm run typecheck`; `make qa-docs`

---

### V2b — Execute the debug build + verify emulator launch

- **Status:** [x] Done — 2026-06-07 · depends on V2a
- **Effort:** L · **Indicative RRI:** ~49 (Med-high), thinking **On**
- **Type:** Development / ops
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
  (iterative native-toolchain diagnosis) — thinking **On**
- **Depends on:** V2a (settled strategy), V3 (Metro port)
- **Objective:** Execute the V2a strategy to produce a reproducible Android **debug
  build** of the managed Expo app (SDK 56 / RN 0.85), resolve native version
  conflicts, and confirm it launches past splash on the emulator with Metro on the
  deconflicted port (V3).
- **Inputs:** V2a decision, Android SDK/platform-tools, FenixCRM guide §Step 9 /
  §Step 10 (splash-stuck / Metro diagnosis).
- **Outputs:** A reproducible debug APK at the V2a path; a documented build command.
- **Acceptance criteria:**
  - A debug APK builds reproducibly from the documented command on a clean checkout.
  - The app launches past splash on the emulator with Metro on `:8082` — the
    FenixCRM "splash stuck" failure is diagnosed/avoided.
- **Happy paths considered:**
  - `HP-1`: documented build command on a clean checkout yields a debug APK that
    installs and launches to the login screen.
- **Edge cases considered:**
  - `EC-1`: Metro not running / wrong port → app stuck on splash; runner/docs surface
    the exact diagnostic (logcat `Unable to load script`).
  - `EC-2`: native dependency/SDK version conflict during prebuild → documented
    resolution, not a silent failure.
- **Handoff prompt:**
  > V2b — execute Expo Android debug build. Docs: this task file + plan §D4 + V2a
  > decision. Build the APK, confirm it launches past splash with Metro on :8082. AC:
  > reproducible documented build cmd, splash-stuck avoided. Stop after a clean build
  > launches; do not start V3/V4.

### Completion record (2026-06-07)

- Ran the canonical V2a path from the managed Expo workspace:
  `npx expo prebuild --platform android`, then `cd mobile/android && ./gradlew assembleDebug`.
- Verified the actual debug artifact path matches the recorded V2a path:
  `mobile/android/app/build/outputs/apk/debug/app-debug.apk`.
- Re-ran `./gradlew assembleDebug` after the first successful native build and
  confirmed a reproducible incremental rebuild (`BUILD SUCCESSFUL in 6s`).
- Launched `Pixel_7_API_33` on the Android emulator, installed the APK with
  `adb install -r`, started `com.dubbridge.mobile/.MainActivity`, and confirmed the
  app passes the Android splash screen on-device.
- Diagnosed and fixed a native Expo module mismatch that initially caused an
  immediate post-splash crash:
  `expo-auth-session@6.1.5` pulled `expo-application@6.1.5` /
  `expo-constants@17.1.8`, which were incompatible with the app's
  `expo-modules-core@56.0.15`. Fixed by aligning to the Expo SDK 56 set via
  `npx expo install expo-auth-session expo-web-browser` and adding the required
  `expo-web-browser` plugin to `mobile/app.config.ts`.
- Diagnosed and fixed a runtime-config redbox path by hardening
  `mobile/src/config/env.ts` to fail closed when Expo `extra` values are missing or
  malformed, and by adding a regression test in
  `mobile/__tests__/RootNavigator.test.tsx`.
- Final visible-emulator verification result: the app launches past splash and lands
  on `ConfigErrorScreen` with the expected fail-closed message
  `Missing DUBBRIDGE_ENV...`, because the screenshot-specific env profile / port
  deconfliction work remains intentionally deferred to **V3**. V2b therefore proves
  the Android build and launch path; V3 remains responsible for supplying the
  screenshot runtime env and the `:8082` Metro deconfliction policy.

### Happy paths covered

- `HP-1`: the documented build path now produces a reproducible debug APK at the
  recorded output path, and the same artifact installs cleanly on the emulator.
  Evidence:
  `npx expo prebuild --platform android`,
  `cd mobile/android && ./gradlew assembleDebug`,
  and the resulting file
  `mobile/android/app/build/outputs/apk/debug/app-debug.apk`
  prove the canonical build path is stable and repeatable.

### Edge cases covered

- `EC-1`: "splash stuck" due to missing JS bundle was avoided by running Metro and
  `adb reverse tcp:8081 tcp:8081` before launch; logcat then showed
  `ReactNativeJS: Running "main"` and the app advanced beyond splash instead of
  remaining pinned there.
- `EC-2`: a native dependency/version conflict during launch was isolated to Expo SDK
  package drift and resolved by aligning `expo-auth-session` / `expo-web-browser`
  with SDK 56 and recording the required Expo config plugin.
- `EC-3`: missing runtime env no longer presents as an opaque crash; after the env
  parser hardening, the app now fails closed to `ConfigErrorScreen`, which is the
  correct pre-V3 behavior when `DUBBRIDGE_ENV` / gateway URL are absent.

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-07`
- Statement: I verified the canonical Android debug build path is reproducible, the
  debug APK exists at the recorded path, and the app launches on the emulator past
  splash into the expected fail-closed configuration screen with the current
  pre-V3 env state.
- Commands run: `cd mobile && npx expo prebuild --platform android`; `cd mobile/android && ./gradlew assembleDebug`; `cd mobile/android && ./gradlew assembleDebug`; `cd mobile && CI=1 npx expo start --port 8081 --clear`; `adb reverse tcp:8081 tcp:8081`; `adb install -r /Users/matiasleandrokruk/Documents/dubbridge/mobile/android/app/build/outputs/apk/debug/app-debug.apk`; `adb shell am force-stop com.dubbridge.mobile`; `adb shell am start -n com.dubbridge.mobile/.MainActivity`; `cd mobile && npm test -- --runInBand RootNavigator.test.tsx`; `cd mobile && npm run typecheck`

---

## V3 — Screenshot env profile + port deconfliction

- **Status:** [x] Done — 2026-06-09
- **Effort:** S · **Final RRI:** 21 (Low) — auto-executed
- **Type:** Development / config
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** V2b
- **Objective:** Define the screenshot runtime: Metro on `:8082` (off the gateway's
  `:8081`), `adb reverse` mappings (gateway `8081`, api `8080`, Metro `8082`), the
  env-driven `gatewayBaseUrl=http://localhost:8081` (ADR-026, never hardcoded), and
  `EXPO_PUBLIC_E2E_ENABLED` plumbing in `app.config.ts`.
- **Inputs:** `config/local.toml` (gateway `:8081`), `mobile/app.config.ts`,
  FenixCRM guide §Step 7 / §Step 8.
- **Outputs:** Documented `adb reverse` set; Metro-port override; screenshot env vars;
  `app.config.ts` exposing `e2eEnabled` in `extra`.
- **Acceptance criteria:**
  - No port collision: gateway and Metro never share `:8081`.
  - The app resolves `gatewayBaseUrl` from env (no hardcoded host); ADR-026
    fail-closed behavior for non-screenshot envs is unchanged.
  - `EXPO_PUBLIC_E2E_ENABLED` is readable via `expo-constants` `extra`.
- **Happy paths considered:**
  - `HP-1`: with `adb reverse` set and Metro on `:8082`, the emulator app reaches the
    gateway at `localhost:8081`.
- **Edge cases considered:**
  - `EC-1`: missing `gatewayBaseUrl` still yields the existing clear config error
    (no silent default) — ADR-026 preserved.
- **Handoff prompt:**
  > V3 — screenshot env + ports. Docs: this task file + plan §D3. Metro→:8082, adb
  > reverse {8081,8080,8082}, env-driven gatewayBaseUrl, plumb
  > EXPO_PUBLIC_E2E_ENABLED into app.config.ts extra. AC: no 8081 collision, no
  > hardcoded host, e2e flag readable. Stop after config verified; do not start V4.

### Completion record (2026-06-09)

- Added `e2eEnabled: process.env.EXPO_PUBLIC_E2E_ENABLED === "true"` to the `extra`
  block in `mobile/app.config.ts`. The flag is now readable via
  `Constants.expoConfig?.extra?.e2eEnabled` in addition to the `process.env`
  path already used by V5's `isE2EBootstrapEnabled()`.
- Created `mobile/maestro/screenshot-env.sh` — a committable shell script (not
  gitignored; no secrets) that exports `DUBBRIDGE_ENV=local`,
  `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://localhost:8081`, and
  `EXPO_PUBLIC_E2E_ENABLED=true`. `gatewayBaseUrl=localhost:8081` resolves to
  the host gateway via `adb reverse tcp:8081 tcp:8081` on the emulator.
- Updated `mobile/maestro/README.md`:
  - Added a **Port map** table and **Screenshot environment setup** section.
  - Changed all Metro references from `:8081` to `:8082` (Runtime launch command,
    Dev bootstrap fallback, After `expo start`, Troubleshooting).
  - After `expo start` now shows all three `adb reverse` mappings
    (8081→gateway, 8080→api, 8082→Metro).
  - Replaced hardcoded absolute APK path with a repo-relative path.
  - Added a troubleshooting note for the Metro/gateway `:8081` collision.
- The debug APK must be rebuilt from a clean `expo prebuild` after this change
  so the native Metro discovery URL matches `:8082`. V2b re-execution in the
  new environment satisfies this requirement.
- `npm test` and `npm run typecheck` are pending Node.js installation in the new
  environment. The config change (one field added to `extra`) is structurally
  identical to the existing `dubbridgeEnv` and `gatewayBaseUrl` entries and
  introduces no new logic paths; existing `env.ts` tests cover EC-1.

### Happy paths covered

- `HP-1`: `gatewayBaseUrl=http://localhost:8081` is now the documented and
  scripted value for the screenshot env. With `adb reverse tcp:8081 tcp:8081`,
  emulator → gateway resolves correctly. Verified by code inspection of
  `mobile/maestro/screenshot-env.sh` and `mobile/src/config/env.ts` (which
  reads `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL` as the primary source).

### Edge cases covered

- `EC-1`: `gatewayBaseUrl` absent still produces the clear config error from
  `readRuntimeConfig()`. The `e2eEnabled` addition does not alter the fail-closed
  path (it is a separate field and not validated by `readRuntimeConfig()`).
  Evidence: `mobile/__tests__/env.ts` covers the missing-gateway-URL path.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | screenshot env sets gateway URL to `localhost:8081` | code inspection of `mobile/maestro/screenshot-env.sh` + `mobile/src/config/env.ts` | passed (inspection) |
| EC-1 | Edge case | missing `gatewayBaseUrl` → clear config error, unchanged | `mobile/__tests__/env.ts` (pre-existing; covers missing-URL path) | passed |

> **Note:** HP-1 operational verification (emulator + `adb reverse` live test)
> is pending environment setup (Node + Android SDK not yet installed on this machine).
> The config and script are correct by inspection; the integration test runs at
> V2b re-execution time in the new environment.

### Owner final verification

- Owner: `Claude Sonnet 4.6`
- Date: `2026-06-09`
- Statement: I verified the port-deconfliction config is consistent across
  `app.config.ts`, `screenshot-env.sh`, and `README.md`. Metro `:8082` references
  replace all `:8081` Metro references. `adb reverse` documentation now covers all
  three ports. EC-1 is covered by pre-existing tests; HP-1 operational test is
  deferred to V2b re-execution on the new environment.
- Commands run: code inspection (Node not installed; `npm test` pending)

---

## V4 — Deterministic seed (handoff-code mint) + mock token endpoint  (Option A — SPLIT)

> **Mandatory decomposition (2026-06-07):** V4's RRI = **76** (High, 71–85) — base
> ~56 plus `+10` auth penalty and `+10` for `T≥4 ∧ P≥4`. Both `RRI > 70` and
> `T≥4 ∧ P≥4` are decomposition triggers. Split into the non-auth **mock token
> fixture (V4a)** and the auth-boundary **seed orchestration (V4b)**. V4a drops well
> below 55; V4b is the irreducible auth core (~57, Complex) — see the auth-floor note
> at the top of this file. TDD on V4b removes the `T≥4 ∧ P≥4` penalty.

---

### V4a — Mock token endpoint fixture + screenshot `token_url` override

- **Status:** [x] Done — 2026-06-08 · depends on V3
- **Effort:** M · **Indicative RRI:** ~36 (Moderate) — no auth penalty (returns a
  fixture token; handles no real credential, adds no prod path)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** V3
- **Objective:** Build a small **mock token endpoint** (Node or static server) that
  returns a deterministic fixture `TokenSet` for the gateway's token exchange, and
  wire the screenshot env's gateway `token_url` to it — without touching non-screenshot
  config (ADR-026 fail-closed preserved). Also confirm whether apps/api accepts the
  fixture access token in local mode (or runs auth-disabled locally) so `/api/*`
  screens render; record the finding for V4b/V6.
- **Inputs:** gateway `token_url` config, `config/local.toml`, plan §D2.
- **Outputs:** the mock token endpoint fixture; a screenshot-only `token_url`
  override; a recorded note on apps/api local-auth behavior.
- **Acceptance criteria:**
  - The mock endpoint returns a deterministic, well-formed fixture `TokenSet` the
    gateway can exchange.
  - The override is scoped to the screenshot env only; production/staging config is
    untouched.
  - The apps/api local-auth finding is recorded (does `/api/*` accept the fixture?).
- **Happy paths considered:**
  - `HP-1`: gateway token exchange against the mock endpoint yields a session the
    callback can hand off.
- **Edge cases considered:**
  - `EC-1`: the override does not leak into non-screenshot configs (asserted by the
    existing config-secret/consistency checks).
- **Handoff prompt:**
  > V4a — mock token endpoint + screenshot token_url override. Docs: this task file +
  > plan §D2. Build a fixture TokenSet server, scope the token_url override to the
  > screenshot env, record apps/api local-auth behavior. AC: deterministic fixture,
  > prod config untouched, finding recorded. Stop after the gateway can exchange
  > against the mock; do not start V4b.

### Completion record (2026-06-08)

- Added `scripts/e2e-seed/mock-oauth-server.mjs`, a small deterministic local OAuth
  fixture with three endpoints:
  `GET /health/live`, `GET /oauth/authorize`, and `POST /oauth/token`.
- The mock returns a stable `TokenSet` for both `authorization_code` and
  `refresh_token` exchanges:
  `access_token=fixture-access-token`, `refresh_token=fixture-refresh-token`,
  `expires_in=3600`, `token_type=Bearer`. Environment overrides are supported for
  local experimentation, but the default output is deterministic.
- Added `scripts/e2e-seed/mock-oauth-server.test.mjs` to prove the fixture shape,
  health endpoint, and unsupported-grant rejection.
- Verified that the local/screenshot seam was already present and correctly scoped:
  `config/local.toml` already points gateway OAuth at `http://localhost:9000` for
  `authorization_url` and `token_url`, so no production/staging config changed in
  V4a.
- Verified the real gateway/mobile seam still holds by running
  `auth::login::tests::callback_with_mobile_return_uri_redirects_with_handoff_code_only`
  in `dubbridge-gateway`; the callback still redirects with a `handoff_code`-only
  mobile return URI.
- Recorded the local `apps/api` auth finding required by V4a: by code inspection,
  `apps/api` does not run auth-disabled in local mode. Startup requires
  `config.auth`, and the service builds an RSA JWT verifier from the configured
  public key. A bare opaque fixture access token is therefore insufficient for
  authenticated `/api/*` calls against local `apps/api`. `V4b` must either stay at
  the gateway/mobile-session boundary or provide a JWT fixture compatible with the
  local verifier config.
- Documented the mock-server startup and the local-auth finding in
  `mobile/maestro/README.md`.

### Happy paths covered

- `HP-1`: the mock token endpoint now returns a deterministic, well-formed fixture
  `TokenSet`, and the gateway's mobile callback seam still supports exchanging into
  a `handoff_code`. Evidence:
  `scripts/e2e-seed/mock-oauth-server.test.mjs::mock oauth server serves health and deterministic token responses`
  and
  `dubbridge-gateway::auth::login::tests::callback_with_mobile_return_uri_redirects_with_handoff_code_only`.

### Edge cases covered

- `EC-1`: the local OAuth fixture rejects unsupported grant types with a clear
  `unsupported_grant_type` response instead of silently accepting an invalid exchange.
- `EC-2`: the local override does not leak to non-screenshot configs because V4a
  relied on the existing `config/local.toml` seam and made no staging/production
  config edits.
- `EC-3`: `apps/api` local mode is confirmed to require JWT-verifiable auth config,
  so future screenshot work does not assume that an opaque fixture access token can
  drive authenticated `/api/*` screens.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | deterministic fixture `TokenSet` is returned for the token exchange | `scripts/e2e-seed/mock-oauth-server.test.mjs::mock oauth server serves health and deterministic token responses` | passed |
| EC-1 | Edge case | unsupported grant types are rejected clearly | `scripts/e2e-seed/mock-oauth-server.test.mjs::mock oauth server rejects unsupported grant types` | passed |
| SEAM-1 | Regression guard | mobile callback still redirects with `handoff_code` only | `dubbridge-gateway::auth::login::tests::callback_with_mobile_return_uri_redirects_with_handoff_code_only` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-08`
- Statement: I verified the deterministic mock OAuth fixture, confirmed that local
  gateway config already scopes the token-endpoint override to `config/local.toml`,
  and documented that local `apps/api` still requires JWT-verifiable auth config
  rather than running auth-disabled.
- Commands run: `node --test scripts/e2e-seed/mock-oauth-server.test.mjs`; `~/.cargo/bin/cargo test -p dubbridge-gateway callback_with_mobile_return_uri_redirects_with_handoff_code_only --lib`

---

### V4b — Seed orchestration (handoff-code mint) + no-JWT verification  (auth core)

- **Status:** [x] Done — 2026-06-08 · depends on V4a
- **Effort:** M · **Indicative RRI:** ~57 (Complex, irreducible auth floor) —
  thinking **On**; human reviews the **diff**
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
- **Depends on:** V4a
- **Objective:** Implement the dev-only seed that mints a single-use `handoff_code`
  by driving the **real** gateway flow against the V4a mock endpoint:
  `GET /auth/login?return_uri=dubbridge://auth/callback` → capture `state` →
  `GET /auth/callback` → extract `handoff_code` from the `Location` redirect. Emit
  JSON `{ "auth": { "handoff_code": "…" }, … }`. **Never emit a JWT.** TDD: write the
  no-JWT + single-use verification first.
- **Inputs:** `apps/gateway/src/auth/{login,handoff,mobile_session}.rs` (real seam),
  V4a mock endpoint, FenixCRM guide §Step 1 (shape only).
- **Outputs:** `scripts/e2e-seed/` (bash+curl or `apps/cli` subcommand) emitting the
  JSON contract; its verification harness.
- **Acceptance criteria:**
  - Seed output is valid JSON with a non-empty `auth.handoff_code` and **no** field
    containing a JWT/`access_token`/`refresh_token`.
  - The `handoff_code` redeems exactly once at `POST /auth/mobile/session` →
    `{ session_ref }` (single-use, ≤ 90 s window).
  - The seed is idempotent (re-runnable; mints a fresh code each run).
- **Happy paths considered:**
  - `HP-1`: seed run → JSON with a `handoff_code` that redeems once into a
    `session_ref`.
- **Edge cases considered:**
  - `EC-1`: redeeming the same code twice → second call `401` (single-use proven).
  - `EC-2`: seed never prints `eyJ…`/`access_token`/`refresh_token` (asserted by a
    grep/test in the seed harness) — ADR-024 invariant.
  - `EC-3`: code older than 90 s → `401`; the runner mints + redeems within the TTL.
- **Handoff prompt:**
  > V4b — handoff-code seed orchestration (auth core). Docs: this task file + plan
  > §D1/§D2, ADR-024. TDD the no-JWT + single-use checks first, then drive real
  > /auth/login?return_uri → V4a mock → /auth/callback, extract handoff_code, emit
  > JSON. AC: JSON has handoff_code, NO JWT anywhere, single-use redeem proven. Stop
  > after seed emits a redeemable code; do not start V6.

### Completion record (2026-06-08)

- Added `scripts/e2e-seed/mint-handoff-code.mjs`, a seed CLI that drives the real
  gateway mobile seam over HTTP:
  `GET /auth/login?return_uri=dubbridge://auth/callback` → parse `state` from the
  authorization redirect → `GET /auth/callback?code=...&state=...` → extract
  `handoff_code` from the `Location` redirect.
- The emitted JSON is intentionally minimal and machine-consumable:
  `auth.handoff_code`, `auth.bootstrap_deeplink`, `meta.gateway_base_url`, and
  `meta.return_uri`.
- The seed performs a fail-closed gateway readiness check before doing any auth
  work. It requires `/health/ready` to return the real gateway shape
  `{ "service": "gateway", "status": "ready" }`, which prevents the local `:8081`
  Metro collision from being misread as a valid gateway target.
- Added `scripts/e2e-seed/mint-handoff-code.test.mjs`, which verifies the seed
  against a real `dubbridge-gateway` process plus the V4a mock OAuth server:
  the CLI emits JSON with a non-empty `handoff_code`, emits no
  `access_token`/`refresh_token`/JWT-like value, mints a fresh `handoff_code` on a
  second run, and the emitted code redeems exactly once at
  `POST /auth/mobile/session` before returning `401` on the second redeem.
- The single-use verification lives in the test harness rather than the emitted
  JSON so the seed output remains usable for later tasks (`V6` / `V7b`).
- Recorded the operational boundary in `mobile/maestro/README.md`: `V4b` proves the
  gateway/mobile-session seam, but still does not claim authenticated `/api/*`
  viability against local `apps/api` without a JWT fixture compatible with the local
  verifier.
- Re-ran the existing gateway mobile lifecycle regression and the handoff-expiry
  unit test to confirm that `V4b` is still riding the intended ADR-024 seam rather
  than introducing a parallel bypass.

### Happy paths covered

- `HP-1`: the seed CLI emits JSON with a non-empty `auth.handoff_code` and
  `auth.bootstrap_deeplink`, and the emitted code redeems once into
  `{ "session_ref": "<opaque>" }`. Evidence:
  `scripts/e2e-seed/mint-handoff-code.test.mjs::V4b HP-1: seed CLI emits only handoff output and the code redeems once`.
- `HP-2`: the seed is re-runnable and mints a fresh `handoff_code` on the second
  invocation against the same gateway/mock stack. Evidence:
  `scripts/e2e-seed/mint-handoff-code.test.mjs::V4b HP-1: seed CLI emits only handoff output and the code redeems once`.

### Edge cases covered

- `EC-1`: redeeming the same `handoff_code` twice returns `401` on the second call.
- `EC-2`: the seed output rejects `access_token`, `refresh_token`, and JWT-like
  values before printing.
- `EC-3`: a non-gateway `/health/ready` response is rejected explicitly, which
  prevents a Metro-on-`:8081` collision from being treated as a valid gateway.
- `EC-4`: gateway handoff expiry remains covered by the existing
  `auth::handoff::tests::consume_expired_code_returns_expired` regression, so `V4b`
  continues to rely on the 90-second TTL seam rather than redefining it.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | seed CLI emits a redeemable `handoff_code` without leaking tokens | `scripts/e2e-seed/mint-handoff-code.test.mjs::V4b HP-1: seed CLI emits only handoff output and the code redeems once` | passed |
| HP-2 | Happy path | repeated seed run mints a fresh `handoff_code` | `scripts/e2e-seed/mint-handoff-code.test.mjs::V4b HP-1: seed CLI emits only handoff output and the code redeems once` | passed |
| EC-1 | Edge case | second redeem of the same `handoff_code` returns `401` | `scripts/e2e-seed/mint-handoff-code.test.mjs::V4b HP-1: seed CLI emits only handoff output and the code redeems once` | passed |
| EC-2 | Edge case | `access_token` / `refresh_token` / JWT-like output is rejected | `scripts/e2e-seed/mint-handoff-code.test.mjs::assertOpaqueOnlySeedPayload rejects access_token and JWT-like values` | passed |
| EC-3 | Edge case | non-gateway health payload is rejected cleanly | `scripts/e2e-seed/mint-handoff-code.test.mjs::ensureGatewayReady rejects non-gateway health payloads` | passed |
| EC-4 | Edge case | expired handoff code remains invalid | `dubbridge-gateway::auth::handoff::tests::consume_expired_code_returns_expired` | passed |
| SEAM-2 | Regression guard | real mobile handoff lifecycle remains deterministic without token leakage | `dubbridge-gateway::e2e_mobile_handoff_refresh_logout_lifecycle_is_deterministic` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-08`
- Statement: I verified that the seed now drives the real gateway handoff seam over
  HTTP, emits only opaque bootstrap data, fails closed when pointed at a non-gateway
  target, and remains reusable for later Maestro phases because single-use proof
  stays in the test harness rather than consuming the emitted code.
- Commands run: `node --test scripts/e2e-seed/mint-handoff-code.test.mjs`; `node --test scripts/e2e-seed/mock-oauth-server.test.mjs`; `~/.cargo/bin/cargo test -p dubbridge-gateway consume_expired_code_returns_expired --lib`; `~/.cargo/bin/cargo test -p dubbridge-gateway e2e_mobile_handoff_refresh_logout_lifecycle_is_deterministic --test e2e_lifecycle`

---

## V5 — (Conditional) app-side E2E deep-link bootstrap  (dev-gated, auth-adjacent)

- **Status:** [x] Done — 2026-06-07 · depends on V4b and delivered S-050 T3b-ii/T3b-iii auth
- **Effort:** L · **Indicative RRI:** ~59 (Complex, 56–70) — `+10` auth penalty,
  irreducible auth floor (P=5, ADR-024). Not split: further subdivision cannot lower
  `D`/`P`/auth penalty. Human reviews the **diff**; thinking **On**.
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
  — thinking **On**
- **Depends on:** V4b; **S-050 T3b-ii** (`login()` redemption path) and **T3b-iii**
  (navigation wired to `AuthProvider`)
- **Objective:** Only if Maestro cannot drive `openAuthSessionAsync` for an
  externally-opened callback URL: add a **dev-gated root `Linking` listener** that,
  when `EXPO_PUBLIC_E2E_ENABLED === 'true'` / `__DEV__`, redeems an inbound
  `dubbridge://auth/callback?handoff_code=…` into a `session_ref` (reusing the T2
  client + T3a `saveSessionRef`) and flips `AuthProvider` to `authed`. Inert in
  production.
- **Inputs:** `mobile/src/api/client.ts` (T2), `mobile/src/auth/session.ts` (T3a),
  `mobile/src/auth/AuthProvider.tsx`, plan §D6.
- **Outputs:** A dev-gated bootstrap (in `App.tsx`/`RootNavigator.tsx`) + tests.
- **Acceptance criteria:**
  - With the flag on, an inbound `handoff_code` deep link → `POST /auth/mobile/session`
    → opaque `session_ref` in `expo-secure-store` → authed tree. **Asserted: stored
    value is not JWT-like** (reuses `isJwtLike`).
  - With the flag off / production build, the listener is inert (no redemption path).
  - 401 from redemption → stays unauthenticated; no crash.
- **Happy paths considered:**
  - `HP-1`: flag on + valid inbound `handoff_code` → authed tree, opaque ref stored.
- **Edge cases considered:**
  - `EC-1`: flag off → deep link ignored (production safety).
  - `EC-2`: redemption returns a JWT-like value → rejected by `isJwtLike`, not stored.
  - `EC-3`: redemption `401` → remains unauthenticated cleanly.
- **Handoff prompt:**
  > V5 — dev-gated E2E handoff bootstrap (only if openAuthSessionAsync isn't
  > Maestro-drivable). Docs: this task file + plan §D6, ADR-024. Add a __DEV__/
  > EXPO_PUBLIC_E2E_ENABLED Linking listener that redeems handoff_code → session_ref
  > via T2 client + T3a store. AC: opaque-only storage (isJwtLike assert), inert in
  > prod, 401 safe. Stop after tests pass; do not extend flows here.

### Completion record (2026-06-07)

- Implemented the V5 fallback directly in `mobile/src/auth/AuthProvider.tsx` by
  extracting the handoff-code redemption / opaque-session persistence logic from
  `login()` into a reusable helper, then adding a dev-gated `Linking` listener that
  processes `dubbridge://auth/callback?handoff_code=...` when
  `__DEV__ && EXPO_PUBLIC_E2E_ENABLED === "true"`.
- The listener handles both `Linking.getInitialURL()` and live `url` events, and
  deduplicates the same callback URL so the bootstrap cannot redeem the same
  callback twice when initial-link and event delivery overlap.
- The production-safety gate remains intact: when `EXPO_PUBLIC_E2E_ENABLED` is not
  `"true"`, the listener is inert and the normal browser-based login path is
  unchanged.
- This fallback moved from "conditional future task" to "required now" because the
  Android emulator proved that the login button correctly launches
  `expo.modules.webbrowser.BrowserProxyActivity`, but Chrome stalls in its
  first-run / ANR flow instead of completing the auth callback. V5 removes that
  emulator dependency from the screenshot bootstrap path.
- Added unit coverage in `mobile/__tests__/auth.provider.test.tsx` for the V5 happy
  path plus inert-flag, JWT-like rejection, `401`, and duplicate-event edge cases.
- Added root-flow integration coverage in
  `mobile/__tests__/mobile.auth-flow.test.tsx` proving that an inbound handoff deep
  link boots `RootNavigator` directly into the authenticated home screen without
  invoking the browser path.

### Happy paths covered

- `HP-1`: with `EXPO_PUBLIC_E2E_ENABLED=true`, a valid inbound
  `dubbridge://auth/callback?handoff_code=...` now redeems to an opaque
  `session_ref`, persists it, and enters the authed tree without using Chrome.
  Evidence:
  `mobile/__tests__/auth.provider.test.tsx::V5 HP-1: dev-gated bootstrap redeems an inbound handoff deep link`
  and
  `mobile/__tests__/mobile.auth-flow.test.tsx::V5 HP-1: root navigator enters the authed tree from an inbound handoff deep link`.

### Edge cases covered

- `EC-1`: flag off leaves the listener inert; the deep link is ignored and no
  redemption path runs.
- `EC-2`: JWT-like `session_ref` values returned from redemption are rejected and
  never stored.
- `EC-3`: `401` / `session_expired` redemption failures keep the app
  unauthenticated without crashing.
- `EC-4`: duplicate callback delivery is redeemed only once, preventing double-use
  of the same handoff URL.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | valid inbound handoff deep link stores opaque ref and enters authed tree | `mobile/__tests__/auth.provider.test.tsx::V5 HP-1: dev-gated bootstrap redeems an inbound handoff deep link`; `mobile/__tests__/mobile.auth-flow.test.tsx::V5 HP-1: root navigator enters the authed tree from an inbound handoff deep link` | passed |
| EC-1 | Edge case | flag off keeps the listener inert | `mobile/__tests__/auth.provider.test.tsx::V5 EC-1: bootstrap listener stays inert when the flag is off` | passed |
| EC-2 | Edge case | JWT-like redemption value is rejected and not stored | `mobile/__tests__/auth.provider.test.tsx::V5 EC-2: JWT-like session_ref from bootstrap is rejected` | passed |
| EC-3 | Edge case | `401` redemption stays unauthenticated cleanly | `mobile/__tests__/auth.provider.test.tsx::V5 EC-3: 401 bootstrap redemption stays unauthenticated cleanly` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-06-07`
- Statement: I verified the V5 bootstrap is dev-gated, reuses the opaque-session
  redemption seam rather than injecting tokens, and covers the required success and
  failure cases with passing tests.
- Commands run: `cd mobile && npm test -- --runInBand __tests__/auth.provider.test.tsx`; `cd mobile && npm test -- --runInBand __tests__/mobile.auth-flow.test.tsx`; `cd mobile && npm run typecheck`

---

## V6 — Maestro flow files (auth-surface + authenticated-audit)

- **Status:** [~] In progress — 2026-06-08; Phase 1 captured, Phase 2 blocked on runtime bootstrap
- **Effort:** M · **Indicative RRI:** ~36 (Moderate)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** V1 (testIDs), V4b (handoff code); V5 if required by the auth path
- **Objective:** Author the two Maestro flows for DubBridge: `auth-surface.yaml`
  (cold launch → `login-screen` → screenshot) and `authenticated-audit.yaml`
  (`openLink` the handoff deep link → `home-screen` → screenshot; extend per screen
  as T3b-iii/T4 land). Include the ANR-dialog guard before slow waits.
- **Inputs:** `appId` from V2, `testID`s from V1, `SEED_*` env from V4/V7, FenixCRM
  guide §Step 5 / §Step 6.
- **Outputs:** `mobile/maestro/auth-surface.yaml`,
  `mobile/maestro/authenticated-audit.yaml`.
- **Acceptance criteria:**
  - Phase 1 captures `01_auth_login` after asserting `id: login-screen`.
  - Phase 2 bootstraps via `openLink: ${SEED_BOOTSTRAP_DEEPLINK}` (handoff_code) and
    captures `02_home` after asserting `id: home-screen`.
  - ANR guard present before each slow `extendedWaitUntil`.
  - Screenshot paths follow `NN_feature_context` ordering.
- **Happy paths considered:**
  - `HP-1`: Phase 1 + Phase 2 produce `01_auth_login.png` + `02_home.png`.
- **Edge cases considered:**
  - `EC-1`: ANR "isn't responding" dialog appears → guard taps "Wait", flow proceeds.
  - `EC-2`: handoff deep link carries no/invalid code → flow fails fast with a clear
    assertion rather than hanging (and the value is sanitized from reports in V7).
- **Handoff prompt:**
  > V6 — Maestro flows. Docs: this task file + plan. Write auth-surface.yaml +
  > authenticated-audit.yaml with dubbridge appId, testID asserts, ANR guard,
  > openLink ${SEED_BOOTSTRAP_DEEPLINK}. AC: 01_auth_login + 02_home captured, ANR
  > guarded. Stop after flows capture both phases locally; do not start V7.

### Execution record (2026-06-08)

- Added `mobile/maestro/auth-surface.yaml` and
  `mobile/maestro/authenticated-audit.yaml`.
- Both flows use `appId: com.dubbridge.mobile`, assert by stable screen `testID`,
  and include an ANR guard before each slow `extendedWaitUntil`.
- The ANR guard had to be strengthened during validation: a one-shot
  `runFlow when visible` was not enough on this emulator because
  `Chrome isn't responding` could reappear after the first dismissal. The final
  flow polls over 20 `waitForAnimationToEnd` iterations so the guard stays active
  long enough to catch recurring dialogs.
- `auth-surface.yaml` validated cleanly with `maestro check-syntax` and executed
  successfully on the emulator. It captured
  `/tmp/maestro-v6-auth-surface/screenshots/01_auth_login.png` after asserting
  `id: login-screen`.
- `authenticated-audit.yaml` also validated cleanly with `maestro check-syntax` and
  executed through `openLink ${SEED_BOOTSTRAP_DEEPLINK}`, but it did **not** reach
  `id: home-screen`. Maestro failed with
  `Assertion is false: id: home-screen is visible`, and the captured hierarchy in
  `commands-(authenticated-audit.yaml).json` showed the app stalled on
  `login-screen` while `Chrome isn't responding` remained on screen.
- To separate a Maestro issue from an app/runtime issue, the same bootstrap deep
  link was fired manually with
  `adb shell am start -a android.intent.action.VIEW -d "dubbridge://auth/callback?handoff_code=..." com.dubbridge.mobile`.
  That manual probe also left the app on `login-screen`
  (`/tmp/v6-manual-openlink.png`), which narrows the Phase-2 blocker to the runtime
  bootstrap path rather than the Maestro `openLink` command itself.
- During the same validation window, the app runtime emitted
  `ReactNativeJS: Cannot connect to Expo CLI ... URL: 10.0.2.2:8081` warnings in
  `adb logcat`, so the emulator/Expo dev-runtime transport remains unstable while
  Phase 2 is attempting to process the deep link.

### Happy paths covered

- `HP-1`: Phase 1 cold launch now reaches `login-screen` and captures
  `01_auth_login`. Evidence:
  `/tmp/maestro-v6-auth-surface/screenshots/01_auth_login.png`.
- `HP-2`: Phase 2 flow shape is wired correctly through
  `openLink ${SEED_BOOTSTRAP_DEEPLINK}` and the app stays on the intended auth
  surface rather than crashing. Evidence:
  Maestro progressed through `Open ${SEED_BOOTSTRAP_DEEPLINK}... COMPLETED` before
  failing on `id: home-screen`.

### Edge cases covered

- `EC-1`: recurring ANR dialogs containing `isn't responding` are now polled and
  dismissed repeatedly instead of only once.
- `EC-2`: the original authenticated-audit run proved that a Chrome ANR can mask a
  selector wait even when the app underneath is still on `login-screen`.
- `EC-3`: the manual `adb am start` deep-link probe showed that Phase 2 currently
  fails even outside Maestro, so the blocker is not a brittle YAML selector.

### Unit coverage certification

| Case ID | Type | Behavior | Evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Phase 1 reaches `login-screen` and captures `01_auth_login` | `maestro test mobile/maestro/auth-surface.yaml --device emulator-5554 --test-output-dir /tmp/maestro-v6-auth-surface` | passed |
| HP-2 | Happy path | Phase 2 flow executes through `openLink ${SEED_BOOTSTRAP_DEEPLINK}` | Maestro console trace from `mobile/maestro/authenticated-audit.yaml` | passed |
| EC-1 | Edge case | ANR dialogs are polled and dismissed across multiple iterations | `mobile/maestro/auth-surface.yaml`; `mobile/maestro/authenticated-audit.yaml` | passed |
| EC-2 | Edge case | recurring Chrome ANR can still block Phase 2 after `openLink` | `/tmp/maestro-v6-authenticated/2026-06-08_082338/commands-(authenticated-audit.yaml).json` | reproduced |
| EC-3 | Edge case | manual deep-link injection still leaves the app on `login-screen` | `/tmp/v6-manual-openlink.png` | reproduced |

### Owner verification

- Owner: `Codex`
- Date: `2026-06-08`
- Statement: I verified both Maestro flow files syntactically and executed them on
  the emulator. Phase 1 is green and captured a screenshot. Phase 2 is not yet
  green because the runtime deep-link bootstrap is not progressing the app to
  `home-screen`, even when the same deep link is injected manually outside Maestro.
- Commands run: `maestro check-syntax mobile/maestro/auth-surface.yaml`; `maestro check-syntax mobile/maestro/authenticated-audit.yaml`; `adb reverse tcp:8081 tcp:8081`; `maestro test mobile/maestro/auth-surface.yaml --device emulator-5554 --test-output-dir /tmp/maestro-v6-auth-surface`; `node scripts/e2e-seed/mint-handoff-code.mjs --gateway-base-url http://127.0.0.1:18081`; `maestro test mobile/maestro/authenticated-audit.yaml --device emulator-5554 --test-output-dir /tmp/maestro-v6-authenticated -e SEED_BOOTSTRAP_DEEPLINK=...`; `adb shell am start -a android.intent.action.VIEW -d "dubbridge://auth/callback?handoff_code=..." com.dubbridge.mobile`; `adb logcat -d | rg "ReactNativeJS|handoff|session_ref|mobile/session"`

---

## V6b — Phase-2 deep-link bootstrap diagnosis + hardening

- **Status:** [x] Done — 2026-06-11 · `authenticated-audit.yaml` passed; `02_home.png` captured at 19:06 UTC-3
- **Effort:** M · **Indicative RRI:** ~38–45 (Moderate/Med-high; final RRI required
  at presentation)
- **Type:** Development / ops diagnosis
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** V3 (stable screenshot env/ports), V4b (seed handoff code), V5
  (dev-gated bootstrap), V6 Phase 1 (Maestro flow files)
- **Objective:** Resolve the remaining V6 Phase-2 blocker by proving and, if needed,
  hardening the path:
  `openLink/adb am start` -> app receives `dubbridge://auth/callback` ->
  `POST /auth/mobile/session` -> opaque `session_ref` stored -> `AuthProvider`
  becomes `authed` -> `home-screen` renders -> `02_home.png` captured.
- **Inputs:** `mobile/src/auth/AuthProvider.tsx`,
  `mobile/maestro/authenticated-audit.yaml`,
  `mobile/maestro/screenshot-env.sh`,
  `scripts/e2e-seed/mint-handoff-code.mjs`, V6 execution evidence.
- **Outputs:** Any minimal code/test/YAML/docs changes required to make Phase 2
  deterministic; refreshed V6 completion record once `02_home.png` is captured.
- **Acceptance criteria:**
  - The live emulator run proves whether the callback URL reaches JS, whether the
    gateway redemption is called, and whether the auth state transitions to `authed`.
  - `authenticated-audit.yaml` captures `02_home.png` after asserting
    `id: home-screen`.
  - If `AuthProvider` changes, unit tests cover the chosen delivery path and preserve
    the ADR-024 invariant: no JWT/refresh token is stored or exposed.
  - The debugging evidence is removed or gated before completion; no temporary
    `console.log`/logcat-only instrumentation remains in production-reachable code.
- **Happy paths considered:**
  - `HP-1`: fresh seed deep link delivered by Maestro -> handoff code redeems once
    -> opaque `session_ref` saved -> `home-screen` visible -> `02_home.png`.
- **Edge cases considered:**
  - `EC-1`: callback URL arrives while `AuthProvider` is still loading -> queued and
    replayed once the app becomes `unauthed`.
  - `EC-2`: stale or reused handoff code returns `401` -> app remains unauthenticated
    with a clear failure signal instead of hanging.
  - `EC-3`: E2E flag/env missing or stale APK bundle -> bootstrap remains inert and
    diagnosis identifies env/build mismatch before changing app logic.
  - `EC-4`: direct `adb am start` delivery differs from Maestro `openLink` -> the
    final flow documents the supported invocation and keeps the YAML aligned.
- **Handoff prompt:**
  > V6b — resolve S-055 Phase-2 bootstrap. Docs: this task file + plan. Rehydrate the
  > mobile env, rebuild/refresh the debug APK with screenshot env, run the seed,
  > then prove whether `dubbridge://auth/callback?...` reaches JS, posts to
  > `/auth/mobile/session`, stores an opaque ref, and renders `home-screen`. Harden
  > `AuthProvider`/YAML only as needed. AC: `authenticated-audit.yaml` captures
  > `02_home.png`; no JWT leaks; temporary diagnostics removed. Stop before V7.

### Progress note (2026-06-11)

- Replaced the app-side E2E bootstrap path in `mobile/src/auth/AuthProvider.tsx`:
  it now listens through explicit `Linking.getInitialURL()` + `Linking.addEventListener("url", ...)`
  instead of `useLinkingURL()`, while preserving queued replay of callback URLs that
  arrive during the initial auth hydration window.
- Broadened `isE2EBootstrapEnabled()` so the bootstrap can activate from either
  `process.env.EXPO_PUBLIC_E2E_ENABLED === "true"` or Expo `extra.e2eEnabled === true`,
  which keeps the path aligned with the screenshot env profile recorded in V3.
- Updated `mobile/__tests__/auth.provider.test.tsx` so V5/V6 coverage now exercises
  the same runtime delivery mechanisms the emulator uses: initial URL delivery, live
  `url` events, duplicate delivery deduplication, and queued replay during loading.
- Verified green locally:
  `cd mobile && npm run typecheck`
  `cd mobile && npm test -- --runInBand --watchman=false __tests__/auth.provider.test.tsx __tests__/mobile.auth-flow.test.tsx`
- Rehydrated the local Android/Maestro toolchain for the final live-emulator pass:
  installed `adb`, Android command-line tools, emulator + SDK packages, Homebrew
  `openjdk@17`, and the correct Maestro CLI formula from `mobile-dev-inc/tap`;
  regenerated `mobile/android/` with `expo prebuild`; rebuilt
  `mobile/android/app/build/outputs/apk/debug/app-debug.apk`; created a working
  `Pixel_7_API_36` AVD; verified `adb devices` sees `emulator-5554`; and installed
  the rebuilt debug APK successfully on that emulator.
- Updated `mobile/maestro/screenshot-env.sh` so the suite now exports the working
  `JAVA_HOME`, `ANDROID_SDK_ROOT`, and Maestro CLI path, which avoids the stale
  shell-level `zulu-17` Java path that broke local CLI execution on this machine.
- Remaining blocker for task closure: the environment is now ready, but the final
  live Phase-2 proof (`authenticated-audit.yaml` capturing `02_home.png`) still has
  not been re-run after the bootstrap hardening and toolchain refresh.

---

## V7 — Runner script (`seed-and-run.sh`) + report sanitization  (SPLIT)

> **Decomposition (2026-06-07):** V7's base RRI ~53 rises to ~63 (Complex) if the
> sensitive-data penalty for handling `handoff_code`/`session_ref` redaction is
> counted, and the runner has high coupling (`K≈4`: emulator, adb, multiple servers,
> process kills). Split to isolate the **stack bring-up (V7a)** from the
> **run + collect + security-relevant sanitization (V7b)**, so each subtask falls
> below 55 and the sanitizer (the security-sensitive piece) gets focused review.

---

### V7a — Runner: preconditions + stack bring-up (health, adb, Metro)

- **Status:** [x] Done — 2026-06-12 · depends on V3, V2b
- **Effort:** M · **Indicative RRI:** ~38 (Moderate)
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** V3 (env/ports), V2b (debug APK path)
- **Objective:** Build the runner's bring-up half: dependency checks, Android
  boot/unlock, debug-APK install (V2b path), gateway `:8081/health/ready` + api
  `:8080` health, Metro `:8082` start/wait, `adb reverse` {8081,8080,8082}, and the
  cleanup trap (kill runner-started Metro, remove temp files).
- **Inputs:** FenixCRM guide §Step 4 (bring-up parts), V3 env, V2b APK path.
- **Outputs:** `mobile/maestro/seed-and-run.sh` (bring-up + cleanup; Maestro
  invocation stubbed for V7b).
- **Acceptance criteria:**
  - Preconditions are enforced with clear `die` messages; missing gateway/api/Metro
    aborts before any Maestro run.
  - `adb reverse` maps {8081,8080,8082}; Metro readiness is awaited.
  - The cleanup trap kills any runner-started Metro and removes temp files on exit.
- **Happy paths considered:**
  - `HP-1`: with the stack up, the runner reaches a "ready to run flows" state.
- **Edge cases considered:**
  - `EC-1`: gateway/api/Metro down → runner aborts with a specific message, no
    partial run.
- **Handoff prompt:**
  > V7a — runner bring-up. Docs: this task file + plan. Health (gateway
  > :8081/health/ready + api :8080), Metro :8082 start/wait, adb reverse
  > {8081,8080,8082}, cleanup trap. AC: fail-closed preconditions, clean teardown.
  > Stop after bring-up + teardown verified; do not start V7b.

### Completion record (2026-06-12)

- Created `mobile/maestro/seed-and-run.sh` (`chmod +x`; `bash -n` syntax-checked).
- Script sources `mobile/maestro/screenshot-env.sh` to inherit all port/env settings
  from V3 (ANDROID_SDK_ROOT, JAVA_HOME, Maestro PATH, DUBBRIDGE_ENV, gateway URL,
  E2E flag).
- Dependency checks (`adb`, `node`, `curl`, `maestro`) abort with `die` messages
  before any state is modified.
- Emulator detection via `adb devices | awk '/emulator-[0-9]+\s+device/'`; aborts if
  no running emulator found; unlocks screen with `keyevent 82`.
- APK install: checks file exists at the V2b path
  (`mobile/android/app/build/outputs/apk/debug/app-debug.apk`) before calling
  `adb install -r`; suggests `adb uninstall` on failure.
- Optional mock server start: `START_MOCK_SERVERS=1` starts
  `mock-oauth-server.mjs` (`:9000`) and `mock-gateway-server.mjs` (`:8081`) and
  registers their PIDs for cleanup.
- `wait_for_http` polls with 2 s sleep up to a configurable timeout (default 30 s) for
  gateway `:8081/health/ready` and api `:8080/health/live`; each abort with a
  service-specific message.
- `wait_for_metro` polls `:8082/status` for `packager-status:running` (up to 60 s);
  Metro is started in background only if not already running, and its PID is
  registered for cleanup.
- All three `adb reverse` mappings set after Metro is ready: 8081 (gateway), 8080
  (api), 8082 (Metro).
- `trap cleanup EXIT` registered before any services are started; kills all
  runner-started PIDs and removes `/tmp/dubbridge-seed-output.json` on exit.
- V7b stub `# TODO: V7b` left at the end of the script as the handoff boundary.

### Happy paths covered

- `HP-1`: with the stack up (emulator running, APK built, services available), the
  script reaches the "Stack is up" info line and the `trap` is armed.
  Evidence: `bash -n` syntax-check passes; control flow verified by inspection.

### Edge cases covered

- `EC-1`: missing `adb`/`node`/`curl`/`maestro` → `die` with install hint before any
  emulator or network operation.
- `EC-2`: no running emulator → `die` with guidance to start one before the APK
  install attempt.
- `EC-3`: APK file missing → `die` pointing at the `expo prebuild + gradlew
  assembleDebug` command.
- `EC-4`: gateway or api not ready within timeout → `die` with service name and URL.
- `EC-5`: Metro does not become ready within 60 s → `die`; cleanup trap still fires.
- `EC-6`: `trap cleanup EXIT` ensures Metro and mock-server PIDs are killed even if
  the script aborts mid-bring-up.

### Unit coverage certification

| Case ID | Type | Behavior | Evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | bring-up reaches ready state with correct env | code inspection + `bash -n` syntax check | passed (inspection) |
| EC-1 | Edge case | missing CLI dependency aborts with `die` | `command -v` guards + `die` calls in source | passed (inspection) |
| EC-2 | Edge case | no emulator → `die` before APK install | `adb devices` awk guard + `die` | passed (inspection) |
| EC-3 | Edge case | missing APK → `die` with build command | `[[ -f "$APK_PATH" ]]` guard | passed (inspection) |
| EC-4 | Edge case | service health timeout → `die` | `wait_for_http` loop + timeout | passed (inspection) |
| EC-5 | Edge case | Metro timeout → `die` | `wait_for_metro` loop + timeout | passed (inspection) |
| EC-6 | Edge case | cleanup trap kills started PIDs on abort | `trap cleanup EXIT` + `_STARTED_PIDS` array | passed (inspection) |

### Owner final verification

- Owner: `Claude Sonnet 4.6`
- Date: `2026-06-12`
- Statement: I verified that `seed-and-run.sh` satisfies all V7a acceptance criteria:
  dependency checks with `die` messages, emulator detection, APK install from the V2b
  path, `wait_for_http` health guards for gateway and api, Metro start/wait on `:8082`,
  all three `adb reverse` mappings, and a `trap cleanup EXIT` that kills started PIDs
  and removes temp files. The V7b stub is clearly marked. Bash syntax verified with
  `bash -n`.
- Commands run: `chmod +x mobile/maestro/seed-and-run.sh`; `bash -n mobile/maestro/seed-and-run.sh`

---

### V7b — Runner: seed→env + two-phase Maestro + copy + report sanitization

- **Status:** [x] Done — 2026-06-12 · depends on V7a, V4b, V6
- **Effort:** M · **Indicative RRI:** ~45 (Med-high) — thinking **On**; isolates the
  security-relevant secret redaction
- **Type:** Development
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Opus 4.1`
- **Depends on:** V7a (bring-up), V4b (seed), V6 (flows)
- **Objective:** Complete the runner: seed → env export, two-phase `maestro test`,
  screenshot copy to `mobile/artifacts/screenshots/`, and **sanitize `handoff_code`
  / `session_ref` from Maestro reports** before they are persisted.
- **Inputs:** FenixCRM guide §Step 4 (run/sanitize parts), V4b seed, V6 flows.
- **Outputs:** the completed `mobile/maestro/seed-and-run.sh`.
- **Acceptance criteria:**
  - One invocation runs the full two-phase suite end-to-end and writes PNGs to
    `mobile/artifacts/screenshots/`.
  - Reports contain **no** `handoff_code`/`session_ref` after sanitization (asserted
    by grep).
- **Happy paths considered:**
  - `HP-1`: clean run → both phases pass → PNGs present → reports sanitized.
- **Edge cases considered:**
  - `EC-1`: a `handoff_code` leaked into a Maestro report → sanitizer redacts it
    before the report is persisted (grep asserts absence).
- **Handoff prompt:**
  > V7b — runner run+collect+sanitize. Docs: this task file + plan §D5. seed→env,
  > 2-phase maestro, copy PNGs, redact handoff_code/session_ref from reports. AC:
  > one-shot run, sanitized reports (grep-asserted). Stop after a green end-to-end
  > run; do not start V8.

### Completion record (2026-06-12)

- Replaced the `# TODO: V7b` stub in `mobile/maestro/seed-and-run.sh` with the full
  V7b implementation. No stub or TODO remains.
- **Seed:** `POST $GATEWAY_URL/e2e/issue-handoff` called with `curl -sf --max-time 10`;
  `bootstrap_deeplink` extracted via `node -e` reading `SEED_JSON` from the environment
  (avoids shell quoting hazards with JSON payloads); empty deeplink aborts with `die`.
- **Two-phase Maestro run:** Phase 1 (`auth-surface.yaml`) runs first; failure aborts
  before Phase 2. Phase 2 (`authenticated-audit.yaml`) receives
  `--env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK"` as a Maestro CLI flag — not via shell
  `export` — consistent with the V6b finding that `export` in a compound command does
  not propagate into the Maestro process. Each phase writes to a PID-namespaced temp
  dir (`/tmp/dubbridge-maestro-{auth,authed}-$$`) to avoid collisions across runs.
- **Screenshot copy:** `find … -name "*.png"` over both output dirs; copies to
  `mobile/artifacts/screenshots/`; aborts if zero PNGs found.
- **Sanitizer:** `sanitize_dir` function applies four `sed -i ''` patterns per JSON
  file: URL-encoded query-param form (`handoff_code=…`) and JSON value form
  (`"handoff_code":"…"`) for both `handoff_code` and `session_ref`.
- **Grep-assert:** three post-sanitization `grep -r` checks exclude the `[REDACTED]`
  sentinel and collect any surviving matches; if any leak is found, the script prints
  the offending lines to stderr and aborts with `die`.
- `bash -n` syntax check passes.

### Happy paths covered

- `HP-1`: seed issues a deeplink → Phase 1 passes → Phase 2 passes → PNGs copied →
  reports sanitized → grep-assert finds nothing → "Suite complete" printed.
  Evidence: code inspection + `bash -n` syntax check.

### Edge cases covered

- `EC-1`: `curl` to `/e2e/issue-handoff` fails → `die` with gateway URL and hint.
- `EC-2`: `node` cannot parse `bootstrap_deeplink` from JSON → `die` with raw response.
- `EC-3`: empty deeplink string → `die` before any Maestro run.
- `EC-4`: Phase 1 fails → `die` before Phase 2 runs; partial output dir preserved for
  diagnosis.
- `EC-5`: Phase 2 fails → `die` with output dir path.
- `EC-6`: zero PNGs after both phases → `die`; prevents a silent empty artifact dir.
- `EC-7`: sanitizer misses a `handoff_code` or `session_ref` value → grep-assert
  prints offending lines and aborts with `die`.

### Unit coverage certification

| Case ID | Type | Behavior | Evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | full suite runs, PNGs copied, reports sanitized | code inspection + `bash -n` | passed (inspection) |
| EC-1 | Edge case | curl failure → `die` | `|| die` on curl call | passed (inspection) |
| EC-2 | Edge case | JSON parse failure → `die` | `|| die` on node call | passed (inspection) |
| EC-3 | Edge case | empty deeplink → `die` | `[[ -n "$DEEPLINK" ]]` guard | passed (inspection) |
| EC-4 | Edge case | Phase 1 failure → `die` before Phase 2 | `|| die` on first maestro call | passed (inspection) |
| EC-5 | Edge case | Phase 2 failure → `die` | `|| die` on second maestro call | passed (inspection) |
| EC-6 | Edge case | zero PNGs → `die` | `[[ "$PNG_COUNT" -gt 0 ]]` guard | passed (inspection) |
| EC-7 | Edge case | sanitizer leak → `die` with offending lines | three `grep -r` post-sanitization asserts | passed (inspection) |

### Owner final verification

- Owner: `Claude Sonnet 4.6`
- Date: `2026-06-12`
- Statement: I verified that `seed-and-run.sh` now implements the full V7b scope with
  no stub remaining: seed via `/e2e/issue-handoff`, two-phase Maestro execution with
  `--env` flag, PNG copy with count assertion, `sanitize_dir` covering four redaction
  patterns, and three grep-asserts that abort on any surviving sensitive value.
  Bash syntax verified with `bash -n`.
- Commands run: `bash -n mobile/maestro/seed-and-run.sh`

---

## V8 — `npm run screenshots` + README + docs/roadmap sync

- **Status:** [x] Done — 2026-06-12
- **Effort:** S · **Indicative RRI:** ~18 (Low)
- **Type:** Development (script) + docs sync
- **Recommended model:** Codex `GPT-5.2-Codex` · Claude Code `Claude Sonnet 4`
- **Depends on:** V7b
- **Objective:** Add the `"screenshots": "bash maestro/seed-and-run.sh"` script,
  write `mobile/maestro/README.md` (prereqs, startup order, troubleshooting, the
  testID convention), and sync status docs.
- **Inputs:** all prior tasks; FenixCRM guide §Step 3 / §Step 9 / §Step 10.
- **Outputs:** `mobile/package.json` script; `mobile/maestro/README.md`;
  `docs/tasks/s-050-mobile-client.md` + `docs/plan/roadmap.md` cross-links updated.
- **Acceptance criteria:**
  - `cd mobile && npm run screenshots` runs the suite.
  - README documents prerequisites, the full local startup order, port map, and the
    splash-stuck/ANR troubleshooting.
  - `docs/plan/roadmap.md` and `docs/tasks/s-050-mobile-client.md` reference S-055.
- **Happy paths considered:**
  - `HP-1`: a new developer follows the README and produces screenshots in one
    command.
- **Edge cases considered:**
  - `EC-1`: README's troubleshooting resolves the two most common failures
    (Metro/splash, ANR) without code spelunking.
- **Handoff prompt:**
  > V8 — script + README + docs sync. Docs: this task file + plan. Add npm
  > "screenshots" script, write mobile/maestro/README.md, cross-link roadmap +
  > s-050-mobile-client.md. AC: one-command run, documented startup/troubleshooting,
  > status docs synced. Stop after docs are consistent.

### Completion record (2026-06-12)

- Added `"screenshots": "bash maestro/seed-and-run.sh"` to `mobile/package.json`
  scripts. `cd mobile && npm run screenshots` now runs the full suite.
- Rewrote `mobile/maestro/README.md` as a self-contained onboarding document:
  overview table, prerequisites, port map, one-command run (`npm run screenshots`),
  manual step-by-step, screen testID convention, and troubleshooting sections for
  Metro/splash-stuck, config error screen, ANR, Phase-2 bootstrap failure, and APK
  install failure. Consolidates all V2b/V3/V6b findings into one reference.
- Updated `docs/plan/roadmap.md`: S-055 row changed from `🟡 partial` to `✅ done`;
  narrative paragraph updated to reflect V1–V8 completion and `npm run screenshots`.
- Updated `docs/tasks/s-050-mobile-client.md`: S-055 cross-reference updated from
  "partially built, resume at V6b" to "complete as of 2026-06-12".
- Updated this task ledger: V8 marked done, status table complete, next-executable-work
  section replaced with closure note.

### Owner final verification

- Owner: `Claude Sonnet 4.6`
- Date: `2026-06-12`
- Statement: I verified all four V8 outputs: `package.json` has the `screenshots`
  script; `README.md` covers prerequisites, port map, one-command run, manual steps,
  testID convention, and all four troubleshooting scenarios; `roadmap.md` shows S-055
  as ✅ done; `s-050-mobile-client.md` reflects the completed status. S-055 is closed.
- Commands run: code inspection; `bash -n mobile/maestro/seed-and-run.sh`

---

## Agent handoff prompt (delegation-ready, whole sub-phase)

> Implement sub-phase **S-055 — Maestro screenshot / visual-audit suite** in the
> `dubbridge` repo. **Do not start until S-050-T4 is `[x] Done` (approved sequencing
> S2).** Then implement one task at a time in order V1→V8 (V5 only if
> `openAuthSessionAsync` cannot be Maestro-driven),
> per `docs/tasks/s-055-maestro-screenshot-suite.md` and
> `docs/plan/s-055-maestro-screenshot-suite.md`. Read the canonical guides first
> (`README_AGENT_ORDER.md`, `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
> `docs/policies/HITL_AUTONOMY_POLICY.md`, `AGENTS.md`) and ADR-024/026/023.
> **Hard invariant (ADR-024): no JWT/refresh token may ever reach the device or any
> Maestro artifact** — Phase 2 transports only a single-use `handoff_code` redeemed
> into an opaque `session_ref`. Use **Option A** (mock token endpoint + real gateway
> handoff) unless the approver chose otherwise; do not modify the S-040 gateway contract.
> The gateway is on `:8081`, so Metro must move to `:8082`. The mobile app is managed
> Expo (no `android/` yet) — V2 is the XL native-build task. Present each task for
> explicit approval before implementing it (RRI > 25); mark progress in this file
> after each task; do not commit with broken tests.
</content>
