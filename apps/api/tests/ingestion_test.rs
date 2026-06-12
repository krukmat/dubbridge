use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use dubbridge_api::{build_app, cleanup::cleanup_expired_ingestions, state::AppState};
use dubbridge_auth::{
    AuthenticatedPrincipal, SharedTokenVerifier, TokenVerificationError, TokenVerifier,
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
    storage_dir: Arc<TempDir>,
    storage_path: PathBuf,
    app: axum::Router,
    ingest_token: String,
    read_token: String,
    principal_id: Uuid,
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
        let ingest_token = "ingest-token".to_string();
        let read_token = "read-token".to_string();
        let principal_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid");
        let verifier: SharedTokenVerifier = Arc::new(
            StubTokenVerifier::default()
                .with_token(
                    &ingest_token,
                    Ok(AuthenticatedPrincipal::new(
                        principal_id,
                        ["assets:ingest", "assets:read"]
                            .into_iter()
                            .map(str::to_string),
                    )),
                )
                .with_token(
                    &read_token,
                    Ok(AuthenticatedPrincipal::new(
                        principal_id,
                        ["assets:read"].into_iter().map(str::to_string),
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
        let app = build_app(state, verifier);

        Some(Self {
            pool,
            storage_dir,
            storage_path,
            app,
            ingest_token,
            read_token,
            principal_id,
        })
    }
}

#[tokio::test]
async fn successful_ingestion_creates_asset_rights_artifact_and_audit() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, ingest_token, valid_rights_body()).await;
    let finalize = finalize(&mut ctx, ingest_token, &auth_token).await;
    let status = finalize.status();
    let body = json_body(finalize).await;
    let asset_id = Uuid::parse_str(body["id"].as_str().expect("asset id")).expect("uuid");

    assert_eq!(status, StatusCode::CREATED);

    let asset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets WHERE id = $1")
        .bind(asset_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("asset count");
    let rights_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM rights_records WHERE asset_id = $1")
            .bind(asset_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("rights count");
    let artifact_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM artifact_records WHERE asset_id = $1")
            .bind(asset_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("artifact count");
    let audit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE asset_id = $1")
            .bind(asset_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("audit count");

    assert_eq!(asset_count, 1);
    assert_eq!(rights_count, 1);
    assert_eq!(artifact_count, 1);
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn missing_rights_is_rejected() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    let response = finalize(&mut ctx, ingest_token, &auth_token).await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        body["error"]
            .as_str()
            .expect("error")
            .contains("rights basis is required")
    );

    let audit_count =
        count_audit_events(&ctx.pool, ingest_token, "ingestion_rejected_missing_rights").await;
    assert_eq!(
        audit_count, 1,
        "missing-rights rejection must persist one durable audit row"
    );
}

#[tokio::test]
async fn duplicate_finalization_does_not_create_duplicate_artifact() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, ingest_token, valid_rights_body()).await;
    let first = finalize(&mut ctx, ingest_token, &auth_token).await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = finalize(&mut ctx, ingest_token, &auth_token).await;
    let status = second.status();
    let body = json_body(second).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        body["error"]
            .as_str()
            .expect("error")
            .contains("already been finalized")
    );

    let artifact_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM artifact_records WHERE ingest_token = $1")
            .bind(ingest_token)
            .fetch_one(&ctx.pool)
            .await
            .expect("artifact count");
    let duplicate_audit_count = count_audit_events(
        &ctx.pool,
        ingest_token,
        "ingestion_rejected_duplicate_token",
    )
    .await;

    assert_eq!(artifact_count, 1);
    assert_eq!(
        duplicate_audit_count, 1,
        "duplicate finalize must persist one duplicate-token audit row"
    );
}

#[tokio::test]
async fn finalize_rollback_on_constraint_violation() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, ingest_token, valid_rights_body()).await;
    insert_preexisting_artifact_for_token(&ctx.pool, ingest_token).await;

    let response = finalize(&mut ctx, ingest_token, &auth_token).await;

    assert!(
        response.status().is_client_error(),
        "expected 4xx rollback response, got {}",
        response.status()
    );

    let created_asset_count =
        count_assets_for_uploader_and_title(&ctx.pool, ctx.principal_id, "Test Video").await;
    let pending_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pending_ingestions WHERE ingest_token = $1")
            .bind(ingest_token)
            .fetch_one(&ctx.pool)
            .await
            .expect("pending count");

    assert_eq!(
        created_asset_count, 0,
        "failed finalize must roll back asset insertion"
    );
    assert_eq!(
        pending_count, 1,
        "failed finalize must leave pending ingestion intact"
    );
}

