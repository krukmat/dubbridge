#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import sys
from pathlib import Path

ROOT = Path(os.environ.get("T6D1_INVENTORY_DIR", "/tmp/dubbridge-t6d1-inventory"))
OUT = Path(os.environ.get("T6D2_MATRIX_FILE", str(ROOT / "adoption-matrix.json")))

EXPECTED = {
    "region": os.environ.get("DO_REGION", "ams3"),
    "droplet_ip": os.environ.get("DO_DROPLET_IP", "46.101.217.151"),
    "droplet_size": os.environ.get("DO_DROPLET_SIZE", "s-2vcpu-4gb"),
    "droplet_image": os.environ.get("DO_DROPLET_IMAGE", "ubuntu-24-04-x64"),
    "database_name": os.environ.get("DO_DATABASE_NAME", "dubbridge-poc-v1"),
    "database_version": os.environ.get("DO_DATABASE_VERSION", "17"),
    "database_size": os.environ.get("DO_DATABASE_SIZE", "db-s-1vcpu-1gb"),
    "media_space": os.environ.get("DO_MEDIA_SPACE", "dubbridge-poc-v1"),
    "domain": os.environ.get("DO_DOMAIN", "iotforce.es"),
    "hostname": os.environ.get("DO_HOSTNAME", "poc"),
}

REQUIRED = [
    "droplet.freeze.json",
    "firewalls.freeze.json",
    "databases.freeze.json",
    "dns.freeze.json",
    "media-space.freeze.json",
    "summary.json",
]


def fail(reason: str) -> None:
    print(f"T6D2_MATRIX=BLOCKED reason={reason}", file=sys.stderr)
    raise SystemExit(2)


def load(name: str):
    path = ROOT / name
    if not path.is_file():
        fail(f"missing-{name}")
    return json.loads(path.read_text(encoding="utf-8"))


def row(resource: str, action: str, address: str, resource_id=None, reasons=None, drift=None):
    return {
        "resource": resource,
        "classification": action,
        "tofu_address": address,
        "resource_id": resource_id,
        "reasons": reasons or [],
        "drift": drift or [],
    }


for required in REQUIRED:
    if not (ROOT / required).is_file():
        fail(f"missing-{required}")

droplets = load("droplet.freeze.json")
firewalls = load("firewalls.freeze.json")
databases = load("databases.freeze.json")
dns = load("dns.freeze.json")
media = load("media-space.freeze.json")
summary = load("summary.json")

matrix = []

# Droplet: it was frozen by exact public IP. Adoption is allowed only when
# region, size and base image are compatible with the frozen descriptor.
if len(droplets) != 1:
    matrix.append(row(
        "droplet", "BLOCKED", "digitalocean_droplet.app[0]",
        reasons=[f"expected exactly one frozen-IP match, got {len(droplets)}"],
    ))
else:
    d = droplets[0]
    reasons = []
    if d.get("region") != EXPECTED["region"]:
        reasons.append(f"region {d.get('region')} != {EXPECTED['region']}")
    if d.get("size_slug") != EXPECTED["droplet_size"]:
        reasons.append(f"size {d.get('size_slug')} != {EXPECTED['droplet_size']}")
    image_slug = (d.get("image") or {}).get("slug")
    if image_slug != EXPECTED["droplet_image"]:
        reasons.append(f"image {image_slug} != {EXPECTED['droplet_image']}")
    drift = []
    if d.get("name") != "dubbridge-poc-v1":
        drift.append(f"name={d.get('name')} -> dubbridge-poc-v1")
    if d.get("monitoring") is not True:
        drift.append("monitoring should be enabled")
    expected_tags = {"dubbridge", "poc-v1", "production"}
    actual_tags = set(d.get("tags") or [])
    missing_tags = sorted(expected_tags - actual_tags)
    if missing_tags:
        drift.append("missing tags: " + ",".join(missing_tags))
    matrix.append(row(
        "droplet",
        "BLOCKED" if reasons else "IMPORT",
        "digitalocean_droplet.app[0]",
        d.get("id"),
        reasons=reasons,
        drift=drift,
    ))

droplet_id = droplets[0].get("id") if len(droplets) == 1 else None

# Firewall: zero attached firewalls means CREATE. More than one attached
# firewall is ambiguous. One firewall is importable only if it does not expose
# unexpected public TCP/UDP ports beyond SSH/HTTP/HTTPS.
attached = []
if droplet_id is not None:
    for fw in firewalls:
        if droplet_id in (fw.get("droplet_ids") or []):
            attached.append(fw)

if len(attached) == 0:
    matrix.append(row(
        "firewall", "CREATE", "digitalocean_firewall.app[0]",
        reasons=["no firewall attached to frozen Droplet"],
    ))
elif len(attached) > 1:
    matrix.append(row(
        "firewall", "BLOCKED", "digitalocean_firewall.app[0]",
        reasons=[f"multiple firewalls attached to frozen Droplet: {len(attached)}"],
    ))
else:
    fw = attached[0]
    forbidden = []
    has80 = False
    has443 = False
    for rule in fw.get("inbound_rules") or []:
        proto = rule.get("protocol")
        ports = str(rule.get("ports") or rule.get("port_range") or "")
        sources = rule.get("sources") or {}
        addrs = sources.get("addresses") if isinstance(sources, dict) else None
        if addrs is None:
            addrs = rule.get("source_addresses") or []
        public = "0.0.0.0/0" in addrs or "::/0" in addrs
        if proto == "tcp" and ports == "80" and public:
            has80 = True
        if proto == "tcp" and ports == "443" and public:
            has443 = True
        if public and proto in {"tcp", "udp"} and ports not in {"22", "80", "443"}:
            forbidden.append(f"{proto}/{ports}")
    drift = []
    if not has80:
        drift.append("missing public tcp/80")
    if not has443:
        drift.append("missing public tcp/443")
    matrix.append(row(
        "firewall",
        "BLOCKED" if forbidden else "IMPORT",
        "digitalocean_firewall.app[0]",
        fw.get("id"),
        reasons=(["unexpected public inbound: " + ",".join(sorted(forbidden))] if forbidden else []),
        drift=drift,
    ))

