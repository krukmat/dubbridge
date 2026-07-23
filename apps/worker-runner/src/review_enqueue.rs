use dubbridge_db::error::DbError;
use dubbridge_db::{review_repo, workspace_repo};
use dubbridge_domain::asset::AssetId;
use dubbridge_domain::review::{ReviewTask, ReviewTaskId};
use dubbridge_domain::workspace::{OrgId, ProjectId};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

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
    let Ok((org_id, target_language_id)) =
        resolve_review_identity(pool, asset_id, project_id, target_language).await
    else {
        return;
    };

    let task = ReviewTask {
        id: ReviewTaskId::new(),
        org_id,
        project_id,
        asset_id,
        target_language_id,
        assignee_subject_id: None,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        assigned_at: None,
    };

    if let Err(error) = review_repo::insert_review_task(pool, &task).await {
        log_insert_outcome(asset_id, &error);
    }
}

/// Resolve the `(org_id, target_language_id)` pair needed to build a `ReviewTask`.
///
/// Returns `Err(())` and logs a warning on any missing dependency or DB error —
/// callers must treat `Err` as "do not enqueue, do not fail the caller".
async fn resolve_review_identity(
    pool: &PgPool,
    asset_id: AssetId,
    project_id: ProjectId,
    target_language: &str,
) -> Result<(OrgId, Uuid), ()> {
    let org_id = resolve_org_id(pool, asset_id, project_id).await?;
    let target_language_id =
        resolve_target_language_id(pool, asset_id, project_id, target_language).await?;
    Ok((org_id, target_language_id))
}

async fn resolve_org_id(
    pool: &PgPool,
    asset_id: AssetId,
    project_id: ProjectId,
) -> Result<OrgId, ()> {
    let project = workspace_repo::get_project(pool, project_id)
        .await
        .map_err(|error| {
            tracing::warn!(
                asset_id = %asset_id.0,
                error = %error,
                "get_project failed for subtitle ready review-task enqueue"
            );
        })?;
    project.map(|p| p.org_id).ok_or_else(|| {
        tracing::warn!(
            asset_id = %asset_id.0,
            "no project found for subtitle ready review-task enqueue"
        );
    })
}

async fn resolve_target_language_id(
    pool: &PgPool,
    asset_id: AssetId,
    project_id: ProjectId,
    target_language: &str,
) -> Result<Uuid, ()> {
    let rows = workspace_repo::list_target_languages(pool, project_id)
        .await
        .map_err(|error| {
            tracing::warn!(
                asset_id = %asset_id.0,
                error = %error,
                "list_target_languages failed for subtitle ready review-task enqueue"
            );
        })?;

    rows.iter()
        .find(|tl| tl.target_lang == target_language)
        .map(|tl| tl.id)
        .ok_or_else(|| {
            tracing::warn!(
                asset_id = %asset_id.0,
                "no matching target-language row for subtitle ready review-task enqueue"
            );
        })
}

/// Log the outcome of a failed `insert_review_task` call. A unique-violation
/// (SQLSTATE 23505, `review_tasks_unique_review_unit`) is an idempotent duplicate and
/// logs at `debug`; every other error is unexpected and logs at `warn`. Either way the
/// error is swallowed — the caller never propagates it.
fn log_insert_outcome(asset_id: AssetId, error: &DbError) {
    if is_unique_violation(error) {
        log_idempotent_duplicate(asset_id);
    } else {
        log_insert_failure(asset_id, error);
    }
}

fn is_unique_violation(error: &DbError) -> bool {
    let DbError::QueryFailed(sqlx::Error::Database(db_err)) = error else {
        return false;
    };
    db_err.code().as_deref() == Some("23505")
}

fn log_idempotent_duplicate(asset_id: AssetId) {
    tracing::debug!(
        asset_id = %asset_id.0,
        "review task already exists (idempotent duplicate)"
    );
}

