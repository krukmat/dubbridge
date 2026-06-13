# Plan: S-110 — Compliance & Consent Center

> **Status:** Complete 2026-06-13 — T0–T3, T5, and T6 done; T4 cancelled and
> superseded by T5 under S-105. Authored 2026-06-11; replanned mobile-only 2026-06-13.
> **Roadmap phase:** `S-110` — governance/product phase before
> `S-150`. Surfaces the immutable audit trail and rights ledger to content owners,
> and adds the voice-cloning consent ledger that gates TTS derivatives
> (cross-cutting obligation X11).
> **Tasks ledger:** `docs/tasks/s-110-compliance-consent-center.md`.

## Purpose

Traceability and authorization are structural in DubBridge: every governance event is
logged and every artifact has a traceable origin (`README.md`). S-010 already persists
this — `audit_events`
([migration 0004](/Users/matias/Documents/projects/dubbridge/infra/migrations/0004_create_audit_events.sql))
and `rights_records`
([migration 0002](/Users/matias/Documents/projects/dubbridge/infra/migrations/0002_create_rights_records.sql)).
But none of it is **visible to the people who need it** (owners, compliance teams),
and a known obligation is unmet: **X11 — enforce consent and voice-cloning permissions
before TTS derivatives**, which currently has no home.

This slice builds the **compliance & consent product layer**: a read-only audit/rights
viewer scoped to the owner/org, and an append-only **voice-consent ledger** with a
fail-closed precondition that S-150 (TTS/dubbing) will enforce. It turns DubBridge's governance
into a visible, sellable trust feature.

## Objective

Deliver a governed compliance and consent surface:

- **Audit timeline**: an owner/org-scoped, read-only view of `audit_events` per asset.
- **Rights ledger viewer**: surface the existing append-only `rights_records`.
- **Voice-consent ledger**: append-only grants/revocations for voice cloning/synthesis,
  with evidence stored by reference and redacted (ADR-025 spirit).
- **Consent precondition (X11)**: a fail-closed gate that blocks any TTS/voice-cloning
  derivative without a valid, unrevoked consent — defined and unit-proven now, enforced
  when S-150 is built.
- **Surface**: a complete mobile compliance and consent center.
- **Prove it**: Gherkin BDD mapped to mobile component/Maestro evidence and backend
  ownership/consent-gate tests.

## Scope decisions (confirmed 2026-06-11)

| Decision | Choice |
|---|---|
| Feature scope | Read-only audit/rights viewer + append-only voice-consent ledger + fail-closed TTS precondition + complete mobile center |
| Audit/rights posture | **Read-only.** S-110 never mutates `audit_events` or `rights_records`; it only reads them, ownership/org-scoped |
| Consent posture | `voice_consents` is append-only (grant/revoke rows); current status is derived from the latest row (mirrors rights ledger, ADR-008) |
| Enforcement | The consent precondition (X11) is built and unit-tested here as a reusable gate; S-150 calls it when TTS lands (forward dependency) |
| BDD home | `docs/bdd/p6-compliance.feature`; mapped to mobile flows, component tests, backend ownership/gate evidence, and `HP-#`/`EC-#` |

## Affected components

| Layer | Path | Change |
|---|---|---|
| BBDD (schema) | `infra/migrations/0017_create_voice_consents.sql` | `voice_consents` (append-only grant/revoke, scope, evidence ref) |
| Backend domain | `crates/domain/src/consent.rs` (new) | Consent entity + status derivation + scope model |
| Backend DB | `crates/db/src/consent_repo.rs` (new) | Append-only consent repo + latest-status query |
| Backend service | `apps/api/src/services/consent_gate.rs` (new) | Fail-closed TTS-derivative precondition (X11) + audit |
| Backend API | `apps/api/src/routes/compliance.rs`, `dto/compliance.rs` (new) | Ownership-scoped audit/rights read + consent grant/revoke |
| Backend DB (read) | `crates/db/src/audit_repo.rs`, `rights_repo.rs` | Add ownership/org-scoped read queries (no mutation) |
| Mobile | `mobile/src/screens/ConsentScreen.tsx`, `ComplianceScreen.tsx`, nav | Timeline, rights ledger, consent history/current state, grant/revoke, and error states |
| E2E backend | `scripts/e2e-seed/mock-gateway-server.mjs` | `/api/*` compliance/consent fixtures |
| BDD | `docs/bdd/p6-compliance.feature`, `docs/bdd/README.md` | Cross-surface Gherkin specs + mapping |

## Design decisions

### D1 — Voice-consent ledger is append-only

