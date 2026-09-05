---
type: TaskList
title: "Tasks: ADR044-D3 publication/outbox decision"
status: pending_owner
slice: MVP0-P2P
parent: ADR044-D3
---

# ADR044-D3 — publication/outbox parent envelope

## Decision header

- **Status:** APPROVED 2026-09-05; S1-S4 complete; stopped at `ADR044-D3-OWNER`.
- **Parent RRI:** **60 → Complex → Effort L**.
- **Decision:** resolve ADR-044 question 3: durable P2P publication state, intent/outbox semantics, idempotent recovery, and the exact meaning of P2P-ready.
- **Dependency:** ADR044-D2 PASS (`K1`).
- **Owner approval:** explicit `aprobado` on 2026-09-05 authorized S1-S4 only up to the mandatory owner-selection checkpoint.
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
| `ADR044-D3-S1` | Freeze canonical readiness/publication/recovery constraints | Complete 2026-09-05 | 14 | D3 approval |
| `ADR044-D3-S2` | Build neutral publication-lifecycle / intent model matrix | Complete 2026-09-05 | 14 | S1 |
| `ADR044-D3-S3` | Build neutral idempotency / crash-recovery matrix | Complete 2026-09-05 | 14 | S1 |
| `ADR044-D3-S4` | Build runtime-responsibility / fail-closed readiness matrix | Complete 2026-09-05 | 14 | S1 |
| `ADR044-D3-OWNER` | Owner selects one coherent publication contract or requests revised evidence | **WAITING OWNER** | human | S2-S4 |
| `ADR044-D3-S5` | Mechanically codify the exact owner selection in audit + ADR | Blocked | 13 | OWNER |
| `ADR044-D3-SYNC` | Propagate D3 status while keeping ADR Proposed/P2 blocked | Blocked | 17 | S5 |

S2-S4 ran after S1 and produced evidence only. No option has been selected by the agent.

## Neutral option set

The owner checkpoint compares these as complete contracts, not individual rows chosen independently.

### `O1 transactional-outbox`

PostgreSQL records the durable P2P package/publication metadata and a separate publication intent/outbox entry in the same relational transaction. A worker claims/dispatches the intent to the Availability Node. Successful idempotent acknowledgement transitions the publication record to P2P-ready and closes the intent. Retry/recovery operates from durable outbox state.

### `O2 durable-state-reconciler`

A PostgreSQL publication row is both the source of truth and the durable work source. Workers claim/lease rows in pending/retryable state, publish idempotently, and transition the same record to ready/failed. No separate outbox entity is required; recovery is driven by scanning/reclaiming durable state.

### `O3 queue-primary-with-reconciliation`

PostgreSQL records authoritative publication state while the existing job/queue mechanism drives normal dispatch. Because queue delivery cannot be the sole recovery proof, a durable reconciliation scan must detect authoritative pending/in-flight records that were never enqueued, lost, timed out, or have unknown external outcome and re-drive them idempotently.

None of these options decides table names, APIs, Availability Node credentials, or exact retry constants.

## Shared comparison criteria

Every option is evaluated identically against:

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

Findings are limited to `satisfied`, `conditional`, `conflict`, or `undecided` plus rationale. There is no aggregate score or agent-selected winner.

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

# D3 decision evidence workspace

## S1 — frozen constraint register

Status: **complete 2026-09-05**.

The following constraints are facts/bounds for D3 and are not option preferences:

