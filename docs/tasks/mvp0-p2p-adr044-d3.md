---
type: TaskList
title: "Tasks: ADR044-D3 publication/outbox decision"
status: pending_approval
slice: MVP0-P2P
parent: ADR044-D3
---

# ADR044-D3 — publication/outbox parent envelope

## Decision header

- **Status:** READY FOR OWNER APPROVAL.
- **Parent RRI:** **60 → Complex → Effort L**.
- **Decision:** resolve ADR-044 question 3: durable P2P publication state, intent/outbox semantics, idempotent recovery, and the exact meaning of P2P-ready.
- **Dependency:** ADR044-D2 PASS (`K1`).
- **Hard stop:** no leaf executes and no option is selected before explicit owner approval of this parent.
- **ADR status:** remains `Proposed`.
- **P2 source authorization:** none.
- **Local/device precheck:** `n/a` — cloud environment; D3 is docs/ADR/task-ledger-only and requires no local models, devices, or emulators.
- **Phase-1/phase-2 review:** `n/a` — ADR/plan/task-ledger-only exemption.

## Frozen architecture constraints

D3 must preserve all of the following independently of the selected option:

1. `PreparationStatus::Ready` remains the existing S-120 HLS-readiness signal. P2P publication must not delay S-120 Ready or its downstream transcription enqueue.
2. P2P readiness is a separate durable concept. It is false until the encrypted package has been successfully published and the control plane has durable evidence of that result.
3. PostgreSQL remains source of truth for structured publication metadata and current P2P publication state.
4. External publication side effects are never treated as successful merely because dispatch was attempted.
5. Publication is at-least-once/idempotent at the integration boundary; D3 makes no distributed exactly-once claim.
6. A crash or timeout with unknown external outcome must be recoverable without advertising readiness and without blindly creating a second logical package.
7. Retries of the same logical package reuse its durable identity and K1 server-wrapped CK; a retry does not silently regenerate a new package/CK unless the prior package is explicitly abandoned/replaced by a later approved implementation rule.
8. Availability Node remains ciphertext-only and does not become a business-authorization, database, KEK, or plaintext-key authority.
9. D2/K1 release remains fail-closed: no P3 device envelope may be released until the D3-defined P2P-ready predicate is satisfied.
10. ADR-032 review playback remains unchanged.

## Scope

### In scope

- semantic P2P publication lifecycle;
- durable publication intent;
- dispatch/retry/recovery semantics;
- idempotency boundary and duplicate-delivery behavior;
- success acknowledgement and P2P-ready transition;
- unknown-outcome reconciliation requirement;
- package identity reuse across retries;
- fail-closed interaction with D2 key release;
- responsibility split across PostgreSQL/control plane, worker/package builder, StorageAdapter, and Availability Node.

### Out of scope

- actual SQL schema/table/column names;
- REST/RPC route names or payload encoding;
- retry counts/backoff constants;
- Availability Node deployment/authentication/credential design (ADR-044 question 4);
- certification profile (question 5);
- complete ADR-018 event list (question 6);
- source implementation for P2;
- accepting ADR-044 or advancing D4.

## Task map

| Task | Objective | Status | RRI | Depends on |
|---|---|---|---:|---|
| `ADR044-D3-S1` | Freeze canonical readiness/publication/recovery constraints | Pending parent approval | 14 | D3 approval |
| `ADR044-D3-S2` | Build neutral publication-lifecycle / intent model matrix | Pending | 14 | S1 |
| `ADR044-D3-S3` | Build neutral idempotency / crash-recovery matrix | Pending | 14 | S1 |
| `ADR044-D3-S4` | Build runtime-responsibility / fail-closed readiness matrix | Pending | 14 | S1 |
| `ADR044-D3-OWNER` | Owner selects one coherent publication contract or requests revised evidence | Human checkpoint | human | S2-S4 |
| `ADR044-D3-S5` | Mechanically codify the exact owner selection in audit + ADR | Blocked | 13 | OWNER |
| `ADR044-D3-SYNC` | Propagate D3 status while keeping ADR Proposed/P2 blocked | Blocked | 17 | S5 |

S2-S4 may run in parallel after S1. They prepare evidence only and cannot select an option.

## Neutral option set

The owner checkpoint will compare these as complete contracts, not choose individual rows independently.

### `O1 transactional-outbox`

PostgreSQL records the durable P2P package/publication metadata and a separate publication intent/outbox entry in the same relational transaction. A worker claims/dispatches the intent to the Availability Node. Successful idempotent acknowledgement transitions the publication record to P2P-ready and closes the intent. Retry/recovery operates from durable outbox state.

### `O2 durable-state-reconciler`

A PostgreSQL publication row is both the source of truth and the durable work source. Workers claim/lease rows in pending/retryable state, publish idempotently, and transition the same record to ready/failed. No separate outbox entity is required; recovery is driven by scanning/reclaiming durable state.

### `O3 queue-primary-with-reconciliation`

