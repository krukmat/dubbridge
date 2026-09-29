# S-230-T6d.1 — Fresh Digital Ocean inventory freeze

Date: 2026-09-28
Branch: `main`
Status: READY FOR OWNER EXECUTION
Cloud mutation: **NONE**

## Purpose

Produce a fresh, sanitized, authoritative inventory immediately before T6d.2
compatibility/adoption classification. The older T6b inventory is planning
evidence and is not reused as proof that cloud state has not changed.

## Read-only scope

- Droplet owning the frozen public IP `46.101.217.151`
- Cloud Firewalls
- Managed PostgreSQL clusters
- `iotforce.es` DNS records
- media Space `dubbridge-poc-v1` in `ams3`

No import, create, update, delete, DNS mutation, state mutation or apply occurs.

## Command

```bash
export DIGITALOCEAN_ACCESS_TOKEN='...'
export DO_SPACES_ACCESS_KEY='...'
export DO_SPACES_SECRET_KEY='...'
make do-inventory-freeze
```

Expected markers:

```text
T6D1_DROPLET_MATCH=PASS count=1
T6D1_MEDIA_SPACE=<PRESENT|ABSENT|ACCESS_DENIED> name=dubbridge-poc-v1 region=ams3
T6D1_DNS_MATCHES=<n>
T6D1_INVENTORY_DIR=/tmp/dubbridge-t6d1-inventory
T6D1_INVENTORY=PASS
```

## Output contract

```text
/tmp/dubbridge-t6d1-inventory/
├── droplet.freeze.json
├── firewalls.freeze.json
├── databases.freeze.json
├── dns.freeze.json
├── media-space.freeze.json
└── summary.json
```

The files contain no credentials. T6d.2 consumes these frozen attributes to
classify each target as IMPORT, CREATE or BLOCKED.


## Media Space probe semantics

A 404 from `HeadBucket` is recorded as `ABSENT`, not as a T6d.1 failure.
That absence is valid inventory evidence and is consumed by T6d.2 when deciding
whether the media Space is a controlled `CREATE` candidate.

A 403 is recorded separately as `ACCESS_DENIED`; it must not be conflated with
absence.
