#!/usr/bin/env python3
"""Validate canonical BDD inventory and strict behavior-to-evidence mappings."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

SCENARIO_RE = re.compile(r"(?m)^\s*Scenario(?: Outline)?:\s+([A-Za-z0-9][A-Za-z0-9_-]*)\b")


def load_manifest(repo: Path) -> dict:
    path = repo / "docs" / "bdd" / "behavior-map-v2.json"
    return json.loads(path.read_text(encoding="utf-8"))


def evidence_path(ref: str) -> str:
    return ref.split("::", 1)[0].strip()


def validate_repo(repo: Path) -> list[str]:
    errors: list[str] = []
    manifest = load_manifest(repo)
    entries = manifest.get("features", [])
    declared = [entry.get("file", "") for entry in entries]
    actual = sorted(path.name for path in (repo / "docs" / "bdd").glob("*.feature"))

    if len(declared) != len(set(declared)):
        errors.append("behavior-map-v2.json contains duplicate feature entries")
    if sorted(declared) != actual:
        missing = sorted(set(actual) - set(declared))
        stale = sorted(set(declared) - set(actual))
        if missing:
            errors.append(f"manifest missing feature(s): {', '.join(missing)}")
        if stale:
            errors.append(f"manifest references missing feature(s): {', '.join(stale)}")

    for entry in entries:
        file_name = entry.get("file", "")
        if file_name not in actual:
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
