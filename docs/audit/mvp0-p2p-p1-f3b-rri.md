---
type: Audit
title: "MVP0-P2P P1.F3b Required Reasoning Index"
task: P1.F3b
status: computed
date: 2026-08-27
---

# P1.F3b — Required Reasoning Index

Computed post-implementation with `scripts/rri.py` over the final diff scope.
The ledger's prospective estimate was `M / 30 Moderate`; the measured score is
**24 Low**, because the delivered change carries no new branching, no new
dependency, and no lockfile mutation — the audit concluded in *retain* for
every contested item, so the diff reduced to identifier renames plus one
mechanically regenerated artifact.

## Command

```bash
python3 scripts/rri.py \
  --touches mobile/package.json --touches mobile/app.config.ts \
  --touches mobile/App.tsx --touches mobile/src/p2p/runtime/worklet.bundle.js \
  --cc 1 --D 1 --K 2 --P 2 --T 1 --A 1 --X 2
```

## Report

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 1 -> score 0 (policy CC table) | High |
| F files | 2 | --touches -> 4 files | High |
| D domain | 1 | agent-supplied (no rubric match) | High |
| T coverage | 1 | agent-supplied | High |
| A ambiguity | 1 | agent-supplied | High |
| K coupling | 2 | agent-supplied (no rubric match) | High |
| P impact | 2 | agent-supplied (no rubric match) | High |
| X context | 2 | agent-supplied | High |

**Base value:** 100 x (weighted / 5) = 24
**Penalties applied:** none
**Final RRI:** 24 -> band Low (0-25) -> Effort S
**Gates for this band:** no full approval card required
**Decomposition:** not triggered

**Advisory:** `mobile/package.json`, `mobile/app.config.ts`, `mobile/App.tsx`,
and `mobile/src/p2p/runtime/worklet.bundle.js` have no anchor-rubric match —
agent judgment governs D/P/K.

## Score justification

- **C = 0 (raw CC 1).** The diff introduces no conditional, loop, or early
  return. `package.json` and `app.config.ts` change string/key identifiers;
  `App.tsx` renames one constant and one prop value; `worklet.bundle.js` is
  machine-generated.
- **F = 2 (4 files).** Two of the four are outside the ledger's declared
  `allowed_paths` — see § Scope extensions in
  `docs/audit/mvp0-p2p-p1-f3b-implementation.md`.
- **D = 1.** Mobile build/config domain. Not auth, rights, consent,
  governance, or migration; no ADR-008/ADR-018 invariant is touched.
- **T = 1.** The changed surface is covered by wired gates: the harness wiring
  by `mobile/__tests__/p2p/p2p-development-harness.test.ts`, and the
  regenerated bundle byte-for-byte by
  `mobile/__tests__/p2p/runtime-protocol.test.ts` -> "HP-F1 builds the
  committed worklet bundle deterministically". The residual gap is the
  Android-device criterion, recorded as blocked rather than scored away.
- **A = 1.** The ledger supplies explicit acceptance criteria plus HP-F3b and
  EC-F3b. The single genuine ambiguity was what qualifies as "proof" for a
  native setting, resolved in favour of EC-F3b's explicit rule that a native
  requirement is never removed on static guesswork.
- **K = 2.** `package.json` is the manifest every other module resolves
  through, and `App.tsx` is the composition root.
- **P = 2.** A wrong removal here would have degraded the Bare RPC data path
  app-wide (see the `react-native-b4a` finding).
- **X = 2.** Reaching a defensible verdict required reading `bare-pack`
  bundling behaviour, `b4a`'s export-condition resolution, React Native
  autolinking, and `react-native-bare-kit`'s Gradle addon packaging.

## Penalty assessment

| Penalty | Applied | Reason |
|---|---|---|
| `arch_decision` | no | No ADR is created, amended, or reinterpreted. |
| `auth_security` | no | No auth, credential, secret, or rights surface. |
| `refactor_and_behavior` | no | Rename only; no behaviour restructure lands beside it. |
| `no_verification` | no | Typecheck, lint, and full Jest all pass. The unmet Android criterion is reported as a blocker, not silently absorbed. |

## Related

- `docs/tasks/mvp0-p2p-p1-replication.md` § P1.F3b
- `docs/audit/mvp0-p2p-p1-f3b-implementation.md`
- `docs/audit/mvp0-p2p-review-exception.md`
