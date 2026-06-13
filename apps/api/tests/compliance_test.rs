use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use dubbridge_api::{build_app, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
};
use dubbridge_db::{asset_repo, audit_repo, consent_repo, rights_repo};
use dubbridge_domain::{
    asset::Asset,
    audit::{AuditEvent, AuditEventKind},
    consent::{ConsentScope, ConsentStatus, new_grant, new_revoke},
    rights::{LicenseType, RightsBasis, RightsRecord, SourceType},
};
use dubbridge_storage::LocalFsAdapter;
use sqlx::PgPool;
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
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
    owner_id: Uuid,
    outsider_id: Uuid,
    assets_read_token: String,
    assets_ingest_token: String,
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
        migrate_db(&pool).await;

        let storage_dir = Arc::new(TempDir::new().expect("temp dir"));
        let storage_path = storage_dir.path().to_path_buf();

        let owner_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid");
        let outsider_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").expect("uuid");
        let assets_read_token = "assets-read-token".to_string();
        let assets_ingest_token = "assets-ingest-token".to_string();

        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default()
                .with_token(
                    &assets_read_token,
                    Ok(AuthenticatedPrincipal::new(
                        owner_id,
                        ["assets:read"].into_iter().map(str::to_string),
                    )),
                )
                .with_token(
                    &assets_ingest_token,
                    Ok(AuthenticatedPrincipal::new(
                        owner_id,
                        ["assets:ingest"].into_iter().map(str::to_string),
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
            owner_id,
            outsider_id,
            assets_read_token,
            assets_ingest_token,
        })
    }
}

#[tokio::test]
async fn get_audit_timeline_returns_owned_events_in_chronological_order() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let asset = insert_asset_for_uploader(&ctx.pool, "Timeline Asset", ctx.owner_id).await;
    let base_time = OffsetDateTime::now_utc();

    let mut revoke_event = AuditEvent::new_consent(
        asset.id,
        AuditEventKind::ConsentRevoked,
        Some("scope=voice_clone".to_string()),
    );
    revoke_event.happened_at = base_time + Duration::seconds(10);
    audit_repo::insert_audit_event(&ctx.pool, &revoke_event)
        .await
        .expect("insert revoke audit");

    let mut grant_event = AuditEvent::new_consent(
        asset.id,
        AuditEventKind::ConsentGranted,
        Some("scope=voice_clone".to_string()),
    );
    grant_event.happened_at = base_time;
    audit_repo::insert_audit_event(&ctx.pool, &grant_event)
        .await
        .expect("insert grant audit");

    let before_count = count_audit_events_for_asset(&ctx.pool, asset.id.0).await;
    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!("/assets/{}/audit", asset.id.0),
        Some(&ctx.assets_read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;
    let after_count = count_audit_events_for_asset(&ctx.pool, asset.id.0).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        before_count, after_count,
        "read API must be side-effect-free"
    );
    assert_eq!(body["events"].as_array().expect("events").len(), 2);
    assert_eq!(body["events"][0]["event_kind"], "consent_granted");
    assert_eq!(body["events"][1]["event_kind"], "consent_revoked");
}

#[tokio::test]
async fn get_audit_timeline_denies_non_owner_without_leaking_data() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let asset = insert_asset_for_uploader(&ctx.pool, "Foreign Asset", ctx.outsider_id).await;
    let event = AuditEvent::new_consent(
        asset.id,
        AuditEventKind::ConsentGranted,
        Some("scope=voice_clone".to_string()),
    );
    audit_repo::insert_audit_event(&ctx.pool, &event)
        .await
        .expect("insert audit");

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!("/assets/{}/audit", asset.id.0),
        Some(&ctx.assets_read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "asset not found");
    assert!(body.get("events").is_none());
}

#[tokio::test]
async fn get_rights_ledger_returns_owned_entries() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let asset = insert_asset_for_uploader(&ctx.pool, "Rights Asset", ctx.owner_id).await;
    let basis = RightsBasis {
        owner: "Acme Rights".to_string(),
        license_type: LicenseType::LicensedDistribution,
        source_type: SourceType::LicensedSource,
        proof_reference: "proof-001".to_string(),
    };
    rights_repo::insert_rights_record(&ctx.pool, &RightsRecord::new(asset.id, &basis))
        .await
        .expect("insert rights");

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!("/assets/{}/rights", asset.id.0),
        Some(&ctx.assets_read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["entries"].as_array().expect("entries").len(), 1);
    assert_eq!(body["entries"][0]["owner"], "Acme Rights");
    assert_eq!(body["entries"][0]["license_type"], "licensed_distribution");
}

