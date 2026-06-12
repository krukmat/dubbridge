# Maestro screenshot suite — S-055 + S-060

Reproducible five-phase screenshot capture for every DubBridge mobile screen on an
Android emulator. S-055 (V1–V8) delivered phases 1–2; S-060 T6 integrated phases 3–5
into the same one-command runner. Complete as of 2026-06-12.

## Overview

| Phase | Flow file | BDD scenarios | Screenshot |
|---|---|---|---|
| 1 — Auth surface | `auth-surface.yaml` | — | `01_auth_login.png` |
| 2 — Authenticated audit | `authenticated-audit.yaml` | — | `02_home.png` |
| 3 — Asset list (populated) | `asset-list.yaml` | `SC-LIST-1` | `03_asset_list.png` |
| 3b — Asset list (empty) | `asset-list.yaml` | `SC-LIST-2` | `03_asset_list.png` |
| 4 — Asset detail | `asset-detail.yaml` | `SC-DETAIL-1` | `04_asset_detail.png` |
| 5 — Asset ingestion | `asset-ingestion.yaml` | `SC-INGEST-1` | `05_upload.png` |

The list flow supports two mock-gateway seed modes:

- default handoff (`POST /e2e/issue-handoff`) returns the two populated seed assets and captures the standard `03_asset_list` screenshot (phase 3 / SC-LIST-1).
- empty handoff (`POST /e2e/issue-handoff?asset_seed=empty`) returns an empty list for that redeemed session and asserts `SC-LIST-2` (phase 3b).

Phase 2 bootstraps a gateway session without UI login by redeeming a seeded one-time
`handoff_code` into an opaque `session_ref` (ADR-024). No JWT or refresh token ever
reaches the device or any Maestro artifact.

## Prerequisites

Install each tool before running the suite:

| Tool | Install | Required for |
|---|---|---|
| `adb` | Android SDK platform-tools | emulator control |
| Android emulator | Android Studio / `sdkmanager` | running the app |
| `node` >= 18 | <https://nodejs.org> | seed scripts |
| `maestro` | `brew install mobile-dev-inc/tap/maestro` | flow execution |
| `curl` | pre-installed on macOS | health checks |

The debug APK must exist at:

```
mobile/android/app/build/outputs/apk/debug/app-debug.apk
```

If it does not exist, build it first:

```sh
cd mobile
npx expo prebuild --platform android
cd android && ./gradlew assembleDebug
```

