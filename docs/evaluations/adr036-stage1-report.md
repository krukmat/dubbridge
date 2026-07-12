---
type: Evaluation
title: "ADR-036 Stage 1 measurement report"
date: 2026-07-12
status: in-progress
---

# Evaluation: ADR-036 Stage 1 — local stack measurement

Tracks the raw data gathered under `docs/tasks/adr036-local-first-pilot.md`
(T1–T4, T7) toward the T8 go/no-go synthesis. Sections are filled
incrementally as each task completes; nothing here is a final verdict until
T8.

## T1 — Model bindings

Both ADR-036 bindings were **already installed** on the target machine when
T1 ran (2026-07-12), discovered via `curl localhost:11434/api/tags`:

| Binding | Tag | Params | Quant | Format | Size on disk | Context (advertised) |
|---|---|---|---|---|---|---|
| Local implementer | `qwen3.6:35b-a3b` | 36.0B (3B active) | Q4_K_M | **GGUF** (llama.cpp backend, not MLX) | 23.9 GB | 262144 |
| Local reviewer/multimodal | `gemma4:26b-a4b-it-qat` | 25.2B (3.8B active) | Q4_0 | **GGUF** (llama.cpp backend, not MLX) | 15.6 GB | not captured in this call |
| Fast lane (pre-existing) | `gemma4:12b-mlx` | 12B | safetensors | **MLX native** | 7.7 GB | — |
| Rejected per ADR §1 (present but unused) | `gemma3:27b` | 27.4B | Q4_K_M | GGUF | 17.4 GB | — |

**Deviation from ADR-036 assumption:** the ADR's memory/speed reasoning
assumed MLX 4-bit builds for the two large bindings (open question 5). The
installed builds are actually **GGUF via Ollama's llama.cpp backend**, not
MLX. `gemma4:12b-mlx` is the only true MLX build present. This does not
block the pilot (Ollama's llama.cpp backend is a supported path per the
ADR's own runtime discussion), but it means:
- the "MLX beats llama.cpp by 20–50%" literature cited in the ADR's
  supporting research does **not** apply to the current bindings as run;
  measured throughput below is the real llama.cpp-backend number, not an
  MLX-upper-bound estimate.
- T2 should record backend (`gguf` vs `mlx`) per measurement so a future MLX
  swap-in is comparable against this baseline rather than conflated with it.

No contingency triggered at T1 — both bindings loaded and answered without
error. Residency check (`/api/ps`) confirms only one large model resident at
a time under normal use (see below), consistent with ADR-036 §6.

## T1 — Smoke test results

| Model | Prompt | eval_count (tokens) | decode tok/s | prompt_eval_count | TTFT-adjacent (prompt_eval_duration) |
|---|---|---|---|---|---|
| `qwen3.6:35b-a3b` | "Reply with exactly one word: OK" | 114 | **41.44** | 17 | 462.7 ms |
| `gemma4:26b-a4b-it-qat` | "Reply with exactly one word: OK" | 73 | **45.36** | 23 | 221.8 ms |

Both figures are **cold-load-inclusive wall-clock is higher** (15.7s and 6.9s
total curl time respectively) — the decode tok/s above is computed from
Ollama's own `eval_duration` (generation-only), excluding model load time.
This single short-prompt sample is *not* the T2 benchmark; it only confirms
both bindings are live and in the interactive range. It does **not** yet
measure prefill at 8K/16K/32K (open question 2) — that is T2's job.

Observed decode throughput on this machine **exceeds** the ADR's conservative
30–50 tok/s estimate range at the upper end for both models, which is a
positive early signal but based on a 17–23 token prompt only; sustained
throughput at longer context is unverified until T2/T4 run.

## T1 — Residency check

`/api/ps` immediately after both smoke tests showed only `gemma4:26b-a4b-it-qat`
resident (15.04 GB VRAM), confirming Ollama's default unload-after-idle
behavior keeps at most one large model loaded without any explicit
`keep_alive` management from the pilot tooling. This is consistent with, but
does not yet fully validate under load, the ADR-036 §6 residency rule.

## T2 — Inference measurement (decode/prefill/TTFT/peak memory)

Not started.

## T3 — Load-cycle and residency measurement

Not started. Partially informed by the T1 residency observation above, but
T3 must still measure explicit cold-load and reload timings under
`keep_alive: 0`.

## T4 — Dev-stack contention soak

Not started.

## T5 — Benchmark corpus

Not started.

## T6 — Runner (a/b/c/d)

Not started.

## T7 — Stage 1 benchmark run

Not started.

## Open questions status (ADR-036)

| # | Question | Status |
|---|---|---|
| 1 | Wired-memory fit under full dev stack | Not yet answered — needs T4 |
| 2 | Prefill tok/s at 8K/16K/32K + KV reuse across turns | Not yet answered — needs T2 |
| 3 | Actual KV-cache footprint of Qwen3.6 hybrid attention at 32K | Not yet answered — needs T2 |
| 4 | Harness choice | Resolved in `docs/plan/adr036-local-first-pilot.md` design decision 1 (bespoke thin wrapper) |
| 5 | `qwen3.6:35b-a3b` availability in Ollama library | **Resolved: available**, but as GGUF Q4_K_M, not MLX — see deviation note above |

## Promotion-gate table (filled at T8)

| Gate | Threshold | Measured | Pass? |
|---|---|---|---|
| Task success without escalation | ≥ 75% | — | — |
| Avg. repair attempts | ≤ 2 | — | — |
| Scope violations | 0 | — | — |
| Boundary violations | 0 | — | — |
| Wall-clock vs cloud equivalent | ≤ 2× | — | — |
| Cloud-token reduction | measured > 0 | — | — |

GO/NO-GO: **pending T8.**
