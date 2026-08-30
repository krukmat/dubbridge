---
type: Audit
title: "MVP0-P2P P1.A1 implementation evidence"
task: P1.A1
date: 2026-08-30
status: pass
---

# MVP0-P2P P1.A1 — implementation evidence

**Task:** `docs/tasks/mvp0-p2p-p1-replication.md` § P1.A1 — Hyperdrive/Corestore
Android bundle smoke proof (planning parent, decomposed on 2026-08-28 into
P1.A1a, P1.A1b.0, P1.A1b, P1.A1c, and the evidence-only P1.A1d closure).

**Parent RRI:** 38 Moderate (`scripts/rri.py --touches mobile/package.json
--touches mobile/package-lock.json --touches mobile/src/p2p/runtime --touches
mobile/__tests__/p2p/hyperdrive-smoke.test.ts --C 2 --D 2 --K 2 --P 2 --T 2
--A 1 --X 2 --platform rn`).

**Parent-level task-analysis review (before decomposition):** gemma
(`gemma4:26b-a4b-it-qat`, 3/3 passes) - PASS. 2 consensus findings (both
`minor`, 0 blocking), incorporated into every child's acceptance criteria:
1. "redacted capability receipt" ambiguity — bounded to `capability` +
   `schema_version` fields only.
2. Risk of conflating the X28 upstream transport defect with a genuine
   bundling test failure — X28-attributable failures must classify as
   `Environment/Blocked`, never a test failure.
3. (likely-false-positive, adopted anyway) "invalid path" EC case must run
   through a local-only/stubbed filesystem driver.
4. (likely-false-positive) same receipt-schema ambiguity as #1.

This document accumulates evidence per child as each closes. P1.A1a, P1.A1b.0,
P1.A1b, P1.A1c, and P1.A1d are PASS. This is a bundle and isolated-test proof
only: X28 continues to block an on-device Bare-runtime execution claim, so this
record does not represent a physical Android runtime PASS.

## P1.A1a — Add Corestore/Hyperdrive dependencies + bundle check

**RRI:** 14 Low (`scripts/rri.py --touches mobile/package.json --C 0 --D 1
--K 1 --P 1 --T 1 --A 0 --X 1 --platform rn`).

**Implementation route:** local-first delegation via
`scripts/delegate-low-rri.py` (`--mode before-after`, target
`mobile/package.json`), `qwen3.8:27b-mlx`. Attempt 1 was rejected: the model
altered `bare-rpc` (`^1.3.8` -> `^2.2.0`, not requested), altered `expo`
(`~56.0.9` -> `~51.0.0`, not requested, an unrequested downgrade), and
deleted the `b4a` line entirely — a scope violation under EC-A1a ("no other
line in the file is changed"). Attempt 2 (repair, explicit before/after
anchors + forbidding invented version changes) produced correct content but
with a stray extra indentation space on all five touched lines; the
orchestrator applied the already-verified two-line content directly
(mechanical formatting fix, no logic authored) rather than a third local
attempt. Phase-2 review (Muse Glimmer, 3/3 passes, consensus) then caught a
real defect the orchestrator introduced: `hyperdrive` was inserted
immediately after `expo` but before the `expo-*` block, violating strict
alphabetical order (HP-A1a). Corrected by moving `hyperdrive` to after the
last `expo-*` entry.

**Final diff (post phase-2 correction):**

```diff
diff --git a/mobile/package.json b/mobile/package.json
index 457bce0..3fdd22c 100644
--- a/mobile/package.json
+++ b/mobile/package.json
@@ -21,6 +21,7 @@
     "@react-navigation/native-stack": "^7.16.0",
     "b4a": "^1.8.1",
     "bare-rpc": "^1.3.8",
+    "corestore": "^7.12.2",
     "expo": "~56.0.9",
     "expo-auth-session": "~56.0.13",
     "expo-build-properties": "~56.0.26",
@@ -32,6 +33,7 @@
     "expo-status-bar": "~56.0.4",
     "expo-video": "^56.1.4",
     "expo-web-browser": "~56.0.5",
+    "hyperdrive": "^13.3.3",
     "react": "19.2.3",
     "react-native": "0.85.3",
     "react-native-b4a": "^0.1.0",
```

