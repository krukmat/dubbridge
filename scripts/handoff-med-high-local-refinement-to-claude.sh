#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
cd "${repo_root}"

emit_handoff() {
  cat <<'HANDOFF'
You are taking over the approved DubBridge `med-high-local-refinement` slice.
Work in the current dirty worktree; preserve every existing user/agent change.
Do not commit, push, discard, reset, or rewrite unrelated changes.

Mandatory read order before editing:
1. `CLAUDE.md`
2. `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (highest workflow authority)
3. `AGENTS.md`
4. `docs/adr/ADR-038-med-high-architect-refined-single-attempt.md`
5. `docs/plan/med-high-local-refinement.md`
6. `docs/tasks/med-high-local-refinement.md`
7. ADR-036, ADR-037, RRI_POLICY, and HITL_AUTONOMY_POLICY where referenced.

Owner approval already exists (2026-07-26: `si, de acuerdo`) for this exact route:
Qwen27 advisory refinement -> primary hash-bound route decision -> either exact
Qwen35 single attempt (1 session, 8 turns, 300 seconds total, 0 repairs) or
immediate Codex/Claude implementation with preserved handoff evidence.

Current truthful state:
- T0 complete: ADR-038, plan, ledger, risk/decomposition.
- Phase-1 for T1-T4: D14 PASS after Qwen27 and Gemma outputs truncated.
- T1 complete: `med-high-refinement-v1` profile in `run_analysis.py`, 17
  tests passing, phase-2 (qwen3.6:27b-q4_K_M) recorded with disposition
  `reviewed_no_change`.
- T2 complete: `scripts/local-agent/med_high_gate.py` hash-bound route gate,
  28 tests passing, phase-2 recorded with disposition `fixed`.
- T3 complete: band-aware `EffectiveLimits`/`resolve_effective_limits` in
  `run_local_task.py`, 76 tests passing, phase-2 recorded with disposition
  `partial_fix`.
- T4 complete: `scripts/local-agent/run_med_high_task.py` process-group
  supervisor (300s wall clock, killpg on timeout) plus the extended
  `escalation_packet` evidence bundle, 15 tests passing, phase-2 recorded
  with disposition `partial_fix`.
- T5 in progress: `docs/policies/RRI_POLICY.md`,
  `docs/policies/HITL_AUTONOMY_POLICY.md`, and
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` have been updated to describe the
  actual ADR-038 route (including a Mermaid diagram in the workflow guide)
  and no longer describe Med-high as direct local-first with a repair
  attempt. `make qa-docs` passes. Confirm no other status-bearing doc still
  carries the retired "1 repair attempt" Med-high language before closing T5.
- T6 code is implemented; `python3 scripts/rri_test.py` has 64 passing tests,
  but phase-1 was skipped and phase-2 remains pending. Do not mark Done yet.
- T7 not started.
- Incidental blocker fixed: D14 can now load `adjudicator-packet.py`;
  `python3 scripts/peer_workflow_review_test.py` has 43 passing tests.

Execute in ledger order:
1. Finish T5: sweep remaining status-bearing docs for stale Med-high
   local-first/repair-attempt language, then update the ledger/plan to
   `[x] Done`.
2. Close T6: run the pending phase-2 review, address/disposition-record
   findings, mark Done.
3. Run T7 focused/integrated tests, three Reflection passes for Med-high code
   tasks, required phase-2 reviews (already recorded inline for T1-T4), then
   synchronize plan/ledger status.

Hard rules:
- Qwen27 is advisory/read-only and may only recommend GO_LOCAL/CLOUD_REQUIRED.
- The primary may downgrade GO_LOCAL to cloud; it may never upgrade CLOUD_REQUIRED.
- Missing, invalid, stale, mismatched, timed-out, or unavailable evidence routes cloud.
- Qwen35 may not silently substitute models and gets exactly 8 turns/300 s/0 repairs.
- Do not use the not-yet-enforced local runner to implement its own enforcement.
- Use `apply_patch` for edits and report exact verification evidence.

Inspect the repository directly instead of asking the user to paste files. Begin by
reconciling the dynamic status below with the documented ledger; update the ledger
only when evidence exists.
HANDOFF

  printf '\nCurrent git status:\n'
  git status --short
  printf '\nCurrent relevant diff summary:\n'
  git diff --stat -- \
    scripts/local-architect \
    scripts/local-agent \
    scripts/rri.py \
    scripts/rri_test.py \
    scripts/peer-workflow-review.py \
    scripts/peer_workflow_review_test.py \
    docs/adr/ADR-038-med-high-architect-refined-single-attempt.md \
    docs/plan/med-high-local-refinement.md \
    docs/tasks/med-high-local-refinement.md
}

case "${1:---print}" in
  --print)
    emit_handoff
    ;;
  --run)
    if ! command -v claude >/dev/null 2>&1; then
      printf 'ERROR: Claude Code CLI (`claude`) is not available on PATH.\n' >&2
      exit 127
    fi
    handoff_prompt="$(emit_handoff)"
    exec claude -p "${handoff_prompt}"
    ;;
  *)
    printf 'Usage: %s [--print|--run]\n' "$0" >&2
    exit 2
    ;;
esac
