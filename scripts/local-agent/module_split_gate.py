#!/usr/bin/env python3
"""Per-module complexity-split routing gate (ADR-040).

Pure, offline decision module. Given a task capsule describing the approved
task's allowed paths, per-path raw cyclomatic complexity, and prior repair
attempt counts, decides whether the task's implementation authorship should
split into a local tramo and a cloud tramo (ADR-040 SS3-8), or stay on the
band's existing whole-task route.

Reuses scripts/rri.py's CC->C score table and DubBridge anchor rubric
(RubricRow / first_matching_row) rather than duplicating either -- the hard
domain exclusion in ADR-040 SS4 is defined as "an anchor-rubric floor of
D/P/K >= 4", the same rubric the RRI formula itself uses, not a separate
hardcoded path-pattern list.

No file writes, no model invocation, no subprocess execution of any
implementer -- decision only, mirroring med_high_gate.py's scope.
"""

from __future__ import annotations

import importlib.util
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

_RRI_SCRIPT = Path(__file__).resolve().parent.parent / "rri.py"
_RRI_SPEC = importlib.util.spec_from_file_location("rri", _RRI_SCRIPT)
if _RRI_SPEC is None or _RRI_SPEC.loader is None:
    raise RuntimeError(f"Unable to load script spec for {_RRI_SCRIPT}")
rri = importlib.util.module_from_spec(_RRI_SPEC)
sys.modules.setdefault(_RRI_SPEC.name, rri)
_RRI_SPEC.loader.exec_module(rri)

DECISION_SPLIT = "split"
DECISION_NO_SPLIT = "no_split"
VALID_DECISIONS = (DECISION_SPLIT, DECISION_NO_SPLIT)

LOCAL_TRAMO_INITIAL_BUDGET = 2
CLOUD_TRAMO_INITIAL_BUDGET = 1

# ADR-040 SS4: a module is hard-excluded from the local tramo when any of its
# anchor-rubric floors reaches this threshold, regardless of its own CC.
_HARD_EXCLUSION_FLOOR = 4

# ADR-040 SS3: split trigger thresholds on the RRI C score (0-5), not raw CC.
_HETEROGENEOUS_HIGH_C_MIN = 2
_HETEROGENEOUS_LOW_C_MAX = 1