| ID | Frozen D3 constraint | Source / authority |
|---|---|---|
| `D3-C1` | S-120 `PreparationStatus::Ready` remains HLS preparation readiness and must not wait for P2P publication; downstream transcription enqueue remains on the existing path. | `docs/plan/mvp0-p2p-first.md` guardrail 8 and P2 EC-2 in `docs/tasks/mvp0-p2p-first.md` |
| `D3-C2` | P2P readiness is separate and fail-closed: the asset/package cannot be advertised as P2P-ready before confirmed publication. | G2 / P2 HP-1 and EC-1 in design inputs and main task ledger |
| `D3-C3` | PostgreSQL remains the source of truth for structured publication metadata/current state. | `docs/architecture.md`, ADR-006, design-input global invariant 3 |
| `D3-C4` | Publication crosses a transactional boundary: durable control-plane state and an external P2P publication side effect cannot be committed atomically as one distributed transaction. D3 therefore requires recoverable intent/state rather than an exactly-once claim. | consequence of `D3-C3`, StorageAdapter boundary, Availability Node separation, and D3 scope |
| `D3-C5` | Dispatch is not success. P2P-ready requires durable evidence of the externally successful logical publication. | P2 HP-1 / EC-1 and parent frozen constraints 2/4 |
| `D3-C6` | Delivery/retry is at-least-once and must be idempotent at the publication integration boundary. | parent frozen constraint 5; recovery requirement from P2 HP-2 |
| `D3-C7` | Unknown remote outcome (timeout, disconnect, crash after remote success) is not failure or success by assumption; it remains non-ready until deterministic reconciliation establishes the result or safely re-drives the same logical publication. | parent constraints 4/6 and fail-closed P2 EC-1 |
| `D3-C8` | Retries preserve one durable logical package identity and the D2/K1 server-wrapped CK. Retry cannot silently create a new package/key lineage. | D2/K1 contract + parent constraint 7 |
| `D3-C9` | Availability Node may store/seed ciphertext and return publication evidence, but cannot own PostgreSQL state, business authorization, plaintext CK, KEK, invite/viewer data, or backend signing authority. | slice guardrail 11; ADR-044 proposed boundary; D2 responsibility split |
| `D3-C10` | Device-envelope release is gated by current P2P-ready state; synchronized ciphertext or a Hyperdrive key alone is insufficient. | D1 `O3 parallel` + D2/K1 fail-closed release predicate |
| `D3-C11` | D3 decides semantic state/recovery only. Concrete schema names, endpoint shapes, queue technology, lease durations, retry constants, Availability Node auth/deployment, and complete audit-event list remain later decisions. | approved D3 scope |

### Frozen semantic lifecycle

All options must preserve this semantic progression even if they encode it differently:

```text
S-120 HLS Ready
      |
      +--> ASR/transcription continues independently
      |
      +--> durable P2P publication intent/state
                    |
                    v
           package prepared/encrypted
                    |
                    v
             publication attempted
                    |
          +---------+----------+
          |                    |
          v                    v
   unknown/retryable     confirmed success
          |                    |
          +---- reconcile -----+
                               v
                           P2P_READY
```

`P2P_READY` is a semantic predicate, not a proposed enum/table/field name.

## S2 — neutral lifecycle / durable-intent matrix

Status: **complete 2026-09-05**.

| Criterion | `O1 transactional-outbox` | `O2 durable-state-reconciler` | `O3 queue-primary-with-reconciliation` |
|---|---|---|---|
| `R1 S-120 independence` | **satisfied** — outbox/publication state is downstream and independent of S-120 Ready. | **satisfied** — publication row is downstream and independent of S-120 Ready. | **satisfied** — authoritative publication row + queue dispatch remains downstream. |
| `R2 source of truth` | **satisfied** — publication row and outbox live in PostgreSQL; remote acknowledgement only informs the authoritative transition. | **satisfied** — one PostgreSQL publication row is explicitly authoritative. | **satisfied** — PostgreSQL remains authoritative; queue state is never treated as product truth. |
| `R3 durable intent` | **satisfied** — publication metadata + outbox intent are committed in the same local transaction before dispatch. | **satisfied** — the durable publication row itself is the work intent. | **conditional** — satisfied only if authoritative pending state is committed before enqueue and reconciliation can discover enqueue loss. |
| `R4 crash windows` | **satisfied** — unclosed outbox work remains reclaimable; remote-success/ack-loss still requires idempotent reconciliation. | **conditional** — requires explicit lease/reclaim semantics on in-flight rows and reconciliation of unknown external outcome. | **conditional** — requires both queue redelivery handling and a durable reconciliation path independent of queue visibility. |
| `R5 idempotency` | **conditional** — stable logical publication identity/idempotency key must be honored across duplicate outbox delivery. | **conditional** — stable identity must be honored across reclaimed/scanned row processing. | **conditional** — stable identity must be honored across queue duplicates and reconciler re-drive. |
| `R6 unknown outcome` | **conditional** — the outbox alone cannot know remote success after ack loss; the Availability boundary must support deterministic same-identity retry or status reconciliation. | **conditional** — same requirement; row state remains non-ready until reconciled. | **conditional** — queue ack is not publication proof; authoritative state + external reconciliation must resolve uncertainty. |
| `R7 fail-closed readiness` | **satisfied** — Ready follows durable successful acknowledgement, never dispatch. | **satisfied** — state transition to Ready occurs only after confirmed publication. | **satisfied** — queue completion alone cannot set Ready; PostgreSQL transition requires confirmed publication. |
| `R8 K1 continuity` | **satisfied** — one durable package/publication record and its outbox retries reuse the same K1 lineage. | **satisfied** — the same durable publication row is re-driven. | **satisfied** — queue/reconciler payloads refer to the same authoritative logical publication. |
| `R9 Availability Node boundary` | **satisfied** — node receives ciphertext publication request and returns evidence; no DB/business authority required. | **satisfied** — same boundary. | **satisfied** — same boundary; queue mechanism does not grant the node control-plane authority. |
| `R10 scope discipline` | **satisfied** — semantic outbox contract can be selected without naming tables/routes. | **satisfied** — semantic durable-row contract can be selected without schema detail. | **satisfied** — semantic queue+reconciliation contract can be selected without choosing a queue implementation. |

