---
type: TaskList
title: "Tasks: Agent Session Preflight Gate"
plan: docs/plan/agent-session-preflight-gate.md
status: active
---
# Tasks: Agent Session Preflight Gate

## Objective

Implement a small startup preflight and write-time gate so fresh Codex and
Claude Code sessions load the DubBridge workflow contract before file edits.

## Governing Documents

- `docs/plan/agent-session-preflight-gate.md`
- `README_AGENT_ORDER.md`
- `AGENTS.md`
- `CLAUDE.md`
- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`
- `docs/policies/HITL_AUTONOMY_POLICY.md`
- `docs/policies/RRI_POLICY.md`

## Task order

```
T0 -> T1 -> T2 -> T3 -> T4a1 -> T4a2 -> T4a3 -> T4a4 -> T4b1 -> T4b2 -> T4b3 -> T4c1 -> T4c2 -> T4c3
```

## T0 — Plan and task ledger

- **Status:** [x] Done
- **Type:** planning
- **Effort:** S
- **RRI:** n/a

### Goal

Create the plan and task ledger for the agent-session preflight work.

### Acceptance Criteria

- Plan and task ledger exist with OKF frontmatter.
- The implementation task scope is explicit.

## T1 — Shared preflight script and tests

- **Status:** [x] Done
- **Type:** development
- **Effort:** L
- **RRI:** 41 -> Med-high
- **Depends on:** T0

### Goal

Add `scripts/agent-preflight.py`, with unit tests, to print the compact workflow
summary and maintain a session-local sentinel under `.agent/`.

### Acceptance Criteria

- `scripts/agent-preflight.py --print-summary` emits the workflow startup summary.
- `scripts/agent-preflight.py --mark` writes `.agent/session-preflight.json`.
- `scripts/agent-preflight.py --check` exits non-zero when the sentinel is absent
  or stale and exits zero after `--mark`.
- Tests cover the missing-sentinel and marked-sentinel paths.
- No task-specific approval decision is encoded in the script.

### Happy path examples

- `HP-1`: Fresh session runs `--mark` -> sentinel exists -> `--check` passes.
- `HP-2`: `--print-summary` -> output names workflow authority, RRI gate,
  approval threshold, mobile `DESIGN.md`, and Gemma/D14 closure review.

### Edge case examples

- `EC-1`: No sentinel exists -> `--check` fails with actionable instructions.
- `EC-2`: Sentinel belongs to a different repository root -> `--check` fails.

### RRI

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 2 | raw CC 12 -> score 2 (policy CC table) | High |
| F files | 1 | `--touches` -> 2 files | High |
| D domain | 3 | agent-supplied: agent workflow/tooling script | High |
| T coverage | 2 | agent-supplied: focused unit tests | High |
| A ambiguity | 1 | task has acceptance criteria and examples | High |
| K coupling | 3 | script feeds future session/hook workflow | High |
| P impact | 2 | developer workflow preflight only; no runtime product path | High |
| X context | 2 | workflow docs plus script/test context | High |

**Final RRI:** 41 -> band Med-high (41-55) -> Effort L.

### Implementation summary

Added `scripts/agent-preflight.py` with:

- compact workflow summary output;
- `.agent/session-preflight.json` sentinel path;
- `--mark` to write the sentinel atomically;
- `--check` to fail when the sentinel is missing, invalid, stale-versioned, or
  marked for another repository root;
- `--repo-root` override for tests and future hook wrappers.

Added `scripts/agent_preflight_test.py` covering the approved happy paths and
edge cases.

### Gemma Reviewer evidence

- Command: `python3 scripts/gemma-code-review.py /tmp/agent-preflight-t1.diff --out /tmp/agent-preflight-t1-review.json --passes 3 --task-id agent-session-preflight-T1`
- Passes run / succeeded: 3 / 3
- Quorum: met
- Aggregate status: `FINDINGS`
- Findings: one minor consensus finding; disposition rejected as non-blocking
  because it explicitly said no immediate action was required and the current
  implementation is sufficient for session setup. The earlier atomic-write
  robustness suggestion was accepted and repaired before this final review.
- Primary-agent disposition: no further code changes required.

### Reflection log

| Pass | Focus | Result |
|---|---|---|
| 1 | API/CLI behavior and sentinel semantics | `--print-summary`, `--mark`, `--check`, and `--repo-root` stay narrow and task-scoped. |
| 2 | Failure modes and repo-root validation | Missing, invalid JSON, wrong root, and version mismatch fail closed with actionable messages. |
| 3 | Test coverage and no hidden approval logic | Tests cover HP/EC cases; script records preflight only and does not encode task-specific RRI approval. |

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | Fresh session runs `--mark` -> sentinel exists -> `--check` passes | `scripts/agent_preflight_test.py::AgentPreflightTest.test_hp1_mark_then_check_passes` | passed |
| HP-2 | Happy path | `--print-summary` names workflow authority, RRI gate, approval threshold, mobile `DESIGN.md`, and Gemma/D14 closure review | `scripts/agent_preflight_test.py::AgentPreflightTest.test_hp2_summary_names_required_workflow_rules` | passed |
| EC-1 | Edge case | No sentinel exists -> `--check` fails with actionable instructions | `scripts/agent_preflight_test.py::AgentPreflightTest.test_ec1_check_fails_when_sentinel_missing` | passed |
| EC-2 | Edge case | Sentinel belongs to another repository root -> `--check` fails | `scripts/agent_preflight_test.py::AgentPreflightTest.test_ec2_check_fails_for_different_repo_root` | passed |

### Owner final verification

- Owner: Codex
- Date: 2026-06-28
- Commands run:
  - `python3 -m py_compile scripts/agent-preflight.py scripts/agent_preflight_test.py`
  - `python3 -m unittest scripts/agent_preflight_test.py -v`
  - `python3 scripts/agent-preflight.py --print-summary`
  - `python3 scripts/gemma-code-review.py /tmp/agent-preflight-t1.diff --out /tmp/agent-preflight-t1-review.json --passes 3 --task-id agent-session-preflight-T1`
- Result: all direct verification commands passed; Gemma Reviewer quorum met with
  only a non-blocking minor finding.

## T2 — Claude and Codex hook wiring

- **Status:** [x] Done
- **Type:** configuration
- **Effort:** L
- **RRI:** 51 -> Med-high
- **Depends on:** T1

### Goal

Wire the shared preflight into Claude and Codex startup/edit hooks.

### Acceptance Criteria

- Claude `SessionStart` prints and marks the preflight.
- Claude `PreToolUse` for edit/write actions calls the preflight check.
- Codex project config has equivalent session-start and pre-tool-use hooks where
  supported by the installed Codex configuration.
- Existing user-local permission entries are preserved.

### RRI

| Variable | Score | Evidence | Confidence |
|---|---|---|---|
| C cyclomatic | 0 | raw CC 4 -> score 0 (policy CC table) | High |
| F files | 1 | `--touches` -> 2 files at presentation; `.gitignore` added during implementation to ignore generated sentinel | High |
| D domain | 3 | agent workflow configuration | High |
| T coverage | 2 | manual hook-command verification plus config parsing | High |
| A ambiguity | 1 | task has acceptance criteria | High |
| K coupling | 4 | session/edit hooks affect agent edit flow | High |
| P impact | 3 | developer workflow gate; no product runtime path | High |
| X context | 3 | Claude/Codex config syntax plus T1 script behavior | High |

**Final RRI:** 51 -> band Med-high (41-55) -> Effort L.

### Implementation summary

- Added Claude `SessionStart` hook in `.claude/settings.json` to run
  `scripts/agent-preflight.py --print-summary --mark`.
- Added Claude `PreToolUse` hook for `Write|Edit` to emit a `deny` decision when
  `scripts/agent-preflight.py --check` fails.
- Added Codex inline TOML hooks in `/Users/matias/.codex/config.toml` for
  `SessionStart` and `PreToolUse`, guarded so they execute only when the current
  git root is `/Users/matias/dubbridge`.
- Added `.agent/` to `.gitignore` so the generated sentinel is never tracked.
- Preserved existing local `.claude/settings.json` permission entries.

### Verification evidence

- `python3 -m json.tool .claude/settings.json` — passed.
- `python3 -c 'import tomli; ... tomli.load(...)'` against `/Users/matias/.codex/config.toml` — passed.
- Claude `PreToolUse` command without sentinel — emitted valid JSON deny decision.
- Codex `PreToolUse` command without sentinel — emitted valid JSON deny decision.
- Codex `SessionStart` command — printed preflight summary and marked `.agent/session-preflight.json`.
- Claude and Codex `PreToolUse` commands after sentinel exists — returned 0 with no deny output.
- `git check-ignore -v .agent/session-preflight.json` — matched `.gitignore:18:.agent/`.

## T3 — Verification and close

- **Status:** [x] Done
- **Type:** docs/config verification
- **Effort:** S
- **RRI:** n/a (closure verification)
- **Depends on:** T1, T2

### Goal

Verify the script, docs frontmatter, and hook configuration, then update this
ledger with evidence.

### Acceptance Criteria

- Unit tests pass.
- OKF frontmatter check passes for the new docs.
- Task ledger records verification commands and any skipped external hook check.

### Completion evidence

- `python3 -m py_compile scripts/agent-preflight.py scripts/agent_preflight_test.py` — passed.
- `python3 -m unittest scripts/agent_preflight_test.py -v` — 6 tests passed.
- `python3 scripts/check_okf_frontmatter.py docs/plan/agent-session-preflight-gate.md docs/tasks/agent-session-preflight-gate.md` — passed.
- `bash scripts/check-task-unit-coverage.sh` — passed.
- External hook behavior was tested by executing the configured commands directly;
  no full new-window Claude/Codex restart was performed in this session.

## Active hardening workstreams

| Workstream | Status | Objective |
|---|---|---|
| `T4a` | active | Replace the reusable sentinel with session-bound, hash-attested receipts |
| `T4b` | pending | Wire native loading and fail-closed gates in Claude and Codex |
| `T4c` | pending | Certify real fresh sessions and document the managed-policy boundary |

`T4a-T4c` remain the reporting umbrellas. Execution now happens through the
subtasks below.

## T4a1 — Receipt schema and source manifest

- **Status:** [x] Done
- **Type:** development
- **Effort:** L
- **RRI:** 47 -> Med-high (recompute before execution if scope changes)
- **Depends on:** T3
- **Approval:** waived by the user on 2026-07-26 for this bounded hardening sequence

### Goal

Introduce the v2 receipt shape, provider/session/actor path derivation, and the
exact native-instruction/document manifest that later gates will verify.

### Acceptance criteria

- `scripts/agent-preflight.py` emits a v2 in-memory receipt payload with the
  required schema fields and source digests.
- Codex and Claude produce different derived receipt identities for different
  provider/session/actor tuples.
- Legacy sentinel payloads or legacy `--mark` output cannot satisfy the v2 validator.

### Happy path examples

- `HP-1`: valid Claude startup payload -> receipt manifest includes native import
  source plus governing document hashes.
- `HP-2`: same repository, different provider/session -> different receipt id.

### Edge case examples

- `EC-1`: unsupported provider or lifecycle event -> fail closed before write.
- `EC-2`: malformed session/actor payload or path-traversal-like identifier ->
  fail closed without deriving a filesystem path from raw ids.

### Evidence to emit

- Focused unit tests for schema, manifest hashing, and identity derivation.
- Updated ledger evidence for the chosen schema version and source list.

### Closure evidence (2026-07-26)

**ADR-038 Med-high gate:** ran end-to-end. qwen27 (`qwen3.6:27b-q4_K_M`)
refinement recommended `GO_LOCAL`; primary route receipt agreed `GO_LOCAL`;
`med_high_gate.py` confirmed the route. The single bounded qwen3.5a
(`qwen3.6:35b-a3b`) attempt (8 turns / 300s, 0 repair budget) ended
`status: budget_exhausted` (`total_turns_exhausted`) with no tests written and
an incorrect guess at the lifecycle event set — verified directly against the
worktree diff and transcript rather than trusted at face value. Per ADR-038
§5 this escalated straight to direct cloud implementation (no local retry).
Also surfaced, but did not fix (out of `allowed_paths`), a latent bug in
`scripts/local-agent/run_med_high_task.py`: `build_evidence_bundle()` crashes
with an uncaught `FileNotFoundError` when the local session ends without ever
producing a diff file, so no escalation bundle is written on this failure
mode. Follow-up ticket filed and closed:
`docs/tasks/med-high-escalation-bundle-crash.md` (fixed this and six
additional related evidence-loss defects found by adversarial peer review;
see that ledger's own closure evidence for the review disposition, Reflection
log, and coverage certification). Its own residual scope exception
(`resolve_rri_table`'s path-vs-text ambiguity) was filed separately as
`docs/tasks/rri-table-path-text-ambiguity.md`.

**Implementation:** added to `scripts/agent-preflight.py`: `V2_VALID_PROVIDERS`,
`V2_VALID_LIFECYCLE_EVENTS` (`startup`, `resume`, `clear`, `compact`, `fork`,
`subagent` — real events per `.claude/settings.json` plus T4b1's planned
`fork`/`subagent`), `ReceiptValidationError`, `validate_provider`,
`validate_lifecycle_event`, `validate_opaque_id`, `compute_receipt_identity`,
`hash_source_file`, `build_v2_receipt_payload`, `validate_v2_receipt_payload`.
Purely additive; `find_repo_root()`, `preflight_summary()`, and the legacy
`--mark`/`--check` sentinel path are unchanged.

**Gemma Reviewer (ADR-036 binding, `gemma4:26b-a4b-it-qat`, 3 passes) — initial
pass (superseded, see re-check below):** `status: findings`, 6 raised
findings, all independently verified against running code and found to be
**false positives**: 3 "blocking" claims (plus 2 restatements) that
`datetime`/`timezone` were unimported — the import is present at line 11 and
a live `build_v2_receipt_payload()` call was executed with no `NameError`;
1 "minor" claim that a missing `provider` key causes a `TypeError` — verified
`validate_provider(None)` correctly raises `ReceiptValidationError` (`None not
in frozenset(...)` is valid Python). One remaining "minor" suggestion (require
`documents` non-empty) is a design preference outside the T4a1 spec, not a
defect. No code changes made in response to this review pass.

This initial pass was later found to have a packet-construction defect: it
sent `git diff` only, so the pre-existing `datetime`/`timezone` import (line
11, unchanged by the diff) never appeared in what Gemma read, and it also
predated the 4 branch-coverage tests added afterwards to close the 84%→88%
coverage gap. Both are process gaps in how the review packet was built and
sequenced, not defects in the reviewed code — `scripts/agent-preflight.py`
itself was byte-identical throughout.

**Gemma Reviewer re-check (2026-07-26, corrected packet, 3 passes):** re-ran
against the final diff (all 14 `AgentPreflightV2ReceiptTest` cases, no gap)
with a packet that included the full current content of
`scripts/agent-preflight.py` alongside the diff, so the line-11
`datetime`/`timezone` import was no longer hidden from the model. Result:
`status: findings`, 0 "blocking" findings (the prior 3 false positives did not
recur — confirms the missing-context theory), 3 "minor" findings remain, all
independently verified as false: (1)/(2) claims that `importlib.util` is
imported but unused, and that `hashlib` is imported twice, in
`scripts/agent_preflight_test.py` — both false; `importlib.util` is used at
lines 12/16 to dynamically load the module under test, and `hashlib` is
imported exactly once (line 5). Inspecting the actual request payload sent to
Ollama (via `--dry-run`) showed why: the corrected packet only embedded the
full file content for `scripts/agent-preflight.py`, the file responsible for
the prior pass's false positives — `scripts/agent_preflight_test.py` was still
sent as diff-only. Its diff hunk boundaries (one hunk near line 2-8, the next
starting at line 82) skip exactly the lines (12-17) where `importlib.util` is
used, reproducing the same missing-context failure mode in the one file the
fix didn't cover. No code changes made in response to either review pass.
This re-check is superseded in turn by the pass-3 re-check below; retained for
the audit trail, not as the operative evidence.

**Gemma Reviewer pass 3 (2026-07-26, packet with full content of BOTH changed
files, 3 passes):** built a corrected packet embedding the complete current
content of `scripts/agent-preflight.py` (324 lines) *and*
`scripts/agent_preflight_test.py` (218 lines), followed by the final git diff
against both — confirmed byte-identical to the diff already reviewed in the
pass-2 re-check (`diff` against the prior packet's diff section showed no
difference), so this run isolates the effect of the packet fix alone. Result:
`status: findings`, 0 blocking, 1 consensus minor finding, 0 pass-specific/
location-inconsistent/severity-inconsistent findings. The prior false claims
that `importlib.util` is unused and `hashlib` is double-imported in
`scripts/agent_preflight_test.py` **did not recur** — confirms the
missing-context theory a second time, this time for the file the earlier fix
had missed. The one remaining finding (`os.replace` in `mark_preflight`
lacking cross-filesystem fallback) cites line 308, which is actually
`if args.mark:` inside `main()`; the real `os.replace` call is at line 104
inside `mark_preflight()` (verified directly) — a location error, but the
substance (mark_preflight uses `os.replace` after `mkdir(parents=True)`) is
correct. This is pre-existing T1 code explicitly out of scope for T4a1 ("must
not touch ... the `--mark`/`--check` sentinel logic"), the finding's own
suggestion says "None required; current implementation satisfies
requirements," and T1's closure evidence already recorded and accepted this
same atomic-write tradeoff. No code change made. No further Gemma-packet-
integrity follow-up task is needed: the fix (embed full current content for
every changed file, not only the file that produced findings in a prior pass)
is confirmed effective across both files and is the operative packet-building
rule for future review passes on this task family.

**Unit coverage certification:**

| Suite | Tests | Result |
|---|---|---|
| Existing T1/T2 (`AgentPreflightTest`) | 6 | pass, unchanged |
| New v2 receipt tests (`AgentPreflightV2ReceiptTest`) | 14 | pass — HP-1, HP-2, EC-1 x2, EC-2 x2, EC-3 x2, EC-4 x2, plus 4 direct `validate_v2_receipt_payload` branch tests |
| **Total** | **20** | **20/20 pass** |

`coverage run --branch` over `scripts/agent-preflight.py`: 88% line, all
uncovered lines confined to legacy `find_repo_root()`/CLI `main()` branches
untouched by T4a1 (git-missing fallback, JSON-decode-error path, `--print-summary`
default toggle) — the new v2 receipt code (lines ~143-260) is fully exercised.

**Reflection:** the ADR-038 gate did its job — it caught a genuinely
incomplete local attempt (no new tests, wrong domain knowledge) before it
could be mistaken for done, and forced escalation without a retry-until-it-
passes loop. Gemma Reviewer's false positives here are a reminder that
"findings" status requires verification against the actual running code
before acting on it, same as the local implementer's output — reviewer
output isn't automatically ground truth either.

### Status artifacts affected

- `docs/plan/agent-session-preflight-gate.md`
- `docs/tasks/agent-session-preflight-gate.md`

## T4a2 — Atomic publish, invalidation, and file-mode guarantees

- **Status:** [x] Done — owner-verified 2026-07-26
- **Type:** development
- **Effort:** L
- **RRI:** 52 -> Med-high (recompute before execution if scope changes)
- **Depends on:** T4a1

### Goal

Publish and invalidate v2 receipts atomically so stale or partial evidence can
never authorize later prompt/tool gates.

### Acceptance criteria

- Receipt publication uses temp-file write, `fsync`, and `os.replace`.
- Prior authorizing receipt for the same provider/session/actor is removed or
  invalidated before a new one is published.
- Receipt directories/files enforce `0700` / `0600` permissions where supported.

### Happy path examples

- `HP-1`: valid reload for the same provider/session replaces the prior receipt
  with a fully readable new one.
- `HP-2`: concurrent publishers for distinct sessions complete without collision
  or partial JSON.

### Edge case examples

- `EC-1`: interruption before `os.replace` leaves no authorizing final receipt.
- `EC-2`: permission or open/read error denies authorization instead of falling
  back to stale state.

### Evidence to emit

- Deterministic atomicity and file-mode unit tests.
- Failure-injection evidence for interrupted publish cleanup.

### Closure evidence (2026-07-26)

**ADR-038 Med-high gate:** ran end-to-end. qwen27 (`qwen3.6:27b-q4_K_M`)
refinement recommended `GO_LOCAL`. The primary route receipt **downgraded to
`CLOUD_REQUIRED`**: publish/invalidate atomicity for the v2 receipt is the
mechanism later prompt/tool gates rely on for authorization, which falls
under ADR-038 §6's explicit exclusion ("authentication or security
boundaries"; "fail-closed governance invariants") regardless of Qwen27's
recommendation. Per ADR-038 §3 the primary may downgrade but never upgrade,
so the bounded `qwen3.6:35b-a3b` local implementer was never invoked; the
primary agent (Claude) implemented directly.

**Implementation:** added to `scripts/agent-preflight.py`:
`RECEIPTS_DIR_RELATIVE`, `v2_receipts_dir`, `v2_receipt_path`,
`_secure_mkdir`, `_invalidate_prior_receipt`, `publish_v2_receipt`. The
publish path: `validate_v2_receipt_payload` -> derive target path from
`compute_receipt_identity` -> `_secure_mkdir` (0700, best-effort) ->
`_invalidate_prior_receipt` (unlink any existing file at the target path) ->
write to a `os.getpid()` + `threading.get_ident()` + `uuid4().hex`-qualified
temp file in the same directory via `O_CREAT | O_EXCL, 0o600` -> `fsync` ->
best-effort `chmod 0600` -> `os.replace` into place. Any exception before
`os.replace` unlinks the temp file and re-raises; the old receipt (already
removed) is not restored, matching the task's "stale or partial evidence can
never authorize" goal (deny-by-default, not fail-safe-to-stale). Purely
additive; `build_v2_receipt_payload`/`validate_v2_receipt_payload` (T4a1) and
the legacy `--mark`/`--check` sentinel path are unchanged. Out of scope for
this task and left untouched: T4a3's CLI contract (no new `--publish`/
`--invalidate` flags added).

**Gemma Reviewer (ADR-036 binding, `gemma4:26b-a4b-it-qat`, 3 passes):**
`status: findings`, 0 blocking, 1 consensus minor finding (3/3 passes), 1
pass-specific minor finding. Consensus finding: `_invalidate_prior_receipt`
unlinks the target before the new file is durably written, so a concurrent
reader could briefly see no receipt during a legitimate update; assessed by
the reviewer as an acceptable, intentional trade-off given the fail-closed
authorization context — no suggested change. Pass-specific finding: the
per-PID temp filename could collide across threads sharing a PID within the
same process. Verified as a real, reproducible race (direct repro:
`os.open(..., O_EXCL)` raised `FileExistsError` under concurrent
same-process/same-identity publish). Fixed in Reflection pass 3 below by
qualifying the temp filename with `threading.get_ident()` and a `uuid4()`
suffix; re-verified with a 10-thread same-identity repro producing zero
errors and exactly one final file.

### Reflection log

Required passes: 3 (RRI 52 -> Med-high)

#### Pass 1

- **Draft verdict:** implemented `publish_v2_receipt` with temp-write ->
  fsync -> `os.replace`, pre-publish invalidation via unlink, and 0700/0600
  permission enforcement; 27 new/updated unit tests passing, covering HP-1,
  HP-2 (distinct sessions), EC-1, EC-2.
- **Critique findings:** none identified in self-review against the 3
  acceptance criteria and 4 HP/EC examples; all criteria appeared satisfied.
- **Revisions applied:** none.

#### Pass 2

- **Draft verdict:** same implementation, now checked against Gemma
  Reviewer's independent 3-pass review of the real diff.
- **Critique findings:** Gemma raised one 3/3-consensus minor finding
  (invalidate-then-write ordering creates a brief no-receipt window for
  concurrent readers) and one 1/3 pass-specific minor finding (per-PID temp
  filename can collide across threads in the same process for the same
  identity — confirmed as a real, reproducible bug, not a false positive,
  via a direct 5-thread same-identity repro that raised `FileExistsError`).
- **Revisions applied:** none yet — findings triaged in this pass;
  consensus finding accepted as intentional design (fail-closed over
  availability); pass-specific finding scheduled for fix in Pass 3.

#### Pass 3

- **Draft verdict:** re-reading the implementation for the flagged
  same-process/same-identity race and for coverage gaps.
- **Critique findings:** (1) confirmed temp-filename collision risk is real
  and worth closing even though HP-2 only requires distinct-session
  concurrency; (2) coverage run showed 3 uncovered branches in newly added
  code: the `except BaseException` around the temp-file write/fsync, the
  `except OSError: pass` around the post-write `chmod`, and the
  `except FileNotFoundError: pass` TOCTOU guard in
  `_invalidate_prior_receipt`.
- **Revisions applied:** qualified the temp filename with
  `threading.get_ident()` + `uuid.uuid4().hex` (in addition to
  `os.getpid()`) to eliminate the collision window; added
  `test_hp2_concurrent_same_identity_never_collides_on_temp_name`,
  `test_ec1_failure_during_temp_write_cleans_up_and_denies_authorization`,
  `test_hp1_chmod_failure_on_temp_file_does_not_block_publish`, and
  `test_hp1_invalidate_prior_receipt_tolerates_toctou_race` to close the
  coverage gaps and lock in the fix. Re-ran the full suite (31/31 pass) and
  a manual 10-thread same-identity repro (0 errors, 1 final file).

**Unit coverage certification:**

| Suite | Tests | Result |
|---|---|---|
| Existing T1/T4a1 (`AgentPreflightTest`, `AgentPreflightV2ReceiptTest`) | 20 | pass, unchanged |
| New T4a2 publish/invalidate tests (`AgentPreflightV2ReceiptPublishTest`) | 11 | pass — HP-1 x4, HP-2 x2, EC-1 x2, EC-2 x2, plus 1 malformed-payload guard |
| **Total** | **31** | **31/31 pass** |

`coverage run --branch --include=scripts/agent-preflight.py`: 91% line
overall. All lines added for T4a2 (`v2_receipts_dir` through the end of
`publish_v2_receipt`) are 100% covered; the remaining uncovered lines are
pre-existing T1/T4a1 code out of scope for this task (`find_repo_root` git
fallback, `load_sentinel` JSON-decode-error path, CLI `main()` branches).

**Reflection:** the ADR-038 downgrade decision was the load-bearing call in
this task — Qwen27's `GO_LOCAL` recommendation was reasonable from a pure
code-complexity standpoint (single-file, well-scoped, deterministic tests),
but the task's actual subject matter (an authorization-relevant atomic
publish/invalidate mechanism) is exactly what ADR-038 §6 carves out for
cloud regardless of the local recommendation. Gemma Reviewer's pass-specific
finding (thread-collision on temp filenames) was a genuine, reproducible
defect rather than a false positive this time — verified by direct repro
before accepting it, consistent with treating reviewer output as needing
verification either way, not automatic trust.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`

