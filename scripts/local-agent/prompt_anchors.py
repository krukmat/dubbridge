from __future__ import annotations
from dataclasses import dataclass

@dataclass(frozen=True)
class Clause:
    text: str
    source_file: str
    source_section: str

ROLE_ANCHORS: dict[str, list[Clause]] = {
    "gemma_reviewer": [
        Clause(
            text="may not write files, apply patches, approve tasks,\n  certify coverage, or mark tasks complete",
            source_file="docs/playbooks/AGENT_WORKFLOW_GUIDE.md",
            source_section="Gemma Reviewer / GPT-OSS 20B Reviewer > Authority boundary"
        )
    ],
    "local_developer": [
        Clause(
            text="Every edit is limited to the card's\n`allowed_paths`; any model-issued read, command, or unlisted-path access\nterminates immediately as `boundary_violation`.",
            source_file="docs/playbooks/AGENT_WORKFLOW_GUIDE.md",
            source_section="Handoff prompt format"
        )
    ],
    "local_architect_default": [
        Clause(
            text="The role may not:\n\n- edit source code, tests, configuration, policies, ledgers, or canonical ADRs;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="run shell commands or operate a repository worktree;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="act as an implementation agent, code reviewer, task-analysis reviewer, technical\n  judge, approver, coverage certifier, or owner verifier;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="replace Gemma Reviewer, the RRI 41+ cross-vendor peer, D14, the primary agent, or\n  the human decision maker;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="declare a design approved, implemented, production-ready, or verified.",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        )
    ],
    "local_architect_med_high": [
        Clause(
            text="The role may not:\n\n- edit source code, tests, configuration, policies, ledgers, or canonical ADRs;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="run shell commands or operate a repository worktree;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="act as an implementation agent, code reviewer, task-analysis reviewer, technical\n  judge, approver, coverage certifier, or owner verifier;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="replace Gemma Reviewer, the RRI 41+ cross-vendor peer, D14, the primary agent, or\n  the human decision maker;",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        ),
        Clause(
            text="declare a design approved, implemented, production-ready, or verified.",
            source_file="docs/adr/ADR-037-qwen36-27b-local-architect-complex-analyst.md",
            source_section="Decision > 1. Add one advisory role: Local Architect / Complex Analyst > The role may not:"
        )
    ]
}
