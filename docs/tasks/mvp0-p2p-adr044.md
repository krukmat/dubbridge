---
type: TaskList
title: "Tasks: MVP0-P2P ADR-044 decision closure"
status: in_progress
slice: MVP0-P2P
plan: docs/plan/mvp0-p2p-first.md
---

# Tasks: MVP0-P2P ADR-044 decision closure

## Purpose

Resolve the architectural questions that keep ADR-044 `Proposed` and block
presentation of P2. Following ADR-038 Amendment 5, this ledger separates
evidence extraction, option comparison, owner selection, and mechanical
codification into independently verifiable Effort S leaves. The coherent
decision outcome remains a Med-high parent approval/review envelope: the split
does not turn an authorization-boundary choice into autonomous Low-band work.

The governing slice plan remains `docs/plan/mvp0-p2p-first.md`; the decision
inputs are in `docs/plan/mvp0-p2p-design-inputs.md` and the proposed decision
record is `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`.

## Task map

| Task | Objective | Status | Depends on |
|---|---|---|---|
| `ADR044-D1` | Parent envelope: resolve ADR-032 grant composition without delegating the architecture choice to an agent | Approved by owner 2026-09-05; S1 PASS, S2 ready | P1 PASS |
| `ADR044-D1-S1` | Extract frozen ADR-032 grant invariants and citations | Complete 2026-09-05; `make qa-docs` PASS | D1 approval satisfied 2026-09-05 |
| `ADR044-D1-S2` | Extract frozen P2 audience/key-release constraints and comparison criteria | Ready | S1 PASS |
| `ADR044-D1-S3` | Populate the neutral three-option decision matrix and expose tradeoffs | Blocked by S2 | S2 |
| `ADR044-D1-OWNER` | Owner selects reuse, bypass, or parallel audience authorization | Blocked by S3 | S3 |
| `ADR044-D1-S4` | Mechanically record the frozen owner selection in the audit record and ADR-044 | Blocked by owner selection | D1-OWNER |
| `ADR044-D2` | Resolve the content-key and device-envelope contract | Deferred; define and score separately | D1-S4 |
| `ADR044-D3` | Resolve publication/outbox state and recovery semantics | Deferred; define and score separately | D2 |
| `ADR044-D4` | Accept ADR-044 and propagate the accepted decision through canonical status documents | Deferred; define and score separately | D1-D3 plus any closure-blocking audit decision |

No task in this ledger authorizes P2 source work. P2 still requires its own
plan, full task definition, RRI, Compact Approval Task Card, and explicit HITL
approval after ADR-044 is accepted.

## ADR044-D1 — grant-composition parent envelope

- **Status:** `[~] Approved by owner on 2026-09-05`; S1 is complete and
  S2 is ready. The envelope remains open through S4.
  ADR-044 remains `Proposed`; D2-D4 and P2 stay separately gated.
- **Type:** architecture-decision envelope around docs/ADR-only Effort S
  leaves; no runtime or schema implementation.
- **Effort:** L (parent RRI 55, Med-high). Executable leaves: S (RRI 23–24,
  Low).
- **Objective:** give the owner cited, neutral evidence for choosing how P2
  audience authorization composes with ADR-032, then codify only the owner's
  frozen selection.
- **In scope:** reuse of `PlaybackGrant`, authorization bypass, and a parallel
  audience-authorization concept; fail-closed authorization, revocation,
  auditability, ADR-032 compatibility, one-viewer/one-device scope, and the
  authority required to release a device-wrapped content key.
- **Out of scope:** agent-selected recommendation; accepting ADR-044; choosing
  algorithms/envelope formats; defining schemas, routes, fields, or tokens;
  changing ADR-032; implementing P2/P3; modifying runtime behavior.
- **Acceptance criteria:**
  1. S1–S3 produce a cited, criteria-consistent matrix without normative
     recommendation language.
  2. The owner explicitly selects one option at D1-OWNER.
  3. S4 records that exact selection without adding implementation detail or
     changing ADR-044 from `Proposed`.
  4. ADR-032 remains unchanged and authoritative for review-time HTTP HLS.
  5. `make qa-docs` passes and no document claims ADR-044 or P2 is approved.
- **Phase-1 review:** `Task-analysis review: n/a -
  ADR/plan/task-ledger-only exemption.`
- **Phase-2 review:** `Code-solution review: n/a -
  ADR/plan/task-ledger-only exemption.`
- **Owner review:** the D1-OWNER decision checkpoint remains mandatory.
- **Verification:** `make qa-docs`; semantic consistency against ADR-032,
  ADR-043, the slice plan, and design inputs.
- **Handoff:** `ADR044-D1 — execute S1 through S3 after envelope approval,
  stop for D1-OWNER, then execute S4 using only the frozen owner selection.`

### Parent RRI report

**Platform:** dubbridge