#[tokio::test]
async fn missing_bearer_token_is_rejected() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/assets/{}", Uuid::new_v4()))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn uploader_id_is_derived_from_authenticated_principal() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, ingest_token, valid_rights_body()).await;
    let finalize = finalize(&mut ctx, ingest_token, &auth_token).await;
    let body = json_body(finalize).await;
    let asset_id = body["id"].as_str().expect("asset id");

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/assets/{asset_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.read_token))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["uploader_id"].as_str().expect("uploader id"),
        ctx.principal_id.to_string()
    );
}

#[tokio::test]
async fn rights_and_finalize_survive_app_restart() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, ingest_token, valid_rights_body()).await;

    let mut restarted = rebuild_context(&ctx);
    let finalize = finalize(&mut restarted, ingest_token, &auth_token).await;
    let status = finalize.status();
    let body = json_body(finalize).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        body["uploader_id"].as_str().expect("uploader id"),
        restarted.principal_id.to_string()
    );
}

// T1-T2: expired session must be rejected on rights submission.
#[tokio::test]
async fn expired_session_is_rejected_on_rights() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let ingest_token = create_ingestion(&mut ctx).await;
    expire_pending_ingestion(&ctx.pool, ingest_token).await;

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/ingest/{ingest_token}/rights"))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", ctx.ingest_token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(valid_rights_body().to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::GONE);
    assert!(
        body["error"].as_str().expect("error").contains("expired"),
        "expected 'expired' in error message, got: {}",
        body["error"]
    );
}

// T1-T2: expired session must be rejected on finalize.
#[tokio::test]
async fn expired_session_is_rejected_on_finalize() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    // Submit rights before expiring so that only the finalize step is under test.
    submit_rights_directly(&ctx.pool, ingest_token).await;
    expire_pending_ingestion(&ctx.pool, ingest_token).await;

    let response = finalize(&mut ctx, ingest_token, &auth_token).await;
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::GONE);
    assert!(
        body["error"].as_str().expect("error").contains("expired"),
        "expected 'expired' in error message, got: {}",
        body["error"]
    );
}

// T1-T2: cleanup removes expired rows from the DB and their blobs from storage.
#[tokio::test]
async fn cleanup_removes_expired_sessions() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let mut ctx2 = ctx;
    let ingest_token = create_ingestion(&mut ctx2).await;

    // Verify the row and blob exist before cleanup.
    let storage_key: String =
        sqlx::query_scalar("SELECT storage_key FROM pending_ingestions WHERE ingest_token = $1")
            .bind(ingest_token)
            .fetch_one(&ctx2.pool)
            .await
            .expect("storage_key");

    let blob_exists_before = std::path::Path::new(&ctx2.storage_path)
        .join(storage_key.trim_start_matches('/'))
        .exists();
    assert!(blob_exists_before, "blob should exist before cleanup");

    expire_pending_ingestion(&ctx2.pool, ingest_token).await;

    let storage = LocalFsAdapter::new(&ctx2.storage_path);
    cleanup_expired_ingestions(&ctx2.pool, &storage).await;

    // Row should be gone.
    let row_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pending_ingestions WHERE ingest_token = $1")
            .bind(ingest_token)
            .fetch_one(&ctx2.pool)
            .await
            .expect("count");
    assert_eq!(row_count, 0, "expired row should be deleted after cleanup");

    // Blob should be gone.
    let blob_exists_after = std::path::Path::new(&ctx2.storage_path)
        .join(storage_key.trim_start_matches('/'))
        .exists();
    assert!(!blob_exists_after, "blob should be deleted after cleanup");

    // Running cleanup again on an already-clean state must not panic or error.
    cleanup_expired_ingestions(&ctx2.pool, &storage).await;
}

