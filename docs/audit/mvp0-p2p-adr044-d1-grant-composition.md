---
type: Audit
title: "MVP0-P2P ADR-044 D1 grant-composition decision"
task: ADR044-D1
date: 2026-09-05
---

# MVP0-P2P ADR-044 D1 — grant-composition decision

## Decision boundary

This audit closes only ADR-044 D1: how P2P audience authorization composes
with ADR-032. It does not accept ADR-044, modify ADR-032, define a schema/API,
choose cryptography or key-envelope mechanics, resolve publication/recovery,
or authorize P2/P3 implementation.

The approved parent envelope was RRI **55 / Med-high / Effort L**. Its honest
subdivision retained the parent gate while using four independently verifiable
Low/Effort S leaves. Mechanical codification S4 reproduced RRI **23 / Low /
Effort S** with the command frozen in
`docs/tasks/mvp0-p2p-adr044.md`.

## Frozen ADR-032 register

The following accepted facts were frozen in S1 and applied without modifying
ADR-032:

| ID | Accepted constraint |
|---|---|
| `A32-F1` | Prepared HLS crosses a backend-owned grant boundary; raw storage keys are not exposed. |
| `A32-F2` | The boundary checks readiness and the applicable authenticated-review or published-audience authorization; scoped references expire. |
| `A32-F3` | Signed URLs, proxying, or CDN URLs are replaceable transport mechanics; the stable contract is the backend grant. |
| `A32-F4` | Remote manifests/segment references remain bounded by the grant and are not durable permission. |
| `A32-F5` | Playback fails closed on unavailable preparation and on the applicable authorization/publication gate. |
| `A32-F6` | Grant/refusal is durably traceable; high-volume segment traffic need not create one durable row per request. |
| `A32-F7` | S-125 owns the server-side playback boundary, not player UX. |
| `A32-F8` | ADR-032 leaves CDN/public delivery for a later audience requirement. |

ADR-032 defines no invitation claim, viewer/device binding, ciphertext
replication, content-key wrapping/unwrap, or key revocation. S1 therefore
allowed authorization semantics to be compared separately from its HTTP
manifest/segment mechanics, without asserting that reuse or separation was
already accepted. The complete citations and the two bounded inferences
`A32-I1`–`A32-I2` remain in the D1 ledger.

## Frozen P2 constraint and rubric register

| ID | Constraint applied to every option |
|---|---|
| `P2-C1` | `apps/api` remains authoritative; Hyperdrive/package/ciphertext possession is never authorization. |
| `P2-C2` | P2P transports ciphertext only, with no HTTP/S3 media fallback in the certified path. |
| `P2-C3` | Authorization must bind eligible asset/viewer and the MVP-0 single active-device path without silent claim rebind. |
| `P2-C4` | Unknown/expired eligibility fails closed; expiry and the MVP-0 revocation limit remain visible. |
| `P2-C5` | Only control-plane authorization gates a device-wrapped content key; raw tokens/plaintext keys and denied credentials never cross their boundaries. |
| `P2-C6` | ADR-032 remains unchanged for authenticated review playback. |
| `P2-C7` | Authorization grant/refusal is durably traceable without per-media-request durable rows. |
| `P2-C8` | HTTP manifest/segment-reference mechanics gain no local-P2P meaning implicitly. |
| `P2-C9` | D1 decides composition only, not records, routes, fields, tokens, algorithms, envelopes, publication, or ADR acceptance. |

S2 applied one shared rubric to every option: `R1 authority`, `R2 binding`,
`R3 expiry/revocation`, `R4 wrapped-key release`, `R5 ADR-032 compatibility`,
`R6 auditability`, and `R7 scope discipline`. Findings were limited to
`satisfied`, `conditional`, `conflict`, or `undecided`; there was no numeric
score or aggregate rank. Full criterion wording and citations remain in the
D1 ledger.

## Neutral option matrix frozen at S3

