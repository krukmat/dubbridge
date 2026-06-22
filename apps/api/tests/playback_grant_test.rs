use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::{artifact_repo, playback_repo, preparation_repo};
use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact, PreparationStatus},
    asset::{Asset, AssetId, IngestionStatus},
    workspace::{OrgId, OrgMember, OrgRole, Organization, Project},
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
    app: axum::Router,
    reviewer_id: Uuid,
    write_token: String,
    outsider_write_token: String,
}

struct PlaybackFixture {
    org: Organization,
    _project: Project,
    asset: Asset,
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

        let reviewer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440130").expect("uuid");
        let outsider_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440131").expect("uuid");

        let write_token = "playback-it-reviewer-write-token".to_string();
        let outsider_write_token = "playback-it-outsider-write-token".to_string();

        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default()
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
                    &outsider_write_token,
                    Ok(AuthenticatedPrincipal::new(
                        outsider_id,
                        ["workspaces:read", "workspaces:write"]
                            .into_iter()
                            .map(str::to_string),
                    )),
                ),
        );

        let state = Arc::new(AppState::new(
            pool.clone(),
            Box::new(LocalFsAdapter::new(&storage_path)),
            verifier.clone(),
            dubbridge_config::AppConfig::from_env(),
        ));

        Some(Self {
            pool,
            app: build_app(state, verifier),
            reviewer_id,
            write_token,
            outsider_write_token,
        })
    }
}

#[tokio::test]
async fn authorized_reviewer_ready_asset_receives_grant_and_audit_row() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    insert_membership(
        &ctx.pool,
        fixture.org.id,
        ctx.reviewer_id,
        OrgRole::Reviewer,
    )
    .await;
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        Some(&ctx.write_token),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;
    let grant_id = Uuid::parse_str(body["grant_id"].as_str().expect("grant id")).expect("uuid");

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(count_playback_grants(&ctx.pool).await, 1);
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        1
    );
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused").await,
        0
    );

    let grant = playback_repo::get_active_grant(
        &ctx.pool,
        dubbridge_domain::playback::PlaybackGrantId(grant_id),
    )
    .await
    .expect("load grant")
    .expect("grant exists");
    assert_eq!(grant.asset_id, fixture.asset.id);
    assert_eq!(grant.principal.principal_id, ctx.reviewer_id);
}

#[tokio::test]
async fn unauthenticated_request_returns_401_and_writes_no_grant_row() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        0
    );
}

#[tokio::test]
async fn authenticated_non_member_returns_403_and_writes_no_grant_row() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    insert_membership(
        &ctx.pool,
        fixture.org.id,
        ctx.reviewer_id,
        OrgRole::Reviewer,
    )
    .await;
    mark_asset_ready_with_manifest(&ctx.pool, fixture.asset.id).await;

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        Some(&ctx.outsider_write_token),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "asset not found");
    assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        0
    );
}

#[tokio::test]
async fn not_ready_asset_returns_fail_closed_denial_and_writes_no_grant_row() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    insert_membership(
        &ctx.pool,
        fixture.org.id,
        ctx.reviewer_id,
        OrgRole::Reviewer,
    )
    .await;
    preparation_repo::upsert_preparation_status(
        &ctx.pool,
        fixture.asset.id,
        PreparationStatus::InProgress,
        None,
    )
    .await
    .expect("mark in progress");

    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", fixture.asset.id.0),
        Some(&ctx.write_token),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "asset not ready for playback");
    assert_eq!(count_playback_grants(&ctx.pool).await, 0);
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        0
    );
}

async fn migrate_and_reset(pool: &PgPool) {
    sqlx::migrate!("../../infra/migrations")
        .run(pool)
        .await
        .expect("migrations");

    sqlx::query(
        "TRUNCATE TABLE playback_grants, publications, review_decisions, review_tasks, asset_preparation_status, target_languages, project_assets, projects, org_members, organizations, pending_ingestions, audit_events, artifact_records, rights_records, assets RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("truncate tables");
}

async fn insert_playback_fixture(pool: &PgPool) -> PlaybackFixture {
    let org = Organization::new("Playback IT Org".to_string());
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org.id.0)
        .bind(&org.name)
        .execute(pool)
        .await
        .expect("insert org");

    let project = Project::new(org.id, "Playback IT Project".to_string());
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project.id.0)
        .bind(project.org_id.0)
        .bind(&project.name)
        .execute(pool)
        .await
        .expect("insert project");

    let mut asset = Asset::new_pending("playback-it-asset".to_string(), Uuid::new_v4());
    asset.status = IngestionStatus::Finalized;
    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset.id.0)
        .bind(&asset.title)
        .bind(asset.uploader_id)
        .bind(asset.status.to_string())
        .execute(pool)
        .await
        .expect("insert asset");

    sqlx::query("INSERT INTO project_assets (project_id, asset_id) VALUES ($1, $2)")
        .bind(project.id.0)
        .bind(asset.id.0)
        .execute(pool)
        .await
        .expect("insert project asset");

    PlaybackFixture {
        org,
        _project: project,
        asset,
    }
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

async fn mark_asset_ready_with_manifest(pool: &PgPool, asset_id: AssetId) {
    preparation_repo::upsert_preparation_status(pool, asset_id, PreparationStatus::Ready, None)
        .await
        .expect("upsert ready");

    let source = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{asset_id}/source.mp4"),
        "video/mp4".into(),
        1_000,
        "deadbeef".into(),
    );
    artifact_repo::insert_artifact_record(pool, &source)
        .await
        .expect("insert source artifact");

    let manifest = DerivedArtifact::new(
        asset_id,
        source.id,
        ArtifactKind::HlsManifest,
        format!("prepared/{asset_id}/index.m3u8"),
        "application/vnd.apple.mpegurl".into(),
        512,
        "manifestchk".into(),
    );
    preparation_repo::insert_derived_artifact(pool, &manifest)
        .await
        .expect("insert manifest");
}

async fn send_request(
    app: &axum::Router,
    method: Method,
    uri: &str,
    bearer_token: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer_token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    app.clone()
        .oneshot(builder.body(Body::empty()).expect("request"))
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

async fn count_playback_grants(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM playback_grants")
        .fetch_one(pool)
        .await
        .expect("count playback grants")
}

async fn count_audit_events_by_kind(pool: &PgPool, asset_id: Uuid, event_kind: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE asset_id = $1 AND event_kind = $2")
        .bind(asset_id)
        .bind(event_kind)
        .fetch_one(pool)
        .await
        .expect("count audit events")
}
