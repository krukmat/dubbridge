---
type: Audit
title: "Incident: muse-glimmer:30b-q4_K_M ignores think:false under real review packets"
status: open
---

# Incident: `muse-glimmer:30b-q4_K_M` ignores `think:false` under real review packets

## Summary

`gemma_local.build_chat_payload` sets `"think": false` in the Ollama `/api/chat`
request for every local role, including the Gemma/Muse Glimmer Reviewer chain
(`scripts/gemma-code-review.py`). For `muse-glimmer:30b-q4_K_M`, this flag is
demonstrably **not honored** once the request carries a realistic review
system-prompt + packet (the same `system_prompt` in `gemma-code-review.py:186-204`
plus a real ~270-line/~2.8k-token diff) — the model consumes the full
`num_predict` budget generating invisible content and returns
`done_reason: "length"` with **empty `content`**, rather than the expected
`STATUS:`/`SUMMARY:`/`FINDING` tagged block.

This is exactly the failure mode already named in
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Mandatory workflow before implementing`,
Step 0: *"A silent `done_reason: 'length'` with empty `content` (thinking-mode
exhausting the token budget before any visible output) is a known failure mode."*
That doc's Local resource-recovery protocol assumes this shows up as a **memory/
capacity** symptom; this incident shows the same signature can also be a pure
**prompt-template/think-flag** defect, reproducible with normal memory headroom
and normal per-token throughput (~7 tok/s, no stall in the underlying inference
loop itself).

## How this was found

While closing `LRPC-2` (`docs/tasks/local-role-prompt-canonicalization.md`), three
consecutive `make qa-gemma-review` runs against the same small, correct diff
(`scripts/local-agent/prompt_builder.py` + its test, ~270 diff lines) appeared to
stall — CPU time on the `llama-server` process advanced only ~1s per 10s of wall
clock, no output for 5-20+ minutes, across two Ollama restarts and one full
model-unload/reload cycle.

Root-cause isolation (bisection against the live model, outside the wrapper
script):

1. A trivial one-line prompt (`"Reply with ONLY: {...}"`) always completed in
   5-20s at normal throughput (~7.6 tok/s), at both `num_ctx=16384` and
   `num_ctx=131072`. Ruled out: host memory pressure, `num_ctx` mismatch against
   the model's native `16384` context window, and general model/inference-loop
   health.
2. The real `gemma-code-review.py` system prompt alone (`186-204`) + a trivial
   user message: completed normally (24.5s, `done_reason: "stop"`).
3. A trivial system prompt + the real diff packet alone: completed normally
   (32.5s, `done_reason: "stop"`, `prompt_eval_count: 2821`).
4. The **real system prompt + the real diff packet together**, exactly as
   `gemma-code-review.py` builds it, with `num_predict` capped to `1024` (down
   from the production default `4096`) to make the failure observable in bounded
   time: **`done_reason: "length"`, `content: ""`, `eval_count: 1024`** (the full
   budget consumed), `eval_duration: ~142s` at normal per-token throughput.

So neither half alone triggers it; only the specific combination of this
system-prompt's phrasing with a real-size packet does, and the model spends the
entire token budget on invisible output before returning nothing — consistent
with unsuppressed internal "thinking" tokens that `think: false` (the API-level
flag) does not actually gate for this model/chat-template combination. No script
in this repository (`gemma_local.py`, `gemma-code-review.py`,
`run_local_task.py`, `run_analysis.py`) currently prepends a textual `/no_think`
(or equivalent) directive to the prompt itself — every role relies solely on the
API flag.

## Scope / blast radius

- Confirmed affected: `scripts/gemma-code-review.py` (Gemma/Muse Glimmer
  Reviewer, phase-1 and phase-2 review for every RRI 0-55 development task —
  see `AGENT_WORKFLOW_GUIDE.md § Band-routed peer review`).
- Likely affected (not yet independently reproduced, same `build_chat_payload`
  path and same `think` handling): `scripts/local-agent/run_local_task.py`
  (Qwen/local-agent developer role) and `scripts/local-architect/run_analysis.py`
  (Local Architect / ADR-037/ADR-038 refinement).
- Not affected: `prompt_builder.py` itself (LRPC-2's deliverable) — it is a pure
  function with no network IO; this incident is entirely in the consumer
  scripts' request construction, which LRPC-2 explicitly does not touch
  (deferred to LRPC-3/4/5 per `docs/plan/local-role-prompt-canonicalization.md`
  § Architecture).
- `gemma_local.DEFAULT_REVIEW_MODEL = "muse-glimmer:30b-q4_K_M"` — the memory
  `feedback_gemma_reviewer_model_binding.md` recorded this as `gemma4:26b-a4b-it-qat`
  per ADR-036; that binding is now **stale relative to the current code** and
  should be corrected separately.

## Why this task's closure was not blocked on fixing it

Per `AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer / Muse Glimmer Reviewer §
Availability`, three failed passes against the primary/intermediate-fallback
model (the same model in this repo's current binding) with no usable
consolidated result is exactly the trigger for the mandatory D14
context-isolated-adjudicator fallback. `LRPC-2`'s phase-2 review was completed
via D14 instead of blocking on this infrastructure defect; see
`docs/tasks/local-role-prompt-canonicalization.md § LRPC-2 § Peer Reviewer
evidence` for that record.

## Suggested remediation (not yet scoped as a task)

- Prepend a literal `/no_think` (or the correct directive for this model's
  chat template) to the system prompt for `muse-glimmer:30b-q4_K_M` specifically,
  or confirm via Ollama/model documentation what actually suppresses this
  model's internal reasoning under its `chatml` template
  (`--chat-template chatml` is visible in the running `llama-server` invocation).
- Add a bounded-`num_predict` canary check inside `gemma-code-review.py`'s
  pass loop: if a pass returns `done_reason == "length"` with empty `content`,
  treat it as a distinct, explicitly logged failure class (not merged into the
  generic "unavailable/invalid output" bucket) so this is diagnosed faster next
  time, without needing manual bisection.
- Correct `feedback_gemma_reviewer_model_binding.md` (agent memory) — the actual
  current default reviewer model is `muse-glimmer:30b-q4_K_M`
  (`scripts/gemma_local.py:32`), not `gemma4:26b-a4b-it-qat`.
- This should become its own scored task (RRI unknown, likely Moderate given it
  touches `gemma_local.py`'s shared `build_chat_payload` and affects three
  consumer scripts) once prioritized — not scoped or estimated here.

## Related

- `docs/tasks/local-role-prompt-canonicalization.md` § LRPC-2
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Mandatory workflow before
  implementing (Step 0, local resource-recovery protocol), § Gemma Reviewer /
  Muse Glimmer Reviewer § Availability
- `scripts/gemma_local.py` (`DEFAULT_REVIEW_MODEL`, `build_chat_payload`)
- `scripts/gemma-code-review.py` (`build_review_payload`, system prompt)