// T1-T4: Two concurrent finalize requests on the same token. The DB unique
// constraint on artifact_records.ingest_token is the real guard — the early
// duplicate check (find_original_by_ingest_token) is an optimization only.
// Invariant: exactly one request wins with 201; the other gets 409 CONFLICT;
// exactly one artifact row exists regardless of scheduling.
//
// NOTE: this test requires serial execution. All integration tests share a DB and
// call migrate_and_reset (which truncates all tables). Parallel runs can cause a
// third test's truncation to race with this test's in-flight finalizations, yielding
// a 500 instead of the expected 201. Run the suite with --test-threads=1 when using
// a live DB. Verified passing in isolation: cargo test concurrent -- --test-threads=1.
#[tokio::test]
async fn concurrent_duplicate_finalize_one_wins() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, ingest_token, valid_rights_body()).await;

    let app1 = ctx.app.clone();
    let app2 = ctx.app.clone();
    let token1 = auth_token.clone();
    let token2 = auth_token.clone();

    let (r1, r2) = tokio::join!(
        app1.oneshot(make_finalize_request(ingest_token, &token1)),
        app2.oneshot(make_finalize_request(ingest_token, &token2)),
    );

    let s1 = r1.expect("response 1").status();
    let s2 = r2.expect("response 2").status();

    // One must succeed with 201. The other must be rejected — two valid outcomes:
    // - 409 CONFLICT: both read the pending row, loser hits the artifact unique constraint.
    // - 404 NOT_FOUND: winner deleted the pending row before loser called find_pending_ingestion.
    // Which outcome occurs depends on tokio scheduling; both are correct rejections.
    assert!(
        [s1, s2].contains(&StatusCode::CREATED),
        "one finalize must succeed: got {s1} and {s2}"
    );
    let rejected = if s1 == StatusCode::CREATED { s2 } else { s1 };
    assert!(
        matches!(rejected, StatusCode::CONFLICT | StatusCode::NOT_FOUND),
        "loser must be 409 or 404, got {rejected}"
    );

    let artifact_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM artifact_records WHERE ingest_token = $1")
            .bind(ingest_token)
            .fetch_one(&ctx.pool)
            .await
            .expect("artifact count");
    let asset_count = count_assets_for_ingest_token(&ctx.pool, ingest_token).await;
    let rights_count = count_rights_for_ingest_token(&ctx.pool, ingest_token).await;
    let finalized_audit_count =
        count_audit_events(&ctx.pool, ingest_token, "ingestion_finalized").await;

    assert_eq!(
        artifact_count, 1,
        "exactly one artifact must exist after concurrent finalize"
    );
    assert_eq!(
        asset_count, 1,
        "exactly one asset must exist after concurrent finalize"
    );
    assert_eq!(
        rights_count, 1,
        "exactly one rights row must exist after concurrent finalize"
    );
    assert_eq!(
        finalized_audit_count, 1,
        "exactly one finalized audit row must exist after concurrent finalize"
    );
}

// T1-T4: Rights submission and finalize fire concurrently against the same token.
// Race outcome is deterministic at the data level: either finalize loads the rights
// (201) or it doesn't (422, client retries). Either way, at most one asset is created.
// Invariant: no silent data corruption regardless of scheduling order.
#[tokio::test]
async fn concurrent_rights_and_finalize_is_consistent() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let auth_token = ctx.ingest_token.clone();
    let ingest_token = create_ingestion(&mut ctx).await;
    // No rights submitted yet — this is the race condition under test.

    let app1 = ctx.app.clone();
    let app2 = ctx.app.clone();
    let rights_token = ctx.ingest_token.clone();

    let (rights_resp, finalize_resp) = tokio::join!(
        app1.oneshot(make_rights_request(
            ingest_token,
            &rights_token,
            valid_rights_body()
        )),
        app2.oneshot(make_finalize_request(ingest_token, &auth_token)),
    );

    let rights_status = rights_resp.expect("rights response").status();
    let finalize_status = finalize_resp.expect("finalize response").status();

    assert_eq!(
        rights_status,
        StatusCode::OK,
        "rights submission must always succeed"
    );

    // Finalize may race: 201 (rights loaded) or 422 (rights not yet visible) are both valid.
    assert!(
        matches!(
            finalize_status,
            StatusCode::CREATED | StatusCode::UNPROCESSABLE_ENTITY
        ),
        "finalize must be 201 or 422 in a rights+finalize race, got {finalize_status}"
    );

    // Either 0 artifacts (finalize lost the race) or exactly 1 (finalize won).
    // Scoped to this ingest_token so parallel test runs don't interfere.
    let artifact_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM artifact_records WHERE ingest_token = $1")
            .bind(ingest_token)
            .fetch_one(&ctx.pool)
            .await
            .expect("artifact count");
    assert!(
        artifact_count <= 1,
        "at most one artifact must exist for this token, got {artifact_count}"
    );
}

