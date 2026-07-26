#!/usr/bin/env python3
"""Tests for escalation_packet.py — golden-file format test plus HP/EC coverage."""

import json
import os
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import escalation_packet


SCRIPT_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "escalation_packet.py")


def write_json(path, data):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f)


def write_text(path, text):
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


REALISTIC_TRANSCRIPT = {
    "status": "budget_exhausted",
    "task_id": "T99",
    "finished_at": "2026-07-12T00:00:00Z",
    "transcript": [
        {"role": "assistant", "raw": {"tool_calls": [{"function": {"name": "write_file", "arguments": "{}"}}]}},
        {"event": "tool_result", "result": {"tool": "write_file", "path": "src/lib.rs", "ok": True}},
        {"event": "tool_result", "result": {"tool": "run_command", "argv": ["cargo", "build"], "ok": True, "returncode": 0, "stdout": "Compiling...\n", "stderr": ""}},
        {"event": "test_result", "result": {"passed": False, "output": "1 test failed: test_foo"}},
        {"event": "tool_result", "result": {"tool": "write_file", "path": "src/lib.rs", "ok": True}},
        {"event": "tool_result", "result": {"tool": "run_command", "argv": ["cargo", "test"], "ok": False, "returncode": 1, "stdout": "", "stderr": "error[E0000]\n"}},
        {"event": "test_result", "result": {"passed": False, "output": "1 test failed: test_foo"}},
    ],
    "reason": "repair_attempts_exhausted",
    "attempts": 2,
}

REALISTIC_CARD = {
    "task_id": "T99",
    "spec": "Fix the failing test_foo unit test in src/lib.rs.",
    "acceptance_tests": ["cargo test -p demo -- test_foo"],
    "allowed_paths": ["src/lib.rs", "src/lib_test.rs"],
}

REALISTIC_DIFF = (
    "diff --git a/src/lib.rs b/src/lib.rs\n"
    "index 111..222 100644\n"
    "--- a/src/lib.rs\n"
    "+++ b/src/lib.rs\n"
    "@@ -1,1 +1,1 @@\n"
    "-fn foo() -> i32 { 0 }\n"
    "+fn foo() -> i32 { 1 }\n"
)

REALISTIC_RRI_TABLE = "| Variable | Score |\n|---|---|\n| C | 1 |\n"


# Captured from the repository state before any task in
# docs/tasks/med-high-escalation-bundle-crash.md lands (plan D4). T1-T3 prove
# their changes are byte-identical to this on every currently-working path;
# never modified by this ticket.
PRE_TICKET_BUNDLE_WITH_DIFF = """# Escalation packet: `T99`

## 1. Task spec + RRI table

Task ID: `T99`

Spec:

Fix the failing test_foo unit test in src/lib.rs.

RRI table:

| Variable | Score |
|---|---|
| C | 1 |


## 2. Plan

MISSING

## 3. Allowed paths

- `src/lib.rs`
- `src/lib_test.rs`

## 4. Full diff

```diff
diff --git a/src/lib.rs b/src/lib.rs
index 111..222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,1 +1,1 @@
-fn foo() -> i32 { 0 }
+fn foo() -> i32 { 1 }

```

## 5. Commands executed with output

### Command 1

argv: `['cargo', 'build']`

returncode: `0`

stdout:
```
Compiling...

```

stderr:
```

```

### Command 2

argv: `['cargo', 'test']`

returncode: `1`

stdout:
```

```

stderr:
```
error[E0000]

```

## 6. Test results

### Attempt 1: FAILED

output:
```
1 test failed: test_foo
```

### Attempt 2: FAILED

output:
```
1 test failed: test_foo
```

## 7. Per-attempt summaries

- Attempt 1: wrote file `src/lib.rs`; ran command `['cargo', 'build']` (returncode 0); tests failed.
- Attempt 2: wrote file `src/lib.rs`; ran command `['cargo', 'test']` (returncode 1); tests failed.
- Final status: `budget_exhausted`.
"""


class GoldenFileFormat(unittest.TestCase):
    def test_golden_output_matches_exactly(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            diff_path = os.path.join(tmp, "diff.txt")
            out_path = os.path.join(tmp, "packet.md")

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, REALISTIC_CARD)
            write_text(diff_path, REALISTIC_DIFF)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                    "--diff-file", diff_path,
                    "--rri-table", REALISTIC_RRI_TABLE,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                actual = f.read()

            self.assertEqual(actual, PRE_TICKET_BUNDLE_WITH_DIFF)


