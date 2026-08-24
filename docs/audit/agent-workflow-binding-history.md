---
type: Audit
title: "Agent workflow binding and directive history"
status: open
---
# Agent workflow binding and directive history

Append-only historical record for the three governing agent documents:
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md`,
`docs/policies/HITL_AUTONOMY_POLICY.md`, and
`docs/policies/RRI_POLICY.md`.

Those three state only the directives currently in force. Every superseded
binding, retired override, and dated owner directive behind a current rule is
recorded here instead, so they stay rule sets and this file stays the audit
trail. Adding a row here is part of the same change that alters a rule.

## Local-model role bindings

| Date | Role | Was | Became | Reason |
|---|---|---|---|---|
| 2026-07-21 | RRI 26–55 phase-1/phase-2 reviewer | Gemma | `qwen3.6:27b-q4_K_M` (Local Architect model) | Scoped override |
| ADR-036 Amendment 2 | RRI 26–55 local implementer | — | `qwen3.6:27b-q4_K_M` | Reassigned from reviewer to implementer role |
| ADR-036 Amendment 3 | RRI 0–25 / 26–40 local implementer | `qwen3.6:27b-q4_K_M` | `nemotron-3.5-lightning:30b-a3b-q4_K_M` | Rebind |
| 2026-08-11 | RRI 0–25 phase-1/phase-2 reviewer | Gemma | Muse Glimmer (`muse-glimmer:30b-q4_K_M`), Gemma intermediate fallback | Local model stack restructure (ADR-037 Amendment 1) |
| 2026-08-11 | RRI 26–55 phase-1/phase-2 reviewer | `qwen3.6:27b-q4_K_M` | Gemma (`gemma4:26b-a4b-it-qat`) reverts to this role, Muse Glimmer intermediate fallback | Retires the 2026-07-21 override; Qwen cannot simultaneously implement and independently review the same band (ADR-036 §5) |
| 2026-08-11 | Local Architect / Complex Analyst (ADR-037) | `qwen3.6:27b-q4_K_M` | `muse-glimmer:30b-q4_K_M` | ADR-037 Amendment 1; role no longer doubles as a phase-1/phase-2 reviewer in any band |
| 2026-08-16 (ADR-036 Amendments 3/4/7) | RRI 0–25 and RRI 26–40 local implementer | `nemotron-3.5-lightning:30b-a3b-q4_K_M` | `qwen3.8:27b-mlx` | Both bands share one implementer model family |
| 2026-08-24 | RRI 26–40, RRI 41–45 after `GO_LOCAL`, and ADR-040 local tramos | `qwen3.8:27b-mlx` | `nemotron-3.5-lightning:30b-a3b-q4_K_M` | Owner-authorized routing correction; Low remains Qwen and RRI 46–55 whole-task routing remains cloud-only |

## Routing and process directives

"Now stated in" names where the resulting rule lives; `Guide` is
`AGENT_WORKFLOW_GUIDE.md`, `RRI` is `RRI_POLICY.md`, `HITL` is
`HITL_AUTONOMY_POLICY.md`.

| Date | Directive | Effect | Now stated in |
|---|---|---|---|
| 2026-06-04 | RRI adopted as the canonical scoring method | Replaced single-axis cyclomatic-complexity scoring as the input that selects the model tier and autonomy gate; workflow policy, so no ADR was required | Guide § RRI — canonical scoring method; RRI |
| 2026-07-15 | Moderate (RRI 26–40) local-first implementation made operative by owner override | Adopted ahead of the original ADR-036 pilot promotion gate | Guide § Local-first and Architect-refined implementation routing; HITL § Local-first implementation |
| 2026-07-21 | Med-high (RRI 41–55) extended to local-first by owner override | Superseded five days later by ADR-038 | ADR-038 |
| 2026-08-11 | Phase-1/phase-2 reviewer restructure (local model stack) | Established the two local reviewer chains in the bindings table above and retired the 2026-07-21 reviewer override; ADR-038 otherwise unchanged | Guide § Band-routed peer review; RRI § Local pipeline phase-1/phase-2 reviewer bindings |
| 2026-07-22 | Target-file size gate (500 lines) for local-first delegation | Owner directive | Guide § Handoff prompt format; RRI § Target-file size gate for local-first delegation |
| 2026-07-22 | Review artifact receipt / `REVIEW-OVERRIDE:` lines (GEG-1) | Owner directive; band-agnostic, superseding the sub-40-only ledger validator. Sections dated before `REVIEW_EVIDENCE_CUTOVER_DATE` in `scripts/check-task-unit-coverage.sh` keep the pre-GEG-1 behavior | Guide § Review artifact receipt and REVIEW-OVERRIDE lines; RRI § Review evidence gate |
| 2026-07-26 | ADR-038 replaced the Med-high local-first override with the Architect-refined single-attempt gate | No whole-task local attempt or repair at 41–55 | ADR-038; Guide § Local-first and Architect-refined implementation routing; RRI § Med-high Architect-refined single-attempt handling |
| 2026-08-06 | Antares Security-Specialist Advisor promoted to active workflow touchpoints (T5 decision) | Promoted **without** a completed calibration run against the fixed thresholds (File F1 ≥ 0.30 macro-averaged per watchlisted CWE, true-negative rate ≥ 0.70) or a completed 30-day pilot window; the T5 record states that gap as an owner-directed deviation, not as evidence the thresholds were met | `docs/tasks/antares-security-specialist-advisor.md` § T5 Decision record; Guide § Antares Security-Specialist Advisor |
| 2026-08-16 | Post-repair-budget Low-band decomposition | Once a whole-task local repair budget is exhausted, Low-band decomposition replaces cloud escalation as the default next step. Validated end-to-end on `S-150-T2c-iv-c` | Guide § Post-repair-budget Low-band decomposition; HITL § Post-repair-budget Low-band decomposition |
| 2026-08-16 | ADR-040 per-module complexity-split routing | Amends ADR-036 and ADR-038 Amendment 2 | ADR-040; Guide § Per-module complexity-split routing; HITL and RRI § Per-module complexity-split routing |

## Retired tooling paths

| Path | Outcome | Reference |
|---|---|---|
| Local-agent Serena / semantic-tool editing surface | Never produced a successful edit; replaced by the `write_file` / `apply_patch` / `finish` tool contract | `docs/plan/local-agent-simple-editing.md` |
