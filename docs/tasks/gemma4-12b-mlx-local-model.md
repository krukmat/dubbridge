---
type: TaskList
title: "Tasks: Gemma 4 12B MLX local model migration"
plan: docs/plan/gemma4-12b-mlx-local-model.md
status: superseded
rri: 51
band: Med-high
effort: L
---

# Tasks: Gemma 4 12B MLX Local Model Migration

**Superseded by ADR-036 Amendment 1 (2026-07-12):** see
`docs/plan/gemma4-12b-mlx-local-model.md`. Retained as historical record only.

## T1 - Evaluate and replace the local Gemma default

- **Status:** Done
- **Effort:** L (`scripts/rri.py` -> RRI 51, Med-high)
- **Scope:** `scripts/gemma_local.py`, `scripts/gemma_local_test.py`,
  `scripts/delegate-low-rri.py`, `scripts/delegate_low_rri_test.py`,
  `scripts/gemma-code-review.py`, `scripts/gemma_code_review_test.py`,
  `scripts/gemma-push-review.py`, `scripts/gemma_push_review_test.py`,
  `docs/playbooks/AGENT_WORKFLOW_GUIDE.md`, `docs/policies/RRI_POLICY.md`,
  `docs/gemma-local-improve.md`
- **Recommended model:** Codex Balanced -> Premium / Claude Code Balanced ->
  Premium, thinking On

### Objective

Verify `gemma4:12b-mlx` exists in Ollama's library and in the local Ollama
runtime, run a contract-oriented smoke evaluation when available, then update
DubBridge's local Gemma primary default to `gemma4:12b-mlx` while preserving the
current `gemma4:26b-a4b-it-qat` model as the automatic fallback.

### Acceptance criteria

- Ollama library evidence for `gemma4:12b-mlx` is recorded in the completion
  note.
- If the primary model is not locally installed, the task records the installation
  gap and verifies fallback resolution to `gemma4:26b-a4b-it-qat` when available.
- `scripts/gemma_local.DEFAULT_MODEL` defaults to `gemma4:12b-mlx`.
- `scripts/gemma_local.DEFAULT_FALLBACK_MODEL` defaults to
  `gemma4:26b-a4b-it-qat`.
- CLI help/default behavior for Developer, Reviewer, and Push Reviewer continues
  to resolve role-specific environment overrides before the shared default.
- Unit tests covering the shared default and wrapper fallback behavior are
  updated and pass.
- Governing workflow/RRI docs that name the old default are synchronized.

### Happy path examples

- **HP-1:** no local model override is set and `gemma4:12b-mlx` is installed ->
  Developer, Reviewer, and Push Reviewer resolve `gemma4:12b-mlx` as the shared
  primary default.
- **HP-2:** `DUBBRIDGE_REVIEW_MODEL` or `DUBBRIDGE_LOW_RRI_MODEL` is set -> the
  explicit environment override wins over the shared default.

### Edge case examples

- **EC-1:** `gemma4:12b-mlx` is absent from local `ollama list` but
  `gemma4:26b-a4b-it-qat` is present -> runtime calls fall back to the current
  model and emit a fallback notice.
- **EC-2:** the model emits invalid tagged output during smoke evaluation ->
  the migration must not claim contract compatibility without documenting the
  failure.

### Completion evidence

External availability:

- Ollama library tag verified externally for `gemma4:12b-mlx` on 2026-07-01.
- Local Ollama runtime upgraded from `0.30.10` to `0.31.1`.
- Local install: `ollama pull gemma4:12b-mlx` completed successfully after the
  Ollama upgrade.
- Local runtime smoke: `ollama run gemma4:12b-mlx` responded successfully.
- Local wrapper smoke: `scripts/delegate-low-rri.py` produced a valid `NO_PATCH`
  result using `gemma4:12b-mlx`.
- Persistent local override updated in `~/.zshrc` to
  `DUBBRIDGE_LOW_RRI_MODEL="gemma4:12b-mlx"`.
- Local fallback model `gemma4:26b-a4b-it-qat` remains installed.

### Reflection log

Required passes: 3 (`51` -> `Med-high`)

#### Pass 1

- **Draft verdict:** Primary default was changed to `gemma4:12b-mlx`, but the
  first draft only failed closed when the primary was absent.
- **Critique findings:** The user's clarified requirement needs the current model
  to remain an operational fallback, not merely a historical option.
- **Revisions applied:** Added `DEFAULT_FALLBACK_MODEL` and shared model
  resolution that falls back from `gemma4:12b-mlx` to
  `gemma4:26b-a4b-it-qat` when no explicit env override is set.

#### Pass 2

