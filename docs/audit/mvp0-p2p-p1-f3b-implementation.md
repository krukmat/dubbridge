---
type: Audit
title: "MVP0-P2P P1.F3b implementation evidence"
task: P1.F3b
date: 2026-08-27
status: blocked_on_android_device_proof
---

# MVP0-P2P P1.F3b — implementation evidence

**Task:** `docs/tasks/mvp0-p2p-p1-replication.md` § P1.F3b — P0
config/dependency cleanup.
**RRI:** 24 Low (`docs/audit/mvp0-p2p-p1-f3b-rri.md`). No approval card
required by band; the owner additionally directed direct execution
("realizala tu sin presentar").
**Dependency:** P1.F3a.2 PASS — satisfied 2026-08-27.

**Outcome:** the rename half of the task is complete and fully verified. The
dependency/build audit is complete and concluded *retain* for every contested
item — so no removal and no lockfile change was warranted. **Closure is
blocked** on the two device-dependent acceptance criteria, which this session
cannot execute (no Android device or emulator). See § Blocked criteria.

## 1. Delivered change

```
 mobile/App.tsx                           | 5 +++--
 mobile/app.config.ts                     | 2 +-
 mobile/package.json                      | 2 +-
 mobile/src/p2p/runtime/worklet.bundle.js | 2 +-
 4 files changed, 6 insertions(+), 5 deletions(-)
```

Obsolete P0 diagnostic naming replaced with generic P2P naming:

| Surface | Before (P0) | After |
|---|---|---|
| npm script | `android:bare-probe` | `android:p2p-dev` |
| env gate | `EXPO_PUBLIC_BARE_RUNTIME_PROBE` | `EXPO_PUBLIC_P2P_DEV_HARNESS` |
| Expo `extra` key | `bareRuntimeProbe` | `p2pDevelopmentHarness` |
| `App.tsx` constant | `bareRuntimeProbeEnabled` | `p2pDevelopmentHarnessEnabled` |

`mobile/package-lock.json` is **unchanged** — the correct outcome, since the
audit removed nothing.

### Residual-reference check

```bash
grep -rn "bareRuntimeProbe\|android:bare-probe\|EXPO_PUBLIC_BARE_RUNTIME_PROBE\|BareRuntimeProbe" \
  --include="*.ts" --include="*.tsx" --include="*.js" --include="*.json" . \
  | grep -v node_modules | grep -v worklet.bundle.js
```

Zero hits in source. Remaining matches are documentation only: immutable
historical audit artifacts, ADR-043 rationale, plan/task ledgers describing
the P0 state, and roadmap X28 — all correctly retained as history.

## 2. Dependency and build-flag audit

EC-F3b is explicit that *"a native requirement is never removed on static
guesswork."* That rule decided this audit. A naive import grep suggested
`react-native-b4a` was dead; a deeper trace proved it is load-bearing, and
that removing it would have failed **silently**.

| Item | Verdict | Consumer proof |
|---|---|---|
| `react-native-bare-kit` | **retain** (uncontested per ledger) | Direct import `mobile/src/p2p/runtime/BareRuntimeClient.ts:1` |
| `minSdkVersion: 31` | **retain** (uncontested per ledger) | — |
| `b4a` | **retain** | Direct import `mobile/src/p2p/runtime/protocol.ts:1`, used at `:82` and `:97`. Also required transitively by `bare-rpc@1.3.8` and `bare-stream@2.13.4`. Linked into the packaged worklet (12 `/node_modules/b4a/` references in `worklet.bundle.js`). |
| `react-native-b4a` | **retain** | See § 2.1 — declared `peerOptional` of `b4a` itself and selected by `b4a`'s `react-native` export condition. |
| `@types/b4a` | **retain** | See § 2.2 — sole type source for `b4a`; removal breaks the `npm run typecheck` gate. |
| `useLegacyPackaging: true` | **retain** (static proof; native A/B not executed) | See § 2.3 |

### 2.1 `react-native-b4a` — proven, invisible to import grep

`npm explain react-native-b4a`:

```
react-native-b4a@0.1.0
node_modules/react-native-b4a
  react-native-b4a@"^0.1.0" from the root project
  peerOptional react-native-b4a@"*" from b4a@1.8.1
```

`b4a/package.json` selects it through an export condition:

```json
"exports": { ".": { "react-native": "./react-native.js",
                    "browser": "./browser.js",
                    "default": "./index.js" } }
```

