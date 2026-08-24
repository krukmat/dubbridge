# S-230-T4p-R1 phase-1 review packet

## Task scope

Change only `scripts/test-production-images.sh:688`-`741` so the API and
worker containers in `run_full-pipeline` use the same existing local MinIO
bucket. The observed failure is:

`failed to load source artifact bytes from
'ingests/<token>/t4o-fixture.wav': object not found`

The API and worker currently force `local_fs` in separate containers without
a shared volume. No application code, production descriptor, DigitalOcean
resource, timeout, cleanup, worker tag, or other harness case may change.

The values `dubbridge` / `dubbridge123` are the disposable local MinIO
credentials already declared in `infra/local/docker-compose.yml`; they are not
production secrets and must remain confined to this local test command.

## Acceptance criteria

- Both containers select `DUBBRIDGE_STORAGE__BACKEND=s3`.
- Both use the same `http://minio:9000` endpoint and `dubbridge-local` bucket.
- Both receive Figment's nested `StorageSettings` variables for region,
  access key, and secret key.
- The redundant AWS environment aliases are removed from these commands.
- `bash -n`, the stopped-dependency EC, and a real full-pipeline run must pass.
- The full run must observe ASR `ready`, reject re-finalization, and clean up.

## Proposed implementation diff

```diff
diff --git a/scripts/test-production-images.sh b/scripts/test-production-images.sh
--- a/scripts/test-production-images.sh
+++ b/scripts/test-production-images.sh
@@ -691,11 +691,11 @@ run_full-pipeline() {
          -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \
          -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \
          -e DUBBRIDGE_ENV=local \
-         -e AWS_ACCESS_KEY_ID=dubbridge \
-         -e AWS_SECRET_ACCESS_KEY=dubbridge123 \
-         -e AWS_REGION=us-east-1 \
-         -e DUBBRIDGE_STORAGE__BACKEND=local_fs \
+         -e DUBBRIDGE_STORAGE__ACCESS_KEY_ID=dubbridge \
+         -e DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=dubbridge123 \
+         -e DUBBRIDGE_STORAGE__REGION=us-east-1 \
+         -e DUBBRIDGE_STORAGE__BACKEND=s3 \
          -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \
          --name dubbridge-api-full-pipeline-test \
          "$api_image"
@@ -730,11 +730,11 @@ run_full-pipeline() {
          -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \
          -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \
          -e DUBBRIDGE_ENV=local \
-         -e AWS_ACCESS_KEY_ID=dubbridge \
-         -e AWS_SECRET_ACCESS_KEY=dubbridge123 \
-         -e AWS_REGION=us-east-1 \
-         -e DUBBRIDGE_STORAGE__BACKEND=local_fs \
+         -e DUBBRIDGE_STORAGE__ACCESS_KEY_ID=dubbridge \
+         -e DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=dubbridge123 \
+         -e DUBBRIDGE_STORAGE__REGION=us-east-1 \
+         -e DUBBRIDGE_STORAGE__BACKEND=s3 \
          -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \
          --name dubbridge-worker-full-pipeline-test \
          "$worker_image"
```

Review only whether this bounded task and proposed diff are ready for Low-band
delegation. Any finding must cite
`scripts/test-production-images.sh` and an integer line in the displayed diff.
