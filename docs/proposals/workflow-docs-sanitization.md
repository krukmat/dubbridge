---
type: Proposal
title: "Workflow Documentation Sanitization"
status: Proposed
---

# Workflow Documentation Sanitization

## Purpose

The five documents that govern agent workflow — `CLAUDE.md`, `AGENTS.md`,
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/HITL_AUTONOMY_POLICY.md`,
`docs/policies/RRI_POLICY.md` — total **3,900 lines**. This proposal identifies
where that size is redundant prose rather than distinct rules, and defines a
sanitization pass that shortens the corpus **without changing any normative
behavior**: no gate, band, fallback chain, model binding, or approval rule
changes as a result of this work. Only exposition changes.

## Measured baseline

| File | Lines | Role |
|---|---|---|
| `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | 1869 | Declared highest authority |
| `docs/policies/RRI_POLICY.md` | 915 | RRI formula/rubric — delegated detail |
| `docs/policies/HITL_AUTONOMY_POLICY.md` | 562 | `Status: Scaffold` — approval rules |
| `AGENTS.md` | 260 | Task-presentation contract summary |
| `CLAUDE.md` | 294 (import-expands to include the two docs above) | Project entry point |

## Root cause

Every time a model binding changed (`qwen3.6:27b-q4_K_M` →
`nemotron-3.5-lightning:30b-a3b-q4_K_M` → `qwen3.8:27b-mlx`; reviewer roles
moving between Gemma / Muse Glimmer on 2026-08-11), the edit pattern was to
**append an explanatory aside at every place the binding is mentioned**,
rather than update the value once and record the change in one changelog
entry. Evidence:

- `qwen3.6:27b-q4_K_M` / `nemotron-3.5-lightning` history: mentioned with a
  near-identical explanatory clause in **12+ locations** across
  `AGENT_WORKFLOW_GUIDE.md`, `HITL_AUTONOMY_POLICY.md`, and `RRI_POLICY.md`.
- `2026-08-11` (the Muse Glimmer / Gemma reviewer-swap directive): appears
  **12 times** in `AGENT_WORKFLOW_GUIDE.md` alone, each with its own
  restatement of "this reverts the 2026-07-21 override...".
- The fallback chain `muse-glimmer:30b-q4_K_M → gemma4:26b-a4b-it-qat → D14`
  (and its 26–55 mirror) is spelled out in full prose, independently, in at
  least 8 places across the three normative docs.

This is not accidental duplication of unrelated content — it is the **same
sentence, restated**, each time treated as if it needed to be locally
self-contained. That instinct is reasonable for a single canonical location;
repeated at every mention it multiplies linearly with how often a concept is
referenced.

## Section-by-section map: HITL_AUTONOMY_POLICY.md vs AGENT_WORKFLOW_GUIDE.md

