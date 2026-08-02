"""Characterization tests for the T3c-0 closure corpus and omission contract."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path

from scripts.antares.packet_schema import (
    CONTEXT_CLOSURE_NO_SEED_PATH,
    CONTEXT_CLOSURE_NO_SEED_REASON,
    OmittedPath,
    Packet,
    PacketValidationError,
    SizeBudgetPolicy,
    build_context_closure_no_seed_omission,
    canonicalize_context_closure_seed_path,
    deterministic_context_closure_seed_order,
    validate_packet,
)


_FIXTURE_ROOT = Path(__file__).with_name("testdata") / "context_closure_characterization"
_SNAPSHOT_ROOT = _FIXTURE_ROOT / "snapshot"
_COLLISION_ROOT = _FIXTURE_ROOT / "collision_snapshot"
_MANIFEST_PATH = _FIXTURE_ROOT / "fixture_manifest.json"


def _load_manifest() -> dict[str, object]:
    return json.loads(_MANIFEST_PATH.read_text())


def _sha256(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


class HappyPathTest(unittest.TestCase):
    def test_hp1_manifest_hashes_match_fixture_files_exactly(self) -> None:
        manifest = _load_manifest()
        fixture_files = manifest["fixture_files"]
        assert isinstance(fixture_files, list)

        for entry in fixture_files:
            assert isinstance(entry, dict)
            actual = _sha256(_SNAPSHOT_ROOT / entry["path"])
            self.assertEqual(
                actual,
                entry["sha256"],
                f"fixture digest mismatch for {entry['path']}",
            )

    def test_hp2_manifest_seed_order_is_sorted_over_canonical_snapshot_relative_posix_paths(self) -> None:
        manifest = _load_manifest()
        seed_order = manifest["canonical_seed_order"]
        assert isinstance(seed_order, list)

        canonicalized = [
            canonicalize_context_closure_seed_path(f"./{path_text}", _SNAPSHOT_ROOT)
            for path_text in reversed(seed_order)
        ]

        self.assertEqual(
            tuple(seed_order),
            deterministic_context_closure_seed_order(canonicalized),
        )

    def test_hp3_representative_omission_rows_validate_under_the_packet_schema(self) -> None:
        manifest = _load_manifest()
        representative = manifest["representative_omissions"]
        assert isinstance(representative, list)

        packet = Packet(
            schema_version=1,
            cwe_id="CWE-22",
            cwe_description="Improper Limitation of a Pathname",
            cwe_source="explicit",
            baseline_snapshot_id="baseline@fixture",
            candidate_snapshot_id="candidate@fixture",
            size_budget_bytes=64,
            budget_policy=SizeBudgetPolicy.FAIL_CLOSED.value,
            included=(),
            omitted=tuple(OmittedPath(**entry) for entry in representative),
        )

        validate_packet(packet)
        self.assertEqual(packet.omitted[0].path, CONTEXT_CLOSURE_NO_SEED_PATH)
        self.assertEqual(packet.omitted[0].reason, CONTEXT_CLOSURE_NO_SEED_REASON)


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_empty_seed_contract_is_exactly_one_reserved_sentinel_omission(self) -> None:
        omission = build_context_closure_no_seed_omission()
        packet = Packet(
            schema_version=1,
            cwe_id="CWE-22",
            cwe_description="Improper Limitation of a Pathname",
            cwe_source="explicit",
            baseline_snapshot_id="baseline@empty",
            candidate_snapshot_id="candidate@empty",
            size_budget_bytes=32,
            budget_policy=SizeBudgetPolicy.FAIL_CLOSED.value,
            included=(),
            omitted=(omission,),
        )

        validate_packet(packet)
        self.assertEqual(packet.included, ())
        self.assertEqual(len(packet.omitted), 1)
        self.assertEqual(packet.omitted[0].path, CONTEXT_CLOSURE_NO_SEED_PATH)
        self.assertEqual(packet.omitted[0].reason, CONTEXT_CLOSURE_NO_SEED_REASON)

    def test_ec2_real_seed_filename_collision_is_rejected(self) -> None:
        with self.assertRaises(PacketValidationError):
            canonicalize_context_closure_seed_path(CONTEXT_CLOSURE_NO_SEED_PATH, _COLLISION_ROOT)

    def test_ec3_outside_snapshot_seed_is_soft_omission_not_exception(self) -> None:
        outside = canonicalize_context_closure_seed_path("../outside.rs", _SNAPSHOT_ROOT)
        self.assertIsNone(outside)


if __name__ == "__main__":
    unittest.main()
