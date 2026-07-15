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

Done. See `docs/tasks/adr036-local-first-pilot.md` T3 completion evidence for
the full test/coverage trail (8/8 mocked unit tests); this report only tracks
Stage 1's live-data questions, and T3 shipped a script, not live-soak data —
no numeric results to duplicate here beyond what T1 already observed.

## T4 — Dev-stack contention soak

**Done.** Two runs: the first failed entirely (120/120 samples errored) due to
two bugs found and fixed during this task; the second (corrected) run
completed cleanly.

### Bugs found and fixed

1. **`OLLAMA_HOST` scheme normalization.** The machine's `OLLAMA_HOST` is set
   to `127.0.0.1:11434` (no `http://` scheme) — valid for Ollama's own CLI,
   but `urllib.request` rejects any URL without an explicit scheme
   (`unknown url type: 127.0.0.1`). Every sample in the first run failed on
   this. Fixed by adding `normalize_host()` (prefixes `http://` when no
   `://` is present) to `measure_inference.py`, `measure_residency.py`, and
   `soak_contention.py`. This bug was latent in T2/T3 as well — it simply
   never fired there because those runs were exercised with an explicit
   `http://localhost:11434` `--host` value in manual testing, not the bare
   environment default.
2. **`llama-server` excluded from peak-memory sampling.** Ollama's actual
   inference engine runs as a separate `llama-server` child process (tens of
   GB RSS); the parent `Ollama`/`ollama serve` processes are only a few MB.
   `ollama_process_rss_bytes()` filtered on `"ollama" in name.lower()`,
   which excluded `llama-server` entirely, undercounting peak memory by
   ~3 orders of magnitude (11 MB instead of ~24 GB) once `psutil` was
   installed (it was previously absent, masking the bug behind the
   `/api/ps size_vram` fallback path in T1–T3). Fixed by matching on either
   `"ollama"` or `"llama-server"` in the process name
   (`OLLAMA_PROCESS_NAME_MARKERS`). A regression test
   (`OllamaProcessRssBytes` in `measure_inference_test.py`) now locks this in.
   `psutil` was also installed in this environment (`pip3 install --user
   psutil`) as part of this fix — previously absent, which is why the swap
   field returned `null` in the first (also-failed) run.

Both fixes verified live against Ollama before the corrected soak was
launched; full test suite re-run after the fix: 29/29 passed
(`measure_inference_test.py` + `measure_residency_test.py` +
`soak_contention_test.py`).

### Corrected run results

- Model: `qwen3.6:35b-a3b`; duration: 3600s; interval: 30s; 120/120 samples
  succeeded (`failed_sample_count: 0`).
- `min_decode_tok_s`: 34.0 · `median_decode_tok_s`: 42.35
- `peak_memory_bytes`: 24.41 GB (stable 24.35–24.38 GB throughout the full
  hour — no growth, no leak)
- `peak_swap_used_bytes`: 7.97 GB

**Swap baseline caveat:** before this corrected run started, swap was already
at ~6.66 GB / 93% used — residual from the first (failed) run's hour of
Docker + cargo-build contention, which macOS does not release without memory
pressure dropping or a process/reboot cycle. Reading the time series directly
(sampled every 5th point): swap moved from **7.00 GB → 7.57 GB** over the
hour — a **~0.57 GB delta attributable to this run**, not the full 7.97 GB
peak in isolation. Decode throughput showed no sustained degradation trend
(bounces between ~34–46 tok/s throughout, no downward slope from start to
end); the 34.0 tok/s minimum appears to be a single transient dip, not a
trend.

### Contingency verdict (ADR-036 §6)

**Important scope correction:** this task only soaked `qwen3.6:35b-a3b`
against the contention scenario. **`gemma4:26b-a4b-it-qat` was never run
through the same 1-hour soak** — it only has the single-prompt smoke test
from T1 (no concurrent Docker/cargo load, no sustained sampling). ADR-036 §6
frames this as a binary choice ("stay with Qwen vs. demote to Gemma"), which
made it easy to read a passing Qwen-only result as if it had beaten Gemma
head-to-head. It did not: Gemma was never tested under the same conditions,
so there is no comparative evidence between the two — only evidence that
Qwen alone did not fail its own soak.

