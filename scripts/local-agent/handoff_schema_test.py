#!/usr/bin/env python3
"""Tests for handoff_schema.py — HP-1, HP-2, EC-1, EC-2 per docs/tasks/local-first-cloud-local-handoff.md T1."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import handoff_schema


REALISTIC_CAPSULE = {
    "work_item_id": "T99",
    "objective": "Fix the failing test_foo unit test in src/lib.rs.",
    "non_goals": ["Do not refactor unrelated modules."],
    "questions": [],
    "current_behavior": "test_foo fails with assertion mismatch.",
    "required_behavior": "test_foo passes.",
    "constraints": ["No new dependencies."],
    "allowed_paths": ["src/lib.rs", "src/lib_test.rs"],
    "acceptance_criteria": ["HP-1: test_foo passes.", "EC-1: negative input rejected."],
    "repo_revision": "abc1234",
}

REALISTIC_BUNDLE = {
    "implementer_id": "qwen35",
    "model_tag": "qwen3.6:35b-a3b",
    "start_ts": "2026-07-23T00:00:00Z",
    "end_ts": "2026-07-23T00:05:00Z",
    "diff_ref": "diff --git a/src/lib.rs b/src/lib.rs\n",
    "test_results": {"passed": True, "output": "1 passed"},
    "review_verdict": "approved",
    "outcome": "success",
}


class HP1CapsuleValidatesAndHashesDeterministically(unittest.TestCase):
    def test_well_formed_capsule_validates(self):
        capsule = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        self.assertEqual(capsule["work_item_id"], "T99")

    def test_manifest_hash_is_deterministic(self):
        capsule_a = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        capsule_b = handoff_schema.validate_capsule(dict(REALISTIC_CAPSULE))
        self.assertEqual(capsule_a.manifest_hash, capsule_b.manifest_hash)
        self.assertEqual(len(capsule_a.manifest_hash), 64)

    def test_manifest_hash_changes_with_content(self):
        capsule_a = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        mutated = dict(REALISTIC_CAPSULE)
        mutated["objective"] = "Something else entirely."
        capsule_b = handoff_schema.validate_capsule(mutated)
        self.assertNotEqual(capsule_a.manifest_hash, capsule_b.manifest_hash)


class HP2AttemptBundleValidatesAgainstKnownCapsule(unittest.TestCase):
    def test_bundle_referencing_valid_capsule_hash_validates(self):
        capsule = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        bundle_data = dict(REALISTIC_BUNDLE)
        bundle_data["capsule_hash"] = capsule.manifest_hash

        bundle = handoff_schema.validate_attempt_bundle(
            bundle_data, known_capsule_hashes={capsule.manifest_hash}
        )
        self.assertEqual(bundle["outcome"], "success")
        self.assertEqual(bundle["capsule_hash"], capsule.manifest_hash)


class EC1CapsuleMissingFieldFailsWithFieldName(unittest.TestCase):
    def test_missing_required_field_raises_with_field_name(self):
        incomplete = dict(REALISTIC_CAPSULE)
        del incomplete["allowed_paths"]

        with self.assertRaises(handoff_schema.ValidationError) as ctx:
            handoff_schema.validate_capsule(incomplete)

        self.assertEqual(ctx.exception.field_name, "allowed_paths")

    def test_non_dict_capsule_raises(self):
        with self.assertRaises(handoff_schema.ValidationError):
            handoff_schema.validate_capsule(["not", "a", "dict"])


class EC2UnknownCapsuleHashRejected(unittest.TestCase):
    def test_bundle_with_unknown_capsule_hash_is_rejected(self):
        capsule = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        bundle_data = dict(REALISTIC_BUNDLE)
        bundle_data["capsule_hash"] = "0" * 64

        with self.assertRaises(handoff_schema.ValidationError) as ctx:
            handoff_schema.validate_attempt_bundle(
                bundle_data, known_capsule_hashes={capsule.manifest_hash}
            )

        self.assertEqual(ctx.exception.field_name, "capsule_hash")

    def test_bundle_missing_required_field_raises_with_field_name(self):
        capsule = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        incomplete = dict(REALISTIC_BUNDLE)
        incomplete["capsule_hash"] = capsule.manifest_hash
        del incomplete["outcome"]

        with self.assertRaises(handoff_schema.ValidationError) as ctx:
            handoff_schema.validate_attempt_bundle(
                incomplete, known_capsule_hashes={capsule.manifest_hash}
            )

        self.assertEqual(ctx.exception.field_name, "outcome")

    def test_bundle_invalid_outcome_enum_rejected(self):
        capsule = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        bad_outcome = dict(REALISTIC_BUNDLE)
        bad_outcome["capsule_hash"] = capsule.manifest_hash
        bad_outcome["outcome"] = "not-a-real-outcome"

        with self.assertRaises(handoff_schema.ValidationError) as ctx:
            handoff_schema.validate_attempt_bundle(
                bad_outcome, known_capsule_hashes={capsule.manifest_hash}
            )

        self.assertEqual(ctx.exception.field_name, "outcome")


class UsageExample(unittest.TestCase):
    def test_one_capsule_and_one_bundle_constructed_and_validated(self):
        capsule = handoff_schema.validate_capsule(REALISTIC_CAPSULE)
        bundle_data = dict(REALISTIC_BUNDLE)
        bundle_data["capsule_hash"] = capsule.manifest_hash
        bundle = handoff_schema.validate_attempt_bundle(
            bundle_data, known_capsule_hashes={capsule.manifest_hash}
        )
        self.assertEqual(bundle["capsule_hash"], capsule.manifest_hash)


if __name__ == "__main__":
    unittest.main()
