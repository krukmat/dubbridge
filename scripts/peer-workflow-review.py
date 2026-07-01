#!/usr/bin/env python3
"""Band-routed, two-phase peer workflow reviewer for DubBridge.

Implements the review contract defined in docs/plan/portable-peer-review-gate.md:

  Phase 1 (--phase task):  task-analysis review before presentation/delegation.
  Phase 2 (--phase code):  code-solution review after implementation, before closure.

Reviewer is resolved from the task's RRI band:
  RRI 0-40  (Low + Moderate)  -> Gemma (local Ollama)
  RRI 41+   (Med-high+)       -> cross-vendor peer, with D14 fallback

Cross-vendor resolution (RRI 41+ only):
  claude-code | claude  -> codex
  codex                 -> claude
  local-provider        -> claude
  remote-provider       -> claude
  unknown               -> claude

Exit codes:
  0   PASS
  1   FINDINGS (non-blocking) or BLOCKED (only when D14 also unavailable)
  2   peer invocation error / unable to review (blocked artifact written)
"""

import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gemma_local

BAND_BOUNDARY = 41  # RRI >= this -> cross-vendor peer

CALLER_TO_PEER = {
    "claude-code": "codex",
    "claude": "codex",
    "codex": "claude",
    "local-provider": "claude",
    "remote-provider": "claude",
    "unknown": "claude",
}

AGENT_DIR = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    ".agent",
)


# ---------------------------------------------------------------------------
# Band / reviewer resolution
# ---------------------------------------------------------------------------

def resolve_band(rri: int) -> str:
    if rri >= 56:
        return "Complex"
    if rri >= 41:
        return "Med-high"
    if rri >= 26:
        return "Moderate"
    return "Low"


def needs_cross_vendor(rri: int) -> bool:
    return rri >= BAND_BOUNDARY


def resolve_peer(caller: str) -> str:
    return CALLER_TO_PEER.get(caller.lower(), "claude")


def peer_cli_available(peer: str) -> bool:
    return shutil.which(peer) is not None


# ---------------------------------------------------------------------------
# Gemma path (RRI 0-40)
# ---------------------------------------------------------------------------

def _gemma_system_prompt(phase: str) -> str:
    if phase == "task":
        return (
            "You are a read-only task-analysis reviewer for DubBridge. "
            "Review the supplied task card for readiness: completeness of acceptance "
            "criteria, clarity of scope, missing happy paths or edge cases, and "
            "consistency with the governing policy references.\n"
            "Return ONLY tagged text in this exact shape:\n"
            "STATUS: PASS\n"
            "SUMMARY: short review summary\n"
            "=== FINDING START ===\n"
            "PATH: docs/tasks/example.md\n"
            "LINE: 1\n"
            "SEVERITY: blocking|major|minor|nit\n"
            "DETAIL: concrete gap or risk\n"
            "SUGGESTION: concise fix direction\n"
            "=== FINDING END ===\n"
            "Rules: STATUS ∈ {PASS, FINDINGS, BLOCKED}. Use PASS with no finding "
            "blocks when no issues are found. BLOCKED only when the packet is not "
            "reviewable. No markdown fences, no JSON, no extra text."
        )
    return (
        "You are a read-only code-solution reviewer for DubBridge. "
        "Review the supplied diff and acceptance criteria for correctness, "
        "fail-closed behavior, missing tests, and side effects.\n"
        "Return ONLY tagged text in this exact shape:\n"
        "STATUS: PASS\n"
        "SUMMARY: short review summary\n"
        "=== FINDING START ===\n"
        "PATH: repo/relative/path.ext\n"
        "LINE: 123\n"
        "SEVERITY: blocking|major|minor|nit\n"
        "DETAIL: concrete bug or risk\n"
        "SUGGESTION: concise fix direction\n"
        "=== FINDING END ===\n"
        "Rules: STATUS ∈ {PASS, FINDINGS, BLOCKED}. No markdown fences, no JSON, "
        "no diff, no patch, no extra text."
    )