class GateError(Exception):
    """A fail-closed gate rejection with a stable machine-readable code."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class ModuleAssignment:
    path: str
    raw_cc: int
    c_score: int
    hard_excluded: bool
    tramo: str  # "local" | "cloud"
    reason: str


@dataclass(frozen=True)
class GateDecision:
    decision: str  # "split" | "no_split"
    reason: str
    modules: tuple[ModuleAssignment, ...] = ()
    local_paths: tuple[str, ...] = ()
    cloud_paths: tuple[str, ...] = ()
    local_repair_budget: int = 0
    cloud_repair_budget: int = 0


def _no_split(reason: str) -> GateDecision:
    return GateDecision(decision=DECISION_NO_SPLIT, reason=reason)


def _require_capsule_fields(capsule: dict[str, Any]) -> None:
    if not isinstance(capsule, dict):
        raise GateError("invalid_capsule", "Task capsule must be a JSON object.")
    for field in ("allowed_paths", "cc_by_path"):
        if field not in capsule:
            raise GateError("missing_field", f"Task capsule is missing required field {field!r}.")
    if not isinstance(capsule["allowed_paths"], list) or not capsule["allowed_paths"]:
        raise GateError("missing_field", "Task capsule allowed_paths must be a non-empty list.")
    if not isinstance(capsule["cc_by_path"], dict):
        raise GateError("missing_field", "Task capsule cc_by_path must be an object.")


def _resolve_cc(capsule: dict[str, Any], path: str) -> int:
    """Return the raw CC for `path`, failing closed if it is missing or invalid."""
    cc_by_path = capsule["cc_by_path"]
    if path not in cc_by_path:
        raise GateError("missing_cc", f"Task capsule has no CC value for allowed path {path!r}.")
    raw_cc = cc_by_path[path]
    if not isinstance(raw_cc, int) or isinstance(raw_cc, bool) or raw_cc < 1:
        raise GateError(
            "invalid_cc",
            f"Task capsule CC value for {path!r} must be a positive integer, got {raw_cc!r}.",
        )
    return raw_cc


def _is_hard_excluded(path: str, rubric: list[Any]) -> tuple[bool, str]:
    """Return (excluded, reason) using the shared DubBridge anchor rubric."""
    row = rri.first_matching_row(path, rubric)
    if row is None:
        return False, "no anchor-rubric match"
    floors = (row.d, row.p, row.k)
    if max(floors) >= _HARD_EXCLUSION_FLOOR:
        return True, f"anchor-rubric floor D={row.d}/P={row.p}/K={row.k} on {row.label!r} ({row.adr})"
    return False, f"anchor-rubric floor D={row.d}/P={row.p}/K={row.k} below exclusion threshold"


def _normalize_allowed_path(path: str) -> str:
    normalized = str(path).removeprefix("./").rstrip("/")
    if not normalized or normalized.startswith("/") or ".." in normalized.split("/"):
        raise GateError("invalid_path", f"allowed_paths entry is not a valid repository-relative path: {path!r}")
    return normalized


def evaluate_split(
    capsule: dict[str, Any],
    *,
    rubric: list[Any] | None = None,
) -> GateDecision:
    """The single fail-closed entry point for ADR-040 per-module split routing.

    `capsule` must contain:
      - allowed_paths: list[str] -- the approved task's full file set.
      - cc_by_path: dict[str, int] -- raw cyclomatic complexity per path, for
        every entry in allowed_paths.

    Returns a `no_split` decision (ADR-040 SS3, SS5) whenever the heterogeneity
    trigger is not met or a clean disjoint partition cannot be formed. Never
    guesses a partition or silently defaults a missing/invalid CC value --
    any such ambiguity raises GateError instead of returning a decision,
    mirroring med_high_gate.py's GateError / fail-closed pattern.
    """
    _require_capsule_fields(capsule)

    if rubric is None:
        rubric = rri.resolve_platform("dubbridge").rubric

    allowed_paths = [_normalize_allowed_path(p) for p in capsule["allowed_paths"]]
    if len(set(allowed_paths)) != len(allowed_paths):
        raise GateError("duplicate_path", "Task capsule allowed_paths contains duplicate entries.")

    if len(allowed_paths) < 2:
        return _no_split("single-file task; per-module split does not apply (ADR-040 SS1).")

    modules: list[ModuleAssignment] = []
    for path in allowed_paths:
        raw_cc = _resolve_cc(capsule, path)
        c_score = rri.cc_to_score(raw_cc)
        hard_excluded, exclusion_reason = _is_hard_excluded(path, rubric)

        # Exclusion check runs independently of, and before, CC-based tramo
        # assignment (ADR-040 SS4) -- a low-CC excluded-path module can never
        # reach the local tramo through the CC branch below.
        if hard_excluded:
            tramo = "cloud"
            reason = f"hard-excluded: {exclusion_reason}"
        elif c_score >= _HETEROGENEOUS_HIGH_C_MIN:
            tramo = "cloud"
            reason = f"C={c_score} (CC={raw_cc}) >= {_HETEROGENEOUS_HIGH_C_MIN}"
        elif c_score <= _HETEROGENEOUS_LOW_C_MAX:
            tramo = "local"
            reason = f"C={c_score} (CC={raw_cc}) <= {_HETEROGENEOUS_LOW_C_MAX}, not hard-excluded"
        else:
            # Unreachable with the current 0-5 C scale and thresholds 1/2, but
            # fail closed rather than silently picking a tramo if the scale
            # or thresholds ever change out of sync.
            raise GateError(
                "unclassified_c_score",
                f"{path!r} scored C={c_score}, which falls outside both the local (<= "
                f"{_HETEROGENEOUS_LOW_C_MAX}) and cloud (>= {_HETEROGENEOUS_HIGH_C_MIN}) bands.",
            )

        modules.append(
            ModuleAssignment(
                path=path,
                raw_cc=raw_cc,
                c_score=c_score,
                hard_excluded=hard_excluded,
                tramo=tramo,
                reason=reason,
            )
        )

    has_high = any(m.c_score >= _HETEROGENEOUS_HIGH_C_MIN or m.hard_excluded for m in modules)
    has_low = any(
        m.c_score <= _HETEROGENEOUS_LOW_C_MAX and not m.hard_excluded for m in modules
    )
    if not (has_high and has_low):
        return _no_split(
            "uniform complexity tier; heterogeneity trigger not met (ADR-040 SS3) -- "
            "route the whole task per its band's existing rule."
        )

    local_paths = tuple(m.path for m in modules if m.tramo == "local")
    cloud_paths = tuple(m.path for m in modules if m.tramo == "cloud")

    # Disjoint-paths invariant (ADR-040 SS5): every allowed path must be
    # assigned to exactly one tramo. Constructed this way, overlap is
    # structurally impossible; completeness is what we verify.
    assigned = set(local_paths) | set(cloud_paths)
    if assigned != set(allowed_paths):
        missing = sorted(set(allowed_paths) - assigned)
        raise GateError(
            "incomplete_partition",
            f"Tramo assignment does not cover every allowed path; unassigned: {missing}.",
        )
    if not local_paths or not cloud_paths:
        return _no_split(
            "heterogeneity trigger met but tramo assignment collapsed to a single "
            "tramo after hard-exclusion routing; not a valid split (ADR-040 SS5)."
        )

    return GateDecision(
        decision=DECISION_SPLIT,
        reason="heterogeneous CC with a clean disjoint partition (ADR-040 SS3, SS5).",
        modules=tuple(modules),
        local_paths=local_paths,
        cloud_paths=cloud_paths,
        local_repair_budget=LOCAL_TRAMO_INITIAL_BUDGET,
        cloud_repair_budget=CLOUD_TRAMO_INITIAL_BUDGET,
    )


def next_cloud_action(attempts_used: int) -> str:
    """Return the next cloud-tramo action for a given attempt count (ADR-040 SS8).

    0 -> "attempt" (first attempt at the initially resolved tier)
    1 -> "escalate" (one escalation to the band's higher cloud tier)
    >=2 -> "stop" (escalated attempt also failed; stop and report blocked)
    """
    if attempts_used < 0:
        raise GateError("invalid_attempts", f"attempts_used must be >= 0, got {attempts_used}.")
    if attempts_used == 0:
        return "attempt"
    if attempts_used == CLOUD_TRAMO_INITIAL_BUDGET:
        return "escalate"
    return "stop"


def main(argv: list[str] | None = None) -> int:
    import argparse
    import json

    parser = argparse.ArgumentParser(description="Evaluate the ADR-040 module-split routing gate.")
    parser.add_argument("--capsule", required=True, help="Path to the task capsule JSON.")
    args = parser.parse_args(argv)

    try:
        capsule = json.loads(Path(args.capsule).read_text(encoding="utf-8"))
        decision = evaluate_split(capsule)
    except GateError as exc:
        print(json.dumps({"decision": DECISION_NO_SPLIT, "error": {"code": exc.code, "message": str(exc)}}))
        return 1
    except (OSError, json.JSONDecodeError) as exc:
        print(json.dumps({"decision": DECISION_NO_SPLIT, "error": {"code": "io_error", "message": str(exc)}}))
        return 1

    print(
        json.dumps(
            {
                "decision": decision.decision,
                "reason": decision.reason,
                "local_paths": list(decision.local_paths),
                "cloud_paths": list(decision.cloud_paths),
                "local_repair_budget": decision.local_repair_budget,
                "cloud_repair_budget": decision.cloud_repair_budget,
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
