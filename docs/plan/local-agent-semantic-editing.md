---
type: Plan
title: "Plan: Local-agent semantic editing"
status: superseded
---

# Plan: Local-agent semantic editing

> **Superseded (2026-07-22) by `docs/plan/local-agent-simple-editing.md`.**
> The Serena/semantic-tool approach below was removed: across three full pilot
> reruns the local model never produced a successful edit, because the target
> file (~14k tokens) fits ~18× inside the implementer's 262k-token context and
> the read/patch size caps built around Serena fenced the model out of the file
> it needed to change. The runner now uses a simple read/write/patch contract
> with gates run after `finish`. This document is kept for history only; do not
> act on it.

## Objective

Replace whole-file local-agent editing with symbol-aware context and bounded edits,
then reject structurally poor diffs before they can receive an implementer signature.

## Problem

`scripts/local-agent/run_local_task.py` currently exposes `read_file` and
`write_file`. On `apps/worker-runner/src/main.rs` (1,622 lines), the local model
read and attempted to regenerate the complete file. Turn 7 spent about four minutes
generating one `write_file` call and was interrupted before a controlled diff existed.

## Design

- Use Serena's Rust language-server backend for symbol overview, symbol lookup,
  references, diagnostics, and symbol-scoped edits.
- Keep `run_local_task.py` as the audited orchestrator entrypoint required by
  ADR-036; Serena is an implementation detail behind its tool boundary.
- Remove whole-file writes for existing source files. New files use a bounded
  create operation; existing files use symbol replacement or bounded patches.
- Add a deterministic organization gate. It checks changed code for file growth,
  composition-root leakage, and new broad lint suppressions. Passing tests alone
  is insufficient for a valid local implementer signature.
- Keep production modules at 250 lines or fewer where practical. Any new helper
  exceeding that budget must be split before closure. Tests live in separate files.

## Operative workflow contract

- Every RRI 26-55 local-first task that changes existing source code must pass a
  Serena health/index preflight in its disposable worktree before model execution.
- The task card, relevant symbol overview, selected symbol bodies, references,
  diagnostics, and nearby module patterns form the implementation context. Large
  source files are not injected wholesale.
- Existing source is changed only through symbol-scoped or explicitly bounded
  operations. Documentation, configuration, and genuinely new files are exempt
  from semantic lookup, but the exemption is recorded in the audit artifact.
- Serena startup, health, index, or semantic-tool failure makes the local route
  ineligible. The orchestrator escalates under ADR-036; it never restores broad
  full-file tools as a fallback.
- A successful `local-implementer` signature requires semantic-preflight evidence,
  bounded-edit metrics, passing organization checks, scope enforcement, and the
  task's independent acceptance commands.
- The canonical workflow guide and RRI/HITL policies must encode these rules before
  the live pilot is accepted, making the behavior mandatory for future tasks rather
  than dependent on prompt wording.

## Tasks

1. `LASE-T1`: install and index pinned Serena with committed project configuration.
2. `LASE-T2`: add a small Serena MCP adapter and focused adapter tests.
3. `LASE-T3`: replace whole-file runner tools with symbol/bounded-edit tools.
4. `LASE-T4`: add the isolated organization gate and focused tests.
5. `LASE-T5`: enforce semantic preflight, signature requirements, and canonical
   workflow policy.
6. `LASE-T6`: rerun `S-140-T2b-i` as the live pilot.

## Affected surfaces

- `.serena/project.yml` and `.gitignore`
- small modules under `scripts/local-agent/` from `LASE-T2` onward
- `scripts/local-agent/run_local_task.py` as a thin integration point
- `Makefile` and the local-agent workflow documentation

## Verification

- Serena project health check and Rust symbol-index smoke test.
- Unit tests per new module; no new monolithic test file.
- Existing local-agent test suite remains green.
- Canonical workflow docs state that semantic preflight is mandatory for existing
  source in RRI 26-55 and that failure escalates instead of broadening tools.
- Live pilot must produce a bounded diff, passing acceptance tests, structural gate
  evidence, and a `local-implementer` audit signature.
