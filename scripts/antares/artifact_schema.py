"""Versioned artifact schema and redacted trace-reference contract (T2d).

Normalizes every `TerminalState` produced by T2a/T2b/T2c-1/T2c-2 into a
durable, versioned `Artifact`. Two properties are structural, not just
validated: raw trace content (stdout/stderr) is never part of the
serialized/committed form -- only a `TraceRef` (hash + external URI) is --
and every artifact carries a mandatory `Disposition` so nothing produced by
this advisory-only tool can appear closed without a durable human decision
(see docs/policies/HITL_AUTONOMY_POLICY.md).

This module is the core of T2d's schema (T2e-pre decomposition, pure
refactor, zero intended behavior change): it owns the dataclasses, the
category partition, and the constants every sibling file depends on.
`artifact_validators.py` (Strategy-based field validation),
`artifact_trace_writer.py` (hash/write/verify), `artifact_serialization.py`
(Data Mapper to/from dict), and `artifact_examples.py` (fixture generation)
are re-exported here as bare module-level names so every symbol
`artifact_schema_test.py` reads off this module object (`_MOD.<name>`)
remains reachable exactly as before the split -- see
docs/tasks/antares-security-specialist-advisor.md § T2e-pre EC-2.
"""

from __future__ import annotations

import importlib.util
import sys
from dataclasses import dataclass, field
from enum import Enum
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
    file produce two distinct, non-`==`-comparable class objects -- this
    also breaks the cycle between this file and its siblings (each of them
    loads this file back for the base types, which would otherwise
    re-execute this module from scratch mid-load). See
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


_TERMINAL_STATE_MOD = _load_sibling_module("antares_terminal_state", "terminal_state.py")
TerminalStateKind = _TERMINAL_STATE_MOD.TerminalStateKind
SUCCESS_KINDS = _TERMINAL_STATE_MOD.SUCCESS_KINDS

SCHEMA_VERSION = 1
CURRENT_REDACTION_VERSION = 1
ALLOWED_TRACE_STORAGE_PREFIX = "var/antares-traces/"


class ValidationError(ValueError):
    """A fail-closed schema/consistency rejection with a stable code.

    Raised only for shape and consistency problems the validator can decide
    from the in-memory artifact alone. It never performs I/O -- see the
    module docstring and `verify_trace_ref_roundtrip` for the one check
    that does need disk access, which is a writer-module concern instead.
    """

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


class DispositionState(Enum):
    """The four durable human-disposition states named in
    AGENT_WORKFLOW_GUIDE.md's Antares authority-boundary section."""

    NEEDS_HUMAN_REVIEW = "needs-human-review"
    ACCEPTED_NOW = "accepted-now"
    ACCEPTED_FOLLOW_UP = "accepted-follow-up"
    REJECTED = "rejected"


# Category partition of all 20 TerminalStateKind values. Each artifact's
# required fields depend on which category its `kind` falls into.
T2A_KINDS = frozenset(
    {
        TerminalStateKind.PARSED_TERMINAL_CALL,
        TerminalStateKind.SUBMITTED_VULNERABLE_FILES,
        TerminalStateKind.SUBMITTED_NO_VULNERABILITY_FOUND,
        TerminalStateKind.MALFORMED_TOOL_CALL,
        TerminalStateKind.UNSUPPORTED_TOOL_NAME,
        TerminalStateKind.MALFORMED_SUBMIT_PAYLOAD,
        TerminalStateKind.DUPLICATE_TERMINAL_SUBMISSION,
    }
)
T2B_KINDS = frozenset(
    {
        TerminalStateKind.COMMAND_PLAN_VALID,
        TerminalStateKind.PATH_CONTAINMENT_VALID,
        TerminalStateKind.COMMAND_REJECTED_SHELL_SYNTAX,
        TerminalStateKind.COMMAND_REJECTED_EXECUTABLE_NOT_ALLOWED,
        TerminalStateKind.COMMAND_REJECTED_OPTION_NOT_ALLOWED,
        TerminalStateKind.PATH_REJECTED_CONTAINMENT_ESCAPE,
    }
)
T2C1_KINDS = frozenset(
    {
        TerminalStateKind.SANDBOX_EXECUTION_COMPLETE,
        TerminalStateKind.SANDBOX_RUNTIME_UNAVAILABLE,
        TerminalStateKind.SANDBOX_COMMAND_TIMED_OUT,
    }
)
T2C2_KINDS = frozenset(
    {
        TerminalStateKind.SANDBOX_OUTPUT_CAP_EXCEEDED,
        TerminalStateKind.SANDBOX_WALL_BUDGET_EXCEEDED,
        TerminalStateKind.SANDBOX_BUDGET_EXHAUSTED,
        TerminalStateKind.SANDBOX_TEARDOWN_UNCONFIRMED,
    }
)

