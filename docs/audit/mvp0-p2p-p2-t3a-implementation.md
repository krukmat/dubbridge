---
type: Audit
title: "MVP0-P2P P2.T3a Availability Node contract implementation"
date: 2026-09-08
task: MVP0-P2P-P2-T3a
---

# P2.T3a — Availability Node contract implementation

## Result

Done; repository-owner final verification recorded on 2026-09-08.
The new `apps/availability-node` package provides the strict
`availability-publication-v1` request/response boundary, bounded HTTP ingress,
and an injected publication executor. Its default executor returns
`503 publication_unavailable`; this task opens no listener, seeds no drive,
and cannot establish `P2P_READY`.

Writable product files stayed inside the approved parent envelope:

- `apps/availability-node/package.json`
- `apps/availability-node/package-lock.json`
- `apps/availability-node/tsconfig.json`
- `apps/availability-node/src/contract.ts`
- `apps/availability-node/src/server.ts`

## Implementation routing evidence

- Parent: RRI 70 Complex / Effort L, approved for its frozen eight-leaf
  decomposition on 2026-09-08.
- Developer: local `qwen3.8:27b-mlx`, digest
  `5642e97495e1a088883805981563dcdc4a040c2f53388b7a41d1f24d3622cf7e`.
- Runtime: Ollama, `num_ctx=65536`, `temperature=0.1`, `think=false`; standard
  `num_predict=4096`, reduced only for bounded before/after repairs.
- Sequence: `T3a-i` through `T3a-viii` dispatched and integrated strictly in
  order with `scripts/delegate-low-rri.py`; every product-logic draft and
  repair came from the local developer.
- Repair use: `T3a-ii`, `T3a-iii`, `T3a-v`, `T3a-vi`, `T3a-vii`, and
  `T3a-viii` each used one bounded repair cycle. The first `T3a-iii` repair
  invocation returned `BLOCKED` because the packet omitted the literal before
  block; the corrected invocation reused attempt 2 because this was an
  orchestrator packet-construction failure, not a model patch attempt.
- Orchestrator-only changes were non-substantive integration mechanics:
  npm generated `package-lock.json`; one leaked `--- CONTENT ---` wrapper
  marker was removed; the required strict compiler flags/Node type binding
  were restored; compiler-directed non-null/type-name fixes and a no-op-loop/
  stale-comment cleanup were applied. No product branch, validator rule, HTTP
  mapping, or executor behavior was authored by the orchestrator.
- Local attempt/result packets and the append-only runtime audit live under
  ignored `.agent/delegations/P2.T3a/` and `logs/gemma-audit/2026-09.jsonl`.
  No cloud implementer or fallback was invoked.

## Implemented boundary

- Exact six-field request allow-list and frozen contract/manifest versions.
- Canonical lowercase UUIDs, lowercase SHA-256 digest, and URL/body identity
  equality.
- NFC, relative, slash-only opaque `package_ref` with traversal, absolute,
  duplicate-separator, control-character, and Windows-drive rejection.
- Exact seven-field success evidence, real RFC3339 UTC validation, stable
  200/201 representation, and exact 400/403/409/422/503 error vocabulary.
- PUT-only 64 KiB JSON ingress with fatal UTF-8 decoding, exact route,
  content-type/query/content-length checks, and redacted deterministic errors.
- One injected executor invocation after validation; success/error results are
  revalidated. Malformed, mismatched, thrown-unknown, or otherwise ambiguous
  results become `503 publication_unavailable`.

## Antares touchpoints

- Refinement: `TYPED-SKIP — no task-relevant CWE hypothesis on the T3a
  watchlist; apps/availability-node does not match the current watched
  crates/db, apps/api, or crates/storage surfaces.`
- Post-implementation: `TYPED-SKIP — the candidate snapshot introduces no
  task-relevant watchlist hypothesis; no generic Antares sweep was run.`

## Task-analysis review

`Task-analysis review: n/a - REVIEW-OVERRIDE: urgency, see
docs/audit/gemma-review-overrides.md row P2.T3a`

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner.
- Scope-note: skips only phase-1 peer review; RRI, approval, tests, Reflection,
  status synchronization, and owner final verification remain mandatory.

## Code-solution review

`Code-solution review: n/a - REVIEW-OVERRIDE: urgency, see
docs/audit/gemma-review-overrides.md row P2.T3a`

- REVIEW-OVERRIDE: urgency — explicit owner-directed MVP0-P2P exception.
- Waiver-by: Matias, repository owner.
- Scope-note: skips only phase-1 and phase-2 peer review; tests, Reflection,
  status synchronization, and owner final verification remain mandatory.
- Authority: `docs/audit/mvp0-p2p-review-exception.md`.

