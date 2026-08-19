#!/usr/bin/env python3
"""Shared runtime prompt builder consuming prompt_anchors.py (LRPC-1).

Assembles a role's canonical, provenance-tagged authority-boundary clauses
(scripts/local-agent/prompt_anchors.py) with a caller-supplied output-format
contract into one system prompt, and fails closed -- before any Ollama call
is constructed -- if the assembled prompt does not fit the caller's token
budget.

Scope (LRPC-2, docs/tasks/local-role-prompt-canonicalization.md): this
module produces the prompt string only. It never performs network IO itself;
wiring the three consumer scripts (gemma-code-review.py, run_local_task.py,
run_analysis.py) to call it is out of scope here (LRPC-3/4/5).

Adopted scoping decision (Option (b), phase-1 review
docs/audit/gemma-evidence/LRPC-2-phase1.json): prompt_anchors.Clause carries
no rationale/permission/prohibition classification, so the plan's Design
Decision 3 three-way cut order cannot be implemented against it yet. This
builder is a hard-limit validator only -- it raises when a role's anchor
does not fit the budget, it never truncates or drops individual clauses.
"""

from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from prompt_anchors import ROLE_ANCHORS  # noqa: E402
from gemma_local import estimate_text_tokens  # noqa: E402

# Fixed token cost reserved for the parts of an assembled prompt this module
# does not itself measure precisely (role-clause joining, minor formatting).
# Mirrors check-review-budget.py's PACKET_OVERHEAD_TOKENS: a small constant
# tunable without a code change, not a value invented independently per
# caller. Distinct from that script's own overhead -- this one bounds a
# system prompt, not a delegation packet.
PROMPT_BUILDER_OVERHEAD_TOKENS = 64


class UnknownRoleError(KeyError):
    """Raised when `role` has no entry in prompt_anchors.ROLE_ANCHORS."""

    def __init__(self, role: str) -> None:
        super().__init__(f"Unknown prompt-builder role: {role!r}")
        self.role = role


class PromptBudgetExceeded(RuntimeError):
    """Raised when the assembled prompt exceeds the derived token budget.

    Raised before any Ollama call is constructed -- this module never
    performs network IO, so the exception is the sole signal a caller needs
    to abort before spending a request on a prompt that would not fit.
    """

    def __init__(self, role: str, estimated_tokens: int, budget_tokens: int) -> None:
        super().__init__(
            f"Prompt for role {role!r} is estimated at {estimated_tokens} tokens, "
            f"exceeding the derived budget of {budget_tokens} tokens."
        )
        self.role = role
        self.estimated_tokens = estimated_tokens
        self.budget_tokens = budget_tokens


def _overhead_tokens() -> int:
    """Fixed token cost reserved before the assembled clauses/output-format text.

    `DUBBRIDGE_PROMPT_BUILDER_OVERHEAD_TOKENS` tunes this without a code
    change, mirroring check-review-budget.py's env-override pattern.
    """
    raw = os.environ.get("DUBBRIDGE_PROMPT_BUILDER_OVERHEAD_TOKENS")
    if raw is None or not raw.strip():
        return PROMPT_BUILDER_OVERHEAD_TOKENS
    try:
        return int(raw.strip())
    except ValueError:
        return PROMPT_BUILDER_OVERHEAD_TOKENS


def derive_prompt_budget(num_ctx: int, num_predict: int) -> int:
    """Return the max prompt tokens available, derived from the invocation's
    context window.

    budget = max(0, num_ctx - num_predict - overhead)

    Mirrors scripts/check-review-budget.py's derive_budget() subtraction
    shape (num_ctx - num_predict - packet_overhead); does not reuse that
    script's TOKENS_PER_DIFF_LINE divisor, which is specific to counting
    diff lines rather than a single assembled prompt string. Clamped to 0
    (Gemma Reviewer phase-2 finding, LRPC-2): an undersized num_ctx/oversized
    num_predict must never produce a negative budget, which would make
    PromptBudgetExceeded's own "estimated vs budget" comparison nonsensical.
    """
    return max(0, num_ctx - num_predict - _overhead_tokens())


def build_system_prompt(
    role: str,
    num_ctx: int,
    num_predict: int,
    *,
    output_format_text: str,
) -> str:
    """Compose role's canonical authority-boundary clauses with the caller's
    output-format contract into one system prompt.

    Pure function: no network IO, no side effects. Raises `UnknownRoleError`
    for an unregistered role, or `PromptBudgetExceeded` if the assembled
    prompt does not fit the budget derived from `num_ctx`/`num_predict` --
    in both cases before returning any string a caller could send to Ollama.
    """
    if role not in ROLE_ANCHORS:
        raise UnknownRoleError(role)

    clauses = ROLE_ANCHORS[role]
    clause_block = "\n".join(clause.text for clause in clauses)
    prompt = f"{clause_block}\n\n{output_format_text}"

    budget_tokens = derive_prompt_budget(num_ctx, num_predict)
    estimated_tokens = estimate_text_tokens(prompt)
    if estimated_tokens > budget_tokens:
        raise PromptBudgetExceeded(role, estimated_tokens, budget_tokens)

    return prompt
