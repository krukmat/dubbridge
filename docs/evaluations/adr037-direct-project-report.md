---
type: Evaluation
title: "ADR-037 direct project report"
date: 2026-07-20
status: in-progress
---

# Evaluation: ADR-037 direct project use

Tracks operational evidence for the direct-project lane defined by
`docs/tasks/adr037-local-architect-direct-project.md`. Sections are filled as each
task completes; this file is not a final verdict until the later tasks run.

## T1 - Resolve, install, and fingerprint the exact model binding

`T1` ran on 2026-07-19 after explicit approval and resolved the exact requested
binding `qwen3.6:27b-q4_K_M` without substitution.

### Pre-mutation state

- `ollama list` did not contain `qwen3.6:27b-q4_K_M`.
- `/api/tags` did not contain `qwen3.6:27b-q4_K_M`.
- `/api/ps` was later observed empty immediately before the smoke run, so no other
  large model remained resident during the verification step.

### Pull and fingerprint

| Field | Value |
|---|---|
| Tag | `qwen3.6:27b-q4_K_M` |
| Digest | `a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e` |
| Ollama list ID | `a50eda8ed977` |
| Format | `gguf` |
| Family | `qwen35` |
| Parameter size | `27.8B` |
| Quantization | `Q4_K_M` |
| Advertised context length | `262144` |
| Embedding length | `5120` |
| Size on disk | `17420432739` bytes |
| Modified at | `2026-07-19T15:26:45.351928331+02:00` |
| Pull result | `success` |

The exact requested tag was accepted by the registry and downloaded successfully.
No alias or fallback model was used.

### Smoke transcript

- Prompt: `Reply with exactly: T1 smoke ok`
- Response: `T1 smoke ok`
- Model: `qwen3.6:27b-q4_K_M`
- Created at: `2026-07-19T13:27:43.513276Z`
- Prompt eval count: `18`
- Prompt eval duration: `721308000` ns
- Eval count: `205`
- Eval duration: `31004857000` ns
- Load duration: `7071147292` ns
- Total duration: `38806040375` ns

The payload also included a `thinking` field, but the externally visible response
still matched the bounded smoke requirement exactly.

### Residency and unload evidence

- Smoke start timestamp: `2026-07-19T13:27:04Z`
- `/api/ps` after smoke timestamp: `2026-07-19T13:27:48Z`
- `/api/ps` after smoke showed exactly one resident large model:
  `qwen3.6:27b-q4_K_M` with digest
  `a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e`,
  `size_vram=18520179997`, `context_length=32768`
- Unload timestamp: `2026-07-19T13:27:53Z`
- `/api/ps` after unload: empty

### Outcome

`T1` passed. The exact ADR-037 binding is now installed locally, fingerprinted, has
a successful smoke transcript, and unload confirmation was recorded. `T4` remains
blocked on `T2` and `T3`; this task did not start any downstream execution.

## T2 - Build the one-shot, tool-free invocation wrapper

`T2` ran on 2026-07-20 after explicit approval and implemented the planned
wrapper in `scripts/local-architect/run_analysis.py` with focused tests in
`scripts/local-architect/run_analysis_test.py`.

### Implementation coverage

- Immutable packet bytes are hashed with SHA-256 before any generation step.
- `/api/tags` is checked to resolve the exact installed digest for
  `qwen3.6:27b-q4_K_M`, and execution fails closed on tag or digest mismatch.
- The wrapper makes exactly one `stream=false` call to `/api/generate`.
- Response validation requires the ADR-037 JSON schema fields
  `objective`, `current_state`, `constraints`, `risks`, `recommendations`,
  `open_questions`, `evidence_gaps`, and labeled `claims`.
- Success/failure artifacts record packet identity, model identity, prompt
  version/hash, runtime parameters, timestamps, and available generation stats.
- Output writes are atomic, and existing outputs are preserved unless
  `--overwrite` is explicitly set.

### Focused test evidence

- `python3 -m unittest scripts/local-architect/run_analysis_test.py`
- Result: `Ran 5 tests ... OK`
- Covered cases:
  - success artifact for valid packet/model
  - packet hash mismatch stops before generation
  - model digest mismatch stops before generation
  - invalid response schema writes failure artifact
  - existing output without overwrite is preserved

### Review evidence

- Task-analysis review: `gemma .agent/peer-task-review-t2.json - PASS`
- Code-solution review: `gemma .agent/peer-code-review-t2.json - PASS`

### Usage surface

Example bounded invocation:

