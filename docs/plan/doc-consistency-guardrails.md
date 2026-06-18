---
type: Plan
title: "Plan: Documentation Consistency Guardrails (ADR change propagation)"
status: closed
---
# Plan: Documentation Consistency Guardrails (ADR change propagation)

**Roadmap position:** Cross-cutting governance tooling (a "G"-class item, sibling to
the G1–G4 ADR-traceability follow-ups recorded in
`docs/audit/2026-05-31-roadmap-adr-architecture-consolidation.md`). Not on the media
pipeline; unblocks nothing but protects every slice that touches canonical docs.

## Problem

`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` already requires syncing status artifacts
on task completion (step 7 + per-task discipline), but three gaps let canonical
documentation drift:

1. The rule is anchored to "completing a task", not to "changing an ADR". An ADR
   amendment outside a task ledger (e.g. the 2026-05-31 S3 replan) does not trigger
   it naturally.
2. There is no **propagation map** stating *which* canonical docs each kind of ADR
   change must touch. It relies on the agent's memory.
3. There is **no verification**. Drift is only caught when a human notices.

Observed drift (pre-existing, found during the S3 replan): `docs/adr/README.md`
listed ADR-020/021 as `Proposed` while the ADR files were `Accepted` since T0c, and
ADR-025 was missing from the index entirely. A deterministic parity check would have
failed the day the drift was introduced. (The index has since been corrected.)

## Objective

Make canonical-documentation consistency **structural, not best-effort**, mirroring
how the repo already enforces Rust QA (pre-push + Makefile + blocking CI). Three
layers:

1. **Propagation contract** (soft, in the guide): an explicit "this ADR change →
   these canonical docs" table plus a "Definition of done for an ADR change".
2. **Deterministic check** (`make qa-docs` + `scripts/check-doc-consistency.sh`):
   index↔file status parity, dangling-reference detection, superseded-successor
   existence, index completeness.
3. **Enforcement** (pre-push trigger on doc changes + blocking CI job), consistent
   with the guide's "mirror critical QA gates locally before remote" rule.

## Scope

### Included
- New guide section "ADR change propagation" + DoD checklist (Layer 1).
- New `scripts/check-doc-consistency.sh` and a `make qa-docs` target (Layer 2).
- `.githooks/pre-push` extended to run `qa-docs` when canonical docs change; a
  `qa-docs` job/step added to CI; `qa-docs` folded into `qa-ci` (Layer 3).
- Re-add the "Keeping this index in sync" note to `docs/adr/README.md` (now that the
  referenced check + guide section actually exist).

### Excluded (deferred)
- Full prose-level semantic consistency (no NLP / link-graph beyond ADR tokens).
- Auto-fixing drift (the check reports + fails; humans/agents fix in the same edit).
- Versioned ADR supersession graphs beyond a single `Superseded by ADR-YYY` line.
- Spell/style/markdown-lint concerns (out of scope; this is consistency only).

## Design Decisions

### Bash script, not a Rust bin
The existing QA surface (`.githooks/pre-push`, `Makefile`) is bash + make. A doc
check needs no compilation and runs faster as bash, matching the established style.
The four checks are line-oriented and tractable in bash + grep/awk.

### Check the *base* status word; allow scope annotations
The index `Status` column may carry an extra parenthetical (e.g.
`Accepted (scope: S3b live recording)`). The parity check compares the leading
status token (`Proposed` / `Accepted` / `Superseded` / `Deprecated`) against the
ADR file's `- **Status:**` leading token, so scope notes do not cause false
failures while a true `Proposed`↔`Accepted` mismatch still fails.

### Canonical doc set is explicit and bounded
The dangling-reference scan runs over a fixed list: `docs/adr/`, `docs/architecture.md`,
`docs/plan/`, `docs/tasks/`, `README.md`. Internal-only material (`docs/proposals/`,
`docs/audit/`) is excluded from enforcement but may be scanned in warn-only mode.

