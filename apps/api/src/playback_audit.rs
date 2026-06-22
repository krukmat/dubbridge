use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
    workspace::{OrgId, ProjectId},
};
use serde_json::json;
use uuid::Uuid;

pub fn playback_grant_refused_event(
    asset_id: AssetId,
    actor_subject_id: Option<Uuid>,
    org_id: Option<OrgId>,
    project_id: Option<ProjectId>,
    reason: &'static str,
) -> AuditEvent {
    AuditEvent::new_playback_event(
        asset_id,
        AuditEventKind::PlaybackGrantRefused,
        Some(
            json!({
                "asset_id": asset_id.0,
                "actor_subject_id": actor_subject_id,
                "org_id": org_id.map(|value| value.0),
                "project_id": project_id.map(|value| value.0),
                "reason": reason,
            })
            .to_string(),
        ),
    )
}
