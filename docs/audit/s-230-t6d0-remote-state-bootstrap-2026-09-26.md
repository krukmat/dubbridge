# S-230-T6d.0 — OpenTofu remote-state bootstrap

Date: 2026-09-26
Branch: `main`
Status: PASS
Cloud mutation: **YES — one dedicated Spaces bucket only**

## Scope

T6d.0 bootstraps the dedicated OpenTofu remote-state bucket before any base
platform import/apply.

Default contract:

- region: `ams3`
- bucket: `dubbridge-poc-v1-opentofu-state`
- state key: `dubbridge/poc-v1/opentofu.tfstate`
- private bucket
- object versioning enabled
- no application/media data
- no Droplet/database/DNS/firewall mutation

## Operator command

```bash
export SPACES_ACCESS_KEY_ID='...'
export SPACES_SECRET_ACCESS_KEY='...'
make do-state-bootstrap
```

Optional overrides: `DO_STATE_BUCKET`, `DO_STATE_REGION`, `DO_STATE_KEY`.

Expected markers:

```text
DO_STATE_BUCKET_CREATE=PASS
# or: DO_STATE_BUCKET_CREATE=SKIP_ALREADY_EXISTS
DO_STATE_VERSIONING=PASS
DO_STATE_BACKEND_INIT=PASS
DO_STATE_BOOTSTRAP=PASS
```

The generated `infra/digitalocean/backend.hcl` contains no credentials and
remains gitignored. The script is idempotent with respect to an already
accessible bucket.

Owner execution completed successfully.

Observed markers:

```text
DO_STATE_BUCKET=dubbridge-poc-v1-opentofu-state
DO_STATE_REGION=ams3
DO_STATE_BUCKET_CREATE=PASS
DO_STATE_VERSIONING=PASS
DO_STATE_BACKEND_INIT=PASS
DO_STATE_BOOTSTRAP=PASS
```

T6d.0 is therefore **PASS**. The dedicated remote-state bucket now exists,
versioning is enabled, and the OpenTofu backend initializes successfully.


## Canonical sequencing

T6a, T6b and T6c are PASS. T6d.0 is the current active block and is the first
allowed Digital Ocean mutation in the base-deployment lane.

After `DO_STATE_BOOTSTRAP=PASS`, T6d continues with controlled import/adopt,
guarded plan/apply and a second no-drift plan. T6e must not start before T6d
PASS.


## Closure

`T6D0=PASS`

Cloud mutation performed: creation of the dedicated private remote-state Space
`dubbridge-poc-v1-opentofu-state` in `ams3` plus versioning enablement.
No Droplet, firewall, database, DNS or media-Space mutation occurred.
