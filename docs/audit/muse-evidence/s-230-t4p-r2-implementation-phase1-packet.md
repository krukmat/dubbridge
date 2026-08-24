# S-230-T4p-R2 implementation packet

## Authorized change

Edit only `scripts/lib/full-pipeline.sh`. The parent harness now sources this
213-line module and is intentionally outside the local implementer's capability
surface. Do not modify application code, Dockerfiles, deployment descriptors,
DigitalOcean resources, or documentation.

## Current behavior and defect

`run_full-pipeline` starts the worker after API readiness but before registering
the account and finalizing the asset. The later asset therefore belongs to no
project with a target-language row. The worker fails transcription with `no
target_languages row found for asset project`.

## Required behavior

1. Keep API startup and health checks unchanged.
2. Do not start the worker in its present location. After the register response,
   parse `token` and `workspaceId` (camel case), validate that `workspaceId` is
   a UUID, then create a project through:
   `POST /orgs/${workspace_id}/projects`, authenticated with the token, JSON
   body `{"name":"t4o-full-pipeline-project","asset_ids":[]}`.
3. Parse and UUID-validate the response `id` as `project_id`. Configure the
   context through:
   `PUT /orgs/${workspace_id}/projects/${project_id}/target-languages`, with
   body `{"source_lang":"en-US","target_languages":["es-ES"]}`. Treat a
   failed HTTP request, missing/invalid response JSON, or missing `en-US` /
   `es-ES` result as a return-1 error before worker startup. Use guarded JSON
   extraction so malformed responses produce a precise `ERROR` and return 1,
   rather than printing a token or leaking a shell traceback.
4. Preserve ingest, rights, finalize, and asset UUID validation unchanged.
   Immediately after asset validation, link the asset through:
   `POST /orgs/${workspace_id}/projects/${project_id}/assets`, body
   `{"asset_id":"${asset_id}"}`. A failed request or a response that does
   not contain the linked asset id must return 1 before worker startup.
5. Move the existing worker `docker run`, three-second wait, and running-state
   check to immediately after that successful link. Preserve every worker
   environment variable, especially the shared MinIO/S3 configuration, exactly.
6. Preserve the status polling, re-finalization rejection, and RETURN cleanup.

## Acceptance criteria

- `bash -n scripts/lib/full-pipeline.sh`
- `bash scripts/test-production-images.sh contract full-pipeline`
- `DUBBRIDGE_WORKER_IMAGE_TAG=dubbridge-worker-runner-t4p:evidence bash
  scripts/test-production-images.sh run full-pipeline` exits 0 and reaches
  `asset_transcription_status = ready`.
- All project, language, and link failures return non-zero before the worker
  container is created, including invalid UUIDs, failed/non-JSON HTTP
  responses, an empty/mismatched language response, and a link response that
  omits the asset id; each emits a specific `ERROR` without printing the token.
- A successful ordering is register → project → languages → ingest → rights →
  finalize → link → worker → status polling, while preserving the worker's
  complete shared MinIO/S3 environment exactly.
- No token is printed.
- Stop after editing the authorized module and call `finish`; do not start any
  other task.

## Independently verified route facts

- Register serializes `workspaceId` (`AuthSuccessResponse` has
  `serde(rename_all = "camelCase")`).
- `POST /orgs/{org_id}/projects` returns a detail object with `id`.
- `PUT .../target-languages` accepts snake-case `source_lang` and
  `target_languages` and returns rows with `source_lang` / `target_lang`.
- `POST .../assets` returns a project detail object whose `assets` entries have
  `id`.

## Phase-1 disposition

Accepted all three Muse Glimmer findings. This revision adds a real local
full-pipeline acceptance command, explicit UUID/JSON/language/link failure
boundaries, token-redaction behavior, and an explicit preservation requirement
for worker S3/MinIO settings.
