"""Composed T2 replay harness (T2e): parser -> policy/containment -> sandbox
-> artifact, in one deterministic entrypoint. Also hosts the direct
antares-cli subprocess dispatch path (Element 3 Subtask B, see the bottom of
this module) that is now the actual live-invocation entrypoint.

Two independent entrypoints live in this module. `dispatch_tool_call` (and
`process_tool_call`/`replay_session`, which wrap it) drives one raw
internal-schema tool-call JSON message through `tool_call_parser.parse_tool_call`
(T2a), then -- for a `terminal` request -- `command_policy.validate_command`
(T2b) and `sandbox_budget.run_budgeted` (composed T2c-1/T2c-2), stopping at the
first non-success `TerminalState`; for a submission, validates any candidate
paths via `path_containment.check_path_containment` (T2b) and checks for a
duplicate terminal submission via `tool_call_parser.check_duplicate_submission`.
Per the resolved Subtask A decision
(docs/plan/antares-local-runtime-adoption.md § Element 3), this path is
**retained only as the synthetic-fixture/replay-test path** -- it serves
`replay_fixtures.py`/`harness_test.py` and is not live-invoked. `dispatch_via_cli`
(bottom of this module) is the actual live-invocation entrypoint: it invokes
the official `antares-cli` as a direct subprocess, bypassing the internal
schema entirely. Every reached `TerminalState` from either path -- success or
rejection -- is converted into a schema-valid `Artifact` (T2d). This module
performs no command execution or path resolution of its own beyond the CLI
subprocess call itself; it never mutates `snapshot_root` (ADR-006).

Canonical-kind landmine (the central design problem this module exists to
solve): `tool_call_parser.py`, `command_policy.py`, `path_containment.py`, and
`sandbox_runner.py` (loaded internally by `sandbox_budget.py`) each load
`terminal_state.py` *unconditionally* -- a fresh `spec_from_file_location` +
`exec_module` call every time their own module body executes, without
checking `sys.modules` first (unlike this module and every T2d/T2e-pre
sibling, which use the cache-checking `_load_sibling_module` below). Each such
load creates a brand-new `TerminalStateKind` *class* object. Python `Enum`
equality/`in`-membership is class-identity-based, so a `TerminalState.kind`
returned by `tool_call_parser.parse_tool_call` is **not** `==`- or
`in`-comparable against `artifact_schema.T2A_KINDS` (a different generation of
the same enum) even though both ultimately come from executing the same
`terminal_state.py` source text. Left unhandled, this would make
`artifact_schema._category_of`/`validate_artifact` spuriously reject every
composed artifact. `_canonical_kind` below is the single, deliberate boundary
where a `kind` from any layer is re-resolved *by value string* into this
module's own canonical generation (`artifact_schema`'s own
`TerminalStateKind`) before it is ever used for category dispatch or embedded
in an `Artifact`. `.value` string equality and the `TerminalState.is_success`/
`is_terminal_submission` properties are safe to use directly on any
generation's instances without this conversion (each is evaluated against
that instance's own class's own module-level frozensets, which are always
self-consistent per-instance by construction); only cross-module identity
comparisons and `Artifact(kind=...)` construction need `_canonical_kind`.
"""

from __future__ import annotations

import dataclasses
import importlib.util
import json
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable


def _load_sibling_module(module_name: str, filename: str):
    """Load `filename` as `module_name`, reusing an already-loaded copy from
    `sys.modules` if one exists. See artifact_schema.py's docstring / T2e-pre
    EC-4 for why this check-first pattern is required for siblings that must
    share class identity."""
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


_TOOL_CALL_PARSER_MOD = _load_sibling_module("antares_tool_call_parser", "tool_call_parser.py")
parse_tool_call = _TOOL_CALL_PARSER_MOD.parse_tool_call
check_duplicate_submission = _TOOL_CALL_PARSER_MOD.check_duplicate_submission

_COMMAND_POLICY_MOD = _load_sibling_module("antares_command_policy", "command_policy.py")
validate_command = _COMMAND_POLICY_MOD.validate_command

