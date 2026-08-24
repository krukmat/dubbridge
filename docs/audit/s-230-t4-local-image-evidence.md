---
type: Audit
title: "S-230-T4p local image evidence"
status: done
task: S-230-T4p
date: 2026-08-24
---

# S-230-T4p — Local image evidence

Date: 2026-08-24

## Outcome

**DONE — HP-1 is satisfied.** The full pipeline ran end-to-end on
2026-08-24 with `dubbridge-worker-runner-t4p:evidence` and
`dubbridge-api-t4c:test`. `asset_transcription_status` reached `ready`;
EC-1 (re-finalize rejection) passed; no containers remained after the run.
No DigitalOcean resource was provisioned and no deployment descriptor was
edited.

## Environment and immutable inputs

| Item | Observed value |
|---|---|
| Docker server | `29.5.2` |
| Dependency stack | `local-postgres-1`, `local-redis-1`, and `local-minio-1` running on `local_default` |
| PostgreSQL | `psql (PostgreSQL) 16.14 (Debian 16.14-1.pgdg13+1)` |
| Builder base | `rust:1-bookworm@sha256:6e957ef098dcc77d33e310261e4ed5843bb108d5c3b5dc2b476cbc8b6caf53fa` |
| Runtime base | `debian:bookworm-slim@sha256:817e6cf99d6fc127ff4ffe8580049b60deba0adfbbb2bd65ddc3ef8fbb7aade0` |
| API image | `dubbridge-api-t4c:test`, `sha256:697a5a13a6d01dc0e674cd7a27b3a5c1ab486a29112b2486b2a00a52436e11b1`, 40,270,941 bytes |
| Legacy worker tag used by the harness default | `dubbridge-worker-runner-t4i:test`, `sha256:8e37af071eaa61b659f9cb24ee2725700d073ae6e7b30074ee3e3acf7705822f`, 194,329,038 bytes |
| Isolated rebuilt evidence tag | `dubbridge-worker-runner-t4p:evidence`, `sha256:95191b249c573256752efffa199df5ebf5949ed96b3f830b8363f850a50c0ec4`, 326,772,557 bytes |

Both inspected images report Debian GNU/Linux 12 (bookworm). The API image
does not contain `python3` (expected for its Rust runtime). The evidence worker
contains `Python 3.11.2`, `faster-whisper==1.1.0`, `ctranslate2==4.8.1`,
`/app/asr_worker/main.py`, and `/app/translation_worker/main.py`.

## Execution transcript

| Command | Exit | Result |
|---|---:|---|
| `DUBBRIDGE_TEST_DEPENDENCY_CONTAINER=dubbridge-t4p-degraded-dependency bash scripts/test-production-images.sh run full-pipeline` | 1 | EC-1 passed: deliberately stopped dependency reported `is not running — bring up infra/local/docker-compose.yml first`. The temporary dependency container was removed. |
| `bash scripts/test-production-images.sh run full-pipeline` | 1 | HP-1 failed using the harness default worker tag. The worker exited 1: `ASR worker script not found via discovered fallback path: /usr/src/app/apps/worker-runner/../../workers/asr-worker-py/main.py`. API `/health/live` and `/health/ready` were both HTTP 200; readiness listed postgres, redis, and storage as `ok`. |
| `docker build -f apps/worker-runner/Dockerfile -t dubbridge-worker-runner-t4p:evidence .` | 0 | Built isolated current-Dockerfile worker image; existing tags were not overwritten. |
| `DUBBRIDGE_WORKER_IMAGE_TAG=dubbridge-worker-runner-t4p:evidence bash scripts/test-production-images.sh run full-pipeline` | interrupted by executor after 30s | API and worker started and worker remained running. The shell process was terminated by the execution environment before it could finish the ASR polling/EC-1 portion, so it is not a passing HP-1 result. Containers were explicitly stopped afterwards. |

### Passing HP-1 / EC-1 run (2026-08-24)

| Command | Exit | Result |
|---|---:|---|
| `DUBBRIDGE_WORKER_IMAGE_TAG=dubbridge-worker-runner-t4p:evidence bash scripts/test-production-images.sh run full-pipeline` | 0 | HP-1 passed: API liveness/readiness OK, worker running, register + org membership + project + target languages succeeded, asset ingested/rights/finalized, `asset_transcription_status = ready`, EC-1 re-finalize rejection passed. Script printed `Run check passed for full-pipeline`. No containers remained after the run. |
| `DUBBRIDGE_TEST_DEPENDENCY_CONTAINER=dubbridge-t4p-final-degraded DUBBRIDGE_WORKER_IMAGE_TAG=dubbridge-worker-runner-t4p:evidence bash scripts/test-production-images.sh run full-pipeline` | 1 | EC-1 preserved: non-existent dependency container rejected before pipeline startup. |

Fixes applied to `scripts/lib/full-pipeline.sh` during execution:
- `workspace_id` → `workspaceId` (camelCase match for `AuthSuccessResponse`
  `#[serde(rename_all = "camelCase")]`)
- Added `userId` extraction and `org_members` INSERT (register does not
  create org membership; the org-scope middleware requires it for
  `POST /orgs/{org_id}/projects`)

### Readiness, migration, and downstream state

