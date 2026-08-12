---
type: Proposal
title: "Proposal: Latency-aware secondary threshold for local review packets"
status: Superseded
---

# Proposal: Latency-Aware Secondary Threshold for Local Review Packets

- **Status:** Superseded by ADR-036 Amendment 2 / ADR-037 Amendment 1
  (2026-08-11) — this proposal's entire premise was `qwen3.6:27b-q4_K_M` as
  the RRI 26–55 phase-1/phase-2 reviewer; the local model stack restructure
  reverted that role to Gemma (with Muse Glimmer as intermediate fallback)
  and reassigned `qwen3.6:27b-q4_K_M` to the local implementer role. The
  specific latency evidence below (prompt-eval cost on large qwen review
  packets) no longer applies to the reviewer path this proposal targeted. If
  a similar latency-budget gap is later observed against Gemma or Muse
  Glimmer as reviewers, it needs its own fresh proposal — do not silently
  re-apply this one's numbers to a different model. See
  `docs/tasks/local-model-stack-restructure-2026-08.md` T3.
- **Original status (as of 2026-08-09):** Proposed — not yet scoped as a
  task, no RRI computed, no implementation started.
- **Date:** 2026-08-09
- **Author:** Investigation carried out by Claude Code during a local-stack
  reliability review, at owner request.
- **Governed by (on implementation):** `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
  § Reviewability budget gate; `docs/policies/RRI_POLICY.md`.

**The remainder of this document is preserved unchanged as the historical
record of the original investigation and proposal.**

## 1. Problem

Two distinct latency failure modes affect the local review pipeline
(`qwen3.6:27b-q4_K_M` as phase-1/phase-2 reviewer for RRI 26–55):

1. **Cold-load / memory-eviction latency** — diagnosed and mitigated earlier
   today via `OLLAMA_MAX_LOADED_MODELS=1`, persisted across reboots by
   `/Users/matias/Library/LaunchAgents/com.dubbridge.ollama-env.plist`. Not
   the subject of this proposal.
2. **Prompt-eval latency on large packets** — **not mitigated**, and this
   proposal addresses it.

### Evidence for (2)

`.agent/peer-code-review-antares-t3c-1.json` (2026-08-05) recorded a
successful qwen review that nonetheless took **147.4s total**, of which
**131.5s (89%)** was `prompt_eval_duration` against an **11,837-token**
prompt. Effective prompt-eval throughput on this hardware: ≈ 11,837 / 131.5
≈ **90 tokens/sec**. The source packet for that specific call does not
survive on disk — `scripts/peer-workflow-review.py` accepts `--content FILE`
or stdin, and this call was very likely streamed rather than persisted
(the only surviving artifact for that task on that date is the phase-1
`peer-task-review-antares-t3c-1-input.md`, a different, earlier round).

**The existing reviewability budget gate does not catch this.**
`scripts/check-review-budget.py` (`make qa-review-budget`) derives its line
budget from `DEFAULT_NUM_CTX` (context-overflow prevention), not from
observed throughput:

```
DEFAULT_NUM_CTX      = 131072   # scripts/gemma_local.py
DEFAULT_NUM_PREDICT  = 4096
PACKET_OVERHEAD_TOKENS = 1300
TOKENS_PER_DIFF_LINE   = 20

derived_budget = (131072 - 4096 - 1300) // 20 = 6283 lines  (~125,660 tokens)
```

The Aug-5 packet's 11,837 real tokens used only **~9.4%** of that derived
budget, so the gate would not have flagged it, yet the call still cost 131.5s
of pure prompt evaluation. **Context-overflow protection and latency
protection are different concerns; only the first is currently gated.**

### Is this the typical case, or an outlier?

Measured the actual size of 32 review-related files currently in `.agent/`
(`*review-packet*`, `*code-review*`, `*task-review*`, `*.packet`):

| Stat | Bytes | Est. tokens (4 chars/token) |
|---|---|---|
| Median | 4,621 | ~1,155 |
| P90 | 18,128 | ~4,532 |
| Max persisted | 29,315 (`peer-code-review-S-140-T3c-i-r3.packet`) | ~7,329 |

**Typical packets are small — median ~1,155 tokens, P90 ~4,532 tokens.** The
Aug-5 case's 11,837 real tokens is above even the largest packet that left a
trace on disk. This is a genuine outlier relative to normal traffic, not the
everyday operating pattern — but the pipeline has no mechanism to detect or
flag such an outlier before it costs 2+ minutes of wall time on a
Med-high/Complex-band review that is already on the critical path to human
approval.

## 2. Proposed mitigation

Two parts, in order. Part 1 is a prerequisite for a trustworthy Part 2.

### Part 1 — Passive calibration (no synthetic test calls needed)

Every real qwen review call already returns `prompt_eval_count` and
`prompt_eval_duration` in the Ollama `/api/chat` response — that is exactly
how the 90 tokens/sec figure above was derived, from one production call.
Instrument `scripts/peer-workflow-review.py`'s qwen call path
(`run_qwen_band_review`) to append these two fields (plus `load_duration`,
`eval_duration` for completeness) to a small rolling log,
`.agent/qwen-latency-log.jsonl`, on every real qwen review — no extra calls,
no synthetic load generation. This accumulates real throughput samples from
actual traffic instead of extrapolating a hard threshold from a single
observation.

### Part 2 — Latency-derived secondary check, pilot as WARN-only

Add a second, independent check to `check-review-budget.py` (or a sibling
function alongside `derive_budget()`), computed from the *same* estimated
token count already used for the line-budget check:

```
tokens_per_second = resolve_calibrated_rate():
    1. DUBBRIDGE_REVIEW_TOKENS_PER_SECOND env override, if set
    2. median of .agent/qwen-latency-log.jsonl, if >= 5 samples
    3. else: documented provisional fallback = 90 (the single Aug-5 sample),
       explicitly flagged low-confidence in the output

