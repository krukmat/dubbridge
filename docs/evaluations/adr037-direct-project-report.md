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
