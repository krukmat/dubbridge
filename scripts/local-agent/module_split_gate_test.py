#!/usr/bin/env python3
"""Unit tests for scripts/local-agent/module_split_gate.py (ADR-040)."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).with_name("module_split_gate.py")
_SPEC = importlib.util.spec_from_file_location("module_split_gate", _SCRIPT)
if _SPEC is None or _SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_SCRIPT}")
_MOD = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _MOD
_SPEC.loader.exec_module(_MOD)


def _capsule(allowed_paths, cc_by_path):
    return {"allowed_paths": allowed_paths, "cc_by_path": cc_by_path}


class EvaluateSplitHappyPathTest(unittest.TestCase):
    def test_hp1_heterogeneous_cc_clean_partition_splits(self) -> None:
        # crates/qc (D/P/K=2, below exclusion floor) simple module + complex module.
        capsule = _capsule(
            ["crates/qc/src/simple.rs", "crates/qc/src/complex.rs"],
            {"crates/qc/src/simple.rs": 3, "crates/qc/src/complex.rs": 15},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_SPLIT)
        self.assertEqual(decision.local_paths, ("crates/qc/src/simple.rs",))
        self.assertEqual(decision.cloud_paths, ("crates/qc/src/complex.rs",))
        self.assertEqual(decision.local_repair_budget, 2)
        self.assertEqual(decision.cloud_repair_budget, 1)

    def test_hp1b_split_with_three_modules_partitions_completely(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs", "crates/qc/c.rs"],
            {"crates/qc/a.rs": 2, "crates/qc/b.rs": 4, "crates/qc/c.rs": 25},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_SPLIT)
        self.assertEqual(set(decision.local_paths), {"crates/qc/a.rs", "crates/qc/b.rs"})
        self.assertEqual(decision.cloud_paths, ("crates/qc/c.rs",))

    def test_hp2_uniform_low_cc_does_not_split(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs"],
            {"crates/qc/a.rs": 2, "crates/qc/b.rs": 3},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_NO_SPLIT)
        self.assertIn("heterogeneity trigger not met", decision.reason)

    def test_hp2b_uniform_high_cc_does_not_split(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs"],
            {"crates/qc/a.rs": 15, "crates/qc/b.rs": 25},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_NO_SPLIT)
        self.assertIn("heterogeneity trigger not met", decision.reason)


class EvaluateSplitEdgeCaseTest(unittest.TestCase):
    def test_ec1_low_cc_hard_excluded_module_forced_to_cloud(self) -> None:
        # crates/auth (D=4,P=4,K=4) is low-CC but must never reach local.
        # crates/qc/simple.rs is genuinely local-eligible, keeping the split valid.
        capsule = _capsule(
            ["crates/auth/src/token.rs", "crates/qc/src/complex.rs", "crates/qc/src/simple.rs"],
            {
                "crates/auth/src/token.rs": 2,
                "crates/qc/src/complex.rs": 15,
                "crates/qc/src/simple.rs": 3,
            },
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_SPLIT)
        self.assertIn("crates/auth/src/token.rs", decision.cloud_paths)
        self.assertNotIn("crates/auth/src/token.rs", decision.local_paths)
        auth_module = next(m for m in decision.modules if m.path == "crates/auth/src/token.rs")
        self.assertTrue(auth_module.hard_excluded)
        self.assertEqual(auth_module.tramo, "cloud")

    def test_ec1b_hard_excluded_module_cannot_reach_local_via_any_c_score(self) -> None:
        # Sweep every CC value that would normally map to the local tier and
        # confirm the exclusion check always wins over the CC-based route.
        # A genuinely local-eligible third module keeps the split itself
        # valid (non-empty local tramo) across the whole sweep.
        for raw_cc in (1, 3, 5, 8, 10):
            with self.subTest(raw_cc=raw_cc):
                capsule = _capsule(
                    [
                        "infra/migrations/0001_init.rs",
                        "crates/qc/src/complex.rs",
                        "crates/qc/src/simple.rs",
                    ],
                    {
                        "infra/migrations/0001_init.rs": raw_cc,
                        "crates/qc/src/complex.rs": 20,
                        "crates/qc/src/simple.rs": 3,
                    },
                )
                decision = _MOD.evaluate_split(capsule)
                self.assertEqual(decision.decision, _MOD.DECISION_SPLIT)
                self.assertNotIn("infra/migrations/0001_init.rs", decision.local_paths)
                self.assertIn("infra/migrations/0001_init.rs", decision.cloud_paths)

    def test_ec1c_all_modules_hard_excluded_collapses_to_no_split(self) -> None:
        # Both modules land in the cloud tramo (one via CC, one via exclusion)
        # -- the local tramo is empty, so this is not a valid split.
        capsule = _capsule(
            ["crates/auth/src/token.rs", "crates/auth/src/session.rs"],
            {"crates/auth/src/token.rs": 2, "crates/auth/src/session.rs": 25},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_NO_SPLIT)

    def test_ec2_capsule_declares_unassignable_gap_is_impossible_but_partition_is_verified(self) -> None:
        # Sanity check: the partition-completeness assertion holds for a
        # normal 2-module split (both paths accounted for, no overlap).
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs"],
            {"crates/qc/a.rs": 2, "crates/qc/b.rs": 20},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(set(decision.local_paths) | set(decision.cloud_paths), {"crates/qc/a.rs", "crates/qc/b.rs"})
        self.assertEqual(set(decision.local_paths) & set(decision.cloud_paths), set())

    def test_ec2b_duplicate_allowed_path_fails_closed(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/a.rs"],
            {"crates/qc/a.rs": 2},
        )
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split(capsule)
        self.assertEqual(ctx.exception.code, "duplicate_path")

    def test_ec3_missing_cc_value_fails_closed(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs"],
            {"crates/qc/a.rs": 2},
        )
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split(capsule)
        self.assertEqual(ctx.exception.code, "missing_cc")

    def test_ec3b_non_integer_cc_value_fails_closed(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs"],
            {"crates/qc/a.rs": 2, "crates/qc/b.rs": "twenty"},
        )
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split(capsule)
        self.assertEqual(ctx.exception.code, "invalid_cc")

    def test_ec3c_zero_or_negative_cc_value_fails_closed(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs"],
            {"crates/qc/a.rs": 2, "crates/qc/b.rs": 0},
        )
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split(capsule)
        self.assertEqual(ctx.exception.code, "invalid_cc")

    def test_ec3d_bool_cc_value_fails_closed(self) -> None:
        # bool is a subclass of int in Python; must not silently pass as CC=1.
        capsule = _capsule(
            ["crates/qc/a.rs", "crates/qc/b.rs"],
            {"crates/qc/a.rs": 2, "crates/qc/b.rs": True},
        )
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split(capsule)
        self.assertEqual(ctx.exception.code, "invalid_cc")

    def test_ec3e_missing_capsule_field_fails_closed(self) -> None:
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split({"allowed_paths": ["a.rs"]})
        self.assertEqual(ctx.exception.code, "missing_field")

    def test_ec3f_non_dict_capsule_fails_closed(self) -> None:
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split("not-a-dict")
        self.assertEqual(ctx.exception.code, "invalid_capsule")

    def test_ec3g_empty_allowed_paths_fails_closed(self) -> None:
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split(_capsule([], {}))
        self.assertEqual(ctx.exception.code, "missing_field")

    def test_ec3h_invalid_path_traversal_fails_closed(self) -> None:
        capsule = _capsule(
            ["crates/qc/a.rs", "../outside.rs"],
            {"crates/qc/a.rs": 2, "../outside.rs": 20},
        )
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.evaluate_split(capsule)
        self.assertEqual(ctx.exception.code, "invalid_path")

    def test_single_file_task_never_splits(self) -> None:
        decision = _MOD.evaluate_split(_capsule(["crates/qc/a.rs"], {"crates/qc/a.rs": 20}))
        self.assertEqual(decision.decision, _MOD.DECISION_NO_SPLIT)
        self.assertIn("single-file", decision.reason)

    def test_path_with_no_anchor_rubric_match_is_not_hard_excluded(self) -> None:
        # scripts/local-agent/* has no dedicated anchor-rubric row (confirmed
        # in the plan doc); absence of a match must not be treated as
        # exclusion -- it falls through to ordinary CC-based routing.
        capsule = _capsule(
            ["scripts/local-agent/foo.py", "crates/qc/src/complex.rs"],
            {"scripts/local-agent/foo.py": 3, "crates/qc/src/complex.rs": 20},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_SPLIT)
        self.assertIn("scripts/local-agent/foo.py", decision.local_paths)
        module = next(m for m in decision.modules if m.path == "scripts/local-agent/foo.py")
        self.assertFalse(module.hard_excluded)

    def test_docs_path_floor_zero_is_never_hard_excluded(self) -> None:
        capsule = _capsule(
            ["docs/plan/example.md", "crates/qc/src/complex.rs"],
            {"docs/plan/example.md": 2, "crates/qc/src/complex.rs": 20},
        )
        decision = _MOD.evaluate_split(capsule)
        self.assertEqual(decision.decision, _MOD.DECISION_SPLIT)
        self.assertIn("docs/plan/example.md", decision.local_paths)

    def test_custom_rubric_parameter_is_honored(self) -> None:
        # custom_row hard-excludes custom/* regardless of CC; simple.rs has
        # no matching row under this restricted rubric, so it is genuinely
        # local-eligible via its own low CC, keeping the split heterogeneous.
        custom_row = _MOD.rri.RubricRow("custom/*", 4, 4, 4, "n/a", "custom hard-excluded")
        capsule = _capsule(
            ["custom/risky.py", "crates/qc/src/simple.rs"],
            {"custom/risky.py": 2, "crates/qc/src/simple.rs": 3},
        )
        decision = _MOD.evaluate_split(capsule, rubric=[custom_row])
        self.assertEqual(decision.decision, _MOD.DECISION_SPLIT)
        self.assertIn("custom/risky.py", decision.cloud_paths)
        self.assertIn("crates/qc/src/simple.rs", decision.local_paths)


class NextCloudActionTest(unittest.TestCase):
    def test_ec4_zero_attempts_used_returns_attempt(self) -> None:
        self.assertEqual(_MOD.next_cloud_action(0), "attempt")

    def test_ec4b_one_attempt_used_returns_escalate(self) -> None:
        self.assertEqual(_MOD.next_cloud_action(1), "escalate")

    def test_ec4c_escalated_attempt_also_used_returns_stop(self) -> None:
        self.assertEqual(_MOD.next_cloud_action(2), "stop")

    def test_ec4d_negative_attempts_fails_closed(self) -> None:
        with self.assertRaises(_MOD.GateError) as ctx:
            _MOD.next_cloud_action(-1)
        self.assertEqual(ctx.exception.code, "invalid_attempts")


class MainCliTest(unittest.TestCase):
    def setUp(self) -> None:
        import tempfile

        self.tmpdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmpdir.cleanup)
        self.root = Path(self.tmpdir.name)

    def _write(self, name, content):
        path = self.root / name
        path.write_text(content, encoding="utf-8")
        return path

    def test_hp1_valid_split_cli_run_exits_zero(self) -> None:
        import contextlib
        import io
        import json as json_mod

        capsule_path = self._write(
            "capsule.json",
            json_mod.dumps(
                _capsule(
                    ["crates/qc/a.rs", "crates/qc/b.rs"],
                    {"crates/qc/a.rs": 2, "crates/qc/b.rs": 20},
                )
            ),
        )
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(["--capsule", str(capsule_path)])
        self.assertEqual(exit_code, 0)
        output = json_mod.loads(buf.getvalue())
        self.assertEqual(output["decision"], _MOD.DECISION_SPLIT)

    def test_ec1_missing_capsule_file_exits_nonzero(self) -> None:
        import contextlib
        import io
        import json as json_mod

        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(["--capsule", str(self.root / "does-not-exist.json")])
        self.assertEqual(exit_code, 1)
        output = json_mod.loads(buf.getvalue())
        self.assertEqual(output["decision"], _MOD.DECISION_NO_SPLIT)
        self.assertEqual(output["error"]["code"], "io_error")

    def test_ec1b_invalid_json_capsule_exits_nonzero(self) -> None:
        import contextlib
        import io
        import json as json_mod

        capsule_path = self._write("capsule.json", "{not valid json")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(["--capsule", str(capsule_path)])
        self.assertEqual(exit_code, 1)
        output = json_mod.loads(buf.getvalue())
        self.assertEqual(output["error"]["code"], "io_error")

    def test_ec1c_gate_rejection_exits_nonzero(self) -> None:
        import contextlib
        import io
        import json as json_mod

        capsule_path = self._write(
            "capsule.json", json_mod.dumps(_capsule(["a.rs", "b.rs"], {"a.rs": 2}))
        )
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            exit_code = _MOD.main(["--capsule", str(capsule_path)])
        self.assertEqual(exit_code, 1)
        output = json_mod.loads(buf.getvalue())
        self.assertEqual(output["decision"], _MOD.DECISION_NO_SPLIT)
        self.assertEqual(output["error"]["code"], "missing_cc")


if __name__ == "__main__":
    unittest.main()