```bash
python3 scripts/local-architect/run_analysis.py \
  --packet /path/to/packet.json \
  --expected-packet-sha256 <packet_sha256> \
  --output /path/to/artifact.json
```

### Outcome

`T2` passed. `T4` remains blocked on `T3`; this task did not run a real project
analysis packet.

## T3 - Select the first real work item and freeze the project packet

`T3` ran on 2026-07-20 after explicit approval and froze the first bounded
project-analysis packet for `S-140` without invoking any model.

### Work-item selection

- Default work item `S-140` was retained.
- No explicit owner override selecting another eligible roadmap item was found in
  the reviewed repository context before freeze.
- The selection remains aligned with ADR-037 and the direct-project plan: `S-130`
  is complete locally and `S-140` is the next natural consumer in the roadmap.

### Frozen packet evidence

| Field | Value |
|---|---|
| Work item | `S-140` |
| Packet path | `.agent/local-architect/adr037/S-140/packet.json` |
| Freeze record | `.agent/local-architect/adr037/S-140/freeze-record.json` |
| Repository revision | `e30653d59465c09e3cb2e8ef060c37b70c300bef` |
| Packet SHA-256 | `1e69aea975e6281e39cc55effbdd312e63d465e63e7f39d88bb6e89fcfbdb02a` |
| Schema version | `adr037-local-architect-packet-v1` |

### Included evidence

- `docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md`
- `docs/plan/roadmap.md`
- `docs/plan/s-130-asr-transcription.md`
- `docs/bdd/s-130-asr-transcription.feature`
- `docs/adr/ADR-030-review-decision-ledger-and-fail-closed-publication-gate.md`
- `docs/plan/s-160-review-publication-workspace.md`
- `docs/architecture.md`

### Exclusions and redaction posture

- Process-only context beyond the packet contract was left out.
- No implementation source files were included in this planning freeze.
- No external vendor/model material was included; the packet is repository-bounded.
- No secrets or live production data were included.
- Missing context was preserved explicitly as `UNKNOWN`.

### Outcome

`T3` passed. The `S-140` packet is now bounded, immutable, attributable, and
hash-addressed, so `T4` can execute against a fixed repository snapshot rather
than a moving planning surface.

## T4 - Run direct Local Architect analysis on the selected work item

`T4` attempted one bounded execution on 2026-07-20 after explicit approval, using
the frozen `S-140` packet and the exact ADR-037 model binding. The task did not
produce a valid advisory analysis artifact because the wrapper failed closed before
generation.

### Execution inputs

| Field | Value |
|---|---|
| Work item | `S-140` |
| Packet path | `.agent/local-architect/adr037/S-140/packet.json` |
| Packet SHA-256 | `1e69aea975e6281e39cc55effbdd312e63d465e63e7f39d88bb6e89fcfbdb02a` |
| Model tag | `qwen3.6:27b-q4_K_M` |
| Expected digest | `a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e` |
| Output artifact | `.agent/local-architect/adr037/S-140/t4-analysis-artifact.json` |
| Recomputed RRI | `37 Moderate` |

### Pre-run verification

- `/api/tags` exposed `qwen3.6:27b-q4_K_M` with digest
  `a50eda8ed977ab48a12431878896b27ffd5cef552c17af3317d9623b939a7f1e`.
- `ollama list` showed the same exact local tag installed.
- `/api/ps` was empty immediately before execution, so no large model was resident.
- Phase-1 review passed: `gemma .agent/peer-task-review-t4.json - PASS`.

### Failure evidence

- Wrapper command attempted once via
  `python3 scripts/local-architect/run_analysis.py ...`.
- The preserved failure artifact reports:
  - `status: failed`
  - `error.code: http_error`
  - `error.message: http://127.0.0.1:11434/api/tags returned HTTP 405: 405 method not allowed`
- `/api/ps` remained empty after the failed attempt, so the model never became
  resident for analysis.
- No structured advisory output, generation metrics, or automatic-failure scan over
  model claims could be produced because generation never started.

### Interpretation

`T4` is blocked by a wrapper defect in `T2`, not by model identity, packet drift, or
runtime memory pressure. The exact execution path required by ADR-037 was attempted
once and preserved as evidence without fallback substitution.

### Resolution and final run (2026-07-20)

The wrapper defect was root-caused to Ollama buffering the entire chain-of-thought
before returning when `thinking` is not explicitly disabled, which exceeded the
client timeout on this model/host. Fixed via Option C: `think: false` plus
thinking-provenance capture fields (`think_disabled`, `thinking_present`,
`thinking_sha256`) added to `run_analysis.py`, covered by a 10-test unit suite
(`run_analysis_test.py`). `T4` was re-run against the same frozen `S-140` packet
and model binding.

