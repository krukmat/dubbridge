"""Dict (de)serialization for the versioned artifact schema (T2d).

Split out of artifact_schema.py (T2e-pre, pure refactor, zero intended
behavior change). Data Mapper: translates between the in-memory `Artifact`
object graph and its committed dict/JSON shape; owns no validation logic of
its own.
"""

from __future__ import annotations

import importlib.util
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Any


def _load_sibling_module(module_name: str, filename: str):
    """Load `filename` as `module_name`, reusing an already-loaded copy from
    `sys.modules` if one exists.

    Required once a single concern is split across sibling files that must
    share one class identity for Enum/dataclass types defined elsewhere
    (e.g. `TerminalStateKind`): `importlib.util.module_from_spec` +
    `exec_module` always re-executes a file from scratch and does not
    consult `sys.modules` on its own, so two independent loads of the same
    file produce two distinct, non-`==`-comparable class objects. See
    docs/tasks/antares-security-specialist-advisor.md § T2e-pre EC-4.
    """
    if module_name in sys.modules:
        return sys.modules[module_name]
    script = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(module_name, script)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load script spec for {script}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = mod
    spec.loader.exec_module(mod)
    return mod


_ARTIFACT_SCHEMA_MOD = _load_sibling_module("antares_artifact_schema", "artifact_schema.py")
Artifact = _ARTIFACT_SCHEMA_MOD.Artifact
Budget = _ARTIFACT_SCHEMA_MOD.Budget
Disposition = _ARTIFACT_SCHEMA_MOD.Disposition
DispositionState = _ARTIFACT_SCHEMA_MOD.DispositionState
Provenance = _ARTIFACT_SCHEMA_MOD.Provenance
TraceRef = _ARTIFACT_SCHEMA_MOD.TraceRef
TerminalStateKind = _ARTIFACT_SCHEMA_MOD.TerminalStateKind


def artifact_to_dict(artifact: Artifact) -> dict[str, Any]:
    """The committed/serialized form. Deliberately has no key for
    raw_stdout/raw_stderr -- those never leave the in-memory pre-redaction
    staging object, regardless of what `validate_artifact` catches."""

    return {
        "schema_version": artifact.schema_version,
        "kind": artifact.kind.value,
        "finding_id": artifact.finding_id,
        "artifact_id": artifact.artifact_id,
        "supersedes": artifact.supersedes,
        "provenance": asdict(artifact.provenance),
        "disposition": {
            "state": artifact.disposition.state.value,
            "reviewer": artifact.disposition.reviewer,
            "reviewed_at": artifact.disposition.reviewed_at,
            "note": artifact.disposition.note,
        },
        "trace_ref": None if artifact.trace_ref is None else asdict(artifact.trace_ref),
        "argv": list(artifact.argv),
        "candidates": list(artifact.candidates),
        "detail": artifact.detail,
        "exit_code": artifact.exit_code,
        "elapsed_seconds": artifact.elapsed_seconds,
        "budget": None if artifact.budget is None else asdict(artifact.budget),
        "teardown_grace_seconds": artifact.teardown_grace_seconds,
    }


def artifact_from_dict(data: dict[str, Any]) -> Artifact:
    disposition_data = data["disposition"]
    trace_ref_data = data.get("trace_ref")
    budget_data = data.get("budget")
    return Artifact(
        schema_version=data["schema_version"],
        kind=TerminalStateKind(data["kind"]),
        finding_id=data["finding_id"],
        artifact_id=data["artifact_id"],
        supersedes=data.get("supersedes"),
        provenance=Provenance(**data["provenance"]),
        disposition=Disposition(
            state=DispositionState(disposition_data["state"]),
            reviewer=disposition_data.get("reviewer"),
            reviewed_at=disposition_data.get("reviewed_at"),
            note=disposition_data.get("note"),
        ),
        trace_ref=None if trace_ref_data is None else TraceRef(**trace_ref_data),
        argv=tuple(data.get("argv", ())),
        candidates=tuple(data.get("candidates", ())),
        detail=data.get("detail", ""),
        exit_code=data.get("exit_code"),
        elapsed_seconds=data.get("elapsed_seconds"),
        budget=None if budget_data is None else Budget(**budget_data),
        teardown_grace_seconds=data.get("teardown_grace_seconds"),
    )
