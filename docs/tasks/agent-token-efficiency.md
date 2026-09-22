---
type: TaskList
title: "Tasks: task-aware agent token efficiency"
status: closed
slice: agent-token-efficiency
---

# Tasks: task-aware agent token efficiency

Plan: `docs/plan/agent-token-efficiency.md`.

## ATE-T1 — Make productive consumption a standing instruction

- Status: [x] Done (2026-09-22)
- Type: docs/policy-only
- Effort: S
- Complexity: Low (RRI 25)
- Depends on: none
- Authorization: owner request in this session; direct primary-agent Low route.
- Resource plan: deterministic search/generation/QA; primary agent for policy
  wording, with one bounded read-only editorial check; local-model calls n/a
  under the docs-only route; expected consumption low, no numeric budget given.

### Acceptance criteria

- The canonical guide requires task-aware productive token use, deterministic
  automation first where sufficient, and local AI preference within existing
  routing and capacity constraints.
- AGENTS and Claude instructions keep the rule visible; the generated Codex
  bootstrap matches `AGENTS.md` byte-for-byte.
- Cloud/context/delegation overhead and retries are bounded by useful progress;
  elevated consumption prompts an actionable human-assistance check.
- Estimates and unavailable telemetry are labeled honestly. Existing approval,
  model, local-precheck, review, verification, and fallback rules remain intact.

### Live phases

- completed — Analyze and score — Codex primary agent.
- completed — Update canonical instructions and summaries — Codex primary agent.
- completed — Verify, synchronize, and close — Codex primary agent.

### RRI evidence

```sh
python3 scripts/rri.py --touches AGENTS.md --touches AGENTS.override.md --touches CLAUDE.md --touches docs/playbooks/AGENT_WORKFLOW_GUIDE.md --touches docs/plan/agent-token-efficiency.md --touches docs/tasks/agent-token-efficiency.md --C 0 --T 0 --A 1 --X 3 --D 1 --K 1 --P 1 --penalty arch_decision
```

- C=0: prose only, no executable branching. F=3: six affected paths.
- T=0: existing bootstrap checks and deterministic documentation QA; no runtime
  behavior changes. A=1: clear preference, wording needs interpretation.
- D=1/K=1/P=1: contained internal instruction change; no runtime, public API,
  security, or data impact. X=3: one agent-instruction module and its policies.
- Process/policy decision penalty: `arch_decision` +12, retained explicitly.
- Calculator: L=0/I=1/Q=1/V=0; ICI=25; risk input=19; final RRI=25, Low/S.
- `honest-low-max: residual` — one coherent instruction change, already Low;
  splitting its canonical rule from the entry-point summaries adds no
  independently useful outcome.

### Handoff

Update only the six scored documentation paths. Keep detailed rules in the
workflow guide, compact reminders in AGENTS/CLAUDE, and regenerate the override
with `python3 scripts/generate-agents-override.py --write`. Run documentation
QA and diff/synchronization checks. Preserve unrelated worktree changes.

### Evidence and status synchronization

Record verification commands/results here. Synchronize this ledger and its
linked plan. No roadmap dependency, product slice status, or ADR changes.

Task-analysis review: n/a — docs/policy-only exemption.
Code-solution review: n/a — docs/policy-only exemption; no coverage certification
or development owner-final-verification gate applies.

### Closure evidence — 2026-09-22

- Added the canonical consumption procedure and analysis-step reference; both
  agent entry points carry the same compact reminder.
- Regenerated `AGENTS.override.md` using
  `python3 scripts/generate-agents-override.py --write`; `cmp AGENTS.md
  AGENTS.override.md` passed. Bootstrap size: 11,197 bytes, below the existing
  24,576-byte guard.
- `make qa-docs` passed: documentation consistency, behavioral coverage and
  BDD mapping checks, 8 + 7 + 24 existing validator tests, task completion
  evidence, roadmap drift, and OKF frontmatter.
- `git diff --check` passed. No new tests: prose-only change with existing
  deterministic checks; no runtime behavior changed.
- One bounded read-only editorial check identified existing mandatory retry,
  local-route eligibility, model-profile, and full-file packet constraints;
  all four were preserved explicitly. This was not a band-routed code review.
- No Ollama-backed role invoked: docs-only precheck exemption applies. Exact
  token/cost telemetry unavailable; no quantitative savings claimed.
- Plan and task ledger synchronized; no product/ADR/roadmap status changes.
