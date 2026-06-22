use std::{collections::HashMap, env, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, Hs256Issuer, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::{artifact_repo, playback_repo, preparation_repo};
use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact, PreparationStatus},
    asset::{Asset, AssetId, IngestionStatus},
    playback::PlaybackGrantId,
    workspace::{OrgId, OrgMember, OrgRole, Organization, Project},
};
use dubbridge_storage::{LocalFsAdapter, StorageAdapter, hls_manifest_key, hls_segment_key};
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
    storage_path: PathBuf,
    reviewer_id: Uuid,
    write_token: String,
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

        let reviewer_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440140").expect("uuid");
        let write_token = "playback-delivery-reviewer-write-token".to_string();

        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default().with_token(
                &write_token,
                Ok(AuthenticatedPrincipal::new(
                    reviewer_id,
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
            storage_path,
            reviewer_id,
            write_token,
        })
    }

    async fn put_manifest(&self, asset_id: AssetId, manifest: &str) {
        self.put_manifest_bytes(asset_id, manifest.as_bytes().to_vec())
            .await;
    }

    async fn put_manifest_bytes(&self, asset_id: AssetId, bytes: Vec<u8>) {
        let adapter = LocalFsAdapter::new(&self.storage_path);
        adapter
            .put(&hls_manifest_key(&asset_id.0.to_string()), bytes)
            .await
            .expect("store manifest");
    }

    async fn put_segment(&self, asset_id: AssetId, filename: &str, bytes: Vec<u8>) {
        let adapter = LocalFsAdapter::new(&self.storage_path);
        adapter
            .put(&hls_segment_key(&asset_id.0.to_string(), filename), bytes)
            .await
            .expect("store segment");
    }
}

#[tokio::test]
async fn valid_grant_returns_manifest_with_short_lived_segment_references() {
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
    ctx.put_manifest(
        fixture.asset.id,
        "#EXTM3U\n#EXT-X-VERSION:3\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n#EXTINF:6.0,\nprepared/asset-123/segment_00001.ts\n#EXT-X-ENDLIST\n",
    )
    .await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00001.ts",
        b"segment-one".to_vec(),
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .expect("content type");
    let body = body_string(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/vnd.apple.mpegurl");
    assert!(body.contains("#EXTM3U"));
    assert!(body.contains(&format!(
        "/assets/{}/playback/segments/segment_00000.ts?token=",
        fixture.asset.id.0
    )));
    assert!(body.contains(&format!(
        "/assets/{}/playback/segments/segment_00001.ts?token=",
        fixture.asset.id.0
    )));
    assert!(!body.contains("prepared/asset-123/segment_00000.ts"));
    assert!(!body.contains(&format!("assets/{}/prepared/hls/", fixture.asset.id.0)));
    assert_ne!(grant_id, Uuid::nil());
}

#[tokio::test]
async fn valid_short_lived_segment_reference_returns_segment_bytes_without_new_audit_row() {
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
    ctx.put_manifest(
        fixture.asset.id,
        "#EXTM3U\n#EXT-X-VERSION:3\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n#EXT-X-ENDLIST\n",
    )
    .await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let manifest_response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let manifest_body = body_string(manifest_response).await;
    let segment_url = manifest_segment_urls(&manifest_body)
        .into_iter()
        .next()
        .expect("segment url");

    let response = send_request(&ctx.app, Method::GET, &segment_url, None).await;
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .expect("content type");
    let body = response_body_bytes(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "video/mp2t");
    assert_eq!(body, b"segment-zero");
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_issued").await,
        1
    );
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, fixture.asset.id.0, "playback_grant_refused").await,
        0
    );
}

#[tokio::test]
async fn expired_grant_manifest_request_is_denied_fail_closed() {
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
    ctx.put_manifest(
        fixture.asset.id,
        "#EXTM3U\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n",
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    playback_repo::expire_grant(&ctx.pool, PlaybackGrantId(grant_id))
        .await
        .expect("expire grant");

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "playback grant expired or revoked");
}

#[tokio::test]
async fn missing_stored_manifest_fails_closed_without_fabricated_playlist() {
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

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "prepared HLS manifest not found");
}

