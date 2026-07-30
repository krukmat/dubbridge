---
type: Evaluation
title: "Antares runtime preflight"
status: blocked
date: 2026-07-29
task: T1
plan: docs/plan/antares-security-specialist-advisor.md
---

# Antares Runtime Preflight

## Result

`BLOCKED`

T1 did not prove a runnable, pinned Antares path on the current self-hosted macOS
ARM64 environment.

Technical gate result: the preflight remains `BLOCKED`.

Owner disposition on 2026-07-29: explicit waiver to accept T1 as sufficient to
proceed. This artifact therefore records a failed technical preflight that was
owner-accepted for workflow continuation; it does not convert the runtime result
into a technical pass.

## Fixed representative packet

This packet was fixed before runtime acquisition to satisfy the T1 preflight
contract.

- Snapshot: current repository `HEAD` on 2026-07-29
- Scope: tracked `apps/` + `crates/` tree only
- CWE ID: `CWE-20`
- CWE description: `Improper Input Validation`
- Purpose: runtime-only fixture for bounded compatibility proof; **not** a
  production watchlist decision

## Host facts

- Host OS: `macOS 26.5.2 (25F84)`
- Host architecture: `arm64`
- Host CPU: `Apple M5`
- Host RAM: `34359738368` bytes (`32 GiB`)
- Existing local model stack:
  - `ollama list` succeeds and shows local DubBridge models
  - `OLLAMA_HOST` is set to `127.0.0.1:11434` (bare host, no scheme)

## Probes run

### Access / provenance

```bash
python3 - <<'PY'
import os
for name in ['HF_TOKEN', 'HUGGINGFACE_HUB_TOKEN', 'HUGGING_FACE_HUB_TOKEN']:
    print(f"{name}={'set' if os.environ.get(name) else 'unset'}")
PY
```

Observed: all three variables were `unset`.

```bash
curl -I -L --max-time 20 \
  https://huggingface.co/fdtn-ai/antares-1b/resolve/main/config.json
```

Observed: `HTTP/2 401`, `x-error-code: GatedRepo`,
`x-error-message: Access to model fdtn-ai/antares-1b is restricted.`

### Runtime availability

```bash
python3 - <<'PY'
import importlib.util
for name in ['huggingface_hub', 'transformers', 'vllm', 'sglang']:
    print(name, importlib.util.find_spec(name) is not None)
PY
```

Observed: all four packages were absent from the active Python environment.

```bash
docker info
```

Observed:

- Docker daemon is available through `colima`
- server architecture: `aarch64`
- server memory: `5.772 GiB`
- `docker model` is not exposed as a usable subcommand on this host, so Docker
  Model Runner is not currently a verified runtime path

```bash
ollama list
```

Observed: local Ollama is reachable and already hosts DubBridge-local models
(`qwen3.6:27b-q4_K_M`, `qwen3.6:35b-a3b`, `gemma4:26b-a4b-it-qat`, etc.).
This does **not** satisfy T1 because Antares itself is not approved or pinned in
Ollama, and T1 forbids silent runtime substitution.

## Technical blockers

1. The original unauthenticated host state returned `GatedRepo`; that access
   blocker was later cleared by the owner, but it still explains the first failed
   probe and why the preflight required a rerun.
2. The authenticated local route did not produce a proven within-threshold runtime
   on this macOS ARM64 host. `MPS` was not stable for this checkpoint in the
   tested runtime path, and the CPU fallback exceeded the T1 total-latency budget.
3. Because the local runtime did not meet the stated thresholds, T1 still failed
   as a technical gate even after lawful acquisition and artifact pinning became
   possible.

## Disposition

- Status: `BLOCKED`
- Blocked reason:
  `local_runtime_unproven; mps_unstable; cpu_latency_over_threshold`
- Owner waiver:
  explicit owner instruction on 2026-07-29 to accept T1 as sufficient to proceed
  despite the blocked technical gate
- T2 authorization:
  allowed by owner waiver; local `antares-1b` runtime is still not a validated
  host path

## Unblock conditions

1. One supported local runtime path must be installed and pinned for Antares on
   macOS ARM64.
2. The representative packet above must then execute successfully with:
   cold start `<= 300s`, each command latency `<= 10s`, total latency `<= 900s`,
   peak RSS `<= min(24 GiB, 75% of physical RAM)`, and swap growth `<= 1 GiB`.