`package-lock.json` regenerated via `npm install` (not hand-authored by the
model): 59 packages added, 0 errors. Pre-existing `react-native-worklets`
peer-dependency warnings unrelated to this change (present before it).

**Verification commands run (final, post-correction):**
- `npm install` → resolved cleanly.
- `npm run build:bare-worklet` → rebuilt clean
  (`sha256=a99e33ec71ee70f723ec1b0182a1a1976340f7e7610bd402e4442db38196c46f`)
  after the alphabetical-order correction (the bundle embeds `package.json`
  as manifest text, so any change to it requires a rebuild). Confirmed the
  bundle pulls in no hyperdrive/corestore logic — it still only resolves
  `bare-events`/`bare-rpc`/`b4a`; the only textual match is the embedded
  `package.json` manifest text, not an import.
- `npm run typecheck` → clean, 0 errors.
- `npx jest __tests__/p2p/` → 3 suites, 27/27 passed (initially 26/27 with
  one failure from bundle drift after the alphabetical-order edit; resolved
  by the rebuild above).

**Acceptance:**
- HP-A1a: satisfied — both dependencies added in strict alphabetical
  position with the exact specified version strings; file remains valid
  JSON.
- EC-A1a: satisfied — final diff touches exactly two inserted lines; no
  existing line altered.

### Code-solution review

- Reviewer: `muse-glimmer` (phase-2, RRI 0-25 Low band primary)
- Command: `python3 scripts/gemma-code-review.py --model muse-glimmer:30b-q4_K_M --num-ctx 65536 --no-think --passes 3`
- Artifact: `scratchpad/p1a1a-phase2-result.json` (session-local; not
  committed — summarized here per evidence-block contract)
- Passes run / usable: 3/3
- Aggregate status: `FINDINGS` → 1 consensus `blocking` finding (alphabetical
  order violation), 0 false positives
- Primary-agent disposition: accepted and fixed (see corrected diff above);
  re-verified via full toolchain re-run (typecheck, rebuild, 27/27 tests)
- Isolated adjudicator (D14): not triggered — Muse Glimmer responsive and
  usable
- disposition_divergence: `none`

## Unit coverage certification (P1.A1a)

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-A1a | Happy path | Corestore/Hyperdrive added in alphabetical order with exact versions; file stays valid JSON | Verified via `npm install` (parses `package.json`), `npm run typecheck`, `npx jest __tests__/p2p/` (all consume the resolved dependency tree without error) | passed |
| EC-A1a | Edge case | No other line in the file changed | `git diff mobile/package.json` — exactly 2 inserted lines, reviewed above | passed |

Low-band task: no dedicated Jest test file was required for this
dependency-manifest-only change; the acceptance criteria are structural
(diff shape, JSON validity, toolchain resolution), not runtime behavior, so
the existing full-suite green run plus the reviewed diff constitute the
evidence.

## Owner final verification (P1.A1a)

- Owner: Matias, repository owner
- Date: 2026-08-28
- Statement: I verified HP-A1a and EC-A1a are satisfied by the final diff
  (corestore/hyperdrive added in strict alphabetical order, no other line
  touched), and that the phase-2 Muse Glimmer finding was correctly accepted
  and fixed rather than dismissed.
- Commands run: `npm install`, `npm run build:bare-worklet`, `npm run
  typecheck`, `npx jest __tests__/p2p/`

**Status: P1.A1a — PASS / Done 2026-08-28.**

## P1.A1b.0 — Proof-storage contract preflight

**Status: PASS / Done 2026-08-30.** The frozen host-to-Bare cache-root, RPC,
receipt, and failure contract is recorded in
`docs/audit/mvp0-p2p-p1-a1b-storage-contract.md`. It authorized no source,
dependency, bundle, Android, device, network, or storage action.

## P1.A1b — Transient drive open/close logic

**Status: PASS / Done 2026-08-30.** Its focused verification, coverage
certification, and owner waiver for the sole no-action phase-2 finding are
recorded in `docs/audit/mvp0-p2p-p1-a1b-forced-closure.md` and the P1 ledger.

