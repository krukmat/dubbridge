#!/usr/bin/env python3
# Unit tests for the RRI calculator. [rri-calculator-script T1]
# Run: python3 scripts/rri_test.py   (or: python3 -m unittest scripts/rri_test.py)
import os
import subprocess
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rri  # noqa: E402

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "rri.py")


def base_of(**kw):
    """Evaluate with safe defaults and return the base value (no git, no rubric)."""
    defaults = dict(f_override=0, d=0, k=0, p=0, t=0, a=0, x=0)
    defaults.update(kw)
    return rri.evaluate(**defaults)["base"]


class CcMapping(unittest.TestCase):
    def test_boundaries(self):
        cases = {5: 0, 6: 1, 10: 1, 11: 2, 20: 2, 21: 3, 30: 3, 31: 4, 50: 4, 51: 5}
        for raw, score in cases.items():
            self.assertEqual(rri.cc_to_score(raw), score, f"cc {raw}")


class FMapping(unittest.TestCase):
    def test_boundaries(self):
        cases = {1: 0, 2: 1, 3: 2, 5: 2, 6: 3, 10: 3, 11: 4, 20: 4, 21: 5, 99: 5}
        for n, score in cases.items():
            self.assertEqual(rri.count_to_f(n), score, f"count {n}")


class BaseFormula(unittest.TestCase):
    def test_all_zero(self):
        self.assertEqual(base_of(c_score=0), 0)

    def test_all_five(self):
        # 100 * (5 * sum(weights=1.00) / 5) = 100.
        self.assertEqual(base_of(c_score=5, f_override=5, d=5, k=5, p=5, t=5, a=5, x=5), 100)

    def test_t1_vector(self):
        # C1 F1 D1 T2 A0 K1 P0 X2 -> 0.99 / 5 * 100 = 19.8 -> 20.
        self.assertEqual(
            base_of(c_score=1, f_override=1, d=1, k=1, p=0, t=2, a=0, x=2), 20)


class AnchorRubric(unittest.TestCase):
    def test_floor_raises_auth(self):
        r = rri.evaluate(c_score=0, touches=["crates/auth/src/lib.rs"],
                         d=1, k=1, p=1, t=0, a=0, x=0)
        self.assertEqual(r["scores"]["D"], 4)
        self.assertEqual(r["scores"]["P"], 4)
        self.assertEqual(r["scores"]["K"], 4)

    def test_floor_only_raises_never_lowers(self):
        # Agent D=4 against a floor-2 path (crates/domain) -> kept at 4.
        r = rri.evaluate(c_score=0, touches=["crates/domain/src/model.rs"],
                         d=4, k=0, p=0, t=0, a=0, x=0)
        self.assertEqual(r["scores"]["D"], 4)

    def test_rights_ledger_more_specific_than_domain(self):
        r = rri.evaluate(c_score=0, touches=["crates/domain/src/rights.rs"],
                         d=0, k=0, p=0, t=0, a=0, x=0)
        self.assertEqual(r["scores"]["P"], 5)  # rights-ledger row, not generic domain

    def test_unmatched_path_advisory(self):
        r = rri.evaluate(c_score=0, touches=["scripts/rri.py"],
                         d=1, k=1, p=0, t=0, a=0, x=0)
        self.assertTrue(any("no anchor-rubric match" in n for n in r["advisories"]))


class Penalties(unittest.TestCase):
    def test_many_files_auto(self):
        r = rri.evaluate(c_score=0, f_override=4, d=0, k=0, p=0, t=0, a=0, x=0)
        self.assertIn("many_files", r["penalties"])
        self.assertEqual(r["penalties"]["many_files"][0], 8)

    def test_complex_and_domain_auto(self):
        r = rri.evaluate(c_score=4, f_override=0, d=3, k=0, p=0, t=0, a=0, x=0)
        self.assertIn("complex_and_domain", r["penalties"])

    def test_no_tests_high_impact_auto(self):
        r = rri.evaluate(c_score=0, f_override=0, d=0, k=0, p=4, t=4, a=0, x=0)
        self.assertIn("no_tests_high_impact", r["penalties"])

    def test_auth_security_auto_from_rubric(self):
        r = rri.evaluate(c_score=0, touches=["crates/auth/src/lib.rs"],
                         d=0, k=0, p=0, t=0, a=0, x=0)
        self.assertIn("auth_security", r["penalties"])

    def test_manual_penalty_dedup(self):
        # Manual auth_security plus rubric auto -> applied once.
        r = rri.evaluate(c_score=0, touches=["crates/auth/src/lib.rs"],
                         d=0, k=0, p=0, t=0, a=0, x=0,
                         manual_penalties=["auth_security"])
        self.assertEqual(r["penalties"]["auth_security"][1],
                         "anchor-rubric P floor >= 4 (auth/audit/rights/secrets)")

    def test_manual_only_penalties(self):
        r = rri.evaluate(c_score=0, f_override=0, d=0, k=0, p=0, t=0, a=0, x=0,
                         manual_penalties=["arch_decision", "no_verification"])
        self.assertEqual(r["penalty_total"], 12 + 15)


