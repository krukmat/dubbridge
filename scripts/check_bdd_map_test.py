#!/usr/bin/env python3
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-bdd-map.py")


def load_module(path: Path):
    spec = importlib.util.spec_from_file_location("bdd_gate", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class BddMapGateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name)
        (self.repo / "docs" / "bdd").mkdir(parents=True)
        (self.repo / "tests").mkdir()
        (self.repo / "tests" / "behavior.py").write_text("def test_behavior():\n    pass\n", encoding="utf-8")
        (self.repo / "tests" / "behavior2.py").write_text("def test_behavior_two():\n    pass\n", encoding="utf-8")
        (self.repo / "docs" / "bdd" / "strict.feature").write_text(
            "Feature: strict\n  Scenario: SC_HP1 works\n    Given a state\n    Then it works\n", encoding="utf-8"
        )
        self.module = load_module(SCRIPT)

    def tearDown(self):
        self.tmp.cleanup()

    def manifest(self, mappings=None):
        features = [{
            "file": "strict.feature",
            "mode": "strict",
            "mappings": mappings if mappings is not None else [{
                "scenario": "SC_HP1",
                "tasks": ["T1"],
                "evidence": ["tests/behavior.py::test_behavior"],
            }],
        }]
        (self.repo / "docs" / "bdd" / "behavior-map-v2.json").write_text(
            json.dumps({"schema": "behavior-map-v2", "features": features}), encoding="utf-8"
        )

    def test_valid_strict_mapping_passes(self):
        self.manifest()
        self.assertEqual(self.module.validate_repo(self.repo), [])

    def test_many_to_many_tasks_and_evidence_passes(self):
        self.manifest([{
            "scenario": "SC_HP1",
            "tasks": ["T1", "T2", "T3"],
            "evidence": [
                "tests/behavior.py::test_behavior",
                "tests/behavior2.py::test_behavior_two",
            ],
        }])
        self.assertEqual(self.module.validate_repo(self.repo), [])

    def test_missing_feature_inventory_fails(self):
        (self.repo / "docs" / "bdd" / "other.feature").write_text("Feature: other\n", encoding="utf-8")
        self.manifest()
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("manifest missing feature" in error for error in errors))

    def test_unknown_scenario_fails(self):
        self.manifest([{"scenario": "UNKNOWN", "tasks": ["T1"], "evidence": ["tests/behavior.py::test_behavior"]}])
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("unmapped scenario" in error for error in errors))
        self.assertTrue(any("unknown scenario" in error for error in errors))

    def test_feature_cannot_be_its_own_evidence(self):
        self.manifest([{"scenario": "SC_HP1", "tasks": ["T1"], "evidence": ["docs/bdd/strict.feature"]}])
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("specification cannot be executable evidence" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
