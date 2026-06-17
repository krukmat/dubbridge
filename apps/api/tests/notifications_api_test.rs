use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::{notification_repo, review_repo};
use dubbridge_domain::{
    asset::{Asset, IngestionStatus},
    review::{ReviewDecisionRow, ReviewTask, ReviewTaskId, ReviewVerdict},
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
    other_id: Uuid,
    read_token: String,
    write_token: String,
    other_read_token: String,
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

        let reviewer_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440020").expect("uuid");
        let other_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440021").expect("uuid");

        let read_token = "notif-read-token".to_string();
        let write_token = "notif-write-token".to_string();
        let other_read_token = "notif-other-read-token".to_string();

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
                    &other_read_token,
                    Ok(AuthenticatedPrincipal::new(
                        other_id,
                        ["workspaces:read"].into_iter().map(str::to_string),
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
            other_id,
            read_token,
            write_token,
            other_read_token,
        })
    }
}

// HP-1: approve via gate → notification emitted → list shows one unread row
#[tokio::test]
async fn approve_decision_emits_notification_visible_to_reviewer() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.write_token,
        r#"{"verdict":"approved","comment":"looks good"}"#,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let rows = notification_repo::list_notifications_for_recipient(&ctx.pool, ctx.reviewer_id)
        .await
        .expect("list notifications");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind.to_string(), "review_task_decided");
    assert_eq!(rows[0].ref_entity_id, fixture.task_id.0);
    assert!(rows[0].read_at.is_none());
}

// HP-1 (reject variant): reject via gate → notification emitted
#[tokio::test]
async fn reject_decision_emits_notification() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;

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
    assert_eq!(response.status(), StatusCode::OK);

    let rows = notification_repo::list_notifications_for_recipient(&ctx.pool, ctx.reviewer_id)
        .await
        .expect("list notifications");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind.to_string(), "review_task_decided");
}

// HP-1 (publish variant): successful publish → notification emitted with kind review_task_published
#[tokio::test]
async fn publish_success_emits_notification() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;
    append_approved_decision(&ctx.pool, fixture.task_id, ctx.reviewer_id).await;

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
    assert_eq!(response.status(), StatusCode::CREATED);

    let rows = notification_repo::list_notifications_for_recipient(&ctx.pool, ctx.reviewer_id)
        .await
        .expect("list notifications");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].kind.to_string(), "review_task_published");
    assert_eq!(rows[0].ref_entity_id, fixture.task_id.0);
}

// HP-2: GET /notifications returns caller's rows in reverse-chronological order
#[tokio::test]
async fn list_notifications_returns_callers_rows() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;
    let fixture2 = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;

    // Approve both → 2 notifications for reviewer
    send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.write_token,
        r#"{"verdict":"approved","comment":"ok"}"#,
    )
    .await;
    send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture2.org.id.0, fixture2.project.id.0, fixture2.task_id.0
        ),
        &ctx.write_token,
        r#"{"verdict":"approved","comment":"ok"}"#,
    )
    .await;

    let response = send_request(
        &ctx.app,
        Method::GET,
        "/notifications",
        Some(&ctx.read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    let notifications = body["notifications"].as_array().expect("notifications");
    assert_eq!(notifications.len(), 2);
    for n in notifications {
        assert_eq!(n["kind"], "review_task_decided");
    }
}

// HP-3: POST /notifications/mark-read sets read_at; re-list shows them read
#[tokio::test]
async fn mark_notifications_read_sets_read_at() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;
    send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.write_token,
        r#"{"verdict":"approved","comment":"ok"}"#,
    )
    .await;

    let rows = notification_repo::list_notifications_for_recipient(&ctx.pool, ctx.reviewer_id)
        .await
        .expect("list");
    let notification_id = rows[0].id;

    let body = format!(r#"{{"ids":["{notification_id}"]}}"#);
    let response = send_json(
        &ctx.app,
        Method::POST,
        "/notifications/mark-read",
        &ctx.read_token,
        &body,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let rows_after =
        notification_repo::list_notifications_for_recipient(&ctx.pool, ctx.reviewer_id)
            .await
            .expect("list after");
    assert!(rows_after[0].read_at.is_some());
}

// EC-1: empty notification list → 200 with empty array
#[tokio::test]
async fn list_notifications_for_user_with_no_notifications_returns_empty() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let response = send_request(
        &ctx.app,
        Method::GET,
        "/notifications",
        Some(&ctx.read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["notifications"]
            .as_array()
            .expect("notifications")
            .len(),
        0
    );
}

// EC-2: mark-read with IDs from another recipient → their read_at stays NULL
#[tokio::test]
async fn mark_notifications_read_does_not_touch_other_recipients() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    // Insert a notification directly owned by other_id
    let notif_id = Uuid::new_v4();
    notification_repo::insert_notification(
        &ctx.pool,
        &dubbridge_db::notification_repo::NotificationRow {
            id: notif_id,
            recipient_subject_id: ctx.other_id,
            kind: dubbridge_db::notification_repo::NotificationKind::ReviewTaskDecided,
            ref_entity_type: dubbridge_db::notification_repo::RefEntityType::ReviewTask,
            ref_entity_id: Uuid::new_v4(),
            actor_subject_id: Some(ctx.reviewer_id),
            read_at: None,
            created_at: OffsetDateTime::now_utc(),
        },
    )
    .await
    .expect("insert notification");

    // reviewer (not other_id) tries to mark the other user's notification read
    let body = format!(r#"{{"ids":["{notif_id}"]}}"#);
    let response = send_json(
        &ctx.app,
        Method::POST,
        "/notifications/mark-read",
        &ctx.read_token, // reviewer's token, not other_id's
        &body,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // other_id's notification must still be unread
    let rows = notification_repo::list_notifications_for_recipient(&ctx.pool, ctx.other_id)
        .await
        .expect("list other");
    assert_eq!(rows.len(), 1);
    assert!(rows[0].read_at.is_none());
}

// EC-2b: GET /notifications returns ONLY the caller's rows — cross-recipient isolation
#[tokio::test]
async fn list_notifications_excludes_other_recipients_rows() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    // Insert a notification for other_id directly
    notification_repo::insert_notification(
        &ctx.pool,
        &dubbridge_db::notification_repo::NotificationRow {
            id: Uuid::new_v4(),
            recipient_subject_id: ctx.other_id,
            kind: dubbridge_db::notification_repo::NotificationKind::ReviewTaskDecided,
            ref_entity_type: dubbridge_db::notification_repo::RefEntityType::ReviewTask,
            ref_entity_id: Uuid::new_v4(),
            actor_subject_id: Some(ctx.reviewer_id),
            read_at: None,
            created_at: OffsetDateTime::now_utc(),
        },
    )
    .await
    .expect("insert other's notification");

    // reviewer calls GET /notifications — should see 0 rows (none addressed to them)
    let reviewer_response = send_request(
        &ctx.app,
        Method::GET,
        "/notifications",
        Some(&ctx.read_token),
        None,
        Body::empty(),
    )
    .await;
    let reviewer_body = json_body(reviewer_response).await;
    assert_eq!(
        reviewer_body["notifications"]
            .as_array()
            .expect("notifications")
            .len(),
        0
    );

    // other_id calls GET /notifications — should see exactly 1 row (their own)
    let other_response = send_request(
        &ctx.app,
        Method::GET,
        "/notifications",
        Some(&ctx.other_read_token),
        None,
        Body::empty(),
    )
    .await;
    let other_body = json_body(other_response).await;
    assert_eq!(
        other_body["notifications"]
            .as_array()
            .expect("notifications")
            .len(),
        1
    );
}

// EC-3: publish refusal does NOT emit a notification (no publication = no publish notification)
#[tokio::test]
async fn publish_refusal_does_not_emit_notification() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;

    // Attempt to publish a non-approved task (will be refused)
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
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let rows = notification_repo::list_notifications_for_recipient(&ctx.pool, ctx.reviewer_id)
        .await
        .expect("list");
    assert_eq!(rows.len(), 0);
}

