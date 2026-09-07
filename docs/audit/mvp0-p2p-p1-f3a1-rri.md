---
type: Audit
title: "MVP0-P2P P1.F3a.1 Required Reasoning Index"
task: P1.F3a.1
status: awaiting_approval
date: 2026-08-27
---

# P1.F3a.1 — Required Reasoning Index

## Frozen scope

- `mobile/App.tsx`
- `mobile/src/p2p/development/P2PDevelopmentHarness.tsx`
- `mobile/__tests__/p2p/p2p-development-harness.test.ts`
- P1.F3a.1 audit evidence only

The legacy P0 probe, custom bridge/protocol, inline worklet, and their existing
tests are read-only characterization oracles in this child. Their retirement,
including imports that only F3a.2 is allowed to remove, is out of scope.

## Scoring basis

- **C1 / raw CC 10:** the new development-only component needs bounded enable,
  lifecycle, release, and typed-error branches, but no protocol implementation.
- **F2:** the three listed source/test files are the entire prospective code
  surface; evidence files do not count as product files.
- **D4 / K4:** Android Bare runtime ownership and its asynchronous lifecycle
  cross React composition, `P2PService`, and `BareRuntimeClient`.
- **T2:** P0 (`bare-bridge.test.ts` / `p2p-provider.test.tsx`) and the new
  runtime have relevant tests, but this new migration harness has no dedicated
  test yet.
- **A0 / P3:** the ledger freezes HP-F3a.1, EC-F3a.1, exact allowed paths, and
  no-network/product-API boundaries; the result changes an internal diagnostic
  API path, not permissions, persistence, or a public product API.
- **X4:** safe implementation requires the App composition, P0 oracle,
  provider/service, and versioned runtime/protocol modules together.
- **Penalties:** none. This child adds a parity harness while retaining P0
  byte-for-byte; the combined refactor-and-behavior penalty belongs to the
  now-separated F3a.2 retirement child, not to this scope.

## Unmodified calculator output

```text
**Platform:** rn

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 10 -> score 1 (policy CC table) | High |
| F files | 2 | --touches -> 3 files | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 2 | agent-supplied | High |
| A ambiguity | 0 | agent-supplied | High |
| K coupling | 4 | agent-supplied (no rubric match) | High |
| P impact | 3 | agent-supplied (no rubric match) | High |
| X context | 4 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 47
**Penalties applied:** none
**Final RRI:** 47 -> band Med-high (41-55) -> Effort L . Codex Balanced -> Premium . Claude Balanced -> Premium . thinking On
**Gates for this band:** Plan + explicit acceptance criteria required before approval.
**Decomposition:** not triggered
**Advisory:** mobile/App.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/development/P2PDevelopmentHarness.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/p2p-development-harness.test.ts: no anchor-rubric match — agent judgment governs D/P/K
```

## Command

```sh
python3 scripts/rri.py --platform rn --cc 10 \
  --touches mobile/App.tsx \
  --touches mobile/src/p2p/development/P2PDevelopmentHarness.tsx \
  --touches mobile/__tests__/p2p/p2p-development-harness.test.ts \
  --D 4 --T 2 --A 0 --K 4 --P 3 --X 4
```

## Required route

P1.F2 is closed PASS. RRI 47 requires a phase-1 Gemma review and explicit
owner approval before source edits. After approval, ADR-038 requires a Muse
Glimmer refinement and a hash-bound primary route receipt. Because 47 is in
the 46–55 sub-band, either advisory outcome yields the approved cloud packet;
three Reflection passes, phase-2 review, coverage certification, owner final
verification, and status synchronization remain mandatory.

## Antares refinement touchpoint

`ANTARES-SKIP`: no task-relevant CWE hypothesis exists on the T3a watchlist.
The task touches Android/React diagnostic runtime composition only; it adds no
SQL, HTTP handler, storage-path, authentication, authorization, secret, or
other watchlisted boundary.
