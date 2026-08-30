---
type: Audit
title: "P1.A1c Nemotron local-stack precheck"
task: P1.A1c
status: failed_capacity
date: 2026-08-30
---

# P1.A1c — Nemotron local-stack precheck

## Result

`nemotron-3.5-lightning:30b-a3b-q4_K_M` is not usable on this 32 GiB host for
the approved Moderate implementation route.

Before either probe, Ollama reported no loaded models, system-wide free memory
was 83%, and no pages were throttled. The full implementation profile
(`think=false`, `temperature=0`, `num_ctx=32768`, `num_predict=8192`) produced
no response content: the curl request remained blocked with a 0-byte output,
while its `llama-server` consumed enough wired memory to leave 1% free. The
reduced recovery profile (`num_ctx=16384`, `num_predict=1024`) reproduced the
same empty/stalled outcome and 1% free memory.

The task-owned curl and `llama-server` processes were terminated, then Ollama
reported no loaded models and memory recovered to 78% free with zero throttled
pages. This recovery certifies neither profile. No model replacement may be
silently substituted.

## Decision

The local implementation route is capacity-blocked. The required next step is
the ADR-039 `fallback-selection-v1` checkpoint for the exact P1.A1c fallback
packet. Its default is `human-select`; no cloud implementation may start until
the owner selects a model and reasoning effort in the matching receipt.
