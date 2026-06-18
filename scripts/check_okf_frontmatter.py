#!/usr/bin/env python3
"""OKF frontmatter validator — enforces docs/knowledge/README.md contract."""
import os
import re
import sys
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent

# Closed type vocabulary: type -> glob pattern relative to repo root.
# Order matters for the location check (most-specific first).
VOCAB: dict[str, re.Pattern[str]] = {
    "Roadmap":      re.compile(r"^docs/plan/roadmap\.md$"),
    "ADR":          re.compile(r"^docs/adr/ADR-\d+.*\.md$"),
    "Playbook":     re.compile(r"^docs/playbooks/[^/]+\.md$"),
    "Policy":       re.compile(r"^docs/policies/[^/]+\.md$"),
    "Plan":         re.compile(r"^docs/plan/(?!roadmap\.md)[^/]+\.md$"),
    "TaskList":     re.compile(r"^docs/tasks/[^/]+\.md$"),
    "Architecture": re.compile(r"^docs/architecture\.md$"),
    "Proposal":     re.compile(r"^docs/proposals/[^/]+\.md$"),
    "Audit":        re.compile(r"^docs/audit/[^/]+\.md$"),
    "Prompt":       re.compile(r"^docs/prompts/[^/]+\.md$"),
}

# Index READMEs that are pure navigation files — skip them.
INDEX_READMES = {
    "docs/adr/README.md",
    "docs/playbooks/README.md",
    "docs/policies/README.md",
    "docs/plan/README.md",
    "docs/tasks/README.md",
    "docs/proposals/README.md",
    "docs/audit/README.md",
    "docs/prompts/README.md",
    "docs/knowledge/README.md",
}

SKIP_PATTERNS = [
    re.compile(r"^docs/daily/"),
    re.compile(r"(^|/)TEMPLATE\.md$"),
]


def _rel(path: Path) -> str:
    return str(path.relative_to(REPO_ROOT))


def should_skip(rel: str) -> bool:
    if rel in INDEX_READMES:
        return True
    return any(p.search(rel) for p in SKIP_PATTERNS)


def parse_frontmatter(text: str) -> dict | None:
    """Return parsed YAML frontmatter dict, or None if absent/malformed."""
    if not text.startswith("---"):
        return None
    end = text.find("\n---", 3)
    if end == -1:
        return None
    block = text[3:end].strip()
    try:
        data = yaml.safe_load(block)
        if isinstance(data, dict):
            return data
        return None
    except yaml.YAMLError:
        return None


def extract_prose_status(text: str) -> str | None:
    """Extract the status token from the prose '- **Status:** ...' line."""
    m = re.search(r"^- \*\*Status:\*\*\s*(.+)$", text, re.MULTILINE)
    if not m:
        return None
    raw = m.group(1).strip()
    for prefix in ("Proposed", "Accepted", "Superseded", "Deprecated"):
        if raw.startswith(prefix):
            return prefix
    return None


def type_matches_location(type_: str, rel: str) -> bool:
    pattern = VOCAB.get(type_)
    if pattern is None:
        return False
    return bool(pattern.match(rel))


def adr_exists(ref: str) -> bool:
    """True if docs/adr/<ref>-*.md exists (ref is e.g. 'ADR-006')."""
    matches = list((REPO_ROOT / "docs" / "adr").glob(f"{ref}-*.md"))
    return len(matches) > 0


def collect_in_scope_files() -> list[Path]:
    docs = REPO_ROOT / "docs"
    files = []
    for path in sorted(docs.rglob("*.md")):
        rel = _rel(path)
        if should_skip(rel):
            continue
        # Only include files whose path falls under a known vocab location.
        if any(p.match(rel) for p in VOCAB.values()):
            files.append(path)
    return files


def validate(files: list[Path]) -> list[str]:
    errors: list[str] = []

    for path in files:
        rel = _rel(path)
        text = path.read_text(encoding="utf-8")

        fm = parse_frontmatter(text)
        if fm is None:
            errors.append(f"{rel}: missing or malformed frontmatter block")
            continue

        type_ = fm.get("type")
        if type_ not in VOCAB:
            errors.append(
                f"{rel}: 'type' value {type_!r} is not in the closed vocabulary"
            )
            continue

        if not type_matches_location(type_, rel):
            errors.append(
                f"{rel}: 'type: {type_}' does not match file location"
            )
            continue

        if type_ == "ADR":
            fm_status = fm.get("status")
            prose_status = extract_prose_status(text)
            if fm_status != prose_status:
                errors.append(
                    f"{rel}: frontmatter status={fm_status!r} does not match "
                    f"prose status={prose_status!r}"
                )

        if type_ in ("Plan", "TaskList"):
            governed_by = fm.get("governed_by") or []
            if isinstance(governed_by, str):
                governed_by = [governed_by]
            for ref in governed_by:
                if not adr_exists(str(ref)):
                    errors.append(
                        f"{rel}: governed_by ref {ref!r} does not resolve to "
                        f"an existing docs/adr/{ref}-*.md file"
                    )

    return errors


def main() -> int:
    files = collect_in_scope_files()
    errors = validate(files)

    if errors:
        print("OKF frontmatter check failed:")
        for e in errors:
            print(f" - {e}")
        return 1

    print("OKF frontmatter check passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
