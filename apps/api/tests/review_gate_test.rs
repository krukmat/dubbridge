use std::env;

use dubbridge_api::review_gate::{
    ReviewGateError, approve_review_task, publish_review_task, reject_review_task,
};
use dubbridge_db::review_repo;
use dubbridge_domain::{
    asset::AssetId,
    review::{ReviewTask, ReviewTaskId, ReviewTaskState},
    workspace::{OrgId, ProjectId},
};
use sqlx::PgPool;
use time::OffsetDateTime;
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
    org_id: OrgId,
    project_id: ProjectId,
    asset_id: AssetId,
    target_language_id: Uuid,
    reviewer_subject_id: Uuid,
}

async fn insert_review_scope(pool: &PgPool) -> ReviewScope {
    let org_id = OrgId(Uuid::new_v4());
    let project_id = ProjectId(Uuid::new_v4());
    let asset_id = AssetId(Uuid::new_v4());
    let target_language_id = Uuid::new_v4();
    let reviewer_subject_id = Uuid::new_v4();

    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id.0)
        .bind("Review Gate Org")
        .execute(pool)
        .await
        .expect("insert org");

    sqlx::query("INSERT INTO org_members (org_id, subject_id, role) VALUES ($1, $2, $3)")
        .bind(org_id.0)
        .bind(reviewer_subject_id)
        .bind("reviewer")
        .execute(pool)
        .await
        .expect("insert member");

    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project_id.0)
        .bind(org_id.0)
        .bind("Review Gate Project")
        .execute(pool)
        .await
        .expect("insert project");

    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset_id.0)
        .bind("review-gate-asset")
        .bind(Uuid::new_v4())
        .bind("finalized")
        .execute(pool)
        .await
        .expect("insert asset");

    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .expect("insert project asset");

    sqlx::query(
        "INSERT INTO target_languages (id, project_id, source_lang, target_lang) VALUES ($1, $2, $3, $4)",
    )
    .bind(target_language_id)
    .bind(project_id.0)
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
        reviewer_subject_id,
    }
}

async fn insert_review_task(pool: &PgPool, scope: &ReviewScope) -> ReviewTaskId {
    let now = OffsetDateTime::now_utc();
    let task = ReviewTask {
        id: ReviewTaskId::new(),
        org_id: scope.org_id,
        project_id: scope.project_id,
        asset_id: scope.asset_id,
        target_language_id: scope.target_language_id,
        assignee_subject_id: Some(scope.reviewer_subject_id),
        created_at: now,
        updated_at: now,
        assigned_at: Some(now),
    };

    review_repo::insert_review_task(pool, &task)
        .await
        .expect("insert review task");
    task.id
}

async fn count_audit_events_by_kind(pool: &PgPool, asset_id: AssetId, event_kind: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE asset_id = $1 AND event_kind = $2")
        .bind(asset_id.0)
        .bind(event_kind)
        .fetch_one(pool)
        .await
        .expect("count audit events")
}

async fn latest_audit_detail(pool: &PgPool, asset_id: AssetId, event_kind: &str) -> Option<String> {
    sqlx::query_scalar(
        "SELECT detail FROM audit_events WHERE asset_id = $1 AND event_kind = $2 ORDER BY happened_at DESC, id DESC LIMIT 1",
    )
    .bind(asset_id.0)
    .bind(event_kind)
    .fetch_optional(pool)
    .await
    .expect("latest audit detail")
}

#[tokio::test]
async fn approve_review_task_appends_decision_and_returns_approved() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = insert_review_scope(&pool).await;
    let task_id = insert_review_task(&pool, &scope).await;

    let state = approve_review_task(
        &pool,
        task_id,
        scope.reviewer_subject_id,
        Some("ship it".to_string()),
    )
    .await
    .expect("approve task");

    let rows = review_repo::list_review_decisions_for_task(&pool, task_id)
        .await
        .expect("decision rows");
    assert_eq!(state, ReviewTaskState::Approved);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        count_audit_events_by_kind(&pool, scope.asset_id, "review_approved").await,
        1
    );
    let detail = latest_audit_detail(&pool, scope.asset_id, "review_approved")
        .await
        .expect("review approved detail");
    assert!(detail.contains(&task_id.0.to_string()));
}

