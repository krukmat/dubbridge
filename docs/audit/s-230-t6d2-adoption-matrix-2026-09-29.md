# S-230-T6d.2 — Adoption compatibility matrix

Date: 2026-09-29
Branch: `main`
Status: BLOCKED — OWNER DISPOSITION REQUIRED
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
