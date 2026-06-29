#!/usr/bin/env python3
"""
Enforce that color.primary is never used as a text color in type.label contexts.

type.label is 12px bold — it requires WCAG AA 4.5:1. color.primary (#E50914)
yields only 3.84:1 on canvas, which fails that threshold. Use color.primaryStrong
(#FF3333, 5.06:1) for kicker labels, eyebrows, and toggles.

Detection: for each line containing `color: color.primary`, check whether
`...type.label` appears within WINDOW lines above it in the same style block.
"""
import sys
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
MOBILE_SRC = ROOT / "mobile" / "src"
WINDOW = 8

violations = []

files = sorted(MOBILE_SRC.rglob("*.tsx")) + sorted(MOBILE_SRC.rglob("*.ts"))
for path in files:
    lines = path.read_text().splitlines()
    for i, line in enumerate(lines):
        stripped = line.strip()
        # Only care about `color: color.primary` (text color), not backgroundColor or similar.
        if "color: color.primary," not in stripped and "color: color.primary}" not in stripped:
            continue
        # primaryStrong and primaryPressed are both acceptable alternatives.
        if "primaryStrong" in stripped or "primaryPressed" in stripped:
            continue
        # Check whether ...type.label appears within WINDOW lines above.
        window_start = max(0, i - WINDOW)
        context = lines[window_start:i + 1]
        if any("...type.label" in w for w in context):
            rel = path.relative_to(ROOT)
            violations.append(f"  {rel}:{i + 1}: color.primary in type.label context — use color.primaryStrong")

if violations:
    print("FAIL: color.primary used as text color in small-text (type.label) style:")
    for v in violations:
        print(v)
    print()
    print("Fix: replace color.primary with color.primaryStrong in these label styles.")
    print("     color.primaryStrong (#FF3333) meets WCAG AA 4.5:1 on canvas.")
    sys.exit(1)

checked = len(files)
print(f"OK: no color.primary in type.label text contexts ({checked} files checked)")
