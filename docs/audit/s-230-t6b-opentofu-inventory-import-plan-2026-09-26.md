# S-230-T6b — OpenTofu descriptor + inventory/import plan

Date: 2026-09-26  
Branch: \`main\`  
Cloud mutation: **NONE**

## Implemented

- OpenTofu root under \`infra/digitalocean/\`.
- OpenTofu/provider constraints; dependency lock remains a local validation output before PASS.
- Partial S3 backend configuration for a dedicated Spaces state bucket.
- Descriptors for Droplet, Cloud Firewall, Managed PostgreSQL, media Space, and \`poc.iotforce.es\`.
- \`prevent_destroy\` on adopted host/stateful resources.
- every \`manage_*\` switch defaults to false.
- read-only inventory helper.
- fail-closed \`make do-plan\`: create/delete/replace are blocked by default.

## Adoption map

| Existing target | OpenTofu address | Import identifier |
|---|---|---|
| owner Droplet at 46.101.217.151 | \`digitalocean_droplet.app[0]\` | exact numeric Droplet ID from inventory |
| production firewall | \`digitalocean_firewall.app[0]\` | firewall UUID |
| Managed PostgreSQL | \`digitalocean_database_cluster.postgres[0]\` | cluster UUID |
| media Space \`dubbridge-poc-v1\` / ams3 | \`digitalocean_spaces_bucket.media[0]\` | provider import ID confirmed during T6d |
| \`poc.iotforce.es\` A record | \`digitalocean_record.poc[0]\` | record numeric ID |

T6d imports only resources proven compatible. Any replacement/destruction is a stop condition.

## T6b/T6d boundary

Creating the dedicated Spaces bucket for OpenTofu state is itself a cloud mutation.
It is therefore **T6d.0**, not T6b. T6b defines the backend contract only.

## Closure evidence

Owner/local runner evidence on 2026-09-26:

\`\`\`bash
./scripts/do-readonly-inventory.sh
cp infra/digitalocean/terraform.tfvars.example infra/digitalocean/terraform.tfvars
cd infra/digitalocean
tofu init -backend=false
tofu providers lock -platform=darwin_arm64 -platform=linux_amd64
cd ../..
make do-plan
\`\`\`

Expected pre-T6d terminal state:

\`\`\`text
DO_PLAN_VALIDATE=PASS
DO_PLAN=BLOCKED reason=remote-state-not-bootstrapped
\`\`\`

That block is intentional: T6b performs no cloud mutation.
