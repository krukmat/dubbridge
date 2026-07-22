#!/usr/bin/env python3
"""Workflow gates for the local runner.

Only the organization gate remains here. The former semantic preflight (Serena
CLI index/health-check plus an MCP session) was removed with the Serena editing
path — see docs/plan/local-agent-simple-editing.md.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


def run_organization_gate(worktree_dir: str) -> dict[str, object]:
    script_path = Path(__file__).with_name("organization_gate.py")
    completed = subprocess.run(
        [sys.executable, str(script_path), "--base", "HEAD"],
        cwd=worktree_dir,
        check=False,
        capture_output=True,
        text=True,
    )
    try:
        payload = json.loads(completed.stdout or "{}")
    except json.JSONDecodeError as exc:
        return {
            "status": "tool_failure",
            "error": f"invalid organization gate output: {exc}",
            "returncode": completed.returncode,
            "stdout": completed.stdout,
            "stderr": completed.stderr,
        }
    payload["returncode"] = completed.returncode
    if completed.stderr:
        payload["stderr"] = completed.stderr
    return payload