`voice_consents` records consent to synthesize or clone a given voice/speaker within a
scope (asset and/or org), with `granted_by`, an evidence **reference** (not the
evidence bytes), and a `status` expressed through append-only rows (grant, then
optionally revoke). Current consent status is the **latest row**, mirroring the
rights-ledger and review-decision posture (ADR-008). Evidence is stored by reference
and redacted in logs (ADR-025, ADR-018).

### D2 — Consent precondition is fail-closed (X11), built ahead of S-150

`consent_gate.rs` owns the rule: **no TTS / voice-cloning derivative may proceed
without a valid, unrevoked consent for the target voice/scope.** This is the intake-
edge twin of the rights gate (ADR-008) for the synthesis stage. S-150 (TTS/dubbing) is not built;
S-110 implements and unit-proves the gate now so S-150 calls it directly when it lands —
closing X11 at the contract level without waiting for the ML worker.

> **Open follow-up (X-S-110-1):** author an ADR for the voice-consent ledger + TTS
> precondition (the X11 decision). Recorded; no number claimed here.

### D3 — Audit/rights viewer is strictly read-only and ownership-scoped

The compliance read API serves `audit_events` and `rights_records` filtered to the
caller's owned assets / org. It **never writes** governance rows — it is a window onto
the existing immutable ledgers. Ownership scoping is the same fail-closed default S-060
(D1) applies to the asset list: a caller sees only their own governance trail.

> **Observation:** this read surface inherits the open `GET /assets/{id}` ownership
> question (S-060 follow-up X-S-060-1). The compliance reads are ownership-scoped from
> the start; the by-id reconciliation remains that slice's follow-up, not changed here.

### D4 — Mobile compliance and consent center + BDD mapping

Mobile shows the chronological audit timeline, rights ledger, consent history and
current state, and grant/revoke actions. It handles loading, empty, forbidden,
network-error, and expired-session states. `testID` values are the UI-flow contract;
ownership and synthesis authorization remain backend-enforced.

## Module dependency direction

```mermaid
flowchart TD
    T0["S-110-T0 · BDD specs + mapping<br/>(RRI 11)"]
    T1["S-110-T1 · schema + domain + repo<br/>(voice_consents, RRI 58)"]
    T2["S-110-T2 · consent ledger + TTS precondition + audit<br/>(X11, RRI 66)"]
    T3["S-110-T3 · compliance read API<br/>(audit/rights viewer, RRI 44)"]
    T4["S-110-T4 · web dashboard<br/>cancelled / superseded"]
    T5["S-110-T5 · mobile compliance + consent center<br/>(RRI 41)"]
    T6["S-110-T6 · mock fixtures + Maestro + docs"]

    T0 --> T1
    T1 --> T2
    T2 --> T3
    T3 --> T5
    T5 --> T6
```

- **T0** fixes acceptance. **T1** lays the consent schema + domain.
- **T2** owns the governance core (append-only consent + fail-closed TTS precondition, X11).
- **T3** adds the read-only audit/rights/consent API; **T4** is retained only as a
  cancelled historical task; **T5** owns the complete mobile surface.
- **T6** wires deterministic fixtures, the Maestro mobile flow, and docs.

## Relationship to other slices

- **Depends on (built/planned):** S-000, S-010 (`audit_events`, `rights_records`
  already persisted), S-040, S-050, and **S-105-T2** (mobile workspace context).
- **Closes obligation:** X11 (consent/voice-cloning permission before TTS) at the
  contract level.
- **Forward integration:** S-150 (TTS/voice synthesis) calls `consent_gate` before any
  derivative; S-180 publication may surface the compliance view.
- **Ordering with S-160:** independent at build time, but S-110 must exist before
  S-150 TTS/dubbing can run fail-closed.

## Governing documents

- `docs/playbooks/AGENT_WORKFLOW_GUIDE.md` (authoritative workflow)
- `docs/policies/HITL_AUTONOMY_POLICY.md`, `docs/policies/RRI_POLICY.md`
- ADR-008 (fail-closed precondition — applied to the consent gate), ADR-018 (durable
  audit + redaction), ADR-025 (owner-credential/evidence stored by reference +
  redacted), ADR-023 (API auth), ADR-024 (gateway transport), ADR-006 (Postgres metadata)
- `docs/plan/s-100-collaborative-workspace.md` (S-100 hard predecessor),
  `docs/plan/s1-asset-ingestion-rights-ledger.md` (the ledgers this reads)

## Open follow-ups

- **X-S-110-1:** author an ADR for the voice-consent ledger + TTS precondition (X11).
- **X-S-110-2:** evidence-store mechanism for consent proof (stored by reference) ties to
  the owner-credential secret-store decision (roadmap X20); align when that is decided.
- **X-S-110-3:** real-stack verification of the compliance reads against live
  `audit_events`/`rights_records` — operational, documented in T6, not automated here.
