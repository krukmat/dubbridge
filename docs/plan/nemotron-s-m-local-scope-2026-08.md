---
type: Plan
title: "Plan: Nemotron local-developer scope — S and M only"
status: Complete
slice: nemotron-s-m-local-scope-2026-08
adr: docs/adr/ADR-036-local-first-agentic-implementation-band.md
---

# Plan: Nemotron Local-Developer Scope — S and M Only

## Objective

Apply the owner-approved temporary routing matrix: Nemotron is the only default
local developer for Low/S (RRI 0–25) and Moderate/M (RRI 26–40); Med-high/L
(RRI 41–55) remains cloud-only after its ADR-038 evidence gate.

## Design decisions

1. Keep reviewer bindings independent: Muse Glimmer/Gemma/D14 remain reviewers;
   this plan changes developer routing only.
2. Low/S uses the existing bounded tagged-patch wrapper, with Nemotron as its
   default and no silent substitute developer model.
3. Moderate/M keeps `run_local_task.py` and its existing Nemotron default.
4. Med-high/L retains the ADR-038 refinement and primary-receipt validation, but
   a `GO_LOCAL` decision produces a cloud handoff bundle without launching a
   local process.

## Ordered tasks

1. `NEM-SM-T1` — record the owner decision in ADR-036/ADR-038 and the governing
   workflow/policy documents. Docs-only.
2. `NEM-SM-T2` — bind the Low/S delegation wrapper to Nemotron and fail closed
   rather than substituting another developer model. Depends on T1.
3. `NEM-SM-T3` — make the Med-high supervisor emit the cloud handoff on an
   otherwise-valid `GO_LOCAL` decision, without launching Nemotron. Depends on T1.
4. `NEM-SM-T4` — synchronize the completed routing evidence and verify the
   end-to-end configuration. Depends on T2 and T3.

**Progress (2026-08-12):** T1–T4 are complete. The approved matrix is active:
Nemotron is limited to eligible Low/S and Moderate/M implementation; Med-high/L,
Complex, and XL routes are cloud-only.

## Affected modules

- `scripts/delegate-low-rri.py` and `scripts/delegate_low_rri_test.py`
- `scripts/local-agent/run_med_high_task.py` and its unit tests
- ADR-036, ADR-038, the RRI/HITL/workflow policies, and the Low-band handoff
  documentation

## Owner authorization

On 2026-08-12 the owner confirmed the exact routing matrix in this plan: S and
M use Nemotron; L and XL use cloud. This is the bounded implementation approval
for these four tasks; it does not authorize a different model, band, or product
runtime change.
