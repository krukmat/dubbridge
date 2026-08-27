---
type: TaskList
title: "Tasks: MVP-0 — P2P-first invited playback"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-first.md
---

# Tasks: MVP-0 — P2P-first invited playback

> **Plan:** `docs/plan/mvp0-p2p-first.md`.
> **External taskpack:** `p2p-mvp/taskpacks/P0.zip` through `P7.zip`.
> **Current task:** P0 is closed. P1 was materially replanned for a maintainable
> mobile/runtime boundary; revised P1 and ADR-043 were approved on 2026-08-27.
> P1.F1 was separately approved and implemented with automated acceptance PASS
> on 2026-08-27; owner final verification is pending and no later child is
> authorized. The external package
> is untracked input, so its
> state/handoff files are evidence but not substitutes for this ledger.

## Task map

| ID | Title | Status | Depends on |
|---|---|---|---|
| P0 | Bare / Expo / React Native compatibility spike | PASS — Android-only; owner verified 2026-08-27 | — |
| P1 | Maintainable mobile P2P foundation + replication proof | Parent approved; P1.F1 implemented/automated PASS pending owner verification; later children deferred | P0 PASS |
| P2 | Encrypted P2P publication after S-120 | Pending | P1 PASS; approved ADR |
| P3 | Invite, claim, and content-key envelope | Pending | P2 PASS; approved key contract |
| P4 | Mobile package sync and verification | Pending | P3 PASS |
| P5 | Local HLS gateway + existing VideoPlayer | Pending | P4 PASS |
| P6 | Minimal My Content + Invites dashboard | Pending | P3–P5 PASS |
| P7 | End-to-end P2P certification | Pending | P2–P6 PASS |

> **Review exception:** the repository owner waived phase-1 and phase-2 peer
> review for P0–P7 only. Each task still requires its RRI-derived approval,
> scope/dependency gate, verification, and all non-review closure evidence.
> See `docs/audit/mvp0-p2p-review-exception.md`.

## P0 — Bare / Expo / React Native compatibility spike

- **Status:** PASS — Android-only proof accepted by the repository owner on
  2026-08-27.
- **Type:** Development feasibility spike.
- **Effort / RRI:** L / 54 → Med-high; full report:
  `docs/audit/mvp0-p2p-p0-rri.md`.
- **Allowed paths:** `mobile/package.json`, `mobile/package-lock.json`,
  `mobile/app.config.ts`, `mobile/App.tsx`, required generated Android
  configuration only, `mobile/src/p2p/**`, `mobile/__tests__/p2p/**`, and P0
  evidence documents. `mobile/App.tsx` may host only the environment-gated,
  Android-only proof bootstrap; it must add no UI or product behavior.
- **Objective:** Validate that a compatible Bare worklet can start in the current
  Expo/RN app and complete a bounded RPC ping/pong, while preserving the app's
  normal build/runtime.
- **Out of scope:** Hyperdrive/HyperSwarm, DHT/networking, media replication,
  HTTP gateway, backend/API/database changes, device identity, encryption,
  invitations, or product UI.

### Happy paths considered

- **HP-1:** A development build loads the Bare bridge, starts the worklet, and an
  `initialize → ping` request returns the expected `pong` response.
- **HP-2:** After the proof, `shutdown` releases the bridge cleanly and the
  existing Expo app still typechecks/builds under the selected native workflow.

### Edge cases considered

- **EC-1:** A worklet/RPC initialization failure is surfaced as a typed,
  diagnosable P0 failure; it does not crash the JavaScript app or silently mark
  the spike as passed.
- **EC-2:** An RPC timeout, invalid reply, or shutdown-before-ready is rejected
  deterministically without retaining a stale worklet handle.

### Acceptance criteria

- The selected package versions have documented compatibility evidence for Expo
  SDK 56 / React Native 0.85 and the required Android native toolchain.
- P0 provides a repeatable Android native-development proof of a worklet and
  bounded ping/pong RPC, with PASS or STOP recorded. iPhone/iOS is expressly
  deferred and is not implied by a P0 PASS.
- Existing mobile typecheck/tests remain green; the Android bridge lifecycle/error
  logic has unit-test evidence for HP-1/HP-2/EC-1/EC-2.
- No P2P media/product behavior or secret/key material is introduced.

### Evidence to emit

- Full `scripts/rri.py` report and phase-1 review artifact.
- Exact package/version selection and native prerequisite evidence.
- Native proof command/output, mobile typecheck/test output, and P0 handoff.

### Status artifacts affected

- This ledger; `docs/plan/mvp0-p2p-first.md`; `p2p-mvp/RUN_STATE.json`; the
  required P0 handoff; and `docs/plan/roadmap.md` at P0 closure.

### Task-analysis review

Task-analysis review: REVIEW-OVERRIDE — owner-directed MVP0-P2P exception;
`docs/audit/mvp0-p2p-review-exception.md`.

### Code-solution review

Code-solution review: REVIEW-OVERRIDE — owner-directed MVP0-P2P exception;
the implemented P0 diff is covered by the bounded exception in
`docs/audit/mvp0-p2p-review-exception.md`.

### Implementation evidence

- Added the Android-native integration dependencies `react-native-bare-kit`
  `0.15.0`, `react-native-b4a` `0.1.0`, `b4a` `1.8.1`, and
  `expo-build-properties` `56.0.26` in `mobile/package.json` and its lockfile.
