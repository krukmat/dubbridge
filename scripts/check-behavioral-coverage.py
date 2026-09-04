#!/usr/bin/env python3
"""Validate behavior-v2 task-ledger evidence across supported stacks."""
from __future__ import annotations

import re
import sys
from pathlib import Path

MARKER = "Behavioral coverage contract: behavior-v2"
SUPPORTED = {".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".yaml", ".yml", ".sh"}
ALLOWED_LAYERS = {"unit", "component", "integration", "contract", "e2e"}
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


def section_rri(section: str) -> int | None:
    match = re.search(r"(?im)^\s*-\s*\*\*RRI:\*\*\s*(\d+)\b", section)
    return int(match.group(1)) if match else None


def required_reflection_passes(rri: int) -> int:
    if rri < 26:
        return 0
    if rri <= 40:
        return 2
    if rri <= 55:
        return 3
    return 4


def validate_closure_sections(section: str, label: str) -> list[str]:
    errors: list[str] = []
    rri = section_rri(section)
    if rri is None:
        errors.append(f"{label}: completed development task must declare numeric RRI")
    else:
        required = required_reflection_passes(rri)
        if required:
            reflection = re.search(
                r"(?ims)^###\s+Reflection log\s*$([\s\S]*?)(?=^###\s+|\Z)",
                section,
            )
            if not reflection:
                errors.append(f"{label}: missing Reflection log for RRI {rri}")
            else:
                match = re.search(r"Required passes:\s*(\d+)", reflection.group(1))
                if not match or int(match.group(1)) < required:
                    errors.append(f"{label}: Reflection log must declare at least {required} required passes for RRI {rri}")

    owner = re.search(
        r"(?ims)^###\s+Owner final verification\s*$([\s\S]*?)(?=^###\s+|\Z)",
        section,
    )
    if not owner:
        errors.append(f"{label}: missing Owner final verification")
    else:
        block = owner.group(1)
        required_patterns = {
            "Owner": r"(?im)^\s*-\s*Owner:\s*\S.+$",
            "Date": r"(?im)^\s*-\s*Date:\s*\d{4}-\d{2}-\d{2}\s*$",
            "Statement": r"(?im)^\s*-\s*Statement:\s*\S.+$",
            "Commands run": r"(?im)^\s*-\s*Commands run:\s*\S.+$",
        }
        for field, pattern in required_patterns.items():
            if not re.search(pattern, block):
                errors.append(f"{label}: Owner final verification missing {field}")
    return errors


def validate_task_file(repo: Path, task_file: Path) -> list[str]:
    text = task_file.read_text(encoding="utf-8")
    if MARKER not in text:
        return []

    errors: list[str] = []
    for title, section in split_sections(text):
        if not is_completed_development(section):
            continue
        label = f"{task_file}: {title}"

        hp = ids_under(section, "Happy paths considered", "HP")
        ec = ids_under(section, "Edge cases considered", "EC")
        if not hp:
            errors.append(f"{label}: missing stable HP-# case")
        if not ec:
            errors.append(f"{label}: missing stable EC-# case")

        rows = certification_rows(section)
        expected = hp | ec
        for case_id in sorted(expected):
            row = rows.get(case_id)
            if row is None:
                errors.append(f"{label}: missing certification row for {case_id}")
                continue
            if row["layer"] not in ALLOWED_LAYERS:
                errors.append(f"{label}: {case_id} layer '{row['layer']}' must be one of: {', '.join(sorted(ALLOWED_LAYERS))}")
            if row["result"] != "passed":
                errors.append(f"{label}: {case_id} result must be 'passed'")
            refs = REF_RE.findall(row["evidence"])
            if not refs:
                errors.append(f"{label}: {case_id} has no backticked executable evidence")
                continue
            for ref in refs:
                issue = validate_reference(repo, ref)
                if issue:
                    errors.append(f"{label}: {case_id}: {issue}")

        for case_id in sorted(set(rows) - expected):
            errors.append(f"{label}: certification row {case_id} has no declared HP/EC case")

        errors.extend(validate_closure_sections(section, label))
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
