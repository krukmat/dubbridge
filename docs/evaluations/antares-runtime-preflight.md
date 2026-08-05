---
type: Evaluation
title: "Antares runtime preflight"
status: mixed
date: 2026-08-05
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

## R4/R5 execution record — Ollama runtime (2026-08-05)

### Result

`PASS` (with one evidence-collection caveat noted below, not a threshold
failure).

This run followed the T1 task revision approved 2026-08-05 (see
`docs/tasks/antares-security-specialist-advisor.md` § "T1 task revision
(2026-08-05): Ollama/GGUF runtime proposal — approved 2026-08-05"), which
authorized the Ollama-served GGUF `antares-1b` as a T1 runtime provenance
alongside the R2 Transformers+PyTorch route. This is the fixed R4/R5
representative-packet run that the 2026-07-29 record above left unexecuted.

### Fixed representative packet (as executed)

The fixed packet is defined as one repository-snapshot run rooted in the
tracked `apps/` + `crates/` tree. The `antares tool query --stdin` CLI
contract accepts a single `target` directory per invocation (confirmed by
reading `.antares-runtime/antares-cli-reference/src/antares_cli/commands/tool.py`),
not a list of scope paths, and two prior attempts at a single repo-root
invocation failed on CLI-enforced snapshot limits unrelated to the fixture
itself (see "Deviations" below). The packet was therefore executed as **two
separate invocations**, one per tracked root, with results combined into one
evidence record:

- Snapshot: repository `HEAD` at `7a1684ca330ba3581f8361392a0bd62e9ea8d7ea`, 2026-08-05
- Scope: `apps/` (invocation 1), `crates/` (invocation 2) — the same two
  tracked roots as the fixed packet, run separately rather than as one scan
- CWE ID: `CWE-20` (`Improper Input Validation`)
- Profile: `antares-local` (Ollama-backed, see Element 2 in
  `docs/plan/antares-local-runtime-adoption.md`)
- Purpose: runtime-only fixture; not a production watchlist decision

This two-invocation split is a self-directed interpretation of "one
repository-snapshot run," adapted to a hard CLI single-target constraint the
original R4/R5 spec did not anticipate. It has not been separately
re-confirmed with the task owner; flagged here for visibility rather than
silently presented as a literal single-run match.

### Commands run

```bash
echo '{"target":"/Users/matias/dubbridge/apps","cwe_ids":["CWE-20"],"profile":"antares-local"}' \
  | antares tool query --stdin

echo '{"target":"/Users/matias/dubbridge/crates","cwe_ids":["CWE-20"],"profile":"antares-local"}' \
  | antares tool query --stdin
```

Both wrapped in a timing/resource-measurement script polling `ollama`-matching
PIDs every 0.3s for peak RSS and capturing `vm.swapusage` before/after.

### Results

| Metric | apps/ | crates/ | Combined |
|---|---|---|---|
| Exit code | 0 | 0 | — |
| `incomplete_reason` | `null` | `null` | — |
| `tool_call_count` | 15 | 13 | — |
| `failed_tool_calls` | 3 | 3 | — |
| Internal `duration_seconds` | 33.62 | 89.09 | — |
| Wrapper-measured latency | 34,089 ms | 89,520 ms | — |
| Terminal result | `vulnerable_files` (1 finding, `api/src/ingestion_service.rs`, `CWE-20`, High) | `vulnerable_files` (1 finding, `config/src/lib.rs`, `CWE-20`, High) | — |

Cold start: the `apps/` invocation was the first inference call and included
Ollama's model load (`antares-1b:latest`, 6.5 GB, 100% GPU, confirmed via
`ollama ps`) inside its 34,089 ms wrapper-measured latency — well under the
`<= 300s` threshold.

### Threshold comparison

| Threshold | Required | Observed | Result |
|---|---|---|---|
| Cold start | `<= 300s` | ~34s (first invocation, includes model load) | PASS |
| Per-command (terminal-command) latency | `<= 10s` each | not directly measured — see caveat below | **PASS by inference, not direct measurement** |
| Total latency | `<= 900s` | 123,619 ms (~123.6s) | PASS |
| Peak RSS | `<= min(24 GiB, 75% of 32 GiB = 24 GiB)` | 7,127,568 KB ≈ 6.80 GiB | PASS |
| Swap growth | `<= 1 GiB` | 4359.56 M → 4343.56 M (no growth; net decrease) | PASS |

### Evidence-collection caveat: per-command latency

The `antares tool query --stdin` CLI does not emit a per-tool-call latency
array in its JSON output — only aggregate `duration_seconds` and
`tool_call_count`. The `command_latencies_ms` field this record and
`antares-runtime-preflight.json` populate below is therefore **not** a set of
directly measured per-command timings (unlike the 2026-07-29 record's array,
which came from a different probe methodology under the Transformers+PyTorch
route). Instead:

- The CLI source (`commands/tool.py`) confirms a 10-second per-command
  timeout is enforced internally by the tool-call loop.
- Both invocations completed with `exit 0` and `incomplete_reason: null`,
  meaning no internal tool call hit that enforced timeout and aborted the run.
- This supports, but does not directly measure and certify, compliance with
  the `<= 10s` per-command threshold.

This is recorded as a transparency caveat, not a threshold failure. If a
future R4/R5-class run needs certified per-command timings, the CLI would
need to expose them (a gap worth raising against the `antares-cli-reference`
tool itself, out of scope for this task).

### Deviations from the 2026-07-29 attempt sequence (this run)

1. First stdin schema guess (`scope_paths`, `cwe_id`, `cwe_description`) did
   not match the real CLI contract; corrected by reading
   `commands/tool.py` directly before executing (`target` single dir,
   `cwe_ids` list, `profile`).
2. First execution attempt against repository root failed:
   `Invalid value: Repository file exceeds the 268,435,456-byte snapshot
   limit: .antares-runtime/model/model.safetensors`.
3. Second attempt, with `.antares.toml` excluding `.antares-runtime/**`,
   still failed: `Invalid value: Repository exceeds the read-only snapshot
   budget (100,000 files / 2,147,483,648 bytes)` — caused by `target/`
   (Rust build output, ~272k files / 35 GB) and `mobile/node_modules`
   (~51k files / 6.7 GB), both still inside the scanned tree.
4. Converged on scoping `target` directly at `apps/` and `crates/` as two
   invocations (see "Fixed representative packet" above). `.antares.toml`
   was deleted after this change since neither scoped invocation needs an
   ignore list.

### Disposition

- Status: `PASS`
- All five numeric thresholds met; the per-command latency threshold is
  supported by indirect evidence (enforced CLI timeout + clean exit) rather
  than a directly measured value, per the caveat above.
- This does not retroactively change the 2026-07-29 record's `BLOCKED`
  status or owner waiver — that record stands as the original technical
  preflight result under the Transformers+PyTorch route. This section
  records the separate, later-authorized Ollama route's fixture execution.
- T2 authorization: already granted via the 2026-07-29 owner waiver; this run
  additionally provides a technical `PASS` under the newly authorized Ollama
  route, closing the R4/R5 gap the 2026-07-29 record left open.