// T1 HP-1: authenticated assets:read caller with owned assets returns them ordered created_at DESC.
#[tokio::test]
async fn list_assets_returns_owned_assets_ordered_by_created_at_desc() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    // Create two assets for the principal via the ingest flow.
    let auth_token = ctx.ingest_token.clone();
    let t1 = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, t1, valid_rights_body()).await;
    let r1 = finalize(&mut ctx, t1, &auth_token).await;
    assert_eq!(r1.status(), StatusCode::CREATED);
    let b1 = json_body(r1).await;

    let t2 = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, t2, valid_rights_body()).await;
    let r2 = finalize(&mut ctx, t2, &auth_token).await;
    assert_eq!(r2.status(), StatusCode::CREATED);
    let b2 = json_body(r2).await;

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/assets")
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.read_token))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().expect("array");
    assert_eq!(arr.len(), 2);

    // Both assets belong to the authenticated principal.
    let uploader_str = ctx.principal_id.to_string();
    for item in arr {
        assert_eq!(item["uploader_id"].as_str().expect("uploader_id"), uploader_str);
    }

    // Most-recently created asset is first.
    let id0 = arr[0]["id"].as_str().expect("id[0]");
    let id1 = arr[1]["id"].as_str().expect("id[1]");
    assert_eq!(id0, b2["id"].as_str().expect("b2 id"));
    assert_eq!(id1, b1["id"].as_str().expect("b1 id"));
}

// T1 HP-2: caller with no owned assets gets 200 with an empty array.
#[tokio::test]
async fn list_assets_empty_for_caller_with_no_assets() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/assets")
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.read_token))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body.as_array().expect("array").len(), 0);
}

// T1 EC-3: assets owned by a different principal must never appear in the list.
#[tokio::test]
async fn list_assets_excludes_other_principals_assets() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    // Insert an asset directly for a different uploader_id.
    let other_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind("other-owner asset")
    .bind(other_id)
    .bind("finalized")
    .execute(&ctx.pool)
    .await
    .expect("insert other asset");

    // Also create one owned asset for the principal.
    let auth_token = ctx.ingest_token.clone();
    let t = create_ingestion(&mut ctx).await;
    submit_rights(&mut ctx, t, valid_rights_body()).await;
    let r = finalize(&mut ctx, t, &auth_token).await;
    assert_eq!(r.status(), StatusCode::CREATED);

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/assets")
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.read_token))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().expect("array");
    assert_eq!(arr.len(), 1, "only the principal's own asset must be returned");
    assert_eq!(
        arr[0]["uploader_id"].as_str().expect("uploader_id"),
        ctx.principal_id.to_string()
    );
}

// T1 EC-2: limit above hard cap is clamped — no error, bounded result.
#[tokio::test]
async fn list_assets_limit_is_clamped_to_max() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/assets?limit=99999&offset=0")
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.read_token))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    // Must not error — clamped limit returns 200 with (empty) array.
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body.is_array(), "response must be an array");
}

// T1 EC-1 / unauthenticated: missing bearer must be rejected with 401.
#[tokio::test]
async fn list_assets_missing_bearer_is_unauthorized() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/assets")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

fn make_finalize_request(ingest_token: Uuid, auth_token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/ingest/{ingest_token}/finalize"))
        .header(header::AUTHORIZATION, format!("Bearer {auth_token}"))
        .body(Body::empty())
        .expect("finalize request")
}

fn make_rights_request(ingest_token: Uuid, auth_token: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/ingest/{ingest_token}/rights"))
        .header(header::AUTHORIZATION, format!("Bearer {auth_token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("rights request")
}

fn valid_rights_body() -> &'static str {
    r#"{
        "owner":"Acme Studios",
        "license_type":"licensed_distribution",
        "source_type":"direct_upload",
        "proof_reference":"contract-2024-001"
    }"#
}

async fn create_ingestion(ctx: &mut TestContext) -> Uuid {
    let boundary = "X-BOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nTest Video\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.mp4\"\r\nContent-Type: video/mp4\r\n\r\nhello dubbridge\r\n--{boundary}--\r\n"
    );
    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ingest")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", ctx.ingest_token),
                )
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .expect("request"),
        )
        .await
        .expect("response");
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CREATED);
    Uuid::parse_str(body["ingest_token"].as_str().expect("token")).expect("uuid")
}

