"""Unit/characterization tests for the T3c-1 dependency and manifest closure."""

from __future__ import annotations

import socket
import unittest
from pathlib import Path
from unittest import mock

from scripts.antares.context_closure import (
    ContextClosureResolutionError,
    compute_context_closure,
)
from scripts.antares.packet_schema import (
    CONTEXT_CLOSURE_NO_SEED_PATH,
    CONTEXT_CLOSURE_NO_SEED_REASON,
)

_ROOT = Path(__file__).with_name("testdata") / "context_closure_dependency_manifest" / "basic_snapshot"


class HappyPathTest(unittest.TestCase):
    def test_hp1_rust_mod_closure_and_cargo_manifest_are_deterministic(self) -> None:
        result = compute_context_closure(_ROOT, ("rust_pkg/src/main.rs",))
        self.assertEqual(
            result.included,
            ("rust_pkg/Cargo.toml", "rust_pkg/src/main.rs", "rust_pkg/src/util/mod.rs"),
        )
        self.assertEqual(result.omitted, ())

    def test_hp1_rust_use_statements_do_not_create_file_edges(self) -> None:
        # rust_pkg/src/main.rs has no `use` statements pulling in extra files;
        # util/mod.rs is reached only via `mod util;`. Confirm no unrelated
        # sibling file is pulled in by a `use` reference.
        result = compute_context_closure(_ROOT, ("rust_pkg/src/main.rs",))
        self.assertNotIn("rust_pkg/src/lib.rs", result.included)

    def test_hp1_cargo_path_dependency_closure_is_canonical(self) -> None:
        result = compute_context_closure(_ROOT, ("pathdep_pkg/src/main.rs",))
        self.assertEqual(
            result.included,
            (
                "dep_target/Cargo.toml",
                "dep_target/src/lib.rs",
                "pathdep_pkg/Cargo.toml",
                "pathdep_pkg/src/main.rs",
            ),
        )
        for path in result.included:
            self.assertNotIn("..", path)

    def test_hp2_python_relative_import_closure_follows_fixed_mapping(self) -> None:
        result = compute_context_closure(_ROOT, ("py_pkg/sub/mod_a.py",))
        self.assertEqual(result.included, ("py_pkg/sub/mod_a.py", "py_pkg/sub/mod_b.py"))
        self.assertEqual(result.omitted, ())

    def test_hp2_python_absolute_import_resolves_full_dotted_path(self) -> None:
        result = compute_context_closure(_ROOT, ("absmain2/main.py",))
        self.assertEqual(result.included, ("absmain2/main.py", "top_pkg/mod_y.py"))

    def test_hp2_external_stdlib_and_plain_directory_imports_are_ignored(self) -> None:
        # absmain2/main.py also imports plain_dir_no_init.thing (no top-level
        # __init__.py) and py_pkg/sub/mod_a.py imports `os` (stdlib). Neither
        # should ever appear in a closure result.
        result = compute_context_closure(_ROOT, ("absmain2/main.py",))
        self.assertNotIn("plain_dir_no_init/thing.py", result.included)
        result2 = compute_context_closure(_ROOT, ("py_pkg/sub/mod_a.py",))
        self.assertFalse(any("os" in p for p in result2.included if p.endswith("os.py")))

    def test_hp3_seed_permutations_produce_byte_for_byte_equivalent_output(self) -> None:
        seeds_a = ("rust_pkg/src/main.rs", "py_pkg/sub/mod_a.py")
        seeds_b = ("py_pkg/sub/mod_a.py", "rust_pkg/src/main.rs")
        result_a = compute_context_closure(_ROOT, seeds_a)
        result_b = compute_context_closure(_ROOT, seeds_b)
        self.assertEqual(result_a.included, result_b.included)
        self.assertEqual(result_a.omitted, result_b.omitted)

    def test_hp3_duplicate_paths_are_emitted_once_and_sorted(self) -> None:
        result = compute_context_closure(
            _ROOT, ("rust_pkg/src/main.rs", "rust_pkg/src/main.rs")
        )
        self.assertEqual(len(result.included), len(set(result.included)))
        self.assertEqual(tuple(sorted(result.included)), result.included)

    def test_hp3_cycle_resolves_each_edge_once_and_remains_deterministic(self) -> None:
        result = compute_context_closure(_ROOT, ("cycle/a.rs",))
        self.assertEqual(result.included, ("cycle/a.rs", "cycle/b.rs"))
        self.assertEqual(result.omitted, ())


