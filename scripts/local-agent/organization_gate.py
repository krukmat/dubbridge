#!/usr/bin/env python3
"""Deterministic organization gate for changed production code."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


MAX_EXISTING_FILE_GROWTH = 35
MAX_NEW_FILE_GROWTH = 80
MAX_MAIN_RS_GROWTH = 12
MAIN_RS_ALLOWED = re.compile(
    r"^(use\b|mod\b|#|\}|\{|let\b|let _|tracing::|[A-Za-z0-9_]+\s*=|[A-Za-z0-9_]+\(|Ok\(|Err\()"
)
MAIN_RS_FORBIDDEN = re.compile(r"\b(fn|struct|enum|trait|impl|if|match|for|while|loop)\b")
LINT_SUPPRESSION = re.compile(r"(allow\s*\(|eslint-disable|@ts-ignore|@ts-nocheck)")


@dataclass(frozen=True)
class ChangedLine:
    path: str
    text: str
    status: str
    line_no: int


def run_git(args: list[str]) -> str:
    result = subprocess.run(
        ["git", *args],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or f"git {' '.join(args)} failed")
    return result.stdout


def classify_path(path: str) -> str | None:
    if re.match(r"^(apps|crates)/.*\.rs$", path) and "/tests/" not in path and not path.endswith("_test.rs"):
        return "rust"
    if re.match(r"^mobile/src/.*\.(ts|tsx)$", path):
        return "mobile"
    return None


def is_meaningful(text: str) -> bool:
    stripped = text.strip()
    return bool(stripped and stripped not in {"{", "}", "},", ");", "];", "};"} and not stripped.startswith(("//", "/*", "*")))


def parse_added_lines(diff_text: str) -> list[ChangedLine]:
    path = None
    status = "M"
    line_no = 0
    lines: list[ChangedLine] = []
    for raw in diff_text.splitlines():
        if raw.startswith("diff --git "):
            path = None
            status = "M"
            continue
        if raw == "new file mode 100644":
            status = "A"
            continue
        if raw.startswith("+++ b/"):
            path = raw.removeprefix("+++ b/")
            continue
        if raw.startswith("@@ "):
            match = re.search(r"\+(\d+)", raw)
            line_no = int(match.group(1)) if match else 0
            continue
        if raw.startswith("+") and not raw.startswith("+++"):
            if path and classify_path(path):
                lines.append(ChangedLine(path=path, text=raw[1:], status=status, line_no=line_no))
            line_no += 1
    return lines


def changed_files(base: str | None, files: list[str] | None) -> list[str]:
    args = ["diff", "--name-only"]
    if base:
        args.append(base)
    args.append("--")
    if files:
        args.extend(files)
    return [line for line in run_git(args).splitlines() if classify_path(line)]


def added_lines_for(base: str | None, files: list[str] | None) -> list[ChangedLine]:
    args = ["diff", "--unified=0", "--no-color"]
    if base:
        args.append(base)
    args.append("--")
    if files:
        args.extend(files)
    return parse_added_lines(run_git(args))


def analyze(lines: list[ChangedLine]) -> list[dict[str, object]]:
    violations: list[dict[str, object]] = []
    grouped: dict[str, list[ChangedLine]] = {}
    for line in lines:
        if is_meaningful(line.text):
            grouped.setdefault(line.path, []).append(line)
        if LINT_SUPPRESSION.search(line.text):
            violations.append(
                {"path": line.path, "line": line.line_no, "rule": "lint_suppression", "message": "new lint suppression in production code"}
            )
    for path, path_lines in grouped.items():
        budget = MAX_NEW_FILE_GROWTH if any(line.status == "A" for line in path_lines) else MAX_EXISTING_FILE_GROWTH
        if path.endswith("/main.rs"):
            budget = min(budget, MAX_MAIN_RS_GROWTH)
            for line in path_lines:
                stripped = line.text.strip()
                if MAIN_RS_FORBIDDEN.search(stripped) or not MAIN_RS_ALLOWED.match(stripped):
                    violations.append(
                        {"path": path, "line": line.line_no, "rule": "composition_root", "message": "main.rs must stay as a thin composition root"}
                    )
                    break
        if len(path_lines) > budget:
            violations.append(
                {"path": path, "line": path_lines[0].line_no, "rule": "file_growth", "message": f"{path} adds {len(path_lines)} meaningful lines (budget {budget})"}
            )
    return violations


def build_report(base: str | None, files: list[str] | None) -> dict[str, object]:
    files_changed = changed_files(base, files)
    lines = added_lines_for(base, files_changed)
    violations = analyze(lines)
    return {
        "status": "pass" if not violations else "violation",
        "checked_files": files_changed,
        "added_meaningful_lines": sum(1 for line in lines if is_meaningful(line.text)),
        "violations": violations,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base")
    parser.add_argument("--files", nargs="*")
    args = parser.parse_args(argv)
    try:
        report = build_report(args.base, args.files)
    except Exception as exc:
        print(json.dumps({"status": "tool_failure", "error": str(exc)}))
        return 2
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
