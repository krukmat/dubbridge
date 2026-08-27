---
type: Audit
title: "MVP0-P2P P0 native compatibility preflight"
date: 2026-08-27
task: P0
---

# MVP0-P2P P0 — Native compatibility preflight

**Date:** 2026-08-27
**Verdict:** **TECHNICAL PASS (Android only)** — the generated Android project
assembled, installed, and executed the bounded Bare lifecycle on an Android 34
ARM64 emulator. The repository owner accepted the Android-only P0 scope on
2026-08-27. No iPhone/iOS support was generated, configured, or tested.

## Candidate integration evidence

- npm registry: `react-native-bare-kit@0.15.0` declares React and React Native
  peer dependencies and documents `Worklet` plus IPC as the bridge API.
- npm registry: `bare-rpc@1.3.8` declares `bare-buffer` as a peer dependency.
- Holepunch's current Expo guide uses `react-native-bare-kit`, `bare-rpc`,
  `b4a`, and `bare-pack`; it documents Gradle 8.10.2, Java 23, Android SDK
  29+, and Android NDK as mobile prerequisites.
- The upstream Bare Kit error-handling guidance says an uncaught exception or
  unhandled rejection inside a worklet aborts the host process. Any later P0
  implementation must install an in-worklet error handler and report failure
  through IPC to satisfy EC-1; a host-only `try/catch` is insufficient.

Sources: [react-native-bare-kit](https://github.com/holepunchto/react-native-bare-kit),
[Bare mobile Expo guide](https://github.com/holepunchto/pear-docs/blob/main/guide/making-a-bare-mobile-app.md),
and npm registry metadata queried on this host.

## Repository baseline

| Check | Result |
|---|---|
| Expo CLI | `56.1.14` |
| App SDK / RN | Expo `~56.0.9` / React Native `0.85.3` |
| JavaScript toolchain | Node `20.20.2`, npm `10.8.2` |
| Selected Bare dependencies | `react-native-bare-kit` 0.15.0, `react-native-b4a` 0.1.0, `b4a` 1.8.1; P0 intentionally does not add `bare-rpc`/`bare-pack` |
| Expo config plugins | existing plugins plus `expo-build-properties` 56.0.26 for Android min SDK 31 and legacy packaging |
| JavaScript verification | `npm run typecheck`, `npm run lint`, and full `npm test` passed (22 suites, 240 tests) |

## Current Android preflight

| Required proof surface | Observed state | Consequence |
|---|---|---|
| Android SDK/NDK | SDK platforms 34/36, build-tools 34/35/36, CMake 3.22.1, and NDK 27.1.12297006 are installed beneath `/opt/homebrew/share/android-commandlinetools` | configure task-local `ANDROID_SDK_ROOT` and use the generated project's wrapper rather than a global Gradle |
| Android emulator | Android 34 Google APIs ARM64 system image and emulator 36.6.11 are installed | an emulator/Android development-build proof is available once the native project exists |
| CocoaPods / Xcode | CocoaPods 1.17.0 and Xcode 26.6 are installed | recorded only as host state; iPhone/iOS is explicitly out of P0 scope |
| Build Java | Homebrew JDK 17.0.19 selected task-locally; host default Java 26 was not used | Gradle 9.3.1 Android assembly passed, so Java 23 was not installed merely to match the older upstream guide |

All Android SDK licenses were accepted. The native project was generated with
`expo prebuild --platform android --no-install`; no `ios/` directory was
generated. CocoaPods and Xcode remain host facts only and were not invoked.

## Android native proof

With `ANDROID_SDK_ROOT` and `ANDROID_HOME` set to
`/opt/homebrew/share/android-commandlinetools` and JDK 17 selected locally:

1. `cd mobile && npx expo prebuild --platform android --no-install` — **PASS**.
2. `cd mobile/android && ./gradlew :app:assembleDebug --console=plain` —
   **PASS** (`BUILD SUCCESSFUL`; Bare Kit Android codegen/link/CMake tasks ran).
3. `cd mobile && EXPO_PUBLIC_BARE_RUNTIME_PROBE=true npx expo run:android --no-bundler`
   — **PASS** (development APK built and installed).
4. The existing `fenix_t7` Android 34 Google APIs ARM64 emulator ran the build.
   Logcat recorded:

   ```text
   [Bare runtime probe] ping=pong
   [Bare runtime probe] shutdown=complete
   ```

The generated configuration enables the proof only for the explicit environment
flag. The normal application continues to render its existing navigator; P0 has
no visible UI and no network, P2P, identity, key, media, API, or database path.

The Gradle build emitted a pre-existing forward-looking deprecation notice for
Gradle 10 compatibility. It did not fail this Gradle 9.3.1 assembly and creates
no P0 product or security risk; address it separately from this bounded spike.

## Closure and downstream gate

The repository owner accepted the Android development-build evidence and the
Android-only scope on 2026-08-27. P0 is closed. P1 still requires its separately
approved task card and dependency check before implementation.
