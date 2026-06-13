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
- the task can be expressed as a small, concrete change with clear acceptance
  criteria.

Best-fit tasks:

- pure development tasks with narrow, mechanical scope;
- tightly scoped mechanical edits;
- single-file or very small multi-file updates;
- predictable additions such as boilerplate, tests, or isolated doc fixes.

Poor-fit tasks:

- broad documentation rewrites;
- large ledger rewrites or structure-heavy edits;
- mixed work types in one pass;
- tasks that require wide editorial reinterpretation.

## Mandatory rules

1. Delegate **step by step**. One objective, one narrow change, one file or one
   tightly scoped change at a time when possible.
2. Prefer **pure development work** or tightly scoped mechanical edits.
3. Do not delegate broad doc rewrites, large ledgers, or mixed work types in a
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
- Prefer concrete edits such as:
  - add one note;
  - replace one bullet;
  - insert one short paragraph after a named heading;
  - create one new file with a named structure.
- Prefer expected wording or expected sections over open-ended prose requests.
- Avoid “improve”, “clean up”, “rewrite”, or other broad editorial instructions.
- Avoid mixing convention updates, structural rewrites, and sync work in one packet.

## Orchestrator review checklist

Before accepting the result, verify all of the following:

- the changed files are within the allowed scope;
- the patch applies cleanly;
- the file structure is preserved;
- the change actually matches the requested objective;
- no unrelated sections were rewritten;
- the resulting document or code still reads coherently in context.

## Failure and retry rules

- If the model returns an out-of-scope path, reject it and retry with a smaller,
  more explicit packet.
- If the patch format or application fails, do not broaden the packet; simplify it.
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
