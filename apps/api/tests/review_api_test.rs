use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::review_repo;
use dubbridge_domain::{
    asset::{Asset, IngestionStatus},
    review::{ReviewTask, ReviewTaskId},
    workspace::{OrgId, OrgMember, OrgRole, Organization, Project, TargetLanguage},
};
use dubbridge_storage::LocalFsAdapter;
use sqlx::PgPool;
use tempfile::TempDir;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone, Default)]
struct StubTokenVerifier {
    responses: HashMap<String, Result<AuthenticatedPrincipal, TokenVerificationError>>,
}

impl StubTokenVerifier {
    fn with_token(
        mut self,
        token: &str,
        result: Result<AuthenticatedPrincipal, TokenVerificationError>,
    ) -> Self {
        self.responses.insert(token.to_string(), result);
        self
    }
}

impl TokenVerifier for StubTokenVerifier {
    fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<AuthenticatedPrincipal, TokenVerificationError> {
        self.responses
            .get(token)
            .cloned()
            .unwrap_or(Err(TokenVerificationError::MalformedToken))
    }
}

struct TestContext {
    pool: PgPool,
    app: axum::Router,
    reviewer_id: Uuid,
    viewer_id: Uuid,
    outsider_id: Uuid,
    read_token: String,
    write_token: String,
    viewer_write_token: String,
}

struct ReviewFixture {
    org: Organization,
    project: Project,
    task_id: ReviewTaskId,
}

impl TestContext {
    async fn new() -> Option<Self> {
        let database_url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect database");
        migrate_and_reset(&pool).await;

        let storage_dir = Arc::new(TempDir::new().expect("temp dir"));
        let storage_path = PathBuf::from(storage_dir.path());

        let reviewer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").expect("uuid");
        let viewer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440011").expect("uuid");
        let outsider_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440012").expect("uuid");

        let read_token = "review-read-token".to_string();
        let write_token = "review-write-token".to_string();
        let viewer_write_token = "review-viewer-write-token".to_string();

        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default()
                .with_token(
                    &read_token,
                    Ok(AuthenticatedPrincipal::new(
                        reviewer_id,
                        ["workspaces:read"].into_iter().map(str::to_string),
                    )),
                )
                .with_token(
                    &write_token,
                    Ok(AuthenticatedPrincipal::new(
                        reviewer_id,
                        ["workspaces:read", "workspaces:write"]
                            .into_iter()
                            .map(str::to_string),
                    )),
                )
                .with_token(
                    &viewer_write_token,
                    Ok(AuthenticatedPrincipal::new(
                        viewer_id,
                        ["workspaces:read", "workspaces:write"]
                            .into_iter()
                            .map(str::to_string),
                    )),
                ),
        );

        let config = dubbridge_config::AppConfig::from_env();
        let state = Arc::new(AppState::new(
            pool.clone(),
            Box::new(LocalFsAdapter::new(&storage_path)),
            verifier.clone(),
            config,
        ));

        Some(Self {
            pool,
            app: build_app(state, verifier),
            reviewer_id,
            viewer_id,
            outsider_id,
            read_token,
            write_token,
            viewer_write_token,
        })
    }
}

#[tokio::test]
async fn list_review_queue_returns_scoped_tasks() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;
    let other_org = insert_review_fixture(&ctx.pool, ctx.outsider_id, OrgRole::Reviewer).await;

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/orgs/{}/projects/{}/review-tasks",
            fixture.org.id.0, fixture.project.id.0
        ),
        Some(&ctx.read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["tasks"].as_array().expect("tasks").len(), 1);
    assert_eq!(body["tasks"][0]["id"], fixture.task_id.0.to_string());
    assert_ne!(body["tasks"][0]["id"], other_org.task_id.0.to_string());
}

#[tokio::test]
async fn decide_requires_reviewer_role() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;
    insert_membership(&ctx.pool, fixture.org.id, ctx.viewer_id, OrgRole::Viewer).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.viewer_write_token,
        r#"{"verdict":"approved","comment":"looks good"}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        review_repo::list_review_decisions_for_task(&ctx.pool, fixture.task_id)
            .await
            .expect("decisions")
            .len(),
        0
    );
}

#[tokio::test]
async fn decide_requires_workspace_write_scope() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.read_token,
        r#"{"verdict":"approved","comment":"looks good"}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        review_repo::list_review_decisions_for_task(&ctx.pool, fixture.task_id)
            .await
            .expect("decisions")
            .len(),
        0
    );
}

#[tokio::test]
async fn approve_decision_via_api_returns_approved_state() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.write_token,
        r#"{"verdict":"approved","comment":"ship it"}"#,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "approved");
    assert_eq!(
        review_repo::latest_review_state(&ctx.pool, fixture.task_id)
            .await
            .expect("latest state")
            .to_string(),
        "approved"
    );
}

#[tokio::test]
async fn reject_decision_via_api_returns_rejected_state() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.write_token,
        r#"{"verdict":"rejected","comment":"needs changes"}"#,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["state"], "rejected");
    assert_eq!(
        review_repo::latest_review_state(&ctx.pool, fixture.task_id)
            .await
            .expect("latest state")
            .to_string(),
        "rejected"
    );
}

