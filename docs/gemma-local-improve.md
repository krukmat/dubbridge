# Local Ollama Rules

This document summarizes the **active** local Gemma contracts used in DubBridge.
It is guidance, not the governing authority.

Authoritative sources:

- `docs/policies/RRI_POLICY.md`
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md` — Gemma Developer (patch delegation)
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Gemma Reviewer` — Gemma Reviewer (code review)
- `docs/adr/ADR-034-gemma-process-audit-and-reviewer-reconciliation.md` — audit + multi-pass ADR
- `scripts/delegate-low-rri.py` — patch delegation wrapper
- `scripts/gemma-code-review.py` — review wrapper (N-pass + reconciliation)
- `scripts/adjudicator-packet.py` — D14 trigger gate + isolation packet builder
- `scripts/gemma-audit-report.py` — read-only audit report tool
- `scripts/gemma-push-review.py` — Push Reviewer wrapper (post-pipeline audit + routing)

## Audit log

Every invocation of both roles appends one JSONL record to
`logs/gemma-audit/YYYY-MM.jsonl` (local only — git-ignored, never committed).
Run `python3 scripts/gemma-audit-report.py` to read per-role metrics and
calibration signals: truncation rate, escalation rate, inter-pass disagreement,
out-of-scope findings, dismissed-major rate.

## Push Reviewer

**Gemma Push Reviewer** is a separate local Gemma role. It starts only after a
GitHub push pipeline has completed, collects GitHub run metadata/log evidence,
runs a push-audit quorum, routes findings through `scripts/rri.py`, and may
dispatch only pure Low eligible incidents to Gemma Developer.

It is an **audit/dispatch orchestrator**, not an approver:

- It does not replace Gemma Reviewer code review.
- It does not compute final RRI itself; `scripts/rri.py` is the only final RRI source.
- It does not accept or close delegated patches.
- Any Gemma Developer patch created from Push Reviewer findings remains
  `review_status: in_review` until a non-Gemma agent completes post-development review.

Operational surfaces:

- Automatic GitHub trigger: `.github/workflows/push-review.yml` via `workflow_run`
  after `ci` completes on a `self-hosted` runner.
- Local replay/debug: `make qa-gemma-push-review`.

## Gemma Developer vs. Gemma Reviewer

The shared primary model is `gemma4:12b-mlx` as of 2026-07-01. The previous
default, `gemma4:26b-a4b-it-qat`, remains the automatic fallback when no explicit
model override is set and the primary model is not installed locally.
Role-specific environment variables still take precedence
(`DUBBRIDGE_REVIEW_MODEL`, `DUBBRIDGE_PUSH_REVIEW_MODEL`, then
`DUBBRIDGE_LOW_RRI_MODEL` where applicable) and are treated as strict explicit
choices. The MLX tag requires an Ollama runtime new enough to pull and run
current Gemma 4 MLX manifests.

| | Gemma Developer | Gemma Reviewer |
|---|---|---|
| **Purpose** | Implement a simple code patch | Review code for correctness and safety |
| **Trigger** | Low RRI (0–25) eligible simple code patches | Low/Moderate RRI (0–40) development task completion |
| **Returns** | Tagged file blocks with complete contents | Tagged finding blocks only |
| **Can write files?** | Yes (via wrapper + git apply) | No |
| **Can approve?** | No | No |
| **Script** | `scripts/delegate-low-rri.py` | `scripts/gemma-code-review.py` |
| **Make target** | n/a (invoked by agent directly) | `make qa-gemma-review` |
| **Audit fields** | `mode`, `diff_added/removed`, `scope_violations`, `apply_result`, `verify_ok` | `passes_run/succeeded`, `degraded`, `consensus_count`, `disposition_divergence` |

## Relationship between the three local Gemma roles

| Role | Primary input | Purpose | Final authority |
|---|---|---|---|
| **Push Reviewer** | Completed GitHub pipeline run + diff | Audit a push, score/reroute findings, optionally dispatch pure Low work | Primary agent / daily workflow |
| **Gemma Developer** | Low-RRI delegation packet | Propose a narrow code/test patch | Delegating agent |
| **Gemma Reviewer** | Final code diff + acceptance criteria | Review completed development work | Primary agent |

