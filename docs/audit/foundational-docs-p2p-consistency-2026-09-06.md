---
type: Audit
title: "Foundational docs vs. MVP0-P2P / S-230 October documentation — consistency audit"
status: active
date: 2026-09-06
description: "Semantic cross-reference audit of the foundational documentation set against the new P2P and S-230 October go-live documentation, covering contradictions, gaps, and one hard test defect that make qa-docs cannot detect."
---

# Foundational docs vs. new P2P / S-230 documentation — consistency audit

**Date:** 2026-09-06
**Branch audited:** `feature/p2p-mvp-core` (HEAD `065560a`, plus the uncommitted
`S-230-T6p-a` presentation in `docs/tasks/s-230-poc-v1-digitalocean.md` and the
untracked `docs/audit/s-230-t6p-a-rri.md`)
**Scope:** read-only. No documentation, plan, ledger, ADR, or source file was
modified by this audit.

## Triage (owner-directed, 2026-09-06)

The owner asked for critical items only; the rest stay recorded here for a later
pass. This triage sets reporting priority — it does not change any finding's
content or evidence below.

| Priority | Findings | Rationale |
|---|---|---|
| **Critical** | F1, F2, F4 | F1 makes CI red on merge; F2 and F4 state, in canonical planning docs, the opposite of the frozen/landed state and will misdirect the next planner or reviewer. |
| **Critical — owner decision** | F5 | Not agent-resolvable: a stated architecture principle currently forbids a language C0 already froze. Needs a decision, not an edit. |
| **Deferred (minor)** | F3, F6, F7, F8, F9 | Real but non-blocking: stale cross-reference labels, missing index/map entries, and undocumented CI wiring. Batch them into a single later docs pass. |

## What was verified

**Foundational set:** `docs/architecture.md`, `docs/plan/roadmap.md`,
`docs/adr/README.md`, `DEVELOPMENT_REFERENCE.md`, `README.md`,
`README_AGENT_ORDER.md`, `AGENTS.md`, `CLAUDE.md`,
`docs/knowledge/README.md`.

**New set:** `docs/adr/ADR-043-*`, `docs/adr/ADR-044-*`,
`docs/plan/mvp0-p2p-first.md`, `docs/tasks/mvp0-p2p-first.md`,
`docs/plan/mvp0-p2p-p2-encrypted-publication.md`,
`docs/tasks/mvp0-p2p-p2-encrypted-publication.md`,
`docs/plan/mvp0-p2p-design-inputs.md`,
`docs/plan/s-230-poc-v1-digitalocean.md`,
`docs/tasks/s-230-poc-v1-digitalocean.md`, and the P2.T1 source landed on the
branch.

## Automated gate result (passes, and what it does not cover)

`make qa-docs` passes all eight checks: doc consistency, legacy `unit-v1` task
coverage, `behavior-v2` coverage, BDD mapping, task unit-coverage unit tests,
task completion evidence, roadmap drift, and OKF frontmatter.

Per `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § ADR change propagation`, that gate
deterministically guarantees only that cited ADR files exist, index↔file status
tokens agree, the index is complete, superseded ADRs name an existing successor,
and no code/migration comment cites a missing ADR. It explicitly does **not**
guarantee that a canonical document's *prose* still describes the current
decision. Every finding below sits in that uncovered space, except F1, which is
a code defect outside the gate's scope entirely.

Also verified correct, and recorded here so the next pass does not re-check them:

- `docs/adr/README.md` rows for ADR-043 and ADR-044 match their file status
  tokens (`Accepted`, `Accepted 2026-09-05`).
- The presented `S-230-T6p-a` gate `P2.T2g PASS + P2.T3d PASS` resolves against
  real leaf IDs (`docs/plan/mvp0-p2p-p2-encrypted-publication.md:171,185`).
- `docs/plan/roadmap.md` § Known planning gaps accurately declares the missing
  per-phase P3–P7 plan files rather than implying they exist.
- `docs/plan/mvp0-p2p-first.md` guardrail 8 correctly records ADR-044 as
  Accepted with D1/D2/D3 selections.

---