// EC-4: no PII in notification payload — kind and ref_entity_id only, no asset title
#[tokio::test]
async fn notification_response_carries_only_reference_fields() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_review_fixture(&ctx.pool, ctx.reviewer_id).await;
    send_json(
        &ctx.app,
        Method::POST,
        &format!(
            "/orgs/{}/projects/{}/review-tasks/{}/decision",
            fixture.org.id.0, fixture.project.id.0, fixture.task_id.0
        ),
        &ctx.write_token,
        r#"{"verdict":"approved","comment":"ok"}"#,
    )
    .await;

    let response = send_request(
        &ctx.app,
        Method::GET,
        "/notifications",
        Some(&ctx.read_token),
        None,
        Body::empty(),
    )
    .await;
    let body = json_body(response).await;
    let notif = &body["notifications"][0];

    // reference fields must be present
    assert!(notif["ref_entity_id"].is_string());
    assert!(notif["kind"].is_string());
    assert!(notif["ref_entity_type"].is_string());
    assert!(notif["actor_subject_id"].is_string());
    // freeform / PII fields must be absent
    assert!(notif.get("title").is_none());
    assert!(notif.get("message").is_none());
    assert!(notif.get("comment").is_none());
}

// EC-5: unauthenticated request → 401
#[tokio::test]
async fn list_notifications_without_token_returns_401() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let response = send_request(
        &ctx.app,
        Method::GET,
        "/notifications",
        None,
        None,
        Body::empty(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// --- helpers ----------------------------------------------------------------

async fn insert_review_fixture(pool: &PgPool, subject_id: Uuid) -> ReviewFixture {
    let org = Organization::new(format!("NotifOrg-{}", Uuid::new_v4()));
    insert_org(pool, &org).await;
    insert_membership(pool, org.id, subject_id, OrgRole::Reviewer).await;

    let project = Project::new(org.id, format!("NotifProject-{}", Uuid::new_v4()));
    insert_project(pool, &project).await;

    let mut asset = Asset::new_pending(format!("notif-asset-{}", Uuid::new_v4()), subject_id);
    asset.status = IngestionStatus::Finalized;
    asset_repo_insert(pool, &asset).await;
    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project.id.0)
        .bind(asset.id.0)
        .execute(pool)
        .await
        .expect("project asset");

    let target_language = TargetLanguage::new(project.id, "en".to_string(), "fr".to_string());
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

async fn append_approved_decision(pool: &PgPool, task_id: ReviewTaskId, reviewer_id: Uuid) {
    review_repo::append_review_decision(
        pool,
        &ReviewDecisionRow {
            id: Uuid::new_v4(),
            review_task_id: task_id,
            verdict: ReviewVerdict::Approved,
            comment: Some("approved".to_string()),
            reviewer_subject_id: reviewer_id,
            happened_at: OffsetDateTime::now_utc(),
        },
    )
    .await
    .expect("append approved decision");
}

async fn migrate_and_reset(pool: &PgPool) {
    sqlx::migrate!("../../infra/migrations")
        .run(pool)
        .await
        .expect("migrations");

    sqlx::query(
        "TRUNCATE TABLE notifications, push_tokens, publications, review_decisions, review_tasks, \
         target_languages, project_assets, projects, org_members, organizations, \
         pending_ingestions, audit_events, artifact_records, rights_records, assets \
         RESTART IDENTITY CASCADE",
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