## T4a3 — CLI contract and provider hook adapters

- **Status:** [ ] Pending
- **Type:** development
- **Effort:** L
- **RRI:** 41 -> Med-high (recompute before execution if scope changes)
- **Depends on:** T4a2

### Goal

Expose the receipt engine through stable `load`, `check`, `hook-load`, and
`hook-gate` entry points with documented exit behavior and stdout/stderr
separation.

### Acceptance criteria

- Direct CLI commands and provider hook adapters share the same validation core.
- Exit codes distinguish success, operational invalidity, and malformed input.
- Legacy `--mark` may remain for diagnostics but cannot authorize any v2 gate.

### Happy path examples

- `HP-1`: `load` prints agent-facing context, publishes the receipt, and exits `0`.
- `HP-2`: `hook-gate` translates a valid receipt into the provider's non-blocking response.

### Edge case examples

- `EC-1`: malformed hook JSON exits `2` with diagnostics on stderr only.
- `EC-2`: invalid or foreign receipt exits `1` and produces the provider's blocking response.

### Evidence to emit

- CLI/unit tests covering exit-code matrix and stdout/stderr separation.
- Hook-adapter fixture evidence for Claude and Codex payload translation.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`

## T4a4 — Deterministic race, replacement, and permission tests

- **Status:** [ ] Pending
- **Type:** development
- **Effort:** L
- **RRI:** 47 -> Med-high (recompute before execution if scope changes)
- **Depends on:** T4a3

### Goal

Lock the receipt engine with deterministic concurrency, replacement, and denial
tests before provider wiring begins.

### Acceptance criteria

- Tests cover simultaneous loaders, `check` racing against replacement, and
  permission/open failures.
- Accepted outcomes are limited to validated old/new receipt success or a clean
  denial; partial/stale success is never accepted.
- T4a closure evidence names the final test commands and any unsupported host-specific checks.

### Happy path examples

- `HP-1`: two simultaneous loaders for different sessions both complete with
  parseable final receipts.

### Edge case examples

- `EC-1`: `check` during invalidation/replacement never returns success for
  partial or stale JSON.
- `EC-2`: denied directory/file access produces a clean authorization failure.

### Evidence to emit

- Barrier-controlled race test results.
- T4a closure note summarizing atomicity/race coverage.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`