class HP1AllSectionsPopulated(unittest.TestCase):
    def test_hp1_all_seven_sections_populated_and_diff_verbatim(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            diff_path = os.path.join(tmp, "diff.txt")
            out_path = os.path.join(tmp, "packet.md")

            card_with_plan = dict(REALISTIC_CARD)
            card_with_plan["plan"] = "1. Read failing test. 2. Patch foo(). 3. Re-run."

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, card_with_plan)
            write_text(diff_path, REALISTIC_DIFF)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                    "--diff-file", diff_path,
                    "--rri-table", REALISTIC_RRI_TABLE,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            for heading in [
                "## 1. Task spec + RRI table",
                "## 2. Plan",
                "## 3. Allowed paths",
                "## 4. Full diff",
                "## 5. Commands executed with output",
                "## 6. Test results",
                "## 7. Per-attempt summaries",
            ]:
                self.assertIn(heading, content)

            self.assertIn(REALISTIC_DIFF, content)
            self.assertIn("1. Read failing test. 2. Patch foo(). 3. Re-run.", content)
            self.assertNotIn("## 2. Plan\n\nMISSING", content)


class EC1MissingArtifactsRenderExplicitMissing(unittest.TestCase):
    def test_ec1_missing_diff_and_rri_table_render_missing_others_populated(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, REALISTIC_CARD)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn("## 4. Full diff\n\nMISSING", content)
            self.assertIn("RRI table:\n\nMISSING", content)
            self.assertIn("## 2. Plan\n\nMISSING", content)

            self.assertIn("### Command 1", content)
            self.assertIn("### Attempt 1: FAILED", content)
            self.assertIn("- Attempt 1:", content)
            self.assertNotIn("## 3. Allowed paths\n\nMISSING", content)

    def test_ec1_no_commands_and_no_tests_render_missing_not_empty(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")

            empty_transcript = {
                "status": "aborted",
                "task_id": "T99",
                "transcript": [
                    {"event": "boundary_violation", "error": "escaped worktree"},
                ],
            }

            write_json(transcript_path, empty_transcript)
            write_json(card_path, REALISTIC_CARD)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn("## 5. Commands executed with output\n\nMISSING", content)
            self.assertIn("## 6. Test results\n\nMISSING", content)
            self.assertIn("Final status: `aborted`", content)
            self.assertIn("escaped worktree", content)


class AllowedPathsRendering(unittest.TestCase):
    def test_missing_allowed_paths_renders_missing(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")

            card_no_paths = {
                "task_id": "T100",
                "spec": "Do something.",
            }

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, card_no_paths)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn("## 3. Allowed paths\n\nMISSING", content)


class RriTableAsInlineStringVsFile(unittest.TestCase):
    def test_rri_table_flag_accepts_literal_string_not_just_file_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, REALISTIC_CARD)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                    "--rri-table", "| a | b |\n|---|---|\n",
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn("| a | b |", content)

    def test_rri_table_flag_accepts_file_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            rri_path = os.path.join(tmp, "rri.md")
            out_path = os.path.join(tmp, "packet.md")

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, REALISTIC_CARD)
            write_text(rri_path, REALISTIC_RRI_TABLE)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                    "--rri-table", rri_path,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn("| Variable | Score |", content)


