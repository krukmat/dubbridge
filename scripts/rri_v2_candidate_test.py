#!/usr/bin/env python3
"""Executable contract tests for the inactive RRI v2 candidate calculator."""

from __future__ import annotations

import copy
import json
import subprocess
import sys
import unittest
from pathlib import Path

import rri_v2_candidate as candidate


ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "docs" / "fixtures"
SCRIPT = Path(__file__).resolve().with_name("rri_v2_candidate.py")


def fixture(name: str) -> dict:
    return json.loads((FIXTURES / name).read_text(encoding="utf-8"))


class SchemaContractTest(unittest.TestCase):
    def test_schema_is_a_closed_draft_2020_12_candidate_contract(self):
        schema = candidate.validate_schema_document()
        self.assertEqual(schema["$schema"], "https://json-schema.org/draft/2020-12/schema")
        self.assertEqual(schema["properties"]["schema_version"]["const"], candidate.SCHEMA_VERSION)
        self.assertFalse(schema["additionalProperties"])

    def test_unknown_top_level_field_is_rejected(self):
        assessment = fixture("rri-v2-candidate-valid.json")
        assessment["pretend_coefficient"] = 0.5
        with self.assertRaisesRegex(candidate.ValidationError, "unknown key"):
            candidate.calculate(assessment)


