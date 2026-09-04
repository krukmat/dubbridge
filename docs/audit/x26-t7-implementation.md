---
type: Audit
title: "X26-T7 Implementation Evidence"
status: recorded
related:
   - docs/tasks/tiger-style-adaptation.md
---

# X26-T7 Implementation Evidence

## Scope

Task: `X26-T7` — ASR worker guard clauses and narrow exceptions (R2/R3).

Implementation is intentionally limited to `workers/asr-worker-py/main.py` and its unit tests. No local service stack or local-agent workflow is required for this task.

## Implementation

- Added a narrow worker-error hierarchy: `InvalidInputError`, `AudioNotFoundError`, and `TranscriptionError`, all carrying a stable error code and job context.
- Replaced input `dict.get()` defaults with explicit required-field guard clauses for `job_id`, `audio_uri`, and `language_hint`.
- Required-field type/emptiness failures now raise `InvalidInputError` and map to `invalid_input`.
- Missing local audio now raises `AudioNotFoundError` and maps to `audio_not_found`.
- Faster-whisper import/model/transcription failures handled by the worker are narrowed to `ImportError`, `OSError`, `RuntimeError`, and `ValueError`, wrapped as `TranscriptionError` and mapped to `transcription_failed`.
- Removed the broad `except Exception` catch-all from the transcription path.
- Preserved the current successful transcription behavior, model-size environment setting, and language-hint pass-through; language allowlisting remains explicitly owned by `X26-T8`.

## Test coverage added/updated

- Missing `job_id`, `audio_uri`, and `language_hint` each raise the named `InvalidInputError`.
- File-not-found behavior is covered both at the subprocess error-output boundary and as a named `AudioNotFoundError`.
- A real transcription-call `RuntimeError` from the mocked faster-whisper model is covered both at the subprocess error-output boundary and as a named `TranscriptionError`.
- Existing successful transcription, language-hint pass-through, invalid JSON, and model-size behavior remain covered.

## Control policy

Per owner direction for this execution sequence, implementation closure is not blocked on local-stack or local-agent controls. Repository CI may still run automatically after the commit. Any control failure that is unrelated to this implementation should be documented separately rather than folded into T7 scope.

## Acceptance mapping

- HP-1: preserved by `test_successful_transcription_emits_output_and_exits_0`.
- EC-1: covered by `test_missing_required_field_raises_named_invalid_input` and the `invalid_input` subprocess path.
- EC-2: covered by `test_audio_not_found_uses_named_exception`, `test_audio_not_found_emits_error_and_exits_1`, `test_transcription_runtime_error_uses_named_exception`, and `test_transcription_exception_emits_error_and_exits_1`.
- Complexity gate: T7 is written under the T6 `ruff.toml` scope and remains subject to the existing `qa-python-complexity` CI gate.
