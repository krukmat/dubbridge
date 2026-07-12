#!/usr/bin/env python3
"""Structural security-contract tests for the temporary object_store RC pin."""

from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[1]
APPROVED_REV = "c7316d29face118e7409eead0cda098f38589428"
UPSTREAM = "https://github.com/apache/arrow-rs-object-store.git"


class ObjectStoreSecurityContract(unittest.TestCase):
    def test_manifest_and_lock_pin_the_verified_upstream_revision(self):
        manifest = (REPO_ROOT / "Cargo.toml").read_text(encoding="utf-8")
        lock = (REPO_ROOT / "Cargo.lock").read_text(encoding="utf-8")

        self.assertIn(f'git = "{UPSTREAM}"', manifest)
        self.assertIn(f'rev = "{APPROVED_REV}"', manifest)
        self.assertIn(f"git+{UPSTREAM}?rev={APPROVED_REV}#{APPROVED_REV}", lock)

    def test_lock_uses_patched_quick_xml(self):
        lock = (REPO_ROOT / "Cargo.lock").read_text(encoding="utf-8")
        package = 'name = "quick-xml"\nversion = "0.41.0"'

        self.assertIn(package, lock)
        self.assertNotIn('name = "quick-xml"\nversion = "0.37.5"', lock)

    def test_source_policy_requires_rev_and_keeps_advisory_ignores_empty(self):
        policy = (REPO_ROOT / "deny.toml").read_text(encoding="utf-8")

        self.assertIn('ignore = []', policy)
        self.assertIn('unknown-git = "deny"', policy)
        self.assertIn('required-git-spec = "rev"', policy)
        self.assertIn(f'  "{UPSTREAM}",', policy)


if __name__ == "__main__":
    unittest.main()