# Loaded after command_policy.py so that if command_policy.py's own internal
# (unconditional) load already populated sys.modules["antares_path_containment"],
# this reuses that same object -- a nice-to-have consistency, not a
# correctness requirement (see module docstring: `_canonical_kind` is what
# actually guarantees correctness regardless of which generation this is).
_PATH_CONTAINMENT_MOD = _load_sibling_module("antares_path_containment", "path_containment.py")
check_path_containment = _PATH_CONTAINMENT_MOD.check_path_containment

# sandbox_budget.py is loaded lazily (first real use), not here at harness.py's
# own import time -- unlike every other sibling above. sandbox_budget.py
# contains its own internal self-check
# (`if TerminalState is not _SANDBOX_SESSION_BUDGET_MOD.TerminalState: raise`,
# T2e-pre EC-4) asserting that its own captured TerminalState generation
# matches sandbox_session_budget.py's. That check is always satisfied within
# a single execution of sandbox_budget.py's module body (both loads happen
# back-to-back with nothing intervening) -- but sandbox_budget_test.py's own
# top-level loader re-executes sandbox_budget.py *unconditionally* (it does
# not cache-check first, unlike this file's `_load_sibling_module`). If this
# module also eagerly cache-populated "antares_sandbox_budget" at its own
# import time -- i.e. at harness_test.py's collection time -- an unrelated
# test file collected in between (e.g. path_containment_test.py, whose own
# raw reload of path_containment.py unconditionally reloads terminal_state.py
# too) can overwrite the shared "antares_terminal_state" cache entry before
# sandbox_budget_test.py's later, independent re-execution runs, desyncing
# the two captures and tripping the assertion -- confirmed empirically: the
# combined `pytest scripts/antares/` run only fails when this module's
# sandbox_budget.py load is eager. Deferring to first real call means this
# module's own load happens during test *execution*, which pytest always
# runs after every file's collection-time imports (including
# sandbox_budget_test.py's) have already completed.
_SANDBOX_BUDGET_MOD_CACHE: Any = None


def _sandbox_budget_mod() -> Any:
    global _SANDBOX_BUDGET_MOD_CACHE
    if _SANDBOX_BUDGET_MOD_CACHE is None:
        _SANDBOX_BUDGET_MOD_CACHE = _load_sibling_module("antares_sandbox_budget", "sandbox_budget.py")
    return _SANDBOX_BUDGET_MOD_CACHE


# Deferring this one load does not weaken the canonical-kind guarantee above:
# `_canonical_kind` re-resolves *every* state by value string regardless of
# which generation produced it or when that generation was loaded (eager at
# this module's import time, or lazy on first `dispatch_tool_call`). Whether
# `run_budgeted`'s own TerminalStateKind generation matches
# tool_call_parser's/command_policy's/path_containment's is irrelevant by
# construction -- none of them are ever compared to each other directly, only
# each individually reconciled against the canonical generation at the point
# of use. The only externally-visible effect of deferring this load is a
# one-time latency cost on a process's first `terminal` command dispatch,
# identical in kind to any other lazily-imported Python module.


