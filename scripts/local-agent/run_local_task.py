#!/usr/bin/env python3
"""Agentic runner: drives a local model through a bounded draft/test/repair loop.

Boundary enforcement (file/command allow-listing, env stripping) is owned by
T6b's `boundary` module; this task only defines the interface it calls
against, so T6b can be implemented independently without touching this file.

Transport (host normalization, streaming chat, model resolution, atomic
result writing) is reused from `gemma_local.py` rather than duplicated, per
the plan's design decision to extend the delegate-low-rri.py lineage instead
of reinventing its transport layer.

KNOWN LIMITATION (T7f): `gemma_local`, `scope_check`, and `boundary` are
imported once, below, from this script's own directory -- not from the
disposable worktree a session edits. If a task card's `allowed_paths`
includes any of those three modules, `run_loop`'s `finish`-time scope_check
call (and the boundary checks during the session) still run against the
pre-session, unedited copy of that module, not the model's changes. A
session whose task is "fix a bug in scope_check.py/boundary.py/gemma_local.py
itself" can therefore get a misleading in_scope/boundary verdict. Verify such
a session's diff independently (run its own tests against the worktree copy)
rather than trusting this runner's own gate for that narrow case.

LRPC-0b (2026-08-19): this module was reduced from a single 1491-line file to
a thin composition/CLI-entry facade under the AGENT_WORKFLOW_GUIDE.md
"Target-file size gate" (500-line ceiling on any file a local implementer
must read in full). The extracted submodules are:
- session_loop.py   -- the turn-by-turn model-interaction state machine
- audit_record.py   -- closure/evidence construction (no model interaction)
- rust_toolchain.py -- Rust-specific formatter/boundary wiring
- cli.py            -- argument parsing and process entry point
Every public symbol this module exposed before LRPC-0b is re-exported below
unchanged, so `import run_local_task as rlt; rlt.<name>` keeps working
exactly as it did before the split (Facade re-export, not a test-suite
rewrite). Behavior-preserving: no logic changed in the extraction itself.
"""

import json
import shlex
import sys

# When this file is run directly (`python3 run_local_task.py ...`, the real
# CLI path), Python executes it as module `__main__`. boundary.py separately
# does `from run_local_task import BoundaryViolation`, which -- absent this
# line -- makes Python import this same file a *second* time under the name
# `run_local_task`, producing a second, distinct BoundaryViolation class.
# run_loop's `except BoundaryViolation` (bound to the __main__ copy) then
# does not match an instance raised via boundary.py's copy, so every real
# boundary violation escaped as an uncaught traceback instead of the clean
# {"status": "boundary_violation"} result the code is designed to return.
# Confirmed live: a real qwen3.6:35b-a3b session hit exactly this crash on a
# legitimate out-of-scope write attempt. Registering this module object
# under its on-disk name before any sibling module can trigger the second
# import makes both names resolve to the identical module, so both refer to
# the same class. Tests that `import run_local_task` directly (never as
# __main__) never exercised this path, which is why 100 passing tests missed
# it.
if __name__ == "__main__":
    sys.modules.setdefault("run_local_task", sys.modules[__name__])

import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gemma_local  # noqa: E402  (re-exported; see below)
import fallback_selection  # noqa: E402  (re-exported; see below)

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scope_check  # noqa: E402  (imported for side effects parity with the pre-split module)
import subprocess  # noqa: E402  (re-exported; integration_test.py patches rlt.subprocess.Popen)

import cli
import session_loop
from audit_record import (  # noqa: F401  (re-exported)
    build_attempt_bundles,
    build_audit_record,
    build_moderate_fallback_checkpoint,
    build_terminal_attempt_packet,
)
from rust_toolchain import (  # noqa: F401  (re-exported)
    build_default_boundary,
    build_default_formatter,
    _rust_edition_for_path,
)
from session_loop import (  # noqa: F401  (re-exported)
    BoundaryViolation,
    MalformedToolCall,
    NullBoundary,
    ToolCall,
    apply_tool_call,
    parse_tool_call,
    render_authorized_context,
    require_argument,
    run_acceptance_tests,
    run_loop as _session_run_loop,
)

