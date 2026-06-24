#!/usr/bin/env python3
"""Unit tests for scripts/adjudicator-packet.py (T6b).

Run:
    python3 -m unittest scripts/adjudicator_packet_test.py
    python3 scripts/adjudicator_packet_test.py
"""

import importlib.util
import os
import unittest

_SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "adjudicator-packet.py")
_spec = importlib.util.spec_from_file_location("adjudicator_packet", _SCRIPT)
_mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_mod)


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def _empty_aggregate():
    return {
        "status": "pass",
        "findings": [],
        "reconciliation": {
            "consensus": [],
            "pass_specific": [],
            "severity_inconsistent": [],
            "location_inconsistent": [],
            "likely_false_positive": [],
            "consensus_count": 0,
            "pass_specific_count": 0,
            "severity_inconsistent_count": 0,
            "location_inconsistent_count": 0,
            "likely_false_positive_count": 0,
        },
    }


def _finding(severity="minor", scope="in-scope"):
    return {
        "path": "src/foo.py",
        "line": 10,
        "severity": severity,
        "detail": "test finding",
        "suggestion": "fix it",
        "scope": scope,
    }


def _diff():
    return (
        "--- a/src/foo.py\n"
        "+++ b/src/foo.py\n"
        "@@ -1 +1 @@\n"
        "-old\n"
        "+new\n"
    )


# ---------------------------------------------------------------------------
# should_adjudicate — gemma_blocked mandatory fallback
# ---------------------------------------------------------------------------

class TestShouldAdjudicateGemmaBlocked(unittest.TestCase):
    """When gemma_blocked=True the trigger must always fire, regardless of
    band, findings, or aggregate content. The isolated subagent is the
    mandatory fallback reviewer when Gemma is unavailable or quorum fails."""

    def test_fires_low_band_no_findings(self):
        self.assertTrue(_mod.should_adjudicate(_empty_aggregate(), "Low", gemma_blocked=True))

    def test_fires_moderate_band_pass(self):
        self.assertTrue(_mod.should_adjudicate(_empty_aggregate(), "Moderate", gemma_blocked=True))

    def test_fires_aggregate_none(self):
        # Gemma unavailable → no aggregate at all; must still fire.
        self.assertTrue(_mod.should_adjudicate(None, "Low", gemma_blocked=True))

    def test_fires_aggregate_empty_dict(self):
        self.assertTrue(_mod.should_adjudicate({}, "Moderate", gemma_blocked=True))

    def test_fires_low_band_nit_only_findings(self):
        # Even if the partial results contain only nit findings, blocked fires.
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("nit")]
        self.assertTrue(_mod.should_adjudicate(agg, "Low", gemma_blocked=True))

    def test_default_not_blocked(self):
        # gemma_blocked defaults to False; existing logic unchanged when omitted.
        self.assertFalse(_mod.should_adjudicate(_empty_aggregate(), "Low"))

    def test_quorum_failure_analog_low_band(self):
        # Quorum failure (<2 passes) → caller passes gemma_blocked=True.
        self.assertTrue(_mod.should_adjudicate(_empty_aggregate(), "Low", gemma_blocked=True))


# ---------------------------------------------------------------------------
# should_adjudicate — does NOT fire
# ---------------------------------------------------------------------------

