#!/usr/bin/env python3
"""Human-operated launcher for existing DubBridge tools; no workflow authority."""
from __future__ import annotations

import argparse
from pathlib import Path
import shlex
import subprocess
import sys


REPO_ROOT = Path(__file__).resolve().parents[1]
TOOLS = {
    "preflight": "scripts/agent-preflight.py",
    "rri": "scripts/rri.py",
    "review": "scripts/peer-workflow-review.py",
    "low": "scripts/delegate-low-rri.py",
    "local": "scripts/local-agent/run_local_task.py",
    "architect": "scripts/local-architect/run_analysis.py",
    "gate": "scripts/local-agent/med_high_gate.py",
}
STATUS_COMMANDS = (
    ("git", "status", "--short", "--branch"),
    ("git", "rev-parse", "--short", "HEAD"),
    ("git", "diff", "--stat"),
    ("git", "worktree", "list"),
)


def invoke(command: list[str]) -> int:
    """Run literal argv and preserve exit status (signals use shell convention)."""
    try:
        result = subprocess.run(command, cwd=REPO_ROOT, check=False)
    except OSError as exc:
        print(f"Unable to launch command: {exc}", file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        return 130
    return result.returncode if result.returncode >= 0 else 128 - result.returncode


def status() -> int:
    """Show local Git facts; stop on the first failed inspection."""
    for command in STATUS_COMMANDS:
        print(f"$ {shlex.join(command)}", flush=True)
        code = invoke(list(command))
        if code:
            return code
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Inspect local status or print an existing tool command.",
        epilog=(
            "No approvals, routing, retries, reviews or closure are inferred. "
            "Check task gates in the ledger before execution. Relative child "
            "paths resolve from the repository root, not your current directory. "
            "See docs/playbooks/HUMAN_ORCHESTRATOR_KT.md."
        ),
    )
    parser.add_argument(
        "--execute", action="store_true",
        help="Actually launch a tool. Place before 'run'; default is print only.",
    )
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("status", help="Run read-only local Git inspection.")
    run = commands.add_parser("run", help="Print a command; requires --execute to run.")
    run.add_argument("alias", choices=TOOLS)
    run.add_argument("args", nargs=argparse.REMAINDER, help="Child arguments, after --.")
    args = parser.parse_args(argv)
    if args.command == "status":
        return status()
    script = REPO_ROOT / TOOLS[args.alias]
    if not script.is_file():
        parser.error(f"Missing tool: {script}")
    forwarded = args.args[1:] if args.args[:1] == ["--"] else args.args
    command = [sys.executable, str(script), *forwarded]
    print(f"cwd: {REPO_ROOT}", flush=True)
    print(f"command: {shlex.join(command)}", flush=True)
    if not args.execute:
        print("DRY RUN: command not executed. Use --execute before run to launch.")
        return 0
    print("Executing existing tool; task gates remain the operator's responsibility.",
          file=sys.stderr, flush=True)
    return invoke(command)


if __name__ == "__main__":
    raise SystemExit(main())
