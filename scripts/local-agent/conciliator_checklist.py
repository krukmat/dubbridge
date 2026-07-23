#!/usr/bin/env python3
"""Cloud conciliator checklist — read-only advisory gate for capsule handoffs.

Evaluates six items (scope, acceptance, review, budget, reflection, status_sync)
on a Capsule + ordered AttemptBundle list and reports PASS / FAIL / UNKNOWN
for each item, plus an overall verdict.
"""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class ChecklistItem:
    name: str
    status: str
    reason: str


# ── helpers ──────────────────────────────────────────────────────────────────

def _normalise_path(path: str) -> str:
    """Strip leading './' and trailing '/' from *path*."""
    normalised = path.removeprefix("./").rstrip("/")
    if not normalised and path:
        raise ValueError(f"invalid path: {path!r} normalises to empty")
    return normalised


def _get_diff_ref(bundle: Any) -> list[Any] | None:
    dr = bundle.fields.get("diff_ref")
    if isinstance(dr, list):
        return dr
    return None


def _check_scope(capsule: Any, bundles: list[Any]) -> ChecklistItem:
    allowed = capsule.fields.get("allowed_paths", [])
    if not bundles:
        return ChecklistItem(
            "scope", "PASS",
            "no attempt bundles to inspect",
        )

    diff_ref = _get_diff_ref(bundles[-1])
    paths_touched: list[str] = []
    bad_paths: list[str] = []
    for entry in diff_ref or []:
        p = entry.get("path") if isinstance(entry, dict) else None
        if isinstance(p, str):
            try:
                paths_touched.append(_normalise_path(p))
            except ValueError:
                bad_paths.append(p)

    if bad_paths:
        return ChecklistItem(
            "scope", "FAIL",
            f"invalid/unnormalisable path(s): {bad_paths}",
        )

    # empty diff_ref -> PASS (vacuously nothing out of scope)
    if not paths_touched:
        return ChecklistItem("scope", "PASS", "no paths touched in last bundle")

    # empty/missing allowed_paths with non-empty diff_ref -> FAIL
    normalised_allowed: list[str] = []
    if isinstance(allowed, list):
        for a in allowed:
            if isinstance(a, str) and a:
                normalised_allowed.append(_normalise_path(a))

    if not normalised_allowed:
        return ChecklistItem(
            "scope", "FAIL",
            f"allowed_paths is empty/missing but {len(paths_touched)} path(s) were touched",
        )

    offending: list[str] = []
    for p in paths_touched:
        if not any(
            p == allowed or p.startswith(f"{allowed}/")
            for allowed in normalised_allowed
        ):
            offending.append(p)

    if offending:
        return ChecklistItem(
            "scope", "FAIL",
            f"out-of-scope paths: {offending}",
        )
    return ChecklistItem("scope", "PASS", "all paths within allowed scope")


def _check_acceptance(bundles: list[Any]) -> ChecklistItem:
    if not bundles:
        return ChecklistItem(
            "acceptance", "PASS",
            "no attempt bundles to inspect",
        )
    test_results = bundles[-1].fields.get("test_results")

    # present but not a dict
    if test_results is not None and not isinstance(test_results, dict):
        return ChecklistItem(
            "acceptance", "UNKNOWN",
            "invalid test_results type",
        )

    if test_results is None or (isinstance(test_results, dict) and not test_results):
        # missing / None / {} -> UNKNOWN naming the expected key
        return ChecklistItem(
            "acceptance", "UNKNOWN",
            "missing test_results or empty \u2014 expecting 'passed' or 'status'",
        )

    # Now test_results is a non-empty dict
    if "passed" in test_results:
        val = test_results["passed"]
        # must be strictly bool
        if not isinstance(val, bool):
            return ChecklistItem(
                "acceptance", "UNKNOWN",
                "'passed' present but not a strict bool (type mismatch)",
            )
        if val is True:
            return ChecklistItem("acceptance", "PASS", "test_results['passed'] is True")
        # val is False
        return ChecklistItem("acceptance", "FAIL", "test_results['passed'] is False")

    # 'passed' absent: fall back to status
    status = test_results.get("status")
    if status == "ok":
        return ChecklistItem("acceptance", "PASS", "test_results['status'] is ok")
    elif isinstance(status, str) and status:
        return ChecklistItem(
            "acceptance", "FAIL",
            f"test_results has non-empty 'status' but not 'ok': {status!r}",
        )
    # neither key present
    return ChecklistItem(
        "acceptance", "UNKNOWN",
        "neither 'passed' nor 'status' present in test_results",
    )


