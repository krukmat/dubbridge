#!/usr/bin/env python3
"""Fail-closed validation of an OCI OpenTofu plan, without cloud API calls.

The guard is a *necessary* condition for O2, not permission to apply.
Outputs markers for bounded agent consumption. OCI_ALLOW_CREATES is a
comma-separated exact-address allowlist, never a boolean bypass.
"""
import argparse
import ipaddress
import json
import os
import sys

ALLOWED = {
    "oci_core_vcn.poc[0]",
    "oci_core_internet_gateway.poc[0]",
    "oci_core_route_table.poc[0]",
    "oci_core_security_list.poc[0]",
    "oci_core_subnet.poc[0]",
    "oci_core_instance.poc[0]",
}


def blocked(reason):
    print("OCI_PLAN_GUARD=BLOCKED reason=" + reason, file=sys.stderr)
    return 2


def validate_plan(plan, allow):
    if not isinstance(plan, dict) or "resource_changes" not in plan:
        return ["missing-resource-changes"], 0
    if not isinstance(plan["resource_changes"], list):
        return ["invalid-resource-changes"], 0
    errors, creations = [], set()
    for item in plan["resource_changes"]:
        if not isinstance(item, dict):
            errors.append("invalid-resource-change")
            continue
        address = item.get("address", "")
        change = item.get("change") or {}
        actions = change.get("actions") or []
        if actions == ["no-op"]:
            continue
        if actions != ["create"]:
            errors.append(f"{address}:non-create-action:{','.join(actions)}")
            continue
        if address not in ALLOWED or address not in allow:
            errors.append(f"{address}:create-not-allowlisted")
            continue
        if address in creations:
            errors.append(f"{address}:duplicate-create")
            continue
        creations.add(address)
        after = change.get("after") or {}
        unknown = change.get("after_unknown") or {}
        if not isinstance(after, dict) or not isinstance(unknown, dict):
            errors.append(f"{address}:missing-after")
            continue
        if address == "oci_core_instance.poc[0]":
            if after.get("shape") != "VM.Standard.A1.Flex":
                errors.append("instance:non-free-shape")
            shape = (after.get("shape_config") or [{}])[0]
            if shape.get("ocpus") != 2 or shape.get("memory_in_gbs") != 12:
                errors.append("instance:shape-limits")
            src = (after.get("source_details") or [{}])[0]
            if src.get("boot_volume_size_in_gbs") != 50 or src.get("source_type") != "image":
                errors.append("instance:boot-allowance")
            vnic = (after.get("create_vnic_details") or [{}])[0]
            if vnic.get("assign_public_ip") is not True:
                errors.append("instance:public-edge-mismatch")
            for field in ("shape", "shape_config", "source_details", "create_vnic_details"):
                if field in unknown:
                    errors.append("instance:critical-unknown-" + field)
        if address == "oci_core_security_list.poc[0]":
            ingress = after.get("ingress_security_rules") or []
            if len(ingress) != 3:
                errors.append("firewall:wrong-ingress-count")
            ports = []
            for rule in ingress:
                opts = (rule.get("tcp_options") or [{}])[0]
                lo, hi = opts.get("min"), opts.get("max")
                source = rule.get("source")
                if rule.get("protocol") != "6" or lo != hi:
                    errors.append("firewall:non-tcp-or-port-range")
                    continue
                if lo in (80, 443) and source != "0.0.0.0/0":
                    errors.append("firewall:http-scope-mismatch")
                if lo == 22:
                    try:
                        net = ipaddress.ip_network(source)
                        if net.version != 4 or net.prefixlen != 32:
                            errors.append("firewall:ssh-not-operator-32")
                    except (ValueError, TypeError):
                        errors.append("firewall:invalid-ssh-cidr")
                ports.append(lo)
            if sorted(ports) != [22, 80, 443]:
                errors.append("firewall:unexpected-public-ports")
            if "ingress_security_rules" in unknown:
                errors.append("firewall:unknown-ingress-rules")
        if address == "oci_core_subnet.poc[0]":
            if after.get("prohibit_public_ip_on_vnic") is not False:
                errors.append("subnet:public-edge-mismatch")
            if "security_list_ids" not in after and "security_list_ids" not in unknown:
                errors.append("subnet:security-list-not-set")
    if allow != creations:
        for addr in sorted(allow - creations):
            errors.append(f"{addr}:allowlist-address-not-in-plan")
    return errors, len(creations)


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", required=True, help="Existing tofu show -json output")
    parser.add_argument(
        "--allow-create", default=os.getenv("OCI_ALLOW_CREATES", ""),
        help="Comma-separated exact resource addresses, empty = no creation",
    )
    args = parser.parse_args(argv)
    allow = {s.strip() for s in args.allow_create.split(",") if s.strip()}
    if allow - ALLOWED:
        return blocked("unrecognized-address-in-allowlist")
    try:
        with open(args.plan, encoding="utf-8") as stream:
            plan = json.load(stream)
    except (OSError, ValueError):
        return blocked("plan-json-unavailable-or-invalid")
    errors, count = validate_plan(plan, allow)
    for error in errors:
        print("OCI_PLAN_BLOCKED_RESOURCE=" + error, file=sys.stderr)
    if errors:
        return blocked("plan-violates-exact-allowlist-or-cost-contract")
    print(f"OCI_PLAN_CREATE_COUNT={count}")
    print("OCI_PLAN_GUARD=PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
