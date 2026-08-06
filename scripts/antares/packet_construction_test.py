"""Unit tests for the T3d touchpoint packet-construction integration."""

from __future__ import annotations

import unittest
from pathlib import Path

from scripts.antares.cwe_watchlist import WatchlistValidationError
from scripts.antares.governing_boundary_map import GOVERNING_BOUNDARY_MAP
from scripts.antares.packet_construction import (
    build_refinement_packet,
    build_watchlist_entry_packet,
)
from scripts.antares.packet_schema import (
    CONTEXT_CLOSURE_NO_SEED_REASON,
    CONTEXT_CLOSURE_OMISSION_REASONS,
    explicit_hypothesis,
)

_ROOT = Path(__file__).with_name("testdata") / "governing_boundary_closure" / "snapshot"


class HappyPathTest(unittest.TestCase):
    def test_hp1_clean_seed_under_fully_covered_boundary_has_no_unexpected_omissions(
        self,
    ) -> None:
        packet = build_refinement_packet(
            explicit_hypothesis("CWE-89", "Improper Neutralization of Special Elements"),
            repository_boundary="crates/db/",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=("crates/db/src/lib.rs",),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
        )
        included_paths = {entry.path for entry in packet.included}
        self.assertEqual(
            included_paths,
            {
                "crates/db/src/lib.rs",
                "crates/db/Cargo.toml",
                "docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md",
            },
        )
        self.assertEqual(packet.omitted, ())

    def test_hp2_distinct_boundaries_do_not_cross_contaminate(self) -> None:
        db_packet = build_watchlist_entry_packet(
            "CWE-89",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=("crates/db/src/lib.rs",),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
        )
        api_packet = build_watchlist_entry_packet(
            "CWE-306",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=("apps/api/Cargo.toml",),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
        )
        db_paths = {entry.path for entry in db_packet.included}
        api_paths = {entry.path for entry in api_packet.included}
        self.assertIn("crates/db/Cargo.toml", db_paths)
        self.assertNotIn("apps/api/Cargo.toml", db_paths)
        self.assertIn("apps/api/Cargo.toml", api_paths)
        self.assertIn("docs/architecture.md", api_paths)
        self.assertNotIn("crates/db/src/lib.rs", api_paths)
        self.assertEqual(db_packet.cwe_id, "CWE-89")
        self.assertEqual(api_packet.cwe_id, "CWE-306")


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_expansion_limit_omission_survives_into_final_packet(self) -> None:
        packet = build_refinement_packet(
            explicit_hypothesis("CWE-89", "Improper Neutralization of Special Elements"),
            repository_boundary="crates/db/",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=("crates/db/src/lib.rs",),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
            expansion_limit=0,
        )
        reasons = {o.reason for o in packet.omitted}
        self.assertIn("context_closure_expansion_limit_reached", reasons)
        for reason in reasons:
            self.assertIn(reason, CONTEXT_CLOSURE_OMISSION_REASONS)

    def test_ec2_missing_governing_boundary_omission_reaches_final_packet(self) -> None:
        # crates/storage/ is a valid, declared boundary root in the committed
        # map, but the seed path below falls under crates/db/ instead -- so
        # when the hypothesis's own repository_boundary points at
        # crates/storage/, T3c-2 has no coverage for the crates/db/ closure
        # path and must emit context_closure_missing_governing_boundary.
        packet = build_refinement_packet(
            explicit_hypothesis("CWE-1004", "Boundary coverage miss probe"),
            repository_boundary="crates/storage/",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=("crates/db/src/lib.rs",),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
        )
        omission_keys = {(o.path, o.reason) for o in packet.omitted}
        self.assertIn(
            ("crates/db/Cargo.toml", "context_closure_missing_governing_boundary"),
            omission_keys,
        )
        self.assertIn(
            ("crates/db/src/lib.rs", "context_closure_missing_governing_boundary"),
            omission_keys,
        )

    def test_ec3_path_in_both_closures_is_deduplicated_exactly_once(self) -> None:
        packet = build_refinement_packet(
            explicit_hypothesis("CWE-89", "Improper Neutralization of Special Elements"),
            repository_boundary="crates/db/",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=("crates/db/src/lib.rs",),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
        )
        # crates/db/Cargo.toml is emitted by both T3c-1 (manifest ancestor of
        # the seed) and T3c-2 (mapped governing-boundary target for
        # crates/db/) -- it must appear exactly once in the final packet.
        matches = [entry for entry in packet.included if entry.path == "crates/db/Cargo.toml"]
        self.assertEqual(len(matches), 1)
        all_paths = [entry.path for entry in packet.included]
        self.assertEqual(len(all_paths), len(set(all_paths)))

    def test_ec3_touchpoints_never_bypass_t3c1_seed_path_contract(self) -> None:
        # A raw path with an out-of-snapshot escape must be caught by T3c-1's
        # own canonicalization, not silently forwarded to build_packet.
        packet = build_refinement_packet(
            explicit_hypothesis("CWE-89", "Improper Neutralization of Special Elements"),
            repository_boundary="crates/db/",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=("../../etc/passwd",),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
        )
        included_paths = {entry.path for entry in packet.included}
        self.assertNotIn("../../etc/passwd", included_paths)
        reasons = {o.reason for o in packet.omitted}
        self.assertTrue(reasons)
        for entry in packet.included:
            self.assertFalse(entry.path.startswith(".."))


    def test_ec_empty_seed_produces_no_seed_sentinel_omission(self) -> None:
        packet = build_refinement_packet(
            explicit_hypothesis("CWE-89", "Improper Neutralization of Special Elements"),
            repository_boundary="crates/db/",
            snapshot_root=_ROOT,
            baseline_snapshot_id="baseline-1",
            candidate_snapshot_id="candidate-1",
            changed_paths=(),
            governing_map=GOVERNING_BOUNDARY_MAP,
            size_budget_bytes=1_000_000,
        )
        self.assertEqual(packet.included, ())
        reasons = {o.reason for o in packet.omitted}
        self.assertIn(CONTEXT_CLOSURE_NO_SEED_REASON, reasons)

    def test_ec_unknown_cwe_id_raises_instead_of_producing_empty_packet(self) -> None:
        with self.assertRaises(WatchlistValidationError):
            build_watchlist_entry_packet(
                "CWE-DOES-NOT-EXIST",
                snapshot_root=_ROOT,
                baseline_snapshot_id="baseline-1",
                candidate_snapshot_id="candidate-1",
                changed_paths=("crates/db/src/lib.rs",),
                governing_map=GOVERNING_BOUNDARY_MAP,
                size_budget_bytes=1_000_000,
            )

if __name__ == "__main__":
    unittest.main()