## Behavioral coverage certification

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy path | frozen request and 201 evidence round-trip through the exact v1 contract | contract | `docs/audit/mvp0-p2p-p2-t3a-contract.test.js::HP-1` | passed |
| HP-2 | Happy path | validated ingress invokes the injected executor once and emits exact 201/200 evidence | integration | `docs/audit/mvp0-p2p-p2-t3a-http.test.js::HP-2` | passed |
| EC-1 | Edge case | unknown/secret-bearing fields and unsafe package references fail closed before executor invocation | contract | `docs/audit/mvp0-p2p-p2-t3a-contract.test.js::EC-1` and `docs/audit/mvp0-p2p-p2-t3a-http.test.js::EC-2` | passed |
| EC-2 | Edge case | unsupported ingress and malformed, mismatched, or ambiguous executor results map to a frozen error and never imply readiness | integration | `docs/audit/mvp0-p2p-p2-t3a-http.test.js::EC-2` plus negative source-boundary scan | passed |

Supporting gates also passed: deterministic `npm ci`, zero runtime dependency
vulnerabilities, strict typecheck/build, and `git diff --check`. The case IDs
above formalize the acceptance boundaries approved in the T3a RRI artifact;
they add no behavior to the implemented scope.

The two tracked Node test runners are the bounded, reproducible executable
evidence required by T3a. Run them after `npm run build` with:

```bash
node --test docs/audit/mvp0-p2p-p2-t3a-contract.test.js \
  docs/audit/mvp0-p2p-p2-t3a-http.test.js
```

The first closure rerun used an invalid uppercase check against an all-numeric
fixture UUID; the harness was corrected to exercise an alphabetic uppercase
UUID and then passed. No product code changed. The broader tracked contract/
mTLS/idempotency/traversal/secret certification files remain owned by `T3d`;
this record does not claim that later full-T3 gate.

## Parent Reflection — 4/4 PASS

### Pass 1 — deterministic contract and identity

- Draft: compare constants, request/evidence field sets, statuses, and golden
  values against the C0 fixture.
- Critique: typecheck alone could not prove URL/body identity or exact
  response-field rejection.
- Revise: the final fixture matrix exercises canonical identities, exact
  allow-lists, digest matching, round-trip serialization, and mismatches.
- Verdict: PASS.

### Pass 2 — trust, secrets, and path confinement

- Draft: inspect imports, accepted request keys, errors, and package-ref rules.
- Critique: normalization that silently rewrites input would permit two textual
  identities for one ref, and extra secret-bearing fields needed executable
  rejection evidence.
- Revise: the local repair rejects non-NFC input; the final matrix iterates the
  frozen deny-list and traversal/absolute/control variants. Source scan proves
  no filesystem, DB, key, Hyperdrive, or listener authority entered T3a.
- Verdict: PASS.

### Pass 3 — ambiguous outcomes and readiness separation

- Draft: verify default-unavailable and executor error mappings.
- Critique: a malformed returned success must not be confused with a directly
  thrown typed contract outcome, and neither may imply readiness.
- Revise: publisher invocation and returned-result validation use separate
  error boundaries. Direct typed outcomes retain their code; malformed,
  mismatched, unknown, or rejected results become 503. No readiness state or
  persistence exists in this package.
- Verdict: PASS.

### Pass 4 — scope, integration, and downstream ownership

- Draft: reconcile every changed product path and planned T3 boundary.
- Critique: strict compiler options drifted during a local repair and initially
  masked Node/type-narrowing errors; generated build/install outputs must not
  become tracked scope.
- Revise: restored the frozen strict config, resolved only compiler-directed
  integration issues, reran clean install/build/matrices, and confirmed
  `dist/` plus `node_modules/` remain ignored. T3b/T3c/T3d are unchanged.
- Verdict: PASS.

## Owner final verification

- Owner: `Matias`
- Date: `2026-09-08`
- Statement: I verified that every T3a happy path and edge case above has
  executable evidence at the declared layer, that the implementation remains
  inside the approved contract/bootstrap boundary, and that T3b-T3d remain
  unstarted and separately gated.
- Commands run: `npm --prefix apps/availability-node ci --ignore-scripts --no-audit --no-fund`; `npm --prefix apps/availability-node audit --omit=dev --json`; `npm --prefix apps/availability-node run typecheck`; `npm --prefix apps/availability-node run build`; `node --test docs/audit/mvp0-p2p-p2-t3a-contract.test.js docs/audit/mvp0-p2p-p2-t3a-http.test.js`; `if rg -n 'createServer|listen\(|hyperdrive|postgres|plaintext_ck|server_kek|P2P_READY' apps/availability-node/src; then exit 1; fi`; `make qa-docs`; `git diff --check`.

T3a is `Done`. The next executable dependency is `T3b`; its RRI and approval
gate must be run separately before implementation.
