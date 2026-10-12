#!/usr/bin/env python3
"""Offline contract checks; no OpenTofu provider, network or credential needed."""
import importlib.util
import pathlib
import unittest

path = pathlib.Path(__file__).with_name("oci-plan-guard.py")
spec = importlib.util.spec_from_file_location("oci_plan_guard", path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


def change(address, after=None, actions=None, after_unknown=None):
    return {
        "address": address,
        "change": {
            "actions": ["create"] if actions is None else actions,
            "after": {} if after is None else after,
            "after_unknown": {} if after_unknown is None else after_unknown,
        },
    }


INSTANCE = "oci_core_instance.poc[0]"
FIREWALL = "oci_core_security_list.poc[0]"
SUBNET = "oci_core_subnet.poc[0]"


def instance(shape="VM.Standard.A1.Flex", cpus=2, memory=12, boot=50):
    return change(INSTANCE, {
        "shape": shape,
        "shape_config": [{"ocpus": cpus, "memory_in_gbs": memory}],
        "source_details": [{"source_type": "image", "boot_volume_size_in_gbs": boot}],
        "create_vnic_details": [{"assign_public_ip": True}],
    })


def ingress(port, source):
    return {
        "protocol": "6",
        "source": source,
        "tcp_options": [{"min": port, "max": port}],
    }


def firewall(ssh_cidr="198.51.100.4/32"):
    return change(FIREWALL, {
        "ingress_security_rules": [
            ingress(22, ssh_cidr),
            ingress(80, "0.0.0.0/0"),
            ingress(443, "0.0.0.0/0"),
        ],
    })


class OraclePlanGuardTests(unittest.TestCase):
    def test_disabled_zero_create(self):
        self.assertEqual(mod.validate_plan({"resource_changes": []}, set()), ([], 0))

    def test_exact_authorized_instance(self):
        self.assertEqual(mod.validate_plan({"resource_changes": [instance()]}, {INSTANCE}), ([], 1))

    def test_unapproved_create_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [instance()]}, set())
        self.assertTrue(any("create-not-allowlisted" in e for e in errors))

    def test_unrecognized_create_denied(self):
        unknown = "oci_database_autonomous_database.surprise[0]"
        errors, _ = mod.validate_plan({"resource_changes": [change(unknown)]}, {unknown})
        self.assertTrue(errors)

    def test_paid_shape_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [instance(shape="VM.Standard.E4.Flex")]}, {INSTANCE})
        self.assertIn("instance:non-free-shape", errors)

    def test_memory_over_budget_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [instance(memory=24)]}, {INSTANCE})
        self.assertIn("instance:shape-limits", errors)

    def test_boot_over_budget_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [instance(boot=250)]}, {INSTANCE})
        self.assertIn("instance:boot-allowance", errors)

    def test_instance_unknown_values_denied(self):
        x = instance()
        x["change"]["after_unknown"] = {"shape_config": True}
        errors, _ = mod.validate_plan({"resource_changes": [x]}, {INSTANCE})
        self.assertIn("instance:critical-unknown-shape_config", errors)

    def test_delete_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [change(INSTANCE, actions=["delete"])]}, {INSTANCE})
        self.assertTrue(any("non-create-action" in e for e in errors))

    def test_replace_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [change(INSTANCE, actions=["delete", "create"])]}, {INSTANCE})
        self.assertTrue(any("non-create-action" in e for e in errors))

    def test_update_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [change(INSTANCE, actions=["update"])]}, set())
        self.assertTrue(any("non-create-action" in e for e in errors))

    def test_stale_allowlist_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": []}, {INSTANCE})
        self.assertTrue(any("not-in-plan" in e for e in errors))

    def test_firewall_allowed(self):
        self.assertEqual(mod.validate_plan({"resource_changes": [firewall()]}, {FIREWALL}), ([], 1))

    def test_firewall_ssh_world_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [firewall("0.0.0.0/0")]}, {FIREWALL})
        self.assertIn("firewall:ssh-not-operator-32", errors)

    def test_firewall_extra_public_port_denied(self):
        x = firewall()
        x["change"]["after"]["ingress_security_rules"].append(ingress(5432, "0.0.0.0/0"))
        errors, _ = mod.validate_plan({"resource_changes": [x]}, {FIREWALL})
        self.assertIn("firewall:wrong-ingress-count", errors)

    def test_subnet_missing_security_list_denied(self):
        errors, _ = mod.validate_plan({"resource_changes": [change(SUBNET, {
            "prohibit_public_ip_on_vnic": False,
        })]}, {SUBNET})
        self.assertIn("subnet:security-list-not-set", errors)


if __name__ == "__main__":
    unittest.main(verbosity=2)
