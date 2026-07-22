---
type: Plan
title: "Plan: Local-agent simple editing (Serena removal)"
status: active
---

# Plan: Local-agent simple editing (Serena removal)

## Objective

Replace the Serena/semantic-tool local-agent editing path with the simplest
tool contract that lets a local model actually edit a file, and remove Serena
entirely — code, tests, and workflow/policy references.

## Why this supersedes `local-agent-semantic-editing.md`

The semantic approach (Serena MCP + symbol tools + bounded anchors) was built
to solve "the file is too big to read into context". Measured against the
actual pilot target it does not hold:

- `apps/worker-runner/src/main.rs` is 1,622 lines ≈ **13,983 tokens**.
- The local implementer (`qwen3.6:35b-a3b`) has a **262,144-token** context.

The file fits in context ~18×. Serena solved a non-problem for this workload.

Worse, the restrictions built around Serena actively broke the model. Across
three full pilot reruns (`LASE-T6`, 2026-07-22) the model:

- explored symbols correctly for ~16–18 turns, then
- fell back to paging the file manually with `run_command` (`sed`/`head`/`wc`), and
- **never once called `apply_patch`/`write_file`/`finish`** — exhausting the
  turn budget with zero edits, every time.

Root cause was not turn budget: it was that the 400-line `read_file` cap forced
the model into unfamiliar symbol tools, and the 80-line/4096-byte `apply_patch`
budget made a real extraction edit impossible. We fenced the model out of the
one file it needed to change and it spent every turn trying to get back in.

Total machinery built for this: ~6,279 lines of Python across seven modules,
net result zero successful edits.

## Design

One bounded draft/test/repair loop. No language server, no symbol tools, no MCP.

Tool contract (plain-text JSON, same framing the loop already uses):

- `read_file {path}` — returns the whole file. **No line cap.** The model may
  read the file it is asked to edit.
- `write_file {path, content}` — creates a new file or overwrites an existing
  one. No size budget.
- `apply_patch {path, anchor, replacement}` — replaces exactly one occurrence
  of `anchor`. Keeps only the "anchor must match exactly once" safety; **no**
  line/byte budget.
- `run_command {argv}` — unchanged.
- `finish {}` — unchanged; triggers the gates.

Gates run **after** `finish`, not wrapped around every tool call, in this order
(all reused unchanged):

1. `scope_check.check_scope` — diff must stay within `allowed_paths`.
2. acceptance tests (the card's own commands, e.g. `cargo test -p ...`).
3. `organization_gate` — file-growth / composition-root / lint-suppression.

A `local-implementer` signature still requires all three to pass plus passing
acceptance tests. The audit record drops semantic-preflight / semantic-tool /
bounded-edit fields (there are none) but keeps scope, organization, acceptance,
and signature.

## What is kept

- `scripts/local-agent/scope_check.py` (+ tests) — unchanged.
- `scripts/local-agent/organization_gate.py` (+ tests) — unchanged.
- `run_local_task.py`'s hard-won robustness: malformed-bounce budget, total-turn
  budget, boundary handling, checkpointing, timeout-safe `run_command`. These are
  model-behavior fixes, not Serena-specific.
- The `O_NOFOLLOW` / atomic-create / unique-anchor filesystem safety, moved into a
  small Serena-free `runner_file_tools.py`.

## What is removed

- `scripts/local-agent/serena_mcp.py` (+ test, + `.cover`).
- `scripts/local-agent/runner_semantic_tools.py` (+ test).
- The Serena half of `runner_workflow_gate.py` (`classify_semantic_requirement`,
  `run_semantic_preflight`); `run_organization_gate` stays.
- `.serena/` config is left in place but unused; it can be deleted separately.
- All Serena/semantic-preflight references in the seven governing docs:
  `AGENT_WORKFLOW_GUIDE.md`, `HITL_AUTONOMY_POLICY.md`, `RRI_POLICY.md`, and the
  four `*local-agent-semantic-editing*` / `s-140` plan/task files.

## Tasks

1. `LASE2-T1`: add `runner_file_tools.py` (read/write/apply_patch, no caps) + tests.
2. `LASE2-T2`: rewrite `run_local_task.py` tool contract; gates after `finish`.
3. `LASE2-T3`: strip Serena from `runner_workflow_gate.py`; keep organization gate.
4. `LASE2-T4`: delete Serena modules/tests; update `run_local_task_test.py`.
5. `LASE2-T5`: remove Serena/semantic-preflight rules from the governing docs.
6. `LASE2-T6`: rerun `S-140-T2b-i` through the simplified runner.

## Verification

- Full `scripts/local-agent/` unit suite green after removal.
- No import of `serena_mcp` or `runner_semantic_tools` remains under `scripts/`.
- Live pilot produces a real diff within `allowed_paths`, passing
  `cargo test -p dubbridge-worker-runner`, passing organization gate, and a signed
  `local-implementer` audit record.