- Configured only Android (`minSdkVersion: 31`, legacy packaging) and the
  environment gate `EXPO_PUBLIC_BARE_RUNTIME_PROBE=true`. No iOS project was
  generated or modified.
- Added a dependency-free worklet source and typed host bridge under
  `mobile/src/p2p/`. `BareBridge` contains only `initialize`, `ping`, and
  `shutdown`, typed protocol/error handling, timeout cleanup, and termination.
- Added an invisible Android-only `AndroidBareRuntimeProbe` in `mobile/App.tsx`.
  It runs only when the explicit development-probe flag is true; it adds no product UI,
  network connection, identity, key, media, or P2P behavior.

### Reflection log

| Pass | Question and result | Evidence |
|---|---|---|
| 1 — lifecycle | Does the state machine prove `idle → starting → ready → stopped` and clean termination? **PASS.** | `bare-bridge.test.ts` covers initialization, ping, shutdown, and release; the Android runtime logged pong then shutdown. |
| 2 — containment | Do worklet errors, malformed/late replies, and shutdown-before-ready fail without stale state? **PASS.** | Typed `WORKLET_FAILURE`, `MALFORMED_REPLY`, and `SHUTDOWN_BEFORE_READY` paths are unit tested; pending callbacks are rejected and listeners removed on release. |
| 3 — native scope | Does the exact Android build prove compatibility while retaining the P0 boundary? **PASS.** | Generated Android project assembled, installed, and ran on Android 34 ARM64. The proof has no iOS output and no P2P/product capabilities. |

### Unit coverage certification

| Requirement | Certified evidence |
|---|---|
| HP-1 | `BareBridge lifecycle` initializes and receives `pong`; Android log records `[Bare runtime probe] ping=pong`. |
| HP-2 | Same lifecycle test verifies shutdown/termination; Android log records `[Bare runtime probe] shutdown=complete`. |
| EC-1 | `surfaces typed worklet failures` asserts `WORKLET_FAILURE` without a host crash. |
| EC-2 | `rejects malformed replies and releases the worklet`, `releases when shutdown precedes ready`, and `ignores late replies after shutdown` cover deterministic cleanup. |

`cd mobile && npm test` passed: 22 suites, 240 tests. The Bare bridge suite has
5 passing tests. This is certification for the P0 bridge scope, not a claim of
coverage for later P2P work.

### Verification summary

- `cd mobile && npm run typecheck` — PASS.
- `cd mobile && npm run lint` — PASS.
- `cd mobile && npm test` — PASS (22 suites, 240 tests).
- `cd mobile && npx expo prebuild --platform android --no-install` — PASS;
  generated Android only.
- `cd mobile/android && ./gradlew :app:assembleDebug --console=plain` with
  task-local Android SDK and JDK 17 — PASS.
- `cd mobile && npm run android:bare-probe -- --no-bundler` — PASS with the
  task-local Android SDK/JDK environment; Android 34 ARM64 logcat recorded
  `[Bare runtime probe] ping=pong` then `shutdown=complete`.
- Full environment/version evidence and exact toolchain conclusion:
  `docs/audit/mvp0-p2p-p0-native-preflight.md`.

### Owner final verification

- **Owner:** Matias (repository owner)
- **Date:** 2026-08-27
- **Statement:** The repository owner approved closure of the Android-only P0
  scope after the recorded native proof and unit-test evidence.
- **Commands run:** `cd mobile && npm run typecheck`; `cd mobile && npm run lint`;
  `cd mobile && npm test -- --runInBand`; `cd mobile && npx expo prebuild --platform android --no-install`;
  `cd mobile/android && ./gradlew :app:assembleDebug --console=plain`; and
  `cd mobile && npm run android:bare-probe -- --no-bundler` with the task-local
  Android SDK and JDK 17 environment.

## Deferred task acceptance summaries

The following tasks stay unpresented until their dependency evidence exists. Their
external taskpacks are useful input, but the detailed ledger entries, RRI reports,
and approval cards must be created at activation time.

- **P1:** the earlier approved topology was superseded before source execution.
  Revised P1 first establishes composition-root/provider/service/runtime
  ownership, reproducible versioned RPC, lifecycle handling, and transient
  storage cleanup; it migrates P0 characterization before deleting its temporary
  probe/bridge scaffold and separately audits P0 config/dependencies; only then
  does its isolated proof runner perform replication.
  Detailed plan/task ledger and revised approval card:
  `docs/plan/mvp0-p2p-p1-replication.md`,
  `docs/tasks/mvp0-p2p-p1-replication.md`, and
  `docs/audit/mvp0-p2p-p1-approval-card.md`; accepted architecture:
  `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md`.
- **P2:** HP: prepared HLS produces a ciphertext-only publication and durable
  publication state; EC: publication failure never exposes an asset as P2P-ready.
- **P3:** HP: an eligible invite is claimed idempotently by one viewer; EC: raw
  tokens are never persisted and expired/other-viewer claims fail closed.
- **P4:** HP: a claimed descriptor syncs and verifies before READY; EC: corrupted
  ciphertext or manifest mismatch cannot become playable.
- **P5:** HP: a verified local package plays through loopback HLS and the existing
  player; EC: gateway shutdown/key failure stops delivery without HTTP fallback.
- **P6:** HP: owner and viewer see only their minimal required state/actions; EC:
  unavailable/expired/not-ready content cannot offer Play.
- **P7:** HP: owner-to-viewer flow completes with control-plane APIs but no legacy
  HTTP media route; EC: any HTTP media fallback makes certification fail.
