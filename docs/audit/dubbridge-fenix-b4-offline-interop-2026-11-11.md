---
type: Audit
title: "DubBridge → Fenix B4 offline cross-repository admission interop"
status: offline_contract_sim_pass_live_e2e_blocked
date: 2026-11-11
slice: INT-B4
---

# B4 — Offline cross-repository interoperability

**Owner:** ChatGPT Cloud; RRI N/A (DB-FENIX-INT/cloud). **Tracker:** [DubBridge #13](https://github.com/krukmat/dubbridge/issues/13).

**Pinned source snapshots**
- Fenix candidate: `7998231fb606e9dd92a22f7ceba9792dcc734675` on `feature/int-v2-dubbridge-consumes-fenix`.
- DubBridge candidate: `66956e05b6af9b1a881d74122306af3ddec4a8ee` on the correspondingly named branch.
- Exact provider contract blobs are recorded in [pinned manifest](../../docs/contracts/fenix-b4-pinned-manifest.json); its three mirrored JSON schemas remain verbatim Git blobs from Fenix, not independently approved external APIs.

## B4 progress, no fabricated E2E

| Task | Completed cloud work | Actual state |
|---|---|---|
| **B4.T1 real HTTPS** | Proposed Fenix `admission-ack`/`admission-error` JSON schemas, Go `s2s_admission_wire.go` adapter, Rust strict response parser; explicit HTTP statuses 202=new, 200=replay, 400=invalid, 403=denied, 409=conflict, 503=unavailable | **BLOCKED_LIVE**: Fenix has no mounted/approved S2S HTTP route, provisioned service token or operator-registered capability. No packets sent |
| **B4.T2 isolation/denial** | Go tests authored for wrong org/project/workspace, unknown/disabled capability, invalid/revoked bearer, sanitized wire errors and zero journal/run mutation; Rust code/status allowlist rejects forged messages | **SOURCE + SIMULATION**, not an executed real API denial |
| **B4.T3 idempotency/outage** | Go tests authored for new/replayed reservation, trace conflict and single operation row, no agent run; Rust strict ack operation/trace/replay-status match, explicit no-resend on unknown outcomes | **SOURCE + SIMULATION**, not cross-service crash recovery |

## Executed evidence

- **28/28 actual Cloud JS contract checks PASS** on exact mirrored Fenix provider schemas: B1 envelope required/unknown fields, UUIDs, capability syntax, request field ceiling; B4 ack operation/trace/replay strictness; B4 public error code/status coherence and failure message sanitization. Tests executed against retrieved GitHub bytes with provider-blob-SHA cross-check; this is **schema and static algorithm simulation**.
- **24/24 static cross-language source assertions PASS**, comparing Go wire tags/error codes/proposed HTTP values to Fenix schemas and Rust success/error handling; authored Go negative tests and Node script are present. These are source inspections, not compiled tests.
- Repeatable Node standard-library script [`scripts/check-fenix-b4-contract.mjs`](../../scripts/check-fenix-b4-contract.mjs) is committed. **The script itself was not run using Node in this session**; the 28 Cloud checks were executed directly in the connector-driven JS interpreter against equivalent fixtures/rules. Do not attribute a Node CLI PASS.
- No full Fenix Go 1.25.13 tests, Rust build/tests/clippy, HTTP E2E, independently reviewed security approval or deployed source was executed. **All remain NOT_RUN**. No local-agent token consumption.

## Cross-repo contract finding resolved in source

The original B1 *pre-acceptance error proposal* lists `CAPABILITY_NOT_CONFIGURED` and `CONFLICT`, whereas Fenix B2.E's actual internal projector uses `REQUEST_DENIED` and `OPERATION_CONFLICT`. Rather than incorrectly alter the lifecycle response schema, B4 now has a separate **transport-admission proposal** matching the actual Go projection. Rust rejects mismatched status/code/body and unexpected fields. Provider 503 and other ambiguous delivery outcomes never trigger automatic retries or fabricated success. Fenix `ProjectS2SAdmissionWireAck` emits the four allowlisted keys and no internal run/status.

## Security and release boundary

- DubBridge's local auth still depends on actual PostgreSQL membership/project checks; it has **no route/worker caller** and no local persisted integration linkage. The UUIDv5 factory requires a caller-owned previously persisted work item.
- Fenix's in-memory capability registry is empty by default, the provider route is not mounted, operator B2.B4 schema administration is not installed, and T4 [security review #8](https://github.com/krukmat/fenix/issues/8) remains separate.
- C4 GET/status deliberately deferred. No HTTP request, secret, paid hosting, CI, production migration, execution/agent, deployment or merge has been activated.

**Disposition:** B4 mock/contract portion delivered as Cloud source. **B4 real E2E remains explicitly blocked**, to be attempted only with a reviewed endpoint, proper credentials and security gates. No further local-agent work is requested.
