#!/usr/bin/env python3
"""Unit tests for boundary.py — adversarial fixtures per ADR-036 §3."""

import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import boundary as b
from run_local_task import BoundaryViolation


class HP1InScopePassesThrough(unittest.TestCase):
    def test_relative_write_within_worktree_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_write("hello.txt")
            boundary.check_write("nested/dir/file.txt")

    def test_allowlisted_command_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["cargo", "test", "-p", "domain"])
            boundary.check_command(["npm", "test"])
            boundary.check_command(["make", "qa-fmt"])

    def test_grep_and_python_unittest_pass(self):
        # added after a real pilot run: the model reasonably wanted to grep
        # a file before editing it, and to run a card's own
        # `python3 -m unittest ...` verify_commands directly.
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["grep", "-n", "foo", "src/lib.rs"])
            boundary.check_command(["python3", "-m", "unittest", "scripts/foo_test.py"])

    def test_cargo_metadata_passes(self):
        # added after a real pilot run: the model reasonably wanted to
        # inspect the workspace/crate structure via `cargo metadata` before
        # editing Rust code — read-only, no compilation or writes.
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["cargo", "metadata", "--format-version", "1"])


class EC1PathEscapeAttempts(unittest.TestCase):
    def test_absolute_path_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_write("/etc/passwd")

    def test_dotdot_traversal_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_write("../../etc/passwd")

    def test_dotdot_traversal_disguised_within_path_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_write("nested/../../outside.txt")

    def test_symlink_escape_rejected(self):
        with tempfile.TemporaryDirectory() as outside:
            with tempfile.TemporaryDirectory() as tmp:
                target = os.path.join(outside, "secret.txt")
                with open(target, "w", encoding="utf-8") as f:
                    f.write("outside data")

                link_path = os.path.join(tmp, "escape-link")
                os.symlink(target, link_path)

                boundary = b.LocalAgentBoundary(tmp)
                with self.assertRaises(BoundaryViolation):
                    boundary.check_write("escape-link")

    def test_symlink_swapped_in_after_first_check_still_caught(self):
        # TOCTOU: a symlink pointing inside the worktree at construction time
        # is swapped to point outside before the actual write would occur.
        # check_write must re-resolve at call time, not cache an earlier
        # resolution, so calling it again after the swap must still reject.
        with tempfile.TemporaryDirectory() as outside:
            with tempfile.TemporaryDirectory() as tmp:
                inside_target = os.path.join(tmp, "inside.txt")
                with open(inside_target, "w", encoding="utf-8") as f:
                    f.write("inside data")

                link_path = os.path.join(tmp, "swappable-link")
                os.symlink(inside_target, link_path)

                boundary = b.LocalAgentBoundary(tmp)
                boundary.check_write("swappable-link")  # passes: points inside

                outside_target = os.path.join(outside, "secret.txt")
                with open(outside_target, "w", encoding="utf-8") as f:
                    f.write("outside data")
                os.remove(link_path)
                os.symlink(outside_target, link_path)

                with self.assertRaises(BoundaryViolation):
                    boundary.check_write("swappable-link")


class EC2AllowlistedCommandArgumentEscape(unittest.TestCase):
    # D14 finding (medium severity): check_command only inspected
    # argv[0]/argv[1] against fixed prefixes — an allowlisted command's own
    # arguments (e.g. --manifest-path, -C) were never checked, letting an
    # otherwise-permitted command act outside the worktree via its flags.
    def test_cargo_manifest_path_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["cargo", "build", "--manifest-path", "/etc/passwd"])

    def test_cargo_manifest_path_escape_rejected_with_equals_form(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["cargo", "build", "--manifest-path=/etc/passwd"])

    def test_make_dash_c_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["make", "qa-fmt", "-C", "/"])

    def test_cargo_manifest_path_within_worktree_still_allowed(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["cargo", "build", "--manifest-path", "Cargo.toml"])

    def test_cargo_metadata_manifest_path_escape_rejected(self):
        # cargo metadata accepts --manifest-path too (confirmed via
        # `cargo metadata --help`) — the shared PATH_ACCEPTING_FLAGS check
        # must cover it the same as cargo build/test.
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["cargo", "metadata", "--manifest-path", "/etc/passwd"])

    # D14 finding (blocking): grep's file operands are bare positional
    # arguments, not named flags — PATH_ACCEPTING_FLAGS never inspected them,
    # so any of these passed check_command completely unchecked before the
    # fix, letting the model read arbitrary files anywhere on disk.
    def test_grep_absolute_path_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["grep", "root", "/etc/passwd"])

    def test_grep_recursive_absolute_path_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["grep", "-r", "foo", "/"])

    def test_grep_pattern_file_flag_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["grep", "-f", "/etc/passwd", "x"])

    def test_grep_dotdot_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["grep", "foo", "../../etc/passwd"])

    def test_grep_within_worktree_still_allowed(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["grep", "-n", "foo", "src/lib.rs"])
            boundary.check_command(["grep", "-r", "foo", "."])

    # D14 finding (major): unittest discover's -s/--start-directory and
    # -t/--top-level-directory take an arbitrary directory, unconstrained by
    # the bare 3-token prefix match, letting the model run test discovery
    # (i.e. execute arbitrary Python modules) rooted outside the worktree.
    def test_unittest_discover_start_directory_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(
                    ["python3", "-m", "unittest", "discover", "-s", "/etc", "-p", "*.py"]
                )

    def test_unittest_discover_top_level_directory_escape_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(
                    ["python3", "-m", "unittest", "discover", "-t", "/"]
                )

    def test_unittest_discover_within_worktree_still_allowed(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["python3", "-m", "unittest", "discover", "-s", "."])