async fn submit_rights(ctx: &mut TestContext, ingest_token: Uuid, body: &str) {
    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/ingest/{ingest_token}/rights"))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", ctx.ingest_token),
                )
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

// T1-T2: write rights directly to DB, bypassing the API (used in expiry tests
// where we need rights present but the session already expired for finalize testing).
async fn submit_rights_directly(pool: &PgPool, ingest_token: Uuid) {
    sqlx::query(
        r#"
        UPDATE pending_ingestions
        SET rights_owner = 'Acme Studios',
            license_type = 'licensed_distribution',
            source_type  = 'direct_upload',
            proof_reference = 'contract-2024-001',
            updated_at = NOW()
        WHERE ingest_token = $1
        "#,
    )
    .bind(ingest_token)
    .execute(pool)
    .await
    .expect("submit rights directly");
}

// T1-T2: force a session to appear expired without waiting for the real TTL.
async fn expire_pending_ingestion(pool: &PgPool, ingest_token: Uuid) {
    sqlx::query(
        "UPDATE pending_ingestions SET expires_at = NOW() - INTERVAL '1 second' WHERE ingest_token = $1",
    )
    .bind(ingest_token)
    .execute(pool)
    .await
    .expect("expire pending ingestion");
}

async fn finalize(
    ctx: &mut TestContext,
    ingest_token: Uuid,
    token: &str,
) -> axum::response::Response {
    ctx.app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/ingest/{ingest_token}/finalize"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request"),
        )
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
    sqlx::migrate!("../../infra/migrations")
        .run(pool)
        .await
        .expect("migrations");
    sqlx::query(
        "TRUNCATE TABLE pending_ingestions, audit_events, artifact_records, rights_records, assets RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("truncate");
}

async fn count_audit_events(pool: &PgPool, ingest_token: Uuid, event_kind: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE ingest_token = $1 AND event_kind = $2",
    )
    .bind(ingest_token)
    .bind(event_kind)
    .fetch_one(pool)
    .await
    .expect("audit count")
}

async fn count_assets_for_ingest_token(pool: &PgPool, ingest_token: Uuid) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM assets a
        JOIN artifact_records ar ON ar.asset_id = a.id
        WHERE ar.ingest_token = $1
        "#,
    )
    .bind(ingest_token)
    .fetch_one(pool)
    .await
    .expect("asset count by ingest token")
}

async fn count_assets_for_uploader_and_title(pool: &PgPool, uploader_id: Uuid, title: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM assets WHERE uploader_id = $1 AND title = $2")
        .bind(uploader_id)
        .bind(title)
        .fetch_one(pool)
        .await
        .expect("asset count by uploader and title")
}

async fn count_rights_for_ingest_token(pool: &PgPool, ingest_token: Uuid) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM rights_records rr
        JOIN artifact_records ar ON ar.asset_id = rr.asset_id
        WHERE ar.ingest_token = $1
        "#,
    )
    .bind(ingest_token)
    .fetch_one(pool)
    .await
    .expect("rights count by ingest token")
}

