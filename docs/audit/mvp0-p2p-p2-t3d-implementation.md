---
type: Audit
title: "P2.T3d implementation and closure evidence"
status: done
task: P2.T3d
---

# P2.T3d implementation and closure evidence

`P2.T3d` / `CONS-T3` was owner-approved on 2026-09-18 at RRI 70 Complex.
Implementation, executable verification, phase-2 review, four Reflection
passes, behavioral coverage certification, and owner final verification are
complete. `P2.T3d`, aggregate T3, and `CONS-T3` are Done as of 2026-09-18.

## Scope and implementation

Only the two frozen test paths were implemented:

- `apps/availability-node/test/fixtures.js` — reusable local OpenSSL mTLS
  identities, real Rust fixture invocation, HTTPS client, console capture,
  secret deny-list loader, and deterministic temporary-root cleanup.
- `apps/availability-node/test/publication-contract.test.js` — full
  mTLS/HTTP/executor/Hyperdrive certification for HP-T3d-1, HP-T3d-2,
  EC-T3d-1, EC-T3d-2, and EC-T3d-3.

No `apps/availability-node/src/` file changed. The pre-existing
`publication-contract-certification.test.js` remains untouched because its
removal was not approved; the correctly-scoped test supersedes it as T3d's
closure evidence without representing that narrower file as sufficient.

## Executable verification

| Command | Result |
|---|---|
| `npm --prefix apps/availability-node run typecheck` | PASS |
| `npm --prefix apps/availability-node run build` | PASS |
| `node --test apps/availability-node/test/publication-contract.test.js` | PASS — 8/8 checks; repeated after cleanup hardening |
| `node --test apps/availability-node/test/*.test.js docs/audit/mvp0-p2p-p2-t3a-contract.test.js docs/audit/mvp0-p2p-p2-t3a-http.test.js` | PASS — 98/98 checks, 0 failed |
| no-index whitespace checks for both new files | PASS — no diagnostics |

The focused evidence includes exact `201`/`200` replay, stable evidence,
unchanged Hypercore block count, public-key binding, byte-identical manifest
and ciphertext, TLS-handshake/`403` identity separation, exact
`409`/`422`/`400`/`503` wire outcomes, absent side effects, and a negative
scan of all eight frozen secret fields and synthetic values across HTTP,
console output, the persisted index record, and raw drive bytes.

## Antares touchpoints

- Refinement: typed skip — inherited from the approved preflight boundary.
  The static watchlist has no hypothesis scoped to
  `apps/availability-node/`; its CWE-22 entry is explicitly restricted to
  `crates/storage/`, so a generic path-traversal sweep is prohibited.
- Post-implementation: typed skip for the same reason. The task adds tests
  only and does not change the watchlist's governed boundary.

## Peer review

The per-task Ollama precheck replaced server PID `43607` with PID `69307`,
confirmed the `127.0.0.1:11434` listener, and warmed `gpt-oss:20b` at the
Complex profile with `done_reason: stop` and non-empty JSON content.

Phase 2 used the direct Ollama API because the approved handoff records the
RRI-56+ routing defect in `scripts/peer-workflow-review.py`.

```
Task-analysis review: gpt-oss .agent/p2-t3d/phase1-review-v1.json - PASS
Code-solution review: gpt-oss .agent/p2-t3d/phase2-review-v1.json - PASS
```

The phase-2 reviewer returned `PASS` with two LOW observations:

1. `createConsoleCapture` could interfere with a second concurrent capture.
   **Disposition: rejected-not-applicable.** This suite invokes it exactly
   once, from one top-level test, without opting into in-file concurrency,
   and restores it in `finally`.
2. Cargo and OpenSSL are external command dependencies. **Disposition:
   accepted-no-change.** Reusing the real Rust fixture binary and the
   established OpenSSL mTLS harness is an explicit acceptance requirement;
   both dependencies passed in focused and integrated execution.

## Reflection log

Required and completed: 4 passes (RRI 70 Complex).

### Pass 1 — behavior and persistence

- **Draft:** the real Rust-built ciphertext package traverses mTLS + HTTP +
  the real executor, returns `201`, survives server/executor/store
  reconstruction, and replays with `200` and identical evidence.
- **Critique:** unchanged bytes alone would not prove absence of a second
  write, and cleanup originally swallowed a possible shared-store close
  failure.
- **Revise:** retained the `drive.core.length` before/after assertion and
  hardened cleanup to remove roots in `finally` while rethrowing any close
  error. Re-ran the focused suite: 8/8 passed.

### Pass 2 — trust and confidentiality boundaries

- **Draft:** no-cert and rogue-cert clients fail during TLS, a CA-trusted
  unlisted client receives exact `403`, and none invokes the real executor.
- **Critique:** secret non-leakage must cover server-owned persistence and
  drive bytes, not only the response schema.
- **Revise:** confirmed the test injects distinct synthetic values for all
  eight frozen deny-list fields, then scans HTTP responses, captured console
  output, persisted index bytes, the verified manifest, and every declared
  ciphertext buffer in the drive. No revision was needed.

