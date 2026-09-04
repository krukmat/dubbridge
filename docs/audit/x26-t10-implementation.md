---
type: Audit
title: "X26-T10 implementation evidence and control notes"
status: recorded
related:
   - docs/tasks/tiger-style-adaptation.md
---

# X26-T10 implementation evidence and control notes

Date: 2026-08-31

## Scope implemented

- Added `workers/asr-worker-py/requirements-lock.txt` with exact Python 3.12 runtime pins for all direct and transitive dependencies reachable from `faster-whisper==1.1.0`, `requests==2.34.2`, and `jsonschema==4.26.0`.
- Docker now installs exclusively from the lockfile with `pip install --no-deps` and immediately executes `pip check`, preventing image-build-time dependency resolution drift and failing closed if the lock is incomplete.
- Docker now copies `input.schema.json`, `output.schema.json`, and `error.schema.json` into `/app`; this fixes the T9 container-packaging omission discovered while wiring the reproducible image build.
- `requirements.txt` remains the concise direct-dependency manifest and the lockfile records an explicit regeneration command for future dependency bumps.

## Resolution basis

The lock was resolved for Python 3.12 from package metadata plus known compatible faster-whisper 1.1.0 environments. Key pinned runtime components include CTranslate2 4.6.0, tokenizers 0.21.1, ONNX Runtime 1.21.0, PyAV 14.2.0, huggingface-hub 0.29.3, and their exact supporting packages.

## Verification disposition

Per the standing execution instruction, no local Docker/service stack was started. The Dockerfile is intentionally self-validating with `pip check`; any remote image build that finds an omitted or incompatible dependency fails rather than silently resolving another version.

The known repository-wide `qa-docs` S-150 historical commit-reference issue remains unrelated and non-blocking for T10.
