# S-230-T4p-R1 delegation packet

Goal: repair only the local `run_full-pipeline` storage configuration in
`scripts/test-production-images.sh`.

RRI: 21 Low. Allowed implementation path: `scripts/test-production-images.sh`.

The API and worker are separate containers. Both currently use `local_fs`, so
the worker cannot read the upload stored inside the API container. Replace the
exact supplied block so both containers use the existing `minio` service on
the same Docker network through the S3 adapter.

Acceptance criteria:

- In both `docker run` commands, set `DUBBRIDGE_STORAGE__BACKEND=s3`.
- Keep `DUBBRIDGE_STORAGE__ENDPOINT_URL=http://minio:9000` and
  `DUBBRIDGE_STORAGE__BUCKET=dubbridge-local`.
- Supply the nested Figment fields required by `StorageSettings` in both
  containers: `DUBBRIDGE_STORAGE__REGION=us-east-1`,
  `DUBBRIDGE_STORAGE__ACCESS_KEY_ID=dubbridge`, and
  `DUBBRIDGE_STORAGE__SECRET_ACCESS_KEY=dubbridge123`.
- Remove the redundant `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and
  `AWS_REGION` entries from those two commands.
- Do not change health polling, worker selection, application code, timeout
  behavior, container names, cleanup, or any other case in the script.
- Return only the replacement block required by the wrapper's tagged contract.

Verification owned by the orchestrator: `bash -n
scripts/test-production-images.sh`, contract checks, the stopped-dependency
case, and a real uninterrupted `run full-pipeline` with the ASR-capable worker
override. Stop after returning the bounded replacement; do not edit docs,
production descriptors, or access DigitalOcean.

Exact BEFORE block is stored at
`docs/audit/qwen-evidence/s-230-t4p-r1-before.txt` and is included by the
delegation wrapper.