class PointFormulaScenariosTest(unittest.TestCase):
    def _with_profile(self, profile: tuple[int, int, int, int]) -> dict:
        assessment = fixture("rri-v2-candidate-valid.json")
        for axis, level in zip(candidate.AXES, profile):
            assessment["axes"][axis]["level"] = level
        return assessment

    def test_documented_point_scenarios(self):
        cases = (
            ((0, 0, 0, 0), 0, [0, 0, 0, 0]),
            ((1, 1, 0, 1), 25, [0, 0, 0, 3]),
            ((2, 2, 1, 2), 50, [0, 0, 3, 4]),
            ((2, 2, 3, 3), 75, [0, 2, 4, 4]),
            ((3, 4, 4, 3), 100, [2, 4, 4, 4]),
            ((4, 1, 0, 1), 100, [1, 1, 1, 3]),
        )
        for profile, ici, breadth in cases:
            with self.subTest(profile=profile):
                result = candidate.calculate(self._with_profile(profile))
                technical = result["technical"]
                self.assertEqual(technical["profile"], dict(zip(candidate.AXES, profile)))
                self.assertEqual(technical["bottleneck"], ici // 25)
                self.assertEqual(technical["ici"], ici)
                self.assertEqual(technical["ici_range"], [ici, ici])
                self.assertEqual(technical["breadth"], breadth)
                self.assertEqual(technical["breadth_range"], {"lower": breadth, "upper": breadth})
                self.assertEqual(technical["assessment_status"], "fully_assessed")


class UncertaintyTest(unittest.TestCase):
    def test_unknown_axis_encloses_without_a_midpoint(self):
        result = candidate.calculate(fixture("rri-v2-candidate-unknown.json"))
        technical = result["technical"]
        self.assertEqual(technical["profile"]["Q"], [0, 4])
        self.assertEqual(technical["bottleneck"], None)
        self.assertEqual(technical["ici"], None)
        self.assertEqual(technical["bottleneck_range"], [1, 4])
        self.assertEqual(technical["ici_range"], [25, 100])
        self.assertEqual(technical["breadth_range"], {"lower": [0, 0, 0, 3], "upper": [1, 1, 1, 4]})
        self.assertEqual(technical["assessment_status"], "unresolved")

    def test_known_dominant_axis_does_not_make_an_unresolved_profile_complete(self):
        assessment = fixture("rri-v2-candidate-valid.json")
        assessment["axes"]["L"]["level"] = 4
        assessment["axes"]["I"].pop("level")
        assessment["axes"]["I"].update({"status": "unknown", "range": [0, 4]})
        assessment["axes"]["Q"]["level"] = 0
        assessment["axes"]["V"]["level"] = 1
        result = candidate.calculate(assessment)["technical"]
        self.assertEqual(result["ici_range"], [100, 100])
        self.assertIsNone(result["ici"])
        self.assertEqual(result["assessment_status"], "unresolved")


class InvalidInputTest(unittest.TestCase):
    def test_documented_invalid_levels_are_rejected(self):
        for bad in (-1, 5, 0.5, True, float("nan")):
            with self.subTest(bad=repr(bad)):
                assessment = fixture("rri-v2-candidate-valid.json")
                assessment["axes"]["L"]["level"] = bad
                with self.assertRaises(candidate.ValidationError):
                    candidate.calculate(assessment)

    def test_missing_axis_reversed_range_unknown_version_and_status_mismatch_fail_closed(self):
        cases = []
        missing = fixture("rri-v2-candidate-invalid.json")
        cases.append(missing)
        reversed_range = fixture("rri-v2-candidate-valid.json")
        reversed_range["axes"]["Q"].pop("level")
        reversed_range["axes"]["Q"]["range"] = [3, 1]
        cases.append(reversed_range)
        wrong_version = fixture("rri-v2-candidate-valid.json")
        wrong_version["schema_version"] = "rri-v2-design-0.1"
        cases.append(wrong_version)
        mismatch = fixture("rri-v2-candidate-valid.json")
        mismatch["axes"]["V"]["status"] = "unknown"
        cases.append(mismatch)
        for assessment in cases:
            with self.subTest(assessment=assessment["task"]["task_id"]):
                with self.assertRaises(candidate.ValidationError):
                    candidate.calculate(assessment)

    def test_unknown_size_cannot_be_silently_zero(self):
        assessment = fixture("rri-v2-candidate-unknown.json")
        assessment["size"]["value"] = 0
        with self.assertRaisesRegex(candidate.ValidationError, "must be null"):
            candidate.calculate(assessment)


class InapplicabilityTest(unittest.TestCase):
    def test_planning_record_returns_null_technical_result(self):
        result = candidate.calculate(fixture("rri-v2-candidate-inapplicable.json"))
        self.assertIsNone(result["technical"])
        self.assertIn("outside the implementation rubric", result["technical_reason"])

    def test_inapplicability_dispatches_before_axis_shape_validation(self):
        assessment = fixture("rri-v2-candidate-inapplicable.json")
        assessment["axes"] = "not an axes object"
        with self.assertRaisesRegex(candidate.ValidationError, "must be omitted"):
            candidate.calculate(assessment)


class CandidateBoundaryTest(unittest.TestCase):
    def test_every_result_retains_prediction_and_authorization_boundary(self):
        for name in (
            "rri-v2-candidate-valid.json",
            "rri-v2-candidate-unknown.json",
            "rri-v2-candidate-inapplicable.json",
        ):
            with self.subTest(name=name):
                result = candidate.calculate(fixture(name))
                self.assertEqual(result["mode"], "candidate_only")
                self.assertEqual(result["authorization"], "governed_by_existing_RRI_and_HITL")
                self.assertEqual(
                    result["effort"],
                    {"prediction_status": "not_calibrated", "p50": None, "p90": None},
                )


class CliTest(unittest.TestCase):
    def _run(self, name: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--input", str(FIXTURES / name)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_valid_cli_emits_only_a_candidate_result(self):
        completed = self._run("rri-v2-candidate-valid.json")
        self.assertEqual(completed.returncode, 0, completed.stderr)
        result = json.loads(completed.stdout)
        self.assertEqual(result["technical"]["ici"], 50)
        self.assertEqual(result["effort"]["prediction_status"], "not_calibrated")

    def test_invalid_cli_emits_no_success_json(self):
        completed = self._run("rri-v2-candidate-invalid.json")
        self.assertEqual(completed.returncode, 2)
        self.assertEqual(completed.stdout, "")
        error = json.loads(completed.stderr)
        self.assertEqual(error["error"]["code"], "validation_error")


if __name__ == "__main__":
    unittest.main(verbosity=2)
