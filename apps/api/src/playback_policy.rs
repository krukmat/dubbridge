use dubbridge_domain::{
    asset::AssetId,
    playback::PlaybackScope,
    workspace::{OrgId, ProjectId},
};
use uuid::Uuid;

use crate::playback_api_error::ApiError;

/// Policy context reserved for S-180 audience-facing decisions.
/// T4c ships a no-op default so the grant contract does not need to change later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackAudiencePolicyContext {
    pub asset_id: AssetId,
    pub actor_subject_id: Uuid,
    pub org_id: OrgId,
    pub project_id: ProjectId,
    pub scope: PlaybackScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum PlaybackAudiencePolicyDecision {
    Allow,
    Deny { reason: &'static str },
}

pub fn apply_audience_policy_hook(context: &PlaybackAudiencePolicyContext) -> Result<(), ApiError> {
    enforce_audience_policy_decision(default_audience_policy_hook(context))
}

fn default_audience_policy_hook(
    _context: &PlaybackAudiencePolicyContext,
) -> PlaybackAudiencePolicyDecision {
    PlaybackAudiencePolicyDecision::Allow
}

fn enforce_audience_policy_decision(
    decision: PlaybackAudiencePolicyDecision,
) -> Result<(), ApiError> {
    match decision {
        PlaybackAudiencePolicyDecision::Allow => Ok(()),
        PlaybackAudiencePolicyDecision::Deny { reason } => Err(ApiError::forbidden(reason)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audience_policy_context() -> PlaybackAudiencePolicyContext {
        PlaybackAudiencePolicyContext {
            asset_id: AssetId(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440120").expect("uuid"),
            ),
            actor_subject_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440121")
                .expect("uuid"),
            org_id: OrgId(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440122").expect("uuid")),
            project_id: ProjectId(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440123").expect("uuid"),
            ),
            scope: PlaybackScope::Review,
        }
    }

    #[test]
    fn default_audience_policy_hook_allows_by_default() {
        assert_eq!(
            default_audience_policy_hook(&audience_policy_context()),
            PlaybackAudiencePolicyDecision::Allow
        );
    }

    #[test]
    fn apply_audience_policy_hook_is_pass_through_by_default() {
        assert!(apply_audience_policy_hook(&audience_policy_context()).is_ok());
    }

    #[test]
    fn denied_audience_policy_decision_maps_to_forbidden_api_error() {
        let error = enforce_audience_policy_decision(PlaybackAudiencePolicyDecision::Deny {
            reason: "policy_denied",
        })
        .expect_err("deny should error");

        let response = axum::response::IntoResponse::into_response(error);
        assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
    }
}
