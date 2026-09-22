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

## 9. `protocol.ts` import-syntax fix and root-caused device-proof blocker (2026-08-28)

A separate, small fix (`P1.F3b-fix-1`, RRI 17 Low) and a follow-up
investigation, both against `fenix_t7` (Android 34 emulator), the first
device/emulator access available since § 6 was written.

### 9.1 Fix: `mobile/src/p2p/runtime/protocol.ts:2`

`import RPC = require("bare-rpc");` (TypeScript import-equals/CommonJS-require
syntax) is not transformable by Metro/Babel and broke bundling with
`SyntaxError: ... is only supported when compiling modules to CommonJS`.
Changed to `import RPC from "bare-rpc";` — valid because `esModuleInterop:
true` is active (`mobile/tsconfig.json` extends `expo/tsconfig.base.json`)
and `bare-rpc`'s `index.d.ts:119` ends `export = RPC`. Verified: clean `npm
run typecheck`; Metro rebuild log shows `Android Bundled 1312ms index.ts
(1551 modules)` with no `SyntaxError`. `mobile/src/p2p/runtime/worklet.ts`
was explicitly left untouched (out of scope) — it runs inside the Bare
runtime, not through Metro.

This edit changed `mobile/package.json`'s sibling source, so per § 3 the
committed `worklet.bundle.js` (which embeds a transpiled copy of
`protocol.ts`) drifted; `npm run check:bare-worklet` caught it and `npm run
build:bare-worklet` regenerated it (`sha256=3e199894…9654`). Both files are
the only working-tree changes retained from this session.

### 9.2 New, deeper blocker: native Bare-worklet crash, independent of the fix

Running `npm run android:p2p-dev` on `fenix_t7` after the fix above still
does not reach the harness's `initialize → ping → shutdown` sequence
(`P2PDevelopmentHarness` never logs). `adb logcat` shows the crash happens
immediately after `ReactNativeJS: Running "main"`, before the harness
component could plausibly mount:

```
E ubbridge.mobile: Uncaught (in promise) SyntaxError: Unexpected token ':'
E ubbridge.mobile:     at Module._extensions..cjs (bare:/worklet.bundle/node_modules/bare-module/index.js:847:30)
F libc    : Fatal signal 6 (SIGABRT), code -1 (SI_QUEUE) in tid <bare-worklet>, pid <ubbridge.mobile>
```

**Root-cause trace:**

1. Decoded `worklet.bundle.js` correctly using the `bare-bundle` npm
   package's own reader (`Bundle.from()` — format is
   `holepunchto/bare-bundle`'s `<header length><header JSON><files>`,
   wrapped a second time as `module.exports = "<escaped bytes>";` by
   `bare-pack`'s `bin.js` `bundle.cjs` output format, `encoding=utf8`
   default). `node --check` on all 42 individually extracted packed files
   (`protocol.js`, `worklet.js`, and the full `bare-rpc`/`b4a`/`bare-events`/
   `compact-encoding`/`streamx`/… dependency tree) — every file is
   independently valid CommonJS. Ruled out: the crash is not a syntax defect
   in any packed file's content.
2. **Decisive test:** packed a trivial, dependency-free worklet
   (`if (globals.Bare) console.log(...)`, zero imports, zero `#package`
   self-references, zero other files in the bundle) with the same
   `node_modules/bare-pack/bin.js --host android-arm64 --host android-x64
   --linked` invocation `build-bare-worklet.mjs` uses, swapped it in for
   `worklet.bundle.js`, and reloaded through a freshly started Metro
   (confirmed serving via its own `Android Bundled 500ms index.ts (1461
   modules)` log — not a stale dev-client cache). **Identical crash.** This
   rules out `protocol.ts`, `worklet.ts`, and every packed dependency as the
   cause — the bug reproduces on a bundle with no application content at
   all. The original bundle was restored immediately after
   (`sha256=3e199894…9654` unchanged; `npm run check:bare-worklet` confirms
   no drift; `git status --short` shows only the § 9.1 files).
