import importlib.util
import os
import sys
from pathlib import Path

_REPO_ROOT = Path(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
_SCHEMA_PATH = _REPO_ROOT / "local-agent" / "handoff_schema.py"

if "handoff_schema" in sys.modules:
    handoff_schema = sys.modules["handoff_schema"]
else:
    _SPEC = importlib.util.spec_from_file_location("handoff_schema", _SCHEMA_PATH)
    handoff_schema = importlib.util.module_from_spec(_SPEC)
    sys.modules["handoff_schema"] = handoff_schema
    _SPEC.loader.exec_module(handoff_schema)

ADVISORY_ONLY_OUTCOME = "advisory-only"

def map_packet_to_capsule(project_packet: dict, work_item_id: str, repo_revision: str) -> handoff_schema.Capsule:
    """Maps an ADR-037 project packet to a Capsule for context passing."""
    
    # Derive objective
    objective = project_packet.get("objective") or f"ADR-037 preplanning capsule for {work_item_id}"
    
    # Build the dict with all CAPSULE_REQUIRED_FIELDS
    capsule_data = {
        "work_item_id": work_item_id,
        "objective": objective,
        "non_goals": ["This capsule grants no implementation or approval authority beyond ADR-037."],
        "questions": project_packet.get("questions", []),
        "current_behavior": project_packet.get("current_behavior") or "See ADR-037 packet for full context.",
        "required_behavior": project_packet.get("required_behavior") or "See ADR-037 packet for full context.",
        "constraints": ["ADR-037 advisory-only: no implementation or approval authority."] + 
                       project_packet.get("constraints", []),
        "allowed_paths": [],  # Local Architect is read-only
        "acceptance_criteria": project_packet.get("acceptance_criteria", []),
        "repo_revision": repo_revision
    }
    
    return handoff_schema.validate_capsule(capsule_data)

def map_artifact_to_attempt_bundle(analysis_artifact: dict, capsule_hash: str, implementer_id: str = "adr037-local-architect") -> dict:
    """Maps an ADR-037 analysis artifact to an AttemptBundle dictionary."""
    
    # Determine outcome based on success/status
    if analysis_artifact.get("success") is True:
        outcome = ADVISORY_ONLY_OUTCOME
    elif analysis_artifact.get("status") == "failed":
        outcome = "blocked"
    else:
        outcome = "escalated"

    bundle_data = {
        "capsule_hash": capsule_hash,
        "implementer_id": implementer_id,
        "model_tag": analysis_artifact.get("model", {}).get("tag"),
        "start_ts": analysis_artifact.get("started_at"),
        "end_ts": analysis_artifact.get("finished_at"),
        "diff_ref": None,
        "test_results": None,
        "review_verdict": None,
        "outcome": outcome
    }

    # Manual validation of BUNDLE_REQUIRED_FIELDS as per requirement 1
    for field in handoff_schema.BUNDLE_REQUIRED_FIELDS:
        if field not in bundle_data:
            raise ValueError(f"Missing required bundle field: {field}")

    return bundle_data