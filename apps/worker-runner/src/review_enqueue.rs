use dubbridge_db::{review_repo, workspace_repo};
use dubbridge_domain::asset::AssetId;
use dubbridge_domain::review::ReviewTaskId;
use dubbridge_domain::workspace::ProjectId;
use sqlx::PgPool;
use time::OffsetDateTime;

/// Prepare the review-task post-ready flow after subtitles reach `Ready`.
///
/// Returns no error to the caller — all failures (missing org/project/target-language,
/// non-unique-violation DB errors, connection failures) are caught internally, logged with
/// `tracing::warn!`, and swallowed so that the subtitle `Ready` status written by T3b
/// always stands regardless of review-task enqueue outcome.
pub async fn prepare_review_post_ready(
    pool: &PgPool,
    asset_id: AssetId,
    project_id: ProjectId,
    target_language: &str,
) {
    let org = match workspace_repo::get_project(pool, project_id).await {
        Ok(Some(p)) => p.org_id,
        Ok(None) => {
            tracing::warn!(
                asset_id = %asset_id.0,
                "no project found for subtitle ready review-task enqueue"
            );
            return;
        }
        Err(error) => {
            tracing::warn!(
                asset_id = %asset_id.0,
                error = %error,
                "get_project failed for subtitle ready review-task enqueue"
            );
            return;
        }
    };

    let target_language_rows = match workspace_repo::list_target_languages(pool, project_id).await {
        Ok(rows) => rows,
        Err(error) => {
            tracing::warn!(
                asset_id = %asset_id.0,
                error = %error,
                "list_target_languages failed for subtitle ready review-task enqueue"
            );
            return;
        }
    };

    let target_language_id = match target_language_rows
        .iter()
        .find(|tl| tl.target_lang == target_language)
        .map(|tl| tl.id)
    {
        Some(id) => id,
        None => {
            tracing::warn!(
                asset_id = %asset_id.0,
                "no matching target-language row for subtitle ready review-task enqueue"
            );
            return;
        }
    };

    let task = dubbridge_domain::review::ReviewTask {
        id: ReviewTaskId::new(),
        org_id: org,
        project_id,
        asset_id,
        target_language_id,
        assignee_subject_id: None,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        assigned_at: None,
    };

    match review_repo::insert_review_task(pool, &task).await {
        Ok(()) => {}
        Err(dubbridge_db::error::DbError::QueryFailed(sqlx::Error::Database(db_err)))
            if db_err.is_foreign_key_violation() =>
        {
            // Foreign key constraint violation (e.g. target_language_id doesn't exist).
            tracing::warn!(
                asset_id = %asset_id.0,
                "insert_review_task failed with FK violation for subtitle ready review-task enqueue"
            );
            return;
        }
        Err(dubbridge_db::error::DbError::QueryFailed(sqlx::Error::Database(db_err))) => {
            // Check for unique-violation SQLSTATE 23505 — idempotent success
            if let Some(code) = db_err.code() {
                if code.as_ref() == "23505" {
                    tracing::debug!(
                        asset_id = %asset_id.0,
                        "review task already exists (idempotent duplicate)"
                    );
                    return;
                }
            }
            // Other QueryFailed errors — log and swallow
            tracing::warn!(
                asset_id = %asset_id.0,
                error = %sqlx::Error::Database(db_err),
                "insert_review_task failed for subtitle ready review-task enqueue"
            );
        }
        Err(error) => {
            // Any other DbError (ConnectionFailed, NotFound, etc.) — log and swallow
            tracing::warn!(
                asset_id = %asset_id.0,
                error = %error,
                "insert_review_task failed for subtitle ready review-task enqueue"
            );
        }
    }
}
