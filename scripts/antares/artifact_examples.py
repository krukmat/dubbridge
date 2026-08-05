"""Example-artifact generation for the versioned artifact schema (T2d).

Split out of artifact_schema.py (T2e-pre, pure refactor, zero intended
behavior change). Fixture/example generation, not core schema.
"""

from __future__ import annotations

import importlib.util
import sys
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
Provenance = _ARTIFACT_SCHEMA_MOD.Provenance
TraceRef = _ARTIFACT_SCHEMA_MOD.TraceRef
SCHEMA_VERSION = _ARTIFACT_SCHEMA_MOD.SCHEMA_VERSION
CURRENT_REDACTION_VERSION = _ARTIFACT_SCHEMA_MOD.CURRENT_REDACTION_VERSION
ALLOWED_TRACE_STORAGE_PREFIX = _ARTIFACT_SCHEMA_MOD.ALLOWED_TRACE_STORAGE_PREFIX
SUCCESS_KINDS = _ARTIFACT_SCHEMA_MOD.SUCCESS_KINDS
TerminalStateKind = _ARTIFACT_SCHEMA_MOD.TerminalStateKind
_category_of = _ARTIFACT_SCHEMA_MOD._category_of

_ARTIFACT_VALIDATORS_MOD = _load_sibling_module("antares_artifact_validators", "artifact_validators.py")
validate_artifact = _ARTIFACT_VALIDATORS_MOD.validate_artifact


def _example_provenance() -> Provenance:
    return Provenance(
        model_version="fdtn-ai/antares-1b@example",
        runtime_version="scripts/antares@example",
        harness_version="antares-harness@example",
        packet_hash="sha256:" + "0" * 64,
        snapshot_hash="sha256:" + "1" * 64,
    )


def _example_trace_ref(kind: Any) -> TraceRef:
    return TraceRef(
        content_hash="sha256:" + "2" * 64,
        storage_uri=f"file://{ALLOWED_TRACE_STORAGE_PREFIX}{kind.value}.trace",
        byte_length=128,
        redaction_version=CURRENT_REDACTION_VERSION,
    )


def generate_example_artifacts() -> dict[Any, Artifact]:
    """One schema-valid, redacted example `Artifact` per TerminalStateKind.

    Every example carries `disposition.state = needs-human-review` (EC-3:
    nothing is created pre-closed, including negative/degraded/rejected
    results) and no raw trace content anywhere in the object graph.
    """

    examples: dict[Any, Artifact] = {}
    provenance = _example_provenance()

    for kind in TerminalStateKind:
        finding_id = f"example-{kind.value}"
        artifact_id = f"{finding_id}-r1"
        category = _category_of(kind)

        kwargs: dict[str, Any] = {
            "schema_version": SCHEMA_VERSION,
            "kind": kind,
            "finding_id": finding_id,
            "artifact_id": artifact_id,
            "provenance": provenance,
        }

        if category in ("t2a_parser", "t2b_policy"):
            if kind is TerminalStateKind.PARSED_TERMINAL_CALL:
                kwargs["argv"] = ("ls", "-la")
            elif kind is TerminalStateKind.COMMAND_PLAN_VALID:
                kwargs["argv"] = ("grep", "-n", "pattern", "file.py")
            elif kind is TerminalStateKind.SUBMITTED_VULNERABLE_FILES:
                kwargs["candidates"] = ("src/example.py",)
            elif kind is TerminalStateKind.PATH_CONTAINMENT_VALID:
                kwargs["candidates"] = ("src/example.py",)
            if kind not in SUCCESS_KINDS:
                kwargs["detail"] = f"example rejection detail for {kind.value}"
        elif category == "t2c1_execution":
            kwargs["argv"] = ("run-candidate-check",)
            kwargs["elapsed_seconds"] = 1.5
            if kind is not TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE:
                kwargs["exit_code"] = 0 if kind is TerminalStateKind.SANDBOX_EXECUTION_COMPLETE else 124
                kwargs["trace_ref"] = _example_trace_ref(kind)
            else:
                kwargs["detail"] = "example: sandbox runtime unavailable"
        elif category == "t2c2_budget":
            kwargs["elapsed_seconds"] = 42.0
            unit = {
                TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED: "bytes",
                TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED: "seconds",
                TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED: "commands",
                TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED: "commands",
            }[kind]
            kwargs["budget"] = Budget(limit=100.0, consumed=100.0, unit=unit)
            kwargs["trace_ref"] = _example_trace_ref(kind)
            if kind is TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED:
                kwargs["teardown_grace_seconds"] = 5.0
        elif category == "t2cli_execution":
            kwargs["argv"] = ("antares", "tool", "query", "--stdin")
            if kind is TerminalStateKind.CLI_BINARY_UNAVAILABLE:
                kwargs["detail"] = "example: antares-cli binary not found on PATH"
            else:
                kwargs["exit_code"] = 0 if kind is TerminalStateKind.CLI_EXECUTION_COMPLETE else 1
                kwargs["trace_ref"] = _example_trace_ref(kind)
                if kind is TerminalStateKind.CLI_EXECUTION_COMPLETE:
                    kwargs["candidates"] = ("src/example.py",)
                else:
                    kwargs["detail"] = f"example rejection detail for {kind.value}"

        artifact = Artifact(**kwargs)
        validate_artifact(artifact)
        examples[kind] = artifact

    return examples
