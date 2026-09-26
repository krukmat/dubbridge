# S-230 T6 — low-context deployment execution packet

Use this file as the first context for T6 execution. Do not read the full
S-230 ledger/history unless a concrete blocker requires it.

## Current gate

- T5: PASS.
- T6p local lane: CLOSED through T6p-c PASS.
- T6a: PASS — deployment contract frozen in
  `docs/audit/s-230-t6a-do-deployment-contract-2026-09-26.md`.
- Next executable child: **T6b**.
- T7a remains BLOCKED until aggregate **T6 PASS**.

## Non-negotiable deployment decisions

- target: single Digital Ocean Droplet + production Docker Compose;
- region: `ams3`;
- hostname: `poc.iotforce.es`;
- media Space: `dubbridge-poc-v1`;
- existing Droplet was reported at `46.101.217.151`: inventory/import first,
  do not recreate blindly;
- Managed PostgreSQL is external; Redis remains Compose-local;
- only Caddy publishes application ports 80/443;
- Availability Node 8443 has no host/public binding;
- production runs immutable OCI digests; production does not build;
- OpenTofu is IaC source of truth; `doctl` is auxiliary;
- remote state uses a dedicated Space, never the media bucket;
- no deploy/restart/rollback command may delete persistent data.

## Child sequence

```text
T6a PASS
  -> T6b IaC descriptor / inventory-import plan (NO APPLY)
  -> T6c immutable release packaging
  -> T6d provision/import/apply + no-drift proof
  -> T6e deploy/migrate/readiness/network proof
  -> T6f real-video E2E downstream-state smoke
  -> T6g operational closeout
  -> T6 PASS
  -> T7a unblocked
```

Never skip a child PASS marker.

## Canonical agent interface

Later children implement this exact public surface:

```text
make do-plan
make do-provision
make do-deploy REV=<git-sha>
make do-smoke
make do-status
make do-rollback RELEASE=<release-id>
```

Successful commands print compact markers. Detailed output goes to
`artifacts/deploy/<run-id>/` and is surfaced only for bounded diagnosis.

## Stop rules

Stop before mutation when:

- required credentials/inputs are missing;
- current Digital Ocean resources cannot be unambiguously matched to desired
  resources;
- plan contains an unexpected destroy/replace;
- release does not resolve to exact OCI digests;
- a required secret boundary cannot be satisfied.

During deploy/smoke, fail closed on migration, health, TLS, network-boundary,
or authoritative downstream-state failure. A 2xx alone is not evidence.

Do not patch product code inside T6 to make deployment evidence pass. Record a
finding and return to the owning development/config task.

## Runtime certification lineage

T6p-c's exact locally runtime-certified P2P baseline is
`c4b8da98c93e80b88ed06a466226fe00273514d2`, Availability Node image
`sha256:9bc98e5590aa3a5fba1478adc99cc64a6185fde9320f0c5323cd8ad134de7d68`.
Later docs-only commits do not change that runtime certification. If a T7a
candidate changes a T6p-c-certified runtime/config surface, reopen and rerun
the bounded local certification before deployed P2P certification.

## Evidence summary contract

Normal chat/agent output should be limited to stable markers such as:

```text
DO_PLAN=PASS
DO_PROVISION=PASS
DO_DEPLOY=PASS
DO_SMOKE=PASS
DO_STATUS=PASS
```

On failure emit:

```text
<STAGE>=FAIL
ERROR_CODE=<stable-code>
EVIDENCE=<path>
```

Then inspect only the referenced bounded evidence needed to diagnose the
failure.