- API readiness during the legacy-tag run: `HTTP/1.1 200 OK`, with postgres,
  redis, and storage all `ok`.
- The full-pipeline harness does not invoke the migration image. Its required
  separate migration transcript is recorded in "Migration evidence (2026-08-24)"
  below.
- Downstream ASR state: no asset reached a recorded `ready` state. The
  legacy worker failed before ASR could start. During the evidence-tag run,
  the worker emitted translation failure-status warnings (`Conflict`) for
  pre-existing queued assets before the executor interruption; those are
  observed state, not an acceptance result.
- Translation rebuild debt: **none**. T4m and T4n are recorded Done and the
  rebuilt evidence image contains the translation worker. The default T4o
  worker tag is nevertheless stale relative to that image.

### Migration evidence (2026-08-24)

- Rebuilt the current local image with `docker build -f apps/cli/Dockerfile
  -t dubbridge-cli:t4f-test .` → exit `0`.
- Image inspected as
  `sha256:5c7fa56f5d3f105e76052765ac5b7428b8ee0ebfe5b13eb2d0a60ac4184036c3`,
  34,321,337 bytes.
- Created the disposable empty database
  `t4p_migration_evidence_20260824`, then ran `docker run --rm --network
  local_default -e DUBBRIDGE_ENV=local -e
  DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@local-postgres-1:5432/t4p_migration_evidence_20260824
  dubbridge-cli:t4f-test` → exit `0`. Output included `dubbridge-cli: applying
  migrations` and `dubbridge-cli: migrations applied successfully`.
- `_sqlx_migrations` contained 29 rows after the run. The disposable database
  was then removed with `DROP DATABASE t4p_migration_evidence_20260824 WITH
  (FORCE)`; no persistent local database was changed.

## Blocking condition and follow-up

The harness default `DUBBRIDGE_WORKER_IMAGE_TAG=dubbridge-worker-runner-t4i:test`
does not represent the ASR/translation-capable image, so T4p continues to use
the recorded `DUBBRIDGE_WORKER_IMAGE_TAG` override. The separate-container
`local_fs` fault was corrected by S-230-T4p-R1: both containers now use the
same local MinIO S3 bucket. Its reviewed post-change run advanced preparation
to `ready`, but transcription became `failed` with `no target_languages row
found for asset project`.

That error is emitted when an asset is not linked to a project with a source
language row. The current harness starts the worker before it registers the
workspace and never creates that project/language context. S-230-T4p-R2 was
the bounded local remediation: create project and languages, finalize and link
the asset, then start the worker. **R2 is Done (2026-08-24):** 27 lines
inserted into `scripts/lib/full-pipeline.sh` adding workspace_id
extraction, project creation with asset linking, and target language setup —
each fail-closed. Two additional fixes were required during the live run:
camelCase field name (`workspaceId`) and org membership INSERT (register does
not create `org_members`). **All four closure conditions are now met
(2026-08-24):** HP-1 passed (exit 0, `asset_transcription_status = ready`),
EC-1 passed, migration evidence captured, no residual containers.

### S-230-T4p-R1 post-change execution

| Command | Exit | Result |
|---|---:|---|
| `DUBBRIDGE_WORKER_IMAGE_TAG=dubbridge-worker-runner-t4p:evidence bash scripts/test-production-images.sh run full-pipeline` | 1 | API readiness and asset preparation succeeded; `asset_transcription_status` reached `failed`, with database error `no target_languages row found for asset project`. This proves shared MinIO storage is reachable by both containers, but does not satisfy HP-1. |
| `DUBBRIDGE_TEST_DEPENDENCY_CONTAINER=dubbridge-t4p-r1-degraded-dependency DUBBRIDGE_WORKER_IMAGE_TAG=dubbridge-worker-runner-t4p:evidence bash scripts/test-production-images.sh run full-pipeline` | 1 | EC-1 preserved: stopped dependency was rejected before pipeline startup. The exact temporary stopped container was removed afterwards. |

`bash -n scripts/test-production-images.sh`, `bash scripts/test-production-images.sh
contract full-pipeline`, and `git diff --check` passed after R1. No
`dubbridge-*-full-pipeline-test` container remained running after each run.

## Independent review

Task-analysis review: muse-glimmer `docs/audit/muse-evidence/s-230-t4p-phase1.json` - PASS

Code-solution review: muse-glimmer `docs/audit/muse-evidence/s-230-t4p-phase2.json` - PASS

The phase-2 reviewer completed 3/3 usable passes with no findings. Its PASS
validates that this artifact accurately preserves the scope and evidence; it
does not turn the unmet HP-1 acceptance criterion into a pass.

### Remediation review evidence

Code-solution review: muse-glimmer
`docs/audit/muse-evidence/s-230-t4p-r1-phase2.json` - PASS (3/3 usable passes,
no findings). The review accepted the MinIO-only diff and identified the
project-language failure as out of scope for R1.

Task-analysis review: gemma
`docs/audit/muse-evidence/s-230-t4p-r2-phase1.json` - PASS.

Code-solution review for R2: user-waived ("lo damos por bueno, cerremos la
task as is"). Implementation verified structurally (`bash -n` passes); 2
Reflection passes completed with no findings. Full closure evidence in the
task ledger at `docs/tasks/s-230-poc-v1-digitalocean.md` § S-230-T4p-R2.
