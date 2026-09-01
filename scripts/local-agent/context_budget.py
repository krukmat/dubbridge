from __future__ import annotations

import json
from dataclasses import dataclass

import gemma_local

DEFAULT_HISTORY_RESERVE_TOKENS = 2048
DEFAULT_SAFETY_MARGIN_TOKENS = 512


@dataclass(frozen=True)
class InvocationBudget:
    num_ctx: int
    num_predict: int
    fixed_tokens: int
    task_tokens: int
    acceptance_tokens: int
    history_reserve_tokens: int
    safety_margin_tokens: int
    retrieval_budget_tokens: int

    def as_dict(self):
        return {
            "num_ctx": self.num_ctx,
            "num_predict": self.num_predict,
            "fixed_prompt_tokens": self.fixed_tokens,
            "task_tokens": self.task_tokens,
            "acceptance_tokens": self.acceptance_tokens,
            "history_reserve_tokens": self.history_reserve_tokens,
            "safety_margin_tokens": self.safety_margin_tokens,
            "retrieval_budget_tokens": self.retrieval_budget_tokens,
        }


def derive_invocation_budget(
    *,
    num_ctx,
    num_predict,
    system_prompt,
    task_spec,
    allowed_paths,
    acceptance_tests,
    history_reserve_tokens=DEFAULT_HISTORY_RESERVE_TOKENS,
    safety_margin_tokens=DEFAULT_SAFETY_MARGIN_TOKENS,
):
    if num_ctx <= 0 or num_predict <= 0:
        raise ValueError("num_ctx and num_predict must be greater than zero")
    fixed_payload = (
        system_prompt
        + "\n\nAllowed paths (complete capability list):\n"
        + json.dumps(allowed_paths, ensure_ascii=False, indent=2)
    )
    acceptance_payload = json.dumps(acceptance_tests, ensure_ascii=False, indent=2)
    fixed_tokens = gemma_local.estimate_text_tokens(fixed_payload)
    task_tokens = gemma_local.estimate_text_tokens(task_spec)
    acceptance_tokens = gemma_local.estimate_text_tokens(acceptance_payload)
    retrieval_budget = max(
        0,
        num_ctx
        - num_predict
        - fixed_tokens
        - task_tokens
        - acceptance_tokens
        - history_reserve_tokens
        - safety_margin_tokens,
    )
    return InvocationBudget(
        num_ctx=num_ctx,
        num_predict=num_predict,
        fixed_tokens=fixed_tokens,
        task_tokens=task_tokens,
        acceptance_tokens=acceptance_tokens,
        history_reserve_tokens=history_reserve_tokens,
        safety_margin_tokens=safety_margin_tokens,
        retrieval_budget_tokens=retrieval_budget,
    )