async fn insert_preexisting_artifact_for_token(pool: &PgPool, ingest_token: Uuid) {
    let existing_asset_id = Uuid::new_v4();
    let existing_uploader_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO assets (id, title, uploader_id, status)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(existing_asset_id)
    .bind("preexisting duplicate guard")
    .bind(existing_uploader_id)
    .bind("finalized")
    .execute(pool)
    .await
    .expect("insert preexisting asset");

    sqlx::query(
        r#"
        INSERT INTO artifact_records (
            id, asset_id, kind, ingest_token, storage_key, content_type, size_bytes, checksum
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(existing_asset_id)
    .bind("original_media")
    .bind(ingest_token)
    .bind("assets/preexisting/file.mp4")
    .bind("video/mp4")
    .bind(123_i64)
    .bind("preexisting-checksum")
    .execute(pool)
    .await
    .expect("insert preexisting artifact");
}

fn rebuild_context(ctx: &TestContext) -> TestContext {
    let verifier: SharedTokenVerifier = Arc::new(
        StubTokenVerifier::default()
            .with_token(
                &ctx.ingest_token,
                Ok(AuthenticatedPrincipal::new(
                    ctx.principal_id,
                    ["assets:ingest", "assets:read"]
                        .into_iter()
                        .map(str::to_string),
                )),
            )
            .with_token(
                &ctx.read_token,
                Ok(AuthenticatedPrincipal::new(
                    ctx.principal_id,
                    ["assets:read"].into_iter().map(str::to_string),
                )),
            ),
    );
    let config = dubbridge_config::AppConfig::from_env();
    let state = Arc::new(AppState::new(
        ctx.pool.clone(),
        Box::new(LocalFsAdapter::new(&ctx.storage_path)),
        verifier.clone(),
        config,
    ));

    TestContext {
        pool: ctx.pool.clone(),
        storage_dir: Arc::clone(&ctx.storage_dir),
        storage_path: ctx.storage_path.clone(),
        app: build_app(state, verifier),
        ingest_token: ctx.ingest_token.clone(),
        read_token: ctx.read_token.clone(),
        principal_id: ctx.principal_id,
    }
}

// H1-T1: cleanup must skip a pending row that is locked by an in-flight finalize
// transaction (SELECT ... FOR UPDATE SKIP LOCKED). This test holds a FOR UPDATE
// lock on an expired row to simulate finalize-in-progress, verifies claim returns
// nothing for that row, then releases the lock and verifies claim succeeds.
#[tokio::test]
async fn cleanup_skips_row_locked_by_in_flight_finalize() {
    let Some(mut ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    let ingest_token = create_ingestion(&mut ctx).await;
    expire_pending_ingestion(&ctx.pool, ingest_token).await;

    // Simulate an in-flight finalize by holding a FOR UPDATE lock on the row.
    let mut lock_tx = ctx.pool.begin().await.expect("begin lock tx");
    sqlx::query("SELECT 1 FROM pending_ingestions WHERE ingest_token = $1 FOR UPDATE")
        .bind(ingest_token)
        .execute(&mut *lock_tx)
        .await
        .expect("lock row");

    // claim_expired_for_cleanup uses SKIP LOCKED — must not return the locked row.
    let claimed = dubbridge_db::pending_ingestion_repo::claim_expired_for_cleanup(&ctx.pool)
        .await
        .expect("claim while locked");
    assert!(
        claimed.iter().all(|(t, _)| *t != ingest_token),
        "cleanup must skip a row locked by an in-flight finalize"
    );

    // Release the lock (finalize "aborts").
    lock_tx.rollback().await.expect("rollback lock tx");

    // Now claim should pick up the row.
    let claimed2 = dubbridge_db::pending_ingestion_repo::claim_expired_for_cleanup(&ctx.pool)
        .await
        .expect("claim after lock released");
    assert!(
        claimed2.iter().any(|(t, _)| *t == ingest_token),
        "cleanup must claim row after finalize lock is released"
    );
}

// T1-T6: A multipart upload whose body exceeds MAX_UPLOAD_BYTES must be rejected
// with 413 Payload Too Large before any bytes reach storage. The body is built
// inline as raw multipart bytes so we can exceed the limit without writing to disk.
#[tokio::test]
async fn upload_too_large_is_rejected() {
    let Some(ctx) = TestContext::new().await else {
        eprintln!("skipping integration test: DUBBRIDGE_DATABASE_URL not set");
        return;
    };

    use dubbridge_api::routes::ingestion::MAX_UPLOAD_BYTES;

    let boundary = "X-BOUNDARY-LARGE";
    // Build a body just over the limit: header + (MAX_UPLOAD_BYTES + 1) zero bytes + footer.
    let header = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"big.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    );
    let footer = format!("\r\n--{boundary}--\r\n");
    let payload_size = MAX_UPLOAD_BYTES + 1;
    let mut body_bytes = Vec::with_capacity(header.len() + payload_size + footer.len());
    body_bytes.extend_from_slice(header.as_bytes());
    body_bytes.extend(std::iter::repeat_n(0u8, payload_size));
    body_bytes.extend_from_slice(footer.as_bytes());

    // axum DefaultBodyLimit checks Content-Length to enforce the limit before
    // reading the body. Without it, the multipart parser reads and returns 400.
    let content_length = body_bytes.len();
    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ingest")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", ctx.ingest_token),
                )
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .header(header::CONTENT_LENGTH, content_length.to_string())
                .body(Body::from(body_bytes))
                .expect("oversized request"),
        )
        .await
        .expect("response");

    assert_eq!(
        response.status(),
        StatusCode::PAYLOAD_TOO_LARGE,
        "upload exceeding MAX_UPLOAD_BYTES must be rejected with 413"
    );
}