MAX_REPAIR_ATTEMPTS = 2
# Independent of MAX_REPAIR_ATTEMPTS (only counts failed finish validation cycles)
# and MAX_MALFORMED_BOUNCES (only counts consecutive malformed calls): a
# model that keeps issuing valid, successful edit calls without ever calling
# finish has no other bound on total turns.
# Discovered live: a real qwen3.6:35b-a3b session ran well past 300 turns in
# a single card with neither counter ever tripping, burning ~14 minutes
# before being killed manually.
MAX_TOTAL_TURNS = 30
# Raised from 1 to 3 after a real qwen3.6:35b-a3b pilot run: the model
# demonstrably can produce a well-formed deeply-nested tool-call JSON
# (confirmed — a read_file call succeeded), but intermittently drops exactly
# one closing brace on the same schema. A budget of 1 killed every pilot
# session on two consecutive misses of an error the model itself recovers
# from given another turn. This is a tolerance/threshold change, not a
# change to the boundary/security model.
MAX_MALFORMED_BOUNCES = 3
DEFAULT_IDLE_TIMEOUT_SECONDS = 180
DEFAULT_MAX_WALL_SECONDS = 1800
COMMAND_TIMEOUT_SECONDS = session_loop.COMMAND_TIMEOUT_SECONDS

# ADR-038 Amendment 3 permits a whole-task local route only for RRI 41-45
# after a GO_LOCAL receipt. It uses the Moderate 30-turn/two-repair budget
# but requires the Devstral Small 2 binding. RRI 46-55 remains cloud-only here;
# the supervisor is the route authority and this runner rejects direct bypasses.
MED_HIGH_BAND_LABEL = "Med-high"
MED_HIGH_REQUIRED_MODEL = "devstral-small-2:24b-instruct-2512-q4_K_M"
MED_HIGH_RRI_MIN = 41
MED_HIGH_LOCAL_RRI_MAX = 45
MED_HIGH_RRI_MAX = 55
LOW_RRI_MAX = 25
LOW_REQUIRED_MODEL = "qwen3.8:27b-mlx"
# Output-token budget per turn. write_file/apply_patch have no size
# cap (see runner_file_tools.py), so a turn can legitimately need to emit a
# large "content"/"replacement" string; 4096 is comfortably above any file in
# this workspace's largest source files while leaving room for the JSON
# envelope around it.
GENERATION_TOKEN_BUDGET = 8192
# Devstral Small 2 uses a 128K normal local-implementer context baseline.
# This deliberately removes the Nemotron-specific 32K operational ceiling:
# use the available budget for relevant authorized context and reduce it only
# after an observed resource/capacity symptom through the normal recovery path.
MODEL_CONTEXT_TOKENS = 131072

# TOOL_CALLING_SYSTEM_PROMPT and TOOL_CALL_JSON_SCHEMA moved to cli.py in
# LRPC-0b (chat-transport/tool-contract concerns, alongside build_live_chat_fn
# below which is the sole consumer of both); re-exported here unchanged.
from cli import TOOL_CALL_JSON_SCHEMA, TOOL_CALLING_SYSTEM_PROMPT  # noqa: E402,F401


def parse_acceptance_commands(commands):
    """Parse operator-authored commands into the exact argv capability set."""
    if not isinstance(commands, list):
        raise ValueError("acceptance_tests must be a list")

    parsed = []
    for command in commands:
        if not isinstance(command, str) or not command.strip():
            raise ValueError(f"invalid acceptance command: {command!r}")
        if "\n" in command or "\r" in command:
            raise ValueError("multiline acceptance commands are not supported")
        try:
            argv = shlex.split(command)
        except ValueError as exc:
            raise ValueError(f"unparsable acceptance command {command!r}: {exc}") from exc
        if not argv:
            raise ValueError(f"empty acceptance command: {command!r}")

        # Preserve the common `NAME=value command ...` card form without a
        # shell: execute it through `env` as explicit argv. The resulting
        # canonical argv is what the model must match exactly.
        assignments = []
        while argv and "=" in argv[0] and not argv[0].startswith("="):
            name, _value = argv[0].split("=", 1)
            if not name.replace("_", "a").isalnum() or name[0].isdigit():
                break
            assignments.append(argv.pop(0))
        if assignments:
            if not argv:
                raise ValueError(
                    f"acceptance command contains assignments but no executable: {command!r}"
                )
            argv = ["env", *assignments, *argv]

        # Shell composition is permitted only when the operator explicitly
        # names a shell executable (for example `bash -lc '...'`). Bare shell
        # syntax would otherwise become misleading literal argv under
        # shell=False, so fail card loading closed.
        shell_tokens = {"|", "||", "&&", ";", "&", "<", ">", ">>", "2>", "2>>"}
        if any(token in shell_tokens for token in argv):
            raise ValueError(
                f"bare shell composition is not supported in acceptance command: {command!r}"
            )
        parsed.append(argv)
    return parsed


