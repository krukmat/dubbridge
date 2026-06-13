use std::env;

use sqlx::PgPool;
use uuid::Uuid;

async fn setup_pool() -> Option<PgPool> {
    let url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("../../infra/migrations")
        .run(&pool)
        .await
        .expect("migrations");
    Some(pool)
}

struct ReviewScope {
    org_id: Uuid,
    project_id: Uuid,
    asset_id: Uuid,
    target_language_id: Uuid,
    assignee_subject_id: Uuid,
}

async fn insert_review_scope(pool: &PgPool) -> ReviewScope {
    let org_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let asset_id = Uuid::new_v4();
    let target_language_id = Uuid::new_v4();
    let assignee_subject_id = Uuid::new_v4();

    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind("Review Org")
        .execute(pool)
        .await
        .expect("insert org");

    sqlx::query("INSERT INTO org_members (org_id, subject_id, role) VALUES ($1, $2, $3)")
        .bind(org_id)
        .bind(assignee_subject_id)
        .bind("reviewer")
        .execute(pool)
        .await
        .expect("insert org member");

    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project_id)
        .bind(org_id)
        .bind("Review Project")
        .execute(pool)
        .await
        .expect("insert project");

    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset_id)
        .bind("review-asset")
        .bind(Uuid::new_v4())
        .bind("finalized")
        .execute(pool)
        .await
        .expect("insert asset");

    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project_id)
        .bind(asset_id)
        .execute(pool)
        .await
        .expect("insert project asset");

    sqlx::query(
        "INSERT INTO target_languages (id, project_id, source_lang, target_lang) VALUES ($1, $2, $3, $4)",
    )
    .bind(target_language_id)
    .bind(project_id)
    .bind("en")
    .bind("es")
    .execute(pool)
    .await
    .expect("insert target language");

    ReviewScope {
        org_id,
        project_id,
        asset_id,
        target_language_id,
        assignee_subject_id,
    }
}

