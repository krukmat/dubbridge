# S-230-T6a — Digital Ocean deployment contract freeze

**Date:** 2026-09-26  
**Task:** S-230-T6a  
**Parent:** S-230-T6 — First deploy and end-to-end smoke on Digital Ocean  
**Disposition:** PASS  
**Cloud mutation:** NONE

## Purpose

Freeze the deployment decisions that T6b–T6g must consume before any new
Digital Ocean mutation is allowed. T6a is a planning/docs contract: it does not
provision, import, build, push, deploy, migrate, rotate secrets, or run the E2E
smoke.

The contract optimizes for a low-context executor. Infrastructure remains
declarative and auditable; normal operation is exposed through six stable
commands with compact machine-readable summaries. Detailed logs are evidence
artifacts, not normal chat output.

## Inputs already frozen before T6a

T5/T5a already established these production inputs and T6a preserves them:

| Input | Frozen value |
|---|---|
| Deployment target | single Digital Ocean Droplet + production Docker Compose |
| Region | `ams3` |
| Spaces endpoint | `https://ams3.digitaloceanspaces.com` |
| Media bucket | `dubbridge-poc-v1` |
| Public hostname | `poc.iotforce.es` |
| Reported public Droplet IP | `46.101.217.151` |
| TLS proxy | Caddy / automatic TLS |
| Public upload ceiling | 100 MiB |
| JWT expiry | 8 h |
| Translation provider | `http`, credentials injected |
| PostgreSQL | Digital Ocean Managed PostgreSQL |
| Redis | production Compose service |
| Production config model | ADR-026 fail-closed, secrets injected |

The T5a ledger records that the owner had already provisioned a Droplet and
that the hostname resolved to `46.101.217.151`. T6a therefore freezes an
**adopt/import-first** policy: T6b inventories and imports compatible existing
resources into IaC instead of blindly recreating them. An existing resource is
replaced only after an explicit owner-approved replacement plan.

## T6a.1 — Deployment identity freeze

A deployment is identified by all of the following, not by Git SHA alone:

- exact Git commit SHA;
- immutable OCI digest for every DubBridge application image;
- rendered production Compose/config hash;
- IaC revision (Git SHA containing the applied OpenTofu descriptor);
- release ID and UTC creation timestamp.

The release manifest is non-secret and must be sufficient to answer "what is
running?" without SSH exploration. Mutable tags such as `latest` are not
deployment identity.

## T6a.2 — Digital Ocean topology freeze

Desired logical topology:

```text
Digital Ocean project
├── VPC (ams3)
├── existing/adopted Droplet when compatible
│   └── Docker Compose runtime
├── Managed PostgreSQL (ams3)
├── Spaces
│   ├── dubbridge-poc-v1            # media
│   └── dedicated IaC-state bucket  # never media
├── Digital Ocean Container Registry
├── Cloud Firewall
├── DNS / existing poc.iotforce.es record
└── monitoring / uptime evidence
```

Rules:

1. T6b discovers current account resources before declaring create actions.
2. Existing compatible Droplet, DNS, media bucket, and database resources are
   imported/adopted where supported; no duplicate resource is created merely to
   satisfy IaC ownership.
3. The existing Droplet must satisfy the previously frozen >=8 GiB RAM floor.
   Its exact slug/image are observed facts to record in T6b, not invented here.
4. A new Droplet, if replacement is explicitly required later, uses Ubuntu LTS
   with cloud-init bootstrap and the same >=8 GiB RAM floor.
5. The IaC-state Space is separate from application media. Its globally unique
   physical bucket name is resolved during T6b bootstrap; its logical role and
   isolation are frozen here.
6. No Kubernetes/App Platform layer is introduced for this POC.

## T6a.3 — Network and exposure contract

Only Caddy owns public application ports.

| Surface | Public internet | VPC / restricted | Compose internal |
|---|---:|---:|---:|
| Caddy 80/443 | yes | n/a | yes |
| SSH 22 | restricted operator source only | n/a | no |
| API 8080 | no | no | yes |
| Gateway 8081 | no | no | yes |
| Redis 6379 | no | no | yes |
| Availability Node 8443 | no | no host binding | `p2p-control` only |
| Managed PostgreSQL | no general public access | trusted-source/TLS only | client connection |

`EXPOSE 8443` in the Availability Node image does not authorize a host port.
The T6p-c contract proved the desired local boundary with no host PortBindings;
T6e/T7a must preserve it.