estimated_eval_seconds = estimated_tokens / tokens_per_second
flag if estimated_eval_seconds > DUBBRIDGE_REVIEW_MAX_EVAL_SECONDS (default: 90)
```

**Start in WARN-only mode** (recorded in the review artifact, does not block
phase-1/phase-2 review) until the calibration log accumulates ≥ 5 real
samples with reasonable spread. Promote to hard-fail — reusing the exact same
`D14-OVERRIDE: <reason>` escape hatch the line-budget gate already has —
only after that pilot window confirms the threshold is reliable.

## 3. Trade-off and recommendation

| Option | Certainty | Cost |
|---|---|---|
| Hard-fail from day 1 using the single Aug-5 datapoint (90 tok/s) | Low — n=1, throughput varies with machine load/thermal state, risk of wrong threshold in either direction | None — immediate |
| Passive instrumentation + WARN-first pilot, promote after ≥5 real samples | High — threshold derived from accumulated real traffic before it can block anything | A pilot window during which an outlier as large as Aug-5 could still slip through uncaught |

**Recommendation: passive instrumentation + WARN-first pilot.** A single
data point is not enough to trust as a hard gate input — it risks either
blocking normal packets on a falsely tight number, or missing the next real
outlier on a falsely loose one. This also mirrors the pattern this repository
already uses for exactly this kind of promotion decision: Antares was piloted
observe-only before being promoted to a blocking touchpoint
(`docs/tasks/antares-security-specialist-advisor.md` § T5). The instrumentation
cost (Part 1) is trivial — it logs fields the response already contains — so
there is no reason to skip straight to an uncalibrated hard gate.

## 4. Affected files (if approved for implementation)

- `scripts/peer-workflow-review.py` — `run_qwen_band_review` (append calibration
  log entries).
- `scripts/check-review-budget.py` — new `derive_latency_estimate()` alongside
  `derive_budget()`; new env vars `DUBBRIDGE_REVIEW_TOKENS_PER_SECOND`,
  `DUBBRIDGE_REVIEW_MAX_EVAL_SECONDS`.
- New: `.agent/qwen-latency-log.jsonl` (rolling calibration data, not committed —
  same treatment as other `.agent/*.json` operational artifacts).
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § Reviewability budget gate — document
  the second threshold once promoted out of WARN-only.

## 5. Explicitly out of scope of this proposal

Two other anomalies surfaced during the same investigation are **not**
addressed here and need a separate decision:

- **Unexplained 30s idle timeout.** `.agent/peer-code-review-fmc-5.json`
  (2026-08-09) shows both qwen and Gemma failing with "idle timeout after 30s
  without a token" on a **1,222-byte** packet
  (`.agent/fmc-5-code-review-packet.md`) — inconsistent with the documented
  `DEFAULT_IDLE_TIMEOUT_SECONDS = 180` in `scripts/gemma_local.py`, and
  inconsistent with a "packet too large" explanation, since this packet is
  small. Seen twice today. Root cause not found.
- **Cross-provider D14 fallback failure.** The same FMC-5 record shows
  `cross_provider_d14_attempt: "unusable: claude -p --bare --model
  claude-sonnet-5 --effort medium returned Not logged in"`, forcing
  `d14_provider_route: "same-provider-degraded"`. Separate operational gap
  (Claude CLI authentication), unrelated to the Ollama/qwen latency
  investigation.

## 6. Next step

This document is a **Proposal**, not an approved Plan — per
`docs/playbooks/AGENT_WORKFLOW_GUIDE.md` § "Mandatory workflow before
implementing", the next step if approved is to promote this into
`docs/plan/<slice>.md` + `docs/tasks/<slice>.md`, compute RRI with
`scripts/rri.py` against the actual touched files, and present the resulting
task card for explicit approval before any code is written.