def __getattr__(name: str) -> Any:
    """PEP 562 lazy module attribute resolution for the three sandbox_budget.py
    names this module re-exports (`SessionBudget`, `run_budgeted`,
    `DEFAULT_OUTPUT_CAP_BYTES`) -- keeps `harness.SessionBudget`-style access
    working exactly as if it were an eager top-level import, without
    triggering the load it defers. See `_sandbox_budget_mod`'s comment
    above."""
    if name in ("SessionBudget", "run_budgeted", "DEFAULT_OUTPUT_CAP_BYTES"):
        return getattr(_sandbox_budget_mod(), name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def _default_session_budget() -> Any:
    return _sandbox_budget_mod().SessionBudget()


# Mirrors sandbox_budget.DEFAULT_OUTPUT_CAP_BYTES's value (1 MiB) as a plain
# local constant so `terminal_state_to_artifact`'s keyword default does not
# itself force the lazy sandbox_budget.py load at function-definition time
# (a `def foo(x=SOME_EXPR):` default is evaluated once, eagerly, when the
# `def` statement runs).
_DEFAULT_OUTPUT_CAP_BYTES = 1024 * 1024

_ARTIFACT_SCHEMA_MOD = _load_sibling_module("antares_artifact_schema", "artifact_schema.py")
TerminalStateKind = _ARTIFACT_SCHEMA_MOD.TerminalStateKind  # the canonical generation
# TerminalState (the dataclass, not TerminalStateKind) is not re-exported by
# artifact_schema.py -- loaded directly here via the same cache-checking
# loader, which reuses artifact_schema.py's already-cached
# "antares_terminal_state" module entry rather than re-executing the file.
TerminalState = _load_sibling_module("antares_terminal_state", "terminal_state.py").TerminalState
SCHEMA_VERSION = _ARTIFACT_SCHEMA_MOD.SCHEMA_VERSION
Provenance = _ARTIFACT_SCHEMA_MOD.Provenance
Disposition = _ARTIFACT_SCHEMA_MOD.Disposition
DispositionState = _ARTIFACT_SCHEMA_MOD.DispositionState
Budget = _ARTIFACT_SCHEMA_MOD.Budget
TraceRef = _ARTIFACT_SCHEMA_MOD.TraceRef
Artifact = _ARTIFACT_SCHEMA_MOD.Artifact
validate_artifact = _ARTIFACT_SCHEMA_MOD.validate_artifact
write_raw_trace = _ARTIFACT_SCHEMA_MOD.write_raw_trace
_category_of = _ARTIFACT_SCHEMA_MOD._category_of

# sandbox_runner.py/sandbox_budget.py do not always populate TerminalState.exit_code
# on a killed/timed-out process, even though the real (frequently
# signal-negative) subprocess.returncode is available at the point their own
# code builds the returned TerminalState -- see the SANDBOX_COMMAND_TIMED_OUT
# branches of both modules. Because "out of scope: modifying any existing
# layer file's public behavior" forbids fixing this at the source, the
# converter below records this documented sentinel instead of fabricating a
# plausible-looking real exit status. Chosen far outside the real range of
# POSIX exit statuses (0-255) and signal-terminated codes (roughly -1..-64)
# so it can never be mistaken for genuine process-exit data.
_EXIT_CODE_UNAVAILABLE_SENTINEL = -1_000_000


def _canonical_kind(kind: Any) -> Any:
    """Re-resolve `kind` (from any TerminalStateKind generation) into this
    module's own canonical generation, by value string. See the module
    docstring's "Canonical-kind landmine" section."""
    return TerminalStateKind(kind.value)


@dataclass
class HarnessSession:
    """One Antares agentic-loop session, scoped to exactly one read-only
    `snapshot_root`. Not reused across sessions -- mirrors `SessionBudget`'s
    own one-session-per-instance contract, which this wraps.
    """

    snapshot_root: Path
    budget: Any = field(default_factory=_default_session_budget)
    command_timeout_seconds: float = 10.0
    output_cap_bytes: int = _DEFAULT_OUTPUT_CAP_BYTES
    network_isolation: Any | None = None
    last_submission: Any = field(default=None, repr=False)


def dispatch_tool_call(raw_json: str, session: HarnessSession) -> Any:
    """Drive one raw tool-call message through the composed pipeline,
    stopping at the first non-success `TerminalState`. Returns the raw
    `TerminalState` reached (any generation) -- see `terminal_state_to_artifact`
    for the conversion step.

    HP-1/HP-2: a fully valid `terminal` command or submission resolves to its
    own successful terminal state. EC-1: a session whose `budget` is already
    exhausted refuses the next command before starting it (via
    `run_budgeted`'s own pre-flight check). EC-2/EC-3: a parser, policy, or
    containment rejection is returned unchanged, in its own distinct kind, and
    the pipeline never proceeds past it. EC-4: a policy-approved command that
    hangs or floods output resolves through `run_budgeted`'s own bounded
    timeout/cap/wall-budget enforcement, never an unbounded hang.
    """
    parsed = parse_tool_call(raw_json)
    if not parsed.is_success:
        return parsed

    if parsed.is_terminal_submission:
        if session.last_submission is not None:
            deduped = check_duplicate_submission(session.last_submission, parsed)
            if deduped.kind.value == "duplicate_terminal_submission":
                return deduped
        if parsed.kind.value == "submitted_vulnerable_files":
            containment = check_path_containment(parsed.candidates, session.snapshot_root)
            if not containment.is_success:
                return containment
            # Validated but unchanged -- check_path_containment preserves the
            # input strings/order on success, so the original submission
            # state already carries the validated candidates.
        session.last_submission = parsed
        return parsed

    # The only remaining T2a success kind is a `terminal` command request.
    command_state = validate_command(parsed.argv, session.snapshot_root)
    if not command_state.is_success:
        return command_state

    result = _sandbox_budget_mod().run_budgeted(
        command_state.argv,
        session.snapshot_root,
        session.budget,
        command_timeout_seconds=session.command_timeout_seconds,
        output_cap_bytes=session.output_cap_bytes,
        network_isolation=session.network_isolation,
    )
    if not result.argv:
        # Backfill for the SANDBOX_RUNTIME_UNAVAILABLE paths that never
        # populate argv on the returned state (missing snapshot root, no
        # network isolation, resource limits unavailable, subprocess
        # bootstrap failure) -- artifact_validators.py requires non-empty
        # argv for every T2C1 kind, including this one.
        result = dataclasses.replace(result, argv=command_state.argv)
    return result


def _encode_trace(state: Any) -> bytes:
    """Deterministic raw-trace encoding: stdout/stderr are the only
    structural fields a sandbox execution produces beyond `kind`/`exit_code`/
    `elapsed_seconds`, so both are preserved, distinguishably, in one blob."""
    return json.dumps({"stdout": state.stdout, "stderr": state.stderr}, sort_keys=True).encode(
        "utf-8"
    )


def _budget_snapshot(
    canonical_kind: Any, session_budget: Any, state: Any, output_cap_bytes: int
) -> Budget:
    value = canonical_kind.value
    if value == "sandbox_output_cap_exceeded":
        consumed = len(state.stdout.encode("utf-8")) + len(state.stderr.encode("utf-8"))
        return Budget(limit=float(output_cap_bytes), consumed=float(consumed), unit="bytes")
    if value == "sandbox_wall_budget_exceeded":
        return Budget(
            limit=session_budget.wall_budget_seconds,
            consumed=session_budget.elapsed_seconds(),
            unit="seconds",
        )
    # sandbox_budget_exhausted / sandbox_teardown_unconfirmed: the command
    # counter is the only session-accounting figure both conditions share.
    # SessionBudget (sandbox_session_budget.py, read-only per approved scope)
    # exposes no public accessor for _commands_started; reading the private
    # field directly here (rather than adding one) keeps that file
    # unmodified, consistent with sandbox_budget_test.py's own precedent of
    # reading/writing `budget._commands_started` directly.
    return Budget(
        limit=float(session_budget.command_budget),
        consumed=float(session_budget._commands_started),
        unit="commands",
    )


def terminal_state_to_artifact(
    state: Any,
    *,
    finding_id: str,
    artifact_id: str,
    provenance: Provenance,
    trace_storage_root: Path,
    request_argv: tuple[str, ...] = (),
    session_budget: Any | None = None,
    output_cap_bytes: int = _DEFAULT_OUTPUT_CAP_BYTES,
    supersedes: str | None = None,
) -> Artifact:
    """Convert any `TerminalState` (any layer, any TerminalStateKind
    generation) into a schema-valid `Artifact`. This is the composition
    harness's one piece of genuinely new logic -- no converter from
    `TerminalState` to `Artifact` exists anywhere else in this package.

    Every artifact is created with `disposition` at its dataclass default
    (`needs-human-review`); this function never sets it explicitly, so
    nothing produced here can appear pre-closed (HITL_AUTONOMY_POLICY.md).
    """
    canonical_kind = _canonical_kind(state.kind)
    category = _category_of(canonical_kind)

    kwargs: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "kind": canonical_kind,
        "finding_id": finding_id,
        "artifact_id": artifact_id,
        "provenance": provenance,
        "supersedes": supersedes,
        "detail": state.detail,
        "argv": state.argv,
        "candidates": state.candidates,
    }

    if category in ("t2a_parser", "t2b_policy"):
        artifact = Artifact(**kwargs)
        validate_artifact(artifact)
        return artifact

    if category == "t2c1_execution":
        kwargs["argv"] = state.argv or request_argv
        kwargs["candidates"] = ()
        kwargs["elapsed_seconds"] = state.elapsed_seconds
        if canonical_kind.value == "sandbox_runtime_unavailable":
            artifact = Artifact(**kwargs)
            validate_artifact(artifact)
            return artifact
        kwargs["exit_code"] = (
            state.exit_code if state.exit_code is not None else _EXIT_CODE_UNAVAILABLE_SENTINEL
        )
        kwargs["trace_ref"] = write_raw_trace(_encode_trace(state), trace_storage_root, artifact_id)
        artifact = Artifact(**kwargs)
        validate_artifact(artifact)
        return artifact

    # category == "t2c2_budget"
    if session_budget is None:
        raise ValueError("session_budget is required to convert a t2c2_budget TerminalState.")
    kwargs["argv"] = state.argv or request_argv
    kwargs["candidates"] = ()
    kwargs["elapsed_seconds"] = state.elapsed_seconds
    kwargs["budget"] = _budget_snapshot(canonical_kind, session_budget, state, output_cap_bytes)
    kwargs["trace_ref"] = write_raw_trace(_encode_trace(state), trace_storage_root, artifact_id)
    if canonical_kind.value == "sandbox_teardown_unconfirmed":
        kwargs["teardown_grace_seconds"] = _sandbox_budget_mod()._TEARDOWN_GRACE_SECONDS
    artifact = Artifact(**kwargs)
    validate_artifact(artifact)
    return artifact