Cloud Firewall and host/runtime configuration must fail closed: 80/443 are the
only public application ports. SSH is not an application surface and is
restricted to the operator source policy selected at provisioning time.

## T6a.4 — Persistence ownership

| State | Owner / persistence |
|---|---|
| Media objects | `dubbridge-poc-v1` Space |
| Relational state | Managed PostgreSQL |
| Redis job/cache state | Compose Redis volume/runtime contract |
| P2P ciphertext packages | persistent Docker volume |
| Hyperdrive data | persistent Docker volume |
| P2P index | persistent Docker volume |
| IaC state + lock | dedicated Spaces bucket |
| Release manifests/evidence | repo/audit artifact + executor evidence directory |

Normal `deploy`, `restart`, and `rollback` operations must never delete
persistent data. Destruction is a separate explicit operation and is not part
of the six-command agent interface.

The remote IaC backend uses the S3-compatible Spaces endpoint, credentials from
environment variables, bucket versioning where supported, and state locking
(`use_lockfile=true`). T6b must prove `tofu init` + locking behavior before
the first infrastructure apply.

## T6a.5 — Secret and credential ownership

No secret value is frozen or committed by T6a. The following ownership classes
are frozen:

| Secret class | Consumer |
|---|---|
| Digital Ocean API token | local/CI infrastructure executor only |
| IaC-state Spaces key/secret | OpenTofu backend executor only |
| media Spaces key/secret | API/worker services that require object storage |
| PostgreSQL credentials/URL | migration + API/worker/gateway as required by current config |
| Redis credentials/URL | application services that require Redis |
| JWT/application auth secret | API/gateway according to current ADR-026 config |
| translation API URL/key | translation worker only |
| registry push credential | build/publish executor only |
| registry pull credential | Droplet; read-only and bounded where supported |
| P2P CA + worker client identity | worker-runner |
| P2P CA + server cert/key + allowed client fingerprint | Availability Node |
| P2P KEK material | API/worker only where current P2 contract requires it; never Availability Node |

Rules:

- no real secret in Git, IaC source, tfvars committed to Git, release manifest,
  normal logs, or audit evidence;
- backend credentials are supplied through environment variables, not
  `-backend-config` secret values;
- generated production PEM material lives outside Git;
- the T6p-b/c service-specific deny-list remains authoritative for P2P secret
  exposure;
- missing required secret material is a pre-mutation or startup failure, never a
  degraded PASS.

## T6a.6 — Runtime and immutable-release contract

Production is a **pull-and-run** target, not a build host.

Allowed release path:

```text
source -> build/test -> private registry -> immutable digest -> production pull
```

The production Droplet must not use `git pull`, `docker compose build`,
`cargo build`, `npm install`, or equivalent as the normal deployment path.

OpenTofu is the canonical IaC CLI. The Digital Ocean provider is version
constrained and its dependency lock file is committed. T6b verifies the exact
OpenTofu/provider combination before apply. `doctl` is an auxiliary
inspection/bootstrap/registry-auth tool, not infrastructure source of truth.

Cloud-init is bootstrap-only for newly created replacement hosts. The already
reported existing Droplet is not replaced merely to gain cloud-init; T6b
records its observed bootstrap state and converges only the minimum safe host
prerequisites needed by the release runner.

## T6a.7 — Low-context agent interface

The canonical operator surface is exactly:

```text
make do-plan
make do-provision
make do-deploy REV=<git-sha>
make do-smoke
make do-status
make do-rollback RELEASE=<release-id>
```

T6a defines the contract; later children implement it. CI must eventually call
the same interface rather than contain a second deployment implementation.

Normal successful output is compact and stable, for example:

```text
DO_DEPLOY=PASS
RELEASE=<release-id>
GIT_SHA=<sha>
API=healthy
GATEWAY=healthy
WORKER=healthy
AVAILABILITY_NODE=healthy
```

Failure output is also bounded:

```text
DO_DEPLOY=FAIL
STAGE=<stage>
ERROR_CODE=<stable-code>
EVIDENCE=<path>
```

Verbose logs are written to evidence files and printed only on explicit request
or bounded failure diagnosis. Every command returns a meaningful process exit
code.

## T6a.8 — Evidence and failure contract

Per-run evidence root:

```text
artifacts/deploy/<run-id>/
├── summary.env
├── summary.json
├── release.json
├── compose-config.sha256
├── health.json
├── plan/
└── logs/
```

This path is runtime evidence and must remain ignored from Git unless a bounded,
redacted audit artifact is intentionally promoted into `docs/audit/`.