### Code and migration comments are in scope for dangling refs
This repo cites ADRs in source and SQL comments (e.g.
`crates/domain/src/recording.rs` "per ADR-020/022",
`crates/domain/src/artifact.rs` "ADR-021", the `0008`/`0009` migrations). Deletion
or renumbering of an ADR would orphan those comments invisibly if only docs were
scanned. The dangling-reference check therefore **also** scans `crates/**/*.rs`,
`apps/**/*.rs`, and `infra/migrations/**/*.sql` for `ADR-0\d\d` tokens and requires
each to resolve. This is the single most valuable check for the deletion case.

### Deletion is a governance event, not a silent file removal
The check enforces *referential* safety on deletion (no orphaned index row, no
dangling citation anywhere). The *policy* that an Accepted ADR must be superseded
rather than deleted lives in the Layer 1 contract and the global "ask before
deleting" rule; the script cannot infer intent, only catch broken references.

### Fail fast, report all
The script collects every violation and exits non-zero with a readable list, rather
than failing on the first — so one run surfaces the full drift set.

## Affected Files

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — new "ADR change propagation" section +
  DoD checklist; add `make qa-docs` to the local QA list.
- `scripts/check-doc-consistency.sh` (new) — the four checks.
- `Makefile` — `qa-docs` target; add to `qa-ci`.
- `.githooks/pre-push` — `DOCS_CHANGED` detector → run `make qa-docs`.
- `.github/workflows/ci.yml` — blocking `qa-docs` step/job.
- `docs/adr/README.md` — re-add the "Keeping this index in sync" note.

## Module Dependencies

None (pure docs + shell + CI config; no Rust crate graph impact).

## The propagation contract (content to land in the guide)

| ADR change | Must review/update in the same change |
|---|---|
| **New ADR** | `docs/adr/README.md` index row; `architecture.md` if it adds/alters a boundary; `roadmap.md` if it changes slice scope/deps; the affected `docs/plan/*` + `docs/tasks/*` |
| **Status change** (Proposed→Accepted→Superseded/Deprecated) | index `Status` column; every doc citing the ADR as authority |
| **Scope narrowed/broadened** | index scope annotation; `architecture.md`; `roadmap.md`; affected plan/tasks; `README.md` if the change is outward-facing |
| **Content/decision change** (the decision itself, not just status/scope) | every canonical doc whose prose describes that decision; this is **semantic** and not machine-verifiable — Layer 1 discipline + human review own it (Layer 2/3 only confirm references still resolve) |
| **Superseded by ADR-YYY** | both ADRs' `Status`; index; all docs citing the superseded ADR |
| **Deletion / renumbering** | see the deletion rule below — Accepted ADRs are **not deleted**; index row removed/renamed; every reference in the canonical set **and in code/migration comments** (`.rs`, `.sql`) updated in the same change |

**Deletion rule (aligns with the global "ask before deleting" policy).** An
`Accepted` ADR is part of the auditable decision record and must **not** be deleted;
it is marked `Superseded by ADR-YYY` or `Deprecated` instead. A `Proposed` ADR that
was never adopted may be deleted, but only after every reference to it (docs *and*
code/migration comments) is removed in the same change and a one-line reason is
recorded. Renumbering is treated as a delete (old number) plus a create (new number)
and must update both docs and code references atomically.

**Definition of done for an ADR change:** the ADR file, the index, every canonical
doc in the matching row above, **and every code/migration comment that cites the
ADR** are consistent in the *same* change, and `make qa-docs` passes.

### What this contract does and does not guarantee

- **Guaranteed (deterministic, Layer 2/3):** the ADR exists where it is cited, the
  index↔file status tokens agree, the index is complete, a superseded ADR names an
  existing successor, and no canonical doc or code comment cites a missing ADR.
- **Not guaranteed (Layer 1 + human review only):** that the *prose* of a canonical
  doc still semantically matches an ADR whose decision changed. Reference integrity
  is automatable; meaning is not. The propagation table exists to tell the author
  *which prose to re-read*, not to prove they updated it correctly.

## Lines Affected After Implementation

Tracked per-task in `docs/tasks/doc-consistency-guardrails.md`.
