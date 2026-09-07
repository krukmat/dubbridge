---
type: Audit
title: "Agent Workflow Guide Detail Archive"
status: reference
---

# Agent Workflow Guide Detail Archive

`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` is the highest-authority, native-instruction
source loaded into every Codex session via `AGENTS.override.md` (see
`docs/plan/agents-override-sync.md`). To keep that per-session token cost down, this
file holds the verbose rationale, worked examples, vendor-citation lists, and
provenance narratives that were trimmed out of the live guide on 2026-08-20 — every
operative rule, gate, table, checklist, and format template stays in the guide
itself; only supporting detail moved here. Nothing below changes or supersedes the
guide; it explains and evidences it.

## § Step 0 — Ollama warm-up command and supporting rationale

This is a supporting excerpt, not a complete relocation of Step 0. The active
guide retains the operative restart procedure: task-boundary restart, protection
for another task's active runner, replacement PID/listener checks, the ordered
resource-recovery protocol, required evidence, and the applicable-role scope.
This archive preserves the warm-up command, context guidance, and explanatory
rationale that support that active procedure.

Before the first Ollama-backed action of every task that will invoke a local model,
restart Ollama even when the current server appears healthy, then verify that the
local stack and the models the task will actually invoke respond correctly under
production generation parameters (`think=false` where applicable, the repo's
default `num_predict`/`num_ctx` from `gemma_local.py`). A silent `done_reason:
"length"` with empty `content` (thinking-mode exhausting the token budget before any
visible output) is a known failure mode. Empty `content` with any terminal reason is
also a possible local-memory or context-capacity failure; it must enter the
resource-recovery protocol rather than be retried unchanged. Catching either
condition here avoids discovering it mid-review, where it forces an avoidable hop
down the band's reviewer chain that a healthy stack would not have needed.

Warm-up probe used for every model the task's band will use:

```bash
curl -s http://127.0.0.1:11434/api/chat -d '{
  "model": "<model>",
  "messages": [{"role": "user", "content": "You are a code reviewer. Reply with ONLY a JSON object: {\"verdict\": \"PASS\", \"findings\": []}"}],
  "stream": false,
  "think": false,
  "options": {"num_predict": 4096, "num_ctx": <role production context>}
}' -m 180
```

Use the role's effective production context: `65536` for the Low/S Qwen Developer
delegation wrapper, `131072` for the Moderate/M Devstral local-agent runner, and the
configured reviewer context for review roles. A `"length"` result with empty
content on a small ping (e.g. `num_predict: 16`) is usually just an undersized
budget, not the real failure — retry at the production `num_predict` before
concluding the model is unhealthy.

This restart/precheck is infrastructure verification, not a review gate: a healthy
precheck does not retroactively change a prior phase's recorded result (e.g. a
historical D14 fallback stays as recorded even if a later precheck shows the
primary chain healthy again).

## § Per-task discipline — S-230-T4a worked example

Full worked example behind the guide's "every local-developer delegation packet
requires its own phase-1 pass" rule: a `declare -A` bash-3.2 incompatibility in a
delegation packet triggered a revised packet. That revised packet's own phase-1
re-review then flagged a second, unrelated claim, which the orchestrator
disproved with a reproducible test (not by asserting) before re-delegating. Both
the original verdict and the resolution were recorded distinctly rather than
overwritten. See `docs/tasks/s-230-poc-v1-digitalocean.md` § S-230-T4a for the
full transcript-level pattern.

## § Model tier resolution — vendor citation basis

