# S-230-T4p-R1 API-container packet

In `scripts/test-production-images.sh:688`, replace exactly the following API
container block and nothing else:

```sh
    docker run -d --rm --network "$test_network" -p 8090:8080 \
         -e DUBBRIDGE_DATABASE_URL=postgres://dubbridge:dubbridge@postgres:5432/dubbridge \
         -e DUBBRIDGE_REDIS_URL=redis://redis:6379 \
         -e DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000 \
         -e DUBBRIDGE_ENV=local \
         -e AWS_ACCESS_KEY_ID=dubbridge \
         -e AWS_SECRET_ACCESS_KEY=dubbridge123 \
         -e AWS_REGION=us-east-1 \
         -e DUBBRIDGE_STORAGE__BACKEND=local_fs \
         -e DUBBRIDGE_STORAGE__BUCKET=dubbridge-local \
         --name dubbridge-api-full-pipeline-test \
         "$api_image"
```

Return the same block with only these substitutions:

- `AWS_ACCESS_KEY_ID` → `DUBBRIDGE_STORAGE__ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY` → `DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY`
- `AWS_REGION` → `DUBBRIDGE_STORAGE__REGION`
- `DUBBRIDGE_STORAGE__BACKEND=local_fs` →
  `DUBBRIDGE_STORAGE__BACKEND=s3`

Proposed diff:

```diff
@@ -693,10 +693,10 @@
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
```

The values are disposable local MinIO defaults. Do not alter any other line,
file, container, polling behavior, image tag, or production/DigitalOcean
configuration. Return only the wrapper's tagged replacement block.
