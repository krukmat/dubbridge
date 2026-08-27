---
type: Audit
title: "MVP0-P2P P1.F1 reproducible worklet and versioned RPC RRI"
task: P1.F1
date: 2026-08-27
---

# MVP0-P2P P1.F1 — RRI assessment

## Command

```text
python3 scripts/rri.py --platform rn --cc 10 --touches mobile/package.json --touches mobile/package-lock.json --touches mobile/scripts/build-bare-worklet.mjs --touches mobile/src/p2p/runtime/protocol.ts --touches mobile/src/p2p/runtime/worklet.ts --touches mobile/src/p2p/runtime/worklet.bundle.js --touches mobile/__tests__/p2p/runtime-protocol.test.ts --D 4 --K 4 --P 3 --T 4 --A 0 --X 3
```

## Result

The following is the unmodified Markdown output emitted by `scripts/rri.py`:

**Platform:** rn

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 10 -> score 1 (policy CC table) | High |
| F files | 3 | --touches -> 7 files | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 4 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 4 | agent-supplied (no rubric match) | High |
| P impact | 3 | agent-supplied (no rubric match) | High |
| X context | 3 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 54
**Penalties applied:** none
**Final RRI:** 54 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered
**Advisory:** mobile/package.json: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/package-lock.json: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/scripts/build-bare-worklet.mjs: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/protocol.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/worklet.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/worklet.bundle.js: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/runtime-protocol.test.ts: no anchor-rubric match — agent judgment governs D/P/K

## Scoring notes

- **C1:** estimated raw CC 10 for the bounded handshake, validators, lifecycle
  messages, timeout/close handling, and fatal receipt path. Recompute after the
  implementation diff exists.
- **D4/K4:** Android Bare packaging, asynchronous IPC, generated-bundle drift,
  and lifecycle events form a platform-specific, coupled runtime boundary.
- **T4:** P0 tests characterize the old bridge, but no focused test exists yet
  for the new bundle pipeline or versioned `bare-rpc` contract.
- **P3:** the task creates an internal host/worklet API; it does not expose a
  product/public API, network, authentication, or persisted product data.
- **A0/X3:** HP-F1, EC-F1, exact paths, exclusions, and deterministic checks are
  frozen within one complete runtime module.
- No `refactor_and_behavior` penalty applies inside F1: it adds the replacement
  seam alongside the unchanged P0 characterization scaffold. Migration and
  deletion of that scaffold are separately gated in P1.F3a/F3b.

## Required route

- Current-session approval is required before source edits.
- RRI 54 follows ADR-038: after approval, run the Muse Glimmer advisory
  refinement and emit the primary hash-bound route receipt. Because 54 is in
  the 46–55 sub-band, `GO_LOCAL` does not launch a local developer; the receipt
  selects the card-bound cloud packet.
- Three complete Draft → Critique → Revise Reflection passes, focused tests,
  typecheck, lint, full Jest, coverage certification, owner verification, and
  status synchronization remain mandatory.
- Phase-1 and phase-2 peer review are waived only by the owner-directed
  MVP0-P2P `REVIEW-OVERRIDE`; all other controls remain active.

## Antares refinement touchpoint

`ANTARES-SKIP`: no task-relevant CWE hypothesis exists on the T3a watchlist.
F1 changes a local packaging/protocol/lifecycle seam and introduces no auth,
secret, authorization, SQL, API-handler, or `crates/storage` contract.
