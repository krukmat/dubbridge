# Maestro screenshot suite

## Build strategy

V2a decision (2026-06-07):

- Use `npx expo prebuild --platform android` to materialize the managed Expo Android
  project when the screenshot suite needs a debug build.
- Build the APK from the generated native project with
  `cd android && ./gradlew assembleDebug`.
- Do not use `npx expo run:android` as the primary screenshot-build path. It is
  useful for iterative local development, but `prebuild + gradlew assembleDebug`
  gives P3-V a stable, inspectable debug-APK output path for Maestro and separates
  the one-time native-project generation step from the actual APK build step.

## Android project policy

- The generated `android/` project is **not committed**.
- The repository stays on the Expo managed-workflow source of truth
  (`app.config.ts`, package manifests, JS/TS sources).
- `android/` is regenerated on demand and ignored from the repo root via
  `mobile/android/`.

## Recorded identifiers for later tasks

- Android package / Maestro `appId`: `com.dubbridge.mobile`
- Expected debug APK path after `assembleDebug`:
  `mobile/android/app/build/outputs/apk/debug/app-debug.apk`

## Follow-up boundary

- V2a is decision-only. It does **not** run prebuild, Gradle, or emulator launch.
- V2b executes the chosen build path and verifies the app launches past splash.

## Port map

| Service | Host port | adb reverse mapping | Notes |
|---|---|---|---|
| `apps/gateway` | `:8081` | `adb reverse tcp:8081 tcp:8081` | gateway and OAuth callback |
| `apps/api` | `:8080` | `adb reverse tcp:8080 tcp:8080` | REST data for authed screens |
| Metro (JS bundler) | `:8082` | `adb reverse tcp:8082 tcp:8082` | deconflicted from gateway |
| `mock-oauth` | `:9000` | none (host-only) | gateway contacts it directly |

Metro runs on `:8082` — never `:8081` — to avoid a port collision with the gateway.
The debug APK must be built (or rebuilt) after this change so the native Metro
discovery URL matches `:8082`.

## Screenshot environment setup

Source `mobile/maestro/screenshot-env.sh` into your shell before starting Metro:

```sh
. mobile/maestro/screenshot-env.sh
cd mobile && npx expo start --port 8082 --clear
```

The script exports `DUBBRIDGE_ENV=local`,
`EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://localhost:8081`, and
`EXPO_PUBLIC_E2E_ENABLED=true`. It also normalizes local Android/Maestro tooling by
exporting `ANDROID_SDK_ROOT=$HOME/Library/Android/sdk`, a Homebrew `openjdk@17`
`JAVA_HOME`, and `/opt/homebrew/opt/maestro/bin` in `PATH` so Gradle and the
Maestro CLI do not inherit a stale shell-level Java path. It is safe to commit —
no secrets.

## Runtime launch command

- Source the screenshot env and start Metro on the deconflicted port:
  `. mobile/maestro/screenshot-env.sh && cd mobile && npx expo start --port 8082 --clear`

## Mock OAuth fixture for seed work

- `V4a` adds a deterministic local OAuth token fixture at
  `scripts/e2e-seed/mock-oauth-server.mjs`.
- Run it from the repo root before `V4b` seed work:
  `node scripts/e2e-seed/mock-oauth-server.mjs`
- Health check:
  `curl http://127.0.0.1:9000/health/live`
- Local gateway config already targets this seam in [local.toml](/Users/matiasleandrokruk/Documents/dubbridge/config/local.toml:17):
  `authorization_url = "http://localhost:9000/oauth/authorize"` and
  `token_url = "http://localhost:9000/oauth/token"`.
- The fixture returns a deterministic `TokenSet` for both
  `grant_type=authorization_code` and `grant_type=refresh_token`:
  `access_token=fixture-access-token`, `refresh_token=fixture-refresh-token`,
  `expires_in=3600`, `token_type=Bearer`.
- Important local-auth finding for later tasks: `apps/api` does not run
  auth-disabled in local mode. By code inspection, it requires auth settings at
  startup and builds an RSA JWT verifier from the configured public key, so a bare
  opaque fixture access token is not sufficient for authenticated `/api/*` calls.
  `V4b` must either keep its verification at the gateway/mobile-session boundary or
  provide a JWT fixture compatible with the local verifier config.