# Self-check, not just documentation: the four category sets must exactly
# partition all 20 TerminalStateKind values with no gap and no overlap.
assert len(T2A_KINDS) == 7
assert len(T2B_KINDS) == 6
assert len(T2C1_KINDS) == 3
assert len(T2C2_KINDS) == 4
assert T2A_KINDS | T2B_KINDS | T2C1_KINDS | T2C2_KINDS == frozenset(TerminalStateKind)
assert not (T2A_KINDS & T2B_KINDS & T2C1_KINDS & T2C2_KINDS)


@dataclass(frozen=True)
class Provenance:
    """Model/runtime/harness identity plus packet/snapshot hashes.

    All five fields are mandatory and unchanged from the pre-existing
    acceptance criterion; this dataclass only gives that requirement a
    concrete shape.
    """

    model_version: str
    runtime_version: str
    harness_version: str
    packet_hash: str
    snapshot_hash: str


@dataclass(frozen=True)
class TraceRef:
    """A redacted reference to raw trace content stored outside git.

    `content_hash` is SHA-256 (prefixed `sha256:`) over the raw,
    pre-redaction bytes -- computed by the writer, never by the validator.
    `storage_uri` must resolve under `ALLOWED_TRACE_STORAGE_PREFIX`, a
    gitignored root outside anything `make qa-docs` or git tracks.
    """

    content_hash: str
    storage_uri: str
    byte_length: int
    redaction_version: int = CURRENT_REDACTION_VERSION


@dataclass(frozen=True)
class Disposition:
    """The mandatory human-disposition contract. `state` defaults to
    `needs-human-review` at creation (EC-3): nothing is ever pre-closed."""

    state: DispositionState = DispositionState.NEEDS_HUMAN_REVIEW
    reviewer: str | None = None
    reviewed_at: str | None = None
    note: str | None = None


@dataclass(frozen=True)
class Budget:
    """A T2c-2 resource-accounting snapshot at the moment a budget kind fired."""

    limit: float
    consumed: float
    unit: str


@dataclass(frozen=True)
class Artifact:
    """One versioned, redacted, human-disposition-gated terminal-state record.

    `raw_stdout`/`raw_stderr` exist only as an in-memory pre-redaction
    staging area (e.g. immediately after a sandboxed run, before the writer
    redacts them into a `trace_ref`); `to_dict` never serializes them, and
    `validate_artifact` rejects any artifact where they are non-empty
    alongside a populated `trace_ref` (EC-2) -- the committed/validated form
    can never carry both raw content and its own redacted reference.
    """

    schema_version: int
    kind: Any  # TerminalStateKind
    finding_id: str
    artifact_id: str
    provenance: Provenance
    disposition: Disposition = field(default_factory=Disposition)
    supersedes: str | None = None
    trace_ref: TraceRef | None = None
    argv: tuple[str, ...] = field(default_factory=tuple)
    candidates: tuple[str, ...] = field(default_factory=tuple)
    detail: str = ""
    exit_code: int | None = None
    elapsed_seconds: float | None = None
    budget: Budget | None = None
    teardown_grace_seconds: float | None = None
    raw_stdout: str = ""
    raw_stderr: str = ""


def _category_of(kind: Any) -> str:
    if kind in T2A_KINDS:
        return "t2a_parser"
    if kind in T2B_KINDS:
        return "t2b_policy"
    if kind in T2C1_KINDS:
        return "t2c1_execution"
    if kind in T2C2_KINDS:
        return "t2c2_budget"
    raise ValidationError("unknown_kind_category", f"{kind!r} is not mapped to any category.")


_ARTIFACT_TRACE_WRITER_MOD = _load_sibling_module("antares_artifact_trace_writer", "artifact_trace_writer.py")
compute_content_hash = _ARTIFACT_TRACE_WRITER_MOD.compute_content_hash
write_raw_trace = _ARTIFACT_TRACE_WRITER_MOD.write_raw_trace
verify_trace_ref_roundtrip = _ARTIFACT_TRACE_WRITER_MOD.verify_trace_ref_roundtrip

_ARTIFACT_SERIALIZATION_MOD = _load_sibling_module("antares_artifact_serialization", "artifact_serialization.py")
artifact_to_dict = _ARTIFACT_SERIALIZATION_MOD.artifact_to_dict
artifact_from_dict = _ARTIFACT_SERIALIZATION_MOD.artifact_from_dict

_ARTIFACT_VALIDATORS_MOD = _load_sibling_module("antares_artifact_validators", "artifact_validators.py")
validate_artifact = _ARTIFACT_VALIDATORS_MOD.validate_artifact
validate_supersede_chain = _ARTIFACT_VALIDATORS_MOD.validate_supersede_chain

_ARTIFACT_EXAMPLES_MOD = _load_sibling_module("antares_artifact_examples", "artifact_examples.py")
generate_example_artifacts = _ARTIFACT_EXAMPLES_MOD.generate_example_artifacts
