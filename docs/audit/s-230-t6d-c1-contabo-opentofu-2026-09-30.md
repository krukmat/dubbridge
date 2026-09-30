# S-230-T6d.C1 — Contabo OpenTofu descriptor and guarded plan

Date: 2026-09-30
Branch: `main`
Status: **IMPLEMENTED — LOCAL VALIDATION PENDING**
Cloud mutation: **NONE**

## Scope implemented

- `infra/contabo/main.tf`
  - OpenTofu >=1.10,<2.0;
  - exact Contabo provider pin `contabo/contabo = 0.1.44`;
  - credentials delegated to provider environment variables;
  - VPS and firewall resources gated by booleans that default to false;
  - explicit product/image/SSH-key requirements before instance creation;
  - `prevent_destroy` on instance and firewall;
  - public firewall surface restricted to 80/443 plus operator-scoped SSH.
- `infra/contabo/backend.hcl.example`
  - Cloudflare R2 S3-compatible backend shape;
  - no credentials in backend config;
  - no unproven `use_lockfile` assumption.
- `infra/contabo/terraform.tfvars.example`
  - defaults to no resource creation;
  - exact SKU/image/SSH IDs remain deliberately unresolved until T6d.C2.
- `scripts/infra-plan.sh`
  - validates before state bootstrap;
  - fails closed when backend is absent;
  - blocks all delete/replace actions;
  - allows create only by exact resource-address allowlist;
  - rejects stale/unused allowlist entries.
- `scripts/infra-state-backup.sh`
  - pulls/validates current state;
  - writes a local checksum copy;
  - uploads a timestamped backup object to the dedicated R2 state bucket;
  - verifies the remote backup object before reporting PASS.
- provider-neutral Make targets:
  - `make infra-plan`
  - `make infra-state-backup`

## Verified external contracts

Contabo's current provider documentation lists version 0.1.44 and supports
`contabo_instance` with product id, region, image id, SSH secret ids and
cloud-init-capable instances. It also exposes a firewall resource.

Cloudflare documents R2 as an S3-compatible Terraform remote backend with
`region=auto`, the account-specific R2 endpoint and S3 validation skips.

OpenTofu recommends remote state and warns that backend credentials should be
provided through environment variables rather than persisted backend
configuration.

## Guard contract

Normal no-create validation:

```bash
cp infra/contabo/terraform.tfvars.example infra/contabo/terraform.tfvars
make infra-plan
```

Before the R2 state bucket is bootstrapped the expected terminal marker is:

```text
INFRA_PLAN_VALIDATE=PASS
INFRA_PLAN=BLOCKED reason=remote-state-not-bootstrapped
```

Once backend and exact T6d.C2 inputs exist, resource creation is authorized
address-by-address, for example:

```bash
INFRA_ALLOW_CREATE='contabo_instance.app[0],contabo_firewall.app[0]' make infra-plan
```

Any unlisted create, any delete, any replacement, or an allowlisted address not
present in the plan blocks the plan.

## Remaining closure evidence

C1 must not be marked PASS until an operator runs the local validation with
OpenTofu/provider initialization and records:

1. provider lockfile generation;
2. `tofu validate` PASS;
3. expected remote-state bootstrap block before C2;
4. no cloud mutation.

No Contabo or Cloudflare resource has been provisioned by this task.