Fail-closed conditions include:

- required input absent;
- unknown/unresolvable release identity;
- unexpected destructive IaC plan;
- existing-resource adoption ambiguity;
- secret/config validation failure;
- migration failure;
- health timeout;
- E2E downstream-state mismatch;
- rollback target ambiguity.

A 2xx response alone is never a smoke PASS. T6f inherits the parent T6 rule
that each pipeline stage is proven by authoritative downstream state.

## T6a.9 — Handoff and child boundaries

T6 is now a non-executable parent:

1. **T6a PASS** — this freeze.
2. **T6b** — implement and validate OpenTofu descriptors; inventory/import plan;
   no apply.
3. **T6c** — build/test/push immutable release and record digests.
4. **T6d** — first controlled cloud mutation/import/apply and no-drift proof.
5. **T6e** — deploy, migrate, TLS/readiness/network-boundary proof.
6. **T6f** — real-video base E2E smoke using authoritative downstream state.
7. **T6g** — restart/rollback/log-access/runbook/cost evidence and aggregate
   T6 closure.

T7a does not reprovision T6. It consumes the same environment and release
mechanism after T6 PASS, then proves ciphertext publication and durable
`P2P_READY`. If T7a requires a source/config change to a surface certified by
T6p-c, T6p-b/c is reopened and recertified before T7a continues.

## Non-goals

T6a does not:

- mutate Digital Ocean;
- create/import resources;
- change DNS;
- build or push images;
- generate production certificates/KEKs;
- deploy Compose;
- run migrations;
- run E2E;
- change product/P2P semantics;
- claim T6 or T7a PASS.

## Closure markers

```text
T6A_CONTRACT=PASS
T6A_TOPOLOGY_FREEZE=PASS
T6A_ADOPT_IMPORT_FIRST=PASS
T6A_NETWORK_FREEZE=PASS
T6A_PERSISTENCE_FREEZE=PASS
T6A_SECRET_OWNERSHIP=PASS
T6A_RELEASE_CONTRACT=PASS
T6A_AGENT_INTERFACE=PASS
T6A_EVIDENCE_CONTRACT=PASS
T6A_CLOUD_MUTATION=NONE
T6A=PASS
```

## Verification basis

- T5a/T5c/T5d production decisions and existing-resource record in the S-230
  task ledger.
- `infra/production/docker-compose.yml`, `.env.example`, and
  `config/README.md` current production/P2P boundaries.
- T6p-c exact runtime certification remains pinned to tested head
  `c4b8da98c93e80b88ed06a466226fe00273514d2`; T6a does not relabel later
  docs-only commits as runtime-certified.
- Digital Ocean documents Spaces as an S3-compatible remote Terraform state
  backend and recommends environment-provided backend credentials.
- OpenTofu documents provider version constraints/lockfiles and S3
  `use_lockfile=true` state locking.
- Digital Ocean documents cloud-init/user-data as first-boot configuration and
  DOCR read-only registry login support.

## RRI / review disposition

Presentation/execution-time RRI is **25 Low / Effort S**. Deterministic input:
six touched planning/docs surfaces
(`docs/audit/s-230-t6a-do-deployment-contract-2026-09-26.md`,
`.agent/s230-t6-execution.md`, task ledger, S-230 plan, roadmap, and HTML
mirror), C=0, D=0, K=0, P=0, T=0, A=1, X=2. With the DubBridge docs anchor
floors at zero and no penalty trigger, RRI-v2's ICI bridge resolves the task to
the Low-band ceiling 25. Equivalent command:

```text
python3 scripts/rri.py --platform dubbridge --C 0 \
  --touches docs/audit/s-230-t6a-do-deployment-contract-2026-09-26.md \
  --touches .agent/s230-t6-execution.md \
  --touches docs/tasks/s-230-poc-v1-digitalocean.md \
  --touches docs/plan/s-230-poc-v1-digitalocean.md \
  --touches docs/plan/roadmap.md \
  --touches docs/audit/go-live-octubre-2026-mirror.html \
  --D 0 --K 0 --P 0 --T 0 --A 1 --X 2
# final: 25 Low / Effort S
```

This child is docs/planning-only: it changes deployment governance and task
structure but no runtime/config/code or cloud state. Under the repository
workflow, task-analysis and code-solution review are exempt for
docs/config/migration/ADR/plan/task-ledger/policy-only work. T6b must perform
its own presentation-time RRI because it introduces executable IaC.