def run_gemma_review(packet: str, phase: str, args) -> dict:
    """Invoke local Gemma and return a result dict."""
    model = gemma_local.resolve_model_with_fallback(
        args.host,
        args.model,
        args.idle_timeout,
        gemma_local.default_fallback_model_for(
            "DUBBRIDGE_REVIEW_MODEL",
            "DUBBRIDGE_LOW_RRI_MODEL",
        ),
    )
    payload = gemma_local.build_chat_payload(
        model=model,
        system_prompt=_gemma_system_prompt(phase),
        packet=packet,
        num_ctx=args.num_ctx,
        num_predict=args.num_predict,
        temperature=args.temperature,
        think=args.think,
    )
    stream_result = gemma_local.stream_chat(
        gemma_local.endpoint(args.host, "/api/chat"),
        payload,
        idle_timeout=args.idle_timeout,
        max_wall=args.max_wall,
        progress_label=f"peer-review/{phase}",
    )
    content = gemma_local.stream_result_content(stream_result)
    # Minimal parse: extract STATUS line; treat the rest as summary.
    verdict = "pass"
    summary = ""
    findings = []
    for line in content.splitlines():
        if line.startswith("STATUS: "):
            raw = line[len("STATUS: "):].strip().lower()
            if raw in ("pass", "findings", "blocked"):
                verdict = raw
        elif line.startswith("SUMMARY: "):
            summary = line[len("SUMMARY: "):].strip()
    return {
        "reviewer": "gemma",
        "phase": phase,
        "verdict": verdict,
        "summary": summary,
        "findings": findings,
        "model": model,
    }


# ---------------------------------------------------------------------------
# Cross-vendor peer path (RRI 41+)
# ---------------------------------------------------------------------------

def _build_peer_packet(phase: str, content: str, task_id: str) -> str:
    header = (
        f"# DubBridge peer workflow review\n"
        f"phase: {phase}\n"
        f"task_id: {task_id or 'n/a'}\n\n"
    )
    if phase == "task":
        return (
            header
            + "Review the following task card for readiness. Check: completeness of "
            "acceptance criteria, scope clarity, missing happy/edge cases, consistency "
            "with policy references. Reply with: VERDICT: PASS or VERDICT: BLOCKED, "
            "then a SUMMARY line, then any FINDING lines (PATH / LINE / SEVERITY / "
            "DETAIL / SUGGESTION). No code, no patches.\n\n"
            + content
        )
    return (
        header
        + "Review the following diff and acceptance criteria. Check: correctness, "
        "fail-closed behaviour, missing tests, side effects. Reply with: "
        "VERDICT: PASS or VERDICT: BLOCKED, then a SUMMARY line, then any FINDING "
        "lines. No patches.\n\n"
        + content
    )


def _parse_peer_response(output: str, peer: str, phase: str) -> dict:
    verdict = "pass"
    summary = ""
    findings = []
    for line in output.splitlines():
        ls = line.strip()
        if ls.startswith("VERDICT:"):
            raw = ls[len("VERDICT:"):].strip().lower()
            if raw in ("pass", "blocked", "findings"):
                verdict = raw
        elif ls.startswith("SUMMARY:"):
            summary = ls[len("SUMMARY:"):].strip()
        elif ls.startswith("FINDING:"):
            findings.append(ls[len("FINDING:"):].strip())
    return {
        "reviewer": peer,
        "phase": phase,
        "verdict": verdict,
        "summary": summary,
        "findings": findings,
    }


def invoke_peer_cli(peer: str, packet: str) -> tuple[bool, str]:
    """Invoke the peer CLI and return (success, output)."""
    try:
        result = subprocess.run(
            [peer, "review", "--stdin"],
            input=packet,
            capture_output=True,
            text=True,
            timeout=120,
        )
        if result.returncode == 0:
            return True, result.stdout
        return False, result.stderr or result.stdout
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError) as exc:
        return False, str(exc)


