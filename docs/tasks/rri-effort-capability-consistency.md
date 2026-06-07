# Tasks: RRI ↔ Effort ↔ Capability Consistency

Governing plan: `docs/plan/rri-effort-capability-consistency.md`
Governing guides: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/RRI_POLICY.md`,
`AGENTS.md`
Related guardrail: `make qa-docs` (must stay green)

## Status Legend
- [ ] Not started · [x] Done · [~] In progress · [!] Blocked

## Default model recommendation (per RRI band of this slice)
Slice RRI ≈ 30 (Moderate) → Effort M → tier Balanced → thinking Off → gate: confirm
the consistency check exists (`make qa-docs`). Each subtask below is individually S.
- Codex: `GPT-5.2-Codex`
- Claude Code: `Claude Sonnet 4`

This slice is docs/policy-only, so the development-task requirements (HP/EC behavioral
examples, Mermaid per task, unit-coverage certification) are not applicable; the
acceptance criterion is referential/semantic consistency plus a green `make qa-docs`.

Build order: **T1 → T2 → T3 → T4 → T5**.

---

## Task T1 — Canonical crosswalk in `RRI_POLICY.md`

**Effort:** S · **Complexity:** Low · **Depends on:** nothing
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Extend the existing "Bands, autonomy gates, and model tiers" table into the single
authoritative crosswalk that co-locates every RRI-derived value, resolving F4 and F5
at the source.

### Scope
- Add an `Effort` column and split the single `Tier` column into `Capability (Codex)`
  and `Capability (Claude Code)` columns. Final columns:
  `RRI band | Label | Effort | Capability (Codex) | Capability (Claude Code) | Thinking | Gate`.
- Populate per the crosswalk: S/Economy (0–25), M/Balanced (26–40),
  L/`Balanced → Premium` for both vendors (41–55), L/Premium (56–70),
  XL/Premium (71–85, 86–100, >100).
- Add a one-line note above the table: "Effort, capability, thinking, and gate are
  each derived in parallel from the RRI band; never derive one output from another."

### Acceptance criteria
- The crosswalk has the seven columns above and every band row is filled.
- Codex and Claude Code both show `Balanced → Premium` at 41–55 (F4 resolved here).
- Effort bands align with the RRI band boundaries (S=0–25, M=26–40, L=41–70, XL=71+).
- `make qa-docs` exits `0`.

### Files affected
- `docs/policies/RRI_POLICY.md`

### Status: [x] Done — 2026-06-07

Files affected:
- `docs/policies/RRI_POLICY.md` — lines 177-201: replaced the 5-column band table
  with a 7-column canonical crosswalk (RRI band | Label | Effort | Capability Codex |
  Capability Claude Code | Thinking | Gate). Added parallel-derivation note above
  the table. Collapsed the separate vendor-ID resolution table into a prose pointer
  to the guide.

Validation: `make qa-docs` exits `0`.

---

## Task T2 — Guide references the crosswalk + explicit parallel-derivation rule

**Effort:** S · **Complexity:** Low · **Depends on:** T1
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Stop the guide from restating partial, divergent tables; point it at the single
crosswalk (T1) and state the derivation topology so the `RRI → Effort → capability`
chain misconception (F1) cannot recur.

### Scope
- Replace the standalone `RRI→Effort` table (`AGENT_WORKFLOW_GUIDE.md:192-197`) with a
  short rule + reference to the canonical crosswalk in `RRI_POLICY.md`.
- Replace the Step 2 capability mapping table (`:308-314`) with a reference to the same
  crosswalk (keep the surrounding "Subsumed by RRI" prose and agent-resolution rules).
- Add the explicit sentence: "Effort, capability tier, and autonomy gate are each
  derived in parallel from the RRI band; never derive capability or gate from Effort."

### Acceptance criteria
- The guide contains no second copy of the RRI→Effort or RRI→capability mappings that
  can drift from `RRI_POLICY.md`; both point to the canonical crosswalk.
- The parallel-derivation rule is present and unambiguous.
- No dangling reference; `make qa-docs` exits `0`.

### Files affected
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`

### Status: [x] Done — 2026-06-07

Files affected:
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — lines 188-194: removed the standalone
  RRI→Effort band table, replaced it with a pointer to `RRI_POLICY.md` §Bands, and
  added the explicit parallel-derivation rule.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — lines 303-309: removed the standalone
  Step 2 RRI→capability table and replaced it with a pointer to the canonical
  crosswalk while preserving the surrounding "Subsumed by RRI" guidance.

Validation: `make qa-docs` exits `0`.

---

## Task T3 — Fix the Effort scale XL contradiction

**Effort:** S · **Complexity:** Low · **Depends on:** nothing (parallel to T1/T2)
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Remove the contradiction where XL is defined by "external tooling" while the rules
forbid encoding toolchain pain in Effort (F2).

### Scope
- Rewrite the XL row's "Agent reasoning" and "Example" cells
  (`AGENT_WORKFLOW_GUIDE.md:186`) to describe RRI-driven very-high reasoning/risk, not
  toolchain pain. Use a non-toolchain illustrative example.
