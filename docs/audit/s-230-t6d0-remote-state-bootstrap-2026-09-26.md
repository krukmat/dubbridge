# S-230-T6d.0 — OpenTofu remote-state bootstrap

Date: 2026-09-26
Branch: `main`
Status: READY FOR OWNER EXECUTION
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

T6d.0 is not PASS until the owner executes the command with a Spaces key that
has bucket-level permission and returns the four markers above.