| Variable | Score | Evidence | Confidence |
|---|---:|---|---|
| C cyclomatic | 1 | Non-development decision-weight heuristic: one architecture selection envelope, no runtime implementation | High |
| F files | 2 | `--touches` names 3 files | High |
| D domain | 4 | Audience authorization-boundary decision | High |
| T coverage | 0 | Documentation task; deterministic `make qa-docs` plus semantic consistency verification | High |
| A ambiguity | 0 | Options, criteria, leaf boundaries, owner checkpoint, and stop boundary are explicit | High |
| K coupling | 3 | Selection must compose ADR-032, invitation claim, key release, audit, and device binding | High |
| P impact | 1 | Records an ADR proposal while retaining `Proposed`; no runtime/public API change | High |
| X context | 3 | One cross-cutting audience-delivery boundary across several canonical documents | High |

**Base value:** 33.

**Penalties applied:** `arch_decision` (+12); `auth_security` (+10).

**Final RRI:** 55 → Med-high → Effort L → Balanced-to-Premium, thinking on.

**Gates:** explicit parent-envelope approval before any leaf; docs/ADR-only
phase-1 and phase-2 review exemptions; mandatory owner-selection checkpoint;
parent band retained through integrated closure. The honest Low-band pass
produced four coherent Effort S leaves and no agent-owned architecture choice.

Canonical command:

```bash
python3 scripts/rri.py \
  --touches docs/tasks/mvp0-p2p-adr044.md \
  --touches docs/audit/mvp0-p2p-adr044-d1-grant-composition.md \
  --touches docs/adr/ADR-044-p2p-audience-delivery-boundary.md \
  --C 1 --D 4 --K 3 --P 1 --T 0 --A 0 --X 3 \
  --penalty arch_decision --penalty auth_security
```

## Executable Effort S leaves

All four leaves are docs/ADR/task-ledger work and therefore run directly by
the primary agent; Low-band scoring does not make them eligible Qwen Developer
code patches. S1–S3 use the same conservative `auth_security` penalty because
their subject matter is authorization even though they cannot change the
authorization boundary.

### ADR044-D1-S1 — ADR-032 constraint register

- **Status:** `[x] Complete 2026-09-05`; citation inspection and
  `make qa-docs` PASS.
- **Effort:** S (RRI 24, Low).
- **Objective:** add a cited register of ADR-032 `PlaybackGrant` semantics and
  explicit non-semantics to the decision evidence workspace below.
- **Allowed path:** this ledger only.
- **Acceptance criteria:** every constraint cites ADR-032; facts are separated
  from inference; no P2 option is scored or recommended; status is updated in
  this ledger.
- **Verification:** inspect every citation against ADR-032; `make qa-docs`.
- **Handoff:** `Extract ADR-032 facts only; do not compare options or edit an ADR.`

### ADR044-D1-S2 — P2 criteria register

- **Status:** `[ ] Ready after S1`.
- **Effort:** S (RRI 24, Low).
- **Objective:** add cited P2 constraints and one shared comparison rubric for
  authorization authority, binding, expiry/revocation, wrapped-key release,
  ADR-032 compatibility, audit, and MVP device scope.
- **Allowed path:** this ledger only.
- **Acceptance criteria:** criteria trace to ADR-043, ADR-044, the slice plan,
  or design inputs; implementation details stay non-binding; no option is
  scored or recommended; status is updated here.
- **Verification:** citation/criteria consistency check; `make qa-docs`.
- **Handoff:** `Extract P2 constraints and freeze neutral criteria only.`

### ADR044-D1-S3 — neutral option matrix

- **Status:** `[ ] Blocked by S2`.
- **Effort:** S (RRI 24, Low).
- **Objective:** compare reuse, bypass, and parallel authorization against the
  frozen S1/S2 registers, exposing satisfied constraints, conflicts,
  assumptions, and unresolved tradeoffs.
- **Allowed path:** this ledger only.
- **Acceptance criteria:** every option uses identical criteria; every cell
  links to a frozen register entry; there is no winner/recommendation; the
  task stops at D1-OWNER and updates status here.
- **Verification:** matrix completeness and neutrality review; `make qa-docs`.
- **Handoff:** `Populate the matrix from frozen evidence; stop before selecting.`

### ADR044-D1-OWNER — explicit selection checkpoint

The owner selects exactly one matrix option or asks for revised evidence. No
agent may infer selection from scores, prior prose, or architectural preference.
Until that selection is explicit, S4, D2, D3, D4, and P2 remain blocked.


### ADR044-D1-S4 — mechanical decision codification

- **Status:** `[ ] Blocked by D1-OWNER`.
- **Effort:** S (RRI 23, Low).
- **Objective:** transcribe the exact owner-selected option into the audit
  record and ADR-044 while preserving `Proposed` status.
- **Allowed paths:** this ledger,
  `docs/audit/mvp0-p2p-adr044-d1-grant-composition.md`, and
  `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`.