### Lifecycle-level tradeoffs

| Option | Direct strength | Visible cost / assumption |
|---|---|---|
| `O1` | Makes “business state + publish intent” atomic inside PostgreSQL and separates intent lifecycle from product publication lifecycle. | Introduces a distinct durable outbox concept and dispatcher lifecycle; still needs external idempotency/reconciliation for ack-loss. |
| `O2` | Uses one durable state machine as both product state and work source; fewer conceptual records. | Product state, work claiming, leases, attempts and recovery become more tightly coupled; implementation must prevent stuck/in-flight rows from becoming ambiguous. |
| `O3` | Reuses queue-driven orchestration for the common path while keeping PostgreSQL authoritative. | Queue and reconciler are two execution mechanisms; correctness depends on proving neither queue loss nor queue success can drift from authoritative publication state. |

No option is ranked or selected by S2.

## S3 — idempotency and crash-recovery matrix

Status: **complete 2026-09-05**.

### Shared crash windows

| Window | Required result for every option |
|---|---|
| `W0 before durable intent/state commit` | No publication is required. A later orchestration attempt may create the first durable intent. |
| `W1 after durable commit, before dispatch` | Durable state proves work remains due; recovery eventually dispatches the same logical publication. |
| `W2 during dispatch / remote operation` | Crash/timeout yields non-ready unknown/retryable state. Same logical identity is reconciled/re-driven. |
| `W3 remote success, acknowledgement lost` | Never create a second logical package. Retry/query must converge on the already-published identity or reproduce the same idempotent result. |
| `W4 acknowledgement received, before local Ready commit` | Durable work remains non-ready/recoverable; replay/reconciliation may confirm the same external publication and then commit Ready. |
| `W5 after local Ready commit, duplicate delivery arrives` | Duplicate is a no-op/idempotent confirmation of the same package; it cannot rotate CK, package identity, or regress readiness. |

### Option behavior by crash window

| Window | `O1 transactional-outbox` | `O2 durable-state-reconciler` | `O3 queue-primary-with-reconciliation` |
|---|---|---|---|
| `W0` | **satisfied** — transaction did not commit, so no outbox obligation exists. | **satisfied** — no durable row means no work exists. | **satisfied** — no authoritative row means no queue obligation exists. |
| `W1` | **satisfied** — pending outbox is durable and reclaimable. | **satisfied** — pending publication row is discoverable by scan/claim. | **conditional** — reconciler must detect authoritative pending state with absent/lost enqueue. |
| `W2` | **conditional** — claim/attempt must become retryable/reclaimable after worker loss; external same-ID operation required. | **conditional** — lease expiry/reclaim must restore processability; same-ID operation required. | **conditional** — queue redelivery and reconciler may race; both must converge through one authoritative same-ID transition. |
| `W3` | **conditional** — outbox replay uses stable identity; remote boundary must return existing publication or deterministically confirm it. | **conditional** — reclaimed row uses stable identity; same remote requirement. | **conditional** — queue retry/reconciler use stable identity; queue message identity alone is insufficient. |
| `W4` | **satisfied** if outbox remains open until PostgreSQL records Ready; replay can reconfirm then close. | **satisfied** if row remains non-ready until Ready transaction commits. | **conditional** — queue may consider delivery complete, so reconciler must still discover authoritative non-ready state. |
| `W5` | **satisfied** — closed/ready publication makes duplicate outbox work an idempotent no-op. | **satisfied** — ready row is terminal for same lineage; stale lease/retry cannot mutate it. | **conditional** — stale queue deliveries/reconciler passes must check authoritative Ready before side effects. |

### Required integration semantics regardless of option

