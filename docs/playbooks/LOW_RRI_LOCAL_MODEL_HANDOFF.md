---
type: Playbook
title: "Low-RRI Local Model Handoff"
governs: "local model delegation for Low-band RRI tasks"
---

# Low-RRI Local Model Handoff

## Purpose

Define the mandatory handoff protocol for delegating **Low-RRI (0–25)** work to a
local model through Ollama. This playbook is operational: it explains how to scope
the work, how to write the packet, how to review the result, and when to shrink the
task or escalate.

## When to use this playbook

Use this playbook when all of the following are true:

- the task is in the **Low (0–25)** RRI band;
- the task is suitable for local delegation;
- the task is a **simple code patch** or similarly narrow test patch;
- the task can be expressed as a small, concrete change with clear acceptance
  criteria.

Best-fit tasks:

- pure development tasks with narrow, mechanical scope;
- tightly scoped mechanical code edits;
- single-file or very small multi-file code/test updates;
- predictable additions such as boilerplate or focused tests.

Poor-fit tasks:

- broad documentation rewrites;
- policy, workflow, ADR, roadmap, plan, or task-ledger edits;
- large ledger rewrites or structure-heavy edits;
- mixed work types in one pass;
- tasks that require wide editorial reinterpretation.

## Mandatory rules

1. Delegate **step by step**. One objective, one narrow change, one file or one
   tightly scoped change at a time when possible.
2. Prefer **pure code/test development work** or tightly scoped mechanical edits.
3. Do not delegate broad doc rewrites, policy/workflow changes, large ledgers, or mixed work types in a
   single handoff.
4. Instructions must be **simple, concrete, and replacement-oriented**.
5. The orchestrator must validate not only that the patch applies, but that
   **structure, scope, and meaning are preserved**.
6. If a step fails, reduce scope before retrying.
7. Do not expand the packet after failure; make the next attempt smaller.

## Step-by-step process

1. Isolate the smallest useful task.
2. Choose the exact allowed file set.
3. Write the packet with simple instructions and explicit stop conditions.
4. Run the local delegation.
5. Validate scope, format, and application result.
6. Review the semantic result in the actual files.
7. Run the required verification commands.
8. If the result is weak or structurally risky, retry with a smaller scope.
9. If the repair cycle fails, escalate instead of forcing a larger retry.

## Packet-writing rules

- State the goal in one or two short sentences.
- List the **exact files** that may be modified.
- Say what must **not** be changed.
- State the required output contract explicitly: tagged text blocks with complete
  file contents, never JSON and never a unified diff.
- **Always include the current content of every file to be modified** in a
  `## Current file content` section. The local model has no filesystem access —
  without this it will hallucinate the file structure and produce a rewrite that
  bears no resemblance to the real file.
  - For **small files** (under ~400 lines / ~3000 estimated tokens): include
    the complete file. Use `--mode full-file` (the default).
  - For **large files** (400+ lines): do **not** ask Gemma to emit the complete
    file — the model's output token ceiling (~8 192 tokens on current hardware)
    makes full-file regeneration unsafe and has caused silent file destruction
    (see `docs/evaluations/large-file-delegation-2026-06-21.md`). Use
    `--mode before-after` instead: include only the exact BEFORE block (the
    function or region to change) in the packet. Gemma emits only the
    replacement AFTER block; the wrapper performs a literal
    `replace(before, after, 1)` on the original file.

**Mode selection rule (orchestrator-owned):**

```python
estimated_file_tokens = len(file_content) // 4
if estimated_file_tokens > 3000:   # approximately 400 lines
    mode = "before-after"
else:
    mode = "full-file"
```

The wrapper (`scripts/delegate-low-rri.py`) never infers the mode itself — it
fails closed if `--mode before-after` is supplied without `--target-path` or
`--before-file`. Mode selection is the orchestrator's decision.

Example invocation for a large file:

```bash
scripts/delegate-low-rri.py packet.md \
  --mode before-after \
  --target-path apps/api/src/routes/workspace.rs \
  --before-file /tmp/workspace-before.txt \
  --allow-path apps/api/src/routes/workspace.rs \
  --apply \
  --out result.json
```

The `--before-file` content must be copied verbatim from the current target
file. The BEFORE block must match exactly once — the wrapper rejects ambiguous
matches before building any diff.
- **Show the exact block to replace and the exact replacement block** as code
  fences, not as prose descriptions. Prose instructions (“remove the closure”,
  “simplify the error handling”) are ambiguous to a small model; literal before/after
  blocks are not.
- **Verify every symbol mentioned in the packet before writing it.** If the packet
  asks to remove an import, confirm first that no other site in the file still
  references that symbol after the change. If the packet asks to delete a helper,
  confirm it has no other callers. Mistakes here produce compile errors that the
  model cannot catch.
- Prefer concrete edits such as:
  - add one note;
  - replace one bullet;
  - insert one short paragraph after a named heading;
  - create one new file with a named structure.
- Use those document-oriented patterns only inside code-adjacent files such as
  tests or narrowly scoped developer-facing comments, not for repository policy or
  planning artifacts.
- Prefer expected wording or expected sections over open-ended prose requests.
- Avoid “improve”, “clean up”, “rewrite”, or other broad editorial instructions.
- Avoid mixing convention updates, structural rewrites, and sync work in one packet.

## Orchestrator review checklist

Before accepting the result, verify all of the following:

- the changed files are within the allowed scope;
- the response matches the tagged-block contract with no extra text;
- the patch applies cleanly;
- the file structure is preserved;
- the change actually matches the requested objective;
- no unrelated sections were rewritten;
- the resulting document or code still reads coherently in context;
- **read the unified diff in `result.json` line by line** — confirm only the
  lines described in the packet changed; any deletion or addition outside that
  scope is a scope violation even if the patch applied cleanly;
- **run `cargo build --workspace` (or the equivalent for the platform)** and
  confirm zero new errors and zero new warnings before marking the task done.

## Failure and retry rules

- If the model returns an out-of-scope path, reject it and retry with a smaller,
  more explicit packet.
- If the tagged-block format or application fails, do not broaden the packet;
  simplify it.
- If the patch applies but the semantic result is destructive, reject it and retry
  with a smaller target.
- Use at most the bounded repair cycle allowed by the governing policy.
- After a failed repair cycle, escalate instead of substituting a larger manual
  rewrite under the guise of local delegation.

## Anti-patterns

- Delegating a whole ledger rewrite in one pass.
- Mixing wording cleanup, structural reorganization, and status-sync in one request.
- Giving a local model a large context dump when a short instruction would do.
- Accepting a patch because it applies without checking structural preservation.
- Asking the local model to infer the desired scope from broad background context.

## Relationship to repo policy

- Workflow authority: `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- RRI authority: `docs/policies/RRI_POLICY.md`
- This playbook adds the **operational handoff discipline** for Low-RRI local-model
  work; it does not replace the governing workflow or RRI policy.
