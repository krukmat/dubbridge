# ADR-028: Voice-consent ledger and fail-closed TTS precondition

- **Status:** Accepted
- **Date:** 2026-06-12
- **Deciders:** DubBridge platform team
- **Closes:** X24 / X-S-110-1, X11

## Context

DubBridge's localization pipeline includes TTS synthesis and voice cloning — operations
that produce derivative audio in the voice of a specific speaker. These operations
carry distinct legal and ethical obligations beyond the general rights gate
(ADR-008): the subject of the voice must have affirmatively consented to
synthesis/cloning in the relevant scope. The platform must be unable to reach a
TTS-derivative state for any asset/speaker without a valid, unrevoked consent. The
safe failure mode is to **refuse**, never to proceed by default.

S-010 already persists `audit_events` and `rights_records`. Neither table captures
voice-cloning consent. S-110 introduces a dedicated append-only consent ledger and a
reusable fail-closed gate (`consent_gate`) that S-150 (TTS/dubbing) will call
directly when it is built — closing obligation X11 at the architecture-contract level
before the ML worker lands.

## Decision

### Voice-consent ledger is append-only

A `voice_consents` table records consent grants and revocations. Its posture mirrors
the rights ledger (ADR-008):

- Rows are **never updated or deleted**. A grant row establishes consent; a subsequent
  revoke row withdraws it.
- The **current consent status** for a (asset, scope) pair is derived from the
  **latest row** (highest `happened_at` / insertion order). This is the same
  derivation used by `rights_records` and the review-decision model.
- The table is FK-constrained to `assets`. Consent without a valid asset is rejected
  at the DB layer.

### Evidence stored by reference, never inline

Every grant row carries an `evidence_ref` — an opaque URI or external identifier
pointing to the consent proof (signed consent form, recording reference, contract
ID). The evidence bytes and any secrets are **never stored in the database** and are
**redacted from all log output** (ADR-025 spirit, ADR-018). The evidence-store
mechanism (where the referenced artifact actually lives) is an open follow-up tied to
the owner-credential secret-store decision (X20, roadmap).

### Consent scope is asset-level

A consent row covers a `scope` string identifying the synthesis type
(e.g. `voice_clone`, `tts_synthesis`) within the context of one asset. The granting
principal (`granted_by`) must be the asset owner or an authorized delegate. Scope
mismatch — consent for a different scope than the requested synthesis — is treated as
absent consent: the gate refuses, fail-closed.

### TTS precondition: `consent_gate` is fail-closed (closes X11)

`apps/api/src/services/consent_gate.rs` owns the rule:

> **No TTS or voice-cloning derivative may proceed without an active, unrevoked
> consent for the target (asset, scope) pair.**

The gate function `require_active_consent(asset_id, scope)`:

1. Queries the latest `voice_consents` row for the (asset, scope) pair.
2. If no row exists → **hard reject** + durable audit row.
3. If the latest row is a revocation → **hard reject** + durable audit row.
4. If the latest row is a grant → **allow** + durable audit row.

Every check — allowed or refused — emits an `audit_events` row (ADR-018). The gate
is implemented and unit-proven in S-110; S-150 calls it directly when built. No TTS
path may bypass or inline this check.

### Every consent mutation emits a durable audit row

Grant and revoke operations both write to `audit_events` before the consent row is
committed, using the same append-only audit path established by H1 (ADR-018). The
audit row carries `subject_id`, `asset_id`, `scope`, `event_type`
(`consent_granted` / `consent_revoked`), and a structured payload with the
`evidence_ref` (but not evidence bytes).

### Revocation is non-destructive

Revoking consent appends a revoke row; it does not modify or delete the prior grant
rows. The full consent history is always recoverable for audit purposes. This is the
same immutability guarantee as `rights_records` (ADR-008).

## Consequences

**Positive**
- A hard, testable gate for TTS/voice-cloning: absent or revoked consent → no
  derivative, enforced in the domain layer before any ML worker runs.
- Legal defensibility: an append-only, audited consent ledger with evidence
  references provides a traceable chain of custody.
- The gate is reusable: S-150 calls `require_active_consent` directly; no inline
  consent logic is permitted in the ML worker.
- Revocation takes effect immediately: the next synthesis request sees the latest
  row (revoke) and is refused.
- Evidence is stored by reference, keeping secrets and consent documents out of the
  primary database.

**Negative / trade-offs**
- Every TTS request pays one additional DB round-trip (consent lookup). Acceptable at
  current scale; a future caching layer could reduce this if necessary.
- Callers must supply an `evidence_ref` on every grant; grants without evidence are
  rejected (fail-closed). This increases intake friction intentionally.
- The evidence-store mechanism for the referenced artifacts is not specified here
  (open follow-up X-S-110-2 / X20). Until that decision is made, the platform stores
  the reference but not the artifact.

## Alternatives considered

- **Store consent in the rights ledger** — rejected: `rights_records` covers
  copyright/license provenance for the asset as a whole. Voice-cloning consent is a
  distinct legal relationship (consent of the speaker, not the rights holder) and
  requires its own typed ledger.
- **Consent checked only at TTS worker time** — rejected: the same argument as
  ADR-008 §"Rights checked only at publication": the API boundary must gate the
  request before any derivative work begins, not after the worker has been dispatched.
- **Fail-open with async consent verification** — rejected: async verification
  creates a window where unconsented synthesis could complete; an unacceptable legal
  and ethical risk.
- **Embed consent status in the asset record** — rejected: consent is append-only
  and multi-scope; a single mutable column cannot represent the audit history or
  support scope-level granularity.

## Open follow-ups

- **X-S-110-2:** Decide the evidence-store mechanism for consent proof artifacts
  (stored by reference). Ties to X20 (S-090 owner-credential secret-store); to be
  aligned when that decision is made.
- **X-S-110-3:** Real-stack verification of consent reads and gate enforcement against
  a live `voice_consents` table — operational, documented in S-110-T6, not automated
  in this slice.

## Related

- ADR-008 (rights ledger fail-closed precondition) — this ADR applies the same
  fail-closed posture to the synthesis stage.
- ADR-018 (structured observability) — audit obligation for every consent mutation
  and gate check.
- ADR-025 (platform connector ingest and owner-authorized credentials) — evidence
  stored by reference and redacted from logs follows the same principle.
- ADR-006 (PostgreSQL metadata) — `voice_consents` and `audit_events` require
  transactional storage.
- ADR-023 (API client authentication) — the consent API enforces identity before
  recording any consent row.
- X11 (roadmap cross-cutting obligation) — closed at the architecture-contract level
  by this ADR and its implementation in S-110-T2.
- Implemented by: `docs/tasks/s-110-compliance-consent-center.md` S-110-T1
  (schema + domain + repo), S-110-T2 (consent gate + audit), S-110-T3 (API).
