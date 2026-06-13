use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::{asset_repo, workspace_repo};
use dubbridge_domain::{
    asset::Asset,
    workspace::{OrgMember, OrgRole, Organization, Project},
};
use dubbridge_storage::LocalFsAdapter;
use sqlx::PgPool;
use tempfile::TempDir;
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
    _storage_dir: Arc<TempDir>,
    _storage_path: PathBuf,
    app: axum::Router,
    admin_id: Uuid,
    viewer_id: Uuid,
    outsider_id: Uuid,
    workspace_write_token: String,
    workspace_read_token: String,
    assets_only_token: String,
    viewer_write_token: String,
}

impl TestContext {
    async fn new() -> Option<Self> {
        let database_url = match env::var("DUBBRIDGE_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => return None,
        };

        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect database");
        migrate_and_reset(&pool).await;

        let storage_dir = Arc::new(TempDir::new().expect("temp dir"));
        let storage_path = storage_dir.path().to_path_buf();

        let admin_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid");
        let viewer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").expect("uuid");
        let outsider_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").expect("uuid");

        let workspace_write_token = "workspace-write-token".to_string();
        let workspace_read_token = "workspace-read-token".to_string();
        let assets_only_token = "assets-only-token".to_string();
        let viewer_write_token = "viewer-write-token".to_string();

        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default()
                .with_token(
                    &workspace_write_token,
                    Ok(AuthenticatedPrincipal::new(
                        admin_id,
                        ["workspaces:read", "workspaces:write"]
                            .into_iter()
                            .map(str::to_string),
                    )),
                )
                .with_token(
                    &workspace_read_token,
                    Ok(AuthenticatedPrincipal::new(
                        admin_id,
                        ["workspaces:read"].into_iter().map(str::to_string),
                    )),
                )
                .with_token(
                    &assets_only_token,
                    Ok(AuthenticatedPrincipal::new(
                        admin_id,
                        ["assets:ingest"].into_iter().map(str::to_string),
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
            _storage_dir: storage_dir,
            _storage_path: storage_path,
            app: build_app(state, verifier),
            admin_id,
            viewer_id,
            outsider_id,
            workspace_write_token,
            workspace_read_token,
            assets_only_token,
            viewer_write_token,
        })
    }
}

#[tokio::test]
async fn create_org_creates_owner_membership_and_audit() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let response = send_json(
        &ctx.app,
        Method::POST,
        "/orgs",
        &ctx.workspace_write_token,
        r#"{"name":"Acme"}"#,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;
    let org_id = Uuid::parse_str(body["id"].as_str().expect("org id")).expect("uuid");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Acme");

    let owner_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM org_members WHERE org_id = $1 AND subject_id = $2 AND role = 'owner'",
    )
    .bind(org_id)
    .bind(ctx.admin_id)
    .fetch_one(&ctx.pool)
    .await
    .expect("owner count");
    assert_eq!(owner_count, 1);

    assert_eq!(
        count_workspace_audit_events(&ctx.pool, "org_created").await,
        1
    );
}

#[tokio::test]
async fn list_orgs_returns_only_subject_memberships() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let admin_org =
        insert_org_for_subject(&ctx.pool, "Admin Org", ctx.admin_id, OrgRole::Owner).await;
    let _viewer_org =
        insert_org_for_subject(&ctx.pool, "Viewer Org", ctx.viewer_id, OrgRole::Viewer).await;

    let response = send_request(
        &ctx.app,
        Method::GET,
        "/orgs",
        Some(&ctx.workspace_read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().expect("array").len(), 1);
    assert_eq!(body[0]["id"], admin_org.id.0.to_string());
}

