---
type: ADR
title: "ADR-034: Gemma process audit log and reviewer multi-pass reconciliation contract"
status: Accepted
supersedes: ""
superseded_by: ""
---

# ADR-034: Gemma process audit log and reviewer multi-pass reconciliation contract

- **Status:** Accepted
- **Date:** 2026-06-24
- **Deciders:** DubBridge platform team
- **Scope:** the `gemma-audit-and-triple-pass` slice
  (`docs/plan/gemma-audit-and-triple-pass.md`)
- **Closes:** the "the local-Gemma processes have no auditable record of their
  inputs/outputs/decision variables, and the reviewer collapses a single opinion
  with no way to see disagreement" gap

## Context

DubBridge runs two local-Gemma roles through Ollama, both advisory and bounded by
existing policy:

- **Gemma Developer** (`scripts/delegate-low-rri.py`) — patch delegation for
  eligible Low-RRI (0–25) simple code patches.
- **Gemma Reviewer** (`scripts/gemma-code-review.py`) — read-only advisory code
  review for Low/Moderate (0–40) development tasks.

Both write a per-invocation `result.json`, but there is **no aggregated record**
across invocations. Any process evaluation today is manual — e.g.
`docs/evaluations/large-file-delegation-2026-06-21.md` reconstructed a
silent-corruption failure by hand from a single artifact. We cannot answer, from
data, questions like "how often does Gemma's output truncate", "how often does a
delegated packet escalate", "how often does the reviewer raise findings the
primary agent then dismisses". Without that, the handoff playbook and the review
contract can only be tuned by anecdote.

Separately, the Reviewer runs a **single pass**. A single small-model review is
noisy: a finding present in one sampling may be absent in the next, and a single
run gives no signal about which findings are stable versus likely false positives.
The prior `docs/plan/gemma-reviewer-triple-pass.md` proposed a three-pass upgrade
but left the durable contracts undefined (aggregate schema, quorum/partial-failure
semantics, reconciliation algorithm, exit codes, artifact naming).

These are cross-cutting, hard-to-reverse decisions: they define a new telemetry
surface (what is recorded, what is never recorded, where it lives) and they change
the advisory contract that every agent consumes when reviewing Low/Moderate work.
They are recorded as an ADR — in the same documentation-and-process category as
the RRI and OKF adoptions — because the slice carries an `arch_decision` and the
decisions bind future agent behavior once enforced.

## Decision

### 1. Append-only audit log, emitted through the shared helper

Both roles emit one structured JSONL record per invocation to
`logs/gemma-audit/YYYY-MM.jsonl`, written by `append_audit_log()` in the shared
`scripts/gemma_local.py` — the same module that already writes `result.json` — so
Developer and Reviewer cannot drift. The log is **local telemetry only**:
git-ignored, never committed, never required by remote/GitHub-hosted CI.

### 2. What is recorded — and what is never recorded

Automatic fields the wrapper can compute are always present (role, outcome,
`done_reason`, mode, file/diff sizes, scope violations, apply result, elapsed).
Orchestrator-only fields (`task_id`, `rri`, `band`, `attempt`, `disposition`) are
optional and default to `null`.

**Prompts are recorded.** The `system_prompt` and `user_prompt` sent to Gemma on
each invocation are written verbatim to the audit record. This is a first-party
local log (git-ignored, never committed) and the owner has elected full prompt
visibility to support prompt tuning and failure reconstruction without manual
artifact inspection.

**What is never written:** raw target-file bodies (the source files being reviewed
or patched, beyond the diff already implicit in the prompt); free-text fields are
secret-redacted per `HITL_AUTONOMY_POLICY.md` before any write.

### 3. Reviewer runs N passes (default 3), sequential

The Reviewer runs N independent passes over the same packet (`--passes`,
`DUBBRIDGE_REVIEW_PASSES`), sequentially — the target hardware is single-GPU and
parallel passes only contend. `--passes 1` reproduces the current single-pass
contract exactly, which is also the rollback path.

### 4. Review is mandatory; isolated subagent is the fallback when Gemma fails

A pass succeeds on `PASS`/`FINDINGS` and fails on `BLOCKED`/timeout/malformed/
`done_reason == "length"`. With **≥2 of 3** successful passes the wrapper emits an
aggregate (exactly 2/3 ⇒ `degraded: true`).

With **<2** successful passes (quorum failure) or when Gemma is entirely
unavailable, the agent **must** spawn a context-isolated subagent as the
mandatory fallback reviewer. The fallback subagent receives the same isolation
packet (diff + acceptance criteria + any partial findings) and its output is
advisory, exactly as Gemma's. The primary agent remains orchestrator of record.

The review step is **mandatory for all development tasks**. Gemma is the
preferred path; the isolated subagent is the required fallback. Neither quorum
failure nor Gemma unavailability may be used to skip the review entirely.
No additional human approval gate beyond what the RRI band already requires is
opened by this fallback path.

### 5. Reconciliation is deterministic and wrapper-owned

