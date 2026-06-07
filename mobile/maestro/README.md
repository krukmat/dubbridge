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
