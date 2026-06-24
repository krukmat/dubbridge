# Maestro screenshot suite — S-055 through S-200

Reproducible eight-phase screenshot capture for DubBridge authenticated mobile
surfaces on an Android emulator. Uses S-200 credential auth (email/password +
HS256 JWT bearer token) via the mock gateway.

## Overview

| Phase | Flow file | BDD scenarios | Screenshots |
|---|---|---|---|
| 1 — Auth surface | `auth-surface.yaml` | SC-AUTH-1 | `01_auth_login.png` |
| 2 — Authenticated home | `authenticated-audit.yaml` | — | `02_home.png` |
| 3 — Asset list (populated) | `asset-list.yaml` | `SC-LIST-1` | `03_asset_list.png` |
| 3b — Asset list (empty) | `asset-list.yaml` | `SC-LIST-2` | `03_asset_list.png` |
| 4 — Asset detail | `asset-detail.yaml` | `SC-DETAIL-1` | `04_asset_detail.png` |
| 5 — Asset ingestion (SC-INGEST-1) | `asset-ingestion.yaml` | `SC-INGEST-1` | `05_upload.png`, `06_ingest_complete.png` |
| 5b — Asset ingestion no-rights (SC-INGEST-2) | `asset-ingestion-no-rights.yaml` | `SC-INGEST-2` | `07_ingest_no_rights.png` |
| 6 — Organizations and projects | `projects.yaml` | `SC-ORG-1`, `SC-PROJECT-1` | `08_home_for_projects.png`, `09_project_list.png`, `10_project_detail.png` |
| 7 — Compliance and consent | `compliance.yaml` | `SC-AUDIT-1`, `SC-RIGHTS-1`, `SC-CONSENT-1/2` | `11_compliance_center.png`, `12_consent_active.png`, `13_consent_revoked.png` |
| 8 — Review and publication | `review.yaml` | `SC-REVIEW-1/2`, `SC-PUBLISH-1/2` | `14_review_inbox.png`, `15_review_detail.png`, `16_review_approved.png`, `17_review_published.png` |
| 8b — Playback surfaces | `playback.yaml` | `SC-PLAYBACK-1`, `SC-PLAYBACK-3` | `playback_review.png`, `playback_asset_detail.png` |

All phases from 2 onwards authenticate via the reusable `e2e-login.yaml` subflow
using the fixed E2E credentials (`e2e@dubbridge.dev` / `e2etestpass123`). The mock
gateway `/auth/login` endpoint returns a fixed bearer token; all API calls use
`Authorization: Bearer <token>`.

Data seeding for per-phase modes (asset_seed, ingest_seed) is done via
`POST /e2e/seed?asset_seed=...&ingest_seed=...` before the relevant phase.

## Prerequisites

Install each tool before running the suite:

| Tool | Install | Required for |
|---|---|---|
| `adb` | Android SDK platform-tools | emulator control |
| Android emulator | Android Studio / `sdkmanager` | running the app |
| `node` >= 18 | <https://nodejs.org> | mock gateway |
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

If the Hermes bundle in the APK is stale (app shows blank screen or "runtime not ready"),
patch it by exporting a fresh bundle and replacing it inside the APK:

```sh
# Export fresh HBC bundle
cd mobile && npx expo export --platform android
cp android/app/build/outputs/apk/debug/app-debug.apk /tmp/app-patched.apk
zip -u /tmp/app-patched.apk assets/index.android.bundle
apksigner sign --ks android/app/debug.keystore \
  --ks-pass pass:android --key-pass pass:android \
  /tmp/app-patched.apk
adb install -r /tmp/app-patched.apk
```

## Port map

| Service | Host port | `adb reverse` mapping | Notes |
|---|---|---|---|
| `mock-gateway-server` | `:8081` | `adb reverse tcp:8081 tcp:8081` | auth + API mock |
| `apps/api` | `:8080` | `adb reverse tcp:8080 tcp:8080` | REST data |
| Metro (JS bundler) | `:8082` | `adb reverse tcp:8082 tcp:8082` | deconflicted from gateway |

Metro **must** run on `:8082` — never `:8081` — to avoid collision with the gateway.

## Running the suite

### One-command run (recommended)

```sh
# From repo root — start supporting services first
node scripts/e2e-seed/mock-gateway-server.mjs &

# Then run the full suite (all 8 phases)
cd mobile && npm run screenshots
```

`npm run screenshots` calls `bash maestro/seed-and-run.sh`, which:

1. Checks dependencies (`adb`, `node`, `curl`, `maestro`).
2. Detects the running Android emulator and installs the debug APK.
3. Waits for gateway `:8081/health/ready` and api `:8080` to be ready.
4. Starts Metro on `:8082` if not already running.
5. Sets all three `adb reverse` mappings.
6. Runs **Phase 1** (`auth-surface.yaml`) — captures `01_auth_login.png`.
7. Smoke-checks `/auth/login`; runs **Phase 2** (`authenticated-audit.yaml`) — captures `02_home.png`.
8. Seeds default asset mode; runs **Phase 3** (`asset-list.yaml` / SC-LIST-1) — captures `03_asset_list.png`.
9. Seeds `asset_seed=empty`; runs **Phase 3b** — captures `03_asset_list.png` (empty state).
10. Runs **Phase 4** (`asset-detail.yaml` / SC-DETAIL-1) — captures `04_asset_detail.png`.
11. Runs **Phase 5** (`asset-ingestion.yaml` / SC-INGEST-1) — captures `05_upload.png` + `06_ingest_complete.png`.
12. Seeds `ingest_seed=no_rights`; runs **Phase 5b** — captures `07_ingest_no_rights.png`.
13. Runs **Phase 6** (`projects.yaml`) — captures organization/project screens.
14. Runs **Phase 7** (`compliance.yaml`) — captures compliance and consent states.
15. Runs **Phase 8** (`review.yaml`) — captures review and publication states.
16. Runs **Phase 8b** (`playback.yaml`) — captures the review-player and asset-detail playback surfaces.
17. Copies all PNGs to `mobile/artifacts/screenshots/`.