def process_tool_call(
    raw_json: str,
    session: HarnessSession,
    *,
    finding_id: str,
    artifact_id: str,
    provenance: Provenance,
    trace_storage_root: Path,
) -> Artifact:
    """One full composed round trip: raw tool-call JSON in, schema-valid
    `Artifact` out."""
    state = dispatch_tool_call(raw_json, session)
    return terminal_state_to_artifact(
        state,
        finding_id=finding_id,
        artifact_id=artifact_id,
        provenance=provenance,
        trace_storage_root=trace_storage_root,
        session_budget=session.budget,
        output_cap_bytes=session.output_cap_bytes,
    )


def replay_session(
    raw_messages: tuple[str, ...],
    session: HarnessSession,
    *,
    finding_id: str,
    provenance: Provenance,
    trace_storage_root: Path,
    artifact_id_for_index: Callable[[int], str],
) -> tuple[Artifact, ...]:
    """Replay an ordered sequence of raw tool-call messages through one
    session, producing exactly one `Artifact` per message -- every message is
    processed regardless of any individual message's outcome, since each is
    independently an auditable record (EC-2). Deterministic replay
    (`sandbox_runner_test.py`'s convention): structural/semantic fields are
    byte-identical across replays of the same fixture input; wall-clock-derived
    fields (`elapsed_seconds`) are bounded, not byte-compared, by the caller.
    """
    artifacts = []
    for index, raw_json in enumerate(raw_messages):
        artifacts.append(
            process_tool_call(
                raw_json,
                session,
                finding_id=finding_id,
                artifact_id=artifact_id_for_index(index),
                provenance=provenance,
                trace_storage_root=trace_storage_root,
            )
        )
    return tuple(artifacts)


