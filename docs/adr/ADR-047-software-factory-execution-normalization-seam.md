---
type: ADR
status: Accepted
date: 2026-09-13
---

# ADR-047 — Software-factory execution normalization seam

## Context

DubBridge already has authoritative software-factory behavior for RRI-based routing, authorized context, bounded local execution, fallback authorization, reviewer chains, and audit evidence.

The remaining problem is fragmentation in how already-resolved execution decisions, model/runtime bindings, local versus cloud-handoff outcomes, and execution economics are represented.

A new gateway/router would duplicate existing authorities and add unnecessary control-plane complexity.

## Decision

Adopt a minimal in-process normalization seam around the existing software-factory path.

The seam owns only:

- normalized resolved-execution constraints;
- logical binding identity separated from runtime/vendor preset values;
- normalized execution/handoff result shape;
- propagation of runtime usage metadata already exposed by the runtime;
- evidence lineage/correlation and explicit execution counters.

The seam does not own:

- RRI scoring or routing eligibility;
- context authorization/retrieval;
- local repair/test state machines;
- fallback recommendation/authorization;
- reviewer policy/verdicts;
- product-runtime or P2P behavior.

### Logical binding vs runtime preset

A Logical Binding identifies the project-facing execution role/binding. It contains no candidate priority or eligibility.

A Runtime Preset contains concrete runtime/vendor parameters such as model tag, context/generation limits, reasoning/thinking mode, and runtime class.

### Cloud boundary

Direct cloud execution is optional. A signed/authorized cloud handoff remains a valid Wave-1 endpoint. The normalization seam may reference existing fallback-selection artifacts and authorization receipts but never replaces them.

### Evidence taxonomy

Execution evidence distinguishes:

- execution session;
- model invocation;
- repair attempt;
- test attempt;
- fallback handoff.

Policy/scoring version is recorded so analytics do not silently mix incompatible RRI semantics.

## Consequences

### Positive

- model/runtime changes can be represented with less scattered workflow logic;
- existing behavior authorities remain intact;
- usage/cost/latency evidence becomes comparable without a new telemetry platform;
- cloud handoff remains fail-closed and authorization-preserving;
- the product/P2P runtime remains isolated from software-factory concerns.

### Trade-offs

- normalized contracts add a small amount of structure to existing local-agent code;
- historical records without policy-version/usage metadata remain partially comparable only;
- direct cloud execution remains outside the seam until a concrete consumer requires it.

## Rejected alternatives

- internal OpenRouter-style gateway/service;
- policy-projection engine that recomputes RRI/workflow decisions;
- routing priority inside model profiles;
- generic retry/fallback controller replacing current state machines;
- mandatory cloud adapter;
- new observability platform;
- cross-project package extraction before a second concrete consumer exists.