class TaskCard:
    def __init__(
        self, task_id, spec, acceptance_tests, allowed_paths, rri=None, band=None,
        capsule_hash=None,
    ):
        self.task_id = task_id
        self.spec = spec
        self.acceptance_tests = acceptance_tests
        self.acceptance_argvs = parse_acceptance_commands(acceptance_tests)
        if not isinstance(allowed_paths, list):
            raise ValueError("allowed_paths must be a list")
        self.allowed_paths = allowed_paths
        self.rri = rri
        self.band = band
        # T2 (docs/tasks/local-first-cloud-local-handoff.md): the T1 capsule
        # hash this card was issued against, so bundles emitted for this
        # session can reference it. None for cards produced before T1/T2
        # existed -- build_attempt_bundles skips emission when unset rather
        # than fabricating a hash, since an invented hash would validate
        # against T1's schema syntactically while being semantically false.
        self.capsule_hash = capsule_hash


class EffectiveLimits:
    """The turn/repair/model budget this session actually runs under.

    Moderate (or a card carrying no band/RRI at all -- e.g. pre-ADR-038
    fixtures and every existing test in this file) resolves to the original
    module-level constants unchanged. Only a card whose band or RRI falls in
    the Med-high range is tightened, per ADR-038 T3.
    """

    def __init__(
        self, band, max_total_turns, max_repair_attempts, required_model,
        local_execution_allowed=True,
    ):
        self.band = band
        self.max_total_turns = max_total_turns
        self.max_repair_attempts = max_repair_attempts
        self.required_model = required_model
        self.local_execution_allowed = local_execution_allowed

    def as_dict(self):
        return {
            "band": self.band,
            "max_total_turns": self.max_total_turns,
            "max_repair_attempts": self.max_repair_attempts,
            "required_model": self.required_model,
            "local_execution_allowed": self.local_execution_allowed,
        }


def _is_med_high(card):
    band = getattr(card, "band", None)
    if isinstance(band, str) and band:
        return band == MED_HIGH_BAND_LABEL
    rri = getattr(card, "rri", None)
    if isinstance(rri, (int, float)) and not isinstance(rri, bool):
        return MED_HIGH_RRI_MIN <= int(rri) <= MED_HIGH_RRI_MAX
    return False


def _rri(card):
    value = getattr(card, "rri", None)
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return int(value)
    return None


def _is_architect_refined_local(card):
    rri = _rri(card)
    return rri is not None and MED_HIGH_RRI_MIN <= rri <= MED_HIGH_LOCAL_RRI_MAX


def _is_cloud_only_med_high(card):
    rri = _rri(card)
    return rri is not None and MED_HIGH_LOCAL_RRI_MAX < rri <= MED_HIGH_RRI_MAX


def default_local_agent_model(card):
    """Resolve the default only after reading the task card.

    Low cards retain Qwen. Moderate cards and ADR-038's permitted 41-45
    GO_LOCAL route use Devstral Small 2; callers can still request an explicit
    model, subject to the Med-high exact-binding check below.
    """
    rri = _rri(card)
    if rri is not None and rri <= LOW_RRI_MAX:
        return LOW_REQUIRED_MODEL
    return MED_HIGH_REQUIRED_MODEL


def resolve_effective_limits(card):
    """Resolve the permitted whole-task local route's limits.

    Moderate and a receipt-authorized RRI 41-45 card share the 30-turn,
    two-repair budget. The latter additionally pins the Devstral Small 2 tag.
    """
    if _is_architect_refined_local(card):
        return EffectiveLimits(
            band=MED_HIGH_BAND_LABEL,
            max_total_turns=MAX_TOTAL_TURNS,
            max_repair_attempts=MAX_REPAIR_ATTEMPTS,
            required_model=MED_HIGH_REQUIRED_MODEL,
        )
    if _is_cloud_only_med_high(card):
        return EffectiveLimits(
            band=MED_HIGH_BAND_LABEL,
            max_total_turns=MAX_TOTAL_TURNS,
            max_repair_attempts=MAX_REPAIR_ATTEMPTS,
            required_model=MED_HIGH_REQUIRED_MODEL,
            local_execution_allowed=False,
        )
    return EffectiveLimits(
        band=getattr(card, "band", None),
        max_total_turns=MAX_TOTAL_TURNS,
        max_repair_attempts=MAX_REPAIR_ATTEMPTS,
        required_model=None,
    )