fn log_insert_failure(asset_id: AssetId, error: &DbError) {
    tracing::warn!(
        asset_id = %asset_id.0,
        error = %error,
        "insert_review_task failed for subtitle ready review-task enqueue"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_domain::workspace::{OrgId, Organization, Project, TargetLanguage};
    use uuid::Uuid;

    async fn setup_pool_for_test() -> Option<PgPool> {
        let url = std::env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("../../infra/migrations")
            .run(&pool)
            .await
            .expect("migrations");
        sqlx::query(
            "TRUNCATE TABLE review_tasks, target_languages, project_assets, projects, org_members, organizations, pending_ingestions, audit_events, artifact_records, rights_records, assets, asset_preparation_status, asset_transcription_status, asset_subtitle_status RESTART IDENTITY CASCADE",
        )
        .execute(&pool)
        .await
        .expect("truncate");
        Some(pool)
    }

    async fn insert_asset_for_test(pool: &PgPool) -> AssetId {
        let asset_id = AssetId::new();
        sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
            .bind(asset_id.0)
            .bind("test-asset")
            .bind(Uuid::new_v4())
            .bind("finalized")
            .execute(pool)
            .await
            .expect("insert asset");
        asset_id
    }

    async fn insert_project_with_target(
        pool: &PgPool,
        asset_id: AssetId,
        source_lang: &str,
        target_lang: &str,
    ) -> (ProjectId, OrgId) {
        let org_id = OrgId(Uuid::new_v4());
        let org = Organization {
            id: org_id,
            name: "test-org".into(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };
        workspace_repo::insert_org(pool, &org)
            .await
            .expect("insert org");

        let project_id = ProjectId(Uuid::new_v4());
        let project = Project {
            id: project_id,
            org_id,
            name: "test-project".into(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };
        workspace_repo::insert_project(pool, &project)
            .await
            .expect("insert project");

        sqlx::query(
            "INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .expect("link asset to project");

        workspace_repo::upsert_target_language(
            pool,
            &TargetLanguage {
                id: Uuid::new_v4(),
                project_id,
                source_lang: source_lang.into(),
                target_lang: target_lang.into(),
                created_at: OffsetDateTime::now_utc(),
            },
        )
        .await
        .expect("insert target language");

        (project_id, org_id)
    }

    async fn count_review_tasks(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM review_tasks")
            .fetch_one(pool)
            .await
            .expect("count review_tasks")
    }

    #[tokio::test]
    async fn prepare_review_post_ready_enqueues_review_task_with_correct_identity() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        let (project_id, org_id) = insert_project_with_target(&pool, asset_id, "en", "es").await;

        prepare_review_post_ready(&pool, asset_id, project_id, "es").await;

        let row: (Uuid, Uuid, Uuid) = sqlx::query_as(
            "SELECT org_id, project_id, asset_id FROM review_tasks WHERE asset_id = $1",
        )
        .bind(asset_id.0)
        .fetch_one(&pool)
        .await
        .expect("fetch review task row");
        assert_eq!(row.0, org_id.0);
        assert_eq!(row.1, project_id.0);
        assert_eq!(row.2, asset_id.0);
    }

    #[tokio::test]
    async fn prepare_review_post_ready_is_idempotent_on_repeated_call() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        let (project_id, _org_id) = insert_project_with_target(&pool, asset_id, "en", "es").await;

        prepare_review_post_ready(&pool, asset_id, project_id, "es").await;
        prepare_review_post_ready(&pool, asset_id, project_id, "es").await;

        assert_eq!(count_review_tasks(&pool).await, 1);
    }

    #[tokio::test]
    async fn prepare_review_post_ready_no_op_when_project_missing() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;

        prepare_review_post_ready(&pool, asset_id, ProjectId(Uuid::new_v4()), "es").await;

        assert_eq!(count_review_tasks(&pool).await, 0);
    }

    #[tokio::test]
    async fn prepare_review_post_ready_no_op_on_db_connection_failure() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        let (project_id, _org_id) = insert_project_with_target(&pool, asset_id, "en", "es").await;

        pool.close().await;

        // The pool is closed, so get_project fails with a connection-level DbError.
        // prepare_review_post_ready must swallow it, not panic, not enqueue.
        prepare_review_post_ready(&pool, asset_id, project_id, "es").await;
    }

    #[tokio::test]
    async fn prepare_review_post_ready_no_op_when_target_language_missing() {
        let Some(pool) = setup_pool_for_test().await else {
            return;
        };

        let asset_id = insert_asset_for_test(&pool).await;
        let (project_id, _org_id) = insert_project_with_target(&pool, asset_id, "en", "es").await;

        prepare_review_post_ready(&pool, asset_id, project_id, "fr").await;

        assert_eq!(count_review_tasks(&pool).await, 0);
    }
}
