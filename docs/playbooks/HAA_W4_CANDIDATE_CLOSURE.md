---
type: Playbook
status: candidate
---

# W4 candidate plan closure

The candidate module builds a deterministic closure manifest from a trusted
snapshot and fresh Git facts. It writes request/consume markers before
network calls, verifies detached HAA grants against the operator key pin,
then re-reads the snapshot and atomically persists PLAN_DONE plus grant digest
in a private HMAC-sealed **candidate** journal.

A pending, rejected, stale, uncertain or invalid grant never creates a done
record. Existing terminal/in-flight records are not automatically retried.

The real plan/task source of truth is not connected: Markdown ledgers
currently do not provide an atomic runtime state machine. The trusted
controller must establish a canonical source and projection and an operation
lock covering snapshot through side effect. This candidate is not
authorization to change a real plan or any Git state. No production CLI or
activation flag is provided. The prior W2 Touch ID smoke used synthetic
closure state and cannot certify this implementation.