| Criterion | `O1 reuse` | `O2 bypass` | `O3 parallel` |
|---|---|---|---|
| `R1 authority` | **conditional:** backend grant meaning must explicitly extend to P2P | **conditional:** fresh backend eligibility check must gate each release | **conditional:** only the control plane may create/evaluate the parallel authorization |
| `R2 binding` | **conditional:** invite/device binding remains outside ADR-032 transport semantics | **conditional:** claim may bind asset/viewer; device remains explicit later work | **conditional:** the distinct concept may own binding; exact semantics remain later work |
| `R3 expiry/revocation` | **conditional:** invite eligibility and post-release revocation remain unresolved | **conditional:** fresh check can reject expiry; post-release revocation remains unresolved | **conditional:** lifetime and post-release revocation remain unresolved |
| `R4 key release` | **conditional:** only a valid reused grant may authorize release | **conditional:** only a successful fresh eligibility check may authorize release | **conditional:** only a valid parallel authorization may authorize release |
| `R5 ADR-032 compatibility` | **conditional:** reuse must exclude HTTP manifest/segment mechanics | **satisfied:** ADR-032 stays on its existing path | **satisfied:** ADR-032 stays unchanged and the P2P concept is distinct |
| `R6 auditability` | **satisfied:** durable grant/refusal already exists | **conditional:** an equivalent durable authorization result is required | **conditional:** creation/refusal or evaluation must be durably recorded |
| `R7 scope discipline` | **satisfied:** semantic reuse can leave shapes undecided | **satisfied:** direct checking can leave persistence/API undecided | **satisfied:** parallel composition can leave name/schema/API/lifecycle undecided |

The matrix exposed these non-ranked tradeoffs: O1 reuses an established grant
but risks coupling to irrelevant HTTP semantics; O2 keeps ADR-032 isolated but
has no durable grant object by definition; O3 cleanly separates audience
authorization from both invitation and transport but creates a second model
whose consistency and audit behavior must remain explicit.

## Exact owner selection

After the options were presented and `O3` was explicitly named in the
immediately preceding exchange, the repository owner responded on 2026-09-05:

> `03`

That exact response is recorded as selection of **`O3 parallel`**. The frozen
semantic choice is:

- a valid invitation claim precedes a distinct backend-owned audience
  authorization;
- only that authorization gates wrapped-content-key release;
- the claim alone, ADR-032 grant, Hyperdrive key, package, and ciphertext cache
  are insufficient authority;
- ADR-032 remains unchanged for its existing review-time HTTP HLS boundary.

## Non-selected alternatives

- **`O1 reuse` was not selected for D1.** This does not deny ADR-032's accepted
  boundary or forbid a future unification decision; it means the P2P path does
  not issue an ADR-032 `PlaybackGrant` as its authorization event under this
  proposal.
- **`O2 bypass` was not selected for D1.** A current invitation/eligibility
  check remains necessary, but that check or claim alone is not the selected
  authorization concept.

These are decision dispositions, not implementation designs or permanent
rejections outside ADR-044's current scope.

## Remaining open questions and gates

- `ADR044-D2`: algorithm, envelope and wrapping contract, device-key
  generation/storage, authorization lifetime, and bounded revocation behavior.
- `ADR044-D3`: publication/outbox state, recovery, and relationship to S-120
  readiness/transcription enqueue.
- ADR-044 questions 4–7: Availability Node operation, certification profile,
  complete durable audit-event obligations, and device lifecycle.
- `ADR044-D4`: integrate the resolved decisions, accept ADR-044, and propagate
  canonical status only after its prerequisites pass.
- P2/P3: separately planned, scored, presented, and owner-approved
  implementation work after the ADR gate permits it.

No record name, field, endpoint, token, schema, algorithm, key format,
publication transition, or runtime behavior was selected by D1.

## Verification

- Exact owner response compared to the immediately preceding named `O3`
  checkpoint — PASS.
- Parent-band integrated inspection 1, architecture boundary: the selected
  semantics were checked against every S1/S2 authority, binding, expiry,
  release, compatibility, audit, and scope criterion — PASS.
- Parent-band integrated inspection 2, non-expansion: no schema, API, token,
  algorithm, envelope, publication, recovery, or runtime contract was added —
  PASS.
- Parent-band integrated inspection 3, status/gates: ADR-044 remains
  `Proposed`, D2-D4 remain separate decisions, and P2 remains unauthorized —
  PASS.
- ADR-032 content and status left unchanged — PASS.
- `make qa-docs` — PASS on 2026-09-05.
- `git diff --check` — PASS on 2026-09-05.

Task-analysis review: n/a - ADR/plan/task-ledger-only exemption.

Code-solution review: n/a - ADR/plan/task-ledger-only exemption.
