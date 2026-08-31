# X26-T8 implementation evidence and control notes

Date: 2026-08-31

## Scope implemented

- Added an explicit transcription deadline controlled by `ASR_TRANSCRIBE_TIMEOUT_SECONDS` with a 300-second default.
- Added a pre-model audio-size bound controlled by `ASR_MAX_AUDIO_BYTES` with a 500 MiB default.
- Added an explicit Whisper language-code allowlist; the empty string remains the documented auto-detection path.
- Added distinct structured errors for oversize audio, invalid language, transcription timeout, and invalid bound configuration.

## Verification performed without local stack

The focused Python unit suite was executed without any local service stack or real model dependency. Result: `13 passed`.

The tests cover the normal transcription path plus: oversized audio rejected before model invocation, unsupported `language_hint`, empty auto-detect hint, and a real deadline interrupt against a deliberately sleeping mocked model call.

Ruff is intentionally not installed ad hoc in the execution environment. The repository CI `python-complexity` job remains the authoritative T6/T8 complexity gate.

## Known control context

Recent X26 CI runs have an unrelated `qa-docs` failure caused by historical S-150 review `commit_sha` values that no longer resolve as Git commit objects. If that control recurs on this commit it is not treated as a T8 implementation defect; the condition is already documented in prior X26 incident notes.