- **Draft verdict:** Developer, Reviewer, and Push Reviewer now use the resolved
  model for real invocations.
- **Critique findings:** Push Reviewer must preserve its blocked-artifact
  behavior when neither primary nor fallback is available.
- **Revisions applied:** Kept Push Reviewer's `ollama_unavailable` blocked
  artifact path around model-resolution failures.

#### Pass 3

- **Draft verdict:** Tests and docs now match the primary/fallback contract.
- **Critique findings:** Local MLX smoke generation required a newer Ollama
  runtime than the initially installed `0.30.10`.
- **Revisions applied:** Upgraded Ollama to `0.31.1`, installed
  `gemma4:12b-mlx`, verified direct generation, and verified the local delegation
  wrapper with the new model.

### Unit coverage certification

| Case ID | Type | Behavior | Unit test evidence | Result |
|---|---|---|---|---|
| HP-1 | Happy path | no override + primary installed -> shared primary default resolves to `gemma4:12b-mlx` | `scripts/gemma_local_test.py::ModelAvailability.test_resolve_model_with_fallback_uses_primary_when_installed`; `scripts/delegate_low_rri_test.py::CliBehavior.test_dry_run_uses_shared_default_model`; `scripts/gemma_code_review_test.py::CliBehavior.test_dry_run_falls_back_to_shared_default`; `scripts/gemma_push_review_test.py::EnvNamespace.test_fallback_to_shared_default_when_overrides_absent` | passed |
| HP-2 | Happy path | explicit env override wins over shared default | `scripts/gemma_code_review_test.py::CliBehavior.test_dry_run_falls_back_to_low_rri_model_env`; `scripts/gemma_push_review_test.py::EnvNamespace.test_push_review_model_takes_priority_over_low_rri`; `scripts/gemma_push_review_test.py::EnvNamespace.test_fallback_to_low_rri_when_push_review_absent`; `scripts/gemma_local_test.py::ModelAvailability.test_default_fallback_model_disabled_by_explicit_override` | passed |
| EC-1 | Edge case | primary absent + current model installed -> fallback resolves to `gemma4:26b-a4b-it-qat` | `scripts/gemma_local_test.py::ModelAvailability.test_resolve_model_with_fallback_uses_current_model_when_primary_absent` | passed |
| EC-2 | Edge case | invalid/unsafe tagged review output is rejected, so compatibility is not silently claimed | `scripts/gemma_code_review_test.py::ParseReviewResponse.test_patch_like_output_rejected`; `scripts/gemma_local_test.py::ModelAvailability.test_ensure_model_available_reports_missing_default` | passed |

### Owner final verification

- Owner: `Codex`
- Date: `2026-07-01`
- Statement: I verified every happy path and edge case defined for this task has
  unit test evidence that replicates the expected behavior. I also verified live
  MLX generation and the local delegation wrapper after upgrading Ollama.
- Commands run:
  - `python3 -m py_compile scripts/gemma_local.py scripts/delegate-low-rri.py scripts/gemma-code-review.py scripts/gemma-push-review.py`
  - `python3 -m unittest scripts.gemma_local_test scripts.gemma_code_review_test scripts.delegate_low_rri_test scripts.gemma_push_review_test`
  - `env -u DUBBRIDGE_LOW_RRI_MODEL -u DUBBRIDGE_REVIEW_MODEL -u DUBBRIDGE_PUSH_REVIEW_MODEL python3 - <<'PY' ...`
  - `env -u DUBBRIDGE_LOW_RRI_MODEL -u DUBBRIDGE_REVIEW_MODEL -u DUBBRIDGE_PUSH_REVIEW_MODEL python3 scripts/delegate-low-rri.py - --dry-run`
  - `ollama pull gemma4:12b-mlx`
  - `printf 'Return exactly: OK\n' | ollama run gemma4:12b-mlx`
  - `env DUBBRIDGE_LOW_RRI_MODEL=gemma4:12b-mlx python3 scripts/delegate-low-rri.py - --out /tmp/dubbridge-gemma4-12b-mlx-env-smoke.json --max-wall 180 --idle-timeout 60 --num-predict 256`
  - `bash scripts/check-doc-consistency.sh`
  - `bash scripts/check-task-unit-coverage.sh`
  - `bash scripts/check-roadmap-drift.sh`
  - `python3 scripts/check_okf_frontmatter.py`

### Handoff prompt

T1 - evaluate and replace local Gemma default. Governing docs:
`docs/plan/gemma4-12b-mlx-local-model.md` and this task file. Verify Ollama
library/local availability, update the shared default and directly related docs,
run focused Python tests, and stop after reporting evaluation evidence.
