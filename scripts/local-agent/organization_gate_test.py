#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import io
import json
import sys
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("organization_gate.py")
SPEC = importlib.util.spec_from_file_location("organization_gate", SCRIPT)
assert SPEC is not None
gate = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules["organization_gate"] = gate
SPEC.loader.exec_module(gate)


def line(path: str, text: str, *, status: str = "M", line_no: int = 1) -> object:
    return gate.ChangedLine(path=path, text=text, status=status, line_no=line_no)


class OrganizationGateTest(unittest.TestCase):
    def test_hp1_thin_main_and_focused_module_pass(self) -> None:
        lines = [
            line("apps/worker-runner/src/main.rs", "let queue = dubbridge_jobs::default_queue();", line_no=20),
            line("apps/worker-runner/src/main.rs", 'tracing::info!(queue = %queue, "ready");', line_no=21),
            line("apps/worker-runner/src/preparation_runtime.rs", "pub async fn run_preparation_runtime() -> anyhow::Result<()> {", status="A"),
            line("apps/worker-runner/src/preparation_runtime.rs", "    Ok(())", status="A"),
            line("apps/worker-runner/src/preparation_runtime.rs", "}", status="A"),
        ]
        self.assertEqual(gate.analyze(lines), [])

    def test_ec1_business_logic_in_main_is_rejected(self) -> None:
        violations = gate.analyze(
            [
                line("apps/api/src/main.rs", "if should_retry(job) {", line_no=44),
                line("apps/api/src/main.rs", "    process_job(job).await?;", line_no=45),
            ]
        )
        self.assertTrue(any(item["rule"] == "composition_root" for item in violations))

    def test_ec2_existing_file_growth_budget_is_rejected(self) -> None:
        violations = gate.analyze(
            [line("crates/domain/src/audit.rs", f"let field_{idx} = build_field();", line_no=idx) for idx in range(40)]
        )
        self.assertTrue(any(item["rule"] == "file_growth" for item in violations))

    def test_ec3_new_lint_suppression_is_rejected(self) -> None:
        violations = gate.analyze(
            [line("apps/api/src/cleanup.rs", "#[allow(clippy::too_many_lines)]", line_no=12)]
        )
        self.assertTrue(any(item["rule"] == "lint_suppression" for item in violations))

    def test_parse_added_lines_marks_new_files(self) -> None:
        diff = """diff --git a/apps/api/src/main.rs b/apps/api/src/main.rs
+++ b/apps/api/src/main.rs
@@ -1,0 +1,2 @@
+let queue = build_queue();
+run(queue);
diff --git a/apps/api/src/runtime.rs b/apps/api/src/runtime.rs
new file mode 100644
+++ b/apps/api/src/runtime.rs
@@ -0,0 +1,2 @@
+pub fn run() {}
+"""
        parsed = gate.parse_added_lines(diff)
        self.assertEqual([item.status for item in parsed], ["M", "M", "A", "A"])

    def test_main_returns_tool_failure_json(self) -> None:
        stdout = io.StringIO()
        with patch.object(gate, "build_report", side_effect=RuntimeError("boom")), patch("sys.stdout", stdout):
            code = gate.main([])
        self.assertEqual(code, 2)
        self.assertEqual(json.loads(stdout.getvalue())["status"], "tool_failure")


if __name__ == "__main__":
    unittest.main()
