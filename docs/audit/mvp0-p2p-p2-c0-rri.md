---
type: Audit
title: "P2.C0 shared-contract freeze RRI"
status: complete
slice: MVP0-P2P
parent: P2.C0
---

# P2.C0 — RRI evidence

## Frozen planning envelope

Base SHA at presentation: `df20a4fdd857ff2783e35cdffa02c60f2e5cb218`.

The owner approved a planning/contract-only envelope of exactly ten repository paths:

1. `docs/plan/mvp0-p2p-p2-encrypted-publication.md`
2. `docs/tasks/mvp0-p2p-p2-encrypted-publication.md`
3. `docs/plan/s-230-poc-v1-digitalocean.md`
4. `docs/tasks/s-230-poc-v1-digitalocean.md`
5. `docs/plan/roadmap.md`
6. `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`
7. `docs/fixtures/mvp0-p2p-manifest-v1.json`
8. `docs/fixtures/mvp0-p2p-publication-contract-v1.json`
9. `docs/audit/mvp0-p2p-p2-c0-contract-freeze.md`
10. `docs/audit/mvp0-p2p-p2-c0-rri.md`

No source, migration, configuration, mobile, deployment descriptor, or completed P2.T1 path is writable under C0.

## Deterministic RRI calculation

The exact path list was frozen before scoring, as required by the workflow. The branch copy of `scripts/rri.py` defines the formula, file-count mapping, anchor rubric, penalties, and bands used below.

Presentation inputs:

| Variable | Score | Evidence |
|---|---:|---|
| C | 0 | planning/docs/JSON-fixture task; no executable branch logic |
| F | 3 | 10 frozen paths -> policy file-count band 6–10 |
| D | 4 | cross-runtime crypto/publication/audit contract freeze constraining several later subsystems |
| T | 0 | no source implementation is changed by C0; downstream leaves define their own executable evidence |
| A | 0 | exact objective, contract topics, acceptance, paths, and downstream decomposition were frozen before presentation |
| K | 4 | the contract couples package crypto, Availability Node, O4 recovery, S-120 activation, audit, and S-230 deployment semantics |
| P | 5 | security-sensitive key-custody, mTLS, ciphertext-only, fail-closed readiness, and release-boundary decisions |
| X | 4 | several crates/services and both MVP0-P2P/S-230 planning contexts are required |

Weighted base:

`round(100 * ((0.18*0 + 0.12*3 + 0.15*4 + 0.15*0 + 0.12*0 + 0.12*4 + 0.10*5 + 0.06*4) / 5)) = 44`

Manual penalties required by the actual content, even though the writable paths themselves are documentation:

- `auth_security +10` — C0 freezes cryptographic key custody, mTLS identity, secret boundaries, and fail-closed release/readiness semantics.
- `arch_decision +12` — C0 freezes cross-runtime/publication contracts and downstream ownership/integration boundaries.

**Final RRI: 66 — Complex (56–70) — Effort L.**

The Complex band requires decomposition, a human-reviewed plan, explicit approval before execution, Premium/high reasoning for implementation-shaped work, and four integrated Reflection passes. P2.C0 is docs/contract-only, so local-model/Ollama precheck and phase-1/phase-2 code review are `n/a` under the workflow exemption.

## Execution note

This session used the connected GitHub source as the repository authority and did not have an executable checkout of `/Users/matias/dubbridge`. Therefore the numeric result above is an exact deterministic evaluation of the branch `scripts/rri.py` formula and mappings, **not a claim that the local command was executed on the user's Mac**. The path-freeze-before-scoring requirement is satisfied; every downstream source-writing leaf must run the repository script again in an executable checkout against its own exact paths before presentation/execution.

## Owner gate

Owner `matias` explicitly approved P2.C0 after the RRI 66 Compact Approval Task Card was presented on 2026-09-06. That approval authorizes only the ten-path docs/fixture contract freeze and does not authorize P2.T2–T6 source implementation.