# --- Element 3 Subtask B: direct antares-cli subprocess dispatch path ---
#
# `dispatch_via_cli` and `terminal_state.py`'s new CLI_* kinds are the
# harness's live-invocation entrypoint, per the resolved Subtask A decision
# (docs/plan/antares-local-runtime-adoption.md § Element 3): invoke the
# official `antares-cli` (Cisco reference implementation, already proven
# end-to-end in T1 R4/R5 -- docs/evaluations/antares-runtime-preflight.md)
# as a direct subprocess, rather than parsing raw model tool-call JSON
# through the internal {"tool":..., "payload":...} schema `dispatch_tool_call`
# above consumes. `dispatch_tool_call`/`process_tool_call`/`replay_session`
# are unchanged and continue to serve `replay_fixtures.py`/`harness_test.py`
# as the retained synthetic-fixture/replay-test path (Subtask C's
# disposition) -- this is an additional entrypoint, not a replacement.
#
# Output schema (confirmed by reading the antares-cli reference
# implementation's `core/service.py::WorkflowResult.to_dict`/`to_json`,
# `output/finding.py::Finding.to_dict`, at
# `.antares-runtime/antares-cli-reference/` -- personal/untracked host
# state per the plan's Design decision #5, read for schema only, not
# depended on at test time): `{"summary": {"total_findings": int, ...},
# "findings": [{"title": str, "file_path": str, "cwe_ids": [str],
# "likelihood_of_exploit": str, "submission_rank": int|omitted}, ...],
# "metadata": {...}}`. Exit code 2 signals `has_operational_failures`
# (partial/degraded run, still valid JSON) -- distinguished from a crash
# (non-zero exit with no parseable JSON) by attempting the JSON parse
# regardless of exit code first.

