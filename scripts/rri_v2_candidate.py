#!/usr/bin/env python3
"""Calculate the RRI v2 ordinal technical summary from an assessment JSON file.

ADR-045 (2026-09-07) promoted rri-v2-design-0.2 to authoritative: the ICI/
bottleneck this tool computes is one of the two inputs `scripts/rri.py` now
uses to determine the final band (the other being the D/P/K risk floor,
which this tool's schema deliberately excludes from ICI). Most tasks should
score through `scripts/rri.py`'s CLI directly; use this tool when a fuller,
schema-validated technical assessment record (explicit per-axis evidence,
ranges, uncertainty) is warranted on its own, independent of that CLI.

This tool still never emits a calibrated effort prediction: P50/P90 remain
null and `prediction_status` stays `not_calibrated` until a real prospective
cohort and fitted coefficients exist (rri-v2-formula.md §5). It does not
independently decide HITL/routing outside the `scripts/rri.py` combination.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any


SCHEMA_VERSION = "rri-v2-design-0.2"
MODE = "candidate_only"
AXES = ("L", "I", "Q", "V")
SCHEMA_PATH = Path(__file__).resolve().parent.parent / "docs" / "schemas" / (
    "rri-v2-candidate-assessment.schema.json"
)


class ValidationError(ValueError):
    """A fail-closed validation error with a stable field path."""

    def __init__(self, path: str, message: str):
        super().__init__(message)
        self.path = path
        self.message = message


def _fail(path: str, message: str) -> None:
    raise ValidationError(path, message)


def _object(value: Any, path: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        _fail(path, "must be an object")
    return value


def _exact_keys(value: Any, required: set[str], optional: set[str], path: str) -> dict[str, Any]:
    obj = _object(value, path)
    missing = required - obj.keys()
    if missing:
        _fail(path, f"missing required key {sorted(missing)[0]!r}")
    extra = obj.keys() - required - optional
    if extra:
        _fail(path, f"unknown key {sorted(extra)[0]!r}")
    return obj


def _nonempty_string(value: Any, path: str) -> str:
    if not isinstance(value, str) or not value.strip():
        _fail(path, "must be a non-empty string")
    return value


def _evidence_refs(value: Any, path: str) -> list[str]:
    if not isinstance(value, list):
        _fail(path, "must be an array")
    return [_nonempty_string(item, f"{path}[{index}]") for index, item in enumerate(value)]


def _level(value: Any, path: str) -> int:
    if type(value) is not int or not 0 <= value <= 4:
        _fail(path, "must be an integer from 0 through 4 (booleans are invalid)")
    return value


def _range(value: Any, path: str) -> tuple[int, int]:
    if not isinstance(value, list) or len(value) != 2:
        _fail(path, "must be a two-item inclusive level range")
    lower = _level(value[0], f"{path}[0]")
    upper = _level(value[1], f"{path}[1]")
    if lower > upper:
        _fail(path, "lower bound must not exceed upper bound")
    return lower, upper


def validate_schema_document(schema_path: Path = SCHEMA_PATH) -> dict[str, Any]:
    """Load and sanity-check the published Draft 2020-12 contract.

    Runtime validation below is intentionally standard-library-only. This check
    catches a missing, malformed, or mismatched published schema rather than
    silently treating the Python implementation as the only contract.
    """
    try:
        with schema_path.open(encoding="utf-8") as handle:
            schema = json.load(handle, parse_constant=_reject_nonfinite_json)
    except OSError as exc:
        raise ValidationError("schema", f"cannot read schema: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise ValidationError("schema", f"invalid JSON schema: {exc.msg}") from exc
    _exact_keys(
        schema,
        {"$schema", "$id", "title", "description", "type", "required", "properties", "additionalProperties", "$defs"},
        set(),
        "schema",
    )
    if schema["$schema"] != "https://json-schema.org/draft/2020-12/schema":
        _fail("schema.$schema", "must declare Draft 2020-12")
    if schema["type"] != "object" or schema["additionalProperties"] is not False:
        _fail("schema", "must declare a closed top-level object")
    properties = _object(schema["properties"], "schema.properties")
    if properties.get("schema_version", {}).get("const") != SCHEMA_VERSION:
        _fail("schema.properties.schema_version", "must pin the candidate version")
    if properties.get("mode", {}).get("const") != MODE:
        _fail("schema.properties.mode", "must pin candidate_only mode")
    _object(schema["$defs"], "schema.$defs")
    axes = _object(properties.get("axes"), "schema.properties.axes")
    axis_properties = _object(axes.get("properties"), "schema.properties.axes.properties")
    if set(axis_properties) != set(AXES) or axes.get("additionalProperties") is not False:
        _fail("schema.properties.axes", "must close exactly L, I, Q, and V")
    return schema


def _validate_task(value: Any) -> dict[str, Any]:
    task = _exact_keys(
        value,
        {"task_id", "lineage_id", "work_family", "kind", "baseline_sha", "snapshot_at", "measurement_stage"},
        {"parent_id"},
        "task",
    )
    for key in ("task_id", "lineage_id", "work_family", "kind", "baseline_sha", "snapshot_at"):
        _nonempty_string(task[key], f"task.{key}")
    if "parent_id" in task and task["parent_id"] is not None:
        _nonempty_string(task["parent_id"], "task.parent_id")
    if task["measurement_stage"] not in {"pre_implementation", "post_implementation"}:
        _fail("task.measurement_stage", "must be pre_implementation or post_implementation")
    return task


def _validate_axis(value: Any, path: str) -> tuple[tuple[int, int], str]:
    axis = _exact_keys(value, {"status", "method", "evidence_refs"}, {"level", "range"}, path)
    status = axis["status"]
    if status not in {"observed", "estimated", "unknown"}:
        _fail(f"{path}.status", "must be observed, estimated, or unknown")
    _nonempty_string(axis["method"], f"{path}.method")
    _evidence_refs(axis["evidence_refs"], f"{path}.evidence_refs")
    has_level, has_range = "level" in axis, "range" in axis
    if has_level == has_range:
        _fail(path, "must contain exactly one of level or range")
    if has_level:
        if status == "unknown":
            _fail(path, "unknown status requires a range, not a point level")
        level = _level(axis["level"], f"{path}.level")
        return (level, level), status
    return _range(axis["range"], f"{path}.range"), status


def _validate_size(value: Any) -> None:
    size = _exact_keys(value, {"status", "method", "value", "evidence_refs"}, set(), "size")
    if size["status"] not in {"observed", "estimated", "unknown", "not_applicable"}:
        _fail("size.status", "has an unsupported status")
    _nonempty_string(size["method"], "size.method")
    _evidence_refs(size["evidence_refs"], "size.evidence_refs")
    numeric = isinstance(size["value"], (int, float)) and not isinstance(size["value"], bool)
    if numeric and (not math.isfinite(size["value"]) or size["value"] < 0):
        _fail("size.value", "must be a finite non-negative number")
    if size["status"] in {"observed", "estimated"} and not numeric:
        _fail("size.value", "is required for observed or estimated size")
    if size["status"] in {"unknown", "not_applicable"} and size["value"] is not None:
        _fail("size.value", "must be null for unknown or not_applicable size")


def _validate_common(assessment: Any) -> dict[str, Any]:
    record = _exact_keys(
        assessment,
        {"schema_version", "mode", "task", "applicability", "size", "uncertainty", "risk"},
        {"axes"},
        "assessment",
    )
    if record["schema_version"] != SCHEMA_VERSION:
        _fail("schema_version", "unsupported candidate schema version")
    if record["mode"] != MODE:
        _fail("mode", "must be candidate_only")
    _validate_task(record["task"])
    applicability = _exact_keys(record["applicability"], {"status", "reason", "evidence_refs"}, set(), "applicability")
    if applicability["status"] not in {"applicable", "not_applicable"}:
        _fail("applicability.status", "must be applicable or not_applicable")
    _nonempty_string(applicability["reason"], "applicability.reason")
    _evidence_refs(applicability["evidence_refs"], "applicability.evidence_refs")
    _validate_size(record["size"])
    unresolved = _exact_keys(record["uncertainty"], {"unresolved"}, set(), "uncertainty")["unresolved"]
    _evidence_refs(unresolved, "uncertainty.unresolved")
    risk = _exact_keys(record["risk"], {"hazards", "likelihood"}, set(), "risk")
    _evidence_refs(risk["hazards"], "risk.hazards")
    if risk["likelihood"] is not None:
        _nonempty_string(risk["likelihood"], "risk.likelihood")
    return record


def calculate(assessment: Any) -> dict[str, Any]:
    """Validate an assessment and derive its non-authorizing candidate summary."""
    validate_schema_document()
    record = _validate_common(assessment)
    task = record["task"]
    if record["applicability"]["status"] == "not_applicable":
        if "axes" in record:
            _fail("axes", "must be omitted when the assessment is not_applicable")
        return _base_output(task["task_id"], None, record["applicability"]["reason"])

    axes = _exact_keys(record.get("axes"), set(AXES), set(), "axes")
    ranges: dict[str, tuple[int, int]] = {}
    statuses: dict[str, str] = {}
    for axis in AXES:
        ranges[axis], statuses[axis] = _validate_axis(axes[axis], f"axes.{axis}")

    lower = {axis: pair[0] for axis, pair in ranges.items()}
    upper = {axis: pair[1] for axis, pair in ranges.items()}
    complete = all(pair[0] == pair[1] and statuses[axis] != "unknown" for axis, pair in ranges.items())
    profile: dict[str, Any] = {
        axis: lower[axis] if lower[axis] == upper[axis] else [lower[axis], upper[axis]]
        for axis in AXES
    }
    bottleneck_range = [max(lower.values()), max(upper.values())]
    breadth_lower = _breadth(lower)
    breadth_upper = _breadth(upper)
    technical: dict[str, Any] = {
        "profile": profile,
        "bottleneck": bottleneck_range[0] if complete else None,
        "bottleneck_range": bottleneck_range,
        "ici": 25 * bottleneck_range[0] if complete else None,
        "ici_range": [25 * bottleneck_range[0], 25 * bottleneck_range[1]],
        "breadth": breadth_lower if complete else None,
        "breadth_range": {"lower": breadth_lower, "upper": breadth_upper},
        "scale": "ordinal",
        "assessment_status": "fully_assessed" if complete else "unresolved",
    }
    return _base_output(task["task_id"], technical, None)


def _breadth(profile: dict[str, int]) -> list[int]:
    return [sum(level >= threshold for level in profile.values()) for threshold in (4, 3, 2, 1)]


def _base_output(task_id: str, technical: dict[str, Any] | None, technical_reason: str | None) -> dict[str, Any]:
    result: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "mode": MODE,
        "task_id": task_id,
        "technical": technical,
        "effort": {"prediction_status": "not_calibrated", "p50": None, "p90": None},
        "authorization": "governed_by_existing_RRI_and_HITL",
    }
    if technical_reason is not None:
        result["technical_reason"] = technical_reason
    return result


def _reject_nonfinite_json(value: str) -> None:
    raise ValueError(f"non-finite JSON constant {value!r} is invalid")


def load_assessment(path: Path) -> Any:
    try:
        with path.open(encoding="utf-8") as handle:
            return json.load(handle, parse_constant=_reject_nonfinite_json)
    except OSError as exc:
        raise ValidationError("input", f"cannot read input: {exc}") from exc
    except (ValueError, json.JSONDecodeError) as exc:
        raise ValidationError("input", f"invalid JSON: {exc}") from exc


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True, help="assessment JSON file")
    args = parser.parse_args(argv)
    try:
        result = calculate(load_assessment(args.input))
    except ValidationError as exc:
        print(json.dumps({"error": {"code": "validation_error", "path": exc.path, "message": exc.message}}), file=sys.stderr)
        return 2
    print(json.dumps(result, sort_keys=True, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