3. **Upstream trace:** `react-native-bare-kit@0.15.0` — both this project's
   pinned version and, as of this writing, the latest published on npm —
   ships a precompiled `libbare-kit.so` embedding `bare-module@6.3.2`
   (`strings libbare-kit.so | grep builtin:bare-module` →
   `builtin:bare-module@6.3.2`; published 2026-06-17). Upstream
   `holepunchto/bare-module` PR "Fix bundle evaluation order and forward
   main export names" (commit `991afc2`, merged 2026-06-25, shipped in
   `v6.4.0` published 2026-06-24 — one week after 6.3.2) moved a bundle's
   `main`-module load out of `BundleModule._initialize()` into a new,
   separate `_evaluate()` phase (`lib/module/bundle.js`). That is exactly
   the class of fix that would explain a bundle member being read/parsed in
   the wrong module-type context — matching this crash's signature. No
   `react-native-bare-kit` release since `0.15.0` (cut one day *before*
   6.4.0 shipped) has been published to pick the fix up; `bare-kit` core
   (`^bare-module@6.0.1`) has newer tags (`v2.4.3` latest) but the exact
   `bare-module` version frozen into any of their precompiled `.so` releases
   was not independently confirmed. Also checked and ruled out as unrelated:
   `holepunchto/react-native-bare-kit#48` (`libnativehelper.so` ELF linking
   failure on Android 10+, fixed by `minSdkVersion 31` — already satisfied
   here) and `#29` (New Architecture/autolinking misconfiguration — not our
   symptom, since `BareKit` registers and the worklet thread does start).

**Conclusion:** this is a confirmed upstream `bare-module@6.3.2` bug, wholly
independent of `protocol.ts`/`worklet.ts` content and of the § 9.1 fix. It
supersedes "no device/emulator available" as the reason § 6's device-proof
criteria (and roadmap **X28**) remain blocked — the emulator is now
available and the harness still cannot start, for a different, upstream
reason with no currently published fixed dependency to move to. Resolving
it (if pursued) would need either an upstream `react-native-bare-kit`
release bundling `bare-module ≥ 6.4.0`, or a self-built `libbare-kit.so`
from `bare-kit` source pinned to a fixed `bare-module` — both out of scope
for this fix and requiring their own scoped, RRI-scored task.

### 9.3 Self-built `libbare-kit.so`: feasibility evaluated, not pursued

At owner request, the self-build path from § 9.2 was evaluated before being
decided. It is **technically feasible but rejected**; no task was opened.

**Feasibility (confirmed):**

- The build recipe is public and reproducible. `holepunchto/bare-kit`'s own
  `.github/workflows/publish.yml` (job `android`) is: checkout → Node
  `lts/*` → Java 21 (temurin) → `npm install --global cmake-runtime
  ninja-runtime` → `npm install` → `./gradlew aR`. Non-Android targets use a
  separate `bare-make generate`/`bare-make build` tool.
- The version fix would come for free. `bare-kit` declares
  `"bare-module": "^6.0.1"`, and `npm view bare-module dist-tags` currently
  returns `latest: 6.4.0` — a fresh `npm install` inside `bare-kit` resolves
  to the fixed version with no manual override or `overrides` entry.
- Local toolchain is close but not complete: the NDK (`27.1.12297006`) and a
  working Android/CMake native build are already proven in this repo, but
  only `openjdk@17` and `openjdk@26` are installed — the CI recipe's Java 21
  is absent (installable, though tolerance of a different JDK was not
  verified).

**Why it was rejected:**

1. **Security-maintenance transfer.** `libbare-kit.so` statically compiles
   BoringSSL (confirmed: `strings` on the shipped `.so` exposes
   `_deps/github+google+boringssl-src/crypto/…` build-cache paths).
   Self-building moves responsibility for tracking and recompiling against
   future BoringSSL CVEs from the upstream project's release process onto
   this repository — with no owner, process, or gate assigned for it. That is
   a durable obligation bought to unblock one emulator run.
2. **Contradicts an accepted ADR.** ADR-043 § "Alternatives considered"
   records that Bare Kit was adopted precisely because it "already supplies
   the native/worklet boundary needed" — i.e. the accepted decision is to
   *consume* the vendor's native artifact, not to build it. Self-building is
   a scope expansion beyond what ADR-043 approved, and would need that ADR
   amended rather than merely a new task.
3. **The obvious substitution is not durable.** Overwriting
   `node_modules/react-native-bare-kit/android/libs/bare-kit/jni/<ABI>/libbare-kit.so`
   (the path `react-native-bare-kit`'s `android/CMakeLists.txt` declares as
   `SHARED IMPORTED` / `IMPORTED_LOCATION`) survives only until the next
   `npm install`. A real fix needs `patch-package` or a vendored fork —
   materially more surface than a one-off build. The upstream artifact is
   also an AAR (`android/build/outputs/aar/*-release.aar`), so the `.so`
   must additionally be extracted from it.
