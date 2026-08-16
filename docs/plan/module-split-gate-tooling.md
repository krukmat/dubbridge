---
type: Plan
title: "Plan: Module-split gate tooling (ADR-040)"
status: implemented
slice: module-split-gate-tooling
adr: docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md
---

# Plan: Module-split Gate Tooling (ADR-040)

## Objective

Build the enforcement surface ADR-040 defined but left as manual/prose-only:
a fail-closed gate module, `scripts/local-agent/module_split_gate.py`, that
turns the per-module complexity-split routing decision (trigger check, hard
domain exclusion, disjoint-path validation, local/cloud tramo assignment,
repair-budget bookkeeping) into a deterministic, tested function instead of
an agent manually applying ADR-040 §§3-8 from memory on every candidate task.

## Why now (reversed prior recommendation)

An earlier pass on this same design question recommended deferring tooling
until a manual pilot showed real demand. That recommendation is withdrawn
based on two pieces of repository evidence gathered before writing this plan:

1. **Repo precedent ties ADR + gate script together, not sequentially.**
   `git log` shows `ADR-036` and `scripts/local-agent/run_local_task.py`
   landed the same day (2026-07-12), and `ADR-038` and
   `scripts/local-agent/med_high_gate.py` landed the same day (2026-07-26,
   task `T2` in `docs/tasks/med-high-local-refinement.md`). Every sibling ADR
   in this routing family shipped its gate script immediately, not after a
   manual-pilot interval. Leaving ADR-040 policy-only is the outlier, not the
   norm.
2. **The domain-exclusion check ADR-040 depends on has never been
   code-enforced anywhere in this repo.** `med_high_gate.py`'s
   `evaluate_route()` only reconciles Muse Glimmer's `GO_LOCAL`/
   `CLOUD_REQUIRED` recommendation against the primary receipt — it contains
   no auth/security/rights/migrations path matching at all. That's because
   ADR-038 Amendment 1 (2026-08-12) made the ADR-038 §6 exclusion list moot:
   Med-high never reaches local execution regardless of route, so nothing
   ever needed to check it. ADR-040 is the **first** mechanism where that
   exclusion list becomes load-bearing (a real local-execution path reopens
   for qualifying modules). A safety check that has never been operationalized
   in code, applied manually and repeatedly by an agent under normal task
   pressure, is a materially higher risk than the same check enforced by a
   tested function — this is the strongest argument for building the gate now
   rather than piloting the manual process first.

## Scope

### In scope

- `scripts/local-agent/module_split_gate.py` — pure, side-effect-free
  decision module (mirrors the shape of `scripts/local-agent/med_high_gate.py`
  and `scripts/local-agent/scope_check.py`): given a task capsule (allowed
  paths, per-path max CC, hard-exclusion path list, prior repair-attempt
  counts), returns a structured decision: `no_split` (with reason) or `split`
  (with the local tramo path set, cloud tramo path set, and remaining repair
  budget per tramo).
- `scripts/local-agent/module_split_gate_test.py` — unit tests covering every
  `HP-#`/`EC-#` case in the linked task ledger.

### Out of scope

- `run_local_task.py` / `run_med_high_task.py` integration (wiring the gate
  into the actual supervisor invocation flow) — follow-up task once the gate
  module itself is reviewed and merged, same sequencing `med_high_gate.py`
  (T2) used before `run_med_high_task.py` (T4) consumed it.
- Per-file CC measurement itself — reused from `scripts/rri.py`'s existing
  `measure_cc_<platform>` functions (already tested), not reimplemented here.
- The module-split capsule *format* is defined by ADR-040 §6/§11; this task
  implements the reader/validator for that format, not a new format.

## Design decisions

1. **Reuse, don't reimplement, CC measurement.** Import `scripts/rri.py`'s
   per-path measurer functions directly (e.g. `measure_cc_clippy`,
   `measure_cc_radon`) and the existing raw-CC → C-score mapping, called once
   per candidate module path. Do not shell out to the `rri.py` CLI per file
   (would require synthesizing dummy D/K/P/T/A/X values just to extract C) and
   do not duplicate the CC → score table.
2. **Hard-exclusion list lives in one place.** Encode the ADR-038 §6 / ADR-040
   §4 excluded-domain path patterns (auth, security, rights/consent/
   governance, migrations, unresolved-ADR, unbounded-scope markers) as a
   single named constant in `module_split_gate.py`, with a docstring pointing
   back to ADR-038 §6 as the source of truth, so a future ADR amendment to the
   list has exactly one place to change — not scattered path-matching in
   caller code.
3. **Fail closed on ambiguity.** Any capsule field missing, any path outside
   the task's own `allowed_paths`, any non-disjoint partition, or any
   unmeasurable CC (`--auto-cc` fallback signal) returns `no_split` with a
   typed reason — never guesses a partition. Mirrors `med_high_gate.py`'s
   `GateError` / fail-closed-to-`CLOUD_REQUIRED` pattern.
4. **Structural, not trust-based, hard-exclusion enforcement.** A module
   matching the hard-exclusion list is routed to the cloud tramo
   unconditionally, regardless of its measured CC — the exclusion check runs
   before and independently of the CC-based routing decision, so a low-CC
   auth-adjacent file can never slip into the local tramo through the CC path.
5. **Repair-budget bookkeeping is a pure counter, not an executor.** The gate
   module only tracks/returns the remaining-attempts state defined by
   ADR-040 §8 (local: 2 uniform; cloud: 1 + 1 tier escalation); it does not
   invoke `run_local_task.py`, a cloud model, or the integration gate itself —
   consistent with `med_high_gate.py`'s scope (decision only, supervisor
   scripts do the invoking).

## Module dependencies

```
scripts/rri.py (CC measurers, reused via import)
        |
        v
scripts/local-agent/module_split_gate.py (new)
        |
        v
(follow-up, out of scope here) run_local_task.py / run_med_high_task.py integration
```

## Related

- `docs/adr/ADR-040-per-module-complexity-split-implementation-routing.md`
- `docs/policies/RRI_POLICY.md` § Per-module complexity-split routing (ADR-040)
- `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md` (source of
  the hard-exclusion list this gate enforces)
- `scripts/local-agent/med_high_gate.py` / `med_high_gate_test.py` (structural
  precedent for this module's shape and test coverage)
