#!/usr/bin/env python3
"""Tests for agent-preflight.py."""
from __future__ import annotations

import hashlib
import importlib.util
import io
import json
import os
import stat
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from unittest import mock


SPEC = importlib.util.spec_from_file_location(
    "agent_preflight",
    Path(__file__).parent / "agent-preflight.py",
)
agent_preflight = importlib.util.module_from_spec(SPEC)  # type: ignore[arg-type]
SPEC.loader.exec_module(agent_preflight)  # type: ignore[union-attr]


class AgentPreflightTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_hp1_mark_then_check_passes(self):
        path = agent_preflight.mark_preflight(self.root)

        self.assertTrue(path.exists())
        data = agent_preflight.check_preflight(self.root)

        self.assertEqual(data["repo_root"], str(self.root.resolve()))
        self.assertEqual(data["version"], agent_preflight.SCRIPT_VERSION)

    def test_hp2_summary_names_required_workflow_rules(self):
        summary = agent_preflight.preflight_summary()

        self.assertIn("AGENT_WORKFLOW_GUIDE.md", summary)
        self.assertIn("docs/architecture.md", summary)
        self.assertIn("ADRs", summary)
        self.assertIn("docs/plan/roadmap.md", summary)
        self.assertIn("BDD/product docs", summary)
        self.assertIn("scripts/rri.py", summary)
        self.assertIn("RRI 26+", summary)
        self.assertIn("DESIGN.md", summary)
        self.assertIn("Gemma Reviewer / D14", summary)

    def test_ec1_check_fails_when_sentinel_missing(self):
        with self.assertRaises(agent_preflight.PreflightError) as ctx:
            agent_preflight.check_preflight(self.root)

        self.assertIn("Missing", str(ctx.exception))
        self.assertIn("--mark", str(ctx.exception))

    def test_ec2_check_fails_for_different_repo_root(self):
        other_root = self.root / "other"
        other_root.mkdir()
        agent_preflight.mark_preflight(other_root)
        sentinel = agent_preflight.sentinel_path(other_root)
        local_sentinel = agent_preflight.sentinel_path(self.root)
        local_sentinel.parent.mkdir(parents=True, exist_ok=True)
        local_sentinel.write_text(sentinel.read_text(encoding="utf-8"), encoding="utf-8")

        with self.assertRaises(agent_preflight.PreflightError) as ctx:
            agent_preflight.check_preflight(self.root)

        self.assertIn("was marked for", str(ctx.exception))
        self.assertIn(str(self.root.resolve()), str(ctx.exception))

    def test_cli_check_returns_nonzero_without_sentinel(self):
        result = agent_preflight.main(["--repo-root", str(self.root), "--check"])

        self.assertEqual(result, 1)

    def test_cli_mark_and_check_returns_zero(self):
        mark_result = agent_preflight.main(["--repo-root", str(self.root), "--mark"])
        check_result = agent_preflight.main(["--repo-root", str(self.root), "--check"])

        self.assertEqual(mark_result, 0)
        self.assertEqual(check_result, 0)