## T4b1 — Claude native load and lifecycle wiring

- **Status:** [ ] Pending
- **Type:** configuration
- **Effort:** L
- **RRI:** 47 -> Med-high (recompute before execution if scope changes)
- **Depends on:** T4a4

### Goal

Wire Claude's native import path and lifecycle hooks to the v2 receipt engine.

### Acceptance criteria

- `CLAUDE.md` imports the authoritative workflow bytes that the receipt records.
- Claude startup/resume/clear/compact/fork/subagent events map to supported
  `hook-load` sources.
- Claude prompt/tool gating uses v2 receipt validation instead of the legacy sentinel.

### Happy path examples

- `HP-1`: fresh Claude startup records native-load evidence and passes gate checks.

### Edge case examples

- `EC-1`: unsupported or missing Claude hook payload fields deny access cleanly.

### Evidence to emit

- Config-parse verification plus hook fixture outputs for each mapped lifecycle.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`

## T4b2 — Codex native bundle, document limit, and gate wiring

- **Status:** [ ] Pending
- **Type:** configuration
- **Effort:** L
- **RRI:** 52 -> Med-high (recompute before execution if scope changes)
- **Depends on:** T4b1

### Goal

Generate and wire the Codex-side native instruction bundle so session and tool
gates validate the same exact workflow bytes.

### Acceptance criteria

- `AGENTS.override.md` round-trips `AGENTS.md` followed by the authoritative workflow guide.
- Codex configuration sets a document byte limit above the generated bundle size.
- Codex session/subagent/prompt/tool gates call the v2 adapters and stop relying
  on the legacy shared sentinel.

### Happy path examples

- `HP-1`: fresh Codex startup loads the generated bundle and publishes a valid v2 receipt.

### Edge case examples

- `EC-1`: bundle drift or byte-limit underflow fails closed before authorization.

### Evidence to emit

- Bundle-generation verification with exact size/hash output.
- Config-parse and hook-fixture evidence for session and tool gates.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`