async fn insert_review_task(pool: &PgPool, scope: &ReviewScope) -> Uuid {
    let review_task_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO review_tasks (
            id, org_id, project_id, asset_id, target_language_id, assignee_subject_id
        ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(review_task_id)
    .bind(scope.org_id)
    .bind(scope.project_id)
    .bind(scope.asset_id)
    .bind(scope.target_language_id)
    .bind(scope.assignee_subject_id)
    .execute(pool)
    .await
    .expect("insert review task");
    review_task_id
}

#[tokio::test]
async fn review_task_accepts_valid_project_asset_target_scope() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let review_task_id = insert_review_task(&pool, &scope).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM review_tasks WHERE id = $1")
        .bind(review_task_id)
        .fetch_one(&pool)
        .await
        .expect("review task count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn review_decisions_are_append_only_noops_for_update_and_delete() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let review_task_id = insert_review_task(&pool, &scope).await;
    let decision_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO review_decisions (id, review_task_id, verdict, comment, reviewer_subject_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(decision_id)
    .bind(review_task_id)
    .bind("approved")
    .bind("looks good")
    .bind(scope.assignee_subject_id)
    .execute(&pool)
    .await
    .expect("insert review decision");

    let update =
        sqlx::query("UPDATE review_decisions SET verdict = $1, comment = $2 WHERE id = $3")
            .bind("rejected")
            .bind("changed")
            .bind(decision_id)
            .execute(&pool)
            .await
            .expect("update no-op");
    assert_eq!(update.rows_affected(), 0);

    let delete = sqlx::query("DELETE FROM review_decisions WHERE id = $1")
        .bind(decision_id)
        .execute(&pool)
        .await
        .expect("delete no-op");
    assert_eq!(delete.rows_affected(), 0);

    let verdict: String = sqlx::query_scalar("SELECT verdict FROM review_decisions WHERE id = $1")
        .bind(decision_id)
        .fetch_one(&pool)
        .await
        .expect("stored verdict");
    assert_eq!(verdict, "approved");
}

#[tokio::test]
async fn review_decision_rejects_unknown_verdict() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let review_task_id = insert_review_task(&pool, &scope).await;

    let err = sqlx::query(
        "INSERT INTO review_decisions (id, review_task_id, verdict, comment, reviewer_subject_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(review_task_id)
    .bind("pending")
    .bind("invalid verdict")
    .bind(scope.assignee_subject_id)
    .execute(&pool)
    .await
    .expect_err("verdict check should reject unknown value");

    let message = err.to_string();
    assert!(
        message.contains("review_decisions_verdict_check"),
        "expected verdict check error, got {message}"
    );
}

#[tokio::test]
async fn review_task_rejects_target_language_from_another_project() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let other_project_id = Uuid::new_v4();
    let other_target_language_id = Uuid::new_v4();

    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(other_project_id)
        .bind(scope.org_id)
        .bind("Other Project")
        .execute(&pool)
        .await
        .expect("insert second project");

    sqlx::query(
        "INSERT INTO target_languages (id, project_id, source_lang, target_lang) VALUES ($1, $2, $3, $4)",
    )
    .bind(other_target_language_id)
    .bind(other_project_id)
    .bind("en")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("insert second target language");

    let err = sqlx::query(
        "INSERT INTO review_tasks (
            id, org_id, project_id, asset_id, target_language_id, assignee_subject_id
        ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(scope.org_id)
    .bind(scope.project_id)
    .bind(scope.asset_id)
    .bind(other_target_language_id)
    .bind(scope.assignee_subject_id)
    .execute(&pool)
    .await
    .expect_err("target language should be project-scoped");

    let message = err.to_string();
    assert!(
        message.contains("review_tasks_target_language_scope_fk"),
        "expected target-language scope FK error, got {message}"
    );
}

#[tokio::test]
async fn review_task_rejects_asset_not_linked_to_project() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let foreign_asset_id = Uuid::new_v4();

    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(foreign_asset_id)
        .bind("foreign-review-asset")
        .bind(Uuid::new_v4())
        .bind("finalized")
        .execute(&pool)
        .await
        .expect("insert foreign asset");

    let err = sqlx::query(
        "INSERT INTO review_tasks (
            id, org_id, project_id, asset_id, target_language_id, assignee_subject_id
        ) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(scope.org_id)
    .bind(scope.project_id)
    .bind(foreign_asset_id)
    .bind(scope.target_language_id)
    .bind(scope.assignee_subject_id)
    .execute(&pool)
    .await
    .expect_err("asset should be linked to the same project");

    let message = err.to_string();
    assert!(
        message.contains("review_tasks_project_asset_fk"),
        "expected project-asset FK error, got {message}"
    );
}

#[tokio::test]
async fn publications_reject_duplicate_review_task_rows() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let review_task_id = insert_review_task(&pool, &scope).await;

    sqlx::query(
        "INSERT INTO review_decisions (id, review_task_id, verdict, comment, reviewer_subject_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(review_task_id)
    .bind("approved")
    .bind("approved for publish")
    .bind(scope.assignee_subject_id)
    .execute(&pool)
    .await
    .expect("insert approved decision");

    sqlx::query(
        "INSERT INTO publications (id, review_task_id, state, published_by)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(review_task_id)
    .bind("published")
    .bind(scope.assignee_subject_id)
    .execute(&pool)
    .await
    .expect("insert publication");

    let err = sqlx::query(
        "INSERT INTO publications (id, review_task_id, state, published_by)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(review_task_id)
    .bind("published")
    .bind(scope.assignee_subject_id)
    .execute(&pool)
    .await
    .expect_err("duplicate publication should fail");

    let message = err.to_string();
    assert!(
        message.contains("publications_review_task_id_key"),
        "expected unique constraint error, got {message}"
    );
}