#[tokio::test]
async fn reject_review_task_appends_decision_and_returns_rejected() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = insert_review_scope(&pool).await;
    let task_id = insert_review_task(&pool, &scope).await;

    let state = reject_review_task(
        &pool,
        task_id,
        scope.reviewer_subject_id,
        Some("not ready".to_string()),
    )
    .await
    .expect("reject task");

    let rows = review_repo::list_review_decisions_for_task(&pool, task_id)
        .await
        .expect("decision rows");
    assert_eq!(state, ReviewTaskState::Rejected);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        count_audit_events_by_kind(&pool, scope.asset_id, "review_rejected").await,
        1
    );
    let detail = latest_audit_detail(&pool, scope.asset_id, "review_rejected")
        .await
        .expect("review rejected detail");
    assert!(detail.contains(&task_id.0.to_string()));
}

#[tokio::test]
async fn publish_review_task_refuses_pending_task() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = insert_review_scope(&pool).await;
    let task_id = insert_review_task(&pool, &scope).await;

    let err = publish_review_task(&pool, task_id, scope.reviewer_subject_id)
        .await
        .expect_err("pending task must be refused");

    assert!(matches!(
        err,
        ReviewGateError::ReviewNotApproved {
            state: ReviewTaskState::Pending,
            ..
        }
    ));
    let publication = review_repo::get_publication_for_review_task(&pool, task_id)
        .await
        .expect("publication lookup");
    assert!(publication.is_none());
    assert_eq!(
        count_audit_events_by_kind(&pool, scope.asset_id, "publication_refused").await,
        1
    );
    let detail = latest_audit_detail(&pool, scope.asset_id, "publication_refused")
        .await
        .expect("publication refused detail");
    assert!(detail.contains("current_state=pending"));
}

#[tokio::test]
async fn publish_review_task_creates_publication_when_approved() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = insert_review_scope(&pool).await;
    let task_id = insert_review_task(&pool, &scope).await;

    approve_review_task(
        &pool,
        task_id,
        scope.reviewer_subject_id,
        Some("approved".to_string()),
    )
    .await
    .expect("approve task");

    let publication = publish_review_task(&pool, task_id, scope.reviewer_subject_id)
        .await
        .expect("publish task");

    let stored = review_repo::get_publication_for_review_task(&pool, task_id)
        .await
        .expect("stored publication")
        .expect("publication row");

    assert_eq!(publication.review_task_id, task_id);
    assert_eq!(stored.id, publication.id);
    assert_eq!(
        count_audit_events_by_kind(&pool, scope.asset_id, "publication_succeeded").await,
        1
    );
    let detail = latest_audit_detail(&pool, scope.asset_id, "publication_succeeded")
        .await
        .expect("publication succeeded detail");
    assert!(detail.contains("publication_state=published"));
}

#[tokio::test]
async fn publish_review_task_refuses_rejected_task() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = insert_review_scope(&pool).await;
    let task_id = insert_review_task(&pool, &scope).await;

    reject_review_task(
        &pool,
        task_id,
        scope.reviewer_subject_id,
        Some("needs changes".to_string()),
    )
    .await
    .expect("reject task");

    let err = publish_review_task(&pool, task_id, scope.reviewer_subject_id)
        .await
        .expect_err("rejected task must be refused");

    assert!(matches!(
        err,
        ReviewGateError::ReviewNotApproved {
            state: ReviewTaskState::Rejected,
            ..
        }
    ));
    let publication = review_repo::get_publication_for_review_task(&pool, task_id)
        .await
        .expect("publication lookup");
    assert!(publication.is_none());
    assert_eq!(
        count_audit_events_by_kind(&pool, scope.asset_id, "publication_refused").await,
        1
    );
    let detail = latest_audit_detail(&pool, scope.asset_id, "publication_refused")
        .await
        .expect("publication refused detail");
    assert!(detail.contains("current_state=rejected"));
}

#[tokio::test]
async fn publish_review_task_refuses_duplicate_publication() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };
    let scope = insert_review_scope(&pool).await;
    let task_id = insert_review_task(&pool, &scope).await;

    approve_review_task(
        &pool,
        task_id,
        scope.reviewer_subject_id,
        Some("approved".to_string()),
    )
    .await
    .expect("approve task");

    publish_review_task(&pool, task_id, scope.reviewer_subject_id)
        .await
        .expect("first publish");

    let err = publish_review_task(&pool, task_id, scope.reviewer_subject_id)
        .await
        .expect_err("second publish must fail");

    assert!(matches!(err, ReviewGateError::AlreadyPublished { .. }));
    assert_eq!(
        count_audit_events_by_kind(&pool, scope.asset_id, "publication_refused").await,
        1
    );
    let detail = latest_audit_detail(&pool, scope.asset_id, "publication_refused")
        .await
        .expect("publication refused detail");
    assert!(detail.contains("reason=already_published"));
}