CLI_QUERY_SUBCOMMAND = ("tool", "query", "--stdin")
CLI_SWEEP_SUBCOMMAND = ("tool", "sweep", "--stdin")
DEFAULT_CLI_BINARY = "antares"
DEFAULT_CLI_TIMEOUT_SECONDS = 300.0


def _cli_binary_path(binary: str) -> str | None:
    """EC-2: resolve `binary` on PATH without spawning anything. Returns
    `None` if it is missing or not executable -- the caller must fail
    closed on `None` before any `subprocess.Popen` call."""
    return shutil.which(binary)


def dispatch_via_cli(
    request: dict[str, Any],
    *,
    snapshot_root: Path,
    binary: str = DEFAULT_CLI_BINARY,
    subcommand: tuple[str, ...] = CLI_QUERY_SUBCOMMAND,
    timeout_seconds: float = DEFAULT_CLI_TIMEOUT_SECONDS,
) -> Any:
    """Invoke `antares-cli` as a direct, argv-only subprocess and return a
    `TerminalState` (this module's own canonical `TerminalStateKind`
    generation -- no `_canonical_kind` reconciliation needed, since this
    path never touches `tool_call_parser.py`'s or any other sibling's
    generation).

    `request` is the CLI's own stdin JSON contract (`target`, `cwe_ids`,
    etc. -- see `.antares-runtime/antares-cli-reference/src/antares_cli/commands/tool.py`);
    the caller builds it, this function does not construct or validate its
    shape beyond passing it through as `stdin` bytes.

    HP-1: a successful run (exit 0 or 2, valid JSON) maps to
    CLI_EXECUTION_COMPLETE with `candidates` populated from the findings
    array's `file_path` values (empty tuple for a genuine
    no-vulnerability-found result -- distinguished from a crash by the kind
    itself, not by an empty `candidates` tuple, which is also valid for
    HP-1). EC-1: a non-zero exit whose stdout does not parse as JSON, or a
    timeout, is CLI_EXECUTION_FAILED. EC-1 (malformed variant): valid JSON
    that is missing the required top-level keys is CLI_OUTPUT_MALFORMED --
    kept distinct from CLI_EXECUTION_FAILED because the subprocess itself
    ran and exited in a way the CLI considers non-crashing; only the
    contract this harness expects was not met. EC-2: `binary` is not found
    on PATH -> CLI_BINARY_UNAVAILABLE, no subprocess spawned. EC-3: argv is
    `[binary, *subcommand]` only -- no shell, no string interpolation of
    `request` (it is passed as stdin bytes, never argv).
    """
    resolved_binary = _cli_binary_path(binary)
    if resolved_binary is None:
        return TerminalState(
            kind=TerminalStateKind.CLI_BINARY_UNAVAILABLE,
            detail=f"CLI binary {binary!r} was not found on PATH; no subprocess was started.",
        )

    argv = (resolved_binary, *subcommand)
    stdin_text = json.dumps(request)

    try:
        process = subprocess.Popen(
            list(argv),
            cwd=str(snapshot_root),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            shell=False,
        )
    except OSError as exc:
        return TerminalState(
            kind=TerminalStateKind.CLI_EXECUTION_FAILED,
            argv=argv,
            detail=f"Failed to start the antares-cli subprocess: {exc}",
        )

    started = time.monotonic()
    try:
        stdout, stderr = process.communicate(input=stdin_text, timeout=timeout_seconds)
    except subprocess.TimeoutExpired:
        process.kill()
        stdout, stderr = process.communicate()
        return TerminalState(
            kind=TerminalStateKind.CLI_EXECUTION_FAILED,
            argv=argv,
            stdout=stdout,
            stderr=stderr,
            elapsed_seconds=time.monotonic() - started,
            detail=f"antares-cli exceeded the {timeout_seconds}s timeout and was killed.",
        )
    elapsed = time.monotonic() - started

    exit_code = process.returncode
    # Exit 0 (completed) and exit 2 (has_operational_failures, still valid
    # JSON per tool.py) both attempt the JSON parse; any other exit code, or
    # a parse failure on 0/2, is a hard failure -- never a malformed-output
    # result, since a non-{0,2} exit means the CLI itself does not claim to
    # have produced its normal report shape.
    if exit_code not in (0, 2):
        return TerminalState(
            kind=TerminalStateKind.CLI_EXECUTION_FAILED,
            argv=argv,
            stdout=stdout,
            stderr=stderr,
            exit_code=exit_code,
            elapsed_seconds=elapsed,
            detail=f"antares-cli exited {exit_code}.",
        )

    try:
        parsed = json.loads(stdout)
    except json.JSONDecodeError as exc:
        return TerminalState(
            kind=TerminalStateKind.CLI_OUTPUT_MALFORMED,
            argv=argv,
            stdout=stdout,
            stderr=stderr,
            exit_code=exit_code,
            elapsed_seconds=elapsed,
            detail=f"antares-cli stdout was not valid JSON: {exc}",
        )

    if (
        not isinstance(parsed, dict)
        or "findings" not in parsed
        or not isinstance(parsed.get("findings"), list)
    ):
        return TerminalState(
            kind=TerminalStateKind.CLI_OUTPUT_MALFORMED,
            argv=argv,
            stdout=stdout,
            stderr=stderr,
            exit_code=exit_code,
            elapsed_seconds=elapsed,
            detail="antares-cli JSON output is missing a 'findings' list.",
        )

    candidates = tuple(
        finding["file_path"]
        for finding in parsed["findings"]
        if isinstance(finding, dict) and isinstance(finding.get("file_path"), str)
    )
    return TerminalState(
        kind=TerminalStateKind.CLI_EXECUTION_COMPLETE,
        argv=argv,
        candidates=candidates,
        stdout=stdout,
        stderr=stderr,
        exit_code=exit_code,
        elapsed_seconds=elapsed,
    )