## F1 — Migration-count assertion is stale; CI will fail (hard defect)

**Severity:** blocking for merge.

`apps/cli/tests/migrate_test.rs:34` asserts:

```rust
assert_eq!(
    count, 31,
    "expected exactly 31 applied migrations, found {count}"
);
```

`infra/migrations/` now holds **32** files — P2.T1b added
`0032_create_p2p_publications_and_outbox.sql` (commit `ce471ce`). The test
early-returns when no database is reachable, so it passes locally without
Postgres and fails wherever `DATABASE_URL` is set, i.e. in CI.

This is the identical failure class as **CIRF-T1** under X28, where the same
assertion was stale at 29 against 31 migrations. It will recur: P2.T2e is
already C0-frozen to add migration `0033` for K1 persistence
(`docs/plan/mvp0-p2p-p2-encrypted-publication.md:171`).

**Recommended fix:** update the constant to 32 as part of the P2.T1 branch, and
consider whether an exact-count assertion is the right invariant at all given it
has now broken twice for the same reason.

---

## F2 — `docs/architecture.md` understates P2 progress and denies existing source

**Severity:** material contradiction.

`docs/architecture.md:50` (Delivery status table):

> P2.T0 PASS with Node.js/TypeScript Availability Node + mTLS (`AN-R1 + AN-A1`);
> **P2.T1 durable publication/outbox foundation is the next owner gate; no P2
> source implementation yet**

Both clauses are false as of 2026-09-06:

- `docs/plan/roadmap.md:157` records `P2.T1a-T1f Done and owner-approved as
  P2.T1; P2.C0 PASS on 2026-09-06`. T1 is not the next gate — it is closed, and
  C0 closed after it.
- P2 source exists on the branch: `crates/domain/src/p2p_publication.rs`
  (`d92c39a`), `crates/db/src/p2p_publication_repo.rs` (`00764b7`, `b1c4739`,
  `bf1ef39`, `344cfbf`), `infra/migrations/0032_*.sql` (`ce471ce`),
  `crates/db/tests/p2p_publication_repo.rs` (`1cf6708`), and
  `.github/workflows/p2p-t1.yml` (`c5470a7`).

The same paragraph at `docs/architecture.md:104-125` still frames the whole P2
data plane in future tense.

---

## F3 — `docs/architecture.md` cites X28 where X29 is the owning obligation

**Severity:** material contradiction.

`docs/architecture.md:49` and `:95` both defer P1.F3b's unmet device-proof
criteria into **X28**. The roadmap's own cross-cutting rows disagree:

- **X29** (`docs/plan/roadmap.md:262`) is the Android physical-device proof
  obligation, and states verbatim: *"**Extended 2026-08-27 by P1.F3b**, which
  has no device access either…"*.
- **X28** (`docs/plan/roadmap.md`) is the unrelated `main` CI-red item
  (CIRF-T1…T5), which has nothing to do with P1.F3b or Bare/Android.

This is the same mislabel that the superseded "Plano P2P" artifact carried and
that the consolidated go-live report corrected; the defect also lives in the
repository, not only in the artifacts.

---

## F4 — `docs/plan/mvp0-p2p-design-inputs.md` still declares ADR-044 Proposed and P2 blocked

**Severity:** material contradiction, highest practical risk of the docs set.

This file is `status: reference`, `governed_by: [ADR-043, ADR-044]`, and is the
document `docs/plan/roadmap.md` § Known planning gaps points to as the
transcribed design-input record for P3–P7 planning — i.e. the first thing the
next planner reads. It currently says:

- `:328-329` — *"the draft is `docs/adr/ADR-044-p2p-audience-delivery-boundary.md`
  (Proposed — **not accepted**, so P2 remains unpresentable)."*
- `:375` — *"`ADR-044…` — Proposed; blocks P2"*.

ADR-044 was accepted 2026-09-05 (`docs/adr/README.md:56`). P2 was not merely
presentable: T0 is PASS, T1a–T1f are Done and owner-approved, and C0 is PASS.

Two further stale claims in the same section:

