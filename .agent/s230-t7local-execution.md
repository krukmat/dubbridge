# S-230-T7local execution packet

## Purpose

Execute the existing non-P2P mobile product flow against
`infra/local/docker-compose.yml` and emit the evidence required by
`docs/tasks/s-230-poc-v1-digitalocean.md § S-230-T7local`.

This is deliberately independent of MVP0-P2P P5. Do not touch
`mobile/src/p2p/**`.

## Preconditions

- Branch: `feature/p2p-mvp-core`.
- Run only after the current P5.T3 Android run has finished so Metro/ADB/app
  installation are not competing.
- Preserve the P5 evidence SHA; do not reinterpret T7local as P5 evidence.
- Local Docker stack is available.
- JDK 17 / Android SDK environment is already working.
- Use the normal mobile app, not `android:p2p-dev` or `android:p2p-cert`.

## Environment

1. Record:
   - `git rev-parse HEAD`
   - emulator/device name, Android API and ABI
   - Docker Compose service status
2. Run the local P2P preflight only as an infrastructure sanity check when the
   full local profile is already up:
   `bash infra/local/p2p/preflight.sh --with-runtime`
   T7local itself remains a non-P2P product-flow certification.
3. Select the gateway address reachable from the target:
   - Android emulator: normally `http://10.0.2.2:8080`
   - physical Android: use the host LAN address
   - never use device-local `127.0.0.1:8080` for the host Compose gateway
4. Set `EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL` explicitly and record the redacted
   configuration.

## Required run

Build/install/launch the normal app against the local Compose gateway, then
exercise one fresh short video through:

`login -> upload -> rights -> finalize -> preparation -> review -> publish -> in-app playback`

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

No P2P invitation, P2P sync, P2P loopback playback or P5 claim belongs in this
artifact.

## Stop conditions

Stop and report rather than patching if:

- the normal product flow requires `mobile/src/p2p/**`
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
