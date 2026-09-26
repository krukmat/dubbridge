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

T6c remains IN PROGRESS until the pushed release manifest is produced and
verified.
