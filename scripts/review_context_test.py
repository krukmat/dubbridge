#!/usr/bin/env python3
"""Unit tests for review_context.py (M3 reviewer intelligence)."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import review_context as rc


DIFF = "\n".join(
    [
        "diff --git a/src/lib.py b/src/lib.py",
        "--- a/src/lib.py",
        "+++ b/src/lib.py",
        "@@ -1,2 +1,3 @@ def changed():",
        " def changed():",
        "-    return 1",
        "+    return helper()",
    ]
)


class FakeAdapter:
    def __init__(self, candidates=None, *, available=True, coverage="verified"):
        self._candidates = list(candidates or [])
        self._available = available
        self._coverage = coverage
        self.discover_calls = []
        self.coverage_calls = []

    def available(self):
        return self._available

    def discover(self, text, worktree, *, force_refresh=False):
        self.discover_calls.append((text, worktree, force_refresh))
        return {
            "project": "fixture",
            "anchors": {},
            "candidates": list(self._candidates),
        }

    def coverage(self, project, paths):
        self.coverage_calls.append((project, list(paths)))
        return {"status": self._coverage, "raw": {}}


class ExtractionTests(unittest.TestCase):
    def test_changed_paths_are_stable_and_repo_relative(self):
        diff = (
            DIFF
            + "\n"
            + "diff --git a/new.py b/new.py\n--- /dev/null\n+++ b/new.py\n+def new(): pass\n"
        )
        self.assertEqual(rc.extract_changed_paths(diff), ["src/lib.py", "new.py"])

    def test_changed_symbols_include_hunk_and_definition_anchors(self):
        diff = DIFF + "\n+class AddedThing:\n+    pass\n+fn rust_name() {}\n"
        symbols = rc.extract_changed_symbols(diff)
        self.assertIn("changed", symbols)
        self.assertIn("AddedThing", symbols)
        self.assertIn("rust_name", symbols)

    def test_extract_task_section_returns_matching_heading_only(self):
        text = "# Tasks\n\n## M3-T1 — First\nA\n\n## M3-T2 — Second\nB\n"
        self.assertEqual(
            rc.extract_task_section(text, "M3-T1"),
            "## M3-T1 — First\nA",
        )


class PacketBudgetTests(unittest.TestCase):
    def test_mandatory_diff_and_acceptance_are_never_truncated_for_impact(self):
        packet, metadata = rc.build_review_context(
            diff_text=DIFF,
            acceptance_text="HP-1: preserve this exact acceptance sentence",
            task_id="M3-T1",
            adapter=FakeAdapter(),
            authorizer=lambda _path: None,
            num_ctx=128,
            num_predict=64,
            prompt_reserve_tokens=64,
            max_impact_tokens=6000,
        )
        self.assertIn(DIFF, packet)
        self.assertIn("HP-1: preserve this exact acceptance sentence", packet)
        self.assertEqual(metadata["budget"]["impact_budget_tokens"], 0)
        self.assertEqual(metadata["status"], "budget_exhausted")

    def test_optional_entries_are_fit_whole_inside_impact_budget(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tests").mkdir()
            (root / "tests" / "test_lib.py").write_text("x = 'large-value'\n" * 100, encoding="utf-8")
            candidate = rc.GraphCandidate(
                path="tests/test_lib.py", label="Test", relation="TESTS"
            )
            packet, metadata = rc.build_review_context(
                diff_text=DIFF,
                worktree=tmp,
                adapter=FakeAdapter([candidate]),
                authorizer=lambda _path: None,
                num_ctx=32768,
                num_predict=1024,
                max_impact_tokens=1,
            )
            self.assertNotIn("large-value", packet)
            self.assertEqual(metadata["selected"], [])
            self.assertEqual(metadata["budget"]["selected_impact_tokens"], 0)


class ImpactRetrievalTests(unittest.TestCase):
    def _fixture(self):
        tmp = tempfile.TemporaryDirectory()
        root = Path(tmp.name)
        (root / "src").mkdir()
        (root / "tests").mkdir()
        (root / "src" / "lib.py").write_text(
            "def changed():\n    return helper()\n\ndef helper():\n    return 2\n",
            encoding="utf-8",
        )
        (root / "tests" / "test_lib.py").write_text(
            "def test_changed():\n    assert changed() == 2\n", encoding="utf-8"
        )
        (root / "src" / "caller.py").write_text(
            "def caller():\n    return changed()\n", encoding="utf-8"
        )
        (root / "src" / "dep.py").write_text(
            "def helper():\n    return 2\n", encoding="utf-8"
        )
        return tmp, root

    def test_depth_one_candidates_are_ranked_for_review_value(self):
        tmp, _root = self._fixture()
        self.addCleanup(tmp.cleanup)
        candidates = [
            rc.GraphCandidate(
                path="src/dep.py", symbol="helper", relation="CALLS", start_line=1, end_line=2
            ),
            rc.GraphCandidate(
                path="src/caller.py", symbol="caller", relation="CALLER", start_line=1, end_line=2
            ),
            rc.GraphCandidate(
                path="tests/test_lib.py", symbol="test_changed", label="Test", relation="TESTS", start_line=1, end_line=2
            ),
            rc.GraphCandidate(
                path="src/lib.py", symbol="changed", label="Function", start_line=1, end_line=2
            ),
        ]
        packet, metadata = rc.build_review_context(
            diff_text=DIFF,
            acceptance_text="tests must preserve changed() behavior",
            worktree=tmp.name,
            adapter=FakeAdapter(candidates),
            authorizer=lambda _path: None,
            num_ctx=32768,
            num_predict=1024,
            max_impact_tokens=12000,
        )
        selected = metadata["selected"]
        self.assertEqual(
            [(item["path"], item["reason"]) for item in selected],
            [
                ("src/lib.py", "changed_symbol_context"),
                ("tests/test_lib.py", "related_test"),
                ("src/caller.py", "direct_caller"),
                ("src/dep.py", "direct_dependency"),
            ],
        )
        self.assertIn("assert changed() == 2", packet)
        self.assertIn("return changed()", packet)
        self.assertEqual(metadata["status"], "enriched")

    def test_current_worktree_source_wins_over_graph_discovery_metadata(self):
        tmp, root = self._fixture()
        self.addCleanup(tmp.cleanup)
        (root / "tests" / "test_lib.py").write_text(
            "CURRENT_WORKTREE_VALUE = 42\n", encoding="utf-8"
        )
        candidate = rc.GraphCandidate(
            path="tests/test_lib.py", symbol="test_changed", label="Test", relation="TESTS"
        )
        packet, _metadata = rc.build_review_context(
            diff_text=DIFF,
            worktree=tmp.name,
            adapter=FakeAdapter([candidate]),
            authorizer=lambda _path: None,
            num_ctx=32768,
            num_predict=1024,
        )
        self.assertIn("CURRENT_WORKTREE_VALUE = 42", packet)

    def test_unauthorized_related_source_becomes_scope_gap_without_body(self):
        tmp, root = self._fixture()
        self.addCleanup(tmp.cleanup)
        (root / "src" / "caller.py").write_text("SECRET_CALLER_BODY = True\n", encoding="utf-8")
        candidate = rc.GraphCandidate(
            path="src/caller.py", symbol="caller", relation="CALLER"
        )

        def authorize(path):
            raise RuntimeError(f"path outside allowed_paths: {path}")

        packet, metadata = rc.build_review_context(
            diff_text=DIFF,
            worktree=tmp.name,
            adapter=FakeAdapter([candidate]),
            authorizer=authorize,
            num_ctx=32768,
            num_predict=1024,
        )
        self.assertNotIn("SECRET_CALLER_BODY", packet)
        self.assertEqual(metadata["selected"], [])
        self.assertEqual(metadata["scope_gaps"][0]["path"], "src/caller.py")
        self.assertEqual(
            metadata["scope_gaps"][0]["reason"], "outside_review_allowed_paths"
        )
        self.assertNotIn("SECRET_CALLER_BODY", json.dumps(metadata))

    def test_unavailable_ckg_falls_back_to_mandatory_review_packet(self):
        packet, metadata = rc.build_review_context(
            diff_text=DIFF,
            acceptance_text="acceptance remains visible",
            adapter=FakeAdapter(available=False),
            authorizer=lambda _path: None,
            num_ctx=32768,
            num_predict=1024,
        )
        self.assertEqual(metadata["status"], "fallback")
        self.assertIn("CKG backend unavailable", metadata["fallback_reason"])
        self.assertIn(DIFF, packet)
        self.assertIn("acceptance remains visible", packet)

    def test_partial_coverage_disables_optional_enrichment(self):
        packet, metadata = rc.build_review_context(
            diff_text=DIFF,
            adapter=FakeAdapter(coverage="partial"),
            authorizer=lambda _path: None,
            num_ctx=32768,
            num_predict=1024,
        )
        self.assertEqual(metadata["status"], "fallback")
        self.assertEqual(metadata["coverage"], "partial")
        self.assertEqual(metadata["selected"], [])
        self.assertIn(DIFF, packet)


class CliTests(unittest.TestCase):
    def test_no_ckg_cli_is_deterministic_and_needs_no_backend(self):
        script = Path(__file__).with_name("review_context.py")
        result = subprocess.run(
            [
                sys.executable,
                str(script),
                "-",
                "--no-ckg",
                "--task-id",
                "M3-CLI",
                "--acceptance-text",
                "must preserve behavior",
            ],
            input=DIFF,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(DIFF, result.stdout)
        self.assertIn("must preserve behavior", result.stdout)
        metadata_line = result.stdout.split("## Local CKG impact metadata\n", 1)[1].splitlines()[0]
        metadata = json.loads(metadata_line)
        self.assertEqual(metadata["status"], "disabled")
        self.assertTrue(metadata["local_only"])


if __name__ == "__main__":
    unittest.main()
