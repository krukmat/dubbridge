# ASR real-model smoke test

`test_real_model_smoke.py` is deliberately opt-in because it executes the real
`faster-whisper` runtime and may download model weights.

Run it from the repository root after installing the ASR worker lockfile:

```bash
DUBBRIDGE_ASR_REAL_MODEL_SMOKE=1 \
pytest -q workers/asr-worker-py/tests/test_real_model_smoke.py
```

The default smoke model is `tiny`. Its Systran CTranslate2 repository is about
78 MB, so the first run pays that download/cache cost; later runs reuse the
Hugging Face cache. Override the smoke model with `DUBBRIDGE_ASR_SMOKE_MODEL`
when needed.

When `DUBBRIDGE_ASR_REAL_MODEL_SMOKE` is unset, pytest reports the test as
**skipped** rather than silently treating a mock as real-model coverage.
The checked-in `fixtures/smoke-tone.wav` is a valid mono PCM WAV and the test
launches the actual worker subprocess, allowing PyAV decoding, model loading,
segment generation, JSON Schema validation, and output emission to run without
mock substitution.