- Its § "Open decisions blocking P2" enumerates decisions that are resolved —
  item 1 (grant composition) was decided as `O3 parallel` (D1), item 2
  (content-key algorithm, envelope, device-key storage) as `K1` (D2).
- It cites *"`docs/plan/mvp0-p2p-first.md` guardrail 9"* as the ADR
  precondition. Guardrails were renumbered: `mvp0-p2p-first.md:43` is now
  guardrail **8** ("ADR-044 is Accepted"), and guardrail 9 is the
  ciphertext-only rule.

---

## F5 — The Rust/Python language-boundary principle was never amended for the Availability Node

**Severity:** gap in a core architectural principle.

`docs/architecture.md` § Core principles admits exactly two backend languages:

> Rust owns API surfaces, orchestration, persistence boundaries, governance
> rules, and quality gates.
> Python is isolated to ML worker implementations where the ecosystem justifies
> an exception (`docs/python-exceptions.md`).

`CLAUDE.md` § Architecture states the same rule. P2.T0 froze `AN-R1`: a
dedicated **Node.js/TypeScript** service, and the C0 leaf table places it at
`apps/availability-node/` (`docs/tasks/mvp0-p2p-p2-encrypted-publication.md:121-124`
— `package.json`, `tsconfig.json`, `src/contract.ts`, `src/mtls.ts`,
`src/server.ts`, `src/hyperdrive_store.ts`, `test/*.test.ts`).

That is a third backend language, living inside the directory both
`docs/architecture.md` § Runtime surfaces and `DEVELOPMENT_REFERENCE.md` §
Workspace map frame as the Rust Cargo workspace's app surface
(*"Rust workspace defined in `Cargo.toml` (edition 2024)"*).

