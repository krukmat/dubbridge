"""Unit tests for the T3c-2 governing security-boundary closure."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from scripts.antares.governing_boundary_closure import (
    GoverningBoundaryValidationError,
    resolve_governing_boundary_closure,
)
from scripts.antares.governing_boundary_map import (
    GOVERNING_BOUNDARY_MAP,
    GoverningBoundaryMapValidationError,
    validate_governing_boundary_map,
)

_ROOT = Path(__file__).with_name("testdata") / "governing_boundary_closure" / "snapshot"
_FIXTURES = Path(__file__).with_name("testdata") / "governing_boundary_closure"


def _load_fixture(name: str) -> dict:
    fixture = json.loads((_FIXTURES / name).read_text(encoding="utf-8"))
    if "governing_map_overrides" in fixture:
        fixture["governing_map_overrides"] = {
            root: tuple(targets)
            for root, targets in fixture["governing_map_overrides"].items()
        }
    return fixture


class HappyPathTest(unittest.TestCase):
    def test_hp1_closure_path_under_declared_boundary_returns_mapped_context(self) -> None:
        result = resolve_governing_boundary_closure(
            _ROOT,
            ("crates/db/src/lib.rs",),
            ("crates/db/",),
            GOVERNING_BOUNDARY_MAP,
        )
        self.assertEqual(
            result.included,
            (
                "crates/db/Cargo.toml",
                "docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md",
            ),
        )
        self.assertEqual(result.omitted, ())
        self.assertEqual(tuple(sorted(result.included)), result.included)

    def test_hp2_equivalent_permutations_are_byte_for_byte_equivalent(self) -> None:
        result_a = resolve_governing_boundary_closure(
            _ROOT,
            ("crates/db/src/lib.rs", "crates/db/Cargo.toml"),
            ("crates/db/",),
            GOVERNING_BOUNDARY_MAP,
        )
        result_b = resolve_governing_boundary_closure(
            _ROOT,
            ("crates/db/Cargo.toml", "crates/db/src/lib.rs"),
            ("crates/db/",),
            GOVERNING_BOUNDARY_MAP,
        )
        self.assertEqual(result_a.included, result_b.included)
        self.assertEqual(result_a.omitted, result_b.omitted)

    def test_hp2_cross_boundary_duplicate_context_path_emitted_once(self) -> None:
        fixture = _load_fixture("ec4_duplicate_context_paths.json")
        governing_map = dict(fixture["governing_map_overrides"])
        result = resolve_governing_boundary_closure(
            _ROOT,
            tuple(fixture["t3c1_closure_paths"]),
            tuple(fixture["boundary_roots"]),
            governing_map,
        )
        self.assertEqual(result.included, tuple(fixture["expected_included"]))
        self.assertEqual(len(result.included), len(set(result.included)))
        self.assertEqual(result.omitted, ())


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_uncovered_closure_path_yields_exactly_one_omission(self) -> None:
        fixture = _load_fixture("ec1_missing_boundary_root.json")
        result = resolve_governing_boundary_closure(
            _ROOT,
            tuple(fixture["t3c1_closure_paths"]),
            tuple(fixture["boundary_roots"]),
            GOVERNING_BOUNDARY_MAP,
        )
        self.assertEqual(result.included, ())
        self.assertEqual(len(result.omitted), 1)
        self.assertEqual(result.omitted[0].path, fixture["expected_omitted"][0]["path"])
        self.assertEqual(
            result.omitted[0].reason, fixture["expected_omitted"][0]["reason"]
        )

    def test_ec1_never_widens_to_nearest_parent_or_whole_repo(self) -> None:
        # crates/ is not declared, only crates/db/ -- a path under an
        # undeclared ancestor must omit, never fall back to a wider scan.
        result = resolve_governing_boundary_closure(
            _ROOT,
            ("crates/storage/src/lib.rs",),
            ("crates/db/",),
            GOVERNING_BOUNDARY_MAP,
        )
        self.assertEqual(result.included, ())
        self.assertEqual(len(result.omitted), 1)
        self.assertEqual(result.omitted[0].path, "crates/storage/src/lib.rs")

    def test_ec2_overlapping_roots_resolve_by_longest_prefix(self) -> None:
        fixture = _load_fixture("ec2_overlapping_roots.json")
        governing_map = dict(fixture["governing_map_overrides"])
        result = resolve_governing_boundary_closure(
            _ROOT,
            tuple(fixture["t3c1_closure_paths"]),
            tuple(fixture["boundary_roots"]),
            governing_map,
        )
        self.assertEqual(result.included, tuple(fixture["expected_included"]))
        self.assertEqual(result.omitted, ())

    def test_ec2_distinct_keys_sharing_a_target_are_not_a_conflict(self) -> None:
        # Two different, individually-valid boundary roots legitimately
        # sharing a governing-context target (e.g. docs/architecture.md) is
        # not a conflict -- HP-2 / EC-4 require the resolver to deduplicate
        # the shared target downstream, not reject it at validation time.
        shared_target_map = {
            "crates/db/": ("crates/db/Cargo.toml",),
            "crates/storage/": ("crates/db/Cargo.toml",),
        }
        validated = validate_governing_boundary_map(
            shared_target_map, snapshot_root=_ROOT
        )
        self.assertEqual(
            validated.entries["crates/db/"], ("crates/db/Cargo.toml",)
        )
        self.assertEqual(
            validated.entries["crates/storage/"], ("crates/db/Cargo.toml",)
        )

    def test_ec2_canonicalization_collision_between_keys_rejected_at_validation(
        self,
    ) -> None:
        # Two distinct raw keys that normalize to the same canonical
        # boundary root are a genuine conflicting/duplicate mapping entry.
        conflicting_map = {
            "crates/db/": ("crates/db/Cargo.toml",),
            "crates/db//": ("docs/adr/ADR-008-rights-ledger-fail-closed-precondition.md",),
        }
        with self.assertRaises(GoverningBoundaryMapValidationError) as ctx:
            validate_governing_boundary_map(conflicting_map, snapshot_root=_ROOT)
        self.assertEqual(ctx.exception.reason, "conflicting_target_mapping")

    def test_ec3_missing_governing_context_target_raises_typed_error(self) -> None:
        fixture = _load_fixture("ec3_invalid_target.json")
        governing_map = dict(fixture["governing_map_overrides"])
        with self.assertRaises(GoverningBoundaryValidationError) as ctx:
            resolve_governing_boundary_closure(
                _ROOT,
                tuple(fixture["t3c1_closure_paths"]),
                tuple(fixture["boundary_roots"]),
                governing_map,
            )
        self.assertEqual(ctx.exception.reason, fixture["expected_error_reason"])

    def test_ec3_non_canonical_target_raises_typed_error_no_partial_result(self) -> None:
        bad_map = {"crates/db/": ("../outside.md",)}
        with self.assertRaises(GoverningBoundaryMapValidationError) as ctx:
            validate_governing_boundary_map(bad_map, snapshot_root=_ROOT)
        self.assertEqual(ctx.exception.reason, "non_canonical_target")

    def test_ec3_outside_snapshot_target_raises_typed_error(self) -> None:
        # A canonical-looking relative path that still escapes the snapshot
        # via a symlink-free lexical parent is rejected structurally by
        # _is_canonical_target (no '..' allowed at all), so this exercises
        # the missing-file path instead: a target the validator resolves
        # inside the snapshot but that does not exist there.
        bad_map = {"crates/db/": ("does/not/exist.md",)}
        with self.assertRaises(GoverningBoundaryMapValidationError) as ctx:
            validate_governing_boundary_map(bad_map, snapshot_root=_ROOT)
        self.assertEqual(ctx.exception.reason, "missing_target")

    def test_ec4_duplicate_closure_paths_under_same_boundary_stay_in_sync(self) -> None:
        result = resolve_governing_boundary_closure(
            _ROOT,
            ("crates/db/src/lib.rs", "crates/db/src/lib.rs"),
            ("crates/db/",),
            GOVERNING_BOUNDARY_MAP,
        )
        self.assertEqual(len(result.included), len(set(result.included)))
        self.assertEqual(tuple(sorted(result.included)), result.included)

    def test_ec4_provenance_and_ordering_stay_deterministic_across_calls(self) -> None:
        fixture = _load_fixture("ec4_duplicate_context_paths.json")
        governing_map = dict(fixture["governing_map_overrides"])
        result_a = resolve_governing_boundary_closure(
            _ROOT,
            tuple(fixture["t3c1_closure_paths"]),
            tuple(fixture["boundary_roots"]),
            governing_map,
        )
        result_b = resolve_governing_boundary_closure(
            _ROOT,
            tuple(reversed(fixture["t3c1_closure_paths"])),
            tuple(reversed(fixture["boundary_roots"])),
            governing_map,
        )
        self.assertEqual(result_a.included, result_b.included)
        self.assertEqual(result_a.omitted, result_b.omitted)


class UndeclaredBoundaryRootTest(unittest.TestCase):
    def test_ec_boundary_root_absent_from_committed_map_raises_typed_error(self) -> None:
        with self.assertRaises(GoverningBoundaryValidationError) as ctx:
            resolve_governing_boundary_closure(
                _ROOT,
                ("crates/db/src/lib.rs",),
                ("crates/nonexistent/",),
                GOVERNING_BOUNDARY_MAP,
            )
        self.assertEqual(ctx.exception.reason, "undeclared_boundary_root")


if __name__ == "__main__":
    unittest.main()
