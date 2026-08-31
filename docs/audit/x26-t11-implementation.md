# X26-T11 implementation evidence and control notes

Date: 2026-08-31

## Scope implemented

- Added checked-in `workers/asr-worker-py/tests/fixtures/smoke-tone.wav`, a real PCM WAV fixture.
- Added `test_real_model_smoke.py`, gated by `DUBBRIDGE_ASR_REAL_MODEL_SMOKE=1`; unset means an explicit pytest skip.
- The smoke test launches the real worker subprocess without mocking `faster_whisper`, defaults to the `tiny` model, and validates the emitted success object against `output.schema.json`.
- Added `tests/README.md` documenting the opt-in command, cache behavior, and approximately 78 MB first-run download for the default tiny CTranslate2 model.

## Runtime-bound correction discovered by this task

Reviewing the real-model execution semantics exposed that faster-whisper returns a lazy segment generator. T8 originally cancelled its signal deadline after `model.transcribe()` returned, before generator iteration performed the effective transcription. `_transcribe_with_timeout` now materializes `list(segments)` before the deadline is cancelled, so the configured timeout covers the actual model work rather than only generator construction.

This correction is intentionally included in T11 because the real-runtime smoke surface is what made the mock-vs-runtime semantic gap observable.

## Verification disposition

The ungated smoke path is designed to report `skipped` when the opt-in env var is absent. The explicitly enabled real-model run was not executed in this session because it would download model weights; under the standing instruction, implementation is not blocked by that control. The test and fixture are committed so the owner/CI can run the real path later without code changes.

The known repository-wide `qa-docs` S-150 historical commit-reference issue remains unrelated and non-blocking.
