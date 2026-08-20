#!/usr/bin/env python3
"""Deterministically generate the bounded Codex instruction bootstrap.

AGENTS.override.md is a byte-exact projection of AGENTS.md. Canonical workflow,
policy, roadmap, architecture, ADR, plan, and task detail stays in its owning
document and is loaded on demand through the bootstrap's routing rules. This
avoids injecting the whole governance/documentation corpus into every Codex
session while preserving one generated file and one byte-exact drift check.

Default mode prints to stdout for check-doc-consistency.sh; --write overwrites
AGENTS.override.md only after the source exists, is non-empty, and fits the
bounded always-loaded-context budget.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SOURCE_RELATIVE_PATHS = (
    "AGENTS.md",
)
OUTPUT_RELATIVE_PATH = "AGENTS.override.md"
# A conservative, dependency-free size guard for the proposed 3k-6k-token
# ceiling. Byte count is not an exact tokenizer result, so closure also reports
# a separate token estimate; the current English Markdown stays well below it.
MAX_OUTPUT_BYTES = 24 * 1024


def read_source(relative_path: str) -> str:
    path = REPO_ROOT / relative_path
    if not path.is_file():
        raise SystemExit(f"generate-agents-override: missing source file: {relative_path}")
    content = path.read_text(encoding="utf-8")
    if not content:
        raise SystemExit(f"generate-agents-override: empty source file: {relative_path}")
    return content


def generate() -> str:
    content = "".join(read_source(relative_path) for relative_path in SOURCE_RELATIVE_PATHS)
    size = len(content.encode("utf-8"))
    if size > MAX_OUTPUT_BYTES:
        raise SystemExit(
            "generate-agents-override: bootstrap exceeds "
            f"{MAX_OUTPUT_BYTES} bytes ({size} bytes); move detail to canonical "
            "on-demand documents instead of expanding always-loaded context"
        )
    return content


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--check",
        action="store_true",
        help="Print the generated content to stdout (default behavior).",
    )
    mode.add_argument(
        "--write",
        action="store_true",
        help="Write the generated content to AGENTS.override.md, overwriting it.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    content = generate()
    if args.write:
        output_path = REPO_ROOT / OUTPUT_RELATIVE_PATH
        output_path.write_text(content, encoding="utf-8")
    else:
        sys.stdout.write(content)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