Set `START_MOCK_SERVERS=1` to have the script start mock-gateway automatically:

```sh
START_MOCK_SERVERS=1 npm run screenshots
```

### Manual step-by-step

If you need to run phases independently:

```sh
# 1. Start mock gateway
node scripts/e2e-seed/mock-gateway-server.mjs &

# 2. Start Metro on :8082
cd mobile && npx expo start --port 8082 --clear &

# 3. Set adb reverse mappings
adb reverse tcp:8081 tcp:8081
adb reverse tcp:8080 tcp:8080
adb reverse tcp:8082 tcp:8082

# 4. Install APK
adb install -r mobile/android/app/build/outputs/apk/debug/app-debug.apk

# 5. Phase 1 — auth surface
maestro test mobile/maestro/auth-surface.yaml \
  --test-output-dir /tmp/dubbridge-maestro-auth

# 6. Phase 2 — authenticated home (login via e2e-login.yaml subflow)
maestro test mobile/maestro/authenticated-audit.yaml \
  --test-output-dir /tmp/dubbridge-maestro-authed

# 7. Phase 3 — asset list populated (SC-LIST-1)
curl -sf -X POST "http://127.0.0.1:8081/e2e/seed?asset_seed=default"
maestro test mobile/maestro/asset-list.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-list

# 8. Phase 3b — asset list empty (SC-LIST-2)
curl -sf -X POST "http://127.0.0.1:8081/e2e/seed?asset_seed=empty"
maestro test mobile/maestro/asset-list.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-list-empty

# 9. Phase 4 — asset detail (SC-DETAIL-1)
maestro test mobile/maestro/asset-detail.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-detail

# 10. Phase 5 — ingestion success (SC-INGEST-1)
maestro test mobile/maestro/asset-ingestion.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-ingestion

# 11. Phase 5b — ingestion no-rights (SC-INGEST-2)
curl -sf -X POST "http://127.0.0.1:8081/e2e/seed?ingest_seed=no_rights"
maestro test mobile/maestro/asset-ingestion-no-rights.yaml \
  --test-output-dir /tmp/dubbridge-maestro-asset-ingestion-no-rights

# 12. Phase 8b — playback surfaces (SC-PLAYBACK-1 / SC-PLAYBACK-3)
maestro test mobile/maestro/playback.yaml \
  --test-output-dir /tmp/dubbridge-maestro-playback
```

## E2E credentials

The mock gateway uses fixed constants for E2E test auth — these are not production secrets:

| Constant | Value |
|---|---|
| E2E email | `e2e@dubbridge.dev` |
| E2E password | `e2etestpass123` |
| E2E bearer token | `e2e-bearer-token` |

All Maestro flows authenticate by running the `e2e-login.yaml` subflow, which types the
credentials into the app's login screen and waits for `home-screen` to appear. No deeplinks
or session refs are used.

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
`gradle.properties` containing `reactNativeDevServerPort=8082`.

### App shows blank screen or "runtime not ready" error

The Hermes bundle in the APK is stale. Patch it using the `expo export` + `zip -u` +
`apksigner` flow described in Prerequisites. The S-200 JS bundle must be present for
the auth flow to work.

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

### Phase login fails (app stays on login-screen after e2e-login.yaml)

Verify the login endpoint is responding:

```sh
curl -sf -X POST http://127.0.0.1:8081/auth/login \
  -H "content-type: application/json" \
  -d '{"email":"e2e@dubbridge.dev","password":"e2etestpass123"}'
# Expected: {"token":"e2e-bearer-token","userId":"e2e-user","workspaceId":"ws-seed-1"}
```

If the curl succeeds but the app stays on login-screen, check that the APK bundle
includes the S-200 auth flow (see "App shows blank screen" above).

### ANR dialog — "Chrome isn't responding"

The Maestro flows include an ANR guard that polls over 3 repeat iterations. If the
dialog reappears after the guard, increase the `repeat.times` value in the affected
flow file or dismiss manually and re-run.

## V6b findings — APK bundle patching (2026-06-11)

Three independent root causes resolved by 2026-06-11:

1. **APK Metro port mismatch.** Default RN dev server port is `:8081`. Fix: add
   `reactNativeDevServerPort=8082` to `mobile/android/gradle.properties` and rebuild.

2. **`app.config` asset baked with null values.** When `expo prebuild` ran without
   `DUBBRIDGE_ENV` set, env vars serialized as `{}`. Fix: set env vars before prebuild.

3. **Stale Hermes bundle (post-S-200).** `gradle clean assembleDebug` broke CMake native
   artifacts; stale bundle from pre-S-200 caused AppRegistry crash. Fix: `expo export`
   bundle patch flow described above.