PostgreSQL records authoritative publication state while the existing job/queue mechanism drives normal dispatch. Because queue delivery cannot be the sole recovery proof, a durable reconciliation scan must detect authoritative pending/in-flight records that were never enqueued, lost, timed out, or have unknown external outcome and re-drive them idempotently.

None of these options decides table names, APIs, Availability Node credentials, or exact retry constants.

## Shared comparison criteria

Every option must be evaluated identically against:

- `R1 S-120 independence` — S-120 Ready and ASR enqueue remain unaffected.
- `R2 source of truth` — Postgres is authoritative for structured publication state.
- `R3 durable intent` — no external side effect can be required without a durable recoverable intent/state.
- `R4 crash windows` — before dispatch, during dispatch, after remote success/before local acknowledgement, and after acknowledgement are all recoverable.
- `R5 idempotency` — duplicate delivery/retry converges to one logical package/publication result.
- `R6 unknown outcome` — timeout/connection loss never produces readiness without reconciliation.
- `R7 fail-closed readiness` — P2P-ready only follows durable evidence of successful publication.
- `R8 K1 continuity` — retry does not silently rotate/regenerate package CK or weaken D2 custody semantics.
- `R9 Availability Node boundary` — node remains ciphertext-only and non-authoritative for business state.
- `R10 scope discipline` — D3 does not decide deployment/auth/API/schema details reserved for later work.

Findings are limited to `satisfied`, `conditional`, `conflict`, or `undecided` plus rationale. There is no aggregate score/winner generated by the agent.

## Parent RRI report

| Variable | Score | Evidence |
|---|---:|---|
| C | 1 | One architecture decision envelope; no source implementation |
| F | 3 | Expected propagation across task/audit/ADR plus canonical status docs |
| D | 5 | Publication-state correctness, cross-store recovery and fail-closed readiness are high-domain-risk |
| T | 0 | Documentation decision; deterministic docs QA only |
| A | 0 | Decision boundary, options, criteria and owner checkpoint are explicit |
| K | 4 | Couples S-120, worker/storage, Postgres, Availability Node, D2 key release and later P2/P3 |
| P | 4 | Shapes the future P2 publication contract and readiness semantics |
| X | 4 | Cross-cutting state/recovery boundary across multiple runtimes/stores |

Base: **48**. Penalty: `arch_decision +12`. Final: **60 → Complex → Effort L**.

Canonical scoring command:

```bash
python3 scripts/rri.py \
  --touches docs/tasks/mvp0-p2p-adr044.md \
  --touches docs/tasks/mvp0-p2p-adr044-d3.md \
  --touches docs/audit/mvp0-p2p-adr044-d3-publication.md \
  --touches docs/adr/ADR-044-p2p-audience-delivery-boundary.md \
  --touches docs/plan/mvp0-p2p-first.md \
  --touches docs/tasks/mvp0-p2p-first.md \
  --touches docs/plan/roadmap.md \
  --touches docs/architecture.md \
  --touches docs/adr/README.md \
  --C 1 --D 5 --K 4 --P 4 --T 0 --A 0 --X 4 \
  --penalty arch_decision
```

### Leaf RRI

S1-S4:

```bash
python3 scripts/rri.py \
  --touches docs/tasks/mvp0-p2p-adr044-d3.md \
  --C 0 --D 3 --K 1 --P 0 --T 0 --A 0 --X 2
```

**RRI 14 → Low → Effort S**.

S5: docs task/audit/ADR mechanical codification only: **RRI 13 → Low → Effort S**.

SYNC: canonical status propagation only: **RRI 17 → Low → Effort S**.

## Acceptance

D3 closes only if:

1. S1-S4 produce cited neutral evidence with identical criteria and no recommendation language.
2. Owner explicitly selects one complete option at `ADR044-D3-OWNER`.
3. The selected contract defines separate P2P readiness, durable intent, idempotency and all crash windows without changing S-120 Ready semantics.
4. Unknown external outcome fails closed until reconciled.
5. Retry preserves logical package/K1 key continuity unless an explicit future replacement rule is approved.
6. Availability Node remains non-authoritative and ciphertext-only.
7. S5 transcribes only the owner selection and keeps ADR-044 `Proposed`.
8. SYNC leaves D4 and P2 source work blocked.
9. `make qa-docs`, `git diff --check`, and `AGENTS.md`/`AGENTS.override.md` parity pass after codification.

## Agent workflow

- Orchestrator / primary: GPT-5.6 Sol cloud-primary.
- Local stack: `n/a` for this docs-only decision envelope.
- Phase-1/phase-2 reviewer: `n/a` under docs/ADR/task-ledger exemption.
- Parent RRI 60 requires **4 integrated Reflection passes** after owner selection:
  1. transactional/crash-window correctness;
  2. idempotency/unknown-outcome recovery;
  3. S-120 independence + D2 fail-closed interaction;
  4. scope/status propagation with no D4/P2 creep.

## Approval checkpoint

No D3 leaf has executed. No publication option has been selected. ADR-044 remains `Proposed` and P2 remains unauthorized.

Execution has not started. Approve this task to proceed.