| Field | Value |
|---|---|
| Output artifact | `.agent/local-architect/adr037/S-140/t4-analysis-artifact.json` (gitignored per `.agent/` convention) |
| `think_disabled` | `true` |
| `thinking_present` | `false` |
| `status` | `ok`, `success: true` |
| Claims returned | 5 `SUPPORTED`, 2 `UNKNOWN` |
| Automatic-failure scan | `PASS` — no invented facts; ADR-006/018/030 verified to exist and applied consistently; no false authority claims; fail-closed readiness gating addressed as risk+recommendation; genuine uncertainty correctly labeled `UNKNOWN` |

**Closure:** Code-solution review: n/a (task-analysis output, not code). Task-analysis
review: `gemma .agent/peer-task-review-t4.json` — `PASS`. `T4` is `Done` (2026-07-20),
committed as `6343df3`. `T5` is unblocked; `T6` remains blocked on `T5` plus the first
downstream S-140 milestone.

## T5 - Verify and author the actual project decision, plan, and tasks

`T5` recomputed RRI for the target canonical planning task at `53 Med-high`
(`python3 scripts/rri.py --C 3 --T 3 --A 3 --X 3 --D 3 --K 2 --P 2 --touches docs/adr
--touches docs/plan/s-140-subtitle-generation.md --touches
docs/tasks/s-140-subtitle-generation.md --json`), matching the ledger's preliminary
`50 Med-high` in the same band, then verified every `T4` claim/recommendation
against repository evidence before authoring anything canonical.

### Reconciliation table

| T4 item | Disposition | Evidence |
|---|---|---|
| S-130 produces TranscriptText/WordAlignment, READY on both | Accepted | `docs/tasks/s-130-asr-transcription.md:28,171,211` |
| S-140 has no canonical plan/tasks | Accepted | `docs/plan/roadmap.md:134` |
| S-160 and S-170 "built on fixtures, awaiting S-140/S-150" | **Rejected as stated; corrected** | S-160 is `✅ done 2026-06-13` (`docs/plan/roadmap.md:136`); only sub-item `X-S-160-3` remains gated on S-140/S-150 (`docs/plan/roadmap.md:384`). S-170 (`⬜ no plan yet`) genuinely depends on S-140 (`docs/plan/roadmap.md:137`). |
| Rust orchestrates; Python isolated to ML workers | Accepted | `CLAUDE.md:58`; precedent `workers/asr-worker-py` |
| Must feed the existing fail-closed review gate, no bypass | **Accepted, re-grounded as binding, not advisory** | `docs/adr/ADR-030-*.md:104` ("The contract operates ahead of real subtitle/dub producers") already places this obligation on S-140 |
| UNKNOWN: exact v1 subtitle output format defined? | Confirmed genuinely open | No SRT/VTT/JSON schema found anywhere in repo; recorded as Design decision D2 in the new plan |
| UNKNOWN: segmentation needs separate ML model vs. reuse word_alignment? | Confirmed genuinely open | No repo evidence either way; recorded as Design decision D1 |
| Rec: scope = consume Transcript/WordAlignment → canonical artifact → immutable object-store record | Accepted | Matches `docs/adr/ADR-006-*.md:36-39` checksum/storage_key pattern |
| Rec: Rust orchestrates; Python only if ML segmentation justified | Accepted | Matches `asr-worker-py` contract split |
| Rec: "SubtitleReady" gate → enqueue S-160 review, no parallel paths | **Accepted, re-grounded** — restated as ADR-030's existing obligation, not a new idea | `docs/adr/ADR-030-*.md:104` |
| Rec: draft new ADR for S-140 lineage/checksum | **Rejected — no new ADR needed** | ADR-006 already covers lineage/checksum generically; ADR-030 already covers the review-gate obligation; no genuinely new architectural decision exists beyond `ArtifactKind::Subtitle`, which follows an established pattern. Cross-references added to the plan instead. |
| Rec: task decomposition (artifact def / orchestration / S-160 integration / observability) | Accepted | Consistent with S-130's own task shape and ADR-018 |
| Open question: is multilingual subtitle generation part of S-140? | **Resolved — was already answered by repo evidence, not actually open** | `docs/plan/roadmap.md:82` pipeline: `S-140 subtitles -> S-150 translation + dubbing` |

### Outputs authored

- `docs/plan/s-140-subtitle-generation.md` (new, `status: proposed`) — canonical
  plan citing verified evidence, with Design decisions D1 (segmentation source)
  and D2 (subtitle schema) explicitly flagged as **open, unresolved by repository
  evidence**, to be ratified by the task approver before `S-140-T1` execution.
