# Evaluation: Gemma Push Reviewer Live Test

- Date: `2026-06-25`
- Repo: `krukmat/dubbridge`
- Task: `T7 - Live audit review dry run and close-out evidence`

## Objective

Run the Push Reviewer against completed GitHub pipeline runs and confirm that:

- dry-run mode collects GitHub evidence and assembles the model payload without
  invoking Gemma;
- live local execution writes usable audit artifacts for daily agents;
- pure Low dispatch stays safe and explicit; and
- blocked/degraded outcomes remain visible instead of disappearing.

## GitHub runs used

1. `28157583084` / `c80bf06bbdfa128c831be159a666e6817cd57688`
   - Title: `docs(workflow): prescribe Balanced tier for D14 adjudicator subagent`
   - Conclusion: `success`
   - Purpose: dry-run packet validation and blocked-live-path validation on
     editorial/workflow-only changes.
2. `28156296888` / `9400b794dc49499aed8912e9d3fbc71ff1460ff5`
   - Title: `style(db): cargo fmt R-02 wildcard arms`
   - Conclusion: `success`
   - Purpose: successful live run with low operational risk and no pure-Low
     auto-dispatch expected.

## Commands executed

### Dry-run evidence

```bash
python3 scripts/gemma-push-review.py \
  --event-path /tmp/dubbridge-push-review-t7/workflow_run_event.json \
  --force \
  --out-dir /tmp/dubbridge-push-review-t7/dry \
  --dry-run > /tmp/dubbridge-push-review-t7/dry/payload.json
```

### Live run (blocked exploratory replay)

```bash
python3 scripts/gemma-push-review.py \
  --event-path /tmp/dubbridge-push-review-t7/workflow_run_event.json \
  --force \
  --out-dir /tmp/dubbridge-push-review-t7/live \
  --no-think \
  --temperature 0
```

### Live run (successful replay)

```bash
python3 scripts/gemma-push-review.py \
  --event-path /tmp/dubbridge-push-review-t7/workflow_run_event_fmt.json \
  --force \
  --out-dir /tmp/dubbridge-push-review-t7/live-fmt \
  --no-think \
  --temperature 0
```

## Dry-run result

- Dry-run succeeded and wrote the packet to
  `/tmp/dubbridge-push-review-t7/dry/packet.json`.
- The assembled payload was captured in
  `/tmp/dubbridge-push-review-t7/dry/payload.json`.
- The payload contains:
  - a Push Reviewer system prompt with the tagged-text response contract;
  - a user message containing the GitHub-backed packet with repo, push range,
    pipeline metadata, jobs, and diff context.
- No audit log entry or push-review result artifact was written by the dry-run
  path, as required.

## Live run results

### Attempt A — blocked but visible

- Run: `28157583084` / `c80bf06`
- Outcome: `blocked`
- Block reason: `parser_rejection`
- Blocked artifact:
  `/tmp/dubbridge-push-review-t7/live/blocked.json`
- Markdown summary:
  `docs/reports/push-review/2026-06-25-c80bf06.md`

Observed behavior:

- The wrapper collected the packet successfully.
- Gemma produced a response that violated the contract:
  `STATUS PASS` plus findings.
- The wrapper failed closed and wrote a blocked artifact + Markdown summary.
- The blocked summary is still daily-readable and points to the fallback packet,
  which satisfies the visibility requirement for degraded outcomes.

### Attempt B — successful aggregate

- Run: `28156296888` / `9400b79`
- Outcome: `pass`
- Aggregate JSON:
  `/tmp/dubbridge-push-review-t7/live-fmt/aggregate.json`
- Markdown summary:
  `docs/reports/push-review/2026-06-25-9400b79.md`

Recorded audit facts:

- Pipeline conclusion: `success`
- Audit status: `pass`
- Quorum: `met`
- Passes: `1/1`
- Grounded findings: `0`
- Observe findings: `0`
- Candidates scored: `0`
- Pure Low candidates: `0`
- Developer dispatch attempted: `0`
- Development reports: `[]`
- Deferred due complexity: `0`
- Needs HITL: `0`

Interpretation:

- The report is usable by daily agents.
- No pure Low patch was dispatched.
- No non-Low finding had to be deferred.
- The resulting Markdown summary is explicit and safe: delegated/deferred/HITL
  sections render `none` rather than disappearing.

## Daily usability verdict

- `docs/reports/push-review/2026-06-25-9400b79.md` is concise and directly usable
  in the daily ledger.
- `docs/reports/push-review/2026-06-25-c80bf06.md` demonstrates the fail-closed
  blocked path remains visible to non-Gemma operators.
- For this validation set, there were no findings to dispatch and no deferred
  non-Low work items.

## Operational findings from T7

1. `--run-id` currently fails on this machine's `gh` version because
   `scripts/gemma-push-review.py` asks for JSON field `runAttempt`, while the
   installed CLI exposes `attempt`.
2. One live replay still produced a parser-blocked artifact because the model
   returned `STATUS PASS` with findings. The wrapper handled this correctly by
   failing closed and writing blocked evidence.

These are real follow-up issues, but they did not prevent completion of the live
validation because the workflow_run replay path succeeded.

## Verification commands carried forward from T1-T6

- T1-T4 integrated script surface:
  `python3 scripts/gemma_push_review_test.py`
- T2 focused re-verification:
  `python3 scripts/gemma_push_review_test.py -v`
- T3-T4 script parse/import verification:
  `python3 -m py_compile scripts/gemma-push-review.py scripts/gemma_push_review_test.py`
- T5 wiring verification:
  `python3 scripts/gemma_push_ops_test.py`
- T5 local skip path:
  `DUBBRIDGE_SKIP_GEMMA_PUSH_REVIEW=1 make qa-gemma-push-review`
- T6 documentation consistency:
  `make qa-docs`

## Conclusion

`T7` is complete.

- Dry-run evidence exists.
- Live run evidence exists for both blocked and successful paths.
- The successful run produced a daily-usable Markdown summary.
- The blocked run proved the fail-closed path stays visible.
- No pure Low dispatch occurred in the validated run, and no non-Low finding
  needed daily deferral for this sample.
