# S-230-T6d.2 — Adoption compatibility matrix

Date: 2026-09-29
Branch: `main`
Status: OWNER-APPROVED TOPOLOGY AMENDMENT — PREPARE BACKUP-FIRST REUSE
Cloud mutation: **NONE**

## Purpose

Convert the frozen T6d.1 inventory into a deterministic resource-by-resource
classification:

- `IMPORT` — an existing resource is structurally compatible with the frozen
  OpenTofu contract;
- `CREATE` — a required resource is unequivocally absent;
- `BLOCKED` — identity is ambiguous or compatibility would require an unsafe
  or owner-reviewed mutation.

## Command

```bash
make do-adoption-matrix
```

Input:

```text
/tmp/dubbridge-t6d1-inventory/
```

Output:

```text
/tmp/dubbridge-t6d1-inventory/adoption-matrix.json
```

Expected terminal shape:

```text
T6D2_RESOURCE=<name> classification=<IMPORT|CREATE|BLOCKED> [id=<id>]
...
T6D2_SUMMARY=import:<n>,create:<n>,blocked:<n>
T6D2_MATRIX_FILE=/tmp/dubbridge-t6d1-inventory/adoption-matrix.json
T6D2_MATRIX=PASS
```

A `BLOCKED` row exits non-zero and prevents T6d.3/T6d.4 progression.

## Safety boundary

This task performs no Digital Ocean API mutation and no OpenTofu import/apply.
It only consumes the sanitized T6d.1 evidence.


## Executed matrix result

Owner execution against the frozen T6d.1 inventory returned:

```text
T6D2_RESOURCE=droplet classification=BLOCKED id=144322723
T6D2_REASON=droplet region fra1 != ams3
T6D2_REASON=droplet size s-1vcpu-2gb != s-2vcpu-4gb
T6D2_REASON=droplet image None != ubuntu-24-04-x64
T6D2_DRIFT=droplet name=wordpress-s-1vcpu-1gb-fra1-01 -> dubbridge-poc-v1
T6D2_DRIFT=droplet monitoring should be enabled
T6D2_DRIFT=droplet missing tags: dubbridge,poc-v1,production
T6D2_RESOURCE=firewall classification=CREATE
T6D2_RESOURCE=database classification=CREATE
T6D2_RESOURCE=media_space classification=CREATE
T6D2_RESOURCE=dns classification=IMPORT id=1830236122
T6D2_DRIFT=dns ttl=3600 -> 300
T6D2_SUMMARY=import:1,create:3,blocked:1
T6D2_MATRIX=BLOCKED
```

## Root cause

The previously frozen public IP `46.101.217.151` resolves to an existing
WordPress Droplet in `fra1`, not to a DubBridge-compatible production host.
Its region, size, image identity, name, monitoring state and tags do not satisfy
the frozen S-230 deployment contract.

This resource must **not** be imported into
`digitalocean_droplet.app[0]` and must not be resized/reimaged/renamed as a
shortcut, because that could destroy or repurpose an unrelated workload.

## Proposed disposition

Preserve Droplet `144322723` unchanged as legacy/external state.

For DubBridge:

- create a new `s-2vcpu-4gb` Ubuntu 24.04 Droplet in `ams3`;
- create the dedicated Cloud Firewall;
- create the required Managed PostgreSQL cluster;
- create media Space `dubbridge-poc-v1`;
- retain/import the existing DNS record object, but do not repoint it until the
  new host has been provisioned and validated.

This changes the T6a assumption from "adopt the reported existing Droplet" to
"preserve unrelated legacy Droplet and provision a dedicated DubBridge host".
Owner approval is required before converting the Droplet row from `BLOCKED`
to an explicit `CREATE` decision.


## Owner disposition — reuse existing Droplet

Owner decision: **reuse Droplet `144322723` for the DubBridge POC rather than
create a second Droplet**. The existing WordPress workload is not currently
needed, but its template/data must be preserved before repurposing.

This supersedes the earlier proposal to preserve the WordPress host unchanged.

### Revised topology

- compute host: existing Droplet `144322723`
- runtime region: `fra1`
- target size after controlled resize: `s-2vcpu-4gb`
- WordPress: backup/export first, then retired from active service
- DubBridge runtime: dedicated Docker/Compose workload on the reused host
- Managed PostgreSQL: create in `fra1`
- media Space: create in `fra1`
- OpenTofu remote-state Space remains in `ams3`
- existing DNS record remains imported and continues to target the same public
  IP, avoiding a cutover to a second host

### Mandatory pre-mutation preservation gate

Before resize, package removal, reverse-proxy replacement, or any other
repurposing step:

1. record current Droplet identity and disk/runtime inventory;
2. export WordPress database;
3. archive WordPress files, uploads and active configuration/theme/plugin data;
4. produce a restore manifest/checksum set;
5. optionally take a DigitalOcean Droplet snapshot as an additional rollback
   artifact;
6. verify the backup artifacts are readable before continuing.

No destructive cleanup is authorized until that preservation gate passes.

### T6d.2 revised classification

The original classifier correctly returned `BLOCKED` because it compared the
host against the old `ams3`/fresh-Droplet contract. Under the owner-approved
topology amendment, the host becomes an explicit **REUSE + RESIZE** target
rather than an `IMPORT` of an already-compatible host.

The remaining resource decisions are:

- Droplet: `REUSE + RESIZE` after WordPress backup gate;
- Firewall: `CREATE`;
- Managed PostgreSQL: `CREATE` in `fra1`;
- media Space: `CREATE` in `fra1`;
- DNS: `IMPORT` with existing record retained.

T6d.2 is considered resolved by owner disposition; execution continues through
a backup-first preparation child before any host mutation.