### Pass 3 — failure semantics and side effects

- **Draft:** conflicts, invalid packages, malformed framing, and forced
  network failure map exactly to `409`, `422`, `400`, and `503`.
- **Critique:** failure codes would be insufficient if a conflict rewrote
  the drive/index or if pre-verification failures created drive state.
- **Revise:** confirmed both conflicts preserve the original index evidence
  and Hypercore length; tamper/symlink cases leave drive storage empty and
  the index absent; the timeout leaves the success index absent. The
  reviewer's Cargo/OpenSSL dependency observation is intentional and
  fail-closed, so no code change was appropriate.

### Pass 4 — scope, integration, and reviewer disposition

- **Draft:** typecheck, build, 8/8 focused checks, and 98/98 integrated
  checks pass with no product-source changes.
- **Critique:** the reviewer noted a hypothetical overlapping console
  capture; the earlier, narrower certification file could also be mistaken
  for the new closure evidence.
- **Revise:** independently confirmed there is one capture invocation and no
  concurrency opt-in, recorded the finding as not applicable, and explicitly
  retained the old file only as pre-existing out-of-scope work. The new
  `publication-contract.test.js` is the sole T3d behavioral evidence.

## Behavioral coverage certification

Behavioral coverage contract: `behavior-v2`.

| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-T3d-1 | Happy path | Allow-listed mTLS client publishes a real Rust-built ciphertext package with `201`; reconstruction over the same roots replays with `200`, identical evidence, and no second drive write | e2e | `apps/availability-node/test/publication-contract.test.js::HP-T3d-1/2 + EC-T3d-3 secrecy` | passed |
| HP-T3d-2 | Happy path | Response public identifier matches the opened drive; manifest and all ciphertext bytes equal `verifyPackage` output; response has no readiness/authorization decision | e2e | `apps/availability-node/test/publication-contract.test.js::HP-T3d-1/2 + EC-T3d-3 secrecy` | passed |
| EC-T3d-1 | Edge case | Absent and rogue certificates fail TLS; trusted-but-unlisted certificate receives `403`; real executor call count remains zero | e2e | `apps/availability-node/test/publication-contract.test.js::EC-T3d-1` | passed |
| EC-T3d-2a | Edge case | Same publication with different lineage receives `409`, stable original evidence, and no second drive write | e2e | `apps/availability-node/test/publication-contract.test.js::EC-T3d-2 conflicts` | passed |
| EC-T3d-2b | Edge case | Same publication/lineage with different digest receives `409`, stable original evidence, and no second drive write | e2e | `apps/availability-node/test/publication-contract.test.js::EC-T3d-2 conflicts` | passed |
| EC-T3d-2c | Edge case | Real ciphertext tamper receives `422` before drive/index creation | e2e | `apps/availability-node/test/publication-contract.test.js::tampered ciphertext` | passed |
| EC-T3d-2d | Edge case | Symlink escape receives `422` before drive/index creation | e2e | `apps/availability-node/test/publication-contract.test.js::symlink escape` | passed |
| EC-T3d-2e | Edge case | Malformed JSON, wrong content type, and oversized body each receive `400 invalid_contract` without executor invocation | e2e | `apps/availability-node/test/publication-contract.test.js::EC-T3d-2 request framing` | passed |
| EC-T3d-3a | Edge case | Forced Hyperswarm join timeout receives `503 publication_unavailable` and commits no success record | e2e | `apps/availability-node/test/publication-contract.test.js::EC-T3d-3` | passed |
| EC-T3d-3b | Edge case | All eight secret fields and synthetic canaries are absent from response, console, index, and raw drive surfaces | e2e | `apps/availability-node/test/publication-contract.test.js::HP-T3d-1/2 + EC-T3d-3 secrecy` | passed |
| INT-T3d-6 | Integration | T3a-T3d tests pass together; temporary roots clean deterministically | integration | integrated Node command above; cleanup assertions in `fixtures.js` | passed |

## Owner final verification

- Owner: `Matias`
- Date: `2026-09-18`
- Statement: the owner accepted the mapped happy-path/edge-case evidence and
  explicitly instructed closure of `P2.T3d`, aggregate T3, and `CONS-T3`, with
  T3/T4c status synchronization. This confirmation does not close P2 or start
  another task.
- Commands accepted from the recorded verification evidence:
  `npm --prefix apps/availability-node run typecheck`;
  `npm --prefix apps/availability-node run build`;
  `node --test apps/availability-node/test/publication-contract.test.js`;
  `node --test apps/availability-node/test/*.test.js
  docs/audit/mvp0-p2p-p2-t3a-contract.test.js
  docs/audit/mvp0-p2p-p2-t3a-http.test.js`.
- Closure-turn rerun: none. The owner explicitly deferred the documentary
  gates (`git diff --check` and `make qa-docs`) for now; they remain pending
  and are not represented as post-closure PASS.

`P2.T3d`, `CONS-T3`, and aggregate T3 are closed. `P2` remains open, and no
downstream task is started by this evidence record.
