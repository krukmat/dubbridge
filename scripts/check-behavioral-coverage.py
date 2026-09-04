#!/usr/bin/env python3
"""Validate behavior-v2 task-ledger evidence across supported stacks."""
from __future__ import annotations

import re
import sys
from pathlib import Path

MARKER = "Behavioral coverage contract: behavior-v2"
SUPPORTED = {".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".yaml", ".yml", ".sh"}
SECTION_RE = re.compile(r"(?m)^##\s+(.+?)\s*$")
REF_RE = re.compile(r"`([^`]+)`")


def split_sections(text: str):
    matches = list(SECTION_RE.finditer(text))
    for i, match in enumerate(matches):
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        yield match.group(1), text[match.start():end]


def is_completed_development(section: str) -> bool:
    completed = bool(
        re.search(r"(?im)^\s*-\s*\*\*Status:\*\*.*(?:\[x\].*Done|Done)", section)
        or re.search(r"(?im)^\s*\*\*Status:\*\*.*Done", section)
    )
    development = bool(re.search(r"(?im)^\s*-\s*\*\*Type:\*\*.*development", section))
    return completed and development


def ids_under(section: str, heading: str, prefix: str) -> set[str]:
    match = re.search(
        rf"(?ims)^###\s+{re.escape(heading)}\s*$([\s\S]*?)(?=^###\s+|\Z)",
        section,
    )
    if not match:
        return set()
    return set(re.findall(rf"\b{prefix}-\d+\b", match.group(1)))


def certification_rows(section: str):
    match = re.search(
        r"(?ims)^###\s+Behavioral coverage certification\s*$([\s\S]*?)(?=^###\s+|\Z)",
        section,
    )
    if not match:
        return {}
    rows = {}
    for line in match.group(1).splitlines():
        if not re.match(r"^\|\s*(HP|EC)-\d+\s*\|", line):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) < 6:
            continue
        rows[cells[0]] = {
            "type": cells[1],
            "behavior": cells[2],
            "layer": cells[3].lower(),
            "evidence": cells[4],
            "result": cells[5].lower(),
        }
    return rows


def split_ref(ref: str):
    if "::" in ref:
        path, selector = ref.split("::", 1)
        return path.strip(), selector.strip()
    return ref.strip(), ""


def validate_reference(repo: Path, ref: str) -> str | None:
    rel, selector = split_ref(ref)
    path = repo / rel
    suffix = path.suffix.lower()
    if suffix not in SUPPORTED:
        return f"unsupported evidence type '{rel}'"
    if not path.is_file():
        return f"missing evidence file '{rel}'"

    if suffix in {".rs", ".py"}:
        if not selector:
            return f"named test selector required for '{rel}'"
        name = re.split(r"[.:>\s]+", selector)[-1]
        contents = path.read_text(encoding="utf-8")
        keyword = r"(?:async\s+)?def" if suffix == ".py" else r"(?:async\s+)?fn"
        if not re.search(rf"(?m)^\s*{keyword}\s+{re.escape(name)}\b", contents):
            return f"missing named test '{name}' in '{rel}'"
    elif suffix in {".ts", ".tsx", ".js", ".jsx"} and selector:
        contents = path.read_text(encoding="utf-8")
        if selector not in contents:
            return f"selector '{selector}' not found in '{rel}'"
    return None


def validate_task_file(repo: Path, task_file: Path) -> list[str]:
    text = task_file.read_text(encoding="utf-8")
    if MARKER not in text:
        return []

    errors: list[str] = []
    for title, section in split_sections(text):
        if not is_completed_development(section):
            continue

        hp = ids_under(section, "Happy paths considered", "HP")
        ec = ids_under(section, "Edge cases considered", "EC")
        if not hp:
            errors.append(f"{task_file}: {title}: missing stable HP-# case")
        if not ec:
            errors.append(f"{task_file}: {title}: missing stable EC-# case")

        rows = certification_rows(section)
        expected = hp | ec
        for case_id in sorted(expected):
            row = rows.get(case_id)
            if row is None:
                errors.append(f"{task_file}: {title}: missing certification row for {case_id}")
                continue
            if row["result"] != "passed":
                errors.append(f"{task_file}: {title}: {case_id} result must be 'passed'")
            refs = REF_RE.findall(row["evidence"])
            if not refs:
                errors.append(f"{task_file}: {title}: {case_id} has no backticked executable evidence")
                continue
            for ref in refs:
                issue = validate_reference(repo, ref)
                if issue:
                    errors.append(f"{task_file}: {title}: {case_id}: {issue}")

        for case_id in sorted(set(rows) - expected):
            errors.append(f"{task_file}: {title}: certification row {case_id} has no declared HP/EC case")
    return errors


def validate_repo(repo: Path) -> list[str]:
    errors: list[str] = []
    for task_file in sorted((repo / "docs" / "tasks").glob("*.md")):
        errors.extend(validate_task_file(repo, task_file))
    return errors


def main() -> int:
    repo = Path(__file__).resolve().parents[1]
    errors = validate_repo(repo)
    if errors:
        print("Behavioral coverage check failed:")
        for error in errors:
            print(f" - {error}")
        return 1
    print("Behavioral coverage check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
