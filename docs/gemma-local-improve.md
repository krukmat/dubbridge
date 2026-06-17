# Local Ollama Delegation Rules

This document summarizes the **active** Low-RRI local delegation contract used in
DubBridge. It is guidance, not the governing authority.

Authoritative sources:

- `docs/policies/RRI_POLICY.md`
- `docs/playbooks/LOW_RRI_LOCAL_MODEL_HANDOFF.md`
- `scripts/delegate-low-rri.py`

## Core rule

The local model never writes files and never authors a unified diff.

It may only propose **complete final file contents** for in-scope files. The
wrapper validates the response, constructs the diff with git, and applies it only
after scope and patch checks pass.

## Active response contract

For the active Low-RRI protocol, the model must return tagged text blocks with
one header section plus zero or more file blocks:

```text
STATUS: PATCH|NO_PATCH|BLOCKED
SUMMARY: short summary
TEST: optional verification command
RISK: optional risk note
=== FILE START ===
PATH: relative/path.ext
ACTION: create|modify|delete
--- CONTENT ---
<COMPLETE final file contents>
=== FILE END ===
```

Rules:

- The model must not return JSON.
- The model must not return a unified diff.
- The model must not return partial file fragments.
- `delete` requires empty content.
- If the change cannot be expressed safely inside the allowed scope, return
  `STATUS: NO_PATCH`.

## Acceptance and rejection

Accept the response only when all of the following are true:

- every required marker is present;
- every changed path is inside the declared allowed scope;
- no extra text appears outside the permitted sections;
- no path is duplicated;
- file actions are policy-valid for the current tree state;
- the wrapper-built diff passes `git apply --check`.

Reject immediately if the response contains JSON, a unified diff, missing
markers, duplicate paths, out-of-scope paths, or invalid file actions.

## Packet discipline

Keep delegation packets small and concrete:

- one narrow objective;
- exact allowed paths;
- explicit `must change` and `must not change` rules;
- minimal relevant context;
- clear stop condition.

If the first attempt is structurally or semantically weak, run at most one
bounded repair cycle with a **smaller** scope and the failure evidence. A second
failure escalates back to the primary agent workflow.
