"""Replay fixture corpus for scripts/antares/harness_test.py (T2e).

Raw tool-call JSON message constants for every approved HP-#/EC-# case, kept
separate from the test assertions so the fixtures can be inspected or
extended independently. Pure data -- no TerminalStateKind/Artifact types
involved, so this module needs none of the sibling-loading machinery every
other file in this directory uses.
"""

from __future__ import annotations

import json


def _msg(tool: str, **payload: object) -> str:
    body: dict[str, object] = {"tool": tool}
    if payload:
        body["payload"] = payload
    return json.dumps(body)


# HP-1: a fully valid terminal command followed by submit_vulnerable_files.
HP1_TERMINAL_COMMAND = _msg("terminal", argv=["cat", "src/main.rs"])
HP1_SUBMIT_VULNERABLE_FILES = _msg("submit_vulnerable_files", candidates=["src/main.rs"])

# HP-2: a fully valid terminal command followed by an explicit negative result.
HP2_TERMINAL_COMMAND = _msg("terminal", argv=["grep", "-n", "unsafe", "src/main.rs"])
HP2_SUBMIT_NO_VULNERABILITY_FOUND = _msg("submit_no_vulnerability_found")

# EC-1: command-budget exhaustion. The same valid terminal command replayed
# repeatedly against a session whose SessionBudget.command_budget is small
# (configured by the test, not here) so the Nth+1 call is refused pre-flight.
EC1_TERMINAL_COMMAND = _msg("terminal", argv=["cat", "src/main.rs"])

# EC-2: one distinct failure per originating layer, proving each is mapped
# through its own existing TerminalStateKind rather than a collapsed generic
# failure.
EC2_MALFORMED_JSON = "{not valid json"
EC2_UNSUPPORTED_TOOL = _msg("delete_repository")
EC2_DISALLOWED_EXECUTABLE = _msg("terminal", argv=["sleep", "5"])
EC2_DISALLOWED_OPTION = _msg("terminal", argv=["find", ".", "-perm", "777"])
EC2_SHELL_METACHARACTER = _msg("terminal", argv=["cat", "src/main.rs; rm -rf /"])
EC2_PATH_TRAVERSAL = _msg("terminal", argv=["cat", "../../etc/passwd"])

# EC-3: sandbox-escape regression fixtures -- the same four escape families as
# EC-2's policy-layer cases, replayed under their own name so a test asserting
# "fails closed identically through the composed harness" reads clearly
# against the ledger's own EC-3 wording, independent of how EC-2 is phrased.
EC3_SHELL_METACHARACTER = EC2_SHELL_METACHARACTER
EC3_DISALLOWED_EXECUTABLE = EC2_DISALLOWED_EXECUTABLE
EC3_DISALLOWED_OPTION = EC2_DISALLOWED_OPTION
EC3_PATH_TRAVERSAL = EC2_PATH_TRAVERSAL

# Supplemental coverage beyond the four named ledger cases: a second
# submission in the same session (of either shape) must be refused as a
# duplicate, proving tool_call_parser.check_duplicate_submission survives
# composition.
SUPPLEMENTAL_DUPLICATE_SUBMISSION_SECOND = HP2_SUBMIT_NO_VULNERABILITY_FOUND

# Supplemental coverage: a submit_vulnerable_files candidate itself escaping
# the snapshot is a distinct code path from EC-3's terminal-command-operand
# path traversal -- the harness's own check_path_containment call for
# submission candidates, not command_policy.validate_command's internal
# resolve_within_snapshot check.
SUPPLEMENTAL_SUBMIT_CANDIDATE_PATH_TRAVERSAL = _msg(
    "submit_vulnerable_files", candidates=["../../etc/passwd"]
)


def ec4_cat_command(relative_path: str) -> str:
    """EC-4: a policy-approved `cat <relative_path>` request. The test
    supplies a per-test fixture path (a FIFO for a hang, or a large file for
    an output-cap breach) -- neither is a static literal, so this is a
    function, not a module-level constant.
    """
    return _msg("terminal", argv=["cat", relative_path])