#[tokio::test]
async fn add_member_emits_audit_and_members_are_listed() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;

    let add_member_response = send_json(
        &ctx.app,
        Method::POST,
        &format!("/orgs/{}/members", org.id.0),
        &ctx.workspace_write_token,
        &format!(r#"{{"subject_id":"{}","role":"reviewer"}}"#, ctx.viewer_id),
    )
    .await;
    assert_eq!(add_member_response.status(), StatusCode::CREATED);

    let list_members_response = send_request(
        &ctx.app,
        Method::GET,
        &format!("/orgs/{}/members", org.id.0),
        Some(&ctx.workspace_read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = list_members_response.status();
    let body = json_body(list_members_response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().expect("array").len(), 2);
    assert!(
        body.as_array()
            .expect("array")
            .iter()
            .any(|member| member["subject_id"] == ctx.viewer_id.to_string()
                && member["role"] == "reviewer")
    );

    assert_eq!(
        count_workspace_audit_events(&ctx.pool, "org_member_added").await,
        1
    );
}

#[tokio::test]
async fn workspace_mutation_rejects_assets_only_scope() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!("/orgs/{}/members", org.id.0),
        &ctx.assets_only_token,
        &format!(r#"{{"subject_id":"{}","role":"viewer"}}"#, ctx.viewer_id),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn viewer_cannot_create_project_and_no_row_is_written() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;
    insert_member(&ctx.pool, org.id.0, ctx.viewer_id, OrgRole::Viewer).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!("/orgs/{}/projects", org.id.0),
        &ctx.viewer_write_token,
        r#"{"name":"Viewer Project","asset_ids":[]}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(count_projects_for_org(&ctx.pool, org.id.0).await, 0);
}

#[tokio::test]
async fn create_project_returns_detail_with_asset_summaries_and_audit() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;
    let asset_a = insert_asset_for_uploader(&ctx.pool, "Trailer A", ctx.admin_id).await;
    let asset_b = insert_asset_for_uploader(&ctx.pool, "Trailer B", ctx.admin_id).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!("/orgs/{}/projects", org.id.0),
        &ctx.workspace_write_token,
        &format!(
            r#"{{"name":"Season 1","asset_ids":["{}","{}"]}}"#,
            asset_a.id.0, asset_b.id.0
        ),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;
    let project_id = Uuid::parse_str(body["id"].as_str().expect("project id")).expect("uuid");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Season 1");
    assert_eq!(body["assets"].as_array().expect("assets").len(), 2);
    assert_eq!(count_project_asset_links(&ctx.pool, project_id).await, 2);
    assert_eq!(
        count_workspace_audit_events(&ctx.pool, "project_created").await,
        1
    );
}

#[tokio::test]
async fn create_project_rejects_asset_owned_by_another_principal() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;
    let foreign_asset =
        insert_asset_for_uploader(&ctx.pool, "Foreign Asset", ctx.outsider_id).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!("/orgs/{}/projects", org.id.0),
        &ctx.workspace_write_token,
        &format!(
            r#"{{"name":"Blocked Project","asset_ids":["{}"]}}"#,
            foreign_asset.id.0
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(count_projects_for_org(&ctx.pool, org.id.0).await, 0);
}

#[tokio::test]
async fn cross_org_project_traversal_is_forbidden() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org_a = insert_org_for_subject(&ctx.pool, "Org A", ctx.admin_id, OrgRole::Owner).await;
    let org_b = insert_org_for_subject(&ctx.pool, "Org B", ctx.admin_id, OrgRole::Owner).await;
    let project_b = insert_project_for_org(&ctx.pool, org_b.id.0, "Project B").await;

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!("/orgs/{}/projects/{}", org_a.id.0, project_b.id.0),
        Some(&ctx.workspace_read_token),
        None,
        Body::empty(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn target_languages_and_project_detail_are_readable() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;
    let project = insert_project_for_org(&ctx.pool, org.id.0, "Launch").await;
    let asset = insert_asset_for_uploader(&ctx.pool, "Launch Trailer", ctx.admin_id).await;
    workspace_repo::link_asset_to_project(&ctx.pool, project.id, asset.id, ctx.admin_id)
        .await
        .expect("link asset");

    let set_response = send_json(
        &ctx.app,
        Method::PUT,
        &format!(
            "/orgs/{}/projects/{}/target-languages",
            org.id.0, project.id.0
        ),
        &ctx.workspace_write_token,
        r#"{"source_lang":"en","target_languages":["es-ES","fr-FR"]}"#,
    )
    .await;
    assert_eq!(set_response.status(), StatusCode::OK);

    let detail_response = send_request(
        &ctx.app,
        Method::GET,
        &format!("/orgs/{}/projects/{}", org.id.0, project.id.0),
        Some(&ctx.workspace_read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = detail_response.status();
    let body = json_body(detail_response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["assets"].as_array().expect("assets").len(), 1);
    assert_eq!(
        body["target_languages"]
            .as_array()
            .expect("target languages")
            .len(),
        2
    );
}

#[tokio::test]
async fn create_org_fails_closed_when_audit_persistence_fails() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    drop_audit_events_table(&ctx.pool).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        "/orgs",
        &ctx.workspace_write_token,
        r#"{"name":"Broken Audit Org"}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(count_organizations(&ctx.pool).await, 0);
}

#[tokio::test]
async fn add_member_fails_closed_when_audit_persistence_fails() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;
    drop_audit_events_table(&ctx.pool).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!("/orgs/{}/members", org.id.0),
        &ctx.workspace_write_token,
        &format!(r#"{{"subject_id":"{}","role":"editor"}}"#, ctx.viewer_id),
    )
    .await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        count_members_for_org_subject(&ctx.pool, org.id.0, ctx.viewer_id).await,
        0
    );
}

