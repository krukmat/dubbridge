// S-160-T2a: fail-closed review gate per ADR-008 / ADR-030
// S-160-T2b: durable audit wiring per ADR-018
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use dubbridge_audit::emit_governance_audit;
use dubbridge_db::{
    error::DbError,
    notification_repo::{self, NotificationKind, NotificationRow, RefEntityType},
    review_repo,
};
use dubbridge_domain::audit::{AuditEvent, AuditEventKind};
use dubbridge_domain::review::{
    PublicationRow, PublicationStatus, ReviewDecisionRow, ReviewTask, ReviewTaskId,
    ReviewTaskState, ReviewVerdict,
};

#[derive(Debug, PartialEq)]
pub enum ReviewGateError {
    ReviewTaskNotFound {
        review_task_id: ReviewTaskId,
    },
    ReviewNotApproved {
        review_task_id: ReviewTaskId,
        state: ReviewTaskState,
    },
    AlreadyPublished {
        review_task_id: ReviewTaskId,
    },
    Db(String),
    AuditFailed(String),
}

impl std::fmt::Display for ReviewGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReviewTaskNotFound { review_task_id } => {
                write!(f, "review task not found: {}", review_task_id.0)
            }
            Self::ReviewNotApproved {
                review_task_id,
                state,
            } => write!(
                f,
                "review task {} is not approved; current state is {}",
                review_task_id.0, state
            ),
            Self::AlreadyPublished { review_task_id } => {
                write!(f, "review task {} is already published", review_task_id.0)
            }
            Self::Db(msg) => write!(f, "review gate db error: {msg}"),
            Self::AuditFailed(msg) => write!(f, "review gate audit failed: {msg}"),
        }
    }
}

impl From<DbError> for ReviewGateError {
    fn from(error: DbError) -> Self {
        Self::Db(error.to_string())
    }
}

impl From<dubbridge_audit::AuditEmitError> for ReviewGateError {
    fn from(error: dubbridge_audit::AuditEmitError) -> Self {
        Self::AuditFailed(error.to_string())
    }
}

fn new_notification(
    recipient_subject_id: Uuid,
    actor_subject_id: Uuid,
    kind: NotificationKind,
    ref_entity_id: Uuid,
) -> NotificationRow {
    NotificationRow {
        id: Uuid::new_v4(),
        recipient_subject_id,
        kind,
        ref_entity_type: RefEntityType::ReviewTask,
        ref_entity_id,
        actor_subject_id: Some(actor_subject_id),
        read_at: None,
        created_at: OffsetDateTime::now_utc(),
    }
}

/// Pure fail-closed publication check over a pre-fetched review state.
pub fn require_approved_for_publish_with(
    state: ReviewTaskState,
    review_task_id: ReviewTaskId,
) -> Result<(), ReviewGateError> {
    match state {
        ReviewTaskState::Approved => Ok(()),
        other => Err(ReviewGateError::ReviewNotApproved {
            review_task_id,
            state: other,
        }),
    }
}

fn new_decision(
    review_task_id: ReviewTaskId,
    reviewer_subject_id: Uuid,
    verdict: ReviewVerdict,
    comment: Option<String>,
) -> ReviewDecisionRow {
    ReviewDecisionRow {
        id: Uuid::new_v4(),
        review_task_id,
        verdict,
        comment,
        reviewer_subject_id,
        happened_at: OffsetDateTime::now_utc(),
    }
}

async fn ensure_task_exists(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
) -> Result<ReviewTask, ReviewGateError> {
    if let Some(task) = review_repo::get_review_task(pool, review_task_id).await? {
        Ok(task)
    } else {
        Err(ReviewGateError::ReviewTaskNotFound { review_task_id })
    }
}

fn audit_kind_for_verdict(verdict: ReviewVerdict) -> AuditEventKind {
    match verdict {
        ReviewVerdict::Approved => AuditEventKind::ReviewApproved,
        ReviewVerdict::Rejected => AuditEventKind::ReviewRejected,
    }
}

fn review_decision_audit_event(
    task: &ReviewTask,
    review_task_id: ReviewTaskId,
    verdict: ReviewVerdict,
    reviewer_subject_id: Uuid,
) -> AuditEvent {
    AuditEvent::new_review_event(
        task.asset_id,
        audit_kind_for_verdict(verdict.clone()),
        Some(format!(
            "review_task_id={};org_id={};project_id={};target_language_id={};actor_subject_id={};verdict={}",
            review_task_id.0,
            task.org_id.0,
            task.project_id.0,
            task.target_language_id,
            reviewer_subject_id,
            verdict
        )),
    )
}

