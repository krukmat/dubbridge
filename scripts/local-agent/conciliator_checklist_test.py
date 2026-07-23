#!/usr/bin/env python3
"""Unit tests for the cloud conciliator checklist."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("conciliator_checklist.py")
SPEC = importlib.util.spec_from_file_location("conciliator_checklist", SCRIPT)
assert SPEC is not None
mod = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["conciliator_checklist"] = mod
SPEC.loader.exec_module(mod)


class StubCapsule:
    def __init__(self, fields: dict):
        self.fields = fields


class StubBundle:
    def __init__(self, fields: dict):
        self.fields = fields


def _capsule(**extra):
    base = {
        "work_item_id": "T-1",
        "objective": "Do something",
        "non_goals": [],
        "questions": [],
        "current_behavior": "bad",
        "required_behavior": "good",
        "constraints": [],
        "allowed_paths": ["src/"],
        "acceptance_criteria": [],
        "repo_revision": "abc123",
    }
    base.update(extra)
    return StubCapsule(base)


def _bundle(**extra):
    base = {
        "capsule_hash": "a" * 64,
        "implementer_id": "qwen35",
        "model_tag": "qwen35-coder",
        "start_ts": "2026-07-01T00:00:00Z",
        "end_ts": "2026-07-01T00:01:00Z",
        "diff_ref": [],
        "test_results": {"passed": True},
        "review_verdict": "approved",
        "outcome": "success",
    }
    base.update(extra)
    return StubBundle(base)


class TestChecklistItems(unittest.TestCase):
    """Test each of the six checklist items."""

    # ── item 1: scope ──────────────────────────────────

    def test_scope_empty_bundles_passes(self) -> None:
        rpt = mod.build_report(_capsule(), [])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "PASS")

    def test_scope_no_paths_touched_passes(self) -> None:
        b = _bundle(diff_ref=[])
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "PASS")

    def test_scope_path_in_allowed_passes(self) -> None:
        b = _bundle(diff_ref=[{"tool": "write_file", "path": "src/foo.rs"}])
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "PASS")

    def test_scope_path_prefix_match_passes(self) -> None:
        b = _bundle(diff_ref=[{"tool": "write_file", "path": "src/sub/bar.rs"}])
        c = _capsule(allowed_paths=["src"])
        rpt = mod.build_report(c, [b])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "PASS")

    def test_scope_out_of_path_fails(self) -> None:
        c = _capsule(allowed_paths=["src/"])
        b = _bundle(diff_ref=[{"tool": "write_file", "path": "docs/readme.md"}])
        rpt = mod.build_report(c, [b])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "FAIL")

    def test_scope_empty_allowed_paths_fails(self) -> None:
        c = _capsule(allowed_paths=[])
        b = _bundle(diff_ref=[{"tool": "write_file", "path": "src/foo.rs"}])
        rpt = mod.build_report(c, [b])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "FAIL")

    def test_scope_normalize_leading_dot_slash(self) -> None:
        c = _capsule(allowed_paths=["./src/"])
        b = _bundle(diff_ref=[{"tool": "write_file", "path": "./src/foo.rs/"}])
        rpt = mod.build_report(c, [b])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "PASS")

    def test_scope_root_path_touched_fails_closed(self) -> None:
        c = _capsule(allowed_paths=["src/"])
        b = _bundle(diff_ref=[{"tool": "write_file", "path": "/"}])
        rpt = mod.build_report(c, [b])
        item = next(i for i in rpt["items"] if i["name"] == "scope")
        self.assertEqual(item["status"], "FAIL")

    # ── item 2: acceptance ──────────────────────────────────

    def test_acceptance_passed_true_passes(self) -> None:
        b = _bundle(test_results={"passed": True})
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "PASS")

    def test_acceptance_passed_false_fails(self) -> None:
        b = _bundle(test_results={"passed": False})
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "FAIL")

    def test_acceptance_status_ok_passes(self) -> None:
        b = _bundle(test_results={"status": "ok"})
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "PASS")

    def test_acceptance_status_non_ok_fails(self) -> None:
        b = _bundle(test_results={"status": "error"})
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "FAIL")

    def test_acceptance_non_dict_unknown(self) -> None:
        b = _bundle(test_results="some_string")
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "UNKNOWN")

    def test_acceptance_passed_int_unknown(self) -> None:
        b = _bundle(test_results={"passed": 1})
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "UNKNOWN")

    def test_acceptance_empty_dict_unknown(self) -> None:
        b = _bundle(test_results={})
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "UNKNOWN")

    def test_acceptance_missing_key_unknown(self) -> None:
        b = _bundle(test_results={"foo": "bar"})
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "acceptance")
        self.assertEqual(item["status"], "UNKNOWN")

    # ── item 3: review ──────────────────────────────────

    def test_review_approved_passes(self) -> None:
        b = _bundle(review_verdict="approved")
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "review")
        self.assertEqual(item["status"], "PASS")

    def test_review_pending_fails(self) -> None:
        b = _bundle(review_verdict="pending")
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "review")
        self.assertEqual(item["status"], "FAIL")

    def test_review_missing_fails(self) -> None:
        b = _bundle(review_verdict=None)
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "review")
        self.assertEqual(item["status"], "FAIL")

    def test_review_empty_string_fails(self) -> None:
        b = _bundle(review_verdict="")
        rpt = mod.build_report(_capsule(), [b])
        item = next(i for i in rpt["items"] if i["name"] == "review")
        self.assertEqual(item["status"], "FAIL")

    # ── item 4: budget ──────────────────────────────────

    def test_budget_moderate_two_passes(self) -> None:
        c = _capsule(rri=30)
        bundles = [_bundle() for _ in range(2)]
        rpt = mod.build_report(c, bundles)
        item = next(i for i in rpt["items"] if i["name"] == "budget")
        self.assertEqual(item["status"], "PASS")

    def test_budget_moderate_three_fails(self) -> None:
        c = _capsule(rri=30)
        bundles = [_bundle() for _ in range(3)]
        rpt = mod.build_report(c, bundles)
        item = next(i for i in rpt["items"] if i["name"] == "budget")
        self.assertEqual(item["status"], "FAIL")

    def test_budget_medhigh_one_passes(self) -> None:
        c = _capsule(rri=50)
        bundles = [_bundle() for _ in range(1)]
        rpt = mod.build_report(c, bundles)
        item = next(i for i in rpt["items"] if i["name"] == "budget")
        self.assertEqual(item["status"], "PASS")

    def test_budget_medhigh_two_fails(self) -> None:
        c = _capsule(rri=50)
        bundles = [_bundle() for _ in range(2)]
        rpt = mod.build_report(c, bundles)
        item = next(i for i in rpt["items"] if i["name"] == "budget")
        self.assertEqual(item["status"], "FAIL")

    def test_budget_no_band_unknown(self) -> None:
        c = _capsule(rri=10)
        bundles = [_bundle()]
        rpt = mod.build_report(c, bundles)
        item = next(i for i in rpt["items"] if i["name"] == "budget")
        self.assertEqual(item["status"], "UNKNOWN")

    def test_budget_caller_band_provided_passes(self) -> None:
        c = _capsule(rri=10)
        bundles = [_bundle() for _ in range(2)]
        rpt = mod.build_report(c, bundles, band="Moderate")
        item = next(i for i in rpt["items"] if i["name"] == "budget")
        self.assertEqual(item["status"], "PASS")

    # ── item 5: reflection ──────────────────────────────────

    def test_reflection_rri_30_log_present_passes(self) -> None:
        c = _capsule(rri=30)
        rpt = mod.build_report(c, [], reflection_log_present=True)
        item = next(i for i in rpt["items"] if i["name"] == "reflection")
        self.assertEqual(item["status"], "PASS")

    def test_reflection_rri_30_no_log_fails(self) -> None:
        # EC-1: RRI known and >= 26, reflection_log_present=None => FAIL
        c = _capsule(rri=30)
        rpt = mod.build_report(c, [], reflection_log_present=None)
        item = next(i for i in rpt["items"] if i["name"] == "reflection")
        self.assertEqual(item["status"], "FAIL")

    def test_reflection_rri_30_false_fails(self) -> None:
        c = _capsule(rri=30)
        rpt = mod.build_report(c, [], reflection_log_present=False)
        item = next(i for i in rpt["items"] if i["name"] == "reflection")
        self.assertEqual(item["status"], "FAIL")

    def test_reflection_rri_low_unknown(self) -> None:
        c = _capsule(rri=10)
        rpt = mod.build_report(c, [])
        item = next(i for i in rpt["items"] if i["name"] == "reflection")
        self.assertEqual(item["status"], "PASS")

    def test_reflection_rri_unknown_via_caller_passes(self) -> None:
        c = _capsule(rri=10)
        rpt = mod.build_report(c, [], rri=30, reflection_log_present=True)
        item = next(i for i in rpt["items"] if i["name"] == "reflection")
        self.assertEqual(item["status"], "PASS")

    def test_reflection_rri_unknown_via_caller_fails(self) -> None:
        # Capsule has no rri but caller provides rri>=26
        c = _capsule()
        rpt = mod.build_report(c, [], rri=30)
        item = next(i for i in rpt["items"] if i["name"] == "reflection")
        self.assertEqual(item["status"], "FAIL")

    # ── item 6: status_sync ──────────────────────────────────

    def test_status_sync_all_synced_passes(self) -> None:
        c = _capsule()
        rpt = mod.build_report(
            c,
            [],
            status_artifacts=[("a.json", True), ("b.json", True)],
        )
        item = next(i for i in rpt["items"] if i["name"] == "status_sync")
        self.assertEqual(item["status"], "PASS")

    def test_status_sync_unsynced_fails(self) -> None:
        c = _capsule()
        rpt = mod.build_report(
            c,
            [],
            status_artifacts=[("a.json", True), ("b.json", False)],
        )
        item = next(i for i in rpt["items"] if i["name"] == "status_sync")
        self.assertEqual(item["status"], "FAIL")

    def test_status_sync_none_unknown(self) -> None:
        c = _capsule()
        rpt = mod.build_report(c, [])
        item = next(i for i in rpt["items"] if i["name"] == "status_sync")
        self.assertEqual(item["status"], "UNKNOWN")


class TestOverallAggregation(unittest.TestCase):
    """Test the overall PASS/FAIL/UNKNOWN aggregation logic."""

    def test_hp1_all_pass_overall_pass(self) -> None:
        c = _capsule(rri=30)
        bundles = [_bundle(test_results={"passed": True}, review_verdict="approved")]
        rpt = mod.build_report(
            c,
            bundles,
            band="Moderate",
            reflection_log_present=True,
            status_artifacts=[("a.json", True)],
        )
        self.assertEqual(rpt["overall"], "PASS")
        for item in rpt["items"]:
            self.assertEqual(item["status"], "PASS")

    def test_ec2_scope_fail_overall_fail(self) -> None:
        c = _capsule(allowed_paths=["src/"], rri=30)
        b = _bundle(
            diff_ref=[{"tool": "write_file", "path": "docs/readme.md"}],
            test_results={"passed": True},
            review_verdict="approved",
        )
        rpt = mod.build_report(c, [b], band="Moderate")
        self.assertEqual(rpt["overall"], "FAIL")

    def test_ec3_fail_and_unknown_overall_fail(self) -> None:
        c = _capsule(allowed_paths=["src/"], rri=10)
        b = _bundle(
            diff_ref=[{"tool": "write_file", "path": "docs/readme.md"}],
            test_results={"passed": True},
            review_verdict="approved",
        )
        rpt = mod.build_report(c, [b])
        self.assertEqual(rpt["overall"], "FAIL")
        items_by_name = {i["name"]: i for i in rpt["items"]}
        self.assertEqual(items_by_name["scope"]["status"], "FAIL")
        self.assertEqual(items_by_name["budget"]["status"], "UNKNOWN")

    def test_ec4_all_unknown_overall_unknown(self) -> None:
        c = _capsule()
        b = _bundle(test_results={"passed": True}, review_verdict="approved")
        rpt = mod.build_report(c, [b])
        items_by_name = {i["name"]: i for i in rpt["items"]}
        for name in ("budget", "reflection", "status_sync"):
            self.assertEqual(items_by_name[name]["status"], "UNKNOWN")
        for name in ("scope", "acceptance", "review"):
            self.assertEqual(items_by_name[name]["status"], "PASS")
        self.assertEqual(rpt["overall"], "UNKNOWN")

    def test_fail_dominates_unknown(self) -> None:
        c = _capsule(allowed_paths=[], rri=10)
        b = _bundle(
            diff_ref=[{"tool": "write_file", "path": "x"}],
            test_results="bad_type",
            review_verdict="approved",
        )
        rpt = mod.build_report(c, [b])
        items_by_name = {i["name"]: i for i in rpt["items"]}
        self.assertEqual(items_by_name["scope"]["status"], "FAIL")
        self.assertEqual(items_by_name["acceptance"]["status"], "UNKNOWN")
        self.assertEqual(rpt["overall"], "FAIL")


class TestBudgetPrecedence(unittest.TestCase):
    """Test item-4 band precedence rules."""

    def test_capsule_band_takes_precedence(self) -> None:
        # Capsule says Med-high (budget=1); caller says Moderate (budget=2).
        # 2 bundles => with capsule budget it FAILs.
        c = _capsule(rri=50)
        bundles = [_bundle() for _ in range(2)]
        rpt = mod.build_report(c, bundles, band="Moderate")
        item = next(i for i in rpt["items"] if i["name"] == "budget")
        self.assertEqual(item["status"], "FAIL")


if __name__ == "__main__":
    unittest.main()