class EC2DenylistAndUnknownCommands(unittest.TestCase):
    def test_git_push_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["git", "push", "origin", "main"])

    def test_git_push_embedded_in_shell_string_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["sh", "-c", "git push origin main"])

    def test_git_push_with_double_space_or_extra_quoting_still_rejected(self):
        # regression: a literal-substring check (" git push", "git push ")
        # is defeated by irregular whitespace or quoting; token-subsequence
        # matching via shlex must still catch these.
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["sh", "-c", "git  push origin main"])
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["sh", "-c", "'git' 'push' origin"])

    def test_rm_rf_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["rm", "-rf", "/"])

    def test_docker_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["docker", "compose", "up"])

    def test_unknown_command_not_on_either_list_rejected_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["curl", "https://example.com"])

    def test_make_non_qa_target_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["make", "release"])

    def test_python_without_m_unittest_still_rejected(self):
        # the allowlist entry is specifically ("python3", "-m", "unittest"),
        # not bare "python3" — arbitrary script execution must stay denied.
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["python3", "arbitrary_script.py"])
            with self.assertRaises(BoundaryViolation):
                boundary.check_command(["python3", "-c", "import os; os.system('rm -rf /')"])

    def test_empty_command_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command([])


class EC3EnvironmentStripping(unittest.TestCase):
    def test_only_allowed_vars_and_path_pass_through(self):
        source_env = {
            "PATH": "/usr/bin:/bin",
            "OLLAMA_HOST": "http://localhost:11434",
            "DUBBRIDGE_ENV": "local",
            "AWS_SECRET_ACCESS_KEY": "leaked-secret-value",
            "DATABASE_URL": "postgres://prod-host/db",
            "GITHUB_TOKEN": "ghp_leaked",
        }
        result = b.stripped_agent_env(source_env)

        self.assertEqual(result["PATH"], "/usr/bin:/bin")
        self.assertEqual(result["OLLAMA_HOST"], "http://localhost:11434")
        self.assertEqual(result["DUBBRIDGE_ENV"], "local")
        self.assertNotIn("AWS_SECRET_ACCESS_KEY", result)
        self.assertNotIn("DATABASE_URL", result)

    def test_dubbridge_prefixed_credential_vars_do_not_pass_through(self):
        # D14 finding (high severity): a blanket "DUBBRIDGE_" prefix
        # allowlist would forward real credential-bearing operator env vars
        # that share the prefix (see crates/config/src/lib.rs). Only the
        # single named DUBBRIDGE_ENV variable may pass; any other
        # DUBBRIDGE_*-prefixed variable, however plausible, must not.
        source_env = {
            "DUBBRIDGE_ENV": "local",
            "DUBBRIDGE_AUTH_JWT_SECRET": "prod-jwt-secret-should-not-leak",
            "DUBBRIDGE_DATABASE_URL": "postgres://prod/should-not-leak",
            "DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET": "oauth-secret-should-not-leak",
        }
        result = b.stripped_agent_env(source_env)

        self.assertEqual(result["DUBBRIDGE_ENV"], "local")
        self.assertNotIn("DUBBRIDGE_AUTH_JWT_SECRET", result)
        self.assertNotIn("DUBBRIDGE_DATABASE_URL", result)
        self.assertNotIn("DUBBRIDGE_GATEWAY__OAUTH__CLIENT_SECRET", result)
        self.assertNotIn("GITHUB_TOKEN", result)

    def test_no_production_descriptor_reaches_stripped_env(self):
        # adversarial: a plausible-looking production env var must not leak
        # even if named similarly to an allowed prefix.
        source_env = {
            "DUBBRIDGE_ENV": "local",
            "DUBBRIDGEPROD_SECRET": "should-not-match-prefix-loosely",
            "PRODUCTION_DUBBRIDGE_URL": "should-not-match-either",
        }
        result = b.stripped_agent_env(source_env)

        self.assertIn("DUBBRIDGE_ENV", result)
        self.assertNotIn("DUBBRIDGEPROD_SECRET", result)
        self.assertNotIn("PRODUCTION_DUBBRIDGE_URL", result)

    def test_env_probe_via_run_command_output_contains_no_secret(self):
        # EC-3 as an actual adversarial probe, not just unit inspection of
        # stripped_agent_env in isolation: simulate the model trying to read
        # back the environment through an allowed command and confirm the
        # transcript-visible output the runner would capture is clean.
        import subprocess

        source_env = {
            "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
            "OLLAMA_HOST": "http://localhost:11434",
            "DUBBRIDGE_ENV": "local",
            "GITHUB_TOKEN": "ghp_should_not_leak",
        }
        stripped = b.stripped_agent_env(source_env)
        completed = subprocess.run(
            ["env"], capture_output=True, text=True, env=stripped
        )
        self.assertNotIn("GITHUB_TOKEN", completed.stdout)
        self.assertNotIn("ghp_should_not_leak", completed.stdout)


if __name__ == "__main__":
    unittest.main(verbosity=2)