fn publish_success_audit_event(
    task: &ReviewTask,
    review_task_id: ReviewTaskId,
    published_by: Uuid,
) -> AuditEvent {
    AuditEvent::new_review_event(
        task.asset_id,
        AuditEventKind::PublicationSucceeded,
        Some(format!(
            "review_task_id={};org_id={};project_id={};target_language_id={};actor_subject_id={};publication_state=published",
            review_task_id.0,
            task.org_id.0,
            task.project_id.0,
            task.target_language_id,
            published_by
        )),
    )
}

fn publish_refused_audit_event(
    task: &ReviewTask,
    review_task_id: ReviewTaskId,
    actor_subject_id: Uuid,
    reason: &str,
    current_state: Option<ReviewTaskState>,
) -> AuditEvent {
    let state = current_state
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    AuditEvent::new_review_event(
        task.asset_id,
        AuditEventKind::PublicationRefused,
        Some(format!(
            "review_task_id={};org_id={};project_id={};target_language_id={};actor_subject_id={};reason={};current_state={}",
            review_task_id.0,
            task.org_id.0,
            task.project_id.0,
            task.target_language_id,
            actor_subject_id,
            reason,
            state
        )),
    )
}

pub async fn approve_review_task(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
    reviewer_subject_id: Uuid,
    comment: Option<String>,
) -> Result<ReviewTaskState, ReviewGateError> {
    let task = ensure_task_exists(pool, review_task_id).await?;
    let row = new_decision(
        review_task_id,
        reviewer_subject_id,
        ReviewVerdict::Approved,
        comment,
    );
    review_repo::append_review_decision(pool, &row).await?;
    let state = review_repo::latest_review_state(pool, review_task_id).await?;
    let event = review_decision_audit_event(
        &task,
        review_task_id,
        ReviewVerdict::Approved,
        reviewer_subject_id,
    );
    emit_governance_audit(pool, &event).await?;
    let notification = new_notification(
        task.assignee_subject_id.unwrap_or(reviewer_subject_id),
        reviewer_subject_id,
        NotificationKind::ReviewTaskDecided,
        review_task_id.0,
    );
    notification_repo::insert_notification(pool, &notification)
        .await
        .map_err(|e| ReviewGateError::Db(e.to_string()))?;
    Ok(state)
}

pub async fn reject_review_task(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
    reviewer_subject_id: Uuid,
    comment: Option<String>,
) -> Result<ReviewTaskState, ReviewGateError> {
    let task = ensure_task_exists(pool, review_task_id).await?;
    let row = new_decision(
        review_task_id,
        reviewer_subject_id,
        ReviewVerdict::Rejected,
        comment,
    );
    review_repo::append_review_decision(pool, &row).await?;
    let state = review_repo::latest_review_state(pool, review_task_id).await?;
    let event = review_decision_audit_event(
        &task,
        review_task_id,
        ReviewVerdict::Rejected,
        reviewer_subject_id,
    );
    emit_governance_audit(pool, &event).await?;
    let notification = new_notification(
        task.assignee_subject_id.unwrap_or(reviewer_subject_id),
        reviewer_subject_id,
        NotificationKind::ReviewTaskDecided,
        review_task_id.0,
    );
    notification_repo::insert_notification(pool, &notification)
        .await
        .map_err(|e| ReviewGateError::Db(e.to_string()))?;
    Ok(state)
}

