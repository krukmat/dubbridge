use std::env;

use dubbridge_db::review_repo;
use dubbridge_domain::{
    asset::AssetId,
    review::{
        PublicationRow, PublicationStatus, ReviewDecisionRow, ReviewTask, ReviewTaskId,
        ReviewTaskState, ReviewVerdict,
    },
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
    other_project_id: ProjectId,
    asset_id: AssetId,
    other_asset_id: AssetId,
    target_language_id: Uuid,
    reviewer_subject_id: Uuid,
    other_reviewer_subject_id: Uuid,
}

async fn insert_review_scope(pool: &PgPool) -> ReviewScope {
    let org_id = OrgId(Uuid::new_v4());
    let project_id = ProjectId(Uuid::new_v4());
    let other_project_id = ProjectId(Uuid::new_v4());
    let asset_id = AssetId(Uuid::new_v4());
    let other_asset_id = AssetId(Uuid::new_v4());
    let target_language_id = Uuid::new_v4();
    let reviewer_subject_id = Uuid::new_v4();
    let other_reviewer_subject_id = Uuid::new_v4();

    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id.0)
        .bind("Repo Review Org")
        .execute(pool)
        .await
        .expect("insert org");

    for subject_id in [reviewer_subject_id, other_reviewer_subject_id] {
        sqlx::query("INSERT INTO org_members (org_id, subject_id, role) VALUES ($1, $2, $3)")
            .bind(org_id.0)
            .bind(subject_id)
            .bind("reviewer")
            .execute(pool)
            .await
            .expect("insert org member");
    }

    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project_id.0)
        .bind(org_id.0)
        .bind("Repo Project")
        .execute(pool)
        .await
        .expect("insert project");

    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(other_project_id.0)
        .bind(org_id.0)
        .bind("Other Repo Project")
        .execute(pool)
        .await
        .expect("insert second project");

    for (asset, title) in [
        (asset_id.0, "repo-review-asset"),
        (other_asset_id.0, "repo-review-other-asset"),
    ] {
        sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
            .bind(asset)
            .bind(title)
            .bind(Uuid::new_v4())
            .bind("finalized")
            .execute(pool)
            .await
            .expect("insert asset");
    }

    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project_id.0)
        .bind(asset_id.0)
        .execute(pool)
        .await
        .expect("insert project asset");

    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(other_project_id.0)
        .bind(other_asset_id.0)
        .execute(pool)
        .await
        .expect("insert second project asset");

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
        other_project_id,
        asset_id,
        other_asset_id,
        target_language_id,
        reviewer_subject_id,
        other_reviewer_subject_id,
    }
}

fn new_task(scope: &ReviewScope, asset_id: AssetId, assignee_subject_id: Uuid) -> ReviewTask {
    let now = OffsetDateTime::now_utc();
    ReviewTask {
        id: ReviewTaskId::new(),
        org_id: scope.org_id,
        project_id: scope.project_id,
        asset_id,
        target_language_id: scope.target_language_id,
        assignee_subject_id: Some(assignee_subject_id),
        created_at: now,
        updated_at: now,
        assigned_at: Some(now),
    }
}

fn approve_decision(task_id: ReviewTaskId, reviewer_subject_id: Uuid) -> ReviewDecisionRow {
    ReviewDecisionRow {
        id: Uuid::new_v4(),
        review_task_id: task_id,
        verdict: ReviewVerdict::Approved,
        comment: Some("approved".to_string()),
        reviewer_subject_id,
        happened_at: OffsetDateTime::now_utc(),
    }
}

fn reject_decision(task_id: ReviewTaskId, reviewer_subject_id: Uuid) -> ReviewDecisionRow {
    ReviewDecisionRow {
        id: Uuid::new_v4(),
        review_task_id: task_id,
        verdict: ReviewVerdict::Rejected,
        comment: Some("rejected".to_string()),
        reviewer_subject_id,
        happened_at: OffsetDateTime::now_utc(),
    }
}

#[tokio::test]
async fn insert_and_scope_list_review_tasks_returns_pending_item() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let task = new_task(&scope, scope.asset_id, scope.reviewer_subject_id);
    review_repo::insert_review_task(&pool, &task)
        .await
        .expect("insert review task");

    let items = review_repo::list_review_tasks_for_scope(
        &pool,
        scope.org_id,
        scope.project_id,
        Some(scope.reviewer_subject_id),
    )
    .await
    .expect("list review tasks");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].task.id, task.id);
    assert_eq!(items[0].state, ReviewTaskState::Pending);
}