def cli_terminal_state_to_artifact(
    state: Any,
    *,
    finding_id: str,
    artifact_id: str,
    provenance: Provenance,
    trace_storage_root: Path,
    supersedes: str | None = None,
) -> Artifact:
    """Convert a `dispatch_via_cli` `TerminalState` into a schema-valid
    `Artifact`. Separate from `terminal_state_to_artifact` above (which only
    handles the T2a-T2c2 internal-schema categories) because the CLI
    category's field requirements are simpler -- no `SessionBudget`, no
    `output_cap_bytes` -- and mixing the two dispatch tables would obscure
    which categories each converter actually supports.
    """
    canonical_kind = _canonical_kind(state.kind)
    kwargs: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "kind": canonical_kind,
        "finding_id": finding_id,
        "artifact_id": artifact_id,
        "provenance": provenance,
        "supersedes": supersedes,
        "detail": state.detail,
        "argv": state.argv,
        "candidates": state.candidates,
    }
    if canonical_kind.value != "cli_binary_unavailable":
        kwargs["exit_code"] = (
            state.exit_code if state.exit_code is not None else _EXIT_CODE_UNAVAILABLE_SENTINEL
        )
        kwargs["elapsed_seconds"] = state.elapsed_seconds
        kwargs["trace_ref"] = write_raw_trace(_encode_trace(state), trace_storage_root, artifact_id)
    artifact = Artifact(**kwargs)
    validate_artifact(artifact)
    return artifact