def _check_review(bundles: list[Any]) -> ChecklistItem:
    if not bundles:
        return ChecklistItem(
            "review", "PASS",
            "no attempt bundles to inspect",
        )
    rv = bundles[-1].fields.get("review_verdict")
    if rv is None or (isinstance(rv, str) and rv.strip() == ""):
        return ChecklistItem(
            "review", "FAIL",
            "review_verdict missing or empty",
        )
    # recognized non-pending values are those that aren't "pending"
    if not isinstance(rv, str):
        # non-string but truthy-ish \u2014 treat as pending-equivalent ambiguity
        return ChecklistItem(
            "review", "UNKNOWN",
            f"review_verdict is not a string: {rv!r}",
        )
    rv_lower = rv.lower().strip()
    if rv_lower == "pending":
        return ChecklistItem(
            "review", "FAIL",
            "review_verdict is still pending",
        )
    return ChecklistItem("review", "PASS", f"review_verdict={rv!r}")


def _get_band_from_capsule(capsule: Any) -> str | None:
    """Pull 'band' or 'rri' from capsule fields if present."""
    band = capsule.fields.get("band")
    if isinstance(band, str) and band:
        return band
    rri = capsule.fields.get("rri")
    if isinstance(rri, (int, float)) and not isinstance(rri, bool):
        val = int(rri)
        if 26 <= val <= 40:
            return "Moderate"
        elif 41 <= val <= 55:
            return "Med-high"
    return None


def _band_to_budget(band: str) -> int | None:
    mapping = {"Moderate": 2, "Med-high": 1}
    return mapping.get(band)


def _get_rri_from_capsule(capsule: Any) -> int | None:
    rri = capsule.fields.get("rri")
    if isinstance(rri, (int, float)) and not isinstance(rri, bool):
        return int(rri)
    return None


def _check_budget(
    capsule: Any,
    bundles: list[Any],
    *,
    band: str | None = None,
) -> ChecklistItem:
    # Capsule fields take precedence (for forward-compatibility)
    effective_band = _get_band_from_capsule(capsule)
    if effective_band is not None:
        budget_val = _band_to_budget(effective_band)
    elif band is not None:
        effective_band = band
        budget_val = _band_to_budget(band)
    else:
        return ChecklistItem(
            "budget", "UNKNOWN",
            "no band source available (capsule fields + caller-supplied) to determine budget",
        )

    if budget_val is None:
        return ChecklistItem(
            "budget", "UNKNOWN",
            f"unrecognised band value: {effective_band!r}",
        )

    count = len(bundles)
    if count > budget_val:
        return ChecklistItem(
            "budget", "FAIL",
            f"{count} attempt(s) exceeds repair budget of {budget_val} for band {effective_band}",
        )
    return ChecklistItem(
        "budget", "PASS",
        f"{count} attempt(s) within budget of {budget_val} for band {effective_band}",
    )


def _check_reflection(
    capsule: Any,
    *,
    reflection_log_present: bool | None = None,
    caller_rri: int | None = None,
) -> ChecklistItem:
    # Determine effective RRI: capsule fields first, then caller override.
    rri = _get_rri_from_capsule(capsule)
    if rri is None and caller_rri is not None:
        rri = caller_rri

    # Still no info \u2192 UNKNOWN (skip-not-applicable).
    if rri is None:
        return ChecklistItem(
            "reflection", "UNKNOWN",
            "RRI unknown \u2014 cannot determine whether reflection is required",
        )

    # RRI is known.
    if rri >= 26:
        if reflection_log_present is False or reflection_log_present is None:
            return ChecklistItem(
                "reflection", "FAIL",
                "RRI >= 26 but no reflection log evidence supplied",
            )
        return ChecklistItem("reflection", "PASS", "RRI >= 26 and reflection log present")

    # RRI < 26: skip-not-applicable => PASS.
    return ChecklistItem("reflection", "PASS", "RRI < 26 \u2014 reflection not required")


