"""Tests for the T3b packet schema and exclusion guarantees."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.antares.packet_schema import (
    PacketSizeBudgetExceeded,
    SizeBudgetPolicy,
    build_packet,
    explicit_hypothesis,
    hypothesis_from_watchlist,
    serialize_packet,
    validate_packet,
)


class _PacketSchemaTestCase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.snapshot_root = Path(self._tmp.name)
        (self.snapshot_root / "src").mkdir()
        (self.snapshot_root / "config").mkdir()
        (self.snapshot_root / "build").mkdir()
        (self.snapshot_root / "secrets").mkdir()
        (self.snapshot_root / "nested" / "config").mkdir(parents=True)
        (self.snapshot_root / "src" / "main.rs").write_text("fn main() { println!(\"hi\"); }\n")
        (self.snapshot_root / "src" / "lib.rs").write_text("pub fn helper() -> u8 { 7 }\n")
        (self.snapshot_root / ".env").write_text("API_KEY=top-secret\n")
        (self.snapshot_root / "config" / "production.toml").write_text("db = \"prod\"\n")
        (self.snapshot_root / "nested" / "config" / "production.toml").write_text("db = \"prod\"\n")
        (self.snapshot_root / "build" / "generated.txt").write_text("generated\n")
        (self.snapshot_root / "secrets" / "api.key").write_text("super-secret\n")
        (self.snapshot_root / "large.txt").write_text("A" * 128)
        (self.snapshot_root / "alias-to-main.rs").symlink_to(
            self.snapshot_root / "src" / "main.rs"
        )
        self._outside = tempfile.TemporaryDirectory()
        self.outside_root = Path(self._outside.name)
        (self.outside_root / "escape.txt").write_text("escape\n")
        (self.snapshot_root / "escape-link").symlink_to(self.outside_root / "escape.txt")

    def tearDown(self) -> None:
        self._tmp.cleanup()
        self._outside.cleanup()


class HappyPathTest(_PacketSchemaTestCase):
    def test_hp1_watchlist_hypothesis_validates_and_serializes_deterministically(self) -> None:
        packet = build_packet(
            hypothesis_from_watchlist("CWE-89"),
            baseline_snapshot_id="baseline@abc123",
            candidate_snapshot_id="candidate@def456",
            snapshot_root=self.snapshot_root,
            raw_paths=("src/main.rs", "./src/lib.rs"),
            size_budget_bytes=4096,
        )
        reordered = build_packet(
            hypothesis_from_watchlist("CWE-89"),
            baseline_snapshot_id="baseline@abc123",
            candidate_snapshot_id="candidate@def456",
            snapshot_root=self.snapshot_root,
            raw_paths=("./src/lib.rs", "src/main.rs"),
            size_budget_bytes=4096,
        )

        validate_packet(packet)
        self.assertEqual(packet.cwe_source, "watchlist")
        self.assertEqual(
            tuple(entry.path for entry in packet.included),
            ("src/lib.rs", "src/main.rs"),
        )
        self.assertEqual(serialize_packet(packet), serialize_packet(reordered))

    def test_hp2_within_budget_packet_is_accepted_as_is(self) -> None:
        packet = build_packet(
            explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            snapshot_root=self.snapshot_root,
            raw_paths=("src/main.rs",),
            size_budget_bytes=4096,
            budget_policy=SizeBudgetPolicy.FAIL_CLOSED,
        )

        self.assertEqual(len(packet.included), 1)
        self.assertEqual(packet.included[0].path, "src/main.rs")
        self.assertIsNone(packet.included[0].fragment)
        self.assertEqual(packet.omitted, ())


class EdgeCaseTest(_PacketSchemaTestCase):
    def test_ec1_sensitive_and_generated_paths_are_excluded_and_recorded(self) -> None:
        packet = build_packet(
            explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            snapshot_root=self.snapshot_root,
            raw_paths=(
                "src/main.rs",
                ".env",
                "config/production.toml",
                "nested/config/production.toml",
                "build/generated.txt",
                "secrets/api.key",
            ),
            size_budget_bytes=4096,
        )

        omitted = {(entry.path, entry.reason) for entry in packet.omitted}
        self.assertIn((".env", "security_excluded_env_file"), omitted)
        self.assertIn(
            ("config/production.toml", "security_excluded_production_config"),
            omitted,
        )
        self.assertIn(
            ("nested/config/production.toml", "security_excluded_production_config"),
            omitted,
        )
        self.assertIn(("build/generated.txt", "security_excluded_generated_output"), omitted)
        self.assertIn(("secrets/api.key", "security_excluded_credentials"), omitted)
        self.assertEqual(tuple(entry.path for entry in packet.included), ("src/main.rs",))

    def test_ec2_out_of_snapshot_path_is_excluded_and_reported(self) -> None:
        packet = build_packet(
            explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            snapshot_root=self.snapshot_root,
            raw_paths=("escape-link",),
            size_budget_bytes=4096,
        )

        self.assertEqual(packet.included, ())
        self.assertEqual(len(packet.omitted), 1)
        self.assertEqual(packet.omitted[0].reason, "path_outside_snapshot")
        self.assertTrue(packet.omitted[0].path.startswith("/"))

    def test_ec3_fail_closed_budget_rejects_oversize_packet(self) -> None:
        with self.assertRaises(PacketSizeBudgetExceeded):
            build_packet(
                explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
                baseline_snapshot_id="baseline@1",
                candidate_snapshot_id="candidate@2",
                snapshot_root=self.snapshot_root,
                raw_paths=("large.txt",),
                size_budget_bytes=32,
                budget_policy=SizeBudgetPolicy.FAIL_CLOSED,
            )

    def test_ec3_security_exclusion_runs_before_size_budget(self) -> None:
        (self.snapshot_root / ".env").write_text("A" * 8192)
        packet = build_packet(
            explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            snapshot_root=self.snapshot_root,
            raw_paths=(".env", "src/main.rs"),
            size_budget_bytes=64,
            budget_policy=SizeBudgetPolicy.FAIL_CLOSED,
        )

        self.assertEqual(tuple(entry.path for entry in packet.included), ("src/main.rs",))
        self.assertIn(
            (".env", "security_excluded_env_file"),
            {(entry.path, entry.reason) for entry in packet.omitted},
        )

    def test_ec3_partition_budget_records_fragment_and_remainder(self) -> None:
        packet = build_packet(
            explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            snapshot_root=self.snapshot_root,
            raw_paths=("large.txt",),
            size_budget_bytes=32,
            budget_policy=SizeBudgetPolicy.DETERMINISTIC_PARTITION,
        )

        self.assertEqual(len(packet.included), 1)
        fragment = packet.included[0].fragment
        self.assertIsNotNone(fragment)
        assert fragment is not None
        self.assertEqual(fragment.start_byte, 0)
        self.assertEqual(fragment.end_byte_exclusive, 32)
        self.assertEqual(fragment.total_file_bytes, 128)
        self.assertEqual(len(packet.omitted), 1)
        self.assertEqual(packet.omitted[0].path, "large.txt")
        self.assertEqual(packet.omitted[0].reason, "size_budget_fragment_omitted_remainder")

    def test_ec4_alternate_spellings_canonicalize_to_one_path(self) -> None:
        packet = build_packet(
            explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            snapshot_root=self.snapshot_root,
            raw_paths=("src/main.rs", "./src/main.rs", "alias-to-main.rs"),
            size_budget_bytes=4096,
        )

        self.assertEqual(tuple(entry.path for entry in packet.included), ("src/main.rs",))
        self.assertEqual(packet.omitted, ())


if __name__ == "__main__":
    unittest.main()
