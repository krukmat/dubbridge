---
type: Audit
title: "B1 implementation evidence: versioned typed task cards"
date: 2026-09-14
status: pass
---

# B1 implementation evidence

## Scope

B1 replaces the ambiguous executable `acceptance_tests: list[str]` packet with
one shared immutable task-card-v2 contract. It does not implement C1 session
provenance, F1 failure taxonomy, R1 routing changes, or E1 evidence aggregation.

## Contract and behavior

- `schema_version`, `card_id`, and `task_id` are required.
- `acceptance_criteria` contains unique stable IDs and prose statements.
- `verification_commands` contains unique command IDs, criterion references,
  and non-empty argv arrays. The runner executes those arrays directly with
  `shell=False` after applying the existing exact-command boundary.
- Unknown fields, duplicate IDs, dangling criterion references, empty argv,
  non-string argv values, and NUL bytes fail closed during card loading.
- Legacy input is accepted only through the explicit `--legacy-card` path and
  receives `source_schema: legacy-v1`. A non-empty legacy
  `acceptance_tests` list raises `LegacyTaskCardConversionRequired` before
  context retrieval or model invocation.
- Audit, escalation, and normalized-execution artifacts retain card,
  criterion, and verification-command identity.

## Verification

Focused suite (Python 3.11, `DUBBRIDGE_CONTEXT_PROVIDER=legacy`): 122 tests,
PASS. It covers the shared parser, exact argv execution and command boundary,
pre-model legacy rejection, runner integration, context budget/provider,
execution normalization, audit/escalation consumers, Med-high bundle consumer,
and Stage-1 benchmark producer.

Stage-1 benchmark producer suite: 7 tests, PASS. An explicit normalized-
evidence assertion also verifies card, criterion, and command identity survive
through `normalize_resolved_execution` and `build_execution_summary`.

`git diff --check`: PASS.

## Workflow evidence

Task-analysis review:
`gpt-oss .agent/peer-task-review-local-agent-packet-hardening-b1-attempt2.json - PASS`

Code-solution review:
`gpt-oss .agent/peer-code-review-local-agent-packet-hardening-b1.json - PASS`

The Complex profile ran 3/3 usable local passes after one earlier invalid
empty-content attempt. Four pass-specific findings were rejected against the
supplied code and green tests: the escalation loader returns a dictionary;
policy version reads the card and defaults to `rri-v2`; `allow_legacy` is
passed by keyword; and context budgeting receives dictionary payloads. No
cloud reviewer or D14 was invoked.