and `b4a/react-native.js` is:

```js
try { module.exports = require('react-native-b4a') } catch { module.exports = require('./browser') }
```

It is an autolinked C++ TurboModule — `react-native-b4a/react-native.config.js`
declares `cxxModuleCMakeListsModuleName: 'react-native-b4a'` — so it is wired
by React Native autolinking from the dependency list, with **no JS import
anywhere**. No import grep can ever find its consumer.

**Failure mode if removed:** the `try/catch` swallows the resolution failure
and silently falls back to the pure-JS `browser` implementation. No error, no
crash — a quiet performance/behaviour regression on the exact path
`protocol.ts` uses for every RPC message (`b4a.from` / `b4a.toString`). This is
precisely the outcome EC-F3b exists to prevent.

### 2.2 `@types/b4a` — proven by resolution trace

```bash
npx tsc --noEmit --traceResolution | grep "'b4a'"
# ======== Module name 'b4a' was successfully resolved to
#   '/Users/matias/dubbridge/mobile/node_modules/@types/b4a/index.d.ts'
#   with Package ID '@types/b4a/index.d.ts@1.6.5'. ========
```

`b4a` declares no `types`/`typings` field and ships zero `.d.ts`
(`find node_modules/b4a -name "*.d.ts"` -> empty). `@types/b4a` is therefore
the only type source for the `import b4a from "b4a"` in `protocol.ts`, and a
wired gate (`npm run typecheck`) depends on it.

### 2.3 `useLegacyPackaging: true` — static mechanism proof

`react-native-bare-kit` ships prebuilt **Bare native addons** per ABI:

```
node_modules/react-native-bare-kit/android/src/main/addons/<abi>/
  libbare-buffer.3.7.0.so  libbare-os.3.9.3.so  libbare-inspect.3.1.5.so
  libbare-structured-clone.1.6.0.so  libbare-subprocess.6.1.0.so
```

packaged as jniLibs — `react-native-bare-kit/android/build.gradle:33`:

```gradle
jniLibs.srcDirs "src/main/addons", "libs/bare-kit/jni"
```

`useLegacyPackaging` is exactly the Android Gradle control over jniLibs
packaging (`android:extractNativeLibs`). Bare resolves and loads addons
dynamically by filesystem path, which requires the `.so` to exist as a real
extracted file — the behaviour `useLegacyPackaging: true` guarantees. With it
`false` (the modern AGP default) the addons stay compressed inside the APK.

**Confidence:** this is a mechanism-level static proof, not the executed
native A/B the ledger asks for. It justifies *retaining* the flag — the
conservative direction, and the one EC-F3b mandates. It does **not** discharge
the "native A/B result for provisional settings" evidence item.

## 3. Root cause — `worklet.bundle.js` drift

Editing `mobile/package.json` broke
`__tests__/p2p/runtime-protocol.test.ts` -> "HP-F1 builds the committed
worklet bundle deterministically", which was non-obvious:
`mobile/scripts/build-bare-worklet.mjs` reads only `protocol.ts` and
`worklet.ts`, copying them into an isolated `mkdtemp` directory that contains
no manifest.

**Cause:** `bare-pack --linked` embeds the **verbatim text of
`mobile/package.json`** into the bundle (needed for runtime module
resolution), and the bundle's `id` is a hash over that content.

Evidence — the generated bundles differ by exactly the 6 bytes the rename
removed (`android:bare-probe` -> `android:p2p-dev` is -3, and
`EXPO_PUBLIC_BARE_RUNTIME_PROBE` -> `EXPO_PUBLIC_P2P_DEV_HARNESS` is -3):

| Bundle | size | sha256 | embedded `id` |
|---|---|---|---|
| committed (pre-change) | 192639 | `eb5d080a…3b55` | `62f8938a…b6a3` |
| regenerated | 192633 | `99b8eb6d…078b` | `adadefb9…0389` |

The manifest text appears inline in the bundle at offset 177638 in both.
Isolated by bisect: reverting `package.json` alone made the check pass while
`App.tsx` and `app.config.ts` stayed modified.

**Resolution:** regenerate the artifact with `npm run build:bare-worklet`.
This is a mechanically forced, non-discretionary consequence of an in-scope
edit — the repository's own wired gate requires it.

