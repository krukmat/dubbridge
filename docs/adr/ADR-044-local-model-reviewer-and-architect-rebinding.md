---
type: ADR
status: Accepted
date: 2026-08-31
---

# ADR-044 — Local reviewer and architect rebinding

## Decision

Retire Muse Glimmer from every active DubBridge role. Historical ADR/audit references remain as historical truth.

- RRI 0–25 implementer remains `qwen3.8:27b-mlx`.
- RRI 26–45 local implementer remains `devstral-small-2:24b-instruct-2512-q4_K_M`.
- Low review is `gpt-oss:20b` -> Gemma -> D14.
- RRI 26–55 review is Gemma -> `gpt-oss:20b` -> D14.
- GPT-OSS uses 64K context and medium reasoning; second-review temperature is 0 and findings are evidence-bound.
- Local Architect / Complex Analyst uses `qwen3.6:27b-q4_K_M` with 64K context, temperature 0, `think=false`, `keep_alive=0`, digest `3a40c32f1450b8380412898385b0e00df5d6d2d801dd192ca1acb7e735cd050e`.
- Reviewer role switches explicitly unload the prior large model and release GPT-OSS after its attempt to minimize simultaneous residency on the 32 GB target host.

## Unchanged

RRI boundaries, cloud escalation, D14 adjudication, implementation repair budgets, reviewer independence requirements, and Devstral/Qwen3.8 implementation responsibilities remain unchanged.
