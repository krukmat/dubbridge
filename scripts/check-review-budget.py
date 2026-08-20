#!/usr/bin/env python3
"""Pre-delegation reviewability budget gate for non-Gemma agents.

Local Gemma reviewer (`gemma-code-review.py`) and developer
(`delegate-low-rri.py`) roles evaluate a change inside a fixed context window
(`DEFAULT_NUM_CTX`) while reserving generation headroom (`DEFAULT_NUM_PREDICT`).
A change whose diff is larger than that effective window either overflows the
context silently or truncates Gemma's response (`done_reason == "length"`).

This gate is the *proactive* counterpart to those fail-closed paths: it runs
before delegation and fails closed when the added/changed lines of the diff
exceed a budget derived from the context window, so the delivering (non-Gemma)
agent is forced to split the change or escalate to a non-Gemma reviewer (D14).

The budget is derived from the same env vars Gemma reads, not hardcoded, so it
tracks the context window instead of drifting from it. Only code paths that
Gemma actually receives are counted; docs/config/markdown are excluded, mirroring
the `qa-gemma-review` packet filter and `check-maintainability.classify_path`.
"""

from __future__ import annotations

import argparse
import importlib.util
import os
import re
import sys
from pathlib import Path


# Reuse the diff parser and code-path classifier already proven by the
# maintainability gate instead of re-deriving them here. Both modules use
# hyphenated/underscored filenames, so load them by path.
def _load(module_name: str, path: Path):
    spec = importlib.util.spec_from_file_location(module_name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


_maint = _load("check_maintainability", Path(__file__).with_name("check-maintainability.py"))
_gemma = _load("gemma_local", Path(__file__).with_name("gemma_local.py"))
_prompt_builder = _load(
    "prompt_builder", Path(__file__).parent / "local-agent" / "prompt_builder.py"
)


# Marker the delivering agent records (commit body or task entry) to take the
# documented escape when a change is legitimately irreducible. The reason is
# captured for the audit log; an empty reason does not satisfy the escape.
D14_OVERRIDE_RE = re.compile(r"D14-OVERRIDE:\s*(?P<reason>\S.*)$", re.MULTILINE)

# Token budget consumed by the fixed parts of a delegation packet (system prompt
# + contract + acceptance criteria) before any diff is added. Conservative
# overhead estimate so the derived line budget errs toward keeping changes
# reviewable rather than toward the raw ceiling.
PACKET_OVERHEAD_TOKENS = 1300

# A diff line averages well under the raw `num_ctx / num_predict` token cost; we
# use the same ~4-chars-per-token basis as gemma_local.estimate_text_tokens and
# assume an ~80-char average reviewable line ≈ 20 tokens/line.
TOKENS_PER_DIFF_LINE = 20

# Floor so a misconfigured tiny context window can never produce an absurd
# 1- or 2-line budget that blocks every change.
MIN_DERIVED_BUDGET = 120


def packet_overhead_tokens() -> int:
    """Fixed token cost of a delegation packet before the diff is added.

    `DUBBRIDGE_REVIEW_PACKET_OVERHEAD_TOKENS` tunes this without code changes
    when the prompt/contract template drifts from the default estimate.
    """
    return _env_int("DUBBRIDGE_REVIEW_PACKET_OVERHEAD_TOKENS", PACKET_OVERHEAD_TOKENS)


# The exact output-format contract gemma-code-review.py's build_review_payload()
# assembles for the "gemma_reviewer" role, kept verbatim so the measurement below
# reflects the real packet Gemma receives rather than a stand-in string. This is
# a cross-check fixture, not a second source of truth: gemma-code-review.py owns
# the live text; drift between the two only weakens the fidelity of this check,
# it does not change what Gemma actually receives.
_GEMMA_REVIEWER_OUTPUT_FORMAT_TEXT = (
    "You are Gemma Reviewer for DubBridge. You are read-only.\n"
    "Review the supplied packet and diff. Do not approve, modify files, "
    "emit patches, emit unified diffs, or output file bodies.\n"
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
    "Rules: use exactly one STATUS value: PASS, FINDINGS, or BLOCKED. "
    "Use PASS with no finding blocks when no issues are found. Use FINDINGS "
    "with one or more finding blocks. Use BLOCKED only when the packet is not "
    "reviewable. No markdown fences, no JSON, no diff, no patch, no extra text."
)

# Relative divergence between the fixed PACKET_OVERHEAD_TOKENS estimate and the
# builder's actual measured prompt size that is worth surfacing. Below this, the
# fixed constant is still a reasonable stand-in; the budget derivation does not
# need to react to normal clause-text edits.
OVERHEAD_DIVERGENCE_THRESHOLD = 0.20


def measured_packet_overhead_tokens() -> int:
    """Return the actual measured token size of the gemma_reviewer system prompt.

    Uses prompt_builder.build_system_prompt() (LRPC-2) against the same
    output-format text gemma-code-review.py sends, at generous budget bounds
    so this measurement never itself raises PromptBudgetExceeded -- it is a
    cross-check reading, not a live-invocation budget enforcement.
    """
    prompt = _prompt_builder.build_system_prompt(
        role="gemma_reviewer",
        num_ctx=1 << 20,
        num_predict=0,
        output_format_text=_GEMMA_REVIEWER_OUTPUT_FORMAT_TEXT,
    )
    return _gemma.estimate_text_tokens(prompt)


def check_overhead_divergence() -> str | None:
    """Return a warning string if PACKET_OVERHEAD_TOKENS has drifted from the
    prompt builder's actual measured prompt size by more than the threshold,
    or None if the two are still in reasonable agreement.

    This is the LRPC-7 cross-check: PACKET_OVERHEAD_TOKENS estimates the whole
    delegation packet (system prompt + contract + acceptance criteria) that
    Gemma receives, while prompt_builder.build_system_prompt() now measures
    the actual assembled system prompt for that same role. The two are
    adjacent, not identical, quantities (LRPC-7's own scope note), so this
    flags divergence for a human to evaluate rather than silently folding one
    into the other.
    """
    fixed = packet_overhead_tokens()
    try:
        measured = measured_packet_overhead_tokens()
    except (_prompt_builder.UnknownRoleError, _prompt_builder.PromptBudgetExceeded) as exc:
        return f"packet-overhead cross-check unavailable: {exc}"

    if fixed == 0:
        return None
    divergence = abs(measured - fixed) / fixed
    if divergence <= OVERHEAD_DIVERGENCE_THRESHOLD:
        return None
    return (
        f"packet-overhead drift: PACKET_OVERHEAD_TOKENS={fixed} vs. "
        f"prompt_builder-measured gemma_reviewer prompt={measured} tokens "
        f"({divergence:.0%} divergence, threshold {OVERHEAD_DIVERGENCE_THRESHOLD:.0%}). "
        "Consider updating PACKET_OVERHEAD_TOKENS or "
        "DUBBRIDGE_REVIEW_PACKET_OVERHEAD_TOKENS."
    )


def _env_int(name: str, default: int) -> int:
    raw = os.environ.get(name)
    if raw is None or not raw.strip():
        return default
    try:
        return int(raw.strip())
    except ValueError:
        return default


def derive_budget() -> int:
    """Return the max reviewable diff lines, derived from Gemma's context window.

    budget = (num_ctx - num_predict - packet_overhead) / tokens_per_line

    `DUBBRIDGE_REVIEW_MAX_DIFF_LINES` overrides the derived value outright when
    an operator needs an explicit ceiling.
    """
    explicit = os.environ.get("DUBBRIDGE_REVIEW_MAX_DIFF_LINES")
    if explicit is not None and explicit.strip():
        try:
            return max(1, int(explicit.strip()))
        except ValueError:
            pass

    num_ctx = _env_int(
        "DUBBRIDGE_REVIEW_NUM_CTX",
        _env_int("DUBBRIDGE_LOW_RRI_NUM_CTX", _gemma.DEFAULT_NUM_CTX),
    )
    num_predict = _env_int(
        "DUBBRIDGE_REVIEW_NUM_PREDICT",
        _env_int("DUBBRIDGE_LOW_RRI_NUM_PREDICT", _gemma.DEFAULT_NUM_PREDICT),
    )
    usable_tokens = num_ctx - num_predict - packet_overhead_tokens()
    derived = usable_tokens // TOKENS_PER_DIFF_LINE
    return max(MIN_DERIVED_BUDGET, derived)


def count_reviewable_lines(base: str | None, files: list[str]) -> int:
    """Count added/changed code lines Gemma would receive for this change."""
    return len(_maint.added_lines_for(base, files))


def find_override(text: str) -> str | None:
    """Return the documented D14 override reason from `text`, or None."""
    match = D14_OVERRIDE_RE.search(text or "")
    if match is None:
        return None
    reason = match.group("reason").strip()
    return reason or None


def _override_text(explicit_message: str | None) -> str:
    """Assemble the text searched for a D14 override marker.

    Sources, in order: an explicit `--override-message`, then the tip commit
    message (so the marker can live in the commit body that ships the change).
    """
    parts: list[str] = []
    if explicit_message:
        parts.append(explicit_message)
    if os.environ.get("DUBBRIDGE_REVIEW_OVERRIDE"):
        parts.append(os.environ["DUBBRIDGE_REVIEW_OVERRIDE"])
    try:
        parts.append(_maint.run_git(["log", "-1", "--pretty=%B"]))
    except RuntimeError:
        pass
    return "\n".join(parts)


def render_report(count: int, budget: int, override_reason: str | None) -> tuple[str, int]:
    """Return (report, exit_code) for the given counts and override state."""
    if count <= budget:
        return (f"Reviewability budget gate passed ({count}/{budget} reviewable diff lines).", 0)
    if override_reason is not None:
        return (
            "Reviewability budget exceeded but D14 override is documented "
            f"({count}/{budget} lines).\n"
            f"- override reason: {override_reason}\n"
            "This change must be reviewed by a non-Gemma (D14) reviewer; "
            "record disposition_divergence in the audit log.",
            0,
        )
    return (
        "Reviewability budget gate failed: "
        f"{count} added/changed code lines exceed the budget of {budget}.\n"
        "Gemma cannot evaluate a change this large in-context with full fidelity.\n"
        "Split the change into smaller delegation units, or — if it is genuinely "
        "irreducible — escalate to a non-Gemma (D14) reviewer and record a\n"
        "  D14-OVERRIDE: <reason>\n"
        "line in the commit body or task entry.",
        1,
    )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="git revision to diff against")
    parser.add_argument("--files", nargs="*", help="explicit files to inspect")
    parser.add_argument(
        "--override-message",
        help="text to scan for a D14-OVERRIDE marker (in addition to the tip commit)",
    )
    args = parser.parse_args(argv)

    base = _maint.discover_base(args.base)
    files = args.files if args.files is not None else _maint.changed_files(base)
    count = count_reviewable_lines(base, files)
    budget = derive_budget()
    override_reason = None
    if count > budget:
        override_reason = find_override(_override_text(args.override_message))

    report, code = render_report(count, budget, override_reason)
    print(report)

    divergence_warning = check_overhead_divergence()
    if divergence_warning is not None:
        print(f"warning: {divergence_warning}")

    return code


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
