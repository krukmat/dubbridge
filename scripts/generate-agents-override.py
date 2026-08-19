#!/usr/bin/env python3
"""Deterministically generate AGENTS.override.md from its five source files.

Concatenates AGENTS.md + docs/playbooks/AGENT_WORKFLOW_GUIDE.md +
docs/policies/HITL_AUTONOMY_POLICY.md + docs/plan/roadmap.md +
docs/architecture.md, in that order, with no separator inserted between them
(each source file's own leading frontmatter fence is what visually reads as
a seam). roadmap.md and architecture.md are appended last, mirroring
CLAUDE.md's native-import order, so they stay the mechanism's most volatile
sources without disturbing the byte offsets of the three governance
documents ahead of them. Default mode prints to stdout for the
check-doc-consistency.sh drift check; --write overwrites AGENTS.override.md.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SOURCE_RELATIVE_PATHS = (
    "AGENTS.md",
    "docs/playbooks/AGENT_WORKFLOW_GUIDE.md",
    "docs/policies/HITL_AUTONOMY_POLICY.md",
    "docs/plan/roadmap.md",
    "docs/architecture.md",
)
OUTPUT_RELATIVE_PATH = "AGENTS.override.md"


def read_source(relative_path: str) -> str:
    path = REPO_ROOT / relative_path
    if not path.is_file():
        raise SystemExit(f"generate-agents-override: missing source file: {relative_path}")
    content = path.read_text(encoding="utf-8")
    if not content:
        raise SystemExit(f"generate-agents-override: empty source file: {relative_path}")
    return content


def generate() -> str:
    return "".join(read_source(relative_path) for relative_path in SOURCE_RELATIVE_PATHS)


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