def build_live_chat_fn(
    host,
    model,
    idle_timeout,
    max_wall,
    *,
    num_predict=GENERATION_TOKEN_BUDGET,
    num_ctx=MODEL_CONTEXT_TOKENS,
    max_total_turns=MAX_TOTAL_TURNS,
):
    """Thin wrapper over cli.build_live_chat_fn supplying this module's own
    defaults (GENERATION_TOKEN_BUDGET/MODEL_CONTEXT_TOKENS/MAX_TOTAL_TURNS),
    preserving the exact pre-LRPC-0b call signature and defaults for callers
    that invoke `rlt.build_live_chat_fn` without those keyword arguments."""
    return cli.build_live_chat_fn(
        host,
        model,
        idle_timeout,
        max_wall,
        num_predict=num_predict,
        num_ctx=num_ctx,
        max_total_turns=max_total_turns,
    )


def build_default_test_runner(card, boundary):
    """Run the parsed operator acceptance argv directly at finish.

    Before this existed, main()'s CLI path left test_runner=None -- unlike
    chat_fn and boundary, which both have `x or build_x(...)` fallbacks -- so
    every real `python3 run_local_task.py --card ...` session that reached
    finish crashed with `TypeError: 'NoneType' object is not callable` inside
    run_acceptance_tests. The whole local-first delegation channel could never
    close a task from the CLI; it only ever worked when an injected caller
    (the unit tests, the benchmark harness) supplied its own test_runner.
    Injected callers still override this and are unaffected.

    An empty acceptance_tests list means "no acceptance gate": finish passes
    rather than crashing, so a card with nothing to verify still closes
    cleanly instead of reintroducing the same NoneType failure by another name.
    """

    def test_runner(worktree_dir):
        outputs = []
        command_results = []
        for command, argv in zip(card.acceptance_tests, card.acceptance_argvs):
            boundary.check_command(argv)
            result = session_loop._run_command_with_timeout(argv, worktree_dir, boundary)
            command_results.append(result)
            outputs.append(
                f"$ {command}\n(exit {result['returncode']})\n"
                f"{result['stdout']}\n{result['stderr']}"
            )
            if not result["ok"]:
                return {
                    "passed": False,
                    "output": "\n\n".join(outputs),
                    "commands": command_results,
                }
        return {
            "passed": True,
            "output": "\n\n".join(outputs),
            "commands": command_results,
        }

    return test_runner


def build_initial_system_message(card, file_tools, max_total_turns):
    return session_loop.build_initial_system_message(
        card, file_tools, max_total_turns, TOOL_CALLING_SYSTEM_PROMPT
    )


def run_loop(
    card,
    chat_fn,
    test_runner,
    worktree_dir,
    boundary,
    file_tools,
    checkpoint_fn=None,
    limits=None,
    formatter_fn=None,
):
    return _session_run_loop(
        card,
        chat_fn,
        test_runner,
        worktree_dir,
        boundary,
        file_tools,
        checkpoint_fn=checkpoint_fn,
        limits=limits,
        formatter_fn=formatter_fn,
        resolve_effective_limits=resolve_effective_limits,
        max_malformed_bounces=MAX_MALFORMED_BOUNCES,
        tool_calling_system_prompt=TOOL_CALLING_SYSTEM_PROMPT,
    )


def load_card(card_path):
    return cli.load_card(card_path, TaskCard)


def parse_args(argv=None):
    return cli.parse_args(
        argv,
        default_num_ctx=MODEL_CONTEXT_TOKENS,
        default_num_predict=GENERATION_TOKEN_BUDGET,
    )


def main(
    argv=None,
    chat_fn=None,
    test_runner=None,
    boundary=None,
):
    return cli.main(
        argv,
        chat_fn=chat_fn,
        test_runner=test_runner,
        boundary=boundary,
        task_card_cls=TaskCard,
        resolve_effective_limits=resolve_effective_limits,
        build_default_test_runner=build_default_test_runner,
        max_malformed_bounces=MAX_MALFORMED_BOUNCES,
        tool_calling_system_prompt=TOOL_CALLING_SYSTEM_PROMPT,
        default_num_ctx=MODEL_CONTEXT_TOKENS,
        default_num_predict=GENERATION_TOKEN_BUDGET,
        default_local_agent_model=default_local_agent_model,
    )


if __name__ == "__main__":
    sys.exit(main())
