---
type: Audit
title: "MVP0-P2P P1.F3a.2 Low-band decomposition"
task: P1.F3a.2
date: 2026-08-27
status: closed_pass
---

# MVP0-P2P P1.F3a.2 — Low-band decomposition

## Why decomposed

The owner requested implementing P1.F3a.2 via a cloud model (Codex
`gpt-5.6-terra`/medium) instead of the approved local-first Moderate route.
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` only authorizes that cloud model as a
takeover for RRI 26-40 on one of three conditions: local runner/model
unavailable, scope-enforcement failure, or 2/2 local repair-attempt
exhaustion — none of which had occurred. Rather than force an unauthorized
route or invoke a bounded owner waiver, the owner chose the policy-preferred
alternative: lower the task's complexity by splitting it into Low-band (RRI
0-25) subtasks per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Post-repair-
budget Low-band decomposition (applied here proactively, before any local
repair attempt, at the owner's explicit direction — not after budget
exhaustion). The orchestrator (Claude Sonnet 5) remains diagnosis/split/
dispatch/review-only; no subtask author's own logic is written directly by
the orchestrator except under the two narrow documented exceptions (tooling
failure, mechanical lint-driven refactor).

Full P1.F3a.2 task definition, acceptance criteria, and dependency-isolation
verification: `docs/tasks/mvp0-p2p-p1-replication.md` § P1.F3a.2 and
`docs/audit/mvp0-p2p-p1-f3a2-rri.md` (whole-task RRI 29 Moderate — retained as
the parent record; superseded for implementation-routing purposes only by
this decomposition).

## Subtask map

| ID | Scope | RRI | Status |
|---|---|---|---|
| F3a.2-i | Delete `mobile/src/p2p/bare-worklet.ts` + `mobile/src/p2p/bare-protocol.ts` | 13 Low | Done |
| F3a.2-ii | Delete `mobile/src/p2p/bare-bridge.ts` (depends on F3a.2-i) | 10 Low | Done |
| F3a.2-iii | Delete `mobile/src/p2p/AndroidBareRuntimeProbe.tsx`, `mobile/__tests__/p2p/bare-bridge.test.ts`, `mobile/__tests__/p2p/p2p-provider.test.tsx` (depends on F3a.2-ii) | 15 Low | Done |
| F3a.2-iv | Physical Android ping proof on the retained `P2PDevelopmentHarness` after deletion (depends on F3a.2-iii) | 0 Low | Deferred to `X28` — owner-directed, moved to a future general verification pass |

Each RRI computed via `scripts/rri.py --touches <path> --cc 1 --D <n> --K <n>
--P <n> --T 1 --A 0 --X 0`, reflecting deletion-only scope, pre-verified zero
external importers, and concretely defined acceptance criteria (A=0).

## Routing per subtask (RRI 0-25)

Per policy: no full approval card; default to local Qwen Developer delegation
for simple, narrow, mechanical patches — which file deletion with no new
logic qualifies as. Route: `scripts/delegate-low-rri.py`, orchestrator
validates diff scope, runs verification, reviews before/after Gemma Reviewer
pass, records evidence in this file's subtask log below.

## Subtask execution log

### F3a.2-i — delete bare-worklet.ts + bare-protocol.ts

- Ollama per-task restart: PID 93632 -> 84070 (confirmed new PID + listening
  port). Warm-up probe at production profile (`num_ctx=4096`) returned
  `done_reason: "length"` with empty `content` (all output routed to
  `thinking`) — a capacity symptom under host memory pressure (`vm_stat`:
  ~75MB free with the 18.4GB model already resident). Resource-recovery
  protocol step 2 applied: retried at `think=false`, `temperature=0`,
  `num_ctx=8192`, `num_predict=1024`; retry succeeded (`done_reason: "stop"`,
  non-empty content). Real packet rebuilt at this reduced profile.
- Delegated via `scripts/delegate-low-rri.py` to `qwen3.8:27b-mlx`,
  `--allow-path mobile/src/p2p/bare-worklet.ts --allow-path
  mobile/src/p2p/bare-protocol.ts`.
- Response: `status: patch`, both files `action: delete` with empty content —
  exactly in scope, no unrequested changes.
- Applied via `--apply`. `git status` confirms only the two target files
  deleted.
- `npm run typecheck`: fails with 3 `TS2307` errors in `bare-bridge.ts`
  (still imports the now-deleted `bare-worklet`/`bare-protocol`) — expected
  and correct at this intermediate point; `bare-bridge.ts` is retired next in
  F3a.2-ii. No error outside `bare-bridge.ts` — confirms no out-of-scope
  breakage.
- Status: **applied, intermediate repo state intentionally broken pending
  F3a.2-ii; do not stop here.**

### F3a.2-ii — delete bare-bridge.ts

- Reused the F3a.2-i Ollama restart boundary (same task, same repository
  task ID). Delegated directly at the already-validated reduced profile
  (`num_ctx=8192`, `num_predict=1024`, `temperature=0`, `think=false`).
- Response: `status: patch`, single file `mobile/src/p2p/bare-bridge.ts`,
  `action: delete`, empty content — exactly in scope.
- Applied via `--apply`. `git status` shows exactly the 3 files deleted so
  far (worklet, protocol, bridge) — no unrequested changes.
- `npm run typecheck`: fails with a single `TS2307` error in
  `__tests__/p2p/bare-bridge.test.ts` (still imports the now-deleted
  `bare-bridge`) — expected; that test file is retired next in F3a.2-iii.
  Verified `AndroidBareRuntimeProbe.tsx` does not itself import
  `bare-bridge` (no match on direct grep) — the earlier broad-glob search
  that listed it as an "importer" was a false positive from the search
  pattern, not an actual import; corrected here.
- Status: **applied, intermediate repo state intentionally broken pending
  F3a.2-iii; do not stop here.**

### F3a.2-iii — delete AndroidBareRuntimeProbe.tsx + bare-bridge.test.ts + p2p-provider.test.tsx

- Reused the same Ollama restart boundary (PID 84070, confirmed still
  listening on `:11434` before this call — same repository task ID, no new
  restart required). Delegated directly at the already-validated reduced
  profile (`num_ctx=8192`, `num_predict=1024`, `temperature=0`,
  `think=false`).
- Response: `status: patch`, three files — `AndroidBareRuntimeProbe.tsx`,
  `bare-bridge.test.ts`, `p2p-provider.test.tsx` — all `action: delete`,
  empty content, exactly matching the packet's allowed paths.
- Applied via `--apply`. `git status` confirms exactly 6 files deleted across
  all three subtasks combined (worklet, protocol, bridge, probe, and the two
  P0 tests) — no unrequested changes at any point in the chain.
- `npm run typecheck`: clean, no errors. `npm run lint`: clean, no warnings.
  `npx jest --runInBand`: **24/24 suites, 262/262 tests passing**, including
  the ADR-043 boundary suite (`__tests__/p2p/`: `runtime-protocol.test.ts`,
  `p2p-service.test.ts`, `p2p-development-harness.test.ts` — 3 suites, 27
  tests, all passing) with zero references to any retired P0 file remaining.
- Status: **applied and fully verified — repo state clean, no P0 scaffold
  remnants.**

### F3a.2-iv — physical Android ping proof (deferred to X28)

Not a code-delegation subtask: RRI 0, no source change, no local-model
routing applies. The ledger's EC-F3a.2 requires "a new physical Android ping
proof" after deletion — distinct from the Jest characterization suite, which
only proves the harness's logic in a simulated environment. F3a.1's own
audit record (`docs/audit/mvp0-p2p-p1-f3a1-implementation.md` line 23)
explicitly states no Android-native proof was performed there either — this
is the first genuine hardware step in the P1.F3a chain, and this agent
session has no physical Android device or emulator access to perform it.

**Owner decision (2026-08-27):** deferred to a future general verification
pass rather than performed now, tracked as `docs/plan/roadmap.md` §
Cross-cutting obligations `X28`. This intentionally leaves P1.F3a.2's
device-proof criterion (part of EC-F3a.2/HP-F3a.2) open while every other
acceptance criterion is closed — see § Closure status below.

**Handoff, to run whenever the general verification pass happens:**

1. Build and install the mobile app on a physical Android device (or an
   emulator with Bare/worklet native module support, if the harness's
   underlying runtime works there — confirm before relying on it as a
   substitute for physical hardware).
2. Mount the app with the `P2PDevelopmentHarness`'s explicit development
   flag enabled (same flag gated in
   `__tests__/p2p/p2p-development-harness.test.ts`'s "initializes, pings,
   and shuts down" case).
3. Observe a real `initialize → ping → shutdown` cycle complete
   successfully end-to-end on-device (matching HP-F3a.2), with no crash, no
   unhandled exception, and no leaked worklet process after shutdown.
4. Record the device model/OS version, exact steps taken, and the observed
   result (pass/fail with logs) back into this file's subtask table and this
   section, then close `X28` in `docs/plan/roadmap.md`.

## Closure status

- F3a.2-i, F3a.2-ii, F3a.2-iii: **Done**, fully verified (typecheck, lint,
  262/262 Jest tests passing including the 27/27 ADR-043 boundary suite).
- F3a.2-iv: **deferred to X28**, not blocking this decomposition's other
  subtasks or the source-level parts of P1.F3a.2's acceptance criteria.
  P1.F3a.2's own closure record must still state explicitly that its
  device-proof criterion remains open under X28 — it is not silently
  dropped, only rescheduled.
