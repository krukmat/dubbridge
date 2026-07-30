#!/usr/bin/env python3
"""HP-2 evidence for T2e-pre: every file produced by the artifact_schema.py /
sandbox_budget.py decomposition stays under the 500-line target-file-size
gate for RRI 26-55 local-first delegation
(docs/playbooks/AGENT_WORKFLOW_GUIDE.md § Target-file size gate).
"""

from __future__ import annotations

import unittest
from pathlib import Path

_ANTARES_DIR = Path(__file__).parent
_MAX_LINES = 500

_DECOMPOSED_FILES = (
    "artifact_schema.py",
    "artifact_validators.py",
    "artifact_trace_writer.py",
    "artifact_serialization.py",
    "artifact_examples.py",
    "sandbox_budget.py",
    "sandbox_resource_limits.py",
    "sandbox_session_budget.py",
    "sandbox_process_io.py",
)


class DecomposedFileSizeTest(unittest.TestCase):
    def test_hp2_every_decomposed_file_stays_under_500_lines(self) -> None:
        for filename in _DECOMPOSED_FILES:
            with self.subTest(file=filename):
                path = _ANTARES_DIR / filename
                line_count = sum(1 for _ in path.open("r", encoding="utf-8"))
                self.assertLess(
                    line_count,
                    _MAX_LINES,
                    f"{filename} has {line_count} lines, exceeding the {_MAX_LINES}-line gate",
                )


if __name__ == "__main__":
    unittest.main()