The wrapper, not the model, classifies findings across passes: `consensus`
(≥2 passes, exact `(path, line, severity)`), `pass-specific` (1 pass),
`severity-inconsistent` (same `(path, line)`, differing severity),
`location-inconsistent` (same path, line within ±3), and `likely-false-positive`
(`pass-specific` ∧ out-of-scope). The constants (±3, ≥2 = consensus) are fixed and
documented, so the contrast step is inspectable and unit-testable.

### 6. Backward-compatible artifacts and exit codes

Per-pass artifacts derive from `--out` (`result.passK.json`); the aggregate is
written at the base `--out` path so existing callers are unaffected. Exit `0` when
quorum is met (`PASS`/`FINDINGS`/`degraded`); non-zero only on quorum failure or
operational failure.

### 7. The disposition of findings is adjudicated context-isolated, not by the implementer

Gemma's three passes are already uncontaminated — the model never sees the
development process. The contamination is in the **agent-side disposition**: today
the same primary agent that wrote the code also decides which findings to accept
or dismiss, and the workflow only asks it to *simulate* detachment ("re-read as if
reviewing someone else's code"). That is role-play, not structural isolation, and
it biases the very `dismissed-*` audit signals this ADR introduces.

When the trigger fires — **consensus** `blocking`/`major` findings, slice band
≥ Med-high, or inter-pass disagreement — the disposition is adjudicated by a
**context-isolated reviewer**: a fresh subagent or fresh session fed *only* the
final diff, the acceptance criteria, and the reconciled findings — never the
development transcript, chain-of-thought, or dead-ends. The deterministic packet
builder guarantees that isolation.

The adjudicator is **advisory**, exactly like Gemma: the primary agent remains
orchestrator of record and owns the final close per `HITL_AUTONOMY_POLICY.md`. Its
obligation changes only in that it must reconcile its disposition against the
adjudicator's and record any **`disposition_divergence`** — which is itself an
audit field that measures implementer bias directly. For trivial cases (Low band,
3/3 `PASS`, no consensus findings) the trigger does not fire and no isolated
adjudication is spawned, bounding the cold-start cost.

## Consequences

### Positive

- Process tuning becomes data-driven: truncation, escalation, destructive-diff,
  finding-quality, and inter-pass disagreement become measurable signals.
- The reviewer surfaces stability of findings instead of a single noisy opinion.
- One schema serves both roles and the triple-pass metrics; no divergent logs.
- The HITL guarantee and the read-only/advisory authority are unchanged.
- Context-isolated adjudication removes implementer anchoring bias from the
  disposition, and `disposition_divergence` makes that bias measurable instead of
  invisible.

### Negative / cost

- Reviewer latency roughly triples (~36–90 s for 3 passes on current hardware).
- A new local artifact surface to manage (rotation, git-ignore, redaction).
- Prompt logging increases per-record size significantly (system prompt alone can
  be several KB); monthly JSONL files will be larger than metrics-only logs.
- More wrapper logic (N-pass loop + reconciliation) to test and maintain.
- The isolated adjudicator adds a gated cold-start spawn and, lacking the
  development rationale, may re-raise intentional decisions as findings; the
  trigger gate bounds when this cost is paid.

### Neutral

- The audit log is advisory telemetry, not a gate; an empty or missing log never
  fails a task.
- `--passes 1` keeps the current behavior available unchanged.

## Alternatives considered

- **Model-owned reconciliation** (ask one completion to self-simulate three
  opinions): rejected — not inspectable or testable, and a small model conflates
  the passes. Reconciliation must be deterministic Python.
- **Per-wrapper audit emission**: rejected — the two roles would drift; emitting
  through the shared helper keeps one schema and one code path.
- **Logging full packets/diffs (diff bodies, target file contents) for richer
  forensics**: rejected — size cost without proportional diagnostic value; the
  prompt already contains the diff, and raw file bodies would dominate log size.
  Prompts (system + user) are logged; file bodies are not.
- **Logging prompts as opt-in / truncated**: rejected — this is a first-party
  local log and full prompt fidelity is required for prompt tuning and failure
  reconstruction; truncation would defeat that purpose.
- **Two separate slices (audit, then triple-pass)**: rejected — they share the
  `gemma_local.py` seam; sequencing audit first lets triple-pass emit into the
  same schema and avoids instrumenting the reviewer twice.
- **No ADR, decisions in the plan only** (the prior reviewer-role slice's
  precedent): rejected for this slice because the audit schema and reconciliation
  contract are durable and cross-cutting enough to warrant an indexed record.
- **Keep simulated self-review for the disposition** ("review as if it were
  someone else's code", status quo): rejected when the trigger fires — role-play
  detachment does not remove anchoring bias and corrupts the new audit signals.
  Retained only below the trigger threshold, where the cost of isolation is not
  justified.
- **Make the isolated adjudicator authoritative** (own the close): rejected — it
  would conflict with the HITL orchestrator-of-record model. The adjudicator stays
  advisory; the primary agent reconciles and closes.