class ReadOptionalTextFileTest(unittest.TestCase):
    """T1 (plan Defect A/D2/D3): read_optional_text_file must never let a
    read failure escape, and must distinguish absent/not-found/unreadable."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_hp2_no_path_renders_bare_missing(self):
        text, missing = escalation_packet.read_optional_text_file(None, label="diff file")
        self.assertIsNone(text)
        self.assertEqual(missing, "MISSING")

    def test_ec1_nonexistent_path_names_path_in_missing_text(self):
        path = os.path.join(self.tmp.name, "does-not-exist.txt")
        text, missing = escalation_packet.read_optional_text_file(path, label="diff file")
        self.assertIsNone(text)
        self.assertEqual(missing, f"MISSING (diff file not found: {path})")

    def test_ec2_unreadable_existing_file_names_error(self):
        import builtins

        path = os.path.join(self.tmp.name, "unreadable.txt")
        write_text(path, "content")
        real_open = builtins.open

        def failing_open(p, *a, **kw):
            if p == path:
                raise PermissionError("denied")
            return real_open(p, *a, **kw)

        builtins.open = failing_open
        try:
            text, missing = escalation_packet.read_optional_text_file(path, label="diff file")
        finally:
            builtins.open = real_open

        self.assertIsNone(text)
        self.assertEqual(missing, f"MISSING (diff file unreadable: {path}: denied)")

    def test_ec3_directory_path_renders_not_found_not_uncaught(self):
        # os.path.isfile() is False for a directory, so this falls into the
        # not-found branch, not the unreadable branch (plan: "Scope boundary
        # on resolve_rri_table" -- same os.path.isfile behavior applies here).
        # It must not raise IsADirectoryError.
        text, missing = escalation_packet.read_optional_text_file(self.tmp.name, label="diff file")
        self.assertIsNone(text)
        self.assertEqual(missing, f"MISSING (diff file not found: {self.tmp.name})")

    def test_hp1_readable_file_returns_content_with_no_missing_text(self):
        path = os.path.join(self.tmp.name, "diff.txt")
        write_text(path, "some diff text")
        text, missing = escalation_packet.read_optional_text_file(path, label="diff file")
        self.assertEqual(text, "some diff text")
        self.assertIsNone(missing)


class EC4DiffFileNonexistentPathViaCli(unittest.TestCase):
    def test_ec4_adr036_cli_nonexistent_diff_file_renders_not_found(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")
            missing_diff_path = os.path.join(tmp, "no-such-diff.txt")

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, REALISTIC_CARD)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                    "--diff-file", missing_diff_path,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn(
                f"## 4. Full diff\n\nMISSING (diff file not found: {missing_diff_path})",
                content,
            )


class EC6RriTableUnreadableExistingFile(unittest.TestCase):
    def test_ec6_rri_table_permission_error_renders_unreadable_not_uncaught(self):
        with tempfile.TemporaryDirectory() as tmp:
            rri_path = os.path.join(tmp, "rri.md")
            write_text(rri_path, REALISTIC_RRI_TABLE)

            orig_open = escalation_packet.read_optional_text_file

            def failing_reader(path, *, label):
                if path == rri_path:
                    return None, f"MISSING ({label} unreadable: {path}: denied)"
                return orig_open(path, label=label)

            escalation_packet.read_optional_text_file = failing_reader
            try:
                result = escalation_packet.resolve_rri_table(rri_path)
            finally:
                escalation_packet.read_optional_text_file = orig_open

            self.assertEqual(result, f"MISSING (RRI table file unreadable: {rri_path}: denied)")

    def test_nonexistent_rri_table_path_still_becomes_literal_text_unchanged(self):
        # Path-vs-text ambiguity stays out of scope (T5 follow-up): a
        # nonexistent path is untouched by this task's read-failure fix.
        result = escalation_packet.resolve_rri_table("not-a-real-path.md")
        self.assertEqual(result, "not-a-real-path.md")


class TranscriptShapeValidationTest(unittest.TestCase):
    """T4 (plan D9): a transcript that parses but has the wrong shape must
    not crash build_packet's .get() calls on the ADR-036 CLI path, and the
    failure reason must actually render (round 6's blocking finding)."""

    def test_ec3_adr036_cli_transcript_list_normalizes_without_crash(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")

            write_json(transcript_path, ["not", "a", "dict"])
            write_json(card_path, REALISTIC_CARD)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn("## 5. Commands executed with output\n\nMISSING", content)
            self.assertIn("## 6. Test results\n\nMISSING", content)
            self.assertIn(
                "- Final status: `transcript_shape_invalid` "
                "(expected a JSON object, got list).",
                content,
            )

    def test_transcript_scalar_also_normalizes_without_crash(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")

            write_json(transcript_path, "not an object")
            write_json(card_path, REALISTIC_CARD)

            exit_code = escalation_packet.main(
                [
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                ]
            )

            self.assertEqual(exit_code, 0)
            with open(out_path, encoding="utf-8") as f:
                content = f.read()

            self.assertIn(
                "expected a JSON object, got str", content,
            )


class CliInvocation(unittest.TestCase):
    def test_cli_subprocess_invocation_exits_zero_and_writes_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            transcript_path = os.path.join(tmp, "transcript.json")
            card_path = os.path.join(tmp, "card.json")
            out_path = os.path.join(tmp, "packet.md")

            write_json(transcript_path, REALISTIC_TRANSCRIPT)
            write_json(card_path, REALISTIC_CARD)

            completed = subprocess.run(
                [
                    sys.executable, SCRIPT_PATH,
                    "--transcript", transcript_path,
                    "--card", card_path,
                    "--out", out_path,
                ],
                capture_output=True,
                text=True,
                timeout=30,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertTrue(os.path.isfile(out_path))


if __name__ == "__main__":
    unittest.main()