## P1.A1c — Typed error handling and coverage

**Status: PASS / Done 2026-08-30.** The implementation record at
`docs/audit/mvp0-p2p-p1-a1c-implementation.md` contains the Moderate-band
routing, independent reviews, focused and full Jest verification, coverage
certification, and child-level owner verification.

## P1.A1d — Evidence and parent-closure record

### Pre-task record

- **RRI:** 10 Low. Recomputed with
  `python3 scripts/rri.py --platform rn --cc 1 --touches docs/audit/mvp0-p2p-p1-a1-implementation.md --touches docs/tasks/mvp0-p2p-p1-replication.md --touches docs/plan/mvp0-p2p-p1-replication.md --touches docs/plan/roadmap.md --D 0 --K 1 --P 0 --T 0 --A 0 --X 2`.
- **Route:** direct primary-agent documentation/status synchronization. Local
  Qwen delegation is inapplicable to documentation and task-ledger work.
- **Task-analysis review:** n/a — documentation/task-ledger/plan-only task.
- **Code-solution review:** n/a — documentation/task-ledger/plan-only task.

### Focused verification

Run on 2026-08-30:

```text
cd mobile && npm test -- --runInBand --coverage __tests__/p2p/runtime-protocol.test.ts
PASS __tests__/p2p/runtime-protocol.test.ts
Test Suites: 1 passed, 1 total
Tests:       20 passed, 20 total
```

Focused coverage: `runtime/` 90.74% lines; `protocol.ts` 91.66%, `worklet.ts`
89.88%, and generated `worklet.bundle.js` 100%. The `proof/` aggregate is not
the P1.A1c source surface; its 70.58% line figure includes unrelated factory
branches not exercised by this focused protocol/worklet test.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-A1 | Happy path | The host passes only its derived proof URI in the worklet arguments. | `runtime-protocol.test.ts::HP-A1 passes only the host-derived proof URI as the worklet argument` | passed |
| HP-A1 | Happy path | A ready then close sequence returns only the frozen two-field receipt. | `runtime-protocol.test.ts::HP-A1 preserves the two-field receipt after ready then close` | passed |
| EC-A1 | Edge case | Invalid bootstrap configuration is rejected before storage load or network activity. | `runtime-protocol.test.ts::EC-A1b rejects invalid proof configuration before storage is required` | passed |
| EC-A1 | Edge case | Dependency-load and malformed-bundle failures are typed and redacted. | `runtime-protocol.test.ts::EC-A1 returns a redacted typed error for dependency load failure`; `… bundle validation failure` | passed |
| EC-A1 | Edge case | Open/ready and partial-construction cleanup failures are typed and redacted. | `runtime-protocol.test.ts::EC-A1 returns a redacted typed error for open failure`; `… partial close failure` | passed |
| EC-A1 | Edge case | A drive-close failure is typed/redacted and does not directly close the drive-owned Corestore. | `runtime-protocol.test.ts::EC-A1 returns a close error without directly closing a drive-owned Corestore` | passed |

### Parent-chain disposition

P1.A1a, P1.A1b.0, P1.A1b, and P1.A1c are PASS with the linked child evidence
above. P1.A1d has no source or test changes and independently re-ran the
focused HP-A1/EC-A1 suite. The repository owner explicitly approved this
parent-chain certification in this session; P1.A1 is PASS and P1.A2 may now be
prepared for its own current RRI, card, and approval gate.

### Owner final verification

- **Owner:** Matias, repository owner.
- **Date:** 2026-08-30.
- **Statement:** Matias approved the presented P1.A1a-d certification in this
  session, authorizing P1.A1 PASS and P1.A2's progression to its own planning
  and approval gate.
- **Commands run:** `cd mobile && npm test -- --runInBand --coverage
  __tests__/p2p/runtime-protocol.test.ts`; `make qa-docs`; `git diff --check`.

**Status: P1.A1d — PASS / Done 2026-08-30.**

**Status: P1.A1 — PASS / Done 2026-08-30 (bundle/isolated-test scope; X28
remains an Android-runtime environment blocker).**
