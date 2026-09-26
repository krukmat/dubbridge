---
type: Audit
title: "MVP0-P2P P1.F3a.2 RRI computation"
task: P1.F3a.2
date: 2026-08-27
status: computed
---

# MVP0-P2P P1.F3a.2 — RRI computation

Task-presentation time (no diff yet). Command and full output:

```
python3 scripts/rri.py \
  --touches mobile/src/p2p/AndroidBareRuntimeProbe.tsx \
  --touches mobile/src/p2p/bare-bridge.ts \
  --touches mobile/src/p2p/bare-protocol.ts \
  --touches mobile/src/p2p/bare-worklet.ts \
  --touches mobile/__tests__/p2p/bare-bridge.test.ts \
  --touches mobile/__tests__/p2p/p2p-provider.test.tsx \
  --cc 1 --D 2 --K 2 --P 2 --T 1 --A 1 --X 1
```

| Variable | Score | Evidence |
|---|---|---|
| C cyclomatic | 0 | raw CC 1 (deletion-only, no new branching) -> score 0 |
| F files | 3 | 6 files touched (4 P0 source + 2 P0 test, all scheduled for deletion) |
| D domain | 2 | no anchor-rubric match for `mobile/src/p2p/**`; judged analogous to "internal crate business logic" floor (2) — no auth/secrets/rights-ledger/migration surface |
| T coverage | 1 | retained harness (F3a.1) already has 100% line/function/statement coverage; this task adds a physical Android proof, not new logic needing new coverage |
| A ambiguity | 1 | acceptance criteria and HP-F3a.2/EC-F3a.2 are concretely defined in the task ledger |
| K coupling | 2 | verified zero external importers of the 4 P0 source files outside themselves and their own tests (see dependency trace below); same floor rationale as D |
| P impact | 2 | no public API, no auth/security/rights impact; affects only an internal dev-only diagnostic path |
| X context | 1 | single mobile app, no cross-service/cross-repo coordination |

**Final RRI: 29 -> Moderate (26-40).** No penalties (pure deletion, not a
combined refactor+behavior-change; clear verification strategy exists via the
retained harness tests plus a new physical Android proof).

## Independent dependency verification (pre-scoring)

Before scoring, confirmed no file outside the six declared paths imports any of
the four P0 source modules:

```
grep -rln "AndroidBareRuntimeProbe\|bare-bridge\|bare-worklet\|bare-protocol" \
  --include="*.ts" --include="*.tsx" . | grep -v node_modules
```

Result: only the four P0 source files (which import each other) and their two
own test files. `mobile/App.tsx` mounts `P2PDevelopmentHarness` and
`P2PProvider` only; `P2PProvider` -> `P2PService` -> `BareRuntimeClient`
(ADR-043 boundary) has no reference to any P0 module.
`runtime-protocol.test.ts` references `scripts/build-bare-worklet.mjs` (an
ADR-043 build script), not the P0 `bare-worklet.ts` file — confirmed by exact
line inspection, not name similarity alone.

This confirms the F3a.2 deletion scope is isolated: no production or
ADR-043-path code depends on the retiring files.
