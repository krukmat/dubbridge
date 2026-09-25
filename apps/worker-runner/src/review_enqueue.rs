use dubbridge_db::error::DbError;
use dubbridge_db::{review_repo, workspace_repo};
use dubbridge_domain::asset::AssetId;
use dubbridge_domain::review::{ReviewTask, ReviewTaskId};
use dubbridge_domain::workspace::ProjectId;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// Create the governed review unit after one translated subtitle reaches Ready.
///
/// Translation readiness must remain durable even if review-task persistence fails,
/// so failures are logged and swallowed. The unique
/// (project_id, asset_id, target_language_id) constraint makes repeated delivery
/// idempotent.
pub async fn prepare_review_post_ready(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
    translated_subtitle_artifact_id: Uuid,
) {
    let Some(project) = resolve_project(pool, project_id, asset_id).await else {
        return;
    };

    let task = ReviewTask {
        id: ReviewTaskId::new(),
        org_id: project.org_id,
        project_id,
        asset_id,
        target_language_id,
        subtitle_artifact_id: Some(translated_subtitle_artifact_id),
        assignee_subject_id: None,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        assigned_at: None,
    };

    if let Err(error) = review_repo::insert_review_task(pool, &task).await {
        log_insert_outcome(asset_id, project_id, target_language_id, &error);
    }
}

async fn resolve_project(
    pool: &PgPool,
    project_id: ProjectId,
    asset_id: AssetId,
) -> Option<dubbridge_domain::workspace::Project> {
    match workspace_repo::get_project(pool, project_id).await {
        Ok(Some(project)) => Some(project),
        Ok(None) => {
            tracing::warn!(
                asset_id = %asset_id.0,
                project_id = %project_id.0,
                "review task not created after translation ready: project missing"
            );
            None
        }
        Err(error) => {
            tracing::warn!(
                asset_id = %asset_id.0,
                project_id = %project_id.0,
                error = %error,
                "review task not created after translation ready: project lookup failed"
            );
            None
        }
    }
}

fn log_insert_outcome(
    asset_id: AssetId,
    project_id: ProjectId,
    target_language_id: Uuid,
    error: &DbError,
) {
    if is_unique_violation(error) {
        tracing::debug!(
            asset_id = %asset_id.0,
            project_id = %project_id.0,
            target_language_id = %target_language_id,
            "review task already exists after translation ready"
        );
        return;
    }

    tracing::warn!(
        asset_id = %asset_id.0,
        project_id = %project_id.0,
        target_language_id = %target_language_id,
        error = %error,
        "review task creation failed after translation ready"
    );
}

fn is_unique_violation(error: &DbError) -> bool {
    let DbError::QueryFailed(sqlx::Error::Database(db_error)) = error else {
        return false;
    };
    db_error.code().as_deref() == Some("23505")
}
