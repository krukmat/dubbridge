"""Tests for the T3b packet schema and exclusion guarantees."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.antares.packet_schema import (
    CONTEXT_CLOSURE_NO_SEED_PATH,
    CONTEXT_CLOSURE_NO_SEED_REASON,
    OmittedPath,
    Packet,
    PacketSizeBudgetExceeded,
    PacketValidationError,
    SizeBudgetPolicy,
    build_packet,
    build_context_closure_no_seed_omission,
    canonicalize_context_closure_seed_path,
    deterministic_context_closure_seed_order,
    explicit_hypothesis,
    hypothesis_from_watchlist,
    serialize_packet,
    validate_packet,
    validate_context_closure_seed_path,
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
        (self.snapshot_root / "python" / "pkg").mkdir(parents=True)
        (self.snapshot_root / "rust" / "src").mkdir(parents=True)
        (self.snapshot_root / "src" / "main.rs").write_text("fn main() { println!(\"hi\"); }\n")
        (self.snapshot_root / "src" / "lib.rs").write_text("pub fn helper() -> u8 { 7 }\n")
        (self.snapshot_root / "python" / "checks.py").write_text("print('ok')\n")
        (self.snapshot_root / "python" / "pkg" / "__init__.py").write_text("__all__ = ['checks']\n")
        (self.snapshot_root / "rust" / "src" / "lib.rs").write_text("pub fn helper() -> u8 { 7 }\n")
        (self.snapshot_root / "rust" / "src" / "main.rs").write_text("fn main() {}\n")
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

    def test_hp3_derived_context_omission_reasons_validate_and_serialize(self) -> None:
        packet = Packet(
            schema_version=1,
            cwe_id="CWE-22",
            cwe_description="Improper Limitation of a Pathname",
            cwe_source="explicit",
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            size_budget_bytes=128,
            budget_policy=SizeBudgetPolicy.FAIL_CLOSED.value,
            included=(),
            omitted=(
                build_context_closure_no_seed_omission(),
                OmittedPath(
                    path="docs/spec.yaml",
                    reason="context_closure_unsupported_file_type",
                    detail="Representative contract row for unsupported file types.",
                ),
                OmittedPath(
                    path="src/security/mod.rs",
                    reason="context_closure_missing_governing_boundary",
                    detail="Representative contract row for a missing governing boundary mapping.",
                ),
                OmittedPath(
                    path="src/deep/module.rs",
                    reason="context_closure_expansion_limit_reached",
                    detail="Representative contract row for synthetic expansion-limit coverage.",
                ),
            ),
        )

        validate_packet(packet)
        self.assertIn(CONTEXT_CLOSURE_NO_SEED_REASON, serialize_packet(packet))

    def test_hp4_seed_paths_canonicalize_and_sort_deterministically(self) -> None:
        ordered = deterministic_context_closure_seed_order(
            (
                canonicalize_context_closure_seed_path("./rust/src/main.rs", self.snapshot_root),
                canonicalize_context_closure_seed_path("python/checks.py", self.snapshot_root),
                canonicalize_context_closure_seed_path("./rust/src/lib.rs", self.snapshot_root),
                canonicalize_context_closure_seed_path("python/pkg/__init__.py", self.snapshot_root),
            )
        )

        self.assertEqual(
            ordered,
            (
                "python/checks.py",
                "python/pkg/__init__.py",
                "rust/src/lib.rs",
                "rust/src/main.rs",
            ),
        )


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

    def test_ec5_unknown_derived_context_omission_reason_is_rejected(self) -> None:
        packet = Packet(
            schema_version=1,
            cwe_id="CWE-22",
            cwe_description="Improper Limitation of a Pathname",
            cwe_source="explicit",
            baseline_snapshot_id="baseline@1",
            candidate_snapshot_id="candidate@2",
            size_budget_bytes=64,
            budget_policy=SizeBudgetPolicy.FAIL_CLOSED.value,
            included=(),
            omitted=(
                OmittedPath(
                    path="src/main.rs",
                    reason="context_closure_missing_boundary",
                    detail="Misspelled representative omission reason.",
                ),
            ),
        )

        with self.assertRaises(PacketValidationError):
            validate_packet(packet)

    def test_ec6_real_seed_file_is_rejected_and_cannot_collide_with_sentinel(self) -> None:
        reserved = self.snapshot_root / CONTEXT_CLOSURE_NO_SEED_PATH
        reserved.write_text("reserved collision\n")
        self.addCleanup(lambda: reserved.unlink(missing_ok=True))

        with self.assertRaises(PacketValidationError):
            canonicalize_context_closure_seed_path(CONTEXT_CLOSURE_NO_SEED_PATH, self.snapshot_root)

        with self.assertRaises(PacketValidationError):
            build_packet(
                explicit_hypothesis("CWE-22", "Improper Limitation of a Pathname"),
                baseline_snapshot_id="baseline@1",
                candidate_snapshot_id="candidate@2",
                snapshot_root=self.snapshot_root,
                raw_paths=(CONTEXT_CLOSURE_NO_SEED_PATH,),
                size_budget_bytes=4096,
            )

    def test_ec7_no_seed_sentinel_shape_is_exact(self) -> None:
        omission = build_context_closure_no_seed_omission()

        self.assertEqual(omission.path, CONTEXT_CLOSURE_NO_SEED_PATH)
        self.assertEqual(omission.reason, CONTEXT_CLOSURE_NO_SEED_REASON)
        self.assertTrue(omission.detail)

    def test_ec8_validate_context_closure_seed_path_rejects_non_canonical_relative_paths(self) -> None:
        with self.assertRaises(PacketValidationError):
            validate_context_closure_seed_path("../escape.rs")

    def test_ec9_no_seed_omission_rejects_blank_detail(self) -> None:
        with self.assertRaises(PacketValidationError):
            build_context_closure_no_seed_omission("   ")

    def test_ec10_reserved_seed_name_is_rejected_even_when_file_is_missing(self) -> None:
        with self.assertRaises(PacketValidationError):
            canonicalize_context_closure_seed_path("./__seed__", self.snapshot_root)

    def test_ec11_invalid_seed_paths_return_none_and_order_rejects_non_canonical_entries(self) -> None:
        self.assertIsNone(canonicalize_context_closure_seed_path("/tmp/escape.rs", self.snapshot_root))
        self.assertIsNone(canonicalize_context_closure_seed_path("../escape.rs", self.snapshot_root))

        with self.assertRaises(PacketValidationError):
            deterministic_context_closure_seed_order(("python/checks.py", None))


if __name__ == "__main__":
    unittest.main()