## T4b3 — Portable path resolution and duplicate-hook cleanup

- **Status:** [ ] Pending
- **Type:** configuration
- **Effort:** M
- **RRI:** 36 -> Moderate (recompute before execution if scope changes)
- **Depends on:** T4b2

### Goal

Remove hard-coded checkout assumptions and eliminate competing user-level hooks
that could race or certify the wrong repository state.

### Acceptance criteria

- All hook/config resolution derives the repository from `cwd` or git root.
- No active Claude/Codex hook path for this repository depends on an absolute checkout path.
- Duplicate or stale user-level gates are removed, neutralized, or explicitly reported as blockers.

### Happy path examples

- `HP-1`: opening the repository from a different checkout path still resolves
  the correct repo root and receipt location.

### Edge case examples

- `EC-1`: a duplicate stale hook is detected and reported instead of silently racing.

### Evidence to emit

- Resolution audit showing live hook sources and final repo-root derivation path.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`

## T4c1 — Fresh-session smoke harness

- **Status:** [ ] Pending
- **Type:** verification
- **Effort:** M
- **RRI:** 36 -> Moderate (recompute before execution if scope changes)
- **Depends on:** T4b3

### Goal

Run real fresh-session startup checks for Claude and Codex instead of relying
only on direct command invocation.

### Acceptance criteria

- Both CLIs are exercised from a fresh session/window path, not only by replaying hook commands.
- The smoke output proves a unique tail marker and current workflow SHA from the
  fully loaded source, not just the compact summary.
- Any provider that cannot be exercised in-session is recorded as unverified, not certified.

### Evidence to emit

- Per-provider smoke transcripts or screenshots with the tail marker and SHA.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`

