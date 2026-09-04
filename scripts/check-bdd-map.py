#!/usr/bin/env python3
"""Validate canonical BDD inventory and strict behavior-to-evidence mappings."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

SCENARIO_RE = re.compile(r"(?m)^\s*Scenario(?: Outline)?:\s+([A-Za-z0-9][A-Za-z0-9_-]*)\b")
FEATURE_NAME_RE = re.compile(r"([A-Za-z0-9][A-Za-z0-9_.-]*\.feature)")
DEV_FEATURE_RE = re.compile(r"docs/bdd/([A-Za-z0-9][A-Za-z0-9_.-]*\.feature)")


def load_manifest(repo: Path) -> dict:
    path = repo / "docs" / "bdd" / "behavior-map-v2.json"
    return json.loads(path.read_text(encoding="utf-8"))


def evidence_path(ref: str) -> str:
    return ref.split("::", 1)[0].strip()


def section(text: str, heading: str) -> str:
    match = re.search(
        rf"(?ms)^##\s+{re.escape(heading)}\s*$([\s\S]*?)(?=^##\s+|\Z)",
        text,
    )
    return match.group(1) if match else ""


def bdd_readme_inventory(repo: Path) -> set[str]:
    text = (repo / "docs" / "bdd" / "README.md").read_text(encoding="utf-8")
    block = section(text, "Canonical spec files")
    return set(FEATURE_NAME_RE.findall(block))


def development_reference_inventory(repo: Path) -> set[str]:
    text = (repo / "DEVELOPMENT_REFERENCE.md").read_text(encoding="utf-8")
    block = section(text, "Behavior specs (BDD)")
    return set(DEV_FEATURE_RE.findall(block))


def compare_inventory(label: str, observed: set[str], canonical: set[str]) -> list[str]:
    errors: list[str] = []
    missing = sorted(canonical - observed)
    stale = sorted(observed - canonical)
    if missing:
        errors.append(f"{label} missing feature(s): {', '.join(missing)}")
    if stale:
        errors.append(f"{label} references non-canonical feature(s): {', '.join(stale)}")
    return errors


def validate_repo(repo: Path) -> list[str]:
    errors: list[str] = []
    manifest = load_manifest(repo)
    entries = manifest.get("features", [])
    declared = [entry.get("file", "") for entry in entries]
    declared_set = set(declared)
    actual = sorted(path.name for path in (repo / "docs" / "bdd").glob("*.feature"))
    actual_set = set(actual)

    if len(declared) != len(declared_set):
        errors.append("behavior-map-v2.json contains duplicate feature entries")
    if declared_set != actual_set:
        missing = sorted(actual_set - declared_set)
        stale = sorted(declared_set - actual_set)
        if missing:
            errors.append(f"manifest missing feature(s): {', '.join(missing)}")
        if stale:
            errors.append(f"manifest references missing feature(s): {', '.join(stale)}")

    errors.extend(compare_inventory("docs/bdd/README.md canonical inventory", bdd_readme_inventory(repo), declared_set))
    errors.extend(compare_inventory("DEVELOPMENT_REFERENCE.md BDD inventory", development_reference_inventory(repo), declared_set))

    for entry in entries:
        file_name = entry.get("file", "")
        if file_name not in actual_set:
            continue
        mode = entry.get("mode", "legacy")
        if mode not in {"legacy", "strict"}:
            errors.append(f"{file_name}: mode must be legacy or strict")
            continue
        if mode == "legacy":
            continue

        feature_path = repo / "docs" / "bdd" / file_name
        ids = SCENARIO_RE.findall(feature_path.read_text(encoding="utf-8"))
        if len(ids) != len(set(ids)):
            errors.append(f"{file_name}: duplicate scenario ID")
        if not ids:
            errors.append(f"{file_name}: strict feature has no inline scenario IDs")
            continue

        mappings = entry.get("mappings", [])
        mapped_ids = [mapping.get("scenario", "") for mapping in mappings]
        missing_maps = sorted(set(ids) - set(mapped_ids))
        unknown_maps = sorted(set(mapped_ids) - set(ids))
        if missing_maps:
            errors.append(f"{file_name}: unmapped scenario(s): {', '.join(missing_maps)}")
        if unknown_maps:
            errors.append(f"{file_name}: mapping references unknown scenario(s): {', '.join(unknown_maps)}")

        for mapping in mappings:
            scenario = mapping.get("scenario", "<unknown>")
            refs = mapping.get("evidence", [])
            tasks = mapping.get("tasks", [])
            if not refs:
                errors.append(f"{file_name}: {scenario}: no executable evidence")
            if not tasks:
                errors.append(f"{file_name}: {scenario}: no mapped task")
            for ref in refs:
                rel = evidence_path(ref)
                if rel.endswith(".feature"):
                    errors.append(f"{file_name}: {scenario}: specification cannot be executable evidence: {rel}")
                    continue
                if not (repo / rel).is_file():
                    errors.append(f"{file_name}: {scenario}: missing evidence file '{rel}'")
    return errors


def main() -> int:
    repo = Path(__file__).resolve().parents[1]
    errors = validate_repo(repo)
    if errors:
        print("BDD mapping check failed:")
        for error in errors:
            print(f" - {error}")
        return 1
    print("BDD mapping check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
