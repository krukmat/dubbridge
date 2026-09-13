#!/usr/bin/env python3
"""Normalized execution-contract types for DubBridge software-factory paths.

This module is deliberately policy-free. It receives already-resolved execution
facts from the current workflow/runtime code and normalizes them into a stable
shape for execution, handoff, and audit correlation.

It does not score RRI, select fallback models, retrieve context, run reviewers,
or own retry/repair state machines.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Optional


POLICY_FAMILY = "rri"
CURRENT_POLICY_VERSION = "rri-v2"

EXECUTION_MODE_LOCAL = "local"
EXECUTION_MODE_CLOUD_HANDOFF = "cloud_handoff_required"


@dataclass(frozen=True)
class LogicalBinding:
    """Portable project-facing execution identity.

    No priority, ranking, eligibility, or fallback policy belongs here.
    """

    binding_id: str
    role: str


@dataclass(frozen=True)
class RuntimePreset:
    """Concrete runtime/vendor parameters for one already-selected binding."""

    runtime_class: str
    model: str
    num_ctx: int
    num_predict: int
    thinking_mode: str


@dataclass(frozen=True)
class ResolvedExecutionConstraints:
    """Normalized view over execution state already resolved upstream."""

    task_id: str
    policy_family: str
    policy_version: str
    rri: Optional[int]
    band: Optional[str]
    execution_mode: str
    logical_binding: LogicalBinding
    runtime_preset: Optional[RuntimePreset]
    local_execution_allowed: bool
    max_total_turns: int
    max_repair_attempts: int
    authorization_ref: Optional[str] = None

    def as_dict(self):
        return asdict(self)


def normalize_resolved_execution(
    *,
    card,
    limits,
    model: str,
    num_ctx: int,
    num_predict: int,
    runtime_class: str = "ollama",
    thinking_mode: str = "off",
    authorization_ref: Optional[str] = None,
) -> ResolvedExecutionConstraints:
    """Normalize already-resolved card/limit/runtime state.

    `limits` remains the authority for whether local execution is permitted.
    This function does not infer or recompute routing policy.
    """

    if limits.local_execution_allowed:
        binding_id = getattr(limits, "logical_binding", None) or "local-implementer"
        logical_binding = LogicalBinding(
            binding_id=binding_id,
            role="local-implementer",
        )
        execution_mode = EXECUTION_MODE_LOCAL
        runtime_preset = RuntimePreset(
            runtime_class=runtime_class,
            model=model,
            num_ctx=num_ctx,
            num_predict=num_predict,
            thinking_mode=thinking_mode,
        )
    else:
        logical_binding = LogicalBinding(
            binding_id="cloud-handoff",
            role="cloud-implementer",
        )
        execution_mode = EXECUTION_MODE_CLOUD_HANDOFF
        runtime_preset = None

    policy_version = (
        getattr(card, "policy_version", None) or CURRENT_POLICY_VERSION
    )

    return ResolvedExecutionConstraints(
        task_id=card.task_id,
        policy_family=POLICY_FAMILY,
        policy_version=policy_version,
        rri=getattr(card, "rri", None),
        band=getattr(card, "band", None),
        execution_mode=execution_mode,
        logical_binding=logical_binding,
        runtime_preset=runtime_preset,
        local_execution_allowed=limits.local_execution_allowed,
        max_total_turns=limits.max_total_turns,
        max_repair_attempts=limits.max_repair_attempts,
        authorization_ref=authorization_ref,
    )
