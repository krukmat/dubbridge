#!/usr/bin/env python3
"""Post-CI touchpoint CLI for the T4 observe-only pilot.

Runs `pilot.run_pilot_touchpoint` for every CWE on the T3a watchlist against
the completed revision's snapshot root, then writes exactly one redacted
JSON summary (candidate counts, dispositions-pending count, degraded flag --
never raw stdout/stderr or full candidate file lists beyond counts) to
`--out-dir`. Raw traces stay under `var/antares-traces/` (already
Git-ignored, written by the harness itself) and this script never reads or
re-emits them.

HP-3 / EC-1: this script always exits 0 -- Antares is non-blocking at the
post-CI touchpoint, and a degraded per-CWE run is recorded in the summary,
never turned into a nonzero exit that could fail the calling CI job.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.antares.cwe_watchlist import load_watchlist
from scripts.antares.disposition_ledger import DispositionLedger, Touchpoint
from scripts.antares.pilot import record_pilot_run, run_pilot_touchpoint


def build_summary(
    *, snapshot_root: Path, snapshot_id: str, triage_owner: str, retention_days: int
) -> dict:
    watchlist = load_watchlist()
    ledger = DispositionLedger()
    per_cwe: list[dict] = []

    for entry in watchlist.entries:
        result = run_pilot_touchpoint(
            touchpoint=Touchpoint.POST_CI,
            cwe_id=entry.cwe_id,
            snapshot_root=snapshot_root,
            snapshot_id=snapshot_id,
            triage_owner=triage_owner,
        )
        record_pilot_run(ledger, result)
        per_cwe.append(
            {
                "cwe_id": entry.cwe_id,
                "degraded": result.degraded,
                "candidate_count": len(result.candidates),
            }
        )

    return {
        "schema_version": 1,
        "touchpoint": "post-ci",
        "snapshot_id": snapshot_id,
        "retention_days": retention_days,
        "cwe_results": per_cwe,
        "total_candidates": sum(r["candidate_count"] for r in per_cwe),
        "degraded_count": sum(1 for r in per_cwe if r["degraded"]),
        "undisposed_count": len(ledger.undisposed()),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--snapshot-root", required=True, type=Path)
    parser.add_argument("--snapshot-id", required=True)
    parser.add_argument("--triage-owner", default="security-team")
    parser.add_argument("--retention-days", type=int, default=30)
    parser.add_argument("--out-dir", required=True, type=Path)
    args = parser.parse_args(argv)

    try:
        summary = build_summary(
            snapshot_root=args.snapshot_root,
            snapshot_id=args.snapshot_id,
            triage_owner=args.triage_owner,
            retention_days=args.retention_days,
        )
        args.out_dir.mkdir(parents=True, exist_ok=True)
        out_path = args.out_dir / "antares-post-ci-summary.json"
        out_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
        print(f"wrote {out_path}")
    except Exception as exc:  # noqa: BLE001 - EC-1: a caller-input/setup problem
        # (e.g. an empty --triage-owner) must degrade the same as an Antares
        # runtime failure, not crash the calling CI job. build_summary's own
        # per-CWE loop already degrades cleanly (pilot.run_pilot_touchpoint);
        # this only guards the outer setup/serialization path around it.
        print(f"antares post-CI summary failed, degrading: {type(exc).__name__}: {exc}", file=sys.stderr)
        args.out_dir.mkdir(parents=True, exist_ok=True)
        degraded = {"schema_version": 1, "touchpoint": "post-ci", "degraded": True, "detail": str(exc)}
        (args.out_dir / "antares-post-ci-summary.json").write_text(
            json.dumps(degraded, indent=2, sort_keys=True) + "\n"
        )

    return 0  # HP-3/EC-1: always non-blocking, regardless of per-CWE degradation.


if __name__ == "__main__":
    sys.exit(main())
