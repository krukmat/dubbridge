# S-230-T6c — Immutable production release packaging

Date: 2026-09-26  
Branch: `main`  
Cloud infrastructure mutation: **NONE**

## Implementation

T6c converts the production runtime from source-build semantics to pull-only
immutable application images.

Application images:

- API
- Gateway
- migration CLI
- worker-runner
- Availability Node

`infra/production/docker-compose.yml` now requires an OCI reference for each
application image. Normal production Compose no longer contains application
`build:` directives.

`scripts/do-release.sh`:

1. requires a full 40-character Git SHA;
2. rejects an unknown revision, mismatched checkout, or dirty tracked worktree;
3. cross-builds `linux/amd64`;
4. pushes to the supplied private registry;
5. resolves the pushed OCI digest;
6. renders production Compose with those digests;
7. hashes the rendered Compose configuration;
8. writes a non-secret `release.json`;
9. fails unless every application image is pinned by `@sha256:<digest>`.

Release identity contains Git SHA, OCI digests, rendered Compose SHA-256,
release ID, UTC creation timestamp, and registry path.

## Command

```bash
make do-release REV="$(git rev-parse HEAD)" \
  REGISTRY="registry.digitalocean.com/<registry-slug>"
```

The executor must already be authenticated for push. T6c does not create or
change Droplets, databases, DNS, Spaces, firewall, or other infrastructure.

## Remaining closure evidence

A real registry slug and push-capable DigitalOcean registry authentication are
required to execute the build/push and obtain authoritative digests.

Expected closure markers:

```text
DO_RELEASE_BUILD_START=<git-sha>
DO_RELEASE_ID=<release-id>
DO_RELEASE_MANIFEST=.../release.json
DO_RELEASE=PASS
RELEASE_VERIFY=PASS
```

T6c is PASS. GitHub Actions run 36257623084 completed successfully on native
amd64. All five application images were pushed to DOCR, `release.json` was
generated, and `RELEASE_VERIFY=PASS` was recorded.

Release identity:

- Git SHA: `222dd061e24522212a8fe0e5169156c60175e436`
- Release ID: `222dd061e245-81319f580679`
- Workflow artifact: `t6c-release-222dd061e24522212a8fe0e5169156c60175e436`


## Local execution finding

Owner execution reached Docker packaging but failed before any image push:

```text
unknown flag: --platform
Usage: docker [OPTIONS] COMMAND [ARG...]
make: *** [do-release] Error 125
```

Diagnosis: the local Docker CLI does not currently expose the Buildx plugin,
so `docker buildx build --platform linux/amd64` cannot run. The release script
now checks `docker buildx version` explicitly and fails closed with:

```text
DO_RELEASE=BLOCKED reason=missing-docker-buildx
```

No release image was certified from this failed attempt.


## GitHub Actions native amd64 path

The owner M5/arm64 run failed inside `aws-lc-sys` while compiling x86_64 C
code under QEMU. The canonical T6c release execution therefore moved to
`.github/workflows/t6c-release.yml` on a native GitHub-hosted x64 runner.

The workflow:

- checks out `main`;
- verifies `x86_64`;
- configures Docker Buildx;
- authenticates `doctl` from repository secret `DIGITALOCEAN_ACCESS_TOKEN`;
- logs Docker into the existing `dubbridge` DigitalOcean registry;
- runs the same `make do-release` contract;
- verifies `release.json`;
- uploads the release directory as workflow evidence.

Local arm64 execution now fails closed by default with
`DO_RELEASE=BLOCKED reason=native-amd64-required-use-ci`. QEMU can only be
re-enabled deliberately with `DUBBRIDGE_ALLOW_QEMU=1`.

Intermediate `artifacts/releases/*/*.ref` files are ignored; the consolidated
`release.json` remains the release evidence of interest.