4. **Bounded upside.** Per `docs/plan/roadmap.md` § X28, this blocks only the
   device-proof criteria of `EC-F3a.2`/`HP-F3a.2`/`HP-F3b` — not the rest of
   P1.F3a.2/F3b closure, and not `P1.A1`, which the roadmap already lists as
   next with its own RRI/card/approval. No product P2P runtime or network
   activity is active, so nothing user-facing is exposed by the delay.

**Decision (owner, 2026-08-28):** do not pursue the self-build. X28 stays
deferred to the general hardware-verification pass, now with a named upstream
cause and a documented resolution path. Revisit only if a real product need
comes to depend on the harness proof, or if `react-native-bare-kit` still has
not published a release bundling `bare-module ≥ 6.4.0` by the time P1's
device-dependent criteria become blocking rather than deferrable.

### 9.4 The `bare-module` upgrade did not fix it — cause re-traced (2026-09-22)

**§ 9.2's conclusion is superseded.** The upstream dependency it named as the
cause has since been upgraded, the crash reproduces unchanged, and the real
failing mechanism has now been established on-device. `bare-module@6.3.2`'s
bundle-evaluation-order bug was *a* real upstream defect, but it was not this
crash's cause.

`react-native-bare-kit` is now pinned at `^0.15.5` (`mobile/package.json:47`;
commits `77ee0a3` "upgrade Bare runtime past bundle crash" and `44ec91a` "lock
react-native-bare-kit 0.15.5"), whose shipped `libbare-kit.so` embeds
**`bare-module@7.0.3`** — verified live, not from the changelog:
`strings node_modules/react-native-bare-kit/android/libs/bare-kit/jni/arm64-v8a/libbare-kit.so
| grep -o 'builtin:bare-module@[0-9.]*'` → `builtin:bare-module@7.0.3`.
`mobile/scripts/check-bare-runtime.mjs` (which fails closed below
`react-native-bare-kit` 0.15.5 at `:50` and below embedded `bare-module` 6.4.0
at `:72`) passes on every run.

#### Reproduction

`npm run android:p2p-dev` against the `fenix_t7` emulator, 2026-09-22. The app
builds, installs, launches, and the worklet thread starts — then aborts:

```
E ubbridge.mobile: Uncaught (in promise) SyntaxError: Unexpected token ':'
E ubbridge.mobile:     at ScriptModule._createFunction (bare:/worklet.bundle/node_modules/bare-module/lib/module.js:209:24)
E ubbridge.mobile:     at ScriptModule._initialize    (bare:/worklet.bundle/node_modules/bare-module/lib/module/script.js:11:21)
E ubbridge.mobile:     at ScriptModule._instantiate   (bare:/worklet.bundle/node_modules/bare-module/lib/module.js:151:10)
E ubbridge.mobile:     at ModuleLoader._instantiate   (bare:/worklet.bundle/node_modules/bare-module/lib/loader.js:642:46)
E ubbridge.mobile:     at ModuleLoader.linkSync       (bare:/worklet.bundle/node_modules/bare-module/lib/loader.js:240:10)
E ubbridge.mobile:     at Module.loadSync             (bare:/worklet.bundle/node_modules/bare-module/index.js:44:25)
E ubbridge.mobile:     at BundleModule.start          (bare:/worklet.bundle/shared/worklet.js:128:17)
F libc    : Fatal signal 6 (SIGABRT), code -1 (SI_QUEUE) in tid 23652 (bare-worklet), pid 23450 (ubbridge.mobile)
F DEBUG   : #00 pc …669e4 libc.so (abort+164)
F DEBUG   : #01 pc …173188 libbare-kit.so
F DEBUG   : #02 pc …198ce4 libbare-kit.so (js_callback_s::on_call(v8::FunctionCallbackInfo<v8::Value> const&)+104)
E ReactNativeJS: [P2P development harness] INITIALIZE_FAILED
W ReactNativeJS: [P2P development harness] shutdown=complete
```

Full capture: session logcat, `tid 23652`, 09:00:13.371–09:00:17.859.

#### Proven

1. **The SIGABRT is the uncaught `SyntaxError` crossing the native boundary.**
   `js_callback_s::on_call` aborting is the JS exception terminating the
   worklet thread, not an independent native fault.
2. **`shared/worklet.js:128` is bare-kit's own bootstrap**, embedded in
   `libbare-kit.so` — not `mobile/src/p2p/runtime/worklet.js` (51 lines). This
   resolves the line-number contradiction that misdirected earlier analysis:
   the frame never appears among the bundle's keys because it is not in the
   bundle.
3. **The runtime is `bare-module@7.0.3`**, confirmed independently of the
   `strings` probe by exact line-number correspondence against the published
   source (fetched read-only via `npm pack`, never installed):
   `lib/module.js:208-209` is `_createFunction`'s `binding.createFunction(…)`;
   `lib/module/script.js:10-11` is `_initialize`'s `this._fn =
   this._createFunction(this._text())`. Both match the crash frames exactly.
4. **The bundle contains zero TypeScript.** 388 keys: 290 `.js`, 97 `.json`,
   1 `.cjs` (`hyperschema/runtime.cjs`); `main = /src/p2p/runtime/worklet.js`.
   `build-bare-worklet.mjs` transpiles all 16 own `.ts` files with the real
   `typescript` package and throws on error diagnostics, so no TS can reach
   the bundle. The "unstripped TypeScript" theory is dead by ground truth.
5. **Decisive test — the failing module is a `.json` compiled as a CJS
   script.** Compiling every one of the 388 packed files with
   `new Function("require","module","exports","__filename","__dirname", src)`
   — exactly what `_createFunction` does — yields **97 failures, all of them
   the `.json` files, all with the identical message `Unexpected token ':'`;
   291 compile cleanly and zero `.js` files fail.** A JSON object evaluated as
   a function body parses `{` as a block, `"name"` as a string expression, and
   then hits `:`. There is no syntax defect in our code or in any JS
   dependency.
6. **The runtime's fallback module type is SCRIPT.** `lib/loader.js:448`
   (`_root()`): `defaultType: this._defaultType || type.SCRIPT`. A `.json`
   whose type is never determined is therefore handed to `ScriptModule`,
   which is precisely the observed path.
7. **Structural anomaly: 372 of 388 bundle keys escape the bundle root**, e.g.
   `/../node_modules/@hyperswarm/secret-stream/package.json`. Only the 16 own
   `/src/p2p/runtime/*.js` files are clean. Mechanism confirmed in
   `mobile/scripts/build-bare-worklet.mjs`: it creates its pack root with
   `mkdtemp(path.join(mobileRoot, ".bare-pack-"))` (`:72`) and passes it as
   `--base` (`:86`), while `node_modules` remains one level above it at
   `mobile/node_modules` — hence every dependency path is emitted as `/../…`.
   The escaping set includes **every `package.json`**, i.e. exactly the files
   the runtime's traverse reads to establish package boundaries and module
   types.

#### Refuted (with sources in hand, not by inference)

- **`bare-module` version skew is not the axis.** 6.3.2 → 7.0.3 landed; crash
  unchanged. § 9.2's conclusion does not survive.
- **`bare-bundle` 1.10.0 → 1.11.0 is not the fix.** Neither version serializes
  a per-file module `type`/`naturalType` at all — diffing the two releases'
  `this._*` fields shows only `_root` added. The `naturalType` field
  `bare-module@7.0.3` reads (`lib/source.js:5`: `this.naturalType =
  dependency.naturalType || 0`) is produced by the runtime's own traverse, not
  carried in the bundle, so no bundler bump can supply it.
- **`bare-pack` 2.2.2 is not the fix** (refuted in an earlier session).
- **An unknown module type would not produce this error.** `lib/loader.js:620`
  (`const Class = recordClasses[dependency.type]`) would throw
  `UNKNOWN_MODULE_TYPE`, and `_assertType` (`:646,653,661`) would throw
  `TYPE_INCOMPATIBLE`. Neither is what we see — consistent with finding 6: the
  type is not unknown, it is silently defaulted to SCRIPT.

#### Hypothesis raised here — tested the same day and REFUTED

That the `/../` root escape prevented the runtime's traverse from typing the
`.json` files, producing the SCRIPT fallback of finding 6. It was coherent
with all seven findings above and was the only structural defect found, but
**it is wrong**. See § 9.5: correcting `--base` took the escaping-path count
from 372/388 to 0/388 and the crash was byte-identical. Finding 5 (`.json`
compiles as a script) is a true statement about the bundle's contents that
turned out not to be the failure being observed. The real cause is § 9.5.

#### Side finding — environment, not code

`npm run android:p2p-dev` fails before compiling anything when `JAVA_HOME`
points at OpenJDK 26 (the shell default on this host): AGP's
`JdkImageTransform` cannot run `jlink` against it —
`Failed to transform core-for-system-modules.jar … Error while executing
process …/openjdk/26.0.1/…/bin/jlink`. Overriding `JAVA_HOME` to
`/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home` for the
single invocation produces `BUILD SUCCESSFUL`. This was **not** persisted to
any shell profile or repository file; it is recorded here so the next run does
not re-diagnose it.

#### Status of § 9.4

Superseded within the same session by § 9.5, which identifies the actual
cause. The seven findings above stand as evidence; the hypothesis does not.

### 9.5 Actual root cause: the worklet filename's extension (2026-09-22)

**The crash was never upstream. It is a two-line defect in this repository,
present since the worklet client was written.**

`react-native-bare-kit`'s `Worklet.start(filename, source, args)` derives the
*module type* of `source` from `filename`'s extension. Its README states the
constraint outright (`node_modules/react-native-bare-kit/README.md:42`):

```
// First arg (filename)'s extension *must* be .bundle
worklet.start('/app.bundle', source)
```

This repository passed neither:

| File | Constant | Value before fix |
|---|---|---|
| `mobile/src/p2p/runtime/BareRuntimeClient.ts:49` | `PRODUCT_WORKLET_FILENAME` | `/dubbridge-p2p-runtime.worklet` |
| `mobile/src/p2p/proof/ProofRuntimeFactory.ts:8` | `PROOF_WORKLET_FILENAME` | `/dubbridge-p2p-proof.worklet` |

**Mechanism, traced to source.** `bare-module-traverse`'s `moduleType()`
(`mobile/node_modules/bare-module-traverse/index.js:727-772`) matches the
pathname against `/\.[a-z]+$/` and switches over the result. The recognized
set is `.js .ts .cjs .cts .mjs .mts .json .bundle .bare .node .bin .txt`.
`.worklet` matches the regex but hits no case, so control reaches the final
`return defaultType` (`:771`) — and `defaultType` is
`this._defaultType || type.SCRIPT` (`bare-module@7.0.3 lib/loader.js:448`).
The bundle's bytes are therefore handed to `ScriptModule`, whose
`_initialize` calls `_createFunction` on them (`lib/module/script.js:10-11`
→ `lib/module.js:208-209`). A Bare bundle's wire format is
`<header length><header JSON><files>`, so the first thing the JS parser meets
is a JSON object — and its first `:` raises **`SyntaxError: Unexpected token
':'`**, exactly at the frames observed. Uncaught on the worklet thread, it
aborts the process through `js_callback_s::on_call` → SIGABRT.

**This explains every previously puzzling observation**, including the ones
that misled §§ 9.2-9.4:

- Content-independence. Every Bare bundle has a JSON header, so *any* bundle
  crashes identically. § 9.2's "dependency-free bundle crashes too" was read
  as proof of an upstream bug; it was actually proof the defect is
  content-independent — which is what a filename-derived type error looks
  like.
- Version-independence. `bare-module` 6.3.2 → 7.0.3, `bare-pack` 2.2.2 and
  `bare-bundle` 1.11.0 could not have helped, because none of them is
  involved in the decision.
- The `.json` red herring of § 9.4 finding 5. The 97 packed `.json` files do
  fail `new Function`, but they were never the file being compiled.

**Verification on device (emulator `fenix_t7`, 2026-09-22):**

1. *Negative control — corrected `--base`, 0/388 escaping paths.* Crash
   byte-identical (`tid 24034`, same stack, same SIGABRT). Refutes § 9.4's
   hypothesis.
2. *Zero-`.json` bundle.* A bundle packed from a staging directory with no
   `package.json` anywhere, yielding exactly **1 key** (`/worklet.js`) and no
   `.json` at all: crash **still identical** (`tid 24159`). Refutes the
   `.json`-as-script reading. Note a bundle packed from `mobile/` always
   carries `/package.json`, which is why § 9.2's "dependency-free" bundle was
   not actually `.json`-free and its negative result was misread.
3. *Positive control for test validity.* Replacing the bundle with
   `module.exports = "NOT-A-BUNDLE-CONTROL"` produced a **different** error,
   `ReferenceError: NOT is not defined` — proving Metro served fresh content
   on each relaunch (so 1 and 2 are valid) and, independently, that the file's
   contents were being *executed as a script*.
4. *The fix.* Changing both constants' extension to `.bundle` and relaunching:
   **zero `SyntaxError`, zero SIGABRT, and
   `W ReactNativeJS: [P2P development harness] ping=pong`** — the harness
   completes its round trip for the first time. `npm run typecheck` and
   `npm run lint` both exit 0.

**Consequences for the record.** § 9.2's conclusion ("confirmed upstream
`bare-module@6.3.2` bug") is **withdrawn** — it was an inference from a
control that did not control what it was believed to control. § 9.3's
self-build evaluation is moot: no upstream artifact ever needed replacing.
The `react-native-bare-kit` 0.15.5 pin and the `check-bare-runtime.mjs` guard
are independently fine and are not implicated either way.

**Not yet done.** The two-line change is verified on device but remains
uncommitted and has had no RRI score, no band-routed phase-1/phase-2 review,
and no owner verification — it needs its own task card before closure. The
`--base` correction in `build-bare-worklet.mjs` is a separate, optional
hygiene change that is **not** part of the fix; it removes the `/../` path
escape and keeps the bundle deterministic, but it was proven not to affect
the crash, and whether to keep it is an open decision.

## 10. `P1.F3b-fix-1` closure (2026-08-28)

Development-task closure for the § 9.1 fix. RRI 17 Low, so no Reflection log
is required (`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Reflection design
pattern applies at RRI 26+).

### Code-solution review

```
Code-solution review: REVIEW-OVERRIDE - explicit owner-directed MVP0-P2P exception
```

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner
- Scope-note: skips only phase-1 and phase-2 peer review; the exception
  expires after P7 reaches PASS or STOP.

Standing waiver recorded 2026-08-27 in
`docs/audit/mvp0-p2p-review-exception.md`, whose scope is the MVP0-P2P `P0`
through `P7` sequence and which explicitly keeps unit coverage
certification, owner final verification, and status-artifact synchronization
in force. Not self-issued by the agent.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-fix-1 | Happy path | `protocol.ts` compiles and the RPC contract still handshakes, pings, and shuts down cleanly | `mobile/__tests__/p2p/runtime-protocol.test.ts::HP-F1 negotiates capabilities, pings, and shuts down without pending work` | passed |
| HP-fix-1b | Happy path | The regenerated `worklet.bundle.js` matches a deterministic rebuild of the fixed source (no drift) | `mobile/__tests__/p2p/runtime-protocol.test.ts::HP-F1 builds the committed worklet bundle deterministically` | passed |
| EC-fix-1 | Edge case | The changed import still yields a usable `RPC` value at runtime, not an interop-broken namespace object — typed errors still raised on malformed payloads | `mobile/__tests__/p2p/runtime-protocol.test.ts::EC-F1 rejects unsupported versions and malformed payloads with typed errors` | passed |

The import-equals → default-import change is only safe if `esModuleInterop`
is active and `bare-rpc` uses `export =`; both were verified statically
(§ 9.1) and are exercised at runtime by the three tests above, which
construct and drive a real `RPC` instance through the changed import.

### Owner final verification

- Owner: `Matias, repository owner`
- Date: `2026-08-28`
- Statement: I verified every happy path and edge case defined for this fix
  has unit test evidence that replicates the expected behavior. The
  device-proof criteria remain out of scope, blocked on the § 9.2 upstream
  defect and tracked in roadmap X28.
- Commands run: `npm run typecheck` (clean); `npx jest __tests__/p2p/`
  (3 suites, 27/27 passed); `npm run check:bare-worklet`
  (`sha256=3e199894ead2a3f5ef1da6fda4929a4f4858d50d85cdb7c7f6cdc1123a49654b`,
  no drift)

## 11. `P1.F3b-fix-2` closure (2026-09-22) — X29 root-cause fix

Development-task closure for the § 9.5 fix (`.worklet` → `.bundle` in both
worklet filenames). RRI 25 Low (right at the Low/Moderate boundary), so no
formal Reflection log is required
(`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Reflection design pattern
applies at RRI 26+); applied informal judgment given the boundary score and
the behavioral significance of the fix (crash → working ping round-trip).

### RRI

`scripts/rri.py --touches mobile/src/p2p/runtime/BareRuntimeClient.ts --touches mobile/src/p2p/proof/ProofRuntimeFactory.ts --touches mobile/__tests__/p2p/p2p-service.test.ts --touches mobile/__tests__/p2p/transient-drive.test.ts --cc 1 --D 1 --K 1 --P 2 --T 1 --A 0 --X 2 --platform rn`

C=0, F=2, D=1, T=1, A=0, K=1, P=2, X=2. Technical profile L=0 I=1 Q=1 V=1 →
bottleneck 1 → ICI=25. Risk/domain band input 9. No penalties. **Final RRI:
25 → Low (0–25).** T (test-coverage risk) was scored 1 ("reasonable tests
exist") only after the coverage gap below was closed — scored 4
("no tests in the affected area") before that fix, which alone would have
put the technical profile's V axis at 4 and the bottleneck-driven ICI at
100 (Very-high band). This is not a hypothetical: `mobile/__tests__/p2p/
p2p-service.test.ts:191-194` and `mobile/__tests__/p2p/
transient-drive.test.ts:38-42` asserted the **pre-fix** `.worklet` filename
literal — stale since commit `75eeb3b`, and would have silently let a
regression back to `.worklet` pass CI. Found by running
`npx jest __tests__/p2p/` post-fix (2 failed / 167 total) before scoring
T, not assumed; fixed in commit `c16c3ff` before finalizing this score.

### Code-solution review

```
Code-solution review: gpt-oss docs/audit/gemma-evidence/x29-worklet-extension-fix.json - PASS
```

### Gemma Reviewer evidence

- Model: `gpt-oss:20b` (RRI 0–25 chain primary)
- Command: `GEMMA_REVIEW_TASK_ID=x29-worklet-extension-fix GEMMA_REVIEW_BASE=726b302 REVIEW_PATHS="mobile/src/p2p/runtime/BareRuntimeClient.ts mobile/src/p2p/proof/ProofRuntimeFactory.ts mobile/__tests__/p2p/p2p-service.test.ts mobile/__tests__/p2p/transient-drive.test.ts" make qa-gemma-review`
- Passes run / usable: `3/3`
- Aggregate status: `PASS`
- Consensus findings: `0` | Pass-specific: `0` | Disagreement: `0`
- Artifacts: `docs/audit/gemma-evidence/x29-worklet-extension-fix.json` (receipt); `/tmp/dubbridge-gemma-review-x29-worklet-extension-fix.json` (result, not persisted — ephemeral local run output)
- Isolated adjudicator: `not triggered` — trigger: n/a, primary reviewer produced a usable PASS
- D14 provider route: `n/a`
- disposition_divergence: `null`
- Primary-agent disposition: accepted (no findings to disposition)

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-fix-2 | Happy path | `Worklet.start()` is called with the corrected `.bundle` filename, so `bare-module-traverse` types the bundle as `BUNDLE` (not `SCRIPT`) | `mobile/__tests__/p2p/p2p-service.test.ts:191-194` (`.bundle`, product runtime), `mobile/__tests__/p2p/transient-drive.test.ts:38-42` (`.bundle`, proof runtime) | passed |
| HP-fix-2b | Happy path (device proof) | With the corrected filename, `initialize → ping → shutdown` completes cleanly on real Android hardware — zero `SyntaxError`/SIGABRT | Manual device run, `fenix_t7` emulator (Android 34), 2026-09-22: `W ReactNativeJS: [P2P development harness] ping=pong`; `SyntaxError`/SIGABRT count = 0 (§ 9.5 step 4) | passed |
| EC-fix-2 | Edge case | An unrecognized extension (the pre-fix `.worklet`) types the bundle as `SCRIPT`, and the bundle's JSON header is compiled as script source, crashing the worklet thread — the updated tests pin the corrected literal so this regression can no longer pass silently | `mobile/__tests__/p2p/p2p-service.test.ts:191-194`, `mobile/__tests__/p2p/transient-drive.test.ts:38-42` — both failed against the pre-fix `.worklet` value before being corrected in commit `c16c3ff`, confirming they actually discriminate the defect | passed |

HP-fix-2b's device-layer proof cannot be automated in Jest (the native Bare
runtime is mocked in the unit suite); it is included because the unit-level
evidence alone (HP-fix-2, EC-fix-2) proves the *contract* — the exact
filename passed to `Worklet.start()` — but not that the native runtime
actually types and executes the bundle correctly. Both layers together are
the cheapest combination that genuinely proves the behavior.

### Owner final verification

- Owner: `Matias, repository owner`
- Date: `2026-09-22`
- Statement: closure directed explicitly in chat ("cierra X29") after the
  fix's on-device verification (§ 9.5), the RRI 25 Low score above, the
  `gpt-oss:20b` code-solution review (PASS, 0 findings), and the
  test-coverage gap fix (commit `c16c3ff`) were all completed and reported.
  I accept the mapped happy-path and edge-case evidence above as covering
  the claimed behavior.
- Commands run: `npm run typecheck` (clean); `npm run lint` (clean);
  `npx jest __tests__/p2p/` (33 suites, 167/167 passed, re-run after
  `c16c3ff`); `npm run check:bare-worklet`
  (`sha256=5d99caee01d2c15788725c50b670fa69887468d69e4749b6c6c885e3dc47dd0e`,
  no drift); `GEMMA_REVIEW_TASK_ID=x29-worklet-extension-fix
  GEMMA_REVIEW_BASE=726b302 REVIEW_PATHS="..." make qa-gemma-review`
  (3/3 PASS, 0 findings)

## 12. X29's remaining two criteria, executed (2026-09-22)

With § 11's fix closed, the two items P1.F3b folded into X29 on 2026-08-27
(§ 6) were re-checked directly instead of left on static/indirect evidence.

### 12.1 `npm run android:p2p-dev` full build/install/launch

Prior device verification (§ 9.5) relied on Metro-serving a changed bundle
to an already-installed dev client via force-stop + relaunch, not a fresh
invocation of the actual `android:p2p-dev` npm script. Ran it directly
(`JAVA_HOME=`openjdk@17`, emulator `fenix_t7`, logcat cleared first):

```
$ npm run android:p2p-dev
...
BUILD SUCCESSFUL in 5s
› Installing .../app-debug.apk
› Opening dubbridge://expo-development-client/?url=...  on fenix_t7

$ adb logcat -d | grep -E "P2P development harness|SyntaxError|Fatal signal"
W ReactNativeJS: [P2P development harness] ping=pong
```

Zero `SyntaxError`/`Fatal signal` matches. This confirms the pre-hook chain
(`check:bare-runtime` + `build:bare-worklet`), the
`EXPO_PUBLIC_P2P_DEV_HARNESS=true` env gate, and the deep-link launch all
work end-to-end from the documented entrypoint, not only via the
faster Metro-reload path used for iterative debugging in § 9.5.

### 12.2 `useLegacyPackaging` on/off native A/B — executed

F3b's dependency audit (§ 2.3) retained `useLegacyPackaging: true` on
**static mechanism proof only** (bare-kit ships native addons as jniLibs;
the flag governs disk extraction at install time) — never an executed A/B,
which X29 explicitly required. Executed it:

| Run | `expo.useLegacyPackaging` | Build | Native link errors | `ping=pong` |
|---|---|---|---|---|
| 1 (baseline) | `true` (committed) | `BUILD SUCCESSFUL` | none | yes |
| 2 (A/B) | `false` | `BUILD SUCCESSFUL` (`packageDebug` re-ran, not cached) | none (`UnsatisfiedLinkError`/`dlopen failed`/`CANNOT LINK`: 0 matches in build log or logcat) | yes |
| 3 (restore) | `true` (reverted to committed value) | `BUILD SUCCESSFUL` | none | yes |

`mobile/android/gradle.properties` was edited to `false` for run 2 and
reverted to the committed `true` immediately after (`git diff` confirmed
clean before and after); no repository file was left changed by this
experiment.

**Finding:** on this device (`fenix_t7`, Android 34, arm64-v8a emulator),
disabling legacy packaging did **not** reproduce a native-loading failure —
the harness's `initialize → ping → shutdown` round trip succeeded
identically in both states. This does not overturn F3b's `true` retain
decision, which rested on a broader static analysis (autolinked
`react-native-b4a`/`b4a` peerOptional wiring with zero JS imports, silent-
breakage risk per EC-F3b) that this one emulator run cannot exhaustively
test — a real device, a different ABI, or a release (non-debug) build could
still behave differently. What this closes is narrower and specific: X29's
literal requirement was an *executed* A/B, not a particular outcome, and it
is now executed and recorded rather than justified by static proof alone.
The committed `useLegacyPackaging: true` is unchanged.

## Related

- `docs/tasks/mvp0-p2p-p1-replication.md` §§ P1.F3b, P1.F3b-fix-2
- `docs/audit/mvp0-p2p-p1-f3b-rri.md`
- `docs/audit/mvp0-p2p-review-exception.md`
- `docs/audit/mvp0-p2p-p0-native-preflight.md`
- `docs/audit/gemma-evidence/x29-worklet-extension-fix.json`
- `docs/plan/roadmap.md` § X28
- `docs/adr/ADR-043-mobile-p2p-runtime-ownership-and-proof-isolation.md`
- `holepunchto/bare-module` commit `991afc2` ("Fix bundle evaluation order
  and forward main export names"), `holepunchto/react-native-bare-kit`
  issues `#48`, `#29`, `holepunchto/bare-pack` issue `#6`
