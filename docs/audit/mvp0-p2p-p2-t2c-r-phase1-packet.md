---
type: Audit
title: "MVP0-P2P P2.T2c-r phase-1 task-analysis packet"
task: P2.T2c-r
phase: task-analysis
status: complete
date: 2026-09-08
---

# P2.T2c-r — proposed approval envelope

## Decision

`P2.T2c-r — fail-closed nonce-collision repair | AWAITING APPROVAL | RRI 55 Med-high | Effort L | current-session parent approval required`

- Parent RRI command: `python3 scripts/rri.py --touches
  crates/p2p/src/crypto.rs --touches crates/p2p/src/nonce_tracker.rs
  --touches crates/p2p/src/lib.rs --touches
  crates/p2p/src/package_builder.rs --cc 8 --D 2 --K 2 --P 2 --T 0 --A 1
  --X 0` -> RRI 55 Med-high / Effort L, no penalties.
- The parent is a review/approval/integrated-closure envelope, not a single
  executable patch. Its six frozen executable leaves each score RRI 25 Low /
  Effort S after the 2026-09-08 ADR-045 correction.
- Antares touchpoint: typed skip. The task has no hypothesis matching the
  current static watchlist (`CWE-89`, `CWE-306`, `CWE-22`) or its repository
  boundaries.

## Scope and acceptance

- Objective: reject any duplicate 96-bit nonce before encrypting the
  corresponding file and return no `SealedPackage` for the failed lineage
  build, while preserving the public CSPRNG behavior and API.
- Allowed production paths: `crates/p2p/src/crypto.rs`,
  `crates/p2p/src/nonce_tracker.rs`, `crates/p2p/src/lib.rs`, and
  `crates/p2p/src/package_builder.rs`.
- Ordered leaves: `T2c-r1a -> T2c-r1b -> (T2c-r2 + T2c-r3a) -> T2c-r3b ->
  T2c-r3c`. Each leaf is independently compilable/verifiable and receives its
  own Low-band delegation packet plus phase-1 review before Qwen authors it.
- In scope: an internal caller-assigned-nonce encryption primitive; routing
  the unchanged public `encrypt_file` CSPRNG entry through it; a pure
  per-build nonce tracker and module export; a private builder nonce-source
  seam; tracker-before-encryption wiring and a typed collision error; and
  deterministic full-build collision tests.
- Out of scope: public API changes, nonce persistence, cross-lineage or
  cross-retry bookkeeping, CK/KEK changes, database/storage/network changes,
  unrelated refactors, and changes to existing untracked T2g contract tests
  except their final rerun/status synchronization.
- Happy-path acceptance: fixed nonce + CK + canonical AAD round-trips through
  the internal primitive; the public entry retains fresh CSPRNG behavior; two
  distinct assigned nonces register and a distinct deterministic sequence
  produces a decryptable multi-file package with distinct manifest nonces.
- Edge acceptance: tampered AAD fails authentication; duplicate tracker
  registration returns the typed collision result; a forced duplicate in the
  real multi-file build returns `PackageBuildError::NonceCollision` before
  that encryption and produces no `SealedPackage`.
- Evidence: focused unit/component RED-GREEN evidence where applicable;
  `cargo fmt --check`; `cargo test -p dubbridge-p2p --all-targets`; focused
  T2g contract rerun after `T2c-r3c`; clippy for the affected crate; per-leaf
  review/delegation artifacts; three integrated Reflection passes; parent
  phase-2 review; behavioral coverage certification and owner verification.
- Status sync: P2 task ledger, P2 plan, roadmap, C0 audit dependency/status,
  and the blocked T2g certification record.

## Routing

- Orchestrator: Codex. Claude is not selected and must not be invoked for this
  task, honoring the owner's recorded capacity constraint.
- Phase 1/phase 2 parent reviewer: `gemma4:26b-a4b-it-qat`, fallback
  `gpt-oss:20b`, then ADR-039-selected D14 only if both are unusable.
- After approval: obtain the required ADR-038 advisory refinement and
  hash-bound primary receipt. The already-frozen Amendment-4 decomposition
  contains no known above-Low residue, so implementation proceeds through six
  bounded `scripts/delegate-low-rri.py` packets using
  `qwen3.8:27b-mlx`, in dependency order.
- Each materially changed delegation packet returns to Low-band phase 1.
  Each leaf has at most one bounded repair. Any discovered inseparable
  above-Low residue stops, is rescored, and follows the parent/cloud gate.
- Cloud takeover is not preauthorized. `human-select` applies at the exact
  fallback packet: operational-only -> `gpt-5.6-terra` / high;
  capability/risk -> `gpt-5.6-sol` / high. Claude's nominal Med-high mapping
  is `claude-sonnet-5` with thinking, escalating to `claude-opus-5` only on
  bounded stall/failure, but it is not selected or authorized here.

## Workflow and closure gates

1. Codex freezes scope/RRI and obtains a Gemma phase-1 PASS.
2. The owner approves this single parent envelope; no leaf-level approval is
   then required while paths, dependencies, and invariants remain frozen.
3. The ADR-038 refinement/receipt runs without expanding scope.
4. Codex orchestrates the six Low leaves; Qwen authors only inside each
   packet's exact allowed paths.
5. Codex runs three complete Draft -> Critique -> Revise Reflection passes:
   API/crypto preservation; fail-closed ordering/no partial result; integrated
   regression and T2g evidence.
6. Gemma performs the parent code-solution review; fallback chain is
   `gpt-oss:20b -> D14`.
7. Codex records behavioral evidence, obtains owner final verification, and
   synchronizes all status artifacts before unblocking T2g.

## Governing references

- `docs/tasks/mvp0-p2p-p2-encrypted-publication.md` § P2.T2c-r.
- `docs/plan/mvp0-p2p-p2-encrypted-publication.md` § P2.T2.
- `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md` § Nonce allocation and T2
  path matrix.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.
- `docs/policies/HITL_AUTONOMY_POLICY.md`.
- `docs/policies/RRI_POLICY.md` and ADR-045 Low-band correction.
- ADR-038 and ADR-039.
