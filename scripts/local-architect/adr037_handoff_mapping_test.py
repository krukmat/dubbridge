import unittest
import os
import sys
import importlib.util

# Setup path for the module under test and handoff_schema
repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, repo_root)

def load_module_from_path(module_name, file_path):
    spec = importlib.util.spec_from_file_location(module_name, file_path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module

# Load handoff_schema (local-agent/, not a valid Python package name)
handoff_schema = load_module_from_path(
    "handoff_schema", os.path.join(repo_root, "local-agent", "handoff_schema.py")
)

# Load the mapping module
mapping = load_module_from_path(
    "adr037_mapping",
    os.path.join(repo_root, "local-architect", "adr037_handoff_mapping.py"),
)

class TestADR037HandoffMapping(unittest.TestCase):

    def setUp(self):
        self.work_item_id = "WP-123"
        self.repo_revision = "abc12345"
        self.packet = {
            "objective": "Analyze the impact of changing auth flow",
            "current_behavior": "Uses legacy JWT",
            "required_behavior": "Use OAuth2",
            "constraints": ["Must not break existing sessions"],
            "questions": ["How to migrate active users?"]
        }

    def test_hp_1_mapping_success(self):
        """HP-1: Verify capsule and bundle creation for a successful analysis."""
        # 1. Test Capsule Mapping
        capsule = mapping.map_packet_to_capsule(self.packet, self.work_item_id, self.repo_revision)
        
        self.assertIsInstance(capsule, handoff_schema.Capsule)
        self.assertEqual(capsule.fields["work_item_id"], self.work_item_id)
        self.assertEqual(capsule.fields["repo_revision"], self.repo_revision)
        self.assertIn("ADR-037 advisory-only", capsule.fields["constraints"][0])
        # Check SHA256 length (64 hex chars)
        self.assertEqual(len(capsule.manifest_hash), 64)

        # 2. Test Bundle Mapping
        artifact = {
            "success": True,
            "status": "ok",
            "started_at": "2023-10-01T12:00:00Z",
            "finished_at": "2023-10-01T12:05:00Z",
            "model": {"tag": "gpt-4-architect"}
        }
        bundle = mapping.map_artifact_to_attempt_bundle(artifact, capsule.manifest_hash)

        # Check all required fields are present in the dict
        for field in handoff_schema.BUNDLE_REQUIRED_FIELDS:
            self.assertIn(field, bundle)
        
        self.assertEqual(bundle["outcome"], mapping.ADVISORY_ONLY_OUTCOME)
        self.assertEqual(bundle["model_tag"], "gpt-4-architect")

    def test_ec_1_validation_rejection(self):
        """EC-1: Verify that 'advisory-only' outcome is rejected by standard validator."""
        artifact = {
            "success": True,
            "status": "ok",
            "started_at": "2023-10-01T12:00:00Z",
            "finished_at": "2023-10-01T12:05:00Z",
            "model": {"tag": "gpt-4-architect"}
        }
        # Create a dummy capsule hash
        dummy_hash = "a" * 64
        bundle = mapping.map_artifact_to_attempt_bundle(artifact, dummy_hash)

        # The standard validator should raise ValidationError because 'advisory-only' is not in VALID_OUTCOMES
        with self.assertRaises(handoff_schema.ValidationError) as cm:
            handoff_schema.validate_attempt_bundle(bundle, {dummy_hash})
        
        self.assertEqual(cm.exception.field_name, "outcome")

    def test_failed_analysis_mapping(self):
        """Verify that a failed analysis maps to 'blocked'."""
        artifact = {
            "success": False,
            "status": "failed",
            "started_at": "2023-10-01T12:00:00Z",
            "finished_at": "2023-10-01T12:05:00Z",
            "model": {"tag": "gpt-4-architect"}
        }
        bundle = mapping.map_artifact_to_attempt_bundle(artifact, "a"*64)
        self.assertEqual(bundle["outcome"], "blocked")

if __name__ == "__main__":
    unittest.main()