# Hx local hardening task packet

## Goal

Close the bounded H1-H3 hardening discovered by the real P2/P3 local E2E certification, then stop before P5.

Work only on the existing branch:

`feature/p2p-mvp-core`

Do not create a branch. Do not reset/rebase/force-push. Preserve unrelated worktree changes.

If any Hx item expands beyond a small bounded hardening task, STOP and report the blocker instead of widening scope.

## Current context

The real local P2/P3 E2E path has already passed:

`ingest -> rights -> finalize -> preparation/S-120 -> P2 activation -> K1 package -> mTLS -> Availability Node/Hyperdrive/Hyperswarm -> durable P2P_READY -> P3 invitation`

Certified asset:
- asset_id: `155bfee4-0aad-4644-9e51-3891e9e006ec`
- publication_id: `4e811878-e947-469d-ad69-e543f091a99f`
- lineage_id: `64d51435-8b49-47e5-bec7-0423128f809b`

Do not expose JWTs, passwords, invitation tokens, KEKs, or private-key material.

Important findings from that run:
1. P2 activation silently skips when KEK config is fully absent.
2. A host `dubbridge-api` process previously occupied port 8080 and accepted traffic instead of Docker.
3. The local worker image did not include `ffmpeg`.
4. API and worker now share `local-app-storage:/tmp/dubbridge-storage`.
5. Availability Node now loads its allowed client fingerprint from the generated local file.

Relevant recent commits already on this branch include:
- `bf49d88` shared local API/worker storage
- `dab1309` compose mTLS fingerprint loading
- `b85c0cc` local P2P runbook update
- `7f69b8b` added `infra/local/worker-runner.Dockerfile` as the start of H1

Inspect current HEAD before changing anything.

## H1 — reproducible local worker runtime

Target outcome:
- local worker image has `ffmpeg` baked in; do not apt-install it on every container start
- Compose builds/uses that local worker image
- shared `local-app-storage` remains intact
- existing P2P CA/client identity/ciphertext mounts remain intact

Preferred paths:
- `infra/local/worker-runner.Dockerfile`
- `infra/local/docker-compose.yml`
- `infra/local/p2p/README.md` only if needed

Keep this local/dev-only. Do not alter the production worker Dockerfile unless a real incompatibility forces it; if so STOP and report.

Verification:
- `docker-compose -f infra/local/docker-compose.yml --profile app build worker-runner`
- start/recreate worker
- `docker-compose ... exec worker-runner ffmpeg -version`
- worker still starts with `p2p_publication_enabled=true`

## H2 — make P2 activation fail-visible

Current code of interest:
- `apps/worker-runner/src/p2p_activation.rs`
- `apps/worker-runner/src/preparation_runtime.rs`
- `apps/worker-runner/src/p2p_publication_runtime.rs`

Required behavior:
- fully disabled P2 is allowed only when the P2 runtime itself is not configured
- if publication runtime is configured/enabled but activation KEK config is missing or partial, startup/config validation must fail visibly rather than silently skipping asset activation
- preserve the existing contract that an individual P2 activation failure after S-120 Ready does not roll back S-120 Ready or suppress transcription
- do not change crypto, KEK format, K1 semantics, database schema, or readiness predicates

Prefer startup validation in worker-runner over converting post-ready activation into a pipeline-fatal error.

Minimum cases to verify:
- P2 runtime absent + KEK absent => worker allowed; P2 disabled
- P2 runtime configured + KEK complete => worker allowed
- P2 runtime configured + KEK absent => worker startup/config error
- P2 runtime configured + KEK partial/invalid => worker startup/config error

Add focused tests using existing test style. Do not introduce global env-test races; if safe env isolation becomes non-trivial, STOP and report.

## H3 — local certification preflight/provenance

Add a small local preflight script, preferably:

`infra/local/p2p/preflight.sh`

It must fail non-zero if a critical check fails and must not print secrets.

Checks:
- current branch is exactly `feature/p2p-mvp-core`
- show HEAD SHA
- required generated mTLS files exist and are non-empty
- `docker-compose ... ps` shows API, Availability Node and worker running when runtime validation is requested
- API `/health/live` works
- API `/health/ready` works
- detect ownership/provenance of TCP 8080 and fail if a non-Docker host process is listening on 8080
- worker container has `ffmpeg`
- recent worker logs contain `p2p_publication_enabled=true`
- recent Availability Node logs contain `Availability Node listening`

Port provenance check should work on macOS with commonly available tools such as `lsof`. If a portable check becomes large/fragile, implement the macOS-local check only and document that scope.

Update `infra/local/p2p/README.md` with the preflight command.

## Scope guards

Do NOT:
- touch P5/mobile
- change P2/P3 cryptographic or authorization contracts
- change migrations/schema
- seed or mutate P2P readiness rows
- alter staging/production secret values
- create a new branch
- expand into monitoring/alerting infrastructure; for this Hx scope, fail-visible startup/config validation + local preflight is enough

If H2 requires architecture changes beyond worker startup validation, STOP and report instead.

## Verification and commits

Run focused tests/checks relevant to changed files. At minimum:
- Rust formatting/check/tests for touched worker code
- shell syntax checks for new/changed scripts
- Compose config/build validation for local files

Do not claim physical Android or P5 validation.

Use coherent commits on the same branch. Do not push unrelated cleanup.

## Required final report

Return only a concise report with:

```
HX HARDENING: PASS | BLOCKED

branch:
HEAD:

H1 reproducible worker:
PASS/BLOCKED
evidence:

H2 P2 fail-visible config:
PASS/BLOCKED
evidence:

H3 local provenance preflight:
PASS/BLOCKED
evidence:

files changed:
- ...

commits:
- ...

tests/checks:
- ...

remaining risk:
- ...

next action:
<one concrete step>

P5 touched: NO
```

If BLOCKED, include:
- exact Hx item
- exact blocker
- why it exceeds bounded hardening
- smallest next decision needed from the owner

Stop after H1-H3. Do not start P5.
