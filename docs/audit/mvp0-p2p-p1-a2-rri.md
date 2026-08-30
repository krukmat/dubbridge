---
type: Audit
title: "P1.A2 current Required Reasoning Index"
task: P1.A2
status: presentation_ready
date: 2026-08-30
---

# P1.A2 — Current RRI

Scope frozen for presentation: the existing worklet/protocol/factory seam,
its regenerated bundle, two proof-only source modules, and one focused test.
This child does not include Hyperswarm discovery, client replication, product
`P2PService` changes, identity, or persistent storage.

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 1 | raw CC 10 -> score 1 (policy CC table) | High |
| F files | 3 | seven presentation-time paths | High |
| D domain | 4 | Android/Bare platform-specific async filesystem lifecycle | High |
| T coverage | 2 | protocol coverage exists; seed/janitor lifecycle requires new focused coverage | High |
| A ambiguity | 0 | task ledger specifies objective, HP-A2, EC-A2, acceptance, and exclusions | High |
| K coupling | 4 | worklet RPC, Hyperdrive/Corestore handles, Expo cache ownership, and crash cleanup | High |
| P impact | 2 | internal proof-only storage behavior; no public/product data API | High |
| X context | 3 | runtime protocol/worklet plus proof factory/storage/runner and focused test module | High |

**Base value:** 100 x (weighted / 5) = 46

**Penalties applied:** none. This child changes behavior but does not refactor
an existing architectural boundary, introduce an ADR decision, lack a
verification strategy, or cross the automatic penalty thresholds.

**Final RRI:** 46 -> Med-high (41–55) -> Effort L; Codex Balanced -> Premium;
Claude Code Balanced -> Premium; thinking On.

**Gates:** plan and explicit acceptance criteria before approval; phase-1
Gemma review; explicit owner approval; ADR-038 refinement/receipt before
implementation; three Reflection passes; phase-2 Gemma review; unit coverage
certification and owner final verification. RRI 46 is cloud-only for the
whole task after ADR-038, unless an approved ADR-040 module split qualifies.

## Canonical invocation

```bash
python3 scripts/rri.py \
  --touches mobile/src/p2p/runtime/worklet.ts \
  --touches mobile/src/p2p/runtime/protocol.ts \
  --touches mobile/src/p2p/runtime/worklet.bundle.js \
  --touches mobile/src/p2p/proof/P1ProofRuntimeFactory.ts \
  --touches mobile/src/p2p/proof/transient-storage.ts \
  --touches mobile/src/p2p/proof/P1SeedProofRunner.ts \
  --touches mobile/__tests__/p2p/transient-seed.test.ts \
  --cc 10 --D 4 --K 4 --P 2 --T 2 --A 0 --X 3
```