#[tokio::test]
async fn create_project_fails_closed_when_audit_persistence_fails() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let org = insert_org_for_subject(&ctx.pool, "Acme", ctx.admin_id, OrgRole::Owner).await;
    drop_audit_events_table(&ctx.pool).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        &format!("/orgs/{}/projects", org.id.0),
        &ctx.workspace_write_token,
        r#"{"name":"Broken Audit Project","asset_ids":[]}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(count_projects_for_org(&ctx.pool, org.id.0).await, 0);
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
        Some("application/json"),
        Body::from(body.to_string()),
    )
    .await
}

async fn send_request(
    app: &axum::Router,
    method: Method,
    uri: &str,
    token: Option<&str>,
    content_type: Option<&str>,
    body: Body,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(uri);

    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    if let Some(content_type) = content_type {
        request = request.header(header::CONTENT_TYPE, content_type);
    }

    app.clone()
        .oneshot(request.body(body).expect("request"))
        .await
        .expect("response")
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("json body")
}

async fn migrate_and_reset(pool: &PgPool) {
    // Fail-closed tests drop audit_events intentionally. If it's missing,
    // remove its migration records so sqlx re-creates it.
    let audit_exists: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM pg_tables WHERE schemaname='public' AND tablename='audit_events'",
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);
    if audit_exists.is_none() {
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version IN (4, 9)")
            .execute(pool)
            .await
            .expect("clear stale migration records");
    }

    sqlx::migrate!("../../infra/migrations")
        .run(pool)
        .await
        .expect("migrations");
    sqlx::query(
        "TRUNCATE TABLE target_languages, project_assets, projects, org_members, organizations, pending_ingestions, audit_events, artifact_records, rights_records, assets RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("truncate");
}

async fn insert_org_for_subject(
    pool: &PgPool,
    name: &str,
    subject_id: Uuid,
    role: OrgRole,
) -> Organization {
    let org = Organization::new(name.to_string());
    workspace_repo::insert_org(pool, &org)
        .await
        .expect("insert org");
    workspace_repo::add_org_member(pool, &OrgMember::new(org.id, subject_id, role))
        .await
        .expect("insert member");
    org
}

async fn insert_member(pool: &PgPool, org_id: Uuid, subject_id: Uuid, role: OrgRole) {
    workspace_repo::add_org_member(
        pool,
        &OrgMember::new(dubbridge_domain::workspace::OrgId(org_id), subject_id, role),
    )
    .await
    .expect("insert member");
}

async fn insert_project_for_org(pool: &PgPool, org_id: Uuid, name: &str) -> Project {
    let project = Project::new(dubbridge_domain::workspace::OrgId(org_id), name.to_string());
    workspace_repo::insert_project(pool, &project)
        .await
        .expect("insert project");
    project
}

async fn insert_asset_for_uploader(pool: &PgPool, title: &str, uploader_id: Uuid) -> Asset {
    let asset = Asset::new_pending(title.to_string(), uploader_id);
    asset_repo::insert_asset(pool, &asset)
        .await
        .expect("insert asset");
    asset
}

async fn drop_audit_events_table(pool: &PgPool) {
    sqlx::query("DROP TABLE audit_events")
        .execute(pool)
        .await
        .expect("drop audit_events");
}

async fn count_workspace_audit_events(pool: &PgPool, event_kind: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE event_kind = $1")
        .bind(event_kind)
        .fetch_one(pool)
        .await
        .expect("audit count")
}

async fn count_organizations(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM organizations")
        .fetch_one(pool)
        .await
        .expect("organization count")
}

async fn count_members_for_org_subject(pool: &PgPool, org_id: Uuid, subject_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM org_members WHERE org_id = $1 AND subject_id = $2")
        .bind(org_id)
        .bind(subject_id)
        .fetch_one(pool)
        .await
        .expect("member count")
}

async fn count_projects_for_org(pool: &PgPool, org_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE org_id = $1")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .expect("project count")
}

async fn count_project_asset_links(pool: &PgPool, project_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM project_assets WHERE project_id = $1")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .expect("project asset count")
}