class AgentPreflightV2ReceiptTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.instruction_path = self.root / "INSTRUCTIONS.md"
        self.instruction_path.write_text("do the task\n", encoding="utf-8")
        self.doc_path = self.root / "docs" / "notes.md"
        self.doc_path.parent.mkdir(parents=True, exist_ok=True)
        self.doc_path.write_text("some notes\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def _build_payload(self, **overrides):
        kwargs = dict(
            provider="claude",
            session_id="session-1",
            actor_id="actor-1",
            repo_root=self.root,
            hook_event_name="startup",
            source="hook",
            transcript_path="/tmp/transcript.json",
            native_instruction_mechanism="hook",
            native_instruction_path="INSTRUCTIONS.md",
            document_paths=["docs/notes.md"],
        )
        kwargs.update(overrides)
        return agent_preflight.build_v2_receipt_payload(**kwargs)

    def test_hp1_valid_claude_payload_has_correct_hashes_and_byte_counts(self):
        payload = self._build_payload()

        instruction_bytes = self.instruction_path.read_bytes()
        doc_bytes = self.doc_path.read_bytes()

        self.assertEqual(payload["schema_version"], 2)
        self.assertEqual(
            payload["native_instruction"]["sha256"],
            hashlib.sha256(instruction_bytes).hexdigest(),
        )
        self.assertEqual(payload["native_instruction"]["bytes"], len(instruction_bytes))
        self.assertEqual(len(payload["documents"]), 1)
        self.assertEqual(
            payload["documents"][0]["sha256"], hashlib.sha256(doc_bytes).hexdigest()
        )
        self.assertEqual(payload["documents"][0]["bytes"], len(doc_bytes))
        self.assertEqual(payload["lifecycle"]["hook_event_name"], "startup")

    def test_hp2_different_identity_tuples_yield_different_hashes(self):
        identity_a = agent_preflight.compute_receipt_identity("claude", "session-1", "actor-1")
        identity_b = agent_preflight.compute_receipt_identity("claude", "session-2", "actor-1")
        identity_c = agent_preflight.compute_receipt_identity("codex", "session-1", "actor-1")

        self.assertNotEqual(identity_a, identity_b)
        self.assertNotEqual(identity_a, identity_c)

    def test_ec1_unsupported_provider_rejected_before_payload_construction(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            self._build_payload(provider="other")

        self.assertFalse((self.root / "other-marker").exists())

    def test_ec1_unsupported_lifecycle_event_rejected(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            self._build_payload(hook_event_name="not_a_real_event")

    def test_ec2_malformed_session_id_rejected_without_path_derivation(self):
        for bad_value in ["", "has\x00nul", "has/slash", "has..dotdot"]:
            with self.assertRaises(agent_preflight.ReceiptValidationError):
                self._build_payload(session_id=bad_value)

    def test_ec2_malformed_actor_id_rejected(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            self._build_payload(actor_id="../escape")

    def test_ec3_legacy_v1_sentinel_rejected_with_distinct_error(self):
        legacy_payload = agent_preflight.sentinel_payload(self.root)

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(legacy_payload)

        self.assertIn("legacy v1 sentinel", str(ctx.exception))

    def test_ec3_valid_v2_payload_passes_validation(self):
        payload = self._build_payload()

        agent_preflight.validate_v2_receipt_payload(payload)

    def test_ec4_missing_native_instruction_file_fails_closed(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            self._build_payload(native_instruction_path="missing.md")

        self.assertIn("missing.md", str(ctx.exception))

    def test_ec4_missing_document_file_fails_closed(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            self._build_payload(document_paths=["docs/notes.md", "docs/missing.md"])

        self.assertIn("docs/missing.md", str(ctx.exception))

    def test_validate_v2_receipt_payload_rejects_non_dict(self):
        with self.assertRaises(agent_preflight.ReceiptValidationError):
            agent_preflight.validate_v2_receipt_payload("not-a-dict")

    def test_validate_v2_receipt_payload_rejects_wrong_schema_version(self):
        payload = self._build_payload()
        payload["schema_version"] = 1

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(payload)

        self.assertIn("schema_version", str(ctx.exception))

    def test_validate_v2_receipt_payload_rejects_non_dict_lifecycle(self):
        payload = self._build_payload()
        payload["lifecycle"] = "not-a-dict"

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(payload)

        self.assertIn("lifecycle", str(ctx.exception))

    def test_validate_v2_receipt_payload_rejects_missing_required_key(self):
        payload = self._build_payload()
        del payload["documents"]

        with self.assertRaises(agent_preflight.ReceiptValidationError) as ctx:
            agent_preflight.validate_v2_receipt_payload(payload)

        self.assertIn("documents", str(ctx.exception))


@unittest.skipIf(sys.platform.startswith("win"), "POSIX permission semantics required")
class AgentPreflightV2ReceiptPublishTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.instruction_path = self.root / "INSTRUCTIONS.md"
        self.instruction_path.write_text("do the task\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def _payload(self, **overrides):
        kwargs = dict(
            provider="claude",
            session_id="session-1",
            actor_id="actor-1",
            repo_root=self.root,
            hook_event_name="startup",
            source="hook",
            transcript_path="/tmp/transcript.json",
            native_instruction_mechanism="hook",
            native_instruction_path="INSTRUCTIONS.md",
            document_paths=[],
        )
        kwargs.update(overrides)
        return agent_preflight.build_v2_receipt_payload(**kwargs)

    def test_hp1_valid_reload_atomically_replaces_prior_receipt(self):
        first = self._payload()
        path_a = agent_preflight.publish_v2_receipt(self.root, first)

        second = self._payload()
        path_b = agent_preflight.publish_v2_receipt(self.root, second)

        self.assertEqual(path_a, path_b)
        entries = list(path_b.parent.iterdir())
        self.assertEqual(entries, [path_b])

        on_disk = json.loads(path_b.read_text(encoding="utf-8"))
        self.assertEqual(on_disk["loaded_at"], second["loaded_at"])

    def test_hp1_receipt_dir_and_file_permissions_are_restrictive(self):
        payload = self._payload()
        path = agent_preflight.publish_v2_receipt(self.root, payload)

        dir_mode = stat.S_IMODE(path.parent.stat().st_mode)
        file_mode = stat.S_IMODE(path.stat().st_mode)

        self.assertEqual(dir_mode, 0o700)
        self.assertEqual(file_mode, 0o600)

    def test_hp2_concurrent_distinct_sessions_do_not_collide(self):
        errors: list[Exception] = []
        paths: list[Path] = []
        lock = threading.Lock()

        def publish(session_id: str) -> None:
            try:
                payload = self._payload(session_id=session_id)
                result = agent_preflight.publish_v2_receipt(self.root, payload)
                with lock:
                    paths.append(result)
            except Exception as exc:  # pragma: no cover - failure path asserted below
                with lock:
                    errors.append(exc)

        threads = [
            threading.Thread(target=publish, args=(f"session-{i}",)) for i in range(5)
        ]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()

        self.assertEqual(errors, [])
        self.assertEqual(len(paths), len(set(paths)))
        for path in paths:
            data = json.loads(path.read_text(encoding="utf-8"))
            self.assertEqual(data["schema_version"], 2)

    def test_hp2_concurrent_same_identity_never_collides_on_temp_name(self):
        errors: list[Exception] = []
        lock = threading.Lock()

        def publish() -> None:
            try:
                agent_preflight.publish_v2_receipt(self.root, self._payload())
            except Exception as exc:  # pragma: no cover - failure path asserted below
                with lock:
                    errors.append(exc)

        threads = [threading.Thread(target=publish) for _ in range(5)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()

        self.assertEqual(errors, [])
        target_path = agent_preflight.v2_receipt_path(self.root, "claude", "session-1", "actor-1")
        self.assertTrue(target_path.exists())
        entries = list(target_path.parent.iterdir())
        self.assertEqual(entries, [target_path])

    def test_ec1_interruption_before_replace_leaves_no_authorizing_receipt(self):
        payload = self._payload()
        target_path = agent_preflight.v2_receipt_path(
            self.root, payload["provider"], payload["session_id"], payload["actor_id"]
        )

        with mock.patch.object(
            agent_preflight.os, "replace", side_effect=InterruptedError("boom")
        ):
            with self.assertRaises(InterruptedError):
                agent_preflight.publish_v2_receipt(self.root, payload)

        self.assertFalse(target_path.exists())
        leftovers = [p for p in target_path.parent.iterdir() if p.name.endswith(".tmp")]
        self.assertEqual(leftovers, [])

    def test_ec1_interruption_does_not_resurrect_prior_receipt(self):
        first = self._payload()
        target_path = agent_preflight.publish_v2_receipt(self.root, first)
        first_contents = target_path.read_text(encoding="utf-8")

        second = self._payload()
        with mock.patch.object(
            agent_preflight.os, "replace", side_effect=InterruptedError("boom")
        ):
            with self.assertRaises(InterruptedError):
                agent_preflight.publish_v2_receipt(self.root, second)

        self.assertFalse(target_path.exists())
        self.assertNotEqual(first_contents, "")

    def test_ec1_failure_during_temp_write_cleans_up_and_denies_authorization(self):
        payload = self._payload()
        target_path = agent_preflight.v2_receipt_path(
            self.root, payload["provider"], payload["session_id"], payload["actor_id"]
        )

        with mock.patch.object(
            agent_preflight.os, "fsync", side_effect=OSError("disk full")
        ):
            with self.assertRaises(OSError):
                agent_preflight.publish_v2_receipt(self.root, payload)

        self.assertFalse(target_path.exists())
        leftovers = [p for p in target_path.parent.iterdir() if p.name.endswith(".tmp")]
        self.assertEqual(leftovers, [])

    def test_hp1_invalidate_prior_receipt_tolerates_toctou_race(self):
        target_path = self.root / "already-gone.json"
        target_path.parent.mkdir(parents=True, exist_ok=True)
        target_path.write_text("{}", encoding="utf-8")

        original_unlink = Path.unlink

        def flaky_unlink(self_path, *args, **kwargs):
            if self_path == target_path:
                original_unlink(self_path, *args, **kwargs)
                raise FileNotFoundError("already removed")
            return original_unlink(self_path, *args, **kwargs)

        with mock.patch.object(Path, "unlink", flaky_unlink):
            agent_preflight._invalidate_prior_receipt(target_path)

        self.assertFalse(target_path.exists())

    def test_hp1_chmod_failure_on_temp_file_does_not_block_publish(self):
        payload = self._payload()

        with mock.patch.object(agent_preflight.os, "chmod", side_effect=OSError("no chmod")):
            path = agent_preflight.publish_v2_receipt(self.root, payload)

        self.assertTrue(path.exists())
        data = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual(data["schema_version"], 2)

    def test_ec2_permission_error_denies_authorization_without_fallback(self):
        payload = self._payload()
        target_path = agent_preflight.v2_receipt_path(
            self.root, payload["provider"], payload["session_id"], payload["actor_id"]
        )

        with mock.patch.object(
            agent_preflight.os, "open", side_effect=PermissionError("denied")
        ):
            with self.assertRaises(PermissionError):
                agent_preflight.publish_v2_receipt(self.root, payload)

        self.assertFalse(target_path.exists())

    def test_ec2_malformed_payload_rejected_before_touching_disk(self):
        payload = self._payload()
        del payload["documents"]
        receipts_dir = agent_preflight.v2_receipts_dir(self.root)

        with self.assertRaises(agent_preflight.ReceiptValidationError):
            agent_preflight.publish_v2_receipt(self.root, payload)

        self.assertFalse(receipts_dir.exists())


class AgentPreflightCliV2CommandsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "CLAUDE.md").write_text("claude instructions\n", encoding="utf-8")
        (self.root / "AGENTS.override.md").write_text("codex instructions\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def _load_args(self, provider="claude", session_id="session-1", actor_id="claude-code"):
        return [
            "load",
            "--repo-root",
            str(self.root),
            "--provider",
            provider,
            "--session-id",
            session_id,
            "--actor-id",
            actor_id,
            "--hook-event-name",
            "startup",
            "--native-instruction-mechanism",
            "@import",
            "--native-instruction-path",
            "CLAUDE.md",
        ]

    def test_hp1_load_publishes_receipt_prints_summary_exits_zero(self):
        result = agent_preflight.main(self._load_args())

        self.assertEqual(result, 0)
        published = agent_preflight.v2_receipt_path(self.root, "claude", "session-1", "claude-code")
        self.assertTrue(published.exists())

    def test_ec2_load_missing_required_flag_exits_two(self):
        args = self._load_args()
        args.remove("--provider")
        args.remove("claude")

        with mock.patch.object(sys, "stderr", io.StringIO()) as fake_stderr:
            result = agent_preflight.main(args)

        self.assertEqual(result, 2)
        self.assertIn("--provider", fake_stderr.getvalue())

    def test_ec1_load_explicit_empty_session_id_exits_one_not_two(self):
        args = self._load_args(session_id="")

        with mock.patch.object(sys, "stderr", io.StringIO()) as fake_stderr:
            result = agent_preflight.main(args)

        self.assertEqual(result, 1)
        self.assertIn("session_id must not be empty", fake_stderr.getvalue())

    def test_hp1_check_command_ok_after_load(self):
        agent_preflight.main(self._load_args())

        result = agent_preflight.main(
            [
                "check",
                "--repo-root",
                str(self.root),
                "--provider",
                "claude",
                "--session-id",
                "session-1",
                "--actor-id",
                "claude-code",
            ]
        )

        self.assertEqual(result, 0)

    def test_ec2_check_command_exits_one_for_unpublished_identity(self):
        with mock.patch.object(sys, "stderr", io.StringIO()) as fake_stderr:
            result = agent_preflight.main(
                [
                    "check",
                    "--repo-root",
                    str(self.root),
                    "--provider",
                    "claude",
                    "--session-id",
                    "never-loaded",
                    "--actor-id",
                    "claude-code",
                ]
            )

        self.assertEqual(result, 1)
        self.assertIn("agent preflight failed", fake_stderr.getvalue())

    def test_ec2_check_command_missing_provider_flag_exits_two(self):
        with mock.patch.object(sys, "stderr", io.StringIO()) as fake_stderr:
            result = agent_preflight.main(
                [
                    "check",
                    "--repo-root",
                    str(self.root),
                    "--session-id",
                    "session-1",
                    "--actor-id",
                    "claude-code",
                ]
            )

        self.assertEqual(result, 2)
        self.assertIn("--provider", fake_stderr.getvalue())

    def test_ec2_hook_load_missing_provider_flag_exits_two(self):
        with mock.patch.object(sys, "stdin", io.StringIO("{}")), mock.patch.object(
            sys, "stderr", io.StringIO()
        ) as fake_stderr:
            result = agent_preflight.main(["hook-load", "--repo-root", str(self.root)])

        self.assertEqual(result, 2)
        self.assertIn("--provider", fake_stderr.getvalue())

    def test_ec2_hook_gate_missing_provider_flag_exits_two(self):
        with mock.patch.object(sys, "stdin", io.StringIO("{}")), mock.patch.object(
            sys, "stderr", io.StringIO()
        ) as fake_stderr:
            result = agent_preflight.main(["hook-gate", "--repo-root", str(self.root)])

        self.assertEqual(result, 2)
        self.assertIn("--provider", fake_stderr.getvalue())

    def test_legacy_mark_does_not_satisfy_v2_check(self):
        agent_preflight.main(["--repo-root", str(self.root), "--mark"])

        result = agent_preflight.main(
            [
                "check",
                "--repo-root",
                str(self.root),
                "--provider",
                "claude",
                "--session-id",
                "session-1",
                "--actor-id",
                "claude-code",
            ]
        )

        self.assertEqual(result, 1)


class AgentPreflightHookAdapterTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "CLAUDE.md").write_text("claude instructions\n", encoding="utf-8")
        (self.root / "AGENTS.override.md").write_text("codex instructions\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def _run_hook_command(self, command, provider, stdin_payload, extra_args=()):
        with mock.patch.object(sys, "stdin", io.StringIO(stdin_payload)), mock.patch.object(
            sys, "stdout", io.StringIO()
        ) as fake_stdout, mock.patch.object(sys, "stderr", io.StringIO()) as fake_stderr:
            result = agent_preflight.main(
                [command, "--repo-root", str(self.root), "--provider", provider, *extra_args]
            )
        return result, fake_stdout.getvalue(), fake_stderr.getvalue()

    def test_hp1_claude_hook_load_publishes_receipt(self):
        payload = json.dumps(
            {
                "session_id": "hook-sess-1",
                "hook_event_name": "SessionStart",
                "source": "startup",
                "transcript_path": "/tmp/transcript.json",
            }
        )

        result, stdout, stderr = self._run_hook_command("hook-load", "claude", payload)

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        self.assertIn("DubBridge agent preflight", stdout)
        published = agent_preflight.v2_receipt_path(self.root, "claude", "hook-sess-1", "claude-code")
        self.assertTrue(published.exists())

    def test_hp1_claude_hook_load_publishes_receipt_for_live_captured_payload(self):
        # T4c1: byte-for-byte the real SessionStart payload captured from a
        # genuine fresh Claude Code session via a temporary, user-authorized
        # diagnostic hook (session_id/transcript_path redacted to fixture
        # values; hook_event_name/source/cwd keys and shape are exactly what
        # Claude Code sent). This is the ground truth the fixture-only tests
        # above were previously blind to.
        payload = json.dumps(
            {
                "session_id": "fixture-live-captured-session",
                "transcript_path": "/tmp/fixture-transcript.jsonl",
                "cwd": str(self.root),
                "hook_event_name": "SessionStart",
                "source": "startup",
            }
        )

        result, stdout, stderr = self._run_hook_command("hook-load", "claude", payload)

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        published = agent_preflight.v2_receipt_path(
            self.root, "claude", "fixture-live-captured-session", "claude-code"
        )
        self.assertTrue(published.exists())
        receipt = json.loads(published.read_text(encoding="utf-8"))
        self.assertEqual(receipt["lifecycle"]["hook_event_name"], "startup")

    def test_hp1_codex_hook_load_publishes_receipt(self):
        payload = json.dumps(
            {"session_id": "codex-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )

        result, stdout, stderr = self._run_hook_command("hook-load", "codex", payload)

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        published = agent_preflight.v2_receipt_path(self.root, "codex", "codex-sess-1", "codex-cli")
        self.assertTrue(published.exists())
        receipt = json.loads(published.read_text(encoding="utf-8"))
        self.assertEqual(receipt["lifecycle"]["hook_event_name"], "startup")

    def test_hp1_codex_hook_load_publishes_receipt_for_live_captured_payload(self):
        # T4c1c: byte-for-byte the real SessionStart payload captured from a
        # genuine fresh `codex exec` session (codex-cli 0.146.0-alpha.3.1) via
        # the DUBBRIDGE_PREFLIGHT_DEBUG_STDIN capture hook (session_id/
        # transcript_path redacted to fixture values; every key and the shape
        # are exactly what Codex sent). Ground truth the fixture-only test
        # above was previously blind to: `hook_event_name` is the hook type
        # "SessionStart", and the lifecycle value lives in `source`.
        payload = json.dumps(
            {
                "session_id": "fixture-codex-live-captured",
                "transcript_path": "/tmp/fixture-codex-rollout.jsonl",
                "cwd": str(self.root),
                "hook_event_name": "SessionStart",
                "model": "gpt-5.6-sol",
                "permission_mode": "bypassPermissions",
                "source": "startup",
            }
        )

        result, stdout, stderr = self._run_hook_command("hook-load", "codex", payload)

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        published = agent_preflight.v2_receipt_path(
            self.root, "codex", "fixture-codex-live-captured", "codex-cli"
        )
        self.assertTrue(published.exists())
        receipt = json.loads(published.read_text(encoding="utf-8"))
        self.assertEqual(receipt["lifecycle"]["hook_event_name"], "startup")

    def test_hp2_claude_hook_gate_allows_after_load(self):
        load_payload = json.dumps(
            {"session_id": "hook-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )
        self._run_hook_command("hook-load", "claude", load_payload)

        gate_payload = json.dumps({"session_id": "hook-sess-1", "hook_event_name": "PreToolUse"})
        result, stdout, stderr = self._run_hook_command("hook-gate", "claude", gate_payload)

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        response = json.loads(stdout)
        self.assertEqual(
            response["hookSpecificOutput"]["permissionDecision"], "allow"
        )
        self.assertEqual(response["hookSpecificOutput"]["hookEventName"], "PreToolUse")

    def test_hp2_codex_hook_gate_allows_after_load(self):
        load_payload = json.dumps(
            {"session_id": "codex-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )
        self._run_hook_command("hook-load", "codex", load_payload)

        gate_payload = json.dumps(
            {
                "session_id": "codex-sess-1",
                "hook_event_name": "PreToolUse",
                "tool_name": "apply_patch",
            }
        )
        result, stdout, stderr = self._run_hook_command("hook-gate", "codex", gate_payload)

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        response = json.loads(stdout)
        self.assertEqual(response["hookSpecificOutput"]["permissionDecision"], "allow")

    def test_ec2_claude_hook_gate_denies_for_unknown_session(self):
        gate_payload = json.dumps({"session_id": "never-loaded", "hook_event_name": "PreToolUse"})

        result, stdout, stderr = self._run_hook_command("hook-gate", "claude", gate_payload)

        self.assertEqual(result, 1)
        response = json.loads(stdout)
        self.assertEqual(response["hookSpecificOutput"]["permissionDecision"], "deny")

    def test_ec2_codex_hook_gate_denies_for_unknown_session(self):
        gate_payload = json.dumps(
            {"session_id": "never-loaded", "hook_event_name": "PreToolUse", "tool_name": "apply_patch"}
        )

        result, stdout, stderr = self._run_hook_command("hook-gate", "codex", gate_payload)

        self.assertEqual(result, 1)
        response = json.loads(stdout)
        self.assertEqual(response["hookSpecificOutput"]["permissionDecision"], "deny")

    def test_ec1_hook_load_malformed_json_exits_two_stderr_only(self):
        result, stdout, stderr = self._run_hook_command("hook-load", "claude", "not json")

        self.assertEqual(result, 2)
        self.assertEqual(stdout, "")
        self.assertIn("malformed hook input", stderr)

    def test_ec1_hook_gate_malformed_json_exits_two_stderr_only(self):
        result, stdout, stderr = self._run_hook_command("hook-gate", "claude", "not json")

        self.assertEqual(result, 2)
        self.assertEqual(stdout, "")
        self.assertIn("malformed hook input", stderr)

    def test_ec1_hook_load_non_object_json_exits_two(self):
        result, stdout, stderr = self._run_hook_command("hook-load", "claude", "[1, 2, 3]")

        self.assertEqual(result, 2)
        self.assertEqual(stdout, "")

    def test_ec1_hook_load_missing_session_id_exits_two(self):
        payload = json.dumps({"hook_event_name": "SessionStart", "source": "startup"})

        result, stdout, stderr = self._run_hook_command("hook-load", "claude", payload)

        self.assertEqual(result, 2)
        self.assertIn("session_id", stderr)

    def test_ec1_hook_load_missing_source_field_exits_two(self):
        payload = json.dumps({"session_id": "hook-sess-1", "hook_event_name": "SessionStart"})

        result, stdout, stderr = self._run_hook_command("hook-load", "claude", payload)

        self.assertEqual(result, 2)
        self.assertIn("source", stderr)

    def test_ec2_hook_load_missing_native_instruction_file_exits_one(self):
        (self.root / "CLAUDE.md").unlink()
        payload = json.dumps(
            {"session_id": "hook-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )

        result, stdout, stderr = self._run_hook_command("hook-load", "claude", payload)

        self.assertEqual(result, 1)
        self.assertIn("CLAUDE.md", stderr)

    def test_ec1_hook_load_missing_session_id_field_codex_exits_two(self):
        payload = json.dumps({"hook_event_name": "SessionStart", "source": "startup"})

        result, stdout, stderr = self._run_hook_command("hook-load", "codex", payload)

        self.assertEqual(result, 2)
        self.assertIn("session_id", stderr)

    def test_adapt_hook_payload_rejects_unsupported_provider_directly(self):
        with self.assertRaises(agent_preflight.HookPayloadError):
            agent_preflight.adapt_hook_payload(
                "other", {"session_id": "x", "hook_event_name": "startup"}
            )

    def test_ec1_hook_load_missing_source_field_codex_exits_two(self):
        # T4c1c: a payload carrying only the hook type and no `source` is
        # exactly the shape that must fail closed -- it is indistinguishable
        # from the pre-fix assumption that `hook_event_name` held the
        # lifecycle value.
        payload = json.dumps({"session_id": "codex-sess-1", "hook_event_name": "SessionStart"})

        result, stdout, stderr = self._run_hook_command("hook-load", "codex", payload)

        self.assertEqual(result, 2)
        self.assertIn("source", stderr)

    def test_ec1_hook_load_unsupported_lifecycle_event_codex_exits_one(self):
        # Peer review (T4c1 phase-2): Codex's hook-load path had no direct
        # test of the fail-closed downstream lifecycle check, unlike the new
        # Claude tests added by this task. adapt_codex_hook_payload accepts
        # any string and defers validation to build_v2_receipt_payload's
        # validate_lifecycle_event -- confirm that still denies cleanly.
        payload = json.dumps(
            {
                "session_id": "codex-sess-1",
                "hook_event_name": "SessionStart",
                "source": "not_a_real_event",
            }
        )

        result, stdout, stderr = self._run_hook_command("hook-load", "codex", payload)

        self.assertEqual(result, 1)
        self.assertIn("not_a_real_event", stderr)

    def test_ec1_hook_load_unsupported_provider_argparse_rejects(self):
        with mock.patch.object(sys, "stderr", io.StringIO()):
            with self.assertRaises(SystemExit):
                agent_preflight.main(
                    ["hook-load", "--repo-root", str(self.root), "--provider", "other"]
                )

    def test_hp2_claude_hook_gate_allows_for_real_pretooluse_payload_shape(self):
        load_payload = json.dumps(
            {"session_id": "hook-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )
        self._run_hook_command("hook-load", "claude", load_payload)

        gate_payload = json.dumps(
            {
                "session_id": "hook-sess-1",
                "transcript_path": "/tmp/transcript.json",
                "cwd": str(self.root),
                "hook_event_name": "PreToolUse",
                "tool_name": "Edit",
                "tool_input": {"file_path": "foo.py"},
            }
        )
        result, stdout, stderr = self._run_hook_command("hook-gate", "claude", gate_payload)

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        response = json.loads(stdout)
        self.assertEqual(response["hookSpecificOutput"]["permissionDecision"], "allow")

    def test_hp2_codex_hook_gate_allows_for_real_tool_call_event_shape(self):
        load_payload = json.dumps(
            {"session_id": "codex-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )
        self._run_hook_command("hook-load", "codex", load_payload)

        gate_payload = json.dumps(
            {
                "session_id": "codex-sess-1",
                "transcript_path": "/tmp/transcript.json",
                "cwd": str(self.root),
                "hook_event_name": "PreToolUse",
                "tool_name": "apply_patch",
                "tool_input": {"path": "foo.py"},
            }
        )
        result, stdout, stderr = self._run_hook_command("hook-gate", "codex", gate_payload)

        self.assertEqual(result, 0)
        response = json.loads(stdout)
        self.assertEqual(response["hookSpecificOutput"]["permissionDecision"], "allow")

    def test_ec1_hook_gate_missing_session_id_exits_two_for_real_pretooluse_shape(self):
        gate_payload = json.dumps(
            {
                "transcript_path": "/tmp/transcript.json",
                "cwd": str(self.root),
                "hook_event_name": "PreToolUse",
                "tool_name": "Edit",
            }
        )
        result, stdout, stderr = self._run_hook_command("hook-gate", "claude", gate_payload)

        self.assertEqual(result, 2)
        self.assertEqual(stdout, "")
        self.assertIn("session_id", stderr)

    def test_extract_hook_gate_identity_rejects_unsupported_provider_directly(self):
        with self.assertRaises(agent_preflight.HookPayloadError):
            agent_preflight.extract_hook_gate_identity("other", {"session_id": "x"})

    def test_adapt_claude_hook_payload_reads_source_for_real_sessionstart_shape(self):
        # T4c1: real Claude Code SessionStart payloads always carry
        # hook_event_name == "SessionStart" (the hook type) and put the
        # lifecycle sub-event in `source` -- confirmed against a live-captured
        # payload. The adapter must read `source`, not `hook_event_name`.
        fields = agent_preflight.adapt_claude_hook_payload(
            {
                "session_id": "claude-sess-1",
                "hook_event_name": "SessionStart",
                "source": "startup",
                "transcript_path": "/tmp/transcript.json",
            }
        )
        self.assertEqual(fields["provider"], "claude")
        self.assertEqual(fields["session_id"], "claude-sess-1")
        self.assertEqual(fields["actor_id"], "claude-code")
        self.assertEqual(fields["hook_event_name"], "startup")
        self.assertEqual(fields["native_instruction_mechanism"], "@import")
        self.assertEqual(fields["native_instruction_path"], "CLAUDE.md")

    def test_adapt_claude_hook_payload_accepts_every_mapped_lifecycle_source(self):
        for lifecycle_value in ("startup", "resume", "clear", "compact", "fork"):
            with self.subTest(source=lifecycle_value):
                fields = agent_preflight.adapt_claude_hook_payload(
                    {
                        "session_id": "claude-sess-1",
                        "hook_event_name": "SessionStart",
                        "source": lifecycle_value,
                    }
                )
                self.assertEqual(fields["hook_event_name"], lifecycle_value)

    def test_adapt_claude_hook_payload_rejects_missing_source_directly(self):
        with self.assertRaises(agent_preflight.HookPayloadError) as ctx:
            agent_preflight.adapt_claude_hook_payload(
                {"session_id": "claude-sess-1", "hook_event_name": "SessionStart"}
            )
        self.assertIn("source", str(ctx.exception))

    def test_adapt_claude_hook_payload_rejects_empty_string_source_directly(self):
        with self.assertRaises(agent_preflight.HookPayloadError) as ctx:
            agent_preflight.adapt_claude_hook_payload(
                {"session_id": "claude-sess-1", "hook_event_name": "SessionStart", "source": ""}
            )
        self.assertIn("source", str(ctx.exception))

    def test_adapt_codex_hook_payload_reads_source_as_lifecycle_directly(self):
        fields = agent_preflight.adapt_codex_hook_payload(
            {"session_id": "codex-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )
        self.assertEqual(fields["provider"], "codex")
        self.assertEqual(fields["session_id"], "codex-sess-1")
        self.assertEqual(fields["actor_id"], "codex-cli")
        # The hook TYPE ("SessionStart") must never leak into the receipt's
        # lifecycle field -- `source` is the lifecycle value.
        self.assertEqual(fields["hook_event_name"], "startup")
        self.assertEqual(fields["native_instruction_mechanism"], "generated-bundle")
        self.assertEqual(fields["native_instruction_path"], "AGENTS.override.md")

    def test_adapt_codex_hook_payload_rejects_missing_source_directly(self):
        with self.assertRaises(agent_preflight.HookPayloadError) as ctx:
            agent_preflight.adapt_codex_hook_payload(
                {"session_id": "codex-sess-1", "hook_event_name": "SessionStart"}
            )
        self.assertIn("source", str(ctx.exception))

    def test_read_hook_stdin_captures_raw_payload_when_debug_env_set(self):
        # T4c1c: the only way to diagnose a provider payload-shape mismatch is
        # to capture the raw stdin -- Codex persists no hook events in its
        # rollout log, which is why this defect survived two tasks. Opt-in via
        # env var; inert otherwise.
        capture = Path(self.tmp.name) / "captured-stdin.json"
        raw = '{"session_id": "s1", "hook_event_name": "SessionStart", "source": "startup"}'
        with mock.patch.dict(
            os.environ, {"DUBBRIDGE_PREFLIGHT_DEBUG_STDIN": str(capture)}, clear=False
        ):
            data = agent_preflight._read_hook_stdin(io.StringIO(raw))

        self.assertEqual(data["source"], "startup")
        self.assertEqual(capture.read_text(encoding="utf-8"), raw)

    def test_read_hook_stdin_debug_capture_failure_never_breaks_parsing(self):
        # The capture is diagnostic only: an unwritable path must not turn a
        # valid payload into a hook failure. The fail-closed governance path
        # must never depend on debug I/O succeeding.
        unwritable = Path(self.tmp.name) / "no-such-dir" / "capture.json"
        raw = '{"session_id": "s1", "hook_event_name": "SessionStart", "source": "startup"}'
        with mock.patch.dict(
            os.environ, {"DUBBRIDGE_PREFLIGHT_DEBUG_STDIN": str(unwritable)}, clear=False
        ):
            data = agent_preflight._read_hook_stdin(io.StringIO(raw))

        self.assertEqual(data["source"], "startup")
        self.assertFalse(unwritable.exists())

    def test_read_hook_stdin_does_not_capture_when_debug_env_unset(self):
        capture = Path(self.tmp.name) / "must-not-exist.json"
        env = {k: v for k, v in os.environ.items() if k != "DUBBRIDGE_PREFLIGHT_DEBUG_STDIN"}
        with mock.patch.dict(os.environ, env, clear=True):
            agent_preflight._read_hook_stdin(io.StringIO('{"session_id": "s1"}'))

        self.assertFalse(capture.exists())

    def test_codex_gate_response_matches_claude_shaped_hook_specific_output(self):
        allow_response = agent_preflight.codex_gate_response(allow=True, reason="ok")
        deny_response = agent_preflight.codex_gate_response(allow=False, reason="denied")
        self.assertEqual(
            allow_response,
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "permissionDecisionReason": "ok",
                }
            },
        )
        self.assertEqual(
            deny_response["hookSpecificOutput"]["permissionDecision"], "deny"
        )

    def test_hp1_claude_hook_load_records_governing_documents_via_document_flags(self):
        (self.root / "AGENTS.md").write_text("agents contract\n", encoding="utf-8")
        (self.root / "docs" / "policies").mkdir(parents=True)
        (self.root / "docs" / "policies" / "HITL_AUTONOMY_POLICY.md").write_text(
            "autonomy policy\n", encoding="utf-8"
        )
        payload = json.dumps(
            {"session_id": "hook-sess-1", "hook_event_name": "SessionStart", "source": "startup"}
        )

        result, stdout, stderr = self._run_hook_command(
            "hook-load",
            "claude",
            payload,
            extra_args=(
                "--document",
                "AGENTS.md",
                "--document",
                "docs/policies/HITL_AUTONOMY_POLICY.md",
            ),
        )

        self.assertEqual(result, 0)
        self.assertEqual(stderr, "")
        published = agent_preflight.v2_receipt_path(self.root, "claude", "hook-sess-1", "claude-code")
        payload_on_disk = json.loads(published.read_text(encoding="utf-8"))
        recorded_paths = {doc["path"] for doc in payload_on_disk["documents"]}
        self.assertEqual(recorded_paths, {"AGENTS.md", "docs/policies/HITL_AUTONOMY_POLICY.md"})


class AgentPreflightRacePermissionTest(unittest.TestCase):
    """T4a4: deterministic, barrier-controlled race/replacement/permission tests.

    Every test here must resolve to exactly one of: a validated old receipt, a
    validated new receipt, or a clean ReceiptValidationError/OSError denial.
    Partial or stale JSON observed as a "success" is a failing assertion, not
    an acceptable outcome.
    """

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.instruction_path = self.root / "INSTRUCTIONS.md"
        self.instruction_path.write_text("do the task\n", encoding="utf-8")

    def tearDown(self):
        self.tmp.cleanup()

    def _payload(self, **overrides):
        kwargs = dict(
            provider="claude",
            session_id="session-1",
            actor_id="actor-1",
            repo_root=self.root,
            hook_event_name="startup",
            source="hook",
            transcript_path="/tmp/transcript.json",
            native_instruction_mechanism="hook",
            native_instruction_path="INSTRUCTIONS.md",
            document_paths=[],
        )
        kwargs.update(overrides)
        return agent_preflight.build_v2_receipt_payload(**kwargs)

    def test_hp1_barrier_controlled_simultaneous_loaders_different_sessions(self):
        """Two distinct-session publishers are released from a shared barrier at
        the same instant; both must complete with a parseable, schema-valid
        receipt loadable via load_v2_receipt for their own identity."""
        session_ids = ["race-session-a", "race-session-b"]
        for session_id in session_ids:
            agent_preflight.publish_v2_receipt(self.root, self._payload(session_id=session_id))

        barrier = threading.Barrier(len(session_ids))
        results: dict[str, dict] = {}
        errors: list[BaseException] = []
        lock = threading.Lock()

        def reload_and_load(session_id: str) -> None:
            try:
                barrier.wait(timeout=5)
                agent_preflight.publish_v2_receipt(
                    self.root, self._payload(session_id=session_id)
                )
                payload = agent_preflight.load_v2_receipt(
                    self.root, "claude", session_id, "actor-1"
                )
                with lock:
                    results[session_id] = payload
            except BaseException as exc:  # pragma: no cover - failure asserted below
                with lock:
                    errors.append(exc)

        threads = [
            threading.Thread(target=reload_and_load, args=(session_id,))
            for session_id in session_ids
        ]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(timeout=5)

        self.assertEqual(errors, [])
        self.assertEqual(set(results), set(session_ids))
        for session_id, payload in results.items():
            self.assertEqual(payload["schema_version"], agent_preflight.RECEIPT_SCHEMA_VERSION)
            self.assertEqual(payload["session_id"], session_id)

    def test_ec1_check_during_invalidate_replace_window_never_returns_partial_or_stale(self):
        """Force load_v2_receipt to observe the target path at the exact instant
        _invalidate_prior_receipt has unlinked the prior receipt but the
        replacement has not yet been os.replace'd in. The only acceptable
        outcomes are a clean ReceiptValidationError (file absent) or, if the
        loader runs after the real replace lands, a fully valid new payload --
        never a partially written or stale-but-still-present file."""
        first = self._payload(session_id="race-check")
        target_path = agent_preflight.publish_v2_receipt(self.root, first)
        first_loaded_at = json.loads(target_path.read_text(encoding="utf-8"))["loaded_at"]

        publish_started = threading.Event()
        allow_replace = threading.Event()
        observed: dict = {}

        original_invalidate = agent_preflight._invalidate_prior_receipt

        def gated_invalidate(path: Path) -> None:
            original_invalidate(path)
            publish_started.set()
            allow_replace.wait(timeout=5)

        def run_check() -> None:
            publish_started.wait(timeout=5)
            try:
                payload = agent_preflight.load_v2_receipt(
                    self.root, "claude", "race-check", "actor-1"
                )
                observed["outcome"] = "success"
                observed["payload"] = payload
            except agent_preflight.ReceiptValidationError as exc:
                observed["outcome"] = "clean_denial"
                observed["error"] = str(exc)

        checker = threading.Thread(target=run_check)
        checker.start()

        second = self._payload(session_id="race-check")
        with mock.patch.object(
            agent_preflight, "_invalidate_prior_receipt", side_effect=gated_invalidate
        ):
            publish_thread = threading.Thread(
                target=agent_preflight.publish_v2_receipt, args=(self.root, second)
            )
            publish_thread.start()
            publish_started.wait(timeout=5)
            allow_replace.set()
            publish_thread.join(timeout=5)
        checker.join(timeout=5)

        self.assertIn("outcome", observed)
        if observed["outcome"] == "clean_denial":
            self.assertNotIn("payload", observed)
        else:
            payload = observed["payload"]
            self.assertEqual(payload["schema_version"], agent_preflight.RECEIPT_SCHEMA_VERSION)
            self.assertIn(payload["loaded_at"], {first_loaded_at, second["loaded_at"]})

        final = json.loads(target_path.read_text(encoding="utf-8"))
        self.assertEqual(final["loaded_at"], second["loaded_at"])

    def test_ec1_load_never_accepts_partially_written_temp_file_as_receipt(self):
        """A temp file mid-write (no os.replace yet) must never be visible at
        the final target path, so load_v2_receipt must reject it as absent
        rather than parse truncated JSON as a valid receipt."""
        payload = self._payload(session_id="race-partial")
        target_path = agent_preflight.v2_receipt_path(self.root, "claude", "race-partial", "actor-1")
        agent_preflight._secure_mkdir(target_path.parent)

        tmp_path = target_path.parent / f".{target_path.name}.partial.tmp"
        tmp_path.write_text('{"schema_version": 2, "session_id": "race-partial"', encoding="utf-8")

        with self.assertRaises(agent_preflight.ReceiptValidationError):
            agent_preflight.load_v2_receipt(self.root, "claude", "race-partial", "actor-1")

        self.assertFalse(target_path.exists())

    def test_ec2_denied_receipts_directory_produces_clean_authorization_failure(self):
        """A receipts directory the process cannot read/traverse must fail
        load_v2_receipt closed (ReceiptValidationError), never raise an
        unhandled exception or silently authorize."""
        payload = self._payload(session_id="race-denied-dir")
        agent_preflight.publish_v2_receipt(self.root, payload)
        receipts_dir = agent_preflight.v2_receipts_dir(self.root)

        if os.geteuid() == 0:
            with mock.patch.object(
                agent_preflight.Path,
                "read_text",
                side_effect=PermissionError("denied"),
            ):
                with self.assertRaises(agent_preflight.ReceiptValidationError):
                    agent_preflight.load_v2_receipt(
                        self.root, "claude", "race-denied-dir", "actor-1"
                    )
            return

        original_mode = stat.S_IMODE(receipts_dir.stat().st_mode)
        try:
            receipts_dir.chmod(0o000)
            with self.assertRaises(agent_preflight.ReceiptValidationError):
                agent_preflight.load_v2_receipt(
                    self.root, "claude", "race-denied-dir", "actor-1"
                )
        finally:
            receipts_dir.chmod(original_mode)

    def test_ec2_denied_receipt_file_produces_clean_authorization_failure(self):
        """A receipt file the process cannot open for read must fail
        load_v2_receipt closed, never partially parse or authorize."""
        payload = self._payload(session_id="race-denied-file")
        target_path = agent_preflight.publish_v2_receipt(self.root, payload)

        if os.geteuid() == 0:
            self.skipTest(
                "Running as root: POSIX file permission denial cannot be enforced "
                "against this process, so EC-2 file-level denial is exercised via "
                "the directory-level monkeypatch case instead."
            )

        original_mode = stat.S_IMODE(target_path.stat().st_mode)
        try:
            target_path.chmod(0o000)
            with self.assertRaises(agent_preflight.ReceiptValidationError):
                agent_preflight.load_v2_receipt(
                    self.root, "claude", "race-denied-file", "actor-1"
                )
        finally:
            target_path.chmod(original_mode)


if __name__ == "__main__":
    unittest.main()