class EdgeCaseTest(unittest.TestCase):
    def test_ec1_empty_seeds_produce_exactly_the_frozen_no_seed_omission(self) -> None:
        result = compute_context_closure(_ROOT, ())
        self.assertEqual(result.included, ())
        self.assertEqual(len(result.omitted), 1)
        self.assertEqual(result.omitted[0].path, CONTEXT_CLOSURE_NO_SEED_PATH)
        self.assertEqual(result.omitted[0].reason, CONTEXT_CLOSURE_NO_SEED_REASON)

    def test_ec2_unsupported_file_type_produces_frozen_omission(self) -> None:
        result = compute_context_closure(_ROOT, ("docs/spec.yaml",))
        self.assertEqual(result.included, ())
        self.assertEqual(len(result.omitted), 1)
        self.assertEqual(result.omitted[0].path, "docs/spec.yaml")
        self.assertEqual(result.omitted[0].reason, "context_closure_unsupported_file_type")

    def test_ec3_expansion_limit_stops_before_next_pending_source(self) -> None:
        result = compute_context_closure(_ROOT, ("cycle/a.rs", "cycle/b.rs"), expansion_limit=1)
        self.assertEqual(result.included, ("cycle/a.rs",))
        self.assertEqual(len(result.omitted), 1)
        self.assertEqual(result.omitted[0].path, "cycle/b.rs")
        self.assertEqual(result.omitted[0].reason, "context_closure_expansion_limit_reached")

    def test_ec4_containment_escape_is_a_soft_omission_with_absolute_path(self) -> None:
        result = compute_context_closure(_ROOT, ("../outside.rs",))
        self.assertEqual(result.included, ())
        self.assertEqual(len(result.omitted), 1)
        omission = result.omitted[0]
        self.assertEqual(omission.reason, "path_outside_snapshot")
        self.assertTrue(Path(omission.path).is_absolute())

    def test_ec5_missing_seed_raises_typed_error_with_no_partial_result(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("does/not/exist.rs",))
        self.assertEqual(ctx.exception.reason, "missing_seed")

    def test_ec5_unresolved_rust_mod_raises_typed_error(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("missing_mod/a.rs",))
        self.assertEqual(ctx.exception.reason, "unresolved_rust_mod")

    def test_ec5_unresolved_relative_import_without_package_ancestor_raises(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("no_pkg_relative/a.py",))
        self.assertEqual(ctx.exception.reason, "unresolved_relative_import")

    def test_ec5_unresolved_absolute_import_under_local_package_raises(self) -> None:
        # top_pkg/__init__.py exists, so top_pkg.* is local, but
        # top_pkg.missing_module does not resolve to any file.
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("absmain_unresolved/main.py",))
        self.assertEqual(ctx.exception.reason, "unresolved_absolute_import")

    def test_ec5_malformed_manifest_raises_typed_error(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("malformed_pkg/src/main.rs",))
        self.assertEqual(ctx.exception.reason, "malformed_manifest")

    def test_ec5_ambiguous_python_module_raises_typed_error(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("absmain3_ambiguous_probe.py",))

    def test_ec6_cycle_expands_each_edge_at_most_once(self) -> None:
        result = compute_context_closure(_ROOT, ("cycle/b.rs",))
        self.assertEqual(result.included, ("cycle/a.rs", "cycle/b.rs"))

    def test_ec6_unresolved_edge_still_raises_despite_a_coexisting_cycle(self) -> None:
        # cycle_with_error/b.rs both cycles back to a.rs (mod a;, a visited
        # back-edge that must be ignored) and declares mod missing_sibling;
        # (a genuinely unresolved edge). EC-6 requires every encountered
        # edge to be resolved before a visited back-edge is ignored, so the
        # unresolved edge must still surface as EC-5, not be masked by the
        # cycle short-circuit.
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("cycle_with_error/a.rs",))
        self.assertEqual(ctx.exception.reason, "unresolved_rust_mod")

    def test_ec7_manifest_ancestor_discovery_finds_all_allowlisted_manifests(self) -> None:
        result = compute_context_closure(_ROOT, ("py_manifests/mod_c.py",))
        self.assertIn("py_manifests/pyproject.toml", result.included)
        self.assertIn("py_manifests/setup.cfg", result.included)
        self.assertIn("py_manifests/requirements.txt", result.included)
        self.assertIn("py_manifests/requirements-dev.txt", result.included)

    def test_ec7_workspace_only_cargo_manifest_is_context_only(self) -> None:
        result = compute_context_closure(_ROOT, ("workspace_root/member/src/main.rs",))
        self.assertEqual(
            result.included,
            (
                "workspace_root/Cargo.toml",
                "workspace_root/member/Cargo.toml",
                "workspace_root/member/src/main.rs",
            ),
        )

    def test_ec7_empty_cargo_manifest_is_a_context_only_no_op(self) -> None:
        result = compute_context_closure(_ROOT, ("empty_manifest_pkg/src/main.rs",))
        self.assertEqual(
            result.included,
            ("empty_manifest_pkg/Cargo.toml", "empty_manifest_pkg/src/main.rs"),
        )

    def test_ec7_autobins_false_excludes_automatic_bin_discovery(self) -> None:
        result = compute_context_closure(_ROOT, ("autobins_pkg/Cargo.toml",))
        self.assertEqual(
            result.included, ("autobins_pkg/Cargo.toml", "autobins_pkg/src/lib.rs")
        )
        self.assertNotIn("autobins_pkg/src/bin/tool.rs", result.included)

    def test_ec7_ambiguous_explicit_bin_without_path_raises(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("ambig_bin_pkg/Cargo.toml",))
        self.assertEqual(ctx.exception.reason, "ambiguous_cargo_bin")

    def test_ec7_explicit_bin_with_path_is_followed_verbatim(self) -> None:
        result = compute_context_closure(_ROOT, ("explicit_bin_path_pkg/Cargo.toml",))
        self.assertEqual(
            result.included,
            (
                "explicit_bin_path_pkg/Cargo.toml",
                "explicit_bin_path_pkg/tools/mytool/main.rs",
            ),
        )

    def test_ec7_explicit_lib_path_is_followed_instead_of_default(self) -> None:
        result = compute_context_closure(_ROOT, ("explicit_lib_pkg/Cargo.toml",))
        self.assertEqual(
            result.included,
            ("explicit_lib_pkg/Cargo.toml", "explicit_lib_pkg/custom_src/entry.rs"),
        )

    def test_ec7_cargo_lock_is_a_context_only_no_op(self) -> None:
        result = compute_context_closure(_ROOT, ("lockfile_pkg/src/main.rs",))
        self.assertEqual(
            result.included,
            ("lockfile_pkg/Cargo.lock", "lockfile_pkg/Cargo.toml", "lockfile_pkg/src/main.rs"),
        )

    def test_ec7_autobins_discovers_directory_form_bin_entrypoints(self) -> None:
        result = compute_context_closure(_ROOT, ("autobins_dir_pkg/Cargo.toml",))
        self.assertEqual(
            result.included,
            ("autobins_dir_pkg/Cargo.toml", "autobins_dir_pkg/src/bin/subtool/main.rs"),
        )

    def test_ec9_python_from_relative_import_multiple_names_resolves_each(self) -> None:
        result = compute_context_closure(_ROOT, ("py_multi_relative/main.py",))
        self.assertEqual(
            result.included,
            (
                "py_multi_relative/main.py",
                "py_multi_relative/mod_p.py",
                "py_multi_relative/mod_q.py",
            ),
        )

    def test_ec10_invalid_encoding_source_raises_typed_error(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("invalid_encoding/a.rs",))
        self.assertEqual(ctx.exception.reason, "invalid_manifest_encoding")

    def test_ec10_malformed_setup_cfg_raises_typed_error(self) -> None:
        with self.assertRaises(ContextClosureResolutionError) as ctx:
            compute_context_closure(_ROOT, ("malformed_setup_cfg/probe.py",))
        self.assertEqual(ctx.exception.reason, "malformed_manifest")

    def test_ec8_canonicalization_is_snapshot_relative_posix_and_case_sensitive(self) -> None:
        result = compute_context_closure(_ROOT, ("rust_pkg/src/main.rs",))
        for path in result.included:
            self.assertNotIn("\\", path)
            self.assertFalse(path.startswith("/"))

    def test_ec8_symlink_escape_is_a_soft_omission_not_an_exception(self) -> None:
        result = compute_context_closure(_ROOT, ("symlink_escape/escape.rs",))
        self.assertEqual(result.included, ())
        self.assertEqual(len(result.omitted), 1)
        omission = result.omitted[0]
        self.assertEqual(omission.reason, "path_outside_snapshot")
        self.assertTrue(Path(omission.path).is_absolute())

    def test_ec9_python_relative_import_always_local_and_fails_closed(self) -> None:
        with self.assertRaises(ContextClosureResolutionError):
            compute_context_closure(_ROOT, ("no_pkg_relative/a.py",))

    def test_ec9_plain_directory_absolute_import_is_external_not_a_failure(self) -> None:
        result = compute_context_closure(_ROOT, ("absmain2/main.py",))
        self.assertNotIn("plain_dir_no_init/thing.py", result.included)

    def test_ec10_setup_py_is_utf8_decoded_only_and_never_executed(self) -> None:
        # setup_py_probe/setup.py raises RuntimeError if executed; its
        # presence in the result with no exception proves it was only
        # UTF-8 decoded, never run.
        result = compute_context_closure(_ROOT, ("setup_py_probe/probe.py",))
        self.assertIn("setup_py_probe/setup.py", result.included)

    def test_network_primitive_sentinel_proves_local_only_behavior(self) -> None:
        """Replace socket.socket with a failing sentinel and prove the
        closure never invokes it (EC-10: no network access)."""

        def _forbidden(*_args: object, **_kwargs: object) -> None:
            raise AssertionError("context_closure must never touch the network")

        with mock.patch.object(socket, "socket", side_effect=_forbidden):
            result = compute_context_closure(_ROOT, ("rust_pkg/src/main.rs",))
            self.assertEqual(
                result.included,
                ("rust_pkg/Cargo.toml", "rust_pkg/src/main.rs", "rust_pkg/src/util/mod.rs"),
            )


if __name__ == "__main__":
    unittest.main()