ADR-044:120 justifies the choice on its own terms (*"avoiding a Bare server
operational dependency"*), but no document reconciles it with the stated
principle, and there is no `docs/node-exceptions.md` counterpart to
`docs/python-exceptions.md`. The principle as written currently forbids what
C0 froze.

---

## F6 — New C0-frozen crate/app boundaries are absent from both foundational maps

**Severity:** gap.

| Boundary | Frozen by | `docs/architecture.md` | `DEVELOPMENT_REFERENCE.md` |
|---|---|---|---|
| `crates/p2p` | C0, T2a–T2g | absent from § Shared crates | absent from § Shared crates and § Directory reference |
| `apps/availability-node` | C0, T3a–T3d | named only in prose, not in § Runtime surfaces' app list | absent from § Apps and § Directory reference |
| `mobile/src/p2p/` | P1, ADR-043 | present (§ Runtime surfaces → Planned) | absent from § Directory reference |

`docs/architecture.md` already has a convention for this case — `connectors` and
`recorder` are listed as *"(planned, …)"* shared crates with their governing
ADRs — so the omission is inconsistent with the file's own practice, not a
question of whether unbuilt boundaries belong there.

---

## F7 — `DEVELOPMENT_REFERENCE.md`'s ADR index stops at ADR-035

**Severity:** gap.

The developer entry point (`DEVELOPMENT_REFERENCE.md:127-183`) groups ADRs into
four themed tables covering 006 through 035. The repository has **29** ADRs
through ADR-044. Missing: **036, 037, 038, 039, 040, 041, 042, 043, 044** —
nine, including the two Accepted product ADRs that govern the entire P2P
initiative.

`make qa-docs` enforces completeness of `docs/adr/README.md` only (which *is*
complete), so this drift is structurally invisible to automation. A developer
following `README_AGENT_ORDER.md` into `DEVELOPMENT_REFERENCE.md` currently gets
an ADR set that predates all P2P architecture.

---

## F8 — The roadmap's S-230 row carries none of the October P2P amendment

**Severity:** gap; asymmetric cross-linking.

`docs/plan/roadmap.md:156` (S-230 row) still ends *"`T6` (first deploy) is next,
unstarted"*, describes the slice as *"adds no new technology beyond Redis
(already in use)"*, and its Depends-on and Source cells name no P2P plan,
ledger, or ADR.

Meanwhile `T6p-a`–`T6p-d`, `T7p`, and `T9g` are **S-230 tasks** in
`docs/tasks/s-230-poc-v1-digitalocean.md`, gated on `P2.C0 PASS` and `P2 PASS`,
and `T6p-a` is presented and awaiting approval as of 2026-09-06. The amendment
appears only in the MVP0-P2P row (`:157`) and the blockquote (`:159-164`).

The linkage is one-directional: MVP0-P2P's Source cell lists both S-230 files;
S-230's lists neither P2P file. The slice row also now understates the slice —
`apps/availability-node` and Hyperdrive/Hyperswarm are new technology beyond
Redis.

---

## F9 — `.github/workflows/p2p-t1.yml` is undocumented

**Severity:** minor gap.

A new path-filtered CI workflow was added by `c5470a7`, triggering on the six
P2.T1 source paths. It is referenced in no document:
`DEVELOPMENT_REFERENCE.md` § QA gates lists only `make` targets and states that
`make qa-ci` is the blocking baseline CI enforces, which is now incomplete.
Nothing records who owns the workflow, whether it blocks merge, or when a
per-task verification workflow is retired.

---

## Observational (no action proposed)

- `docs/tasks/mvp0-p2p-p1-replication.md:2525` states P2 *"remains blocked on
  ADR-044 (`Proposed`)"*. This is a closed P1 ledger and legitimate as a
  point-in-time record, but it reads present-tense. The D2/D3 decision records
  (`docs/tasks/mvp0-p2p-adr044-d2.md`, `-d3.md`) and
  `docs/audit/mvp0-p2p-adr044-d3-publication.md` scope their equivalent
  statements to their own decision date and are correct as written.
- `README.md` is outward-facing product copy with no ADR, roadmap, or P2P
  reference. Correct today — nothing P2P ships yet. If the October controlled
  Android beta lands, its "Where things stand" table needs a row.
- `DEVELOPMENT_REFERENCE.md:85` describes `apps/gateway` citing `(ADR-024,
  ADR-031)`. ADR-024 is Superseded; naming both is defensible, but the
  propagation contract asks that every doc citing a superseded ADR be reviewed.

## Remediation grouping and status

| Group | Findings | Type | Note | Status |
|---|---|---|---|---|
| A | F1 | development | Code change; needs its own RRI and normal band route. Blocks CI. | **Resolved 2026-09-06** — see `docs/tasks/foundational-docs-p2p-consistency-fixes.md` T1. Migration-count assertion now derives its expected count from `infra/migrations/*.sql` instead of a hardcoded `31`. |
| B | F2, F3, F4 | docs-only | Factual corrections against verified state; workflow-exempt. | **F2 and F4 resolved 2026-09-06** — T2/T3 in the same ledger. F3 (X28/X29 mislabel) remains deferred per the owner's critical-only triage; tracked under Group D scope going forward. |
| C | F5 | ADR/policy | Needs an owner decision, not an edit: amend the principle, or record a language exception. Not agent-resolvable. | **Resolved 2026-09-06** — owner decided "permit a new stack" (Node.js/TypeScript, scoped to the Availability Node). `docs/architecture.md` and `CLAUDE.md` amended; `docs/node-exceptions.md` created. See ledger T2. |
| D | F3, F6, F7, F8, F9 | docs-only | Additive linkage / stale label; workflow-exempt. | **Deferred** — owner asked for critical items only on 2026-09-06; batch these into a later docs pass. |

Groups A and C are resolved; the F2/F4 portion of Group B is resolved.
Remediation ledger: `docs/tasks/foundational-docs-p2p-consistency-fixes.md`.
The remaining deferred items (F3, F6, F7, F8, F9) are documentation-consistency
work of the kind
`docs/policies/HITL_AUTONOMY_POLICY.md § Permitted without prior approval`
covers once explicitly authorized to fix inconsistencies.

## Related

- `docs/plan/roadmap.md` — canonical sequencing map audited here
- `docs/architecture.md` — foundational architecture document audited here
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § ADR change propagation` — the
  contract these findings are measured against
- `docs/audit/ci-red-findings-2026-09-01.md` — X28 record; F1 is the same
  failure class as its CIRF-T1
