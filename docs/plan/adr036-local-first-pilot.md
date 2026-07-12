---
type: Plan
title: "Plan: ADR-036 local-first pilot — Stage 1 measurement + Stage 2 pilot"
status: proposed
slice: adr036-local-first-pilot
governed_by: [ADR-036]
---

# Plan: ADR-036 Local-First Pilot

## Objective

Execute the staged adoption defined in ADR-036 §10: measure the local model
stack on the real machine (Stage 1), run a bounded pilot on real Moderate-band
tasks (Stage 2), and produce a go/no-go decision against the promotion gates —
without changing any workflow policy until promotion is earned.

## Context

ADR-036 (Proposed) decides the policy: a local agentic implementation path for
RRI 26–40, a role-based local model stack (Qwen3.6-35B-A3B implementer /
Gemma 4 26B A4B reviewer / Gemma 4 12B fast lane), a fail-closed execution
boundary, and promotion/rollback gates. This slice is the work that validates
or refutes the five open questions recorded in the ADR. Nothing in this slice
modifies `AGENT_WORKFLOW_GUIDE.md`, `RRI_POLICY.md`, or
`HITL_AUTONOMY_POLICY.md`; policy propagation is a separate gated task (T10)
that only runs if the promotion gate passes.

## Design decisions

1. **Harness: thin bespoke wrapper, not an off-the-shelf agent.** ADR-036 open
   question 4 is resolved in favor of extending the `delegate-low-rri.py`
   lineage with a tool loop (`scripts/local-agent/`). Rationale: the ADR-034
   audit trail and the §3 boundary enforcement are non-negotiable acceptance
   requirements, and retrofitting them into an external harness costs more
   than a thin loop. The chat protocol is OpenAI-compatible against
   `OLLAMA_HOST`, so an external harness can be swapped in later behind the
   same boundary module if the bespoke loop underperforms.
2. **Measurement before machinery.** The inference/measurement tasks (T2–T4)
   run before the agentic runner is finished; if the wired-memory contingency
   fires (ADR-036 §6), the runner pilot proceeds directly with the demoted
   binding instead of discovering it late.
3. **Delegation-oriented granularization.** Every task in the ledger carries an
   **Executor tier** field (`gemma-developer | economy | balanced | primary`)
   so token-expensive agents are reserved for judgment work. Isolated new-file
   Python with pre-designed contracts goes to cheap executors; corpus
   selection, the security boundary, and the go/no-go synthesis stay with the
   primary agent or Balanced tier.

## Affected files

- `scripts/local-bench/` — new: measurement scripts + tests (T2, T3, T4)
- `scripts/local-agent/` — new: runner, boundary module, packet builder + tests (T6a–T6d)
- `scripts/gemma_local.py` — extended audit emission for runner records (T6c)
- `docs/evaluations/adr036-stage1-report.md` — new: measurement + benchmark report (T8)
- `.gitignore` — local bench artifacts (T2)
- No runtime crates, no mobile code, no CI-blocking gates are touched.

## Module dependencies

`local-bench` scripts depend only on the Ollama HTTP API and stdlib/psutil.
`local-agent` depends on `gemma_local.py` (audit emission) and git worktrees.
Nothing under `apps/` or `crates/` changes.

## Execution strategy

- **Economy-tier / local-delegable tasks (T2, T3, T4, T6c, T6d):** new isolated
  Python files with pre-designed contracts, fixtures, and stop conditions.
  Eligible for Gemma Developer packets when split to packet size per
  `LOW_RRI_LOCAL_MODEL_HANDOFF.md`; otherwise Economy cloud tier. The primary
  agent authors the contract and reviews per the standard gates.
- **Balanced-tier tasks (T6a, T6b):** the runner loop and the boundary module.
  T6b is security-critical and additionally requires primary-agent review
  regardless of reviewer routing.
- **Primary-agent tasks (T1 ops, T5 corpus, T7 orchestration, T8 synthesis,
  T9 pilot orchestration, T10 policy propagation):** judgment, operations on
  the physical machine, or policy-touching work that HITL policy keeps with
  the primary agent.
- Per-task RRI values in the ledger are **preliminary estimates**; recompute
  with `scripts/rri.py` at presentation time before execution, per the
  workflow guide.

## Verification

- Python: `python3 -m unittest` per new script test module.
- Deterministic docs gates: `bash scripts/check-doc-consistency.sh`,
  `python3 scripts/check_okf_frontmatter.py`.
- Stage 1 exit: `docs/evaluations/adr036-stage1-report.md` with the promotion
  gate table filled from measured data.

## Related

- `docs/adr/ADR-036-local-first-agentic-implementation-band.md`
- `docs/tasks/adr036-local-first-pilot.md` (task ledger)
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md`