#[tokio::test]
async fn publish_non_approved_task_returns_conflict() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/publish",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        Some(&ctx.write_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body["error"],
        "review approval is required before publication"
    );
}

#[tokio::test]
async fn publish_approved_task_creates_publication() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;
    review_repo::append_review_decision(
        &ctx.pool,
        &dubbridge_domain::review::ReviewDecisionRow {
            id: Uuid::new_v4(),
            review_task_id: fixture.task_id,
            verdict: dubbridge_domain::review::ReviewVerdict::Approved,
            comment: Some("approved".to_string()),
            reviewer_subject_id: ctx.reviewer_id,
            happened_at: OffsetDateTime::now_utc(),
        },
    )
    .await
    .expect("append approved decision");

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/publish",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        Some(&ctx.write_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "published");
    assert_eq!(body["review_task_id"], fixture.task_id.0.to_string());
}

#[tokio::test]
async fn queue_rejects_cross_org_project_traversal() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id, OrgRole::Reviewer).await;
    let foreign = insert_review_fixture(&ctx.pool, ctx.outsider_id, OrgRole::Reviewer).await;

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/orgs/{}/projects/{}/review-tasks",
            fixture.org.id.0, foreign.project.id.0
        ),
        Some(&ctx.read_token),
        None,
        Body::empty(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

async fn insert_review_fixture(pool: &PgPool, subject_id: Uuid, role: OrgRole) -> ReviewFixture {
    let org = Organization::new(format!("Org-{subject_id}"));
    insert_org(pool, &org).await;
    insert_membership(pool, org.id, subject_id, role).await;

    let project = Project::new(org.id, format!("Project-{subject_id}"));
    insert_project(pool, &project).await;

    let mut asset = Asset::new_pending(format!("asset-{subject_id}"), subject_id);
    asset.status = IngestionStatus::Finalized;
    asset_repo_insert(pool, &asset).await;
    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project.id.0)
        .bind(asset.id.0)
        .execute(pool)
        .await
        .expect("project asset");

    let target_language = TargetLanguage::new(project.id, "en".to_string(), "es".to_string());
    sqlx::query(
        "INSERT INTO target_languages (id, project_id, source_lang, target_lang) VALUES ($1, $2, $3, $4)",
    )
    .bind(target_language.id)
    .bind(project.id.0)
    .bind(&target_language.source_lang)
    .bind(&target_language.target_lang)
    .execute(pool)
    .await
    .expect("target language");

    let now = OffsetDateTime::now_utc();
    let task = ReviewTask {
        id: ReviewTaskId::new(),
        org_id: org.id,
        project_id: project.id,
        asset_id: asset.id,
        target_language_id: target_language.id,
        assignee_subject_id: Some(subject_id),
        created_at: now,
        updated_at: now,
        assigned_at: Some(now),
    };
    review_repo::insert_review_task(pool, &task)
        .await
        .expect("insert review task");

    ReviewFixture {
        org,
        project,
        task_id: task.id,
    }
}

async fn migrate_and_reset(pool: &PgPool) {
    sqlx::migrate!("../../infra/migrations")
        .run(pool)
        .await
        .expect("migrations");

    sqlx::query(
        "TRUNCATE TABLE publications, review_decisions, review_tasks, target_languages, project_assets, projects, org_members, organizations, pending_ingestions, audit_events, artifact_records, rights_records, assets RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("truncate tables");
}

async fn insert_org(pool: &PgPool, org: &Organization) {
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org.id.0)
        .bind(&org.name)
        .execute(pool)
        .await
        .expect("insert org");
}

async fn insert_project(pool: &PgPool, project: &Project) {
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project.id.0)
        .bind(project.org_id.0)
        .bind(&project.name)
        .execute(pool)
        .await
        .expect("insert project");
}

async fn insert_membership(pool: &PgPool, org_id: OrgId, subject_id: Uuid, role: OrgRole) {
    let member = OrgMember::new(org_id, subject_id, role);
    sqlx::query("INSERT INTO org_members (org_id, subject_id, role) VALUES ($1, $2, $3)")
        .bind(member.org_id.0)
        .bind(member.subject_id)
        .bind(member.role.to_string())
        .execute(pool)
        .await
        .expect("insert membership");
}

async fn asset_repo_insert(pool: &PgPool, asset: &Asset) {
    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset.id.0)
        .bind(&asset.title)
        .bind(asset.uploader_id)
        .bind(asset.status.to_string())
        .execute(pool)
        .await
        .expect("insert asset");
}

async fn send_json(
    app: &axum::Router,
    method: Method,
    uri: &str,
    token: &str,
    body: &str,
) -> axum::response::Response {
    send_request(
        app,
        method,
        uri,
        Some(token),
        Some(("application/json", body)),
        Body::from(body.to_string()),
    )
    .await
}

async fn send_request(
    app: &axum::Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    content_type: Option<(&str, &str)>,
    body: Body,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if let Some((kind, _)) = content_type {
        builder = builder.header(header::CONTENT_TYPE, kind);
    }
    app.clone()
        .oneshot(builder.body(body).expect("request"))
        .await
        .expect("response")
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("body"),
    )
    .expect("json body")
}
