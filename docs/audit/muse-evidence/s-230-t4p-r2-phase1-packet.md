# S-230-T4p-R2 — Task-analysis review packet

## Objective

Make the local `run_full-pipeline` harness establish the project-language
context required by the transcription worker before that worker can claim the
fixture asset. This is a local test-harness correction only.

## Confirmed cause

`run_full-pipeline` currently starts the worker before the register/ingest/
finalize sequence. It creates no project and no `target_languages` row. Once
S-230-T4p-R1 switched the separate containers to shared MinIO, preparation
reached `ready`, after which the worker recorded transcription `failed` with
`no target_languages row found for asset project`.

`apps/worker-runner/src/subtitle_enqueue.rs::resolve_source_language` resolves
source language via the asset's project and returns exactly that error when no
row exists. Existing workspace routes support the required sequence:

1. `POST /orgs/{org_id}/projects` with `{ "name": "...", "asset_ids": [] }`.
2. `PUT /orgs/{org_id}/projects/{project_id}/target-languages` with
   `{ "source_lang": "en-US", "target_languages": ["es-ES"] }`.
3. Finalize ingest and `POST /orgs/{org_id}/projects/{project_id}/assets`
   with `{ "asset_id": "<finalized UUID>" }`.
4. Start the worker only after step 3 succeeds, then poll transcription.

The register response provides the workspace/org identifier. Project, language,
and asset-link requests use the already-created bearer token.

## Scope

- In: reorder/add authenticated `curl` calls within
  `scripts/test-production-images.sh::run_full-pipeline`; task/audit status
  synchronization.
- Out: application routes and code, database schema/migrations, images,
  worker defaults, MinIO configuration, production descriptors, and DigitalOcean.

## Acceptance

- HP-1: with the evidence worker tag, create project/languages, finalize/link
  the fixture asset, start worker, observe `asset_transcription_status=ready`,
  then receive exit 0.
- EC-1: if any project, language, or link call fails, return non-zero before
  starting worker; existing re-finalization is still rejected; cleanup leaves
  no full-pipeline test containers.
- Verification: `bash -n`, contract/full-pipeline, a successful full local run,
  stopped-dependency run, cleanup inspection, and audit recording.

## Risk and routing

RRI 33 (Moderate): test workflow/order and authenticated API coupling are the
main drivers. No penalties. Tests exist in the workspace route and worker
enqueue areas. Implementation requires approval and follows the Moderate
local-first route. This packet requests analysis only; no implementation has
started.
