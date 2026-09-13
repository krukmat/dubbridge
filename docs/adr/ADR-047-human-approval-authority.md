---
type: ADR
status: Accepted
date: 2026-09-13
---

# ADR-047 — Human Approval Authority protocol and authenticator boundary

- **Status:** Accepted
- **Date:** 2026-09-13

## Decision

Introduce a Human Approval Authority (HAA) as an additive capability to Dubbridge. The existing `consent_gate` remains unchanged until HAA reaches its integration/value gate.

The stable protocol is authenticator-neutral:

1. an authenticated requester submits a typed `ActionSpec`;
2. HAA canonicalizes it and derives domain-separated SHA-256 action/intent digests;
3. HAA creates an `ApprovalRequest` plus a fresh random challenge;
4. HAA signs an `ApprovalChallengePackage` containing request/digests, audience, policy snapshot, display claims and expiry;
5. a registered authenticator verifies the package, presents structured claims to the human, performs local human verification and returns `ApprovalEvidence`;
6. an authenticator-specific verifier converts valid evidence into `VerifiedEvidence`;
7. HAA records the decision and issues an `ApprovalReceipt` for audit;
8. immediately before execution, the executor presents the exact `ActionSpec` and `execution_id` to atomic `authorize_and_consume`;
9. HAA rechecks digest, audience, expiry, state and immutable preconditions and atomically changes `APPROVED -> CONSUMED`, returning a short-lived `ExecutionGrant`.

`ApprovalReceipt` is audit evidence, not an execution capability. `ExecutionGrant` is separate and exists only at the enforcement boundary.

## Core invariants

- One exact action, one approval and one consumption in v0.1.
- Fresh challenges come from a CSPRNG; they are never derived from action hashes.
- Requester and executor/audience identity are explicit.
- Mutable targets use immutable/versioned preconditions so TOCTOU changes fail closed.
- Display claims derive from canonical structured action fields; requester prose is never authoritative.
- The domain never receives or stores fingerprint images/templates or Touch ID data.
- New authenticators change evidence production/verification only, never action/lifecycle/execution semantics.
- Consumption is atomic and idempotent by `execution_id`; a different execution after consumption is denied.

## Placement

- Domain contracts/state: `crates/domain`.
- PostgreSQL repositories and atomic consumption: `crates/db`.
- Audit: existing Dubbridge audit patterns.
- HTTP/service integration: `apps/api`.
- macOS authenticator: native Swift helper outside the Rust domain.
- DIY hardware authenticator: gated subplan after a real agent integration passes the value gate.

## Apple authenticator

The primary Apple implementation is a native macOS helper using Touch ID to authorize use of a device-bound Secure Enclave P-256 key. It verifies the HAA package, renders structured claims, obtains local biometric authorization, signs the package digest and returns generic `ApprovalEvidence`.

## Hardware authenticator

The hardware track is a subplan, not a parallel product. After the value gate, target a Seeed XIAO ESP32-S3 + local-matching fingerprint sensor (SparkFun FPC2534 target) + dedicated display + physical reject control over USB. A hardened iteration adds signed firmware, secure boot, flash protection/debug lockdown and a current secure-element option.

Its value is a physically separate approval boundary and trusted display. It is not claimed to be FIDO-certified, bank-grade, tamper-proof, or a qualified electronic-signature device. Firmware remains part of the trusted computing base because a secure element cannot independently prove that fingerprint matching occurred.

## Consequences

- HAA can support Apple, DIY hardware, WebAuthn/security keys or future authenticators without domain changes.
- Existing P2P consent remains operational while HAA is introduced additively.
- Exact-action enforcement requires executors to participate in `authorize_and_consume`; authentication alone is insufficient.
- Physical Touch ID and hardware tests require owner-local validation. CI still covers protocol, signatures, state transitions, replay, atomicity and adapter contracts with deterministic test authenticators.