- Optionally add one line clarifying the S/M/L/XL descriptions are illustrative and the
  RRI band is authoritative for assignment.

### Acceptance criteria
- The XL row no longer attributes the level to external tooling / toolchain conflicts.
- The Effort scale is consistent with the anti-toolchain rule at `:200-203`.
- `make qa-docs` exits `0`.

### Files affected
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`

### Status: [x] Done — 2026-06-07

Files affected:
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — lines 181-186: rewrote the `XL` row
  so it describes RRI-driven very-high reasoning/risk and uses a non-toolchain
  illustrative example.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — lines 188-197: added the clarification
  that the S/M/L/XL descriptions are illustrative and that the RRI band is
  authoritative for assignment, keeping it consistent with the anti-toolchain rule.

Validation: `make qa-docs` exits `0`.

---

## Task T4 — Rename the "Complexity label" vocabulary collision

**Effort:** S · **Complexity:** Low · **Depends on:** nothing (parallel to T1/T2/T3)
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Eliminate the ambiguity where "Complexity label" names two different scales (F3).

### Scope
- Rename the CC table's column (`AGENT_WORKFLOW_GUIDE.md:268-273`) from "Complexity
  label" to "Cyclomatic (C) label".
- Keep the RRI-band table's label column but ensure it reads as the RRI band label.
- Update the presentation-format block (`:362-366`) so `Complexity score → <label>`
  unambiguously refers to the cyclomatic/decision-weight label.

### Acceptance criteria
- No two tables share the bare header "Complexity label" for different scales.
- The presentation format is unambiguous about which label is meant.
- `make qa-docs` exits `0`.

### Files affected
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`

### Status: [x] Done — 2026-06-07

Files affected:
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — lines 268-273: renamed the CC table
  header from `Complexity label` to `Cyclomatic (C) label` to separate it from the
  RRI band vocabulary.
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — lines 354-358: clarified the
  presentation-format block so `Complexity score` explicitly refers to the
  cyclomatic/decision-weight label.

Validation: `make qa-docs` exits `0`.

---

## Task T5 — Verify and sync status artifacts

**Effort:** S · **Complexity:** Low · **Depends on:** T1, T2, T3, T4
**Recommended model:** Codex `GPT-5.2-Codex` / Claude `Sonnet 4`

### Objective
Prove the change is consistent and leave no stale status doc.

### Scope
- Run `make qa-docs` and confirm exit `0`.
- Re-read the two edited canonical docs end-to-end to confirm no residual contradiction
  among RRI, Effort, and capability.
- Mark the plan progress ledger and these tasks `[x]` with files/lines affected.
- Check whether `docs/plan/roadmap.md` or any other status doc must reference this slice
  (cross-cutting workflow-policy change); update if so.

### Acceptance criteria
- `make qa-docs` exits `0`.
- Plan + tasks ledgers updated with evidence (files + line ranges).
- No remaining cross-doc contradiction among the three concepts.

### Files affected
- `docs/plan/rri-effort-capability-consistency.md`
- `docs/tasks/rri-effort-capability-consistency.md`
- `docs/plan/roadmap.md` (only if a reference is warranted)

### Status: [x] Done — 2026-06-07

Files affected:
- `docs/plan/rri-effort-capability-consistency.md` — lines 102-108: finalized the
  progress ledger so all tasks in the slice are marked complete.
- `docs/tasks/rri-effort-capability-consistency.md` — lines 178-208: recorded the
  final verification outcome and status-artifact sync for the slice.

Validation:
- `make qa-docs` exits `0`.
- Re-read `docs/policies/RRI_POLICY.md` lines 171-199 and
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` lines 179-358; no residual contradiction
  remains among RRI, Effort, capability, thinking, or autonomy-gate derivation.
- `docs/plan/roadmap.md` does not reference this slice or require additional sync for
  this docs-only policy consistency change.

---

## Agent handoff prompt (for delegation)

```
You are implementing RRI ↔ Effort ↔ Capability consistency for DubBridge.

Repo: /Users/matiasleandrokruk/Documents/dubbridge
Plan: docs/plan/rri-effort-capability-consistency.md
Tasks: docs/tasks/rri-effort-capability-consistency.md

Work one approved task at a time in order: T1 -> T5. After each task:
1. Run `make qa-docs` and confirm exit 0.
2. Mark the task [x] and record files/lines affected in plan + tasks.
3. Report a summary and WAIT for approval before the next task.

Hard invariants:
- Docs/policy only. No code, no ADR, no CI change. Do not add/remove ADR references.
- Single source of truth: the canonical crosswalk lives in RRI_POLICY.md; the guide
  references it, it does not copy it.
- Topology rule: Effort, capability, thinking, and gate derive in PARALLEL from the
  RRI band. Never derive one output from another.
- Effort band boundaries must match RRI band boundaries (S 0-25, M 26-40, L 41-70,
  XL 71+).
- Resolve the Codex 41-55 disagreement to `Balanced -> Premium` for both vendors.
- Do NOT edit CLAUDE.md (documented override, out of scope).
- User-facing comms in Spanish; docs in English.
```