- **Acceptance criteria:** the audit record includes the frozen registers,
  neutral matrix, exact owner selection, rejected alternatives, and remaining
  open questions; ADR-044 records no choice beyond the selection; no schemas,
  APIs, tokens, algorithms, or P2 source work are introduced; statuses are
  updated; `make qa-docs` passes.
- **Verification:** exact comparison to owner selection; semantic consistency;
  `make qa-docs`.
- **Handoff:** `Codify only the frozen D1-OWNER selection; keep ADR-044 Proposed.`

### Leaf RRI reports

S1, S2, and S3 each use this command and score:

```bash
python3 scripts/rri.py \
  --touches docs/tasks/mvp0-p2p-adr044.md \
  --C 0 --D 3 --K 1 --P 0 --T 0 --A 0 --X 2 \
  --penalty auth_security
```

Final RRI: **24 → Low → Effort S** (base 14 + `auth_security` 10).

S4 uses:

```bash
python3 scripts/rri.py \
  --touches docs/tasks/mvp0-p2p-adr044.md \
  --touches docs/audit/mvp0-p2p-adr044-d1-grant-composition.md \
  --touches docs/adr/ADR-044-p2p-audience-delivery-boundary.md \
  --C 0 --D 1 --K 1 --P 0 --T 0 --A 0 --X 2 \
  --penalty auth_security
```

Final RRI: **23 → Low → Effort S** (base 13 + `auth_security` 10). The
architecture choice is not an agent decision in S4; it is frozen at D1-OWNER.

## Decision evidence workspace

Parent-envelope approval was given explicitly by the repository owner in this
session on 2026-09-05: `aprobada task y subidivision`. That approval authorizes
the bounded S1-S4 sequence but does not select an architecture option at
D1-OWNER and does not authorize P2 source work.

### S1 ADR-032 constraints

Status: **complete** (`ADR044-D1-S1`); citation inspection and
`make qa-docs` passed on 2026-09-05.

#### Accepted facts

| ID | Frozen ADR-032 constraint | Citation |
|---|---|---|
| `A32-F1` | Prepared HLS is not exposed through raw object-storage keys. Access crosses a backend-owned grant boundary and storage keys remain an implementation detail. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § Prepared HLS is never exposed as raw object-storage keys |
| `A32-F2` | Before granting playback, that boundary validates S-120 readiness. For review playback it also validates caller authentication and org/project authorization; for audience playback it validates publication visibility; any playback token or signed URL remains scoped and expiring. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § Prepared HLS is never exposed as raw object-storage keys |
| `A32-F3` | The backend may realize delivery with signed object-store URLs, backend proxying, or CDN-backed signed URLs, but the stable contract is that the backend grants access. ADR-032 does not freeze one of those transport implementations. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § Prepared HLS is never exposed as raw object-storage keys |
| `A32-F4` | A client-followable manifest is produced without revealing storage layout. Segment access remains bounded by the same playback grant, either by rechecking it per request or by issuing short-lived scoped segment references. A manifest is not durable permission. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § Manifest and segment URLs are generated by the playback boundary |
| `A32-F5` | Playback fails closed when the prepared package is missing, malformed, processing, or failed. Review playback additionally follows authenticated review/workspace gates; audience playback follows the S-180 publication gate and ADR-030. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § Playback is readiness-gated and publication-gated |
| `A32-F6` | Creating or refusing a playback grant is governance-significant and durably traceable under ADR-018. Individual segment requests require traces/metrics but not necessarily one durable audit row each. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § Observability is split between durable grants and high-volume segment traffic |
| `A32-F7` | S-125 owns the server-side playback boundary only. It does not define public-player or consumer-player UX. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § S-125 owns HLS playback delivery |
| `A32-F8` | ADR-032 deliberately leaves CDN/public delivery undecided until S-180 or a later public-distribution slice defines the audience requirement. | `docs/adr/ADR-032-hls-playback-delivery-boundary.md` § Consequences, Negative / trade-offs |

#### Explicitly non-semantic for this decision

The accepted ADR-032 text does not define an invitation claim, a
viewer/device binding, ciphertext replication, content-key wrapping or unwrap,
or key revocation. It therefore supplies no accepted record shape or token
shape for those P2P concerns. This is a bounded absence observation over the
decision, consequences, and alternatives in
`docs/adr/ADR-032-hls-playback-delivery-boundary.md`; it is not a proposal to
add those meanings to ADR-032.

#### Inferences reserved for the neutral comparison

| ID | Non-binding inference from the accepted facts |
|---|---|
| `A32-I1` | The authorization question captured by `A32-F1`, `A32-F2`, `A32-F5`, and `A32-F6` can be compared independently from ADR-032's HTTP-oriented manifest and segment mechanics in `A32-F3` and `A32-F4`. |
| `A32-I2` | Reusing an authorization decision does not by itself require reusing every delivery field or transport mechanism; conversely, ADR-032 does not establish that a separate record would preserve its fail-closed and audit invariants. |

No P2 option is scored or preferred by this register.