def _check_status_sync(*, status_artifacts: list[tuple[str, bool]] | None = None) -> ChecklistItem:
    if status_artifacts is None:
        return ChecklistItem(
            "status_sync", "UNKNOWN",
            "no status artifacts supplied \u2014 cannot verify sync",
        )
    unsynced = [p for p, synced in status_artifacts if not synced]
    if unsynced:
        return ChecklistItem(
            "status_sync", "FAIL",
            f"un-synced status artifacts: {unsynced}",
        )
    return ChecklistItem("status_sync", "PASS", "all status artifacts synced")


# ── public API ────────────────────────────────────────────────────────────────


def build_report(
    capsule: Any,
    bundles: list[Any],
    *,
    band: str | None = None,
    rri: int | None = None,
    reflection_log_present: bool | None = None,
    status_artifacts: list[tuple[str, bool]] | None = None,
) -> dict[str, Any]:
    """Build the checklist report for *capsule* + ordered *bundles*.

    Parameters
    ----------
    capsule :
        A ``Capsule`` instance (must have a ``fields : dict`` attribute).
    bundles :
        Ordered list of ``AttemptBundle`` instances
        (each must have a ``fields : dict`` attribute).
    band :
        Optional caller-supplied budget band.  Capsule fields take
        precedence when available.
    rri :
        Optional caller-supplied RRI override. Used only when capsule
        fields do not carry an ``rri`` value.
    reflection_log_present :
        Optional flag indicating whether the required reflection log exists
        for RRI-26+ capsules. ``None`` (missing) is treated as ``False``
        when the RRI requirement is known to apply (i.e. RRI >= 26).
    status_artifacts :
        Optional list of ``(artifact_path, synced: bool)`` pairs.

    Returns
    -------
    dict with keys ``items`` (list[ChecklistItem]) and ``overall`` ("PASS"
    | "FAIL" | "UNKNOWN").
    """
    # Determine effective band for budget item when capsule has no band/rri.
    effective_band = _get_band_from_capsule(capsule)
    if effective_band is None and rri is not None:
        if 26 <= rri <= 40:
            effective_band = "Moderate"
        elif 41 <= rri <= 55:
            effective_band = "Med-high"

    items: list[ChecklistItem] = [
        _check_scope(capsule, bundles),
        _check_acceptance(bundles),
        _check_review(bundles),
        _check_budget(capsule, bundles, band=band or effective_band),
        _check_reflection(capsule, reflection_log_present=reflection_log_present, caller_rri=rri),
        _check_status_sync(status_artifacts=status_artifacts),
    ]

    statuses = [i.status for i in items]
    if "FAIL" in statuses:
        overall = "FAIL"
    elif "UNKNOWN" in statuses:
        overall = "UNKNOWN"
    else:
        overall = "PASS"

    return {
        "items": [vars(i) for i in items],
        "overall": overall,
    }


def print_report(report: dict[str, Any]) -> None:
    """Pretty-print *report* to stdout."""
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
    # Quick smoke-test via JSON on stdin for debugging.
    data = json.loads(sys.stdin.read())
    # minimal stub objects so the script works standalone
    class _Capsule:
        fields: dict = data["capsule"]
    class _Bundle:
        def __init__(self, f):
            self.fields = f
    c = _Capsule()
    bs = [_Bundle(b) for b in data["bundles"]]
    opts = {k: v for k, v in data.items() if k not in ("capsule", "bundles")}
    rpt = build_report(c, bs, **opts)
    print_report(rpt)