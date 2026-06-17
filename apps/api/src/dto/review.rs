use dubbridge_domain::review::{PublicationStatus, ReviewTaskState, ReviewVerdict};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_db::review_repo::ReviewTaskWithState;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ReviewTaskResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub asset_id: Uuid,
    pub target_language_id: Uuid,
    pub assignee_subject_id: Option<Uuid>,
    pub state: ReviewTaskState,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub assigned_at: Option<OffsetDateTime>,
}

impl From<ReviewTaskWithState> for ReviewTaskResponse {
    fn from(value: ReviewTaskWithState) -> Self {
        Self {
            id: value.task.id.0,
            org_id: value.task.org_id.0,
            project_id: value.task.project_id.0,
            asset_id: value.task.asset_id.0,
            target_language_id: value.task.target_language_id,
            assignee_subject_id: value.task.assignee_subject_id,
            state: value.state,
            created_at: value.task.created_at,
            updated_at: value.task.updated_at,
            assigned_at: value.task.assigned_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ReviewQueueResponse {
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub tasks: Vec<ReviewTaskResponse>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewDecisionRequest {
    pub verdict: ReviewVerdict,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ReviewDecisionResponse {
    pub review_task_id: Uuid,
    pub state: ReviewTaskState,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ReviewPublicationResponse {
    pub review_task_id: Uuid,
    pub status: PublicationStatus,
    pub published_by: Uuid,
    pub published_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use dubbridge_domain::{
        asset::AssetId,
        review::{PublicationRow, ReviewTask, ReviewTaskId},
        workspace::{OrgId, ProjectId},
    };

    use super::*;

    #[test]
    fn review_task_response_maps_repo_fields() {
        let task = ReviewTask {
            id: ReviewTaskId::new(),
            org_id: OrgId::new(),
            project_id: ProjectId::new(),
            asset_id: AssetId::new(),
            target_language_id: Uuid::new_v4(),
            assignee_subject_id: Some(Uuid::new_v4()),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            assigned_at: Some(OffsetDateTime::now_utc()),
        };
        let item = ReviewTaskWithState {
            task: task.clone(),
            state: ReviewTaskState::Pending,
        };

        let response = ReviewTaskResponse::from(item);
        assert_eq!(response.id, task.id.0);
        assert_eq!(response.org_id, task.org_id.0);
        assert_eq!(response.project_id, task.project_id.0);
        assert_eq!(response.state, ReviewTaskState::Pending);
    }

    #[test]
    fn review_queue_response_contains_tasks() {
        let response = ReviewQueueResponse {
            org_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            tasks: vec![],
        };

        assert!(response.tasks.is_empty());
    }

    #[test]
    fn review_publication_response_maps_publication_fields() {
        let row = PublicationRow {
            id: Uuid::new_v4(),
            review_task_id: ReviewTaskId::new(),
            status: PublicationStatus::Published,
            published_by: Uuid::new_v4(),
            published_at: OffsetDateTime::now_utc(),
        };

        let response = ReviewPublicationResponse {
            review_task_id: row.review_task_id.0,
            status: row.status.clone(),
            published_by: row.published_by,
            published_at: row.published_at,
        };

        assert_eq!(response.review_task_id, row.review_task_id.0);
        assert_eq!(response.status, PublicationStatus::Published);
    }
}