1. One **stable logical publication identity** exists before the first external side effect.
2. The encrypted package identity, manifest/hash evidence and K1 wrapped-key lineage are stable for retries of that logical publication.
3. The Availability boundary must permit either:
   - idempotent publish under that stable identity; or
   - deterministic reconciliation/query sufficient to distinguish “already published” from “must safely re-drive”.
4. A transport timeout or queue acknowledgement is never publication success evidence by itself.
5. A future explicit package replacement is a new lineage, not an accidental retry side effect. D3 does not define the replacement API/state machine.

No option is selected by S3.

## S4 — responsibility and fail-closed readiness matrix

Status: **complete 2026-09-05**.

### Fixed responsibilities

| Runtime / boundary | Owns | Must not own / infer |
|---|---|---|
| PostgreSQL / control plane | authoritative logical publication identity; current P2P publication state; package metadata; durable intent/work evidence; confirmed Ready transition | P2P transport execution; assumption that dispatch/queue ack equals publication success |
| Worker / package builder | consume S-120 output after Ready; create/reuse K1 encrypted package lineage; dispatch/reconcile publication; persist result through control-plane repositories | redefine S-120 Ready; rotate CK/package identity on retry; mark Ready without durable confirmed evidence |
| `StorageAdapter` / integration seam | preserve existing binary-artifact abstraction where applicable; carry package artifacts without becoming product-state authority | business authorization; authoritative publication lifecycle |
| Availability Node | receive/write/seed ciphertext; expose the publication result/evidence needed by the selected recovery contract | PostgreSQL credentials/state authority; invite/viewer data; plaintext CK; KEK; backend signing authority; declaration of business `P2P_READY` |
| P3 key release | consume the authoritative D3 Ready predicate as one release precondition | infer readiness from Hyperdrive key possession, synchronized bytes, queue status or Availability Node reachability |

### Option-specific execution ownership

| Concern | `O1 transactional-outbox` | `O2 durable-state-reconciler` | `O3 queue-primary-with-reconciliation` |
|---|---|---|---|
| durable work source | separate outbox intent committed with publication state | publication state row itself | authoritative publication row + queue for normal dispatch |
| normal claim | outbox claim/lease | publication-row claim/lease | queue delivery |
| recovery of lost pre-dispatch work | scan/reclaim pending outbox | scan/claim pending/retryable rows | reconciler scans authoritative pending state and re-enqueues/re-drives |
| recovery of unknown remote result | reopen/reconcile same outbox logical identity | row remains unknown/retryable until reconciled | reconciler operates independently of queue success/visibility |
| Ready writer | control-plane repository transaction after confirmed publication | same | same |
| queue/outbox/lease status authoritative for product Ready? | **No** | **No** — processing state is evidence/work state, not external success by itself | **No** |

### Fail-closed readiness predicate

Regardless of option, `P2P_READY` is true only when the authoritative PostgreSQL publication record durably proves all of the following semantic facts:

1. the logical package/publication lineage is current and not abandoned/replaced;
2. K1 encrypted package construction completed for that lineage;
3. the external publication result for that same lineage is confirmed, not merely dispatched;
4. the persisted publication identifiers/evidence correspond to that lineage;
5. no unresolved unknown-outcome condition remains for the current attempt/lineage.

If any fact is missing, stale, conflicting or unknown, the publication is **not P2P-ready** and D2 device-envelope release fails closed.

No option is selected by S4.

## D3-OWNER checkpoint

S1-S4 are complete. The owner must now select exactly one complete contract or request revised evidence:

- `O1 transactional-outbox`
- `O2 durable-state-reconciler`
- `O3 queue-primary-with-reconciliation`

The owner selection decides only the semantic durable-publication/recovery model described above. It does **not** accept ADR-044, authorize P2 source work, choose schema/API details, or resolve ADR-044 questions 4-7.

Until an explicit owner selection exists, S5, SYNC, D4 and P2 remain blocked.

## Acceptance

D3 closes only if:

1. S1-S4 produce cited neutral evidence with identical criteria and no recommendation language. **PASS.**
2. Owner explicitly selects one complete option at `ADR044-D3-OWNER`. **WAITING OWNER.**
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

## Approval checkpoint history

- 2026-09-05: parent presented at RRI 60 with Compact Approval Task Card v2.
- 2026-09-05: owner replied `aprobado`; S1-S4 authorized.
- 2026-09-05: S1-S4 completed with neutral evidence; execution stopped at `ADR044-D3-OWNER`.

Execution has stopped at the mandatory owner-selection checkpoint.
