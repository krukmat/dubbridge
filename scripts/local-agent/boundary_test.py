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

    def test_previously_allowlisted_commands_still_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["cargo", "test", "-p", "domain"])
            boundary.check_command(["npm", "test"])
            boundary.check_command(["make", "qa-fmt"])
            boundary.check_command(["grep", "-n", "foo", "src/lib.rs"])
            boundary.check_command(["python3", "-m", "unittest", "scripts/foo_test.py"])
            boundary.check_command(["cargo", "metadata", "--format-version", "1"])


class HP2UnrestrictedCommandsNoLongerRejectedByCheckCommand(unittest.TestCase):
    # T7b-3: these are representative commands that were previously rejected
    # by check_command's allowlist and caused real T7 aborts — a legitimate
    # development command being blocked, not an adversarial probe. This
    # class proves check_command itself no longer rejects them (unit level);
    # integration_test.py's RealBoundaryWiredIntoRunner proves a previously
    # denied command actually reaches real subprocess execution end-to-end.
    #
    # Security model note (two distinct containment layers, only one of
    # which is exercised here): command-execution containment is now
    # minimal — check_command no longer restricts which commands run, so a
    # permitted command can still read files or make network calls the
    # process itself can reach. Filesystem-write containment remains the
    # primary security gate and is unaffected by this class — it lives in
    # check_write's worktree jail (see EC1PathEscapeAttempts below) and in
    # the post-run diff-scope validation (T7c-a/b2/b3), neither of which
    # this task touches.
    def test_previously_unlisted_dev_commands_now_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["cargo", "run", "-p", "dubbridge-api"])
            boundary.check_command(["npm", "run", "screenshots"])
            boundary.check_command(["python3", "arbitrary_script.py"])
            boundary.check_command(["sh", "-c", "cargo build && cargo test"])
            boundary.check_command(["curl", "https://example.com"])
            boundary.check_command(["python3", "-m", "unittest", "discover", "-s", "."])

    def test_make_non_qa_target_now_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            boundary.check_command(["make", "release"])

    def test_empty_command_still_rejected(self):
        # the only remaining check_command rejection: an empty argv is not a
        # command-policy decision, it's a malformed-call guard the runner
        # itself relies on (see run_local_task.py's tool-call handling).
        with tempfile.TemporaryDirectory() as tmp:
            boundary = b.LocalAgentBoundary(tmp)
            with self.assertRaises(BoundaryViolation):
                boundary.check_command([])


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
