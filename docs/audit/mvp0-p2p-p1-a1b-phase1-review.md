---
type: Audit
title: "MVP0-P2P P1.A1b phase-1 task-analysis review"
task: P1.A1b
phase: task-analysis
reviewer: gemma4:26b-a4b-it-qat
verdict: BLOCKED
date: 2026-08-30
---

# P1.A1b — Phase-1 task-analysis review

Task-analysis review: gemma
`docs/audit/mvp0-p2p-p1-a1b-phase1-review.md` - BLOCKED

Gemma ran two usable passes, both returning `FINDINGS` with the same blocking
conclusion: P1.A1b is not sufficiently specified for a Med-high approval card.
The local result receipts are `scratchpad/p1-a1b-phase1-result.pass1.json` and
`scratchpad/p1-a1b-phase1-result.pass2.json`.

## Blocking finding

The task does not freeze the new RPC command or its receipt schema. It must
define the method name, request shape, and the exact structure of the only
permitted successful result: `capability` and `schema_version`.

## Required corrections before a new presentation packet

1. Define the trusted source and lexical/realpath validation rule for the
   proof-cache root, including how an out-of-root path is rejected.
2. Define the Corestore storage adapter/import contract available inside the
   packaged Bare worklet.
3. Define resource order and failure handling: Corestore/Hyperdrive readiness,
   `drive.close()`/`store.close()` order, and typed close-failure result.
4. Extend the allowed paths to include the focused protocol/worklet test and
   the generated bundle, then add HP-A1 and invalid/out-of-root EC coverage.

## RRI impact

The executable, testable minimum surface scores RRI 53 Med-high, not the
ledger's historical RRI 25 Low. See
`docs/audit/mvp0-p2p-p1-a1b-rri.md` for the unmodified calculator output.

## Local-stack precheck

- No other local-model runner was active.
- Ollama was restarted for P1.A1b: PID changed from `84233` to `82108`; the
  listener on `127.0.0.1:11434` was confirmed.
- Gemma completed a JSON-only warm test at `num_ctx=131072`,
  `num_predict=8192`, `think=false`, with non-empty content and
  `done_reason=stop`.
- Muse Glimmer returned empty content at `num_ctx=65536`, then again after the
  required reduced-profile retry at `num_ctx=16384`, `num_predict=512`,
  `think=false`, `temperature=0`; it is not a usable fallback for this task.
- Gemma supplied a usable non-pass verdict, so the Muse/D14 fallback chain was
  not invoked.
