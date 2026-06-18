---
type: ADR
title: "ADR-033: Adopt the Open Knowledge Format (OKF) for repository knowledge"
status: Accepted
---

# ADR-033: Adopt the Open Knowledge Format (OKF) for repository knowledge

- **Status:** Accepted
- **Date:** 2026-06-18
- **Deciders:** DubBridge platform team
- **Closes:** the "knowledge has no machine-readable classification" gap surfaced
  while reviewing the OKF adoption concept

## Context

DubBridge already runs as an agent-first repository: `CLAUDE.md`, `AGENTS.md`, the
canonical guides under `docs/playbooks/` and `docs/policies/`, the ADR set, and the
slice plan/task ledgers are the working memory that agents load before acting. That
knowledge is plain Markdown organized by directory convention.

This works, but three structural limits are now concrete pain, not theory:

1. **No per-document classification an agent or tool can read.** Agents load whole
   files (`CLAUDE.md`, `AGENT_WORKFLOW_GUIDE.md`) and navigate directories by
   convention. There is no way to ask "give me every `ADR` with `status: Accepted`"
   or "every `Plan` that is still active" without reading the bodies.
2. **Lifecycle is invisible to machines.** `docs/plan/` holds ~30 slice plans;
   several are delivered (S-080, S-030) and intermixed with active and planned ones.
   Nothing marks a plan active vs. closed in a form a script or an agent can filter.
3. **The ADR index is maintained twice.** `docs/adr/README.md` carries a status
   column that must be kept in lockstep with each ADR's `- **Status:**` prose line.
   The `check-doc-consistency.sh` gate enforces parity, but the *authoring* burden is
   manual and duplicated, and a task ledger's governing ADRs live only in prose.

Google Cloud published the **Open Knowledge Format (OKF)** v0.1 on 2026-06-12: a
vendor-neutral specification that represents project knowledge as a directory of
Markdown files with YAML frontmatter. `type` is the only required field; one concept
per file; standard Markdown links; consumers must tolerate unknown keys. It is the
formalization of exactly the pattern this repository already uses informally.

DubBridge is unusually well positioned to adopt it: unlike a typical knowledge
vault, this repo already has **referential-integrity enforcement** —
`check-doc-consistency.sh` (ADR index parity, status tokens, dangling ADR refs,
superseded→successor existence), `check-roadmap-drift.sh`, and the
**ADR change-propagation contract** in `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`.
The missing piece is a *general document-metadata contract*; OKF supplies the
standard, and the existing gate machinery can enforce it.

This is a documentation-and-process decision, in the same category as the RRI
adoption (a workflow policy, not a runtime architecture decision). It is recorded as
an ADR because the user requested it and because it establishes a binding,
hard-to-reverse contract once ~50 files carry frontmatter and `make qa-docs`
enforces it.

## Decision

### 1. Adopt OKF frontmatter in-place on governed Markdown documents

Add an OKF-compliant YAML frontmatter block to the Markdown documents that
constitute agent knowledge. `type` is required; `title`, `description`, `tags`, and
`timestamp` are recommended. Domain-specific keys (`status`, `slice`, `plan`,
`governed_by`, `supersedes`, `superseded_by`) are permitted extensions, consistent
with OKF's rule that producers may add fields and consumers must tolerate unknown
keys.

### 2. The `type` vocabulary is closed and aligned to the directory structure

The vocabulary is defined canonically in `docs/knowledge/README.md` (created by the
implementation). Because DubBridge's `docs/` tree is already cleanly partitioned,
`type` maps to location, which makes validation trivial:

| `type` | Location |
|---|---|
| `ADR` | `docs/adr/ADR-*.md` |
| `Playbook` | `docs/playbooks/*.md` |
| `Policy` | `docs/policies/*.md` |
| `Plan` | `docs/plan/*.md` (except the roadmap) |
| `Roadmap` | `docs/plan/roadmap.md` (singleton) |
| `TaskList` | `docs/tasks/*.md` |
| `Architecture` | `docs/architecture.md` |
| `Proposal` | `docs/proposals/*.md` |
| `Audit` | `docs/audit/*.md` |
| `Prompt` | `docs/prompts/*.md` |

Ephemeral and template documents (`docs/daily/*`, `*/TEMPLATE.md`) and pure index
READMEs are out of scope. `.feature` BDD files are non-Markdown and are deferred (an
optional later wrapper), not migrated.

