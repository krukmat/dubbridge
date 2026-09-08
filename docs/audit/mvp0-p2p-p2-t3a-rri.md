---
type: Audit
title: "MVP0-P2P P2.T3a RRI and mandatory decomposition"
date: 2026-09-08
task: MVP0-P2P-P2-T3a
---

# P2.T3a — RRI and mandatory decomposition

## Approval record

Approved by Matias on 2026-09-08 for sequential execution of the complete
frozen `T3a-i` through `T3a-viii` set without additional per-leaf approval.
The owner additionally directed maximum feasible local-model authorship and
minimum primary-agent implementation; the resulting route is recorded in
`docs/audit/mvp0-p2p-p2-t3a-implementation.md`.

## Parent outcome envelope

Command:

```bash
python3 scripts/rri.py --platform generic \
  --touches apps/availability-node/package.json \
  --touches apps/availability-node/package-lock.json \
  --touches apps/availability-node/tsconfig.json \
  --touches apps/availability-node/src/contract.ts \
  --touches apps/availability-node/src/server.ts \
  --cc 12 --D 3 --K 3 --P 2 --T 2 --A 2 --X 2
```

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C | 2 | Estimated raw CC 12 across request parsing, validation, routing, and error mapping | High |
| F | 2 | Five C0-frozen writable paths | High |
| D | 3 | New ciphertext-publication service boundary implementing a frozen cross-runtime domain contract | High |
| T | 2 | Compile plus fixture-driven behavioral checks; the full certification suite remains T3d | High |
| A | 2 | Wire contract is frozen, but the new Node service/toolchain shape still requires bounded implementation choices | High |
| K | 3 | Coupled to the C0 fixture, future mTLS/Hyperdrive leaves, and the Rust T4 client | High |
| P | 2 | New private data-plane service; cannot establish product readiness or receive secrets | High |
| X | 2 | C0, ADR-044, fixture, task ledger, and new service files | High |

Technical profile: `L=2 I=3 Q=3 V=2`, bottleneck `B=3`, `ICI=75`.
Risk/domain input: `20`. Penalties: none.

**Final RRI: 70 — Complex (56–70), Effort L.** Human approval, cloud-primary
implementation, four integrated Reflection passes, and decomposition are
mandatory. The MVP0-P2P owner exception waives only phase-1 and phase-2 peer
review; it does not waive these gates.

## Honest Low-band maximization and executable leaves

The first mandatory split produced two RRI 55 leaves, which was sufficient to
remove the `>=56` hard trigger but did not maximize bounded local authorship.
The user then explicitly requested maximum feasible local-model participation
and minimum primary-agent implementation. A second pass found eight sequential,
independently verifiable increments. Repeated ownership of `contract.ts` and
`server.ts` is intentional: each leaf adds one complete validation or adapter
invariant and verifies the file at that revision before the next leaf starts.

