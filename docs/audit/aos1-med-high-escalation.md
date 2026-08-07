---
type: Audit
title: "Audit: AOS1 Med-high ADR-038 escalation packet"
date: 2026-08-07
status: closed
---

# Audit: AOS1 Med-high ADR-038 escalation packet

Governing task: `docs/tasks/agents-override-sync.md` (Task AOS1).

The bounded local implementation session
(`scripts/local-agent/run_med_high_task.py`, model `qwen3.6:35b-a3b`, its own
process group, budget ≤8 turns / ≤300 seconds / 0 repair attempts per
ADR-038) stopped at turn 6 of 8 with `status: wall_clock_exceeded` after
300.02 seconds. No diff had been written to the disposable worktree at that
point — the session was still in its read/context-gathering phase — so there
was nothing to salvage. Per ADR-038, Med-high carries zero repair attempts;
this escalated directly to cloud implementation (Claude Code, Sonnet 5, same
session), exactly as designed.

Supporting JSON evidence (Qwen27 refinement artifact, primary route receipt,
phase-1/phase-2 peer-review transcripts) is at
`docs/audit/med-high/aos1/*.json`.

## Escalation packet body

## 1. Task spec + RRI table

Task ID: `AOS1`

Spec:

Task AOS1: Generate AGENTS.override.md deterministically + wire a drift check into qa-docs.

CONFIRMED THIS SESSION (do not re-derive): AGENTS.override.md's current content is
EXACTLY content(AGENTS.md) + content(AGENT_WORKFLOW_GUIDE.md), byte for byte, with
NO separator inserted between them (each source file's own leading '---'
frontmatter fence is what visually looks like a seam -- there is no extra '---'
line added by the concatenation itself).

Build two things:

1. scripts/generate-agents-override.py
   - A standalone Python 3 script, no third-party dependencies, no LLM calls.
   - Reads exactly these three files, in this exact order, relative to the repo root:
       1. AGENTS.md
       2. docs/playbooks/AGENT_WORKFLOW_GUIDE.md
       3. docs/policies/HITL_AUTONOMY_POLICY.md
   - For each of the three files: if the file does not exist, OR exists but is
     zero-length (empty), the script must exit with a non-zero exit code and a
     clear stderr message naming which file was missing/empty. It must NOT write
     any output file in that case (no partial writes).
   - When all three files are present and non-empty, concatenate their raw text
     content in the order above with NO separator inserted between them (plain
     string concatenation of the three file contents, in that order).
   - Support two modes selected by CLI arguments:
       a) default (no flags, or --check): print the concatenated content to stdout
          only. Do not write any file. This is what the drift-check function in
          step 2 will invoke and capture.
       b) --write: write the concatenated content to AGENTS.override.md at the
          repo root (overwriting it), instead of printing to stdout.
   - The script must be deterministic: running it twice with unchanged source
     files must produce byte-identical output both times.
   - Repo root resolution: resolve paths relative to the script's own location
     (e.g. script is at scripts/generate-agents-override.py, repo root is one
     directory up), not relative to the current working directory, so it works
     regardless of where it's invoked from.

2. A new function in scripts/check-doc-consistency.sh (bash script, already has
   three existing check functions: check_status_parity_and_completeness,
   check_dangling_refs, check_superseded_successors -- each calls add_violation
   on failure and is invoked unconditionally near the bottom of the file before
   the final violations check). Add a new function, e.g. check_agents_override_drift,
   that:
   - Runs "python3 scripts/generate-agents-override.py" (default/--check mode,
     stdout capture) to get the expected content.
   - Compares it byte-for-byte against the current committed AGENTS.override.md
     file's content.
   - If AGENTS.override.md does not exist at all, OR its content differs from the
     generator's expected output, call add_violation with a message that names the
     exact fix command: "python3 scripts/generate-agents-override.py --write".
   - Must not crash with an unhandled/unset-variable error under "set -euo pipefail"
     (which is already active at the top of the script) in either the missing-file
     case or the generator-invocation-fails case.
   - Call this new function alongside the other three existing checks (same flat
     dispatch pattern -- add one more function-call line near
     check_status_parity_and_completeness / check_dangling_refs /
     check_superseded_successors, do not restructure the existing three checks).
   - Do NOT add any new Makefile target. The check must run as part of the
     existing "make qa-docs" invocation (which already runs this whole script).

3. Regenerate AGENTS.override.md once: run
   "python3 scripts/generate-agents-override.py --write" so the committed file
   matches the new generator's output exactly (this adds the missing
   HITL_AUTONOMY_POLICY.md section; do not hand-edit the file).

STOP CONDITION: after "bash scripts/check-doc-consistency.sh" runs clean (no
violations) against the regenerated AGENTS.override.md, stop. Do not touch the
content of AGENTS.md, docs/playbooks/AGENT_WORKFLOW_GUIDE.md, or
docs/policies/HITL_AUTONOMY_POLICY.md themselves -- only how AGENTS.override.md
is produced and verified.

RRI table: see `docs/tasks/agents-override-sync.md` § RRI (Final RRI 51, Med-high).

## 2. Plan

See `docs/plan/agents-override-sync.md`.

## 3. Allowed paths

- `scripts/generate-agents-override.py`
- `scripts/check-doc-consistency.sh`
- `AGENTS.override.md`

## 4. Full diff

Not captured by the local session (stopped before any write). The actual
implementing diff was produced by the cloud escalation path (Claude Code) and
is available via `git log`/`git show` on the commit(s) closing this task.

## 5. Commands executed with output

Not captured by the local session (stopped before any write); see
`docs/tasks/agents-override-sync.md` § Closure record § Owner final
verification for the commands the cloud path ran.

## 6. Test results

Not captured by the local session; see `docs/tasks/agents-override-sync.md`
§ Unit coverage certification.

## 7. Per-attempt summaries

- Final status: `wall_clock_exceeded` at turn 6/8, 300.02s elapsed.

## 8. Acceptance tests

- `python3 scripts/generate-agents-override.py --write`
- `bash scripts/check-doc-consistency.sh`
- `python3 scripts/generate-agents-override.py | diff -q - AGENTS.override.md`

## 9. Refinement artifact (Qwen27)

See `docs/audit/med-high/aos1/refinement_artifact.json`. Summary:
`route_recommendation: GO_LOCAL`, model `qwen3.6:27b-q4_K_M`, packet SHA-256
`c9c06859ce300640d3441c6e6a3becb4c44db063a253c3d9e2b15c5e57df3ad0`, resolved
digest matches expected digest.

## 10. Primary route receipt

See `docs/audit/med-high/aos1/primary_receipt.json`. Summary: `decision:
GO_LOCAL`, `primary_id: claude-code-sonnet-5`, concurred with Qwen27 (no
ADR-038 §6 hard exclusion applies).

## 11. Effective limits

```json
{
  "band": "Med-high",
  "max_repair_attempts": 0,
  "max_total_turns": 8,
  "required_model": "qwen3.6:35b-a3b"
}
```

## 12. Stop reason and hashes

- Stop reason: `wall_clock_exceeded`
- Card hash: `c9c06859ce300640d3441c6e6a3becb4c44db063a253c3d9e2b15c5e57df3ad0`
- Refinement artifact SHA-256: `9554b26d157b7cd4e3a0db55b1d904dd64e2bced12251e3d8f458d5b5195d3b0`
- Runner model: `qwen3.6:35b-a3b`
- Runner status: `wall_clock_exceeded`
- Elapsed: `300.01543025s`

## Related

- `docs/tasks/agents-override-sync.md`
- `docs/plan/agents-override-sync.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md` § Med-high Architect-refined single-attempt gate
