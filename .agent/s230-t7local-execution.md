# S-230-T7local execution packet

## Purpose

Execute the base S-230 mobile product flow against the **gateway** exposed by
`infra/local/docker-compose.yml` and emit the evidence required by
`docs/tasks/s-230-poc-v1-digitalocean.md § S-230-T7local`.

This lane is deliberately independent of MVP0-P2P P6/P5 certification so it can
run in parallel. The normal binary may contain P6/P2P product code, but this task
must neither modify nor exercise Invite/Claim/P2P Sync/Verify/loopback/HPKE
behavior. It certifies only the base S-230 flow.

## Preconditions

- Branch: `feature/p2p-mvp-core`.
- P5.T3/P5-CERT is **not a prerequisite**. Do not overlap T7local with an
  actively running device-certification session only when both would contend
  for the same Metro/ADB/emulator resources.
- Preserve any existing P5 evidence SHA; do not reinterpret T7local as P5
  evidence.
- Local Docker stack is available.
- JDK 17 / Android SDK environment is already working.
- Use the normal mobile app, not `android:p2p-dev` or `android:p2p-cert`.

## Environment

1. Record:
   - `git rev-parse HEAD`
   - emulator/device name, Android API and ABI
   - Docker Compose service status
2. Start the Compose app profile with the **gateway service included**. Run the
   local P2P preflight only as an infrastructure sanity check:
   `bash infra/local/p2p/preflight.sh --with-runtime`
   Then explicitly record the gateway checks until the shared preflight owns
   them:
   - `curl -fsS http://localhost:8082/health/live`
   - `curl -fsS http://localhost:8082/health/ready`
   T7local itself remains a base/non-P2P product-flow certification.
3. Select the gateway address reachable from the target:
   - Android emulator: `http://10.0.2.2:8082`
   - physical Android: `http://<host-LAN-IP>:8082`
   - host `8080` is the API port; using it bypasses the gateway and invalidates
     T7local
   - never use device-local `127.0.0.1` for the host Compose gateway
4. Set `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL` explicitly and record the redacted
   configuration.

## Required run

Build/install/launch the normal app against the local Compose gateway, then
exercise one fresh short video through:

`login -> upload -> rights -> finalize -> preparation -> review -> publish -> normal HLS playback`

Do not open or execute My Content/Invites/P2P Claim/Sync/Verify/loopback actions
as part of T7local evidence. Those belong to the MVP0-P2P lane.

Do not accept HTTP 2xx alone. For every stage, capture downstream evidence using
existing API/read-model/DB/audit/artifact surfaces as appropriate:

- login: authenticated session established
- upload/finalize: asset id and finalized ingest
- preparation: authoritative preparation state + expected HLS artifacts
- review: review task is created and visible
- review decision: accepted decision persisted
- publish: publication state persisted
- playback: existing non-P2P player renders the published media

Use only normal product behavior. Do not seed state manually.

## Negative controls

- malformed/missing gateway URL -> existing `ConfigErrorScreen`
- expired/rejected token -> existing logout/auth-expiry path, not a silent stall

Do not mutate product code to force either result.

## Verification

Run:

```bash
cd mobile
npm run typecheck
npm run lint
npm test
```

If the repository's `make qa-mobile` target is available from the repo root,
run it as the canonical aggregate mobile gate.

## Evidence artifact

Write a new audit artifact under `docs/audit/` containing:

- exact HEAD
- gateway profile (no credentials)
- device/emulator
- asset id
- per-stage observed downstream evidence
- negative controls actually executed
- mobile QA results
- screenshots/logs only as supporting evidence

No P2P invitation, P2P claim, P2P sync/verification, P2P loopback playback,
HPKE or P5.T3 evidence belongs in this artifact.

Record the exact T7local HEAD. T7local may legitimately PASS while P6 is still
moving. Before T6p-a, compare that HEAD with the exact DEV-HANDOFF head. If
relevant base-flow/mobile/gateway/local-compose paths changed, run the bounded
regression defined in the S-230 ledger and attach a supplemental freshness
artifact; do not rerun P2P certification.

## Stop conditions

Stop and report rather than patching if:

- the base product flow unexpectedly requires invoking P2P Invite/Claim/Sync/
  Verify/loopback behavior
- any stage returns 2xx but downstream state does not advance
- local Compose ownership/provenance is ambiguous
- a defect would require changing product behavior

Do not provision or deploy Digital Ocean.

## Final output

```
S-230-T7local: PASS | BLOCKED
HEAD:
device:
gateway:
login:
upload/rights/finalize:
preparation:
review:
publish:
playback:
negative config:
negative auth-expiry:
qa-mobile:
code changed: YES/NO
audit artifact:
next gate: T7c | BLOCKED
```