- `docs/tasks/s-140-subtitle-generation.md` (new, `status: proposed`) — task
  ledger; only `T1` carries full HP/EC/acceptance-criteria detail (still
  provisional pending D1/D2 ratification); `T2`–`T6` are decomposition
  placeholders, explicitly not ready for execution.
- No new ADR was authored — see reconciliation row above.

### Acceptance criteria check

- Canonical target docs cite verified repository evidence, not model authority: met.
- All adopted claims fact-checked or rewritten: met (see table).
- Rejected/partial recommendations recorded with reasons: met.
- Normal RRI, phase-1 review, and human approval gates control downstream work:
  the canonical `S-140` plan/tasks are `proposed`, not approved — they require
  their own presentation and cross-vendor peer/D14 review before any `S-140`
  task begins execution. `T5` authored the docs; `T5` did not grant them
  approval.

### Task-analysis review for T5 itself

RRI 53 Med-high required an RRI 41+ cross-vendor phase-1 review before `T5`
could be presented for closure. Codex was invoked directly (binary resolved
from the VS Code extension bundle, since `scripts/peer-workflow-review.py`'s
built-in `codex review --stdin` invocation is broken against the current
Codex CLI — it expects a bare `-` argument, not `--stdin`; not patched in
this task, worked around by invoking the codex binary manually). Full
evidence: `.agent/peer-task-review-t5.json`.

- **Round 1 — BLOCKED (2 P2 findings):** the authored `S-140` docs claimed
  `X-S-160-3` was unblocked via the existing `review_tasks`/ADR-030 gate, but
  `review_tasks` (`infra/migrations/0014_create_review_tasks.sql`,
  `crates/domain/src/review.rs`) stores only `(project_id, asset_id,
  target_language_id)` with no derived-artifact-identity column; and `T1`'s
  acceptance criteria assumed a single `parent_artifact_id` suffices for
  subtitle lineage without addressing the open D1b branch (dedicated
  segmentation worker consuming both `TranscriptText` and `WordAlignment`).
  Both independently re-verified against the cited schema/code before
  accepting.
- **Fix round 1:** corrected the pipeline-diagram/X-S-160-3 framing (now
  explicitly "remains open," with a new risk-table row and an explanatory
  paragraph) and made `T1`'s lineage acceptance criterion conditional on
  D1a/D1b.
- **Round 2 — BLOCKED (1 P2 finding):** the `X-S-160-3` fix was accepted as
  correct, but the lineage fix was incomplete — the plan's Objective bullet
  still described lineage as pointing to "transcript/alignment artifacts"
  (plural), contradicting the newly-conditional `T1` criteria.
- **Fix round 2:** corrected the remaining dual-parent phrasing in four more
  locations (plan Objective bullet, pipeline diagram, ADR-006
  governing-constraints bullet, tasks `HP-1`), confirmed by grep that no
  dual-lineage phrasing remained except one accurate non-lineage reference
  (the `ArtifactKind` precedent citation).
- **Round 3 — PASS, zero findings.**

**Closure:** `T5` approved by owner 2026-07-20 after a PASS cross-vendor
phase-1 review (3 rounds; see `.agent/peer-task-review-t5.json`). No
Gemma Reviewer/D14 code-solution gate applies — `T5`'s deliverable is
docs-only target planning, not code. `T5` is `[x] Done`.

## ADR-037 direct-project cycle status (2026-07-20)

`T0`-`T5` are `Done`; the cycle is substantively complete. `T6` (downstream
outcome/utility trace) is left `Open` rather than forced closed: its
acceptance criteria require comparing `T4`'s recommendations against real
`S-140` implementation/review evidence, and none exists yet — `S-140-T1` has
not started (blocked on Design decisions D1/D2 ratification, then its own
presentation/approval). Recording a `T6` verdict without that evidence would
itself be the kind of unsupported claim ADR-037 §9 exists to catch.

Full `§8`/`§9` standing as of `T5` closure — including two measurement gaps
(constraint-recovery percentage not explicitly tracked; real `T4` run
throughput not re-reported after the `T1` smoke measurement) and one
near-miss worth recording honestly (two overclaims in `T4`'s adopted output —
`X-S-160-3` framing and single-parent lineage — caught only after a second
independent review round, not the first) — is recorded in the **"ADR-037
direct-project cycle summary"** section of
`docs/tasks/adr037-local-architect-direct-project.md`. This file remains
`status: in-progress` until `T6` closes.