def run_d14_fallback(packet: str, phase: str, peer: str) -> dict:
    """D14: spawn a context-isolated subagent via adjudicator-packet.py logic."""
    import adjudicator_packet  # lazy: scripts/ is on sys.path at runtime  # noqa: PLC0415
    # Build a minimal isolation packet from the review content.
    ap = adjudicator_packet.build_adjudicator_packet(
        diff=packet if phase == "code" else "",
        criteria=packet if phase == "task" else "",
        reconciled_findings=[],
    )
    # D14 adjudication is spawned externally by the orchestrating agent.
    # This script signals that D14 is needed by returning reviewer="d14" and a
    # structured packet so the caller can relay it.
    return {
        "reviewer": "d14",
        "phase": phase,
        "verdict": "d14_required",
        "summary": f"Peer CLI '{peer}' unavailable; D14 adjudication required.",
        "findings": [],
        "d14_packet": ap,
    }


def run_cross_vendor_review(packet: str, phase: str, peer: str) -> dict:
    """Try peer CLI; fall back to D14 signal if unavailable."""
    if not peer_cli_available(peer):
        return run_d14_fallback(packet, phase, peer)

    ok, output = invoke_peer_cli(peer, packet)
    if not ok:
        print(f"[peer-review] peer '{peer}' invocation failed: {output}", file=sys.stderr)
        return run_d14_fallback(packet, phase, peer)

    return _parse_peer_response(output, peer, phase)


# ---------------------------------------------------------------------------
# Artifact writing
# ---------------------------------------------------------------------------

def default_artifact_path(task_id: str, phase: str) -> str:
    os.makedirs(AGENT_DIR, exist_ok=True)
    slug = task_id.lower().replace(" ", "-") if task_id else "unknown"
    return os.path.join(AGENT_DIR, f"peer-{phase}-review-{slug}.json")


def write_artifact(result: dict, path: str) -> None:
    result = {
        **result,
        "ts": datetime.datetime.utcnow().isoformat() + "Z",
    }
    gemma_local.write_result(result, path)
    print(f"[peer-review] artifact written to {path}", file=sys.stderr)


