---
type: TaskList
title: "Tasks: Human Approval Authority"
status: in-progress
plan: docs/plan/haa-approval-authority.md
---
# Tasks: Human Approval Authority

Governing ADR: `docs/adr/ADR-047-human-approval-authority.md`
Governing plan: `docs/plan/haa-approval-authority.md`
Behavioral coverage contract: behavior-v2

## Status legend

- [ ] Not started
- [x] Done
- [~] In progress
- [!] Blocked/manual

## Execution constraints

- Work only on `feature/haa-approval-authority`.
- Existing `apps/api/src/consent_gate.rs` remains behaviorally unchanged in this plan.
- Tasks are intentionally bounded for low-token Codex/Claude Code sessions: one concern, small file set, executable evidence, explicit handoff.
- Local-model implementation/reviewer stages in the generic workflow are replaced in this branch by direct implementation plus an independent self-review/evidence pass.

## Dependency map

`W0-T1 -> W1-T1 -> W1-T2 -> W2-T1 -> W2-T2 -> W2-T3 -> W3-T1 -> W4-T1 -> W4-T2(Value Gate) -> W5-T1 -> W5-T2 -> W6-T1 -> W7-T1`

W3-T1 and W4-T1 both depend on W2-T3 and may be prepared in parallel; W4-T2 depends on both.

---

## W0 — Foundation

### W0-T1 — Freeze HAA architecture and task graph
**Status:** [x] Done
**Effort:** S
**Depends on:** none
**Files:** ADR-047, HAA plan, this ledger

Acceptance criteria:
- [x] authenticator-neutral boundary is explicit;
- [x] hardware is gated subplan;
- [x] receipt and execution grant are separate;
- [x] atomic consumption is mandatory.

HP-1: a future authenticator can be described as an evidence adapter only.
EC-1: an implementation requiring fingerprint fields in core violates ADR-047.

---

## W1 — Universal domain core

### W1-T1 — Add canonical action/intent/digest contracts
**Status:** [~] In progress
**Effort:** M
**Depends on:** W0-T1
**Primary files:** `crates/domain/src/haa.rs`, `crates/domain/src/lib.rs`

Acceptance criteria:
- typed principals/action/intent/preconditions;
- deterministic canonical representation with explicit supported JSON subset;
- domain-separated SHA-256 action and intent digests;
- input technology does not appear in action/intent types.

HP-1: object key order does not change digest.
EC-1: semantic action change changes digest; unsupported numeric shape fails closed.

### W1-T2 — Add approval lifecycle and protocol contracts
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W1-T1

Acceptance criteria:
- request/challenge/evidence/verified evidence/receipt/execution grant types;
- legal state transitions enforced;
- challenge and protocol types carry version/domain labels;
- receipt is not accepted as execution grant.

HP-1: pending -> approved -> consumed.
EC-1: consumed -> approved and double consume are invalid.

---

## W2 — Persistence, enforcement and API

### W2-T1 — Add PostgreSQL HAA repository/schema
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W1-T2

Acceptance criteria:
- authenticators, requests, evidence/receipt and consumption state persist using existing DB conventions;
- unique constraints support nonce/execution idempotency/replay protection;
- no biometric data columns.

HP-1: registered authenticator/request round-trip.
EC-1: duplicate credential/challenge/execution identity fails deterministically.

### W2-T2 — Implement atomic authorize-and-consume
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W2-T1

Acceptance criteria:
- one DB transaction recomputes/compares digest and checks state/audience/expiry/preconditions;
- `APPROVED -> CONSUMED` atomic;
- same execution id is idempotent; competing execution id is denied.

HP-1: first valid execution gets a grant.
EC-1: concurrent/different second execution cannot get a grant.

### W2-T3 — Add HAA service/API and deterministic test authenticator
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W2-T2

