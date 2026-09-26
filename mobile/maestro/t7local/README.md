# S-230-T7local — real-stack Maestro harness

This directory drives the **real T7local base-product path** against the local
Docker Compose gateway. It is certification support, not product behavior.

## Hard boundaries

- gateway: Android emulator -> `http://10.0.2.2:8082`;
- backend: real `infra/local/docker-compose.yml --profile app` stack;
- auth: real `/auth/register` + `/auth/login`;
- media: a fresh synthetic MP4 copied to Android Documents/Downloads;
- state evidence: read-only PostgreSQL queries scoped to the current run;
- no mock gateway, `/e2e/seed`, seeded record IDs, or E2E fast-path;
- `EXPO_PUBLIC_E2E_ENABLED` must stay unset/false;
- P2P Invite/Claim/Sync/Verify, HPKE and loopback playback remain out of scope.

The existing screenshot flows under `mobile/maestro/*.yaml` remain unchanged.
This harness reuses their stable product `testID` contract only.

## Execution units

```text
M0 contract/boundaries
  |
  +--> M1 real login
  +--> M2 real media + rights
          |
          v
M3 real ingestion/finalize
          |
          v
M4 DB correlation + preparation/review probes
          |
          v
M5 review -> publish -> normal HLS playback
          |
          v
M6 one-command B3 -> C4 runner
```

The final T7local certification still runs the canonical packet A -> E. M6 only
drives B3-C4 and emits bounded evidence inputs; it does not mark T7local PASS or
edit the ledger.

## Runtime variables

The real flows receive values at runtime through Maestro `-e` arguments:

- `T7LOCAL_EMAIL`
- `T7LOCAL_PASSWORD`
- `T7LOCAL_OWNER`
- `T7LOCAL_PROOF_REFERENCE`
- `T7LOCAL_FILENAME`
- later phases additionally receive `T7LOCAL_ASSET_ID` and
  `T7LOCAL_REVIEW_TASK_ID`.

Credentials must never be written to audit artifacts.

## One-command B3 -> C4 execution

Prerequisites: the normal Android development build is already installed with
`EXPO_PUBLIC_DUBBRIDGE_GATEWAY_URL=http://10.0.2.2:8082`, Compose is healthy,
and the preflight is PASS.

From repo root:

```bash
bash mobile/maestro/t7local/run-real.sh
```

By default the runner creates a disposable real account through
`POST /auth/register`. To reuse an existing real local account, provide both
`T7LOCAL_EMAIL` and `T7LOCAL_PASSWORD`.

Before ingestion, the runner creates a real organization/project and target
language through supported API routes. After finalization it links the newly
created asset to that project. This is setup through the product API, not DB
seeding. The runner then fails closed unless the normal backend pipeline creates
a real review task for that asset.

The runner writes only non-secret identifiers to
`/tmp/dubbridge-t7local-<run-id>/summary.env`. That summary is an input to the
final A->E audit; it is not itself the certification artifact.