def write_blocked_artifact(reason: str, phase: str, path: str, peer: str) -> None:
    write_artifact(
        {
            "reviewer": peer,
            "phase": phase,
            "verdict": "blocked",
            "summary": reason,
            "findings": [],
            "blocked": True,
        },
        path,
    )


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def parse_args():
    parser = argparse.ArgumentParser(
        description="Band-routed, two-phase peer workflow reviewer for DubBridge.",
    )
    parser.add_argument(
        "--phase",
        choices=["task", "code"],
        required=True,
        help="Review phase: 'task' (analysis before presentation) or 'code' (solution after implementation).",
    )
    parser.add_argument(
        "--rri",
        type=int,
        required=True,
        help="Task RRI score. Determines reviewer band (0-40 -> Gemma, 41+ -> cross-vendor peer).",
    )
    parser.add_argument(
        "--caller",
        default="unknown",
        help=(
            "Caller identity for cross-vendor resolution. "
            "One of: claude-code, claude, codex, local-provider, remote-provider, unknown."
        ),
    )
    parser.add_argument(
        "--content",
        default=None,
        metavar="FILE",
        help="Path to the content to review (task card or diff). Reads stdin when omitted or '-'.",
    )
    parser.add_argument(
        "--artifact",
        default=None,
        metavar="FILE",
        help="Path to write the review artifact JSON. Defaults to .agent/peer-<phase>-review-<task-id>.json.",
    )
    parser.add_argument(
        "--task-id",
        default=None,
        dest="task_id",
        metavar="ID",
        help="Task ID recorded in the artifact and used for the default artifact path.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the resolved reviewer and content length without invoking any model.",
    )
    # Gemma transport options (forwarded to gemma_local when in Gemma band).
    parser.add_argument(
        "--host",
        default=os.environ.get("OLLAMA_HOST", gemma_local.DEFAULT_HOST),
    )
    parser.add_argument(
        "--model",
        default=os.environ.get(
            "DUBBRIDGE_REVIEW_MODEL",
            os.environ.get("DUBBRIDGE_LOW_RRI_MODEL", gemma_local.DEFAULT_MODEL),
        ),
    )
    parser.add_argument(
        "--idle-timeout",
        type=int,
        dest="idle_timeout",
        default=int(os.environ.get("DUBBRIDGE_REVIEW_IDLE_TIMEOUT_SECONDS", str(gemma_local.DEFAULT_IDLE_TIMEOUT_SECONDS))),
    )
    parser.add_argument(
        "--max-wall",
        type=int,
        dest="max_wall",
        default=int(os.environ.get("DUBBRIDGE_REVIEW_MAX_WALL_SECONDS", str(gemma_local.DEFAULT_MAX_WALL_SECONDS))),
    )
    parser.add_argument(
        "--num-ctx",
        type=int,
        dest="num_ctx",
        default=int(os.environ.get("DUBBRIDGE_REVIEW_NUM_CTX", str(gemma_local.DEFAULT_NUM_CTX))),
    )
    parser.add_argument(
        "--num-predict",
        type=int,
        dest="num_predict",
        default=int(os.environ.get("DUBBRIDGE_REVIEW_NUM_PREDICT", str(gemma_local.DEFAULT_NUM_PREDICT))),
    )
    parser.add_argument(
        "--temperature",
        type=float,
        default=float(os.environ.get("DUBBRIDGE_REVIEW_TEMPERATURE", str(gemma_local.DEFAULT_TEMPERATURE))),
    )
    parser.add_argument(
        "--think",
        action="store_true",
        default=gemma_local.bool_from_env("DUBBRIDGE_REVIEW_THINK", gemma_local.DEFAULT_THINK),
    )
    parser.add_argument("--no-think", action="store_false", dest="think")
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    content = gemma_local.read_packet(args.content).strip()
    if not content:
        print("[peer-review] error: content is empty", file=sys.stderr)
        return 2

    band = resolve_band(args.rri)
    cross_vendor = needs_cross_vendor(args.rri)
    peer = resolve_peer(args.caller) if cross_vendor else "gemma"
    artifact = args.artifact or default_artifact_path(args.task_id, args.phase)

    print(
        f"[peer-review] rri={args.rri} band={band} phase={args.phase} "
        f"reviewer={peer} caller={args.caller}",
        file=sys.stderr,
    )

    if args.dry_run:
        print(json.dumps({
            "rri": args.rri,
            "band": band,
            "phase": args.phase,
            "reviewer": peer,
            "caller": args.caller,
            "content_len": len(content),
            "artifact": artifact,
        }, indent=2))
        return 0

    if cross_vendor:
        packet = _build_peer_packet(args.phase, content, args.task_id)
        result = run_cross_vendor_review(packet, args.phase, peer)

        if result["verdict"] == "d14_required":
            # Signal D14 is needed; artifact carries the isolation packet.
            write_artifact(result, artifact)
            print(
                f"[peer-review] D14 required — peer '{peer}' unavailable. "
                f"Spawn D14 adjudicator with artifact: {artifact}",
                file=sys.stderr,
            )
            return 1

        write_artifact(result, artifact)
        verdict = result["verdict"]
        print(f"[peer-review] verdict={verdict.upper()} artifact={artifact}", file=sys.stderr)
        return 0 if verdict == "pass" else 1

    # Gemma band (RRI 0-40).
    try:
        result = run_gemma_review(content, args.phase, args)
    except (gemma_local.GemmaIdleTimeout, gemma_local.GemmaWallTimeout, RuntimeError) as exc:
        print(f"[peer-review] Gemma unavailable: {exc}", file=sys.stderr)
        write_blocked_artifact(str(exc), args.phase, artifact, "gemma")
        return 2

    write_artifact(result, artifact)
    verdict = result["verdict"]
    print(f"[peer-review] verdict={verdict.upper()} artifact={artifact}", file=sys.stderr)
    return 0 if verdict == "pass" else 1


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (gemma_local.GemmaIdleTimeout, gemma_local.GemmaWallTimeout) as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(exc.exit_code)
    except (RuntimeError, OSError) as exc:
        print(f"[peer-review] fatal: {exc}", file=sys.stderr)
        raise SystemExit(2)