See [V6b findings](#v6b-findings--phase-2-blockers-resolved) if the APK was built
without the correct env vars set.

## Port map

| Service | Host port | `adb reverse` mapping | Notes |
|---|---|---|---|
| `apps/gateway` / `mock-gateway` | `:8081` | `adb reverse tcp:8081 tcp:8081` | gateway and OAuth callback |
| `apps/api` | `:8080` | `adb reverse tcp:8080 tcp:8080` | REST data for authed screens |
| Metro (JS bundler) | `:8082` | `adb reverse tcp:8082 tcp:8082` | deconflicted from gateway |
| `mock-oauth` | `:9000` | none (host-only) | gateway contacts it directly |

Metro **must** run on `:8082` — never `:8081` — to avoid collision with the gateway.

## Running the suite

### One-command run (recommended)

```sh
# From repo root — start supporting services first
node scripts/e2e-seed/mock-oauth-server.mjs &
node scripts/e2e-seed/mock-gateway-server.mjs &

# Then run the full suite (all 5 phases)
cd mobile && npm run screenshots
```

`npm run screenshots` calls `bash maestro/seed-and-run.sh`, which:

1. Checks dependencies (`adb`, `node`, `curl`, `maestro`).
2. Detects the running Android emulator and installs the debug APK.
3. Waits for gateway `:8081/health/ready` and api `:8080` to be ready.
4. Starts Metro on `:8082` if not already running.
5. Sets all three `adb reverse` mappings.
6. Runs **Phase 1** (`auth-surface.yaml`) — captures `01_auth_login.png`.
7. Mints a handoff code; runs **Phase 2** (`authenticated-audit.yaml`) — captures `02_home.png`.
8. Mints a handoff code; runs **Phase 3** (`asset-list.yaml` / SC-LIST-1) — captures `03_asset_list.png`.
9. Mints an empty handoff code; runs **Phase 3b** (`asset-list.yaml` / SC-LIST-2) — captures `03_asset_list.png` (empty state).
10. Mints a handoff code; runs **Phase 4** (`asset-detail.yaml` / SC-DETAIL-1) — captures `04_asset_detail.png`.
11. Mints a handoff code; runs **Phase 5** (`asset-ingestion.yaml` / SC-INGEST-1) — captures `05_upload.png`.
12. Copies all PNGs to `mobile/artifacts/screenshots/`.
13. Sanitizes `handoff_code` and `session_ref` from all Maestro JSON reports.
14. Asserts no sensitive values remain in reports.

Set `START_MOCK_SERVERS=1` to have the script start mock-oauth and mock-gateway
automatically:

```sh
START_MOCK_SERVERS=1 npm run screenshots
```

### Manual step-by-step

If you need to run phases independently:

```sh
# 1. Source the screenshot env
. mobile/maestro/screenshot-env.sh

# 2. Start supporting services
node scripts/e2e-seed/mock-oauth-server.mjs &
node scripts/e2e-seed/mock-gateway-server.mjs &

# 3. Start Metro on :8082
cd mobile && npx expo start --port 8082 --clear &

# 4. Set adb reverse mappings
adb reverse tcp:8081 tcp:8081
adb reverse tcp:8080 tcp:8080
adb reverse tcp:8082 tcp:8082

# 5. Install APK
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk

# 6. Phase 1 — auth surface
maestro test mobile/maestro/auth-surface.yaml \
  --test-output-dir /tmp/dubbridge-maestro-auth

# 7. Mint a handoff code
DEEPLINK=$(curl -sf -X POST http://127.0.0.1:8081/e2e/issue-handoff \
  | node -e "const d=require('fs').readFileSync('/dev/stdin','utf8'); \
             console.log(JSON.parse(d).auth.bootstrap_deeplink)")

# 8. Phase 2 — authenticated home
maestro test mobile/maestro/authenticated-audit.yaml \
  --test-output-dir /tmp/dubbridge-maestro-authed \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK"

# 9. Mint a new code; Phase 3 — asset list (SC-LIST-1)
DEEPLINK=$(curl -sf -X POST http://127.0.0.1:8081/e2e/issue-handoff \
  | node -e "const d=require('fs').readFileSync('/dev/stdin','utf8'); \
             console.log(JSON.parse(d).auth.bootstrap_deeplink)")
maestro test mobile/maestro/asset-list.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-list \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK"

# 10. Phase 3b — asset list empty (SC-LIST-2)
EMPTY_DEEPLINK=$(curl -sf -X POST \
  "http://127.0.0.1:8081/e2e/issue-handoff?asset_seed=empty" \
  | node -e "const d=require('fs').readFileSync('/dev/stdin','utf8'); \
             console.log(JSON.parse(d).auth.bootstrap_deeplink)")
maestro test mobile/maestro/asset-list.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-list-empty \
  --env SEED_BOOTSTRAP_DEEPLINK="$EMPTY_DEEPLINK"

# 11. Phase 4 — asset detail (SC-DETAIL-1)
DEEPLINK=$(curl -sf -X POST http://127.0.0.1:8081/e2e/issue-handoff \
  | node -e "const d=require('fs').readFileSync('/dev/stdin','utf8'); \
             console.log(JSON.parse(d).auth.bootstrap_deeplink)")
maestro test mobile/maestro/asset-detail.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-detail \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK"

# 12. Phase 5 — ingestion (SC-INGEST-1); requires EXPO_PUBLIC_E2E_ENABLED=true in APK
DEEPLINK=$(curl -sf -X POST http://127.0.0.1:8081/e2e/issue-handoff \
  | node -e "const d=require('fs').readFileSync('/dev/stdin','utf8'); \
             console.log(JSON.parse(d).auth.bootstrap_deeplink)")
maestro test mobile/maestro/asset-ingestion.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-ingestion \
  --env SEED_BOOTSTRAP_DEEPLINK="$DEEPLINK"
```

Note: always pass `SEED_BOOTSTRAP_DEEPLINK` via `--env` to Maestro, not via shell
`export`. A shell `export` inside a compound command does not propagate into the
Maestro process (V6b finding).

`asset-ingestion.yaml` requires `EXPO_PUBLIC_E2E_ENABLED=true` baked into the APK:
the upload screen then skips manual rights entry, starts at the `Pick file` step with
seeded rights metadata, and injects the deterministic `dubbridge-e2e-upload.mov` asset
for screenshot capture.

## Screen testID convention

Every screen captured by Maestro exposes a stable `testID` on its root view:

| Screen | `testID` |
|---|---|
| `LoginScreen` | `login-screen` |
| `HomeScreen` | `home-screen` |
| `AssetListScreen` | `asset-list-screen` |
| `AssetDetailScreen` | `asset-detail-screen` |
| `UploadScreen` | `upload-screen` |
| `ConfigErrorScreen` | `config-error-screen` |

New screens added to the suite must follow the pattern `<feature>-screen`.

## Troubleshooting

### App hangs on splash screen

Metro is not reachable from the emulator. Confirm Metro is running on `:8082` (not
`:8081`) and repeat all three reverse mappings:

```sh
adb reverse tcp:8081 tcp:8081
adb reverse tcp:8080 tcp:8080
adb reverse tcp:8082 tcp:8082
```

If the APK was built before the port was changed to `:8082`, rebuild it with
`gradle.properties` containing `reactNativeDevServerPort=8082`:

```sh
cd mobile
npx expo prebuild --platform android
cd android && ./gradlew assembleDebug
```

### App shows "Missing DUBBRIDGE_ENV" or config error screen

The APK baked null env values at prebuild time. Rebuild with the env vars set:

```sh
DUBBRIDGE_ENV=local \
EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://localhost:8081 \
EXPO_PUBLIC_E2E_ENABLED=true \
npx expo prebuild --platform android

cd android && \
DUBBRIDGE_ENV=local \
EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://localhost:8081 \
./gradlew assembleDebug
```

### ANR dialog — "Chrome isn't responding"

The Maestro flows include an ANR guard that polls over 20 iterations of
`waitForAnimationToEnd`. If the dialog reappears after the guard, increase the
`repeat.times` value in the affected flow file or dismiss manually and re-run.

### Phase 2 fails on `id: home-screen` (app stays on login screen)

Verify the bootstrap independently before blaming the Maestro selector:

```sh
adb shell am start -a android.intent.action.VIEW \
  -d "dubbridge://auth/callback?handoff_code=<seeded-code>" \
  com.dubbridge.mobile
```

If this manual probe also leaves the app on `login-screen`, the blocker is the runtime
bootstrap path, not the Maestro YAML. Check:

1. `mock-gateway-server` is running and `POST /auth/mobile/session` returns `{ session_ref }`.
2. `EXPO_PUBLIC_E2E_ENABLED=true` was set when the APK was built.
3. The handoff code has not expired (90 s TTL) or been redeemed already.

### APK install fails

```sh
adb uninstall com.dubbridge.mobile
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk
```

## V6b findings — Phase-2 blockers resolved

Three independent root causes kept the app on `login-screen` after the deep link
(resolved 2026-06-11):

1. **APK Metro port mismatch.** The React Native default dev server port is `:8081`.
   Fix: add `reactNativeDevServerPort=8082` to `mobile/android/gradle.properties` and
   rebuild the APK.

2. **`app.config` asset baked with null values.** When `expo prebuild` ran without
   `DUBBRIDGE_ENV` set, `process.env.DUBBRIDGE_ENV` serialized as `{}`. Fix: set env
   vars before `expo prebuild` and `assembleDebug`.

3. **Gateway not available for `POST /auth/mobile/session`.** The real Rust gateway
   requires PostgreSQL + Redis. Fix: use `scripts/e2e-seed/mock-gateway-server.mjs`,
   which serves `GET /health/ready` and `POST /auth/mobile/session` in-process.

4. **`SEED_BOOTSTRAP_DEEPLINK` not passed to Maestro.** Shell `export` inside a
   compound command did not propagate into the Maestro process. Fix: pass via
   `maestro test --env SEED_BOOTSTRAP_DEEPLINK=<value>`.
