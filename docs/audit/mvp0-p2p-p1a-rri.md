---
type: Audit
title: "MVP0-P2P P1.A seed boundary RRI"
task: P1.A
date: 2026-08-27
---

# MVP0-P2P P1.A — RRI assessment

> **SUPERSEDED — HISTORICAL ONLY.** This score described the abandoned combined
> P1.A plan and carries no implementation authority. The maintainability replan
> is scored in `docs/audit/mvp0-p2p-p1-rri.md` and decomposed under proposed
> ADR-043. Its memory-only Hyperdrive premise was invalid; replacement P1.A1/A2
> use path-backed run-scoped cache storage with verified cleanup.

## Repository finding

The implemented P0 `BareBridge` starts the fixed `BARE_WORKLET_SOURCE` and
parses the closed P0 result union. A P1-specific worklet therefore cannot reuse
that lifecycle/RPC implementation without first extracting a configurable
worklet-source and protocol seam. Combining that boundary refactor with the new
Hyperdrive seed behavior would mix refactor and behavior in one executable
task, so the policy penalty and decomposition gate apply.

## Command

```text
python3 scripts/rri.py --platform rn --cc 12 --touches mobile/package.json --touches mobile/package-lock.json --touches mobile/src/p2p/bare-rpc-bridge.ts --touches mobile/src/p2p/bare-bridge.ts --touches mobile/src/p2p/replication-worklet.ts --touches mobile/src/p2p/replication-bridge.ts --touches mobile/src/p2p/AndroidBareRuntimeProbe.tsx --touches mobile/__tests__/p2p/bare-rpc-bridge.test.ts --touches mobile/__tests__/p2p/bare-bridge.test.ts --touches mobile/__tests__/p2p/replication-bridge.test.ts --D 4 --K 4 --P 2 --T 4 --A 1 --X 4 --penalty refactor_and_behavior
```

## Result

| Variable | Score | Basis |
|---|---:|---|
| C — cyclomatic | 2 | Estimated raw CC 12 across generic RPC cleanup/validation plus seed-state branching. |
| F — files | 3 | Ten anticipated dependency, boundary, seed, probe, and test paths remain in score 3. |
| D — domain | 4 | Android Bare worklet bundling plus asynchronous lifecycle behavior. |
| T — test risk | 4 | P0 lifecycle tests exist, but no Hyperdrive or configurable-protocol coverage exists. |
| A — ambiguity | 1 | HP-A1/HP-A2 and EC-A1/EC-A2 freeze the no-network seed boundary. |
| K — coupling | 4 | P0 wrapper compatibility, generic RPC, bundled worklet dependencies, and Android proof interact. |
| P — impact | 2 | Opt-in feasibility path only; normal UI and product data plane remain unchanged. |
| X — context | 4 | P0 protocol/bridge, P1 dependencies/worklet/bridge, probe, and tests are all required. |

**Base RRI:** 59.

**Penalty:** `refactor_and_behavior` +8.

**Final RRI:** **67 — Complex (56–70), Effort L**. Mandatory plan approval
and decomposition apply; no P1.A source edit is authorized by this parent.

## Required decomposition

- **P1.A1 — Configurable Bare RPC boundary:** preserve the existing P0 wrapper
  while extracting the minimum reusable source/protocol/lifecycle seam.
- **P1.A2 — Ephemeral Hyperdrive seed and Android bundle proof:** add compatible
  dependencies and the no-network seed behavior only after P1.A1 passes.

Prospective script runs place P1.A1 at RRI 42 and P1.A2 at RRI 49, both below
the split target of 55 and with ambiguity 1. Those scores are planning evidence,
not executable authorization; each child receives its own current RRI artifact,
card, and approval immediately before implementation.

## Gates

- P1.A parent approval in the current session.
- P1.A1 then P1.A2, each independently scored, carded, and approved.
- Owner-directed `REVIEW-OVERRIDE` for phases 1 and 2 only; all other gates
  remain mandatory.
- Four P1.A parent Reflection passes after both children, plus child-specific
  passes, tests, Android proof, coverage certification, owner verification, and
  status synchronization.
- `human-select` and a valid ADR-039 `fallback-selection-v1` receipt before any
  terminal D14/cloud fallback that is not already the approved primary route.

## Antares refinement touchpoint

`ANTARES-SKIP`: no task-relevant CWE hypothesis exists on the T3a watchlist.
P1.A changes a local process/lifecycle and bundling boundary; it does not add an
authentication, secret, authorization, or externally trusted-input contract.
