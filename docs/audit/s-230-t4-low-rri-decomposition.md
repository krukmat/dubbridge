---
type: Audit
title: "S-230-T4 Low-RRI decomposition evidence"
task: S-230-T4
date: 2026-08-17
---

# S-230-T4 Low-RRI decomposition evidence

## Decision

The owner directed that the RRI 47 whole-task cloud route be replaced before
implementation by independently executable Low-band tasks so production-image
authoring can stay on local models. `S-230-T4` is therefore a non-executable
parent. Its aggregate acceptance contract closes only after every applicable
child below closes.

Every development child has one writable path. Test-contract tasks land before
the Dockerfile task they verify, so the later task has task-specific automated
coverage. The shared test file must remain below 500 lines; if it reaches that
limit, the next child must split a new test file and recompute RRI before
delegation. Base-image digests are operator-supplied immutable inputs in each
delegation packet; the local model does not select or update them.

The conditional translation children (`T4m`, `T4n`) execute only when
`S-150-T3b` and `S-150-T3c` are closed before worker-image integration. If that
condition is false, `T4p` records the required image-rebuild debt and `T4q` may
close without them.

## Routing

- Development children: `qwen3.8:27b-mlx` through
  `scripts/delegate-low-rri.py`, one bounded patch and at most one repair.
- Phase 1 and phase 2: `muse-glimmer:30b-q4_K_M` ->
  `gemma4:26b-a4b-it-qat` -> D14.
- Operational evidence (`T4p`) and docs closeout (`T4q`): primary orchestrator;
  they are not eligible Qwen authoring patches.
- Every child that invokes Ollama is a new restart boundary under the workflow
  guide.
- The current `scripts/rri.py` result text still says "Local Gemma via Ollama";
  its numeric result is canonical, but that routing label is stale. The
  higher-authority workflow guide binds Low implementation to Qwen and Low
  review to Muse Glimmer.

## Consolidated RRI results

All runs used platform `dubbridge`, high-confidence inputs, no penalties, and
reported `Decomposition: not triggered`. `F=0` means exactly one touched file;
`T4q` has `F=2` for three documentation files.

| Task | C | F | D | T | A | K | P | X | Final | Band / Effort |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| T4a | 0 | 0 | 1 | 2 | 0 | 2 | 0 | 1 | 15 | Low / S |
| T4b | 0 | 0 | 2 | 1 | 0 | 2 | 0 | 2 | 16 | Low / S |
| T4c | 0 | 0 | 2 | 0 | 0 | 3 | 1 | 2 | 18 | Low / S |
| T4d | 0 | 0 | 2 | 1 | 0 | 2 | 0 | 2 | 16 | Low / S |
| T4e | 0 | 0 | 2 | 0 | 0 | 3 | 1 | 2 | 18 | Low / S |
| T4f | 0 | 0 | 2 | 1 | 0 | 2 | 2 | 2 | 20 | Low / S |
| T4g | 0 | 0 | 2 | 0 | 0 | 2 | 2 | 2 | 17 | Low / S |
| T4h | 0 | 0 | 1 | 1 | 0 | 2 | 1 | 1 | 14 | Low / S |
| T4i | 0 | 0 | 2 | 1 | 0 | 3 | 1 | 2 | 21 | Low / S |
| T4j | 0 | 0 | 3 | 0 | 0 | 3 | 1 | 2 | 21 | Low / S |
| T4k | 0 | 0 | 3 | 1 | 0 | 3 | 1 | 2 | 24 | Low / S |
| T4l | 0 | 0 | 3 | 0 | 0 | 3 | 1 | 2 | 21 | Low / S |
| T4m | 0 | 0 | 3 | 1 | 0 | 3 | 1 | 2 | 24 | Low / S |
| T4n | 0 | 0 | 3 | 0 | 0 | 3 | 1 | 2 | 21 | Low / S |
| T4o | 0 | 0 | 3 | 1 | 0 | 3 | 1 | 3 | 25 | Low / S |
| T4p | 0 | 0 | 2 | 0 | 0 | 3 | 1 | 3 | 19 | Low / S |
| T4q | 0 | 2 | 0 | 0 | 0 | 1 | 0 | 2 | 10 | Low / S |

## Exact commands

```bash
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 4 --D 1 --K 2 --P 0 --T 2 --A 0 --X 1
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 3 --D 2 --K 2 --P 0 --T 1 --A 0 --X 2
python3 scripts/rri.py --touches apps/api/Dockerfile --cc 1 --D 2 --K 3 --P 1 --T 0 --A 0 --X 2
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 3 --D 2 --K 2 --P 0 --T 1 --A 0 --X 2
python3 scripts/rri.py --touches apps/gateway/Dockerfile --cc 1 --D 2 --K 3 --P 1 --T 0 --A 0 --X 2
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 3 --D 2 --K 2 --P 2 --T 1 --A 0 --X 2
python3 scripts/rri.py --touches apps/cli/Dockerfile --cc 1 --D 2 --K 2 --P 2 --T 0 --A 0 --X 2
python3 scripts/rri.py --touches workers/asr-worker-py/requirements.txt --cc 1 --D 1 --K 2 --P 1 --T 1 --A 0 --X 1
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 3 --D 2 --K 3 --P 1 --T 1 --A 0 --X 2
python3 scripts/rri.py --touches apps/worker-runner/Dockerfile --cc 1 --D 3 --K 3 --P 1 --T 0 --A 0 --X 2
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 3 --D 3 --K 3 --P 1 --T 1 --A 0 --X 2
python3 scripts/rri.py --touches apps/worker-runner/Dockerfile --cc 1 --D 3 --K 3 --P 1 --T 0 --A 0 --X 2
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 3 --D 3 --K 3 --P 1 --T 1 --A 0 --X 2
python3 scripts/rri.py --touches apps/worker-runner/Dockerfile --cc 1 --D 3 --K 3 --P 1 --T 0 --A 0 --X 2
python3 scripts/rri.py --touches scripts/test-production-images.sh --cc 5 --D 3 --K 3 --P 1 --T 1 --A 0 --X 3
python3 scripts/rri.py --touches docs/audit/s-230-t4-local-image-evidence.md --cc 1 --D 2 --K 3 --P 1 --T 0 --A 0 --X 3
python3 scripts/rri.py --touches docs/tasks/s-230-poc-v1-digitalocean.md --touches docs/plan/s-230-poc-v1-digitalocean.md --touches docs/plan/roadmap.md --cc 1 --D 0 --K 1 --P 0 --T 0 --A 0 --X 2
```

The commands correspond to `T4a` through `T4q` in order. The task ledger
records each command's purpose, writable path, dependency, behavioral examples,
verification evidence, and closure artifacts.