## Multi-pass review and mandatory fallback

The Reviewer runs **N sequential passes** (default 3, `--passes N`,
`DUBBRIDGE_REVIEW_PASSES`). A deterministic wrapper-owned reconciliation step
classifies findings as `consensus`, `pass-specific`, `severity-inconsistent`,
`location-inconsistent`, or `likely-false-positive`. `--passes 1` reproduces the
previous single-pass contract exactly.

**The review is mandatory for all development tasks.** Gemma is the preferred
path. When Gemma is unavailable or quorum fails (<2 passes succeed), the agent
must spawn a **context-isolated subagent** as the fallback reviewer. The subagent
receives an isolation packet (diff + acceptance criteria + any partial findings)
built by `scripts/adjudicator-packet.py`. Its output is advisory; the primary
agent reconciles and records `disposition_divergence` in the audit log.

The D14 trigger (`should_adjudicate()` in `scripts/adjudicator-packet.py`) also
fires on: consensus blocking/major findings, slice band ≥ Med-high, or inter-pass
disagreement — in addition to the mandatory `gemma_blocked=True` path.

## Gemma Reviewer response contract

For the active review-only protocol, the model must return tagged finding blocks:

```text
STATUS: PASS
SUMMARY: short summary
=== FINDING START ===
PATH: repo/relative/path
LINE: integer line number
SEVERITY: blocking|major|minor|nit
DETAIL: concise issue description
SUGGESTION: concise remediation
=== FINDING END ===
```

Use exactly one status value: `PASS` (no findings), `FINDINGS` (one or more
finding blocks), or `BLOCKED` (packet not reviewable). The model must never
return file contents, JSON, or a unified diff in review mode.

Exit codes for `scripts/gemma-code-review.py`:

- Exit `0`: review ran; result artifact written (`PASS` or `FINDINGS`).
- Exit non-zero: operational failure only (Ollama unavailable, invalid response,
  or `STATUS: BLOCKED`).

A `BLOCKING` finding does not fail the gate by itself. The primary agent reads
the artifact and decides disposition.

## Gemma Developer response contract (patch delegation)

The local model never writes files and never authors a unified diff.

It may only propose **complete final file contents** for in-scope files. The
wrapper validates the response, constructs the diff with git, and applies it only
after scope and patch checks pass.

## Active response contract

For the active Low-RRI protocol, the model must return tagged text blocks with
one header section plus zero or more file blocks:

```text
STATUS: PATCH|NO_PATCH|BLOCKED
SUMMARY: short summary
TEST: optional verification command
RISK: optional risk note
=== FILE START ===
PATH: relative/path.ext
ACTION: create|modify|delete
--- CONTENT ---
<COMPLETE final file contents>
=== FILE END ===
```

Rules:

- The model must not return JSON.
- The model must not return a unified diff.
- The model must not return partial file fragments.
- `delete` requires empty content.
- If the change cannot be expressed safely inside the allowed scope, return
  `STATUS: NO_PATCH`.

## Acceptance and rejection

Accept the response only when all of the following are true:

- every required marker is present;
- every changed path is inside the declared allowed scope;
- no extra text appears outside the permitted sections;
- no path is duplicated;
- file actions are policy-valid for the current tree state;
- the wrapper-built diff passes `git apply --check`.

Reject immediately if the response contains JSON, a unified diff, missing
markers, duplicate paths, out-of-scope paths, or invalid file actions.

## Packet discipline

Keep delegation packets small and concrete:

- one narrow objective;
- exact allowed paths;
- explicit `must change` and `must not change` rules;
- minimal relevant context;
- clear stop condition.

If the first attempt is structurally or semantically weak, run at most one
bounded repair cycle with a **smaller** scope and the failure evidence. A second
failure escalates back to the primary agent workflow.