#[tokio::test]
async fn invalid_utf8_manifest_fails_closed_without_leaking_storage_key() {
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
    ctx.put_manifest_bytes(fixture.asset.id, vec![0xff, 0xfe, 0xfd])
        .await;

    let grant_id = issue_grant(&ctx, fixture.asset.id).await;
    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let error = body["error"].as_str().expect("error string");
    assert!(error.contains("stored manifest is not valid UTF-8"));
    assert!(!error.contains("prepared/hls"));
    assert!(!error.contains("index.m3u8"));
}

#[tokio::test]
async fn expired_short_lived_segment_reference_is_denied_fail_closed() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture = insert_playback_fixture(&ctx.pool).await;
    ctx.put_segment(
        fixture.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;

    let expired_token = expired_segment_token(fixture.asset.id, "segment_00000.ts");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/segments/segment_00000.ts?token={expired_token}",
            fixture.asset.id.0
        ),
        None,
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "segment reference expired or invalid");
}

#[tokio::test]
async fn scoped_segment_reference_cannot_be_replayed_against_another_asset() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let fixture_a = insert_playback_fixture(&ctx.pool).await;
    let fixture_b = insert_playback_fixture(&ctx.pool).await;
    insert_membership(
        &ctx.pool,
        fixture_a.org.id,
        ctx.reviewer_id,
        OrgRole::Reviewer,
    )
    .await;
    mark_asset_ready_with_manifest(&ctx.pool, fixture_a.asset.id).await;
    ctx.put_manifest(
        fixture_a.asset.id,
        "#EXTM3U\n#EXTINF:6.0,\nprepared/asset-123/segment_00000.ts\n#EXT-X-ENDLIST\n",
    )
    .await;
    ctx.put_segment(
        fixture_a.asset.id,
        "segment_00000.ts",
        b"segment-zero".to_vec(),
    )
    .await;

    let grant_id = issue_grant(&ctx, fixture_a.asset.id).await;
    let manifest_response = send_request(
        &ctx.app,
        Method::GET,
        &format!(
            "/assets/{}/playback/{}/manifest",
            fixture_a.asset.id.0, grant_id
        ),
        None,
    )
    .await;
    let manifest_body = body_string(manifest_response).await;
    let segment_url = manifest_segment_urls(&manifest_body)
        .into_iter()
        .next()
        .expect("segment url");
    let replay_url = segment_url.replace(
        &fixture_a.asset.id.0.to_string(),
        &fixture_b.asset.id.0.to_string(),
    );

    let response = send_request(&ctx.app, Method::GET, &replay_url, None).await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "segment reference expired or invalid");
}

async fn issue_grant(ctx: &TestContext, asset_id: AssetId) -> Uuid {
    let response = send_request(
        &ctx.app,
        Method::POST,
        &format!("/assets/{}/playback-grants", asset_id.0),
        Some(&ctx.write_token),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CREATED);
    Uuid::parse_str(body["grant_id"].as_str().expect("grant id")).expect("uuid")
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
    let org = Organization::new("Playback Delivery Org".to_string());
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org.id.0)
        .bind(&org.name)
        .execute(pool)
        .await
        .expect("insert org");

    let project = Project::new(org.id, "Playback Delivery Project".to_string());
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project.id.0)
        .bind(project.org_id.0)
        .bind(&project.name)
        .execute(pool)
        .await
        .expect("insert project");

    let mut asset = Asset::new_pending("playback-delivery-asset".to_string(), Uuid::new_v4());
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

async fn body_string(response: axum::response::Response) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("body")
            .to_vec(),
    )
    .expect("utf8 body")
}

async fn response_body_bytes(response: axum::response::Response) -> Vec<u8> {
    to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body")
        .to_vec()
}

fn manifest_segment_urls(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(str::to_string)
        .collect()
}

fn expired_segment_token(asset_id: AssetId, filename: &str) -> String {
    let issuer = Hs256Issuer::new("local-dev-jwt-secret-placeholder", Duration::from_secs(0))
        .expect("issuer");
    issuer
        .generate_jwt(
            asset_id.0,
            asset_id.0,
            &[format!("playback_segment:{filename}")],
        )
        .expect("token")
}

async fn count_audit_events_by_kind(pool: &PgPool, asset_id: Uuid, event_kind: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE asset_id = $1 AND event_kind = $2")
        .bind(asset_id)
        .bind(event_kind)
        .fetch_one(pool)
        .await
        .expect("count audit events")
}