**Note for future tasks:** any change to `mobile/package.json`, including
unrelated dependency bumps, requires regenerating `worklet.bundle.js`. Also
worth an owner decision later: the whole manifest ships verbatim inside the
app bundle, so `mobile/package.json` must never carry sensitive values.

## 4. Scope extensions beyond declared `allowed_paths`

The ledger declares `mobile/package.json`, `mobile/package-lock.json`,
`mobile/app.config.ts`, and F3b evidence. Two files outside that list were
edited; both are mechanically forced, neither is discretionary:

| File | Why forced |
|---|---|
| `mobile/App.tsx` | Sole consumer of the `bareRuntimeProbe` Expo `extra` key. Renaming the key in `app.config.ts` without updating it silently disables the harness (`undefined === true` -> `false`). Was F3a.1's allowed path. |
| `mobile/src/p2p/runtime/worklet.bundle.js` | Generated artifact; regeneration forced by the `package.json` edit per § 3. Leaving it stale keeps the wired Jest gate red, violating the "do not commit with broken tests" rule. |

Flagged for owner acknowledgement rather than assumed authorised.

## 5. Verification

| Check | Command | Result |
|---|---|---|
| Typecheck | `npm run typecheck` | PASS |
| Lint | `npm run lint` | PASS (`--max-warnings 0`) |
| Full Jest | `npm test` | PASS — 24/24 suites, 262/262 tests |
| Bundle determinism | `npm run check:bare-worklet` | PASS — `sha256=99b8eb6d…078b` |
| Lockfile integrity | `git status --short mobile/package-lock.json` | unchanged (nothing removed) |
| Android build/ping | — | **NOT RUN** — see § 6 |

## 6. Blocked criteria

Two acceptance/evidence items cannot be satisfied in this session, which has
no Android device or emulator:

1. **"Full Android build/ping passes"** (Acceptance, HP-F3b) — requires
   running `npm run android:p2p-dev` on hardware to confirm the renamed
   command and env gate actually start `P2PDevelopmentHarness` and complete a
   bounded `initialize -> ping -> shutdown`.
2. **"native A/B result for provisional settings"** (Evidence to emit) — the
   executed `useLegacyPackaging` on/off comparison. § 2.3 supplies static
   mechanism proof in its place.

This is the same constraint already recorded as roadmap **X28** for
P1.F3a.2-iv, which the owner deferred to a future general hardware
verification pass. Both items are folded into that same pass.

**F3b is therefore reported as implemented-and-audited, not PASS.** The owner
decides whether to (a) hold F3b open until the hardware pass, or (b) close it
on the static evidence and carry the device proof under X28, as was done for
F3a.2-iv.

## 7. Review

```md
- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner
- Scope-note: skips only phase-1 and phase-2 peer review; the exception expires
  after P7 reaches PASS or STOP.
```

Task-analysis review: REVIEW-OVERRIDE
`docs/audit/mvp0-p2p-review-exception.md`.
Code-solution review: REVIEW-OVERRIDE
`docs/audit/mvp0-p2p-review-exception.md`.

Per that exception, tests, coverage, scope checks, and owner final
verification remain mandatory.

## 8. Behavioral coverage

| Case | Behavior | Evidence | Result |
|---|---|---|---|
| HP-F3b (config half) | Renamed key/env/script wire the harness | `mobile/__tests__/p2p/p2p-development-harness.test.ts`; `npm run typecheck` resolves `p2pDevelopmentHarness` end-to-end | passed |
| HP-F3b (device half) | Renamed command starts the harness on Android | — | **blocked** (§ 6) |
| HP-F3b (proof half) | Every retained native setting has a documented consumer/proof | § 2 matrix | passed |
| EC-F3b (removal) | An unused dependency or unjustified flag is removed | Vacuously satisfied — every contested item proved a live consumer, so nothing qualified for removal | passed |
| EC-F3b (no guesswork) | A native requirement is never removed on static guesswork | `react-native-b4a` retained despite zero JS imports, on `peerOptional` + export-condition + autolinking proof (§ 2.1) | passed |

## Related

- `docs/tasks/mvp0-p2p-p1-replication.md` § P1.F3b
- `docs/audit/mvp0-p2p-p1-f3b-rri.md`
- `docs/audit/mvp0-p2p-review-exception.md`
- `docs/audit/mvp0-p2p-p0-native-preflight.md`
- `docs/plan/roadmap.md` § X28
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md`