class TestShouldAdjudicateNoFire(unittest.TestCase):

    def test_low_band_pass_no_findings(self):
        self.assertFalse(_mod.should_adjudicate(_empty_aggregate(), "Low"))

    def test_moderate_band_pass_no_findings(self):
        self.assertFalse(_mod.should_adjudicate(_empty_aggregate(), "Moderate"))

    def test_low_band_consensus_nit_only(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("nit")]
        agg["reconciliation"]["consensus_count"] = 1
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_low_band_consensus_minor_only(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("minor")]
        agg["reconciliation"]["consensus_count"] = 1
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_moderate_band_consensus_minor_only_no_disagreement(self):
        # EC-1: findings exist but all nit/minor, band below Med-high, no disagreement.
        agg = _empty_aggregate()
        agg["status"] = "findings"
        agg["reconciliation"]["consensus"] = [_finding("nit"), _finding("minor")]
        agg["reconciliation"]["consensus_count"] = 2
        self.assertFalse(_mod.should_adjudicate(agg, "Moderate"))

    def test_low_band_pass_specific_only(self):
        # pass_specific alone is not inter-pass disagreement for the trigger.
        agg = _empty_aggregate()
        agg["reconciliation"]["pass_specific_count"] = 3
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_low_band_likely_false_positive_only(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["likely_false_positive_count"] = 2
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_no_reconciliation_block_low_band(self):
        # Single-pass result has no reconciliation block.
        agg = {"status": "pass", "findings": []}
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_no_reconciliation_block_moderate_band(self):
        agg = {"status": "findings", "findings": [_finding("minor")]}
        self.assertFalse(_mod.should_adjudicate(agg, "Moderate"))


# ---------------------------------------------------------------------------
# should_adjudicate — fires on consensus blocking/major
# ---------------------------------------------------------------------------

class TestShouldAdjudicateConsensus(unittest.TestCase):

    def test_fires_consensus_blocking_low_band(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("blocking")]
        agg["reconciliation"]["consensus_count"] = 1
        self.assertTrue(_mod.should_adjudicate(agg, "Low"))

    def test_fires_consensus_major_low_band(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("major")]
        agg["reconciliation"]["consensus_count"] = 1
        self.assertTrue(_mod.should_adjudicate(agg, "Low"))

    def test_fires_consensus_major_moderate_band(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("major")]
        agg["reconciliation"]["consensus_count"] = 1
        self.assertTrue(_mod.should_adjudicate(agg, "Moderate"))

    def test_no_fire_consensus_minor_low_band(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("minor")]
        agg["reconciliation"]["consensus_count"] = 1
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_no_fire_consensus_nit_low_band(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("nit")]
        agg["reconciliation"]["consensus_count"] = 1
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_fires_mixed_consensus_one_blocking(self):
        # Any consensus blocking/major is sufficient even alongside nit/minor.
        agg = _empty_aggregate()
        agg["reconciliation"]["consensus"] = [_finding("nit"), _finding("blocking")]
        agg["reconciliation"]["consensus_count"] = 2
        self.assertTrue(_mod.should_adjudicate(agg, "Low"))


# ---------------------------------------------------------------------------
# should_adjudicate — fires on band >= Med-high
# ---------------------------------------------------------------------------

class TestShouldAdjudicateBand(unittest.TestCase):

    def test_fires_med_high_band_no_findings(self):
        self.assertTrue(_mod.should_adjudicate(_empty_aggregate(), "Med-high"))

    def test_fires_complex_band_no_findings(self):
        self.assertTrue(_mod.should_adjudicate(_empty_aggregate(), "Complex"))

    def test_no_fire_moderate_band_pass_no_findings(self):
        self.assertFalse(_mod.should_adjudicate(_empty_aggregate(), "Moderate"))

    def test_fires_med_high_no_reconciliation_block(self):
        agg = {"status": "pass", "findings": []}
        self.assertTrue(_mod.should_adjudicate(agg, "Med-high"))

    def test_fires_complex_no_reconciliation_block(self):
        agg = {"status": "pass", "findings": []}
        self.assertTrue(_mod.should_adjudicate(agg, "Complex"))


# ---------------------------------------------------------------------------
# should_adjudicate — fires on inter-pass disagreement
# ---------------------------------------------------------------------------

class TestShouldAdjudicateDisagreement(unittest.TestCase):

    def test_fires_severity_inconsistent_low_band(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["severity_inconsistent_count"] = 1
        self.assertTrue(_mod.should_adjudicate(agg, "Low"))

    def test_fires_location_inconsistent_low_band(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["location_inconsistent_count"] = 2
        self.assertTrue(_mod.should_adjudicate(agg, "Moderate"))

    def test_fires_both_disagreement_types(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["severity_inconsistent_count"] = 1
        agg["reconciliation"]["location_inconsistent_count"] = 1
        self.assertTrue(_mod.should_adjudicate(agg, "Low"))

    def test_no_fire_pass_specific_only_low_band(self):
        # pass_specific is single-pass noise, not cross-pass contradiction.
        agg = _empty_aggregate()
        agg["reconciliation"]["pass_specific_count"] = 5
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))

    def test_no_fire_zero_disagreement_counts(self):
        agg = _empty_aggregate()
        agg["reconciliation"]["severity_inconsistent_count"] = 0
        agg["reconciliation"]["location_inconsistent_count"] = 0
        self.assertFalse(_mod.should_adjudicate(agg, "Low"))


# ---------------------------------------------------------------------------
# build_adjudicator_packet — happy paths
# ---------------------------------------------------------------------------

class TestBuildAdjudicatorPacket(unittest.TestCase):

    def test_happy_path_shape(self):
        # HP-1: consensus blocking finding — packet built with all three sections.
        findings = [_finding("blocking")]
        packet = _mod.build_adjudicator_packet(
            diff=_diff(),
            criteria="all unit tests pass; no regressions",
            reconciled_findings=findings,
        )
        self.assertEqual(set(packet.keys()), _mod.ALLOWED_PACKET_SECTIONS)
        self.assertIn("--- a/src/foo.py", packet["diff"])
        self.assertEqual(packet["criteria"], "all unit tests pass; no regressions")
        self.assertEqual(packet["reconciled_findings"], findings)

    def test_empty_findings_allowed(self):
        # HP-2 analog: Low task, 3/3 PASS — trigger would not fire, but if
        # build_adjudicator_packet is called it still produces a valid packet.
        packet = _mod.build_adjudicator_packet(
            diff=_diff(),
            criteria="criteria text",
            reconciled_findings=[],
        )
        self.assertEqual(packet["reconciled_findings"], [])
        self.assertEqual(set(packet.keys()), _mod.ALLOWED_PACKET_SECTIONS)

    def test_multiple_findings_preserved(self):
        findings = [_finding("blocking"), _finding("major"), _finding("minor")]
        packet = _mod.build_adjudicator_packet(
            diff=_diff(),
            criteria="acceptance criteria",
            reconciled_findings=findings,
        )
        self.assertEqual(len(packet["reconciled_findings"]), 3)


# ---------------------------------------------------------------------------
# build_adjudicator_packet — isolation (allowlist enforcement)
# ---------------------------------------------------------------------------

class TestPacketIsolation(unittest.TestCase):

    def test_packet_contains_only_allowed_sections(self):
        # Core isolation assertion: output has exactly the allowlisted keys.
        packet = _mod.build_adjudicator_packet(
            diff="diff content",
            criteria="acceptance criteria",
            reconciled_findings=[],
        )
        disallowed = set(packet.keys()) - _mod.ALLOWED_PACKET_SECTIONS
        self.assertEqual(disallowed, set(), msg=f"packet contains disallowed sections: {disallowed}")

    def test_assert_isolation_raises_on_extra_key(self):
        # EC-2: a caller tries to pass development-transcript text — builder
        # excludes it; isolation test fails closed if it ever appears.
        with self.assertRaises(ValueError) as ctx:
            _mod._assert_packet_isolation({
                "diff": "d",
                "criteria": "c",
                "reconciled_findings": [],
                "development_transcript": "secret chain-of-thought",
            })
        self.assertIn("disallowed sections", str(ctx.exception))
        self.assertIn("development_transcript", str(ctx.exception))

    def test_assert_isolation_raises_on_thinking_key(self):
        with self.assertRaises(ValueError) as ctx:
            _mod._assert_packet_isolation({
                "diff": "d",
                "criteria": "c",
                "reconciled_findings": [],
                "thinking": "internal reasoning",
            })
        self.assertIn("disallowed sections", str(ctx.exception))

    def test_assert_isolation_raises_on_missing_section(self):
        with self.assertRaises(ValueError) as ctx:
            _mod._assert_packet_isolation({"diff": "d", "criteria": "c"})
        self.assertIn("missing required sections", str(ctx.exception))

    def test_assert_isolation_passes_on_exact_allowlist(self):
        # Should not raise when given exactly the three allowed keys.
        _mod._assert_packet_isolation({
            "diff": "d",
            "criteria": "c",
            "reconciled_findings": [],
        })

    def test_allowed_packet_sections_constant_is_closed(self):
        # The allowlist must be exactly three sections; no more, no less.
        self.assertEqual(
            _mod.ALLOWED_PACKET_SECTIONS,
            {"diff", "criteria", "reconciled_findings"},
        )


# ---------------------------------------------------------------------------
# disposition_divergence field contract
# ---------------------------------------------------------------------------

class TestDispositionDivergenceField(unittest.TestCase):

    def test_valid_string_values_documented(self):
        self.assertIn("none", _mod.DISPOSITION_DIVERGENCE_VALUES)
        self.assertIn("partial", _mod.DISPOSITION_DIVERGENCE_VALUES)
        self.assertIn("full", _mod.DISPOSITION_DIVERGENCE_VALUES)

    def test_none_string_represents_agreement(self):
        # EC-3: adjudicator disposition matches the primary's →
        # disposition_divergence records "none", still logged.
        self.assertIn("none", _mod.DISPOSITION_DIVERGENCE_VALUES)

    def test_null_python_none_is_not_a_string_value(self):
        # null (Python None) means "not yet populated" and is not a string entry.
        self.assertNotIn(None, _mod.DISPOSITION_DIVERGENCE_VALUES)

    def test_value_set_is_exactly_three(self):
        # The documented set is closed; no undocumented values should be added.
        self.assertEqual(len(_mod.DISPOSITION_DIVERGENCE_VALUES), 3)

    def test_divergence_value_none_distinct_from_python_none(self):
        # "none" (string) != None (null) — they have different semantics.
        self.assertNotEqual("none", None)
        self.assertIn("none", _mod.DISPOSITION_DIVERGENCE_VALUES)


if __name__ == "__main__":
    unittest.main()
