---
type: Audit
title: "MVP0-P2P P1 isolated replication proof RRI"
task: P1
date: 2026-08-27
---

# MVP0-P2P P1 — Revised RRI assessment

The original P1 score of 57 is superseded. The maintainability review expands
P1 from an isolated replication spike into an architecture foundation plus the
proof, and explicitly scores its architecture decision and mixed
refactor/behavior surface.

## Command

```text
python3 scripts/rri.py --platform rn --cc 10 --touches mobile/package.json --touches mobile/package-lock.json --touches mobile/app.config.ts --touches mobile/App.tsx --touches mobile/src/navigation/RootNavigator.tsx --touches mobile/src/p2p/AndroidBareRuntimeProbe.tsx --touches mobile/src/p2p/bare-bridge.ts --touches mobile/src/p2p/bare-protocol.ts --touches mobile/src/p2p/bare-worklet.ts --touches mobile/src/p2p/runtime/BareRuntimeClient.ts --touches mobile/src/p2p/runtime/protocol.ts --touches mobile/src/p2p/runtime/worklet.ts --touches mobile/src/p2p/runtime/worklet.bundle.js --touches mobile/src/p2p/P2PService.ts --touches mobile/src/p2p/P2PProvider.tsx --touches mobile/src/p2p/development/P2PDevelopmentHarness.tsx --touches mobile/src/p2p/proof/P1SeedProofRunner.ts --touches mobile/src/p2p/proof/P1ReplicationProofRunner.ts --touches mobile/src/p2p/proof/transient-storage.ts --touches mobile/scripts/build-bare-worklet.mjs --touches mobile/__tests__/p2p/bare-bridge.test.ts --touches mobile/__tests__/p2p/runtime-protocol.test.ts --touches mobile/__tests__/p2p/service.test.ts --touches mobile/__tests__/p2p/provider.test.ts --touches mobile/__tests__/p2p/p0-migration.test.ts --touches mobile/__tests__/p2p/hyperdrive-smoke.test.ts --touches mobile/__tests__/p2p/transient-seed.test.ts --touches mobile/__tests__/p2p/hyperswarm-replication.test.ts --touches mobile/__tests__/p2p/verification-witness.test.ts --D 4 --K 5 --P 3 --T 4 --A 1 --X 5 --penalty refactor_and_behavior --penalty arch_decision
```

## Result

The following is the unmodified Markdown output emitted by `scripts/rri.py`:

**Platform:** rn

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 1 | raw CC 10 -> score 1 (policy CC table) | High |
| F files | 5 | --touches -> 29 files | High |
| D domain | 4 | agent-supplied (no rubric match) | High |
| T coverage | 4 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 5 | agent-supplied (no rubric match) | High |
| P impact | 3 | agent-supplied (no rubric match) | High |
| X context | 5 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 66
**Penalties applied:** arch_decision (+12, manual flag); many_files (+8, F=5 >= 4); refactor_and_behavior (+8, manual flag)
**Final RRI:** 94 -> band Very high (86-100) -> Effort XL . Codex Premium . Claude Premium . thinking On
**Gates for this band:** Do not implement directly. Produce an ADR + risk analysis + decompose into subtasks.
**Decomposition:** triggered by RRI >= 56, F >= 4 and K >= 3, refactor + behavior (+8) active — split before implementing
**Advisory:** mobile/package.json: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/package-lock.json: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/app.config.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/App.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/navigation/RootNavigator.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/AndroidBareRuntimeProbe.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/bare-bridge.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/bare-protocol.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/bare-worklet.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/BareRuntimeClient.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/protocol.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/worklet.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/runtime/worklet.bundle.js: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/P2PService.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/P2PProvider.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/development/P2PDevelopmentHarness.tsx: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/proof/P1SeedProofRunner.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/proof/P1ReplicationProofRunner.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/src/p2p/proof/transient-storage.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/scripts/build-bare-worklet.mjs: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/bare-bridge.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/runtime-protocol.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/service.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/provider.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/p0-migration.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/hyperdrive-smoke.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/transient-seed.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/hyperswarm-replication.test.ts: no anchor-rubric match — agent judgment governs D/P/K
**Advisory:** mobile/__tests__/p2p/verification-witness.test.ts: no anchor-rubric match — agent judgment governs D/P/K

## Required route

- ADR-043 and its risk analysis were accepted with the revised P1 parent on
  2026-08-27; this parent gate is satisfied.
- P1 is a planning parent only and decomposes into P1.F1, P1.F2, P1.F3a,
  P1.F3b, P1.A1, P1.A2, P1.B1, and P1.B2.
- Each child is recomputed, carded, and explicitly approved before source work.
- The external taskpack's `gpt-5.6-terra` / high declaration remains input; it
  cannot bypass the repository's Very-high gate or child-specific routing.
