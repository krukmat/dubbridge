#![allow(dead_code)]

use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::{artifact_repo, preparation_repo};
use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact, PreparationStatus},
    asset::{Asset, AssetId, IngestionStatus},
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

pub struct PlaybackTestContext {
    pub pool: PgPool,
    pub app: axum::Router,
    pub storage_path: PathBuf,
    pub reviewer_id: Uuid,
    pub write_token: String,
    outsider_write_token: Option<String>,
}

pub struct PlaybackFixture {
    pub org: Organization,
    pub _project: Project,
    pub asset: Asset,
}

impl PlaybackTestContext {
    pub async fn grant_suite() -> Option<Self> {
        new_playback_context(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440130").expect("uuid"),
            "playback-it-reviewer-write-token",
            Some((
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440131").expect("uuid"),
                "playback-it-outsider-write-token",
            )),
        )
        .await
    }

    pub async fn delivery_suite() -> Option<Self> {
        new_playback_context(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440140").expect("uuid"),
            "playback-delivery-reviewer-write-token",
            None,
        )
        .await
    }

    pub fn outsider_write_token(&self) -> Option<&str> {
        self.outsider_write_token.as_deref()
    }

    pub async fn put_manifest(&self, asset_id: AssetId, manifest: &str) {
        self.put_manifest_bytes(asset_id, manifest.as_bytes().to_vec())
            .await;
    }

    pub async fn put_manifest_bytes(&self, asset_id: AssetId, bytes: Vec<u8>) {
        let adapter = LocalFsAdapter::new(&self.storage_path);
        adapter
            .put(&hls_manifest_key(&asset_id.0.to_string()), bytes)
            .await
            .expect("store manifest");
    }

    pub async fn put_segment(&self, asset_id: AssetId, filename: &str, bytes: Vec<u8>) {
        let adapter = LocalFsAdapter::new(&self.storage_path);
        adapter
            .put(&hls_segment_key(&asset_id.0.to_string(), filename), bytes)
            .await
            .expect("store segment");
    }
}

async fn new_playback_context(
    reviewer_id: Uuid,
    write_token: &str,
    outsider: Option<(Uuid, &str)>,
) -> Option<PlaybackTestContext> {
    let database_url = env::var("DUBBRIDGE_DATABASE_URL").ok()?;
    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect database");
    migrate_and_reset(&pool).await;

    let storage_dir = Arc::new(TempDir::new().expect("temp dir"));
    let storage_path = PathBuf::from(storage_dir.path());

    let mut verifier = StubTokenVerifier::default().with_token(
        write_token,
        Ok(AuthenticatedPrincipal::new(
            reviewer_id,
            ["workspaces:read", "workspaces:write"]
                .into_iter()
                .map(str::to_string),
        )),
    );
    let outsider_write_token = if let Some((outsider_id, token)) = outsider {
        verifier = verifier.with_token(
            token,
            Ok(AuthenticatedPrincipal::new(
                outsider_id,
                ["workspaces:read", "workspaces:write"]
                    .into_iter()
                    .map(str::to_string),
            )),
        );
        Some(token.to_string())
    } else {
        None
    };

    let verifier: SharedTokenVerifier = Arc::new(verifier);
    let state = Arc::new(AppState::new(
        pool.clone(),
        Box::new(LocalFsAdapter::new(&storage_path)),
        verifier.clone(),
        dubbridge_config::AppConfig::from_env(),
    ));

    Some(PlaybackTestContext {
        pool,
        app: build_app(state, verifier),
        storage_path,
        reviewer_id,
        write_token: write_token.to_string(),
        outsider_write_token,
    })
}

pub async fn insert_playback_fixture(pool: &PgPool) -> PlaybackFixture {
    let org = Organization::new("Playback Test Org".to_string());
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org.id.0)
        .bind(&org.name)
        .execute(pool)
        .await
        .expect("insert org");

    let project = Project::new(org.id, "Playback Test Project".to_string());
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(project.id.0)
        .bind(project.org_id.0)
        .bind(&project.name)
        .execute(pool)
        .await
        .expect("insert project");

    let mut asset = Asset::new_pending("playback-test-asset".to_string(), Uuid::new_v4());
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

pub async fn insert_membership(pool: &PgPool, org_id: OrgId, subject_id: Uuid, role: OrgRole) {
    let member = OrgMember::new(org_id, subject_id, role);
    sqlx::query("INSERT INTO org_members (org_id, subject_id, role) VALUES ($1, $2, $3)")
        .bind(member.org_id.0)
        .bind(member.subject_id)
        .bind(member.role.to_string())
        .execute(pool)
        .await
        .expect("insert membership");
}

pub async fn mark_asset_ready_with_manifest(pool: &PgPool, asset_id: AssetId) {
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

pub async fn send_request(
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

pub async fn json_body(response: axum::response::Response) -> serde_json::Value {
    serde_json::from_slice(
        &to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("body"),
    )
    .expect("json body")
}

pub async fn body_string(response: axum::response::Response) -> String {
    String::from_utf8(response_body_bytes(response).await).expect("utf-8 body")
}

pub async fn response_body_bytes(response: axum::response::Response) -> Vec<u8> {
    to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body")
        .to_vec()
}

pub async fn count_playback_grants(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM playback_grants")
        .fetch_one(pool)
        .await
        .expect("count playback grants")
}

pub async fn count_audit_events_by_kind(pool: &PgPool, asset_id: Uuid, event_kind: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE asset_id = $1 AND event_kind = $2")
        .bind(asset_id)
        .bind(event_kind)
        .fetch_one(pool)
        .await
        .expect("count audit events")
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
