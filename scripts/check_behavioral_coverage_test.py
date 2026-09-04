#!/usr/bin/env python3
import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-behavioral-coverage.py")


def load_module(path: Path):
    spec = importlib.util.spec_from_file_location("behavior_gate", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class BehavioralCoverageGateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name)
        (self.repo / "docs" / "tasks").mkdir(parents=True)
        (self.repo / "tests").mkdir()
        (self.repo / "mobile" / "__tests__").mkdir(parents=True)
        (self.repo / "mobile" / "maestro").mkdir(parents=True)
        (self.repo / "tests" / "sample.py").write_text("def test_python_case():\n    pass\n", encoding="utf-8")
        (self.repo / "mobile" / "__tests__" / "sample.test.tsx").write_text(
            'test("renders dashboard", () => {});\n', encoding="utf-8"
        )
        (self.repo / "mobile" / "maestro" / "flow.yaml").write_text(
            "appId: com.example\n---\n- assertVisible: home-screen\n", encoding="utf-8"
        )
        self.module = load_module(SCRIPT)

    def tearDown(self):
        self.tmp.cleanup()

    def ledger(self, body: str):
        (self.repo / "docs" / "tasks" / "fixture.md").write_text(
            "Behavioral coverage contract: behavior-v2\n\n" + body, encoding="utf-8"
        )

    def closure(self, rri=30):
        reflection = ""
        if rri >= 26:
            passes = 2 if rri <= 40 else 3 if rri <= 55 else 4
            reflection = f"""### Reflection log
Required passes: {passes}
#### Pass 1
- Draft verdict: fixture
- Critique findings: none
- Revisions applied: none
#### Pass 2
- Draft verdict: fixture
- Critique findings: none
- Revisions applied: none
"""
        return reflection + """### Owner final verification
- Owner: test-agent
- Date: 2026-09-04
- Statement: verified fixture behavior evidence.
- Commands run: python3 fixture
"""

    def valid_body(self, extra=""):
        return f"""## T1
- **Status:** [x] Done
- **Type:** development
- **RRI:** 30
### Happy paths considered
- HP-1: valid behavior
### Edge cases considered
- EC-1: failure behavior
### Behavioral coverage certification
| Case ID | Type | Behavior | Layer | Executable evidence | Result |
|---|---|---|---|---|---|
| HP-1 | Happy | valid | component | `mobile/__tests__/sample.test.tsx::renders dashboard`; `tests/sample.py::test_python_case` | passed |
| EC-1 | Edge | failure | e2e | `mobile/maestro/flow.yaml` | passed |
{self.closure()}{extra}"""

    def test_valid_cross_stack_evidence_passes(self):
        self.ledger(self.valid_body())
        self.assertEqual(self.module.validate_repo(self.repo), [])

    def test_missing_evidence_file_fails_closed(self):
        body = self.valid_body().replace(
            "`mobile/__tests__/sample.test.tsx::renders dashboard`; `tests/sample.py::test_python_case`",
            "`tests/missing.py::test_missing`",
        )
        self.ledger(body)
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("missing evidence file" in error for error in errors))

    def test_missing_edge_case_fails_closed(self):
        body = self.valid_body().replace("- EC-1: failure behavior\n", "")
        self.ledger(body)
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("missing stable EC-# case" in error for error in errors))

    def test_missing_python_function_fails_closed(self):
        body = self.valid_body().replace("tests/sample.py::test_python_case", "tests/sample.py::test_does_not_exist")
        self.ledger(body)
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("missing named test" in error for error in errors))

    def test_invalid_layer_fails_closed(self):
        self.ledger(self.valid_body().replace("| HP-1 | Happy | valid | component |", "| HP-1 | Happy | valid | magic |"))
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("layer 'magic'" in error for error in errors))

    def test_missing_reflection_for_moderate_task_fails(self):
        body = self.valid_body()
        start = body.index("### Reflection log")
        end = body.index("### Owner final verification")
        self.ledger(body[:start] + body[end:])
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("missing Reflection log" in error for error in errors))

    def test_missing_owner_verification_fails(self):
        body = self.valid_body()
        start = body.index("### Owner final verification")
        self.ledger(body[:start])
        errors = self.module.validate_repo(self.repo)
        self.assertTrue(any("missing Owner final verification" in error for error in errors))

    def test_in_progress_task_is_not_closed_by_gate(self):
        self.ledger("""## T1
- **Status:** [~] In progress
- **Type:** development
- **RRI:** 30
### Happy paths considered
- HP-1: valid behavior
### Edge cases considered
- EC-1: failure behavior
""")
        self.assertEqual(self.module.validate_repo(self.repo), [])


if __name__ == "__main__":
    unittest.main()