## Seed handoff-code mint

- `V4b` adds the seed CLI at `scripts/e2e-seed/mint-handoff-code.mjs`.
- Usage:
  `node scripts/e2e-seed/mint-handoff-code.mjs --gateway-base-url http://127.0.0.1:8081`
- Output contract:
  - `auth.handoff_code`
  - `auth.bootstrap_deeplink`
  - `meta.gateway_base_url`
  - `meta.return_uri`
- The seed drives the real gateway flow:
  `GET /auth/login?return_uri=...` → parse `state` from the auth redirect →
  `GET /auth/callback?code=...&state=...` → extract `handoff_code` from the mobile
  `Location` redirect.
- The CLI fails closed if `--gateway-base-url` does not answer
  `/health/ready` as `{ "service": "gateway", "status": "ready" }`. This matters in
  local dev because `:8081` can still be occupied by Metro. If that happens, either
  stop Metro or point `--gateway-base-url` at the actual gateway port.
- The seed output never includes `access_token`, `refresh_token`, or a JWT-like
  value. The single-use redeem proof stays in the verification harness, not in the
  emitted JSON, so the emitted `handoff_code` remains usable by later tasks.

## Dev bootstrap fallback

- If the Android emulator blocks the browser-based login in Chrome first-run / ANR,
  source the screenshot env (which already includes `EXPO_PUBLIC_E2E_ENABLED=true`)
  and start Metro on `:8082`:
  `. mobile/maestro/screenshot-env.sh && cd mobile && npx expo start --port 8082 --clear`
- With that flag on, the app accepts an inbound
  `dubbridge://auth/callback?handoff_code=...` deep link in `__DEV__`, redeems it
  into an opaque `session_ref`, and enters the authed tree without relying on
  Chrome/Custom Tabs.
- Once V4b provides a real seed, the emulator bootstrap command shape is:
  `adb shell am start -a android.intent.action.VIEW -d "dubbridge://auth/callback?handoff_code=<seeded-code>" com.dubbridge.mobile`

## Maestro flows

- `auth-surface.yaml` cold-launches the app, polls for ANR dialogs containing
  `isn't responding`, waits for `id: login-screen`, and captures
  `01_auth_login`.
- `authenticated-audit.yaml` cold-launches the app, waits for `id: login-screen`,
  opens `${SEED_BOOTSTRAP_DEEPLINK}`, polls again for ANR dialogs, waits for
  `id: home-screen`, and captures `02_home`.
- The ANR guard intentionally polls over multiple `waitForAnimationToEnd`
  iterations instead of firing only once. On this emulator, a one-shot guard
  could finish before the `Chrome isn't responding` dialog reappeared.

## After `expo start`

- Leave Metro running in that terminal.
- In a second terminal, set up all three `adb reverse` mappings:
  ```sh
  adb reverse tcp:8081 tcp:8081   # gateway
  adb reverse tcp:8080 tcp:8080   # apps/api
  adb reverse tcp:8082 tcp:8082   # Metro (JS bundle)
  ```
- Install or refresh the debug APK:
  `adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk`
- Clear any stale app process before relaunch:
  `adb shell am force-stop com.dubbridge.mobile`
- Start the app explicitly:
  `adb shell am start -n com.dubbridge.mobile/.MainActivity`
- The success checkpoint for `V2b` is: the app passes the Android splash screen and
  renders React UI on the emulator instead of hanging on splash.

## Troubleshooting

- If the app hangs on splash, confirm Metro is still running on `:8082` and repeat
  all three reverse mappings:
  ```sh
  adb reverse tcp:8081 tcp:8081
  adb reverse tcp:8080 tcp:8080
  adb reverse tcp:8082 tcp:8082
  ```
- If `adb reverse tcp:8081` maps to Metro instead of the gateway (collision), stop
  Metro, start the gateway, then restart Metro on `:8082` using the screenshot env.
- If the app opens but shows `Missing DUBBRIDGE_ENV`, Metro alone was not enough;
  rebuild with the same env vars applied to both:
  `DUBBRIDGE_ENV=local EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://localhost:8081 npx expo prebuild --platform android`
  and
  `cd android && DUBBRIDGE_ENV=local EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://localhost:8081 ./gradlew assembleDebug`