Both files declare `AGENT_WORKFLOW_GUIDE.md` as highest authority
(`HITL_AUTONOMY_POLICY.md` line 11: *"CLAUDE.md is authoritative on
conflict"*, and the guide's own header: *"It overrides CLAUDE.md ... and
AGENTS.md without exception"*). In practice `HITL_AUTONOMY_POLICY.md`
duplicates rather than delegates:

| HITL section | Lines | Overlaps with (Workflow Guide) | Verdict |
|---|---|---|---|
| Principle | 14–35 | Scattered across guide's role sections (ADR-037, Antares) | Compress to pointer |
| Always requires explicit approval | 37–47 | `AGENTS.md` Approval Boundary + guide's RRI gate table | Keep (this is the one place it's stated as a flat list) — trim prose |
| Per-task local-stack restart | 49–69 | Guide § Mandatory workflow before implementing, Step 0 (full procedure, ~95 lines) | Compress to pointer — HITL currently re-narrates the same PID/listener checks |
| Local delegation (RRI 0–25) | 71–118 | Guide § Model and thinking-mode selection + RRI_POLICY § Low RRI local delegation (full script contract, env vars, tagged-block format) | Compress to pointer — HITL's 10-step numbered list re-describes the same procedure as RRI_POLICY's, in different words |
| Local-first implementation (26–40) | 120–182 | Guide § Local-first and Architect-refined implementation routing | Compress to pointer |
| Post-repair-budget Low-band decomposition | 184–281 | Guide § Post-repair-budget Low-band decomposition (near-identical, ~95 lines each) | **This is the worst offender** — two ~95-line blocks say the same 9-step route almost verbatim. Keep full text in ONE place (guide, since it's cited as authority), reduce HITL to the worked-example pointer only |
| Med-high Architect-refined gate | 283–326 | Guide § same title, ADR-038 | Compress to pointer |
| Per-module complexity-split routing | 328–367 | Guide § same title, ADR-040 | Compress to pointer |
| Approval checkpoint wording | 369–377 | `AGENTS.md` Approval Boundary (identical wording) | Keep one line, drop restatement |
| Fallback model-selection checkpoint | 379–399 | Guide § Human-selected fallback checkpoint (ADR-039) | Compress to pointer |
| Permitted without prior approval | 401–412 | Not duplicated elsewhere — genuinely HITL-specific | **Keep as-is** |
| Safety rules | 414–420 | Guide § Testing and commit rules | Keep (short, distinct framing: rules *for autonomy*, not *for QA*) |
| Band-routed peer review | 422–486 | Guide § Band-routed peer review (two phases) — same table, same failure modes, restated | Compress to pointer, keep only the routing table (it's genuinely useful inline) |
| Gemma Reviewer availability | 488–523 | Guide § Gemma Reviewer / Muse Glimmer Reviewer § Availability | Compress to pointer |
| Reviewability budget escape | 525–539 | Guide § Reviewability budget gate | Compress to pointer |
| Review evidence override | 541–555 | Guide § Review artifact receipt and REVIEW-OVERRIDE lines + RRI_POLICY § Review evidence gate | Keep (this is the one place the `urgency`/human-only distinction is stated cleanly) — trim |

**Estimated reduction:** `HITL_AUTONOMY_POLICY.md` 562 → ~190 lines (drop
~370 lines of restated procedure, keep the approval-gate framing that is
genuinely this file's job to state).

## Internal repetition inside AGENT_WORKFLOW_GUIDE.md

Independent of the HITL overlap, the guide repeats itself:

1. **The model-binding changelog clause** (`qwen3.6:27b-q4_K_M` →
   `nemotron-3.5-lightning` → current; "2026-08-11 restructure") appears with
   slightly different wording at every mention of Gemma/Muse Glimmer/the
   Local Architect role (§ Model and thinking-mode selection, § Band-routed
   peer review, § Gemma Reviewer, § Local Architect, § Development task
   closure checklist — 5+ restatements).
2. **The fallback chain prose** (`muse-glimmer → gemma → D14` and its mirror)
   is spelled out as a full sentence at least 6 times instead of being stated
   once and referenced by band name afterward.
3. **The D14 provider-resolution rule** ("first use a responsive reviewer
   from a different provider... same-provider only as degraded fallback...")
   is restated in nearly identical language in § Band-routed peer review, §
   Context-isolated adjudicator, and twice more inside the Step 1-A/1-B/1-C
   closure checklist blocks.

**Fix:** Introduce one short **"Model binding history"** note (a 6–8 line
table: date, old binding, new binding, reason) placed once near the top of
`AGENT_WORKFLOW_GUIDE.md`, and replace every inline restatement with the
current value plus `(see Model binding history)`. Same treatment for the
fallback chain: state it fully once per band (already done in § Band-routed
peer review's table), and have every other mention just name the band
("RRI 0–25 fallback chain") instead of re-deriving it.

**Estimated reduction:** ~150–200 lines removed from the guide's ~1869,
without touching the RRI formula, bands, gates, or any procedural step.

## RRI_POLICY.md and AGENTS.md / CLAUDE.md

- `RRI_POLICY.md` has real unique content (formula, anchor rubric, penalty
  table, platform profiles) that must stay. It shares the same model-binding
  and fallback-chain repetition pattern as the guide (§ Local pipeline
  phase-1/phase-2 reviewer bindings, § Bands table footnotes) — same fix
  (point at the guide's single changelog note instead of re-deriving it).
  Estimated reduction: ~60–80 lines.
- `AGENTS.md` and `CLAUDE.md` are already positioned as summaries and mostly
  behave that way. The one drift: both carry their own copy of the Codex
  cloud-takeover table rows (`AGENTS.md` § Complexity And Model Guidance),
  which the guide's own text says should live in exactly one place (guide §
  Current Claude Code capability resolution: *"CLAUDE.md and AGENTS.md must
  not carry their own copy of the concrete model names"* — a rule the repo
  states but `AGENTS.md` doesn't fully follow for the Codex table). Fix:
  replace the inline table with a pointer, matching what the guide already
  requires for the Claude table one paragraph away. Estimated reduction:
  ~15 lines.

## What this proposal does NOT change

- No RRI band, gate, threshold, or penalty value.
- No model binding, fallback chain, or reviewer routing.
- No approval requirement (HITL gate stays exactly where it fires today).
- No section is deleted if any other file cites it by `§ Header Name` —
  headers that are cross-referenced (`§ Post-repair-budget Low-band
  decomposition`, `§ Local-first implementation`, `§ Local delegation (RRI
  0-25)`) are kept with the same title so the ~5 external `§`-anchored
  citations found in ADRs/tasks/plans keep resolving.

## Execution order (low risk → higher leverage)

1. Collapse `HITL_AUTONOMY_POLICY.md`'s duplicated procedural sections to
   pointers into `AGENT_WORKFLOW_GUIDE.md`, keeping section headers stable.
2. Add the "Model binding history" table to the guide; replace inline
   restatements with pointers to it.
3. Replace the guide's repeated fallback-chain prose with band-name
   references to the one canonical table.
4. Apply the same pointer treatment to `RRI_POLICY.md`'s repeated clauses.
5. Fix `AGENTS.md`'s inline Codex table to match its own stated rule.
6. Run `make qa-docs` (frontmatter, dangling-reference, ADR-index checks) to
   confirm no reference broke.

## Execution record

Executed in three passes. `make qa-docs` passes after each;
`AGENTS.override.md` regenerated with
`python3 scripts/generate-agents-override.py --write`.

| File | Before | After | Δ |
|---|---|---|---|
| `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` | 1869 | 1784 | −85 |
| `docs/policies/RRI_POLICY.md` | 915 | 874 | −41 |
| `docs/policies/HITL_AUTONOMY_POLICY.md` | 562 | 309 | −253 |
| `AGENTS.md` | 260 | 252 | −8 |
| `CLAUDE.md` | 294 | 294 | 0 |
| **Total** | **3900** | **3513** | **−387 (−10%)** |

Pass 3 additionally created `docs/audit/agent-workflow-binding-history.md`
(53 lines) as the new home for the history evicted from all three governing
documents.

**Pass 1** — steps 1, 2, 4, 5 of the execution order above: HITL procedural
sections collapsed to pointers; § Model binding history added to the guide
(it grew by design, absorbing 4 scattered restatements); RRI_POLICY and
`AGENTS.md` pointed at it.

**Pass 2** — step 3, the reviewer-chain deduplication. The chain
(`muse-glimmer → gemma → D14` / its 26–55 mirror) was stated in full 6 times
in the guide and 5 times in RRI_POLICY. It is now stated once per file — the
guide's § Band-routed peer review and RRI_POLICY's § Local pipeline
phase-1/phase-2 reviewer bindings — and named by band everywhere else.
Specific collapses: the guide's 4 table footnotes (†‡§¶) merged into one
canonical chain list; § Interaction with existing gates bullets 2–4 → 1;
§ Report line contract's per-band token derivation → 1 bullet; § Gemma
Reviewer intro, § When it runs, § Availability → pointers; Step 1-A/1-B
directive preambles deleted (their headers already name the chain); the three
D14 "Balanced tier, cross-provider first" paragraphs → 1 line each pointing at
§ Context-isolated adjudicator; Step 1-C's `1g` evidence template → a delta
against Step 1-B's; the Antares pre-closure block → a pointer to its role
section; RRI_POLICY's `††` footnote, Moderate/Med-high binding paragraphs, and
Qwen-vs-Gemma paragraph → pointers to its own canonical section.

**Pass 3** — owner directive: *the workflow guide must contain directives
only; anything historical does not belong there.* History was **moved, not
deleted**, to the new append-only
`docs/audit/agent-workflow-binding-history.md` (local-model binding lineage,
dated routing/process directives, retired tooling paths), and the guide now
opens with a `Directives only` note stating that rule. Evicted from the
guide: the routing-override blockquote and the § Model binding history table
(→ replaced by a `## Local-model role bindings` table stating current
bindings only); the 2026-07-15 Moderate-override and 2026-07-26 ADR-038
adoption dates; the `(owner directive 2026-08-16)` / `(owner directive,
2026-07-22)` / `(adopted 2026-06-04)` heading and paragraph stamps; the
`**Adoption note:**` on RRI superseding CC scoring; both `Subsumed by RRI`
"only the input has changed" notes; the retired Serena/semantic-tool
parenthetical; the ADR-037 `**Retired scoped exception:**` paragraph
(→ replaced by the standing boundary it produced); the Antares `As of the T5
promote decision` / T4-pilot framing; and the T1 RRI self-scoring anecdote
in § Communication format (→ replaced by the rule it illustrated).
The same rule was then applied to the other two governing documents, so all
three carry directives only. Evicted from `HITL_AUTONOMY_POLICY.md`: the
2026-07-15/07-21/07-26 routing-override chronology paragraph, the
`(owner directive 2026-08-16)` heading stamp and its `**Owner directive,
2026-08-16:**` paragraph opener, the `ADR-038 (2026-07-26)` date, the
`Owner directive, 2026-08-16, formalized as ADR-040` attribution, and three
`unchanged` / `no longer` / `now` framings that only make sense against a
prior state. Evicted from `RRI_POLICY.md`: the `ADR-038 (2026-07-26)
replaces it` date, the `(owner directive, 2026-08-11; ADR-038 otherwise
unchanged)` and `Fallback (owner directive, 2026-08-11)` stamps, the
`Owner directive, 2026-07-22` openers on the target-file size gate and the
GEG-1 review evidence gate, the paragraph narrating what the 2026-07-21
reviewer override was and that Gemma "reverts" to its prior role
(→ replaced by the standing rule: the reviewer is never the band's own
implementer model and never the cross-vendor peer), the ADR-037 scope note's
retirement narrative, the `not only to the sub-40 tier the original ledger
validator checked` clause, and the stale `run_local_task.py (918 lines as of
2026-07-22)` measurement.

Cross-references updated in `RRI_POLICY.md` (lineage pointer → audit doc) and
`HITL_AUTONOMY_POLICY.md` (dated guide anchor → undated). Historical task
ledgers and proposals that cite the old dated anchors were left alone: they
are themselves records of when the work happened.

### Evaluated and deliberately not cut

- **`CLAUDE.md`** — already a thin summary using `@imports`; it does not
  exhibit the restatement pattern.
- **The `### Peer Reviewer evidence` template in Step 1-B** — duplicated
  structure is intentional for a fill-in template.
- **The "changes only who authors the code, never the RRI/band/reviewer/
  Reflection count/approval gate" disclaimer** in § Post-repair-budget
  Low-band decomposition and § Per-module complexity-split routing — one
  sentence each, scoped to a different sub-route, and load-bearing against
  exactly the misreading each route invites.
- **Vendor-table verification dates (`2026-08-09`) and the `2026-08-31`
  GPT-5.4 Codex retirement** — not repo history. The first is the validity
  stamp the "re-verify if older than roughly two months" rule reads; the
  second is a forward-looking vendor fact.
- **The PPR enforcement note and the ADR-040 tooling-status note** — these
  state what is and is not yet built, which conditions the directive
  ("invoke the gate manually and say so"). Current state, not history.

## Related

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`
- `AGENTS.md`, `CLAUDE.md`