**Verdict: no demotion triggered, on the evidence available.**
`qwen3.6:35b-a3b` showed no failure signal (flat memory, no sustained
throughput degradation, modest attributable swap growth) under a full hour of
real contention (Postgres + Redis + MinIO via Docker, plus a repeating
`cargo build --workspace` loop). That is sufficient to clear the §6 trigger
condition (which is about Qwen's own viability, not a comparative bake-off),
so the primary-implementer binding stands. It is **not** sufficient to claim
Qwen is "better than" Gemma under contention — that comparison was never run.

Rationale: the swap growth directly attributable to this run (~0.57 GB) is
modest; the high swap baseline observed is a macOS memory-compressor artifact
from the prior failed run's contention, not evidence that Qwen3.6-35B-A3B
itself destabilizes the dev stack. This is a single 1-hour sample on one
machine state, for one of the two bindings. T9 (Stage 2 pilot) will be the
first place this verdict gets tested against real multi-hour, multi-day
working sessions — if swap or throughput degrades over longer horizons than
this soak covered, the verdict should be revisited before promotion (T10).

**Open gap, not closed by this task:** a head-to-head contention soak of
`gemma4:26b-a4b-it-qat` under the identical scenario (same Docker+cargo load,
same duration, same sampling) has not been run. If a comparative
primary-vs-contingency verdict is ever needed (rather than just "does the
primary clear its own bar"), that requires a second soak against the Gemma
binding — not assumed from this task's Qwen-only data. This gap should be
closed before T10 treats the §6 binding choice as fully validated, or
explicitly accepted as a known limitation if T9's pilot results make it moot.

## T5 — Benchmark corpus

Not started.

## T6 — Runner (a/b/c/d)

Not started.

## T7 — Stage 1 benchmark run

**Stopped by owner after 14/16 completed sessions. This run is diagnostic
baseline evidence, not promotion evidence.** Command allowlisting dominated
the failures, so continuing the final two cards would not answer the intended
model-productivity question. The owner directed the pilot to move to the
offline-productivity correction defined in ADR-036 §3.

| Card | Raw status | Wall-clock | Test attempts | Raw boundary count |
|---|---|---:|---:|---:|
| CC-01 | success | 247.014s | 1 | 0 |
| CC-02 | success | 28.928s | 1 | 0 |
| CC-03 | boundary_violation | 86.404s | 0 | 1 |
| DC-01 | success | 399.095s | 1 | 0 |
| DC-02 | boundary_violation | 91.833s | 0 | 1 |
| MC-01 | boundary_violation | 713.120s | 1 | 1 |
| MC-02 | boundary_violation | 262.881s | 0 | 1 |
| MC-03 | budget_exhausted | 73.076s | 3 | 0 |
| MC-04 | boundary_violation | 32.483s | 0 | 1 |
| RC-01 | boundary_violation | 42.320s | 1 | 1 |
| RC-02 | boundary_violation | 235.641s | 1 | 1 |
| RC-03 | boundary_violation | 39.715s | 1 | 1 |
| RC-04 | boundary_violation | 12.949s | 0 | 1 |
| RF-01 | boundary_violation | 211.277s | 0 | 1 |
| RF-02 | interrupted by owner | — | — | — |
| RF-03 | not attempted | — | — | — |

Raw completed-session totals: **3 success, 10 boundary_violation, 1
budget_exhausted**. All 14 completed sessions have matching transcript and
audit JSONL records. `RF-02` was interrupted before either artifact was
emitted; its partial worktree contained no recorded diff and was deleted.
`RF-03` never started. No result was rewritten after the stop.

The ten raw `boundary_violation` statuses all came from the command-policy
allowlist (`cat`, direct Python invocation, `cd`/shell composition, `wc`, or
`find`), not from a diff escaping into the primary checkout. They remain raw
baseline outcomes, but ADR-036 no longer uses command vocabulary as the safety
or promotion boundary for the offline pilot.

## T7 — Real-task evidence lane (`T7f` + `T7k`)

This section is the post-corpus evidence lane. Unlike the retired 16-card
benchmark, these are single real tasks run from fresh disposable worktrees and
classified individually as either trustworthy promotion evidence or
diagnostic-only evidence.

| Trial | Task focus | Raw status | Wall-clock | Classification | Promotion-usable? |
|---|---|---:|---:|---|---|
| `T7f` | `scope_check.py` false-positive fix for ignored runner artifacts | success | see task ledger | trustworthy real-task success | Yes |
| `T7K-01` | Gemma-band `findings -> exit 0` in `peer-workflow-review.py` | success | 258.7s | diagnostic-only: whole-file rewrite, unrelated string churn, no targeted regression test | No |
| `T7K-02` | D14 fallback import repair in `peer-workflow-review.py` | success | 293.4s | diagnostic-positive: scoped diff, but still unrelated string churn and no dedicated regression test | No |
| `T7K-03` | stale-import false-negative for runner self-edits | transport_error | 501.9s | diagnostic failure: no diff, token-limit transport collapse after repeated exploration | No |

### Current reading

- The evidence lane is **better than the retired corpus** because each run is
  a real task with explicit transcripts and, where preserved, the exact
  worktree diff.
- It is **not yet strong enough for T8**. Only `T7f` is currently clean enough
  to cite as trustworthy promotion evidence.
- `T7K-03` adds an important failure mode: when the task asks the local runner
  to repair one of its own import/scope semantics, convergence cost rises
  sharply and transport fragility (`response cut by token limit`) becomes a
  first-order failure class.
- Net result after `T7k`: the real-task lane is useful for diagnosis, but it
  has not yet produced enough clean wins to populate the promotion-gate table.

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
| Accepted out-of-scope diffs | 0 | — | — |
| Wall-clock vs cloud equivalent | ≤ 2× | — | — |
| Cloud-token reduction | measured > 0 | — | — |

GO/NO-GO: **pending T8.**
