#!/usr/bin/env python3
"""Adjudicator trigger gate and isolation packet builder for DubBridge (D14).

Implements the deterministic half of the context-isolated adjudication step:
  - should_adjudicate() decides when isolation is required.
  - build_adjudicator_packet() assembles the clean input for the isolated reviewer,
    enforcing by allowlist that no development-transcript content is included.

The adjudicator is advisory; the primary agent remains orchestrator of record
and owns the close (HITL_AUTONOMY_POLICY.md). Its only new obligation is to
reconcile its disposition against the adjudicator's and record
disposition_divergence in the audit log.

disposition_divergence audit field (orchestrator-supplied, defaults to null):
  "none"    -- adjudicator agreed with the primary's disposition
  "partial" -- a subset of findings diverged between primary and adjudicator
  "full"    -- adjudicator disposition fully differs from the primary's
  null      -- adjudicator was not spawned, or the field has not been populated

Authority unchanged: spawning the isolated reviewer is an orchestrator-runtime
action. This module delivers the inspectable, testable scaffolding it runs on.
"""

# Exhaustive allowlist of sections the adjudicator packet may contain.
# The isolation guarantee is enforced by this set — if a caller injects a key
# outside it, _assert_packet_isolation() raises before the packet is returned.
ALLOWED_PACKET_SECTIONS = frozenset({"diff", "criteria", "reconciled_findings"})

# Valid string values for the disposition_divergence audit field.
# null (Python None) is also valid and means "not yet populated".
DISPOSITION_DIVERGENCE_VALUES = frozenset({"none", "partial", "full"})


def should_adjudicate(aggregate, band, *, gemma_blocked=False):
    """Return True when context-isolated adjudication is required (D14).

    D14 is now a fallback for unusable local review output, not an escalation
    path for the content of a usable Gemma packet. Findings of every severity
    and every reconciliation bucket are left to the primary developer's
    disposition once Gemma produces a parseable consolidated result.

    Args:
        aggregate:     the reconciled aggregate result dict produced by
                       gemma-code-review.py. Missing or BLOCKED aggregates are
                       unusable and trigger fallback; findings content does not.
        band:          the slice RRI band string as defined in CLAUDE.md
                       ("Low", "Moderate", "Med-high", "Complex"). Kept for API
                       compatibility; band no longer triggers D14 by itself.
        gemma_blocked: True when Gemma was unavailable, stalled, returned invalid
                       output, or otherwise failed to produce a usable
                       consolidated result.
    """
    if gemma_blocked:
        return True
    if not aggregate:
        return True
    return aggregate.get("status") == "blocked"


def build_adjudicator_packet(diff, criteria, reconciled_findings):
    """Assemble the clean input packet for a context-isolated adjudicator.

    The packet contains exactly the three allowed sections. No development-
    transcript content (chain-of-thought, implementation notes, dead-ends)
    crosses the boundary. Isolation is enforced by an allowlist at construction
    time, not by a denylist applied after the fact.

    Args:
        diff:                 the final unified diff string.
        criteria:             the task acceptance criteria text.
        reconciled_findings:  list of finding dicts from the reconciliation
                              block of the T5 aggregate result.

    Returns:
        dict with exactly the keys {"diff", "criteria", "reconciled_findings"}.

    Raises:
        ValueError: if the constructed packet violates the allowlist (fails closed).
    """
    packet = {
        "diff": diff,
        "criteria": criteria,
        "reconciled_findings": reconciled_findings,
    }
    _assert_packet_isolation(packet)
    return packet


def _assert_packet_isolation(packet):
    """Raise ValueError if packet contains sections outside the allowlist.

    Called after construction so the check is on the actual output, not on
    caller intent. Fails closed: any extra key triggers the error before the
    packet is returned to callers.
    """
    keys = set(packet.keys())
    extra = keys - ALLOWED_PACKET_SECTIONS
    if extra:
        raise ValueError(
            f"adjudicator packet isolation violation: disallowed sections "
            f"{sorted(extra)}; allowed: {sorted(ALLOWED_PACKET_SECTIONS)}"
        )
    missing = ALLOWED_PACKET_SECTIONS - keys
    if missing:
        raise ValueError(
            f"adjudicator packet missing required sections {sorted(missing)}"
        )