- If install fails due to stale package state, run:
  `adb uninstall com.dubbridge.mobile`
  and then reinstall the APK with `adb install -r`.
- If the emulator window looks blank white or blank black even though the process
  stays alive, verify the mounted screen via accessibility dump before treating it
  as an app-logic failure:
  `adb shell uiautomator dump /sdcard/window_dump.xml && adb pull /sdcard/window_dump.xml /tmp/window_dump.xml`
  then inspect `/tmp/window_dump.xml` for nodes such as `resource-id="login-screen"`
  or the text `DubBridge mobile`.
  On `2026-06-07`, this proved the app had passed splash and mounted
  `LoginScreen` even while the emulator surface rendered as a blank white frame.
- If `authenticated-audit.yaml` fails on `id: home-screen`, verify the runtime
  bootstrap independently before blaming the selector:
  `adb shell am start -a android.intent.action.VIEW -d "dubbridge://auth/callback?handoff_code=<seeded-code>" com.dubbridge.mobile`
  On `2026-06-08`, this manual deep-link probe still left the app on
  `login-screen`, which means the Phase-2 blocker was the runtime bootstrap path,
  not Maestro's `openLink` syntax.
  On `2026-06-11`, all Phase-2 blockers were resolved — see V6b findings below.

## V6b findings (2026-06-11) — Phase-2 blockers resolved

Three independent root causes kept the app on `login-screen` after the deep link:

1. **APK Metro port mismatch.** React Native's default dev server port is `:8081`.
   The screenshot env runs Metro on `:8082` to avoid collision with `apps/gateway`.
   The APK read `react_native_dev_server_port` from the compiled-in resource, which
   defaulted to `8081`. Fix: add `reactNativeDevServerPort=8082` to
   `mobile/android/gradle.properties` and rebuild.

2. **`app.config` asset baked with null values.** `expo-constants` reads
   `assets/app.config` at runtime to populate `Constants.expoConfig?.extra`. That
   file is generated by `expo prebuild`. When prebuild ran without `DUBBRIDGE_ENV`
   set, `process.env.DUBBRIDGE_ENV` serialized as `{}` (not `null`).
   The app showed `config-error-screen` instead of `login-screen`.
   Fix: create `mobile/android/app/src/main/assets/app.config` with the correct
   values before `assembleDebug`. Gradle's `mergeDebugAssets` picks it up.

3. **Gateway not available for `POST /auth/mobile/session`.** The real Rust gateway
   requires PostgreSQL + Redis + first compilation. The handoff code redemption
   call failed silently (10 s timeout → `login_failed` state), leaving the app on
   `login-screen`. Fix: `scripts/e2e-seed/mock-gateway-server.mjs` — a minimal
   Node.js gateway stub that serves `GET /health/ready` and
   `POST /auth/mobile/session`. Pre-seed a code with `POST /e2e/issue-handoff`.

4. **`SEED_BOOTSTRAP_DEEPLINK` not passed to Maestro.** Shell `export` inside a
   compound command (`export X=Y && maestro test`) did not propagate the var into
   the Maestro process (arrived as `undefined`). Fix: use
   `maestro test --env SEED_BOOTSTRAP_DEEPLINK=<value>`.

## Running the full Phase-2 flow

```sh
# 1. Start supporting services
node scripts/e2e-seed/mock-oauth-server.mjs &
node scripts/e2e-seed/mock-gateway-server.mjs &

# 2. Start Metro (if not already running)
. mobile/maestro/screenshot-env.sh
cd mobile && npx expo start --port 8082 --clear &

# 3. adb reverse
adb reverse tcp:8081 tcp:8081
adb reverse tcp:8080 tcp:8080
adb reverse tcp:8082 tcp:8082

# 4. Issue handoff code
DEEPLINK=$(curl -s -X POST http://127.0.0.1:8081/e2e/issue-handoff \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['auth']['bootstrap_deeplink'])")

# 5. Run Phase 2
. mobile/maestro/screenshot-env.sh
maestro test --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK" \
  mobile/maestro/authenticated-audit.yaml
```
