# S-230-T4p-R1 — Phase 2 code-solution review packet

Task: share local pipeline storage through MinIO.

Scope: review only the following change in `run_full-pipeline`. Both the API and
worker containers must read/write the same local S3-compatible MinIO bucket.
Do not assess DigitalOcean, production descriptors, application code, or the
default worker tag.

Acceptance:

- HP-1: API and worker use one MinIO bucket through the local S3 adapter.
- EC-1: a stopped dependency still exits non-zero.
- The observed post-change run reached `preparation_status = ready`; its later
  transcription failure is `no target_languages row found for asset project`,
  which is outside this storage-only change and remains an open T4p precondition.

Verification already passed: `bash -n scripts/test-production-images.sh`,
`bash scripts/test-production-images.sh contract full-pipeline`, `git diff --check`,
and the stopped-dependency negative run (exit 1).

Diff under review:

```diff
diff --git a/scripts/test-production-images.sh b/scripts/test-production-images.sh
@@ -690,10 +690,10 @@ run_full-pipeline() {
          -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \\
          -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \\
          -e DUBBRIDGE_ENV=local \\
-         -e AWS_ACCESS_KEY_ID=dubbridge \\
-         -e AWS_SECRET_ACCESS_KEY=dubbridge123 \\
-         -e AWS_REGION=us-east-1 \\
-         -e DUBBRIDGE_STORAGE__BACKEND=local_fs \\
+         -e DUBBRIDGE_STORAGE__ACCESS_KEY_ID=dubbridge \\
+         -e DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=dubbridge123 \\
+         -e DUBBRIDGE_STORAGE__REGION=us-east-1 \\
+         -e DUBBRIDGE_STORAGE__BACKEND=s3 \\
          -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \\
@@ -729,10 +729,10 @@ run_full-pipeline() {
          -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \\
          -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \\
          -e DUBBRIDGE_ENV=local \\
-         -e AWS_ACCESS_KEY_ID=dubbridge \\
-         -e AWS_SECRET_ACCESS_KEY=dubbridge123 \\
-         -e AWS_REGION=us-east-1 \\
-         -e DUBBRIDGE_STORAGE__BACKEND=local_fs \\
+         -e DUBBRIDGE_STORAGE__ACCESS_KEY_ID=dubbridge \\
+         -e DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=dubbridge123 \\
+         -e DUBBRIDGE_STORAGE__REGION=us-east-1 \\
+         -e DUBBRIDGE_STORAGE__BACKEND=s3 \\
          -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \\
```