### 3. Frontmatter is additive and non-destructive

Existing prose is not removed. In particular, each ADR keeps its
`- **Status:**` line, on which `check-doc-consistency.sh` already depends. Frontmatter
**mirrors** that line; it does not replace it. Every current gate keeps passing
unchanged. The migration adds metadata; it deletes nothing.

### 4. The OKF layer is machine-verifiable, integrated into `make qa-docs`

A new validator (`scripts/check_okf_frontmatter.py`, wired into `qa-docs`) enforces:

- every in-scope file has a frontmatter block with a `type` from the closed
  vocabulary, and the `type` matches the file's location;
- for ADRs, frontmatter `status:` equals the prose `- **Status:**` token (preventing
  the two layers from drifting);
- for `TaskList` and `Plan`, any `governed_by:` / `supersedes:` ADR reference
  resolves to an existing ADR (reusing the dangling-reference guarantee).

This is the decisive difference from a descriptive-only adoption: in DubBridge OKF
becomes an **enforced contract**, not optional metadata that silently rots.

### 5. No separate knowledge vault; in-place is the single source of truth

DubBridge has no Obsidian vault and no wikilink dialect, so the coexistence problem
that complicates vault-based adoptions does not exist here. Frontmatter lives in the
source documents. A separate `docs/knowledge/` directory is used only for the
vocabulary definition and (later, if adopted) wrappers for non-Markdown artifacts —
never as a parallel copy of Markdown that already exists.

### 6. Scope boundary: documentation and process only

OKF adoption changes **no** runtime, crate, or API boundary. `docs/architecture.md`
needs no structural change for this decision (only its own frontmatter). This ADR
governs how knowledge is classified and validated, nothing about how the platform
executes.

## Consequences

**Positive**

- Agents and tooling can filter knowledge by `type`, `status`, `slice`, and
  `governed_by` instead of loading and scanning whole files.
- Plan lifecycle (active / planned / closed) becomes machine-readable, ending the
  "which of these 30 plans still matters?" problem.
- The ADR index status column and a task's governing ADRs become checkable against
  frontmatter, reducing the dual-maintenance burden in the ADR-propagation contract.
- New architectural and planning documents are born OKF-compliant, so the contract
  does not decay after the one-time migration.
- The knowledge layer becomes portable and vendor-neutral; it is not tied to Claude
  Code or any single agent runtime.

**Negative / trade-offs**

- A one-time migration touches ~50 documents to add frontmatter.
- One more validator to maintain and keep aligned with CI.
- Frontmatter and prose can drift; mitigated by the parity check in Decision §4 and
  by treating frontmatter updates as part of the ADR-propagation contract.
- `type`-vocabulary drift between contributors; mitigated by a *closed* vocabulary
  enforced by the validator, not a convention.

## Alternatives considered

- **Keep the status quo (no frontmatter).** Rejected: no machine-readable
  classification, no lifecycle filtering, and the ADR index maintenance burden keeps
  growing as the ADR set grows.
- **A separate `docs/knowledge/` vault that points at sources with `resource:`**
  (the vault-based adoption pattern). Rejected for Markdown: it creates dual
  maintenance and mostly-empty pointer files. DubBridge has no Obsidian vault to
  justify the separation. Retained only as the mechanism for *non-Markdown*
  artifacts if those are ever brought in.
- **Make frontmatter the sole source of truth and regenerate the prose status lines
  and the ADR index from it.** Rejected for v1: high blast radius on the existing
  grep-based gates. Revisit once the additive layer is stable and trusted.
- **Adopt OKF without enforcement (descriptive metadata only).** Rejected:
  descriptive-only metadata drifts. DubBridge's entire value proposition here is the
  *enforced* contract enabled by the existing gate machinery.

## Related

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — the ADR change-propagation contract this
  ADR extends with a frontmatter obligation.
- `docs/policies/RRI_POLICY.md` — precedent for adopting a binding process contract
  without a runtime architecture change.
- `scripts/check-doc-consistency.sh` — the existing referential-integrity gate the
  new OKF validator complements.
- `docs/plan/okf-knowledge-format-adoption.md` — the migration plan governed by this
  ADR.
- `docs/tasks/okf-knowledge-format-adoption.md` — the decomposed, gated task ledger.
- OKF v0.1 specification — GoogleCloudPlatform/knowledge-catalog (`okf/SPEC.md`).
