#!/usr/bin/env python3
"""Session preflight for DubBridge coding agents."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import threading
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


SCRIPT_VERSION = 1
SENTINEL_RELATIVE = Path(".agent") / "session-preflight.json"

RECEIPT_SCHEMA_VERSION = 2
RECEIPTS_DIR_RELATIVE = Path(".agent") / "receipts" / "v2"
V2_VALID_PROVIDERS = frozenset({"codex", "claude"})
V2_VALID_LIFECYCLE_EVENTS = frozenset(
    {
        "startup",
        "resume",
        "clear",
        "compact",
        "fork",
        "subagent",
    }
)

SUMMARY_LINES = [
    "DubBridge agent preflight",
    "",
    "Authority:",
    "- docs/playbooks/AGENT_WORKFLOW_GUIDE.md is the highest-authority workflow source.",
    "- CLAUDE.md and AGENTS.md are summaries for topics not overridden by the workflow guide.",
    "",
    "Before implementation:",
    "- Identify and read affected files plus material governing docs.",
    "- Include docs/architecture.md, applicable ADRs, docs/plan/roadmap.md, slice plan/task ledger, BDD/product docs, and relevant policies/configs when they constrain the task.",
    "- Ensure plan/task ledger exists for staged work.",
    "- Run scripts/rri.py before presenting or delegating a task.",
    "- RRI 0-25: no full approval packet; use Gemma only for eligible simple code patches.",
    "- RRI 26+: present the task and wait for explicit approval before editing.",
    "- Mobile UI/presentation work under mobile/ must read root DESIGN.md first.",
    "",
    "Before closure:",
    "- For development tasks, evaluate Gemma Reviewer / D14 review before coverage or Done status.",
    "- Sync materially affected status docs before reporting completion.",
]


class PreflightError(Exception):
    """Raised when the session preflight has not been satisfied."""


def find_repo_root(start: Path | None = None) -> Path:
    """Return the git repository root, falling back to the script parent."""
    if start is None:
        start = Path.cwd()
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=start,
            capture_output=True,
            check=True,
            text=True,
        )
        return Path(result.stdout.strip()).resolve()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return Path(__file__).resolve().parents[1]


def sentinel_path(repo_root: Path) -> Path:
    return repo_root / SENTINEL_RELATIVE


def preflight_summary() -> str:
    return "\n".join(SUMMARY_LINES) + "\n"


def sentinel_payload(repo_root: Path) -> Dict[str, Any]:
    return {
        "version": SCRIPT_VERSION,
        "repo_root": str(repo_root.resolve()),
        "marked_at": datetime.now(timezone.utc).isoformat(),
        "requirements": [
            "read AGENT_WORKFLOW_GUIDE.md before staged work",
            "identify architecture, ADR, roadmap, plan, task, BDD/product, policy, and config docs that materially constrain the task",
            "run scripts/rri.py before implementation",
            "wait for approval when RRI is 26 or higher",
            "read DESIGN.md for mobile UI/presentation work",
            "run Gemma Reviewer or D14 before development closure when required",
        ],
    }


def mark_preflight(repo_root: Path) -> Path:
    path = sentinel_path(repo_root)
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_suffix(f"{path.suffix}.tmp")
    tmp_path.write_text(json.dumps(sentinel_payload(repo_root), indent=2) + "\n", encoding="utf-8")
    os.replace(tmp_path, path)
    return path


def load_sentinel(path: Path) -> Dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise PreflightError(
            f"Missing {path}. Run scripts/agent-preflight.py --mark before editing."
        ) from exc
    except json.JSONDecodeError as exc:
        raise PreflightError(
            f"Invalid {path}. Re-run scripts/agent-preflight.py --mark before editing."
        ) from exc
    if not isinstance(data, dict):
        raise PreflightError(
            f"Invalid {path}: expected a JSON object. Re-run scripts/agent-preflight.py --mark."
        )
    return data


def check_preflight(repo_root: Path) -> Dict[str, Any]:
    path = sentinel_path(repo_root)
    data = load_sentinel(path)
    marked_root = data.get("repo_root")
    if marked_root != str(repo_root.resolve()):
        raise PreflightError(
            f"{path} was marked for {marked_root!r}, not {str(repo_root.resolve())!r}. "
            "Run scripts/agent-preflight.py --mark in this repository."
        )
    if data.get("version") != SCRIPT_VERSION:
        raise PreflightError(
            f"{path} has version {data.get('version')!r}; expected {SCRIPT_VERSION}. "
            "Run scripts/agent-preflight.py --mark again."
        )
    return data


class ReceiptValidationError(PreflightError):
    """Raised when a v2 receipt payload or its inputs fail validation."""


def validate_provider(provider: str) -> None:
    if provider not in V2_VALID_PROVIDERS:
        raise ReceiptValidationError(
            f"Unsupported provider {provider!r}; expected one of {sorted(V2_VALID_PROVIDERS)}."
        )


def validate_lifecycle_event(hook_event_name: str) -> None:
    if hook_event_name not in V2_VALID_LIFECYCLE_EVENTS:
        raise ReceiptValidationError(
            f"Unsupported lifecycle hook_event_name {hook_event_name!r}; "
            f"expected one of {sorted(V2_VALID_LIFECYCLE_EVENTS)}."
        )


def validate_opaque_id(label: str, value: str) -> None:
    if not value:
        raise ReceiptValidationError(f"{label} must not be empty.")
    if "\x00" in value:
        raise ReceiptValidationError(f"{label} must not contain a NUL byte.")
    if "/" in value or "\\" in value:
        raise ReceiptValidationError(f"{label} must not contain a path separator.")
    if ".." in value:
        raise ReceiptValidationError(f"{label} must not contain a '..' segment.")


def compute_receipt_identity(provider: str, session_id: str, actor_id: str) -> str:
    validate_provider(provider)
    validate_opaque_id("session_id", session_id)
    validate_opaque_id("actor_id", actor_id)
    digest_input = "\x00".join([provider, session_id, actor_id]).encode("utf-8")
    return hashlib.sha256(digest_input).hexdigest()


def hash_source_file(repo_root: Path, relative_path: str) -> Dict[str, Any]:
    full_path = repo_root / relative_path
    try:
        data = full_path.read_bytes()
    except OSError as exc:
        raise ReceiptValidationError(
            f"Could not read source file {relative_path!r} under {repo_root}: {exc}"
        ) from exc
    return {
        "path": relative_path,
        "sha256": hashlib.sha256(data).hexdigest(),
        "bytes": len(data),
    }


def build_v2_receipt_payload(
    *,
    provider: str,
    session_id: str,
    actor_id: str,
    repo_root: Path,
    hook_event_name: str,
    source: str,
    transcript_path: str,
    native_instruction_mechanism: str,
    native_instruction_path: str,
    document_paths: List[str],
) -> Dict[str, Any]:
    validate_provider(provider)
    validate_lifecycle_event(hook_event_name)
    validate_opaque_id("session_id", session_id)
    validate_opaque_id("actor_id", actor_id)

    identity = compute_receipt_identity(provider, session_id, actor_id)

    native_instruction = hash_source_file(repo_root, native_instruction_path)
    native_instruction["mechanism"] = native_instruction_mechanism

    documents = [hash_source_file(repo_root, doc_path) for doc_path in document_paths]

    return {
        "schema_version": RECEIPT_SCHEMA_VERSION,
        "provider": provider,
        "session_id": session_id,
        "actor_id": actor_id,
        "receipt_identity": identity,
        "repo_root": str(repo_root.resolve()),
        "lifecycle": {
            "hook_event_name": hook_event_name,
            "source": source,
            "transcript_path": transcript_path,
        },
        "loaded_at": datetime.now(timezone.utc).isoformat(),
        "native_instruction": native_instruction,
        "documents": documents,
    }


def validate_v2_receipt_payload(payload: Dict[str, Any]) -> None:
    if not isinstance(payload, dict):
        raise ReceiptValidationError("v2 receipt payload must be a JSON object.")
    if "version" in payload and "schema_version" not in payload:
        raise ReceiptValidationError(
            "Payload looks like a legacy v1 sentinel (has 'version', missing 'schema_version')."
        )
    if payload.get("schema_version") != RECEIPT_SCHEMA_VERSION:
        raise ReceiptValidationError(
            f"Unsupported receipt schema_version {payload.get('schema_version')!r}; "
            f"expected {RECEIPT_SCHEMA_VERSION}."
        )
    validate_provider(payload.get("provider"))
    validate_opaque_id("session_id", payload.get("session_id", ""))
    validate_opaque_id("actor_id", payload.get("actor_id", ""))
    lifecycle = payload.get("lifecycle")
    if not isinstance(lifecycle, dict):
        raise ReceiptValidationError("v2 receipt payload requires a 'lifecycle' object.")
    validate_lifecycle_event(lifecycle.get("hook_event_name"))
    for required_key in ("native_instruction", "documents"):
        if required_key not in payload:
            raise ReceiptValidationError(f"v2 receipt payload missing {required_key!r}.")


def v2_receipts_dir(repo_root: Path) -> Path:
    return repo_root / RECEIPTS_DIR_RELATIVE


def v2_receipt_path(repo_root: Path, provider: str, session_id: str, actor_id: str) -> Path:
    identity = compute_receipt_identity(provider, session_id, actor_id)
    return v2_receipts_dir(repo_root) / f"{identity}.json"


def load_v2_receipt(repo_root: Path, provider: str, session_id: str, actor_id: str) -> Dict[str, Any]:
    """Load and validate a previously published v2 receipt.

    Raises ReceiptValidationError (a PreflightError subclass) for a missing,
    unreadable, malformed, or schema-invalid receipt -- callers must treat
    that as operational invalidity, never as an authorizing receipt.
    """
    path = v2_receipt_path(repo_root, provider, session_id, actor_id)
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise ReceiptValidationError(f"No published receipt at {path}: {exc}") from exc
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ReceiptValidationError(f"Receipt at {path} is not valid JSON: {exc}") from exc
    validate_v2_receipt_payload(payload)
    if payload.get("provider") != provider:
        raise ReceiptValidationError(
            f"Receipt at {path} is for provider {payload.get('provider')!r}, expected {provider!r}."
        )
    if payload.get("session_id") != session_id or payload.get("actor_id") != actor_id:
        raise ReceiptValidationError(f"Receipt at {path} does not match session/actor identity.")
    return payload


def _secure_mkdir(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    try:
        os.chmod(path, 0o700)
    except OSError:
        pass


def _invalidate_prior_receipt(target_path: Path) -> None:
    if not target_path.exists() and not target_path.is_symlink():
        return
    try:
        target_path.unlink()
    except FileNotFoundError:
        pass


def publish_v2_receipt(
    repo_root: Path,
    payload: Dict[str, Any],
) -> Path:
    """Atomically publish a v2 receipt, invalidating any prior receipt for the
    same provider/session/actor identity first. A crash or error at any point
    before the final `os.replace` leaves no authorizing receipt on disk."""
    validate_v2_receipt_payload(payload)

    target_path = v2_receipt_path(
        repo_root, payload["provider"], payload["session_id"], payload["actor_id"]
    )
    receipts_dir = target_path.parent
    _secure_mkdir(receipts_dir)

    _invalidate_prior_receipt(target_path)

    tmp_path = (
        receipts_dir
        / f".{target_path.name}.{os.getpid()}.{threading.get_ident()}.{uuid.uuid4().hex}.tmp"
    )
    try:
        fd = os.open(str(tmp_path), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(json.dumps(payload, indent=2) + "\n")
                handle.flush()
                os.fsync(handle.fileno())
        except BaseException:
            try:
                os.close(fd)
            except OSError:
                pass
            raise
        try:
            os.chmod(tmp_path, 0o600)
        except OSError:
            pass
        os.replace(tmp_path, target_path)
    except BaseException:
        try:
            tmp_path.unlink()
        except FileNotFoundError:
            pass
        raise

    return target_path


class HookPayloadError(Exception):
    """Raised when provider hook JSON cannot be parsed or is missing required fields."""


CLAUDE_NATIVE_INSTRUCTION_PATH = "CLAUDE.md"
CODEX_NATIVE_INSTRUCTION_PATH = "AGENTS.override.md"


def _read_hook_stdin(stream: Any) -> Dict[str, Any]:
    raw = stream.read()
    debug_path = os.environ.get("DUBBRIDGE_PREFLIGHT_DEBUG_STDIN")
    if debug_path:
        try:
            Path(debug_path).write_text(raw, encoding="utf-8")
        except OSError:
            pass
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise HookPayloadError(f"Hook stdin is not valid JSON: {exc}") from exc
    if not isinstance(data, dict):
        raise HookPayloadError("Hook stdin must be a JSON object.")
    return data


def adapt_claude_hook_payload(hook_input: Dict[str, Any]) -> Dict[str, Any]:
    """Translate a Claude Code hook stdin payload into v2 receipt identity fields.

    Claude `SessionStart` hook payloads carry `session_id`, `transcript_path`,
    `cwd`, `hook_event_name`, and `source`, per the Claude Code hooks
    contract. `hook_event_name` is the hook *type* and is always the literal
    string "SessionStart" -- it is never a lifecycle value. The lifecycle
    sub-event ("startup", "resume", "clear", "compact", "fork") is carried in
    the separate `source` field, confirmed against a live-captured payload:
    `{"session_id":"...","hook_event_name":"SessionStart","source":"startup"}`.
    This mirrors T4b1's `extract_hook_gate_identity` finding for `PreToolUse`
    (`hook_event_name` there is likewise the hook type, not a lifecycle
    value) -- `hook-load`'s job is different (it must validate and record a
    real lifecycle event), so the fix here is to read the correct field,
    not to skip lifecycle validation the way the gate path does.
    """
    session_id = hook_input.get("session_id")
    lifecycle_event = hook_input.get("source")
    if not isinstance(session_id, str) or not session_id:
        raise HookPayloadError("Claude hook payload missing string 'session_id'.")
    if not isinstance(lifecycle_event, str) or not lifecycle_event:
        raise HookPayloadError("Claude hook payload missing string 'source'.")
    return {
        "provider": "claude",
        "session_id": session_id,
        # The receipt schema's `hook_event_name` field holds the lifecycle
        # value, not the hook type -- map the payload's `source` into it.
        "hook_event_name": lifecycle_event,
        "actor_id": "claude-code",
        "source": "hook",
        "transcript_path": hook_input.get("transcript_path", ""),
        "native_instruction_mechanism": "@import",
        "native_instruction_path": CLAUDE_NATIVE_INSTRUCTION_PATH,
    }


def adapt_codex_hook_payload(hook_input: Dict[str, Any]) -> Dict[str, Any]:
    """Translate a Codex hook stdin payload into v2 receipt identity fields.

    Codex hook payloads carry the same field *names* Claude uses, and the same
    semantics: `hook_event_name` is the hook *type* (always the literal
    "SessionStart" for this hook) and the lifecycle sub-event ("startup",
    "resume", "clear", "compact", "fork") is carried in the separate `source`
    field. Confirmed against a live-captured payload from codex-cli
    0.146.0-alpha.3.1:
    `{"session_id":"019fa419-...","transcript_path":"...","cwd":"...",
      "hook_event_name":"SessionStart","model":"gpt-5.6-sol",
      "permission_mode":"bypassPermissions","source":"startup"}`.

    T4b2 read a serde struct dump from the binary and correctly concluded that
    the field is named `hook_event_name` (not `event`, as T4a3 assumed), but a
    struct dump shows field *names*, not *values* -- it inferred that this
    field also holds the lifecycle value, which it does not. That half-correct
    inference is why every genuine Codex session failed `hook-load` with
    "Unsupported lifecycle hook_event_name 'SessionStart'" while emitting only
    an undiagnosable `hook: SessionStart Failed` (Codex persists no hook events
    in its rollout log).

    `source` is required rather than falling back to `hook_event_name`: this is
    a fail-closed governance path, and silently accepting the wrong field is
    precisely the failure mode being fixed. Codex has no per-actor identity
    concept, so the actor is pinned to "codex-cli".
    """
    session_id = hook_input.get("session_id")
    lifecycle_event = hook_input.get("source")
    if not isinstance(session_id, str) or not session_id:
        raise HookPayloadError("Codex hook payload missing string 'session_id'.")
    if not isinstance(lifecycle_event, str) or not lifecycle_event:
        raise HookPayloadError("Codex hook payload missing string 'source'.")
    return {
        "provider": "codex",
        "session_id": session_id,
        "actor_id": "codex-cli",
        # The receipt schema's `hook_event_name` field holds the lifecycle
        # value, not the hook type -- map the payload's `source` into it.
        "hook_event_name": lifecycle_event,
        "source": "hook",
        "transcript_path": hook_input.get("transcript_path", ""),
        "native_instruction_mechanism": "generated-bundle",
        "native_instruction_path": CODEX_NATIVE_INSTRUCTION_PATH,
    }


HOOK_ADAPTERS = {
    "claude": adapt_claude_hook_payload,
    "codex": adapt_codex_hook_payload,
}


def adapt_hook_payload(provider: str, hook_input: Dict[str, Any]) -> Dict[str, Any]:
    try:
        adapter = HOOK_ADAPTERS[provider]
    except KeyError:
        raise HookPayloadError(
            f"Unsupported hook provider {provider!r}; expected one of {sorted(HOOK_ADAPTERS)}."
        ) from None
    return adapter(hook_input)


HOOK_ACTOR_IDS = {
    "claude": "claude-code",
    "codex": "codex-cli",
}


def extract_hook_gate_identity(provider: str, hook_input: Dict[str, Any]) -> Dict[str, Any]:
    """Extract only the identity fields a gate check needs from hook stdin.

    Gate hooks (e.g. Claude's PreToolUse) fire for arbitrary tool events whose
    `hook_event_name`/`event` value is the hook type itself (e.g. "PreToolUse"),
    not a session lifecycle value from V2_VALID_LIFECYCLE_EVENTS. A gate check
    only needs to know which already-published receipt to look up -- it must
    not run that value through lifecycle validation the way hook-load does.
    """
    try:
        actor_id = HOOK_ACTOR_IDS[provider]
    except KeyError:
        raise HookPayloadError(
            f"Unsupported hook provider {provider!r}; expected one of {sorted(HOOK_ACTOR_IDS)}."
        ) from None
    session_id = hook_input.get("session_id")
    if not isinstance(session_id, str) or not session_id:
        raise HookPayloadError(f"{provider.capitalize()} hook payload missing string 'session_id'.")
    return {"provider": provider, "session_id": session_id, "actor_id": actor_id}


def claude_gate_response(*, allow: bool, reason: str) -> Dict[str, Any]:
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow" if allow else "deny",
            "permissionDecisionReason": reason,
        }
    }


def codex_gate_response(*, allow: bool, reason: str) -> Dict[str, Any]:
    """Build the Codex PreToolUse response shape.

    Confirmed against the installed Codex CLI binary (codex-cli
    0.146.0-alpha.3.1): the serde struct dump groups `hookEventName`,
    `permissionDecision`, `permissionDecisionReason`, `additionalContext`
    together as one `PreToolUsePermissionDecisionWire`-shaped struct --
    field-for-field the same shape Claude uses (`claude_gate_response`
    above), not a distinct `{"decision", "reason"}` contract.
    """
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow" if allow else "deny",
            "permissionDecisionReason": reason,
        }
    }


GATE_RESPONSE_BUILDERS = {
    "claude": claude_gate_response,
    "codex": codex_gate_response,
}


V2_COMMANDS = ("load", "check", "hook-load", "hook-gate")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Print and validate the DubBridge agent-session workflow preflight."
    )
    parser.add_argument(
        "command",
        nargs="?",
        choices=V2_COMMANDS,
        default=None,
        help=(
            "v2 receipt command: 'load' builds+publishes a receipt from explicit "
            "identity flags; 'check' validates an already-published receipt; "
            "'hook-load' and 'hook-gate' read a provider hook JSON payload on stdin."
        ),
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=None,
        help="Repository root override, mainly for tests and hook wrappers.",
    )
    parser.add_argument(
        "--print-summary",
        action="store_true",
        help="Print the compact workflow startup summary.",
    )
    parser.add_argument(
        "--mark",
        action="store_true",
        help="Write the legacy v1 session preflight sentinel (diagnostics only; "
        "cannot authorize any v2 gate).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Fail unless the legacy v1 session preflight sentinel is present and valid.",
    )
    parser.add_argument("--provider", choices=sorted(V2_VALID_PROVIDERS), default=None)
    parser.add_argument("--session-id", default=None)
    parser.add_argument("--actor-id", default=None)
    parser.add_argument("--hook-event-name", default=None)
    parser.add_argument("--source", default="cli")
    parser.add_argument("--transcript-path", default="")
    parser.add_argument("--native-instruction-mechanism", default=None)
    parser.add_argument("--native-instruction-path", default=None)
    parser.add_argument("--document", action="append", default=[], dest="documents")
    return parser


def resolve_repo_root(raw: Path | None) -> Path:
    if raw is not None:
        return raw.resolve()
    return find_repo_root()


def _run_load_command(args: argparse.Namespace, repo_root: Path) -> int:
    missing = [
        name
        for name, value in (
            ("--provider", args.provider),
            ("--session-id", args.session_id),
            ("--actor-id", args.actor_id),
            ("--hook-event-name", args.hook_event_name),
            ("--native-instruction-mechanism", args.native_instruction_mechanism),
            ("--native-instruction-path", args.native_instruction_path),
        )
        if value is None
    ]
    if missing:
        print(f"agent preflight malformed input: missing {', '.join(missing)}", file=sys.stderr)
        return 2
    try:
        payload = build_v2_receipt_payload(
            provider=args.provider,
            session_id=args.session_id,
            actor_id=args.actor_id,
            repo_root=repo_root,
            hook_event_name=args.hook_event_name,
            source=args.source,
            transcript_path=args.transcript_path,
            native_instruction_mechanism=args.native_instruction_mechanism,
            native_instruction_path=args.native_instruction_path,
            document_paths=list(args.documents),
        )
        path = publish_v2_receipt(repo_root, payload)
    except PreflightError as exc:
        print(f"agent preflight failed: {exc}", file=sys.stderr)
        return 1
    sys.stdout.write(preflight_summary())
    print(f"agent preflight receipt published: {path}")
    return 0


def _run_check_command(args: argparse.Namespace, repo_root: Path) -> int:
    if args.provider is None or args.session_id is None or args.actor_id is None:
        print(
            "agent preflight malformed input: --provider, --session-id, and --actor-id are required",
            file=sys.stderr,
        )
        return 2
    try:
        payload = load_v2_receipt(repo_root, args.provider, args.session_id, args.actor_id)
    except PreflightError as exc:
        print(f"agent preflight failed: {exc}", file=sys.stderr)
        return 1
    print(f"agent preflight receipt ok: {payload.get('loaded_at', 'unknown time')}")
    return 0


def _run_hook_load_command(args: argparse.Namespace, repo_root: Path) -> int:
    if not args.provider:
        print("agent preflight malformed input: --provider is required", file=sys.stderr)
        return 2
    try:
        hook_input = _read_hook_stdin(sys.stdin)
        identity_fields = adapt_hook_payload(args.provider, hook_input)
    except HookPayloadError as exc:
        print(f"agent preflight malformed hook input: {exc}", file=sys.stderr)
        return 2
    try:
        payload = build_v2_receipt_payload(
            repo_root=repo_root,
            native_instruction_mechanism=identity_fields.pop("native_instruction_mechanism"),
            native_instruction_path=identity_fields.pop("native_instruction_path"),
            document_paths=list(args.documents),
            **identity_fields,
        )
        publish_v2_receipt(repo_root, payload)
    except PreflightError as exc:
        print(f"agent preflight failed: {exc}", file=sys.stderr)
        return 1
    sys.stdout.write(preflight_summary())
    return 0


def _run_hook_gate_command(args: argparse.Namespace, repo_root: Path) -> int:
    if not args.provider:
        print("agent preflight malformed input: --provider is required", file=sys.stderr)
        return 2
    try:
        hook_input = _read_hook_stdin(sys.stdin)
        identity_fields = extract_hook_gate_identity(args.provider, hook_input)
    except HookPayloadError as exc:
        print(f"agent preflight malformed hook input: {exc}", file=sys.stderr)
        return 2

    response_builder = GATE_RESPONSE_BUILDERS[args.provider]
    try:
        load_v2_receipt(
            repo_root,
            identity_fields["provider"],
            identity_fields["session_id"],
            identity_fields["actor_id"],
        )
    except PreflightError as exc:
        print(json.dumps(response_builder(allow=False, reason=str(exc))))
        return 1

    print(json.dumps(response_builder(allow=True, reason="agent preflight receipt valid")))
    return 0


V2_COMMAND_HANDLERS = {
    "load": _run_load_command,
    "check": _run_check_command,
    "hook-load": _run_hook_load_command,
    "hook-gate": _run_hook_gate_command,
}


def main(argv: List[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    repo_root = resolve_repo_root(args.repo_root)

    if args.command is not None:
        return V2_COMMAND_HANDLERS[args.command](args, repo_root)

    if not (args.print_summary or args.mark or args.check):
        args.print_summary = True

    if args.print_summary:
        sys.stdout.write(preflight_summary())

    if args.mark:
        path = mark_preflight(repo_root)
        print(f"agent preflight marked: {path}")

    if args.check:
        try:
            data = check_preflight(repo_root)
        except PreflightError as exc:
            print(f"agent preflight failed: {exc}", file=sys.stderr)
            return 1
        print(f"agent preflight ok: {data.get('marked_at', 'unknown time')}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