#[tokio::test]
async fn get_consent_ledger_returns_rows_and_current_status() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let asset = insert_asset_for_uploader(&ctx.pool, "Consent Asset", ctx.owner_id).await;
    let grant =
        new_grant(asset.id, ConsentScope::VoiceClone, "ref-001", ctx.owner_id).expect("grant row");
    consent_repo::append_consent(&ctx.pool, &grant)
        .await
        .expect("append grant");
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    let revoke = new_revoke(asset.id, ConsentScope::VoiceClone, ctx.owner_id);
    consent_repo::append_consent(&ctx.pool, &revoke)
        .await
        .expect("append revoke");

    let response = send_request(
        &ctx.app,
        Method::GET,
        &format!("/assets/{}/consents", asset.id.0),
        Some(&ctx.assets_read_token),
        None,
        Body::empty(),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["current_status"], "revoke");
    assert_eq!(body["rows"].as_array().expect("rows").len(), 2);
    assert_eq!(body["rows"][0]["status"], "grant");
    assert_eq!(body["rows"][1]["status"], "revoke");
}

#[tokio::test]
async fn post_consents_grant_persists_row_and_audit() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let asset = insert_asset_for_uploader(&ctx.pool, "Grant Asset", ctx.owner_id).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        "/consents",
        &ctx.assets_ingest_token,
        &format!(
            r#"{{"asset_id":"{}","scope":"voice_clone","status":"grant","evidence_ref":"consent-proof-1"}}"#,
            asset.id.0
        ),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["current_status"], "grant");

    let latest =
        consent_repo::latest_consent_status(&ctx.pool, asset.id, &ConsentScope::VoiceClone)
            .await
            .expect("latest status");
    assert_eq!(latest, Some(ConsentStatus::Grant));
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, asset.id.0, "consent_granted").await,
        1
    );
}

#[tokio::test]
async fn post_consents_revoke_appends_row_and_preserves_history() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let asset = insert_asset_for_uploader(&ctx.pool, "Revoke Asset", ctx.owner_id).await;
    let grant =
        new_grant(asset.id, ConsentScope::VoiceClone, "ref-002", ctx.owner_id).expect("grant row");
    consent_repo::append_consent(&ctx.pool, &grant)
        .await
        .expect("append grant");

    let response = send_json(
        &ctx.app,
        Method::POST,
        "/consents",
        &ctx.assets_ingest_token,
        &format!(
            r#"{{"asset_id":"{}","scope":"voice_clone","status":"revoke"}}"#,
            asset.id.0
        ),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["current_status"], "revoke");

    let rows = consent_repo::list_consents_for_asset(&ctx.pool, asset.id)
        .await
        .expect("list consents");
    assert_eq!(rows.len(), 2);
    assert_eq!(derive_latest_status(&rows), Some(ConsentStatus::Revoke));
    assert_eq!(
        count_audit_events_by_kind(&ctx.pool, asset.id.0, "consent_revoked").await,
        1
    );
}

#[tokio::test]
async fn post_consents_grant_rejects_missing_evidence_ref() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let asset = insert_asset_for_uploader(&ctx.pool, "Invalid Grant Asset", ctx.owner_id).await;

    let response = send_json(
        &ctx.app,
        Method::POST,
        "/consents",
        &ctx.assets_ingest_token,
        &format!(
            r#"{{"asset_id":"{}","scope":"voice_clone","status":"grant"}}"#,
            asset.id.0
        ),
    )
    .await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        body["error"]
            .as_str()
            .expect("error")
            .contains("evidence_ref")
    );

    let rows = consent_repo::list_consents_for_asset(&ctx.pool, asset.id)
        .await
        .expect("list consents");
    assert!(rows.is_empty());
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

async fn migrate_db(pool: &PgPool) {
    sqlx::migrate!("../../infra/migrations")
        .run(pool)
        .await
        .expect("migrations");
}

async fn insert_asset_for_uploader(pool: &PgPool, title: &str, uploader_id: Uuid) -> Asset {
    let asset = Asset::new_pending(title.to_string(), uploader_id);
    asset_repo::insert_asset(pool, &asset)
        .await
        .expect("insert asset");
    asset
}

async fn count_audit_events_for_asset(pool: &PgPool, asset_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE asset_id = $1")
        .bind(asset_id)
        .fetch_one(pool)
        .await
        .expect("audit count")
}

async fn count_audit_events_by_kind(pool: &PgPool, asset_id: Uuid, event_kind: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE asset_id = $1 AND event_kind = $2")
        .bind(asset_id)
        .bind(event_kind)
        .fetch_one(pool)
        .await
        .expect("audit count by kind")
}

fn derive_latest_status(rows: &[dubbridge_domain::consent::ConsentRow]) -> Option<ConsentStatus> {
    rows.iter()
        .max_by_key(|row| row.happened_at)
        .map(|row| row.status.clone())
}
