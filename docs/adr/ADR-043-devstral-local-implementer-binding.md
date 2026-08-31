# ADR-043: Devstral Small 2 as the Moderate local implementer

- **Status:** Accepted
- **Date:** 2026-08-31
- **Scope:** Local-agent runtime binding and context configuration
- **Supersedes for the active runtime binding:** the Nemotron-specific binding portions of ADR-036 and ADR-038

## Context

DubBridge previously bound the Moderate local implementer and the authorized RRI 41–45 local route to `nemotron-3.5-lightning:30b-a3b-q4_K_M`. The runner also carried a 32K context ceiling introduced specifically to keep that model within the host memory envelope.

The replacement is intended as a direct software-engineering change on the normal product path rather than a POC, A/B experiment, shadow route, or new model-selection layer.

## Decision

Use the following pinned local implementer binding for the existing Moderate and authorized Med-high local path:

```text
devstral-small-2:24b-instruct-2512-q4_K_M
```

Use a normal local-implementer context baseline of:

```text
131072 tokens (128K)
```

The runtime constants are defined in:

- `scripts/local-agent/run_local_task.py`
  - `MED_HIGH_REQUIRED_MODEL`
  - `MODEL_CONTEXT_TOKENS`
- `scripts/local-agent/cli.py`
  - `_DEFAULT_MODEL_CONTEXT_TOKENS`

The runner and CLI defaults must remain synchronized.

## Preserved behavior

This ADR does not change:

- the Qwen binding for RRI 0–25;
- RRI boundaries;
- the existing RRI 41–45 authorization semantics;
- the cloud-only behavior for RRI 46+;
- reviewer bindings or reviewer ordering;
- `MAX_TOTAL_TURNS = 30`;
- `MAX_REPAIR_ATTEMPTS = 2`;
- `MAX_MALFORMED_BOUNCES = 3`;
- `GENERATION_TOKEN_BUDGET = 8192`;
- the structured JSON tool-call contract;
- `think = false`;
- runner-controlled acceptance tests and boundary enforcement.

Nemotron is not retained as an automatic fallback by this change.

## Context policy

128K is the normal configured baseline for Devstral. The runtime should use the available budget for relevant authorized context rather than retaining the former Nemotron-specific 32K ceiling.

A reduced context is a recovery behavior, not the normal configuration. If real host execution later demonstrates a reproducible capacity problem, context may be reduced independently without changing the model-selection decision in this ADR.

## Tests

` scripts/local-agent/devstral_binding_test.py ` provides regression coverage for the key software contracts:

- Low RRI continues to resolve to Qwen;
- Moderate resolves to Devstral;
- authorized RRI 41–45 requires the Devstral binding;
- RRI 46+ retains its existing cloud-only behavior;
- runner and CLI use the same 128K context default.

These are software-level binding regressions. This ADR does not require a model-quality benchmark or local-model workflow exercise as a prerequisite to adopting the binding.

## Consequences

### Positive

- removes the active Nemotron dependency from the Moderate local implementer path;
- removes the 32K context limit that existed specifically for the previous model;
- keeps the replacement isolated from routing, reviewer, and tool-contract changes;
- provides explicit regression coverage for the active binding and context defaults.

### Trade-offs

- 128K can increase memory pressure relative to the previous 32K profile;
- model-quality and host-runtime behavior will be learned from normal use rather than a pre-adoption POC;
- later tuning may adjust context or operational model lifecycle independently.

## Migration rule

Historical ADR, audit, and evaluation records that describe Nemotron remain historically correct and must not be rewritten. Current runtime code and current software-facing documentation should use the Devstral binding defined here.