#[tokio::test]
async fn latest_review_state_uses_latest_append_only_decision() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let task = new_task(&scope, scope.asset_id, scope.reviewer_subject_id);
    review_repo::insert_review_task(&pool, &task)
        .await
        .expect("insert review task");

    review_repo::append_review_decision(
        &pool,
        &approve_decision(task.id, scope.reviewer_subject_id),
    )
    .await
    .expect("append approve");
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    review_repo::append_review_decision(
        &pool,
        &reject_decision(task.id, scope.other_reviewer_subject_id),
    )
    .await
    .expect("append reject");

    let state = review_repo::latest_review_state(&pool, task.id)
        .await
        .expect("latest state");
    let rows = review_repo::list_review_decisions_for_task(&pool, task.id)
        .await
        .expect("decision list");

    assert_eq!(rows.len(), 2);
    assert_eq!(state, ReviewTaskState::Rejected);
    assert_eq!(rows[0].verdict, ReviewVerdict::Approved);
    assert_eq!(rows[1].verdict, ReviewVerdict::Rejected);
}

#[tokio::test]
async fn approve_decision_round_trips_to_approved_state() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let task = new_task(&scope, scope.asset_id, scope.reviewer_subject_id);
    review_repo::insert_review_task(&pool, &task)
        .await
        .expect("insert review task");

    review_repo::append_review_decision(
        &pool,
        &approve_decision(task.id, scope.reviewer_subject_id),
    )
    .await
    .expect("append approve");

    let state = review_repo::latest_review_state(&pool, task.id)
        .await
        .expect("latest state");
    let latest = review_repo::latest_review_decision(&pool, task.id)
        .await
        .expect("latest decision")
        .expect("decision row");

    assert_eq!(state, ReviewTaskState::Approved);
    assert_eq!(latest.verdict, ReviewVerdict::Approved);
}

#[tokio::test]
async fn scoped_queue_filters_out_other_projects_and_assignees() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let primary = new_task(&scope, scope.asset_id, scope.reviewer_subject_id);
    review_repo::insert_review_task(&pool, &primary)
        .await
        .expect("insert primary task");

    let other_now = OffsetDateTime::now_utc();
    let other_project_task = ReviewTask {
        id: ReviewTaskId::new(),
        org_id: scope.org_id,
        project_id: scope.other_project_id,
        asset_id: scope.other_asset_id,
        target_language_id: Uuid::new_v4(),
        assignee_subject_id: Some(scope.reviewer_subject_id),
        created_at: other_now,
        updated_at: other_now,
        assigned_at: Some(other_now),
    };

    // Seed target language required by the other project before insert.
    sqlx::query(
        "INSERT INTO target_languages (id, project_id, source_lang, target_lang) VALUES ($1, $2, $3, $4)",
    )
    .bind(other_project_task.target_language_id)
    .bind(scope.other_project_id.0)
    .bind("en")
    .bind("fr")
    .execute(&pool)
    .await
    .expect("insert other target language");

    review_repo::insert_review_task(&pool, &other_project_task)
        .await
        .expect("insert other project task");

    let other_assignee = new_task(&scope, scope.asset_id, scope.other_reviewer_subject_id);
    let err = review_repo::insert_review_task(&pool, &other_assignee).await;
    assert!(err.is_err(), "expected unique review unit constraint");

    let items = review_repo::list_review_tasks_for_scope(
        &pool,
        scope.org_id,
        scope.project_id,
        Some(scope.reviewer_subject_id),
    )
    .await
    .expect("list scoped tasks");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].task.id, primary.id);
    assert_eq!(items[0].task.project_id, scope.project_id);
    assert_eq!(
        items[0].task.assignee_subject_id,
        Some(scope.reviewer_subject_id)
    );
}

#[tokio::test]
async fn insert_and_get_publication_round_trips() {
    let Some(pool) = setup_pool().await else {
        eprintln!("skipping: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let scope = insert_review_scope(&pool).await;
    let task = new_task(&scope, scope.asset_id, scope.reviewer_subject_id);
    review_repo::insert_review_task(&pool, &task)
        .await
        .expect("insert review task");
    review_repo::append_review_decision(
        &pool,
        &approve_decision(task.id, scope.reviewer_subject_id),
    )
    .await
    .expect("append approve");

    let publication = PublicationRow {
        id: Uuid::new_v4(),
        review_task_id: task.id,
        status: PublicationStatus::Published,
        published_by: scope.reviewer_subject_id,
        published_at: OffsetDateTime::now_utc(),
    };

    review_repo::insert_publication(&pool, &publication)
        .await
        .expect("insert publication");

    let fetched = review_repo::get_publication_for_review_task(&pool, task.id)
        .await
        .expect("get publication")
        .expect("publication row");

    assert_eq!(fetched.id, publication.id);
    assert_eq!(fetched.review_task_id, task.id);
    assert_eq!(fetched.status, PublicationStatus::Published);
    assert_eq!(fetched.published_by, scope.reviewer_subject_id);
}