class Bands(unittest.TestCase):
    def test_band_boundaries(self):
        self.assertEqual(rri.resolve_band(25)["label"], "Low")
        self.assertEqual(rri.resolve_band(26)["label"], "Moderate")
        self.assertEqual(rri.resolve_band(55)["label"], "Med-high")
        self.assertEqual(rri.resolve_band(56)["label"], "Complex")
        self.assertEqual(rri.resolve_band(70)["label"], "Complex")
        self.assertEqual(rri.resolve_band(71)["label"], "High")
        self.assertEqual(rri.resolve_band(100)["label"], "Very high")
        self.assertEqual(rri.resolve_band(101)["label"], "Excessive")

    def test_crosswalk_fields(self):
        b = rri.resolve_band(30)
        self.assertEqual(b["effort"], "M")
        self.assertEqual(b["thinking"], "Off")


class Decomposition(unittest.TestCase):
    def test_complex_and_domain_triggers(self):
        r = rri.evaluate(c_score=4, f_override=0, d=3, k=0, p=0, t=0, a=0, x=0)
        self.assertTrue(any("C >= 4 and D >= 3" in t for t in r["triggers"]))

    def test_high_rri_triggers(self):
        r = rri.evaluate(c_score=5, f_override=5, d=5, k=5, p=5, t=5, a=5, x=5)
        self.assertTrue(any("RRI > 70" in t for t in r["triggers"]))

    def test_no_trigger_low(self):
        r = rri.evaluate(c_score=1, f_override=1, d=1, k=1, p=0, t=2, a=0, x=2)
        self.assertEqual(r["triggers"], [])


class LowConfidence(unittest.TestCase):
    def test_bump_marks_low_and_raises(self):
        r = rri.evaluate(c_score=2, f_override=0, d=2, k=2, p=2, t=2, a=2, x=2,
                         low_conf=["D"])
        self.assertEqual(r["scores"]["D"], 3)
        self.assertEqual(r["confidence"]["D"], "Low")

    def test_bump_capped_at_five(self):
        r = rri.evaluate(c_score=0, f_override=0, d=5, k=0, p=0, t=0, a=0, x=0,
                         low_conf=["D"])
        self.assertEqual(r["scores"]["D"], 5)


class CliBehavior(unittest.TestCase):
    """Exit-code and CLI-parsing behavior via subprocess (EC-1, EC-2, EC-3, EC-6)."""

    def run_cli(self, *args):
        return subprocess.run([sys.executable, SCRIPT, *args],
                              capture_output=True, text=True)

    def test_ok_markdown(self):
        r = self.run_cli("--cc", "12", "--F", "1", "--D", "2", "--K", "2",
                         "--P", "2", "--T", "2", "--A", "0", "--X", "2")
        self.assertEqual(r.returncode, 0)
        self.assertIn("Final RRI:", r.stdout)

    def test_json_output(self):
        import json
        r = self.run_cli("--cc", "12", "--F", "1", "--D", "2", "--K", "2",
                         "--P", "2", "--T", "2", "--A", "0", "--X", "2", "--json")
        self.assertEqual(r.returncode, 0)
        data = json.loads(r.stdout)
        for key in ("variables", "base", "penalties", "final", "band", "triggers"):
            self.assertIn(key, data)

    def test_score_out_of_range(self):  # EC-1
        r = self.run_cli("--C", "6", "--F", "1", "--D", "2", "--K", "2",
                         "--P", "2", "--T", "2", "--A", "0", "--X", "2")
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("0-5", r.stderr)

    def test_unknown_penalty(self):  # EC-2
        r = self.run_cli("--cc", "12", "--F", "1", "--D", "2", "--K", "2",
                         "--P", "2", "--T", "2", "--A", "0", "--X", "2",
                         "--penalty", "bogus")
        self.assertNotEqual(r.returncode, 0)

    def test_both_cc_and_C(self):  # EC-6
        r = self.run_cli("--cc", "12", "--C", "2", "--F", "1", "--D", "2",
                         "--K", "2", "--P", "2", "--T", "2", "--A", "0", "--X", "2")
        self.assertNotEqual(r.returncode, 0)

    def test_neither_cc_nor_C(self):  # EC-6
        r = self.run_cli("--F", "1", "--D", "2", "--K", "2", "--P", "2",
                         "--T", "2", "--A", "0", "--X", "2")
        self.assertNotEqual(r.returncode, 0)

    def test_cc_below_one(self):
        r = self.run_cli("--cc", "0", "--F", "1", "--D", "2", "--K", "2",
                         "--P", "2", "--T", "2", "--A", "0", "--X", "2")
        self.assertNotEqual(r.returncode, 0)


class GitDiffBehavior(unittest.TestCase):
    """Document the git-diff F=0 limitation explicitly."""

    def test_empty_diff_yields_f_zero(self):
        # When git diff returns no files (e.g. no commits ahead of base, or
        # all changes are uncommitted), the script correctly reports F=0.
        # This is expected behaviour — use --touches at task-presentation time
        # to declare paths before committing (documented in RRI_POLICY.md
        # § Script automation).
        empty_git = lambda base: []  # noqa: E731
        r = rri.evaluate(c_score=0, d=0, k=0, p=0, t=0, a=0, x=0,
                         git=empty_git)
        self.assertEqual(r["scores"]["F"], 0)
        self.assertIn("0 files", r["evidence"]["F"])

    def test_touches_overrides_git(self):
        # --touches takes precedence over git diff; git is never called.
        def git_should_not_be_called(base):
            raise AssertionError("git should not be called when --touches is given")
        r = rri.evaluate(c_score=0, touches=["crates/db/src/lib.rs"],
                         d=0, k=0, p=0, t=0, a=0, x=0,
                         git=git_should_not_be_called)
        self.assertEqual(r["scores"]["F"], 0)  # 1 file -> F score 0


if __name__ == "__main__":
    unittest.main(verbosity=2)