Acceptance criteria:
- create/get request, challenge, submit evidence/reject, revoke authenticator, authorize-and-consume endpoints/service paths;
- requester identity and audience mandatory;
- fresh CSPRNG challenges;
- deterministic test authenticator uses the same protocol path, no core bypass.

HP-1: API/service request reaches APPROVED using test evidence.
EC-1: expired/replayed/wrong-audience evidence fails closed.

---

## W3 — Apple authenticator

### W3-T1 — Add macOS Secure Enclave authenticator helper
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W2-T3

Acceptance criteria:
- standalone Swift source/package, not Rust-domain code;
- device-bound P-256 key enrollment and public-key registration contract;
- signed package verification before rendering;
- structured display claims; Touch ID authorizes key use; signed evidence returned;
- manual physical validation checklist included.

HP-1: valid package can reach local biometric ceremony and evidence creation on supported Mac.
EC-1: invalid HAA package is rejected before biometric prompt.

---

## W4 — Real agent integration and value gate

### W4-T1 — Integrate one real Dubbridge action with HAA enforcement
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W2-T3

Acceptance criteria:
- select a bounded existing agent/API action with immutable/versioned preconditions;
- requester creates HAA intent instead of direct privileged execution;
- executor must obtain ExecutionGrant at boundary;
- existing P2P consent behavior unaffected.

HP-1: approved exact action executes.
EC-1: target/parameter/precondition mutation after approval is denied.

### W4-T2 — Value Gate security/reliability review
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W3-T1, W4-T1

Acceptance criteria:
- unit/repository/API/integration evidence reviewed;
- replay, expiry, wrong audience, stale target and double-consume fail closed;
- CI green or failures explicitly demonstrated unrelated;
- hardware recommendation recorded as PASS or DEFER.

HP-1: deterministic end-to-end approval provides useful HITL boundary.
EC-1: any ability to execute without exact valid grant blocks hardware wave.

---

## W5 — DIY hardware POC (blocked until Value Gate PASS)

### W5-T1 — Add hardware protocol/firmware skeleton
**Status:** [!] Blocked by W4-T2
**Effort:** M
**Depends on:** W4-T2 PASS

Acceptance criteria:
- target XIAO ESP32-S3 over USB;
- parse/verify signed challenge package before display;
- local fingerprint adapter interface exposes match/no-match only;
- device evidence uses the same core wire contract.

HP-1: deterministic firmware/test harness produces verifiable evidence.
EC-1: changed/unsigned challenge package never reaches approval state.

### W5-T2 — Add trusted display + fingerprint integration boundary
**Status:** [!] Blocked by W5-T1
**Effort:** M
**Depends on:** W5-T1

Acceptance criteria:
- FPC2534-class module isolated behind adapter;
- HAA-signed structured display fields only;
- explicit reject/timeout path;
- no fingerprint image/template leaves sensor boundary;
- physical test checklist records unverified items where hardware unavailable.

---

## W6 — Hardware hardening

### W6-T1 — Harden device identity/firmware lifecycle
**Status:** [!] Blocked by W5-T2
**Effort:** M
**Depends on:** W5-T2

Acceptance criteria:
- secure-boot/signed-firmware/flash/debug-lockdown design and config guidance;
- revocation/re-enrollment/lost-device path;
- optional current secure-element integration boundary (ATECC608C-class/equivalent);
- documentation explicitly states firmware remains TCB and certification limitations.

---

## W7 — Packaging

### W7-T1 — Stabilize protocol docs/examples and final review
**Status:** [ ] Not started
**Effort:** M
**Depends on:** W4-T2; hardware docs only if W6-T1 completed

Acceptance criteria:
- versioned HAA protocol documentation;
- example requester/verifier flow;
- migration/coexistence note for current consent gate;
- final architecture/security review with unresolved physical tests clearly separated.

---

## Behavioral coverage certification

To be completed as tasks finish. No task may be marked Done without executable evidence for its HP/EC cases or an explicit manual-only gate for physical hardware.