**Codex/OpenAI basis** (verified 2026-08-09): OpenAI describes `gpt-5.6-sol` as the
flagship for complex coding and `gpt-5.6-terra` as the intelligence/cost balance;
`gpt-5.6-luna` is the cost-sensitive option
(<https://developers.openai.com/api/docs/models>). Codex guidance positions Sol for
complex/open-ended work, Terra as the everyday workhorse, and Luna for
clear/repeatable work, and recommends the lowest reasoning effort that meets the
quality bar (<https://learn.chatgpt.com/docs/models>). `gpt-5.5` and `gpt-5.4`
remain task-local compatibility choices, not new defaults: OpenAI classifies
GPT-5.5 as previous-generation, and GPT-5.4 / GPT-5.4 mini retire from Codex with
ChatGPT sign-in on 2026-08-31 (API-key usage unaffected). Do not silently rewrite
historical task pins.

**Claude Code/Anthropic basis** (verified 2026-08-09): the active Claude Code
runtime environment reports the current lineup as the Claude 5 family
(`claude-opus-5`, `claude-sonnet-5`, `claude-fable-5`) plus
`claude-haiku-4-5-20251001`. If the active runtime's model roster is unavailable or
the recommendation is more than roughly two months old, re-verify against official
Anthropic documentation (<https://docs.anthropic.com>) before presenting a concrete
model ID.

Both bases are presentation-time defaults, not permanent pins: re-check official
guidance whenever preparing a new task card, and preserve any explicit task-local
pin until an approved documentation change replaces it.

## § Gemma Reviewer / Muse Glimmer Reviewer — prompt canonicalization provenance

The Authority-boundary sentence in the live guide ("may not write files, apply
patches, approve tasks, certify coverage, or mark tasks complete") is the canonical
source for the authority-boundary clause actually sent to Ollama as part of Gemma
Reviewer's system prompt — it is no longer a hand-paraphrased string maintained
independently inside `scripts/gemma-code-review.py`. `scripts/local-agent/
prompt_anchors.py` holds a verbatim, provenance-tagged extraction of this clause
under the `gemma_reviewer` role key (enforced byte-for-byte by
`scripts/local-agent/prompt_anchors_test.py`'s `EC2EveryClauseIsVerbatimInItsCitedSource`
test), and `scripts/local-agent/prompt_builder.py`'s
`build_system_prompt(role="gemma_reviewer", ...)` assembles it with the script's own
output-format contract, enforcing a token budget derived from the invocation's
`num_ctx`/`num_predict` and raising before any Ollama call if the assembled prompt
does not fit. `gemma-code-review.py`'s `build_review_payload()` consumes this
builder output directly. This closes the drift class of bug that previously let the
live prompt diverge from this prose (a missing "certify coverage" and a paraphrased
"close tasks" in place of "mark tasks complete") — see
`docs/plan/local-role-prompt-canonicalization.md` and
`docs/tasks/local-role-prompt-canonicalization.md` (LRPC-1 through LRPC-5) for the
full mechanism and delivery record. Edits to the live clause should be mirrored into
`prompt_anchors.py`'s `gemma_reviewer` entry in the same change.

## § Local Architect / Complex Analyst — prompt canonicalization provenance and LRPC-6 defect

The ADR-037 §1 may/may-not boundary the live guide summarizes is the canonical
source for the authority-boundary clause `scripts/local-architect/run_analysis.py`
sends to Ollama for both `DEFAULT_PROFILE` (`local_architect_default`) and
`MED_HIGH_REFINEMENT_PROFILE` (`local_architect_med_high`). As with Gemma Reviewer,
that clause is a verbatim, provenance-tagged extraction in
`scripts/local-agent/prompt_anchors.py`, assembled at call time by
`scripts/local-agent/prompt_builder.py`'s `build_system_prompt()` rather than
hand-maintained inline in `run_analysis.py`.

**Defect record (LRPC-6):** a live-production defect found during this
canonicalization — `prompt_anchors.py`'s original extraction omitted ADR-037 line
70's governing header, "The role may not:", before its prohibition list — meant both
`gemma4` and `muse-glimmer` read the assembled prompt as *permitting* what the full
ADR-037 prose correctly prohibits. It was corrected by prepending that verbatim
header substring. See `docs/tasks/local-role-prompt-canonicalization.md` § LRPC-6
for the full defect record and fix. Edits to ADR-037's authority-boundary prose
should be mirrored into `prompt_anchors.py`'s `local_architect_default` /
`local_architect_med_high` entries in the same change.

## § Handoff prompt format — local_developer prompt canonicalization provenance

The `allowed_paths`/`boundary_violation` clause in the live guide's Handoff prompt
format section is the canonical source for `local_developer`'s authority-boundary
text. `scripts/local-agent/cli.py`'s `TOOL_CALLING_SYSTEM_PROMPT` sources it from
`scripts/local-agent/prompt_anchors.py` via `scripts/local-agent/prompt_builder.py`'s
`build_system_prompt(role="local_developer", ...)`, built once at import time — not
hand-maintained inline — mirroring the same mechanism as Gemma Reviewer and Local
Architect. See `docs/tasks/local-role-prompt-canonicalization.md` § LRPC-4 for the
delivery record. Edits to the live clause should be mirrored into
`prompt_anchors.py`'s `local_developer` entry in the same change.

## Related

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` — the live, condensed guide this archive supports
- `docs/audit/agent-workflow-binding-history.md` — superseded model bindings and dated lineage
- `docs/plan/local-role-prompt-canonicalization.md`, `docs/tasks/local-role-prompt-canonicalization.md`
- `docs/plan/agents-override-sync.md` — why the guide is loaded whole into every Codex session