## T4c2 — Audit coverage report and certification math

- **Status:** [ ] Pending
- **Type:** docs/config verification
- **Effort:** M
- **RRI:** 28 -> Moderate (recompute before execution if scope changes)
- **Depends on:** T4c1

### Goal

Publish an auditable coverage report that counts opened sessions, certified
sessions, and missing-evidence sessions without overstating certainty.

### Acceptance criteria

- The audit command/report distinguishes opened sessions from certified sessions.
- A `100%` claim is refused whenever any session lacks native-load plus receipt evidence.
- The coverage report names the exact criteria for certification.

### Evidence to emit

- Audit report output with certified/opened counts and refusal behavior.

### Status artifacts affected

- `docs/tasks/agent-session-preflight-gate.md`
- `docs/plan/agent-session-preflight-gate.md`

## T4c3 — Managed-policy boundary and blocker handoff

- **Status:** [ ] Pending
- **Type:** docs/policy
- **Effort:** M
- **RRI:** 26 -> Moderate (recompute before execution if scope changes)
- **Depends on:** T4c2

### Goal

Document the difference between repository-level certification and literal
non-bypassable enforcement, and leave a clean blocker/handoff if the admin
layer cannot be installed from this repository.

### Acceptance criteria

- Repository/user-hook certification and administrator-managed
  non-bypassability are reported as separate enforcement levels.
- Any host-policy step that cannot be completed from repository permissions is
  recorded as a blocker or handoff, not as completed certification.
- The final task note includes the exact admin-layer artifacts or commands that
  must be applied outside the repo.

### Evidence to emit

- Boundary/handoff note with admin-managed requirements and unresolved blockers.

### Status artifacts affected

- `docs/plan/agent-session-preflight-gate.md`
- `docs/tasks/agent-session-preflight-gate.md`

## Closure

T0-T3 are complete. Hardening now proceeds through `T4a1-T4c3`. No commit has
been made.