# Managed PostgreSQL: exact name is the identity anchor. Absence => CREATE.
# Multiple same-name clusters or mismatched engine/version/region/size/nodes
# require owner review before mutation.
named_dbs = [d for d in databases if d.get("name") == EXPECTED["database_name"]]
if len(named_dbs) == 0:
    matrix.append(row(
        "database", "CREATE", "digitalocean_database_cluster.postgres[0]",
        reasons=["no managed PostgreSQL cluster with frozen name"],
    ))
elif len(named_dbs) > 1:
    matrix.append(row(
        "database", "BLOCKED", "digitalocean_database_cluster.postgres[0]",
        reasons=[f"multiple clusters named {EXPECTED['database_name']}"],
    ))
else:
    db = named_dbs[0]
    reasons = []
    if db.get("engine") not in {"pg", "postgres", "postgresql"}:
        reasons.append(f"engine {db.get('engine')} is not PostgreSQL")
    if str(db.get("version")) != EXPECTED["database_version"]:
        reasons.append(f"version {db.get('version')} != {EXPECTED['database_version']}")
    region = db.get("region")
    if isinstance(region, dict):
        region = region.get("slug")
    if region != EXPECTED["region"]:
        reasons.append(f"region {region} != {EXPECTED['region']}")
    if db.get("size") != EXPECTED["database_size"]:
        reasons.append(f"size {db.get('size')} != {EXPECTED['database_size']}")
    if db.get("num_nodes") not in {1, None}:
        reasons.append(f"num_nodes {db.get('num_nodes')} != 1")
    matrix.append(row(
        "database",
        "BLOCKED" if reasons else "IMPORT",
        "digitalocean_database_cluster.postgres[0]",
        db.get("id"),
        reasons=reasons,
    ))

# Media Space: T6d.1 already proved state. ABSENT is a valid CREATE candidate.
media_state = media.get("state")
if media_state == "ABSENT":
    matrix.append(row(
        "media_space", "CREATE", "digitalocean_spaces_bucket.media[0]",
        reasons=[f"{EXPECTED['media_space']} absent in {EXPECTED['region']}"],
    ))
elif media_state == "PRESENT":
    matrix.append(row(
        "media_space", "IMPORT", "digitalocean_spaces_bucket.media[0]",
        EXPECTED["media_space"],
    ))
else:
    matrix.append(row(
        "media_space", "BLOCKED", "digitalocean_spaces_bucket.media[0]",
        reasons=[f"media Space state is {media_state}"],
    ))

# DNS: exact hostname may be represented by DO as "poc" or FQDN.
fqdn = f"{EXPECTED['hostname']}.{EXPECTED['domain']}"
host_records = [
    r for r in dns
    if r.get("type") == "A" and r.get("name") in {EXPECTED["hostname"], fqdn}
]
matching = [r for r in host_records if r.get("data") == EXPECTED["droplet_ip"]]
if len(matching) == 1 and len(host_records) == 1:
    r = matching[0]
    drift = []
    if r.get("ttl") != 300:
        drift.append(f"ttl={r.get('ttl')} -> 300")
    matrix.append(row(
        "dns", "IMPORT", "digitalocean_record.poc[0]", r.get("id"), drift=drift
    ))
elif len(host_records) == 0:
    matrix.append(row(
        "dns", "CREATE", "digitalocean_record.poc[0]",
        reasons=[f"no A record for {fqdn}"],
    ))
else:
    matrix.append(row(
        "dns", "BLOCKED", "digitalocean_record.poc[0]",
        reasons=[
            f"hostname records={len(host_records)}, matching frozen IP={len(matching)}"
        ],
    ))

blocked = [r for r in matrix if r["classification"] == "BLOCKED"]
imports = [r for r in matrix if r["classification"] == "IMPORT"]
creates = [r for r in matrix if r["classification"] == "CREATE"]

doc = {
    "schema": "dubbridge-t6d2-adoption-matrix-v1",
    "inventory_schema": summary.get("schema"),
    "inventory_dir": str(ROOT),
    "expected": EXPECTED,
    "summary": {
        "import": len(imports),
        "create": len(creates),
        "blocked": len(blocked),
    },
    "resources": matrix,
}
OUT.write_text(json.dumps(doc, indent=2, sort_keys=True) + "\n", encoding="utf-8")

for r in matrix:
    rid = r.get("resource_id")
    rid_part = f" id={rid}" if rid not in {None, ""} else ""
    print(f"T6D2_RESOURCE={r['resource']} classification={r['classification']}{rid_part}")
    for reason in r.get("reasons") or []:
        print(f"T6D2_REASON={r['resource']} {reason}")
    for drift in r.get("drift") or []:
        print(f"T6D2_DRIFT={r['resource']} {drift}")

print(
    f"T6D2_SUMMARY=import:{len(imports)},create:{len(creates)},blocked:{len(blocked)}"
)
print(f"T6D2_MATRIX_FILE={OUT}")

if blocked:
    print("T6D2_MATRIX=BLOCKED")
    raise SystemExit(3)

print("T6D2_MATRIX=PASS")