| Leaf | Outcome and exact writable paths | RRI | Acceptance boundary / direct evidence |
|---|---|---:|---|
| `P2.T3a-i` | Deterministic Node/TypeScript package boundary: `package.json`, `package-lock.json` | **25 Low / S** | Runtime/toolchain and scripts are pinned coherently; `npm ci` succeeds from the lock and dependency audit is recorded. No source placeholder. |
| `P2.T3a-ii` | Compiler contract plus frozen v1 vocabulary/types: `tsconfig.json`, `src/contract.ts` | **25 Low / S** | Strict compile succeeds and the exported contract/manifest/error constants exactly match C0. No runtime validation yet. |
| `P2.T3a-iii` | Strict object/version/allowed-field request boundary: `src/contract.ts` | **25 Low / S** | Golden object shape/version passes; non-object, missing/unknown fields, unsupported version, and secret-bearing extra fields fail closed via direct assertions. |
| `P2.T3a-iv` | Publication/lineage/path-identity and digest validation: `src/contract.ts` | **25 Low / S** | Canonical UUIDs and lowercase SHA-256 pass; malformed identity/digest and path/body publication mismatch fail closed. |
| `P2.T3a-v` | Opaque normalized relative `package_ref` validation: `src/contract.ts` | **25 Low / S** | Frozen relative ref passes; absolute, empty, backslash, `.`, `..`, duplicate separator, and root-escape forms fail closed without filesystem access. |
| `P2.T3a-vi` | Frozen success/error envelope validation and serialization: `src/contract.ts` | **25 Low / S** | 201/200 evidence and declared 400/403/409/422/503 codes round-trip; invalid status/code/body, non-RFC3339 time, or mismatched identity/digest fails closed. |
| `P2.T3a-vii` | Bounded HTTP ingress/routing adapter: `src/server.ts` | **25 Low / S** | Only the frozen PUT route accepts bounded JSON and delegates to the pure validator; unsupported method/path, oversized body, invalid JSON, and path/body mismatch return deterministic fail-closed errors. No listener exposure claim or publication side effect. |
| `P2.T3a-viii` | Injected publication seam plus frozen response/error mapping: `src/server.ts` | **25 Low / S** | Handler success emits 201/200 evidence; typed contract/package/conflict/unavailable outcomes map exactly; thrown/unknown outcomes become 503 and never imply `P2P_READY`. Default seam remains unavailable until T3c. |

RRI command shapes:

```bash
# T3a-i
python3 scripts/rri.py --platform generic \
  --touches apps/availability-node/package.json \
  --touches apps/availability-node/package-lock.json \
  --cc 1 --D 1 --K 1 --P 1 --T 1 --A 0 --X 1

# T3a-ii (two paths) and T3a-iii..viii (one path each)
python3 scripts/rri.py --platform generic --touches <exact-path> \
  --cc <2..7> --D 1 --K 1 --P 1 --T 1 --A <0|1> --X 1
```

Every recorded variant resolves to **RRI 25 Low / Effort S**, with no
penalty. The low D/K scores describe one localized invariant per leaf, not the
parent system boundary; the parent RRI 70 continues to govern approval, review
independence, four Reflection passes, and integrated closure. `T3a-i` through
`T3a-viii` execute strictly in order. Each is eligible for the bounded local
Qwen Developer route after parent approval; the Codex primary agent remains
orchestrator-only for packet freeze, diff validation/application, integration,
Reflection, and evidence/status synchronization.

Further splitting stops here: dividing any listed validator into individual
field checks or separating handler invocation from its error mapping would
fragment a behavioral invariant and game RRI rather than produce a meaningful
leaf.

## Local-model advisory preflight

- Ollama restarted once for this task: server PID `30775` -> `31061`; listener
  confirmed on `127.0.0.1:11434`.
- `qwen3.6:27b-q4_K_M` warm-up at `num_ctx=65536`, `num_predict=4096`,
  `think=false`, `temperature=0`: `done_reason=stop`, non-empty JSON content.
- Full Local Architect analysis timed out at 180 seconds with no response;
  failure artifact: `.agent/local-architect/adr037/P2.T3a-low-max/analysis.json`.
- Resource recovery unloaded the model; host reported 80% free memory. Reduced
  warm-up at `num_ctx=16384`, `num_predict=1024` passed, but the single permitted
  reduced analysis retry also timed out at 180 seconds; failure artifact:
  `.agent/local-architect/adr037/P2.T3a-low-max/analysis-reduced.json`.
- Disposition: no Local Architect recommendation is claimed or used. Do not
  retry this advisory unchanged. After approval, local Qwen Developer packets
  will be much smaller and independently bounded to one Low leaf.

## Review disposition

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner.
- Scope-note: skips only phase-1 and phase-2 peer review; the exception expires
  after P7 reaches PASS or STOP.
- Authority: `docs/audit/mvp0-p2p-review-exception.md`.