pub async fn publish_review_task(
    pool: &PgPool,
    review_task_id: ReviewTaskId,
    published_by: Uuid,
) -> Result<PublicationRow, ReviewGateError> {
    let task = ensure_task_exists(pool, review_task_id).await?;

    if review_repo::get_publication_for_review_task(pool, review_task_id)
        .await?
        .is_some()
    {
        let event = publish_refused_audit_event(
            &task,
            review_task_id,
            published_by,
            "already_published",
            Some(ReviewTaskState::Approved),
        );
        emit_governance_audit(pool, &event).await?;
        return Err(ReviewGateError::AlreadyPublished { review_task_id });
    }

    let state = review_repo::latest_review_state(pool, review_task_id).await?;
    if let Err(err) = require_approved_for_publish_with(state.clone(), review_task_id) {
        let event = publish_refused_audit_event(
            &task,
            review_task_id,
            published_by,
            "review_not_approved",
            Some(state),
        );
        emit_governance_audit(pool, &event).await?;
        return Err(err);
    }

    let row = PublicationRow {
        id: Uuid::new_v4(),
        review_task_id,
        status: PublicationStatus::Published,
        published_by,
        published_at: OffsetDateTime::now_utc(),
    };
    review_repo::insert_publication(pool, &row).await?;
    let event = publish_success_audit_event(&task, review_task_id, published_by);
    emit_governance_audit(pool, &event).await?;
    let notification = new_notification(
        task.assignee_subject_id.unwrap_or(published_by),
        published_by,
        NotificationKind::ReviewTaskPublished,
        review_task_id.0,
    );
    notification_repo::insert_notification(pool, &notification)
        .await
        .map_err(|e| ReviewGateError::Db(e.to_string()))?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_id() -> ReviewTaskId {
        ReviewTaskId::new()
    }

    #[test]
    fn approved_state_allows_publish() {
        let result = require_approved_for_publish_with(ReviewTaskState::Approved, task_id());
        assert!(result.is_ok());
    }

    #[test]
    fn pending_state_fails_closed() {
        let id = task_id();
        let result = require_approved_for_publish_with(ReviewTaskState::Pending, id);
        assert!(matches!(
            result,
            Err(ReviewGateError::ReviewNotApproved { review_task_id, state })
                if review_task_id == id && state == ReviewTaskState::Pending
        ));
    }

    #[test]
    fn rejected_state_fails_closed() {
        let id = task_id();
        let result = require_approved_for_publish_with(ReviewTaskState::Rejected, id);
        assert!(matches!(
            result,
            Err(ReviewGateError::ReviewNotApproved { review_task_id, state })
                if review_task_id == id && state == ReviewTaskState::Rejected
        ));
    }

    #[test]
    fn db_error_converts_to_gate_error() {
        let error = ReviewGateError::from(DbError::NotFound);
        assert!(matches!(error, ReviewGateError::Db(_)));
    }

    #[test]
    fn audit_emit_error_converts_to_gate_error() {
        let db_err = DbError::NotFound;
        let emit_err = dubbridge_audit::AuditEmitError::Db(db_err);
        let gate_err = ReviewGateError::from(emit_err);
        assert!(matches!(gate_err, ReviewGateError::AuditFailed(_)));
    }

    #[test]
    fn already_published_display_includes_task_id() {
        let id = task_id();
        let err = ReviewGateError::AlreadyPublished { review_task_id: id };
        assert!(err.to_string().contains(&id.0.to_string()));
    }

    #[test]
    fn review_decision_audit_event_includes_task_and_actor() {
        use dubbridge_domain::{
            asset::AssetId,
            workspace::{OrgId, ProjectId},
        };

        let task = ReviewTask {
            id: task_id(),
            org_id: OrgId(Uuid::new_v4()),
            project_id: ProjectId(Uuid::new_v4()),
            asset_id: AssetId::new(),
            target_language_id: Uuid::new_v4(),
            assignee_subject_id: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            assigned_at: None,
        };
        let actor = Uuid::new_v4();

        let event = review_decision_audit_event(&task, task.id, ReviewVerdict::Approved, actor);

        assert_eq!(event.event_kind, AuditEventKind::ReviewApproved);
        assert_eq!(event.asset_id, Some(task.asset_id));
        assert!(
            event
                .detail
                .as_deref()
                .expect("detail")
                .contains(&actor.to_string())
        );
    }

    #[test]
    fn publish_refused_audit_event_includes_reason_and_state() {
        use dubbridge_domain::{
            asset::AssetId,
            workspace::{OrgId, ProjectId},
        };

        let task = ReviewTask {
            id: task_id(),
            org_id: OrgId(Uuid::new_v4()),
            project_id: ProjectId(Uuid::new_v4()),
            asset_id: AssetId::new(),
            target_language_id: Uuid::new_v4(),
            assignee_subject_id: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            assigned_at: None,
        };

        let event = publish_refused_audit_event(
            &task,
            task.id,
            Uuid::new_v4(),
            "review_not_approved",
            Some(ReviewTaskState::Rejected),
        );

        assert_eq!(event.event_kind, AuditEventKind::PublicationRefused);
        let detail = event.detail.as_deref().expect("detail");
        assert!(detail.contains("reason=review_not_approved"));
        assert!(detail.contains("current_state=rejected"));
    }
}
