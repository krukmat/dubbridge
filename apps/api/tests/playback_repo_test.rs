// S-125-T2b-iii: integration tests for playback_repo — real DB, no mocks.
use std::env;

use dubbridge_db::{artifact_repo, playback_repo, preparation_repo};
use dubbridge_domain::{
    artifact::{ArtifactKind, ArtifactRecord, DerivedArtifact, PreparationStatus},
    asset::AssetId,
    playback::{GrantPrincipal, PlaybackDenial, PlaybackGrant, PlaybackGrantId, PlaybackScope},
    workspace::{OrgId, ProjectId},
};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
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

async fn insert_org(pool: &PgPool) -> OrgId {
    let org_id = OrgId(Uuid::new_v4());
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id.0)
        .bind("playback-test-org")
        .execute(pool)
        .await
        .expect("insert org");
    org_id
}

async fn insert_asset(pool: &PgPool) -> AssetId {
    let asset_id = AssetId::new();
    sqlx::query("INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, $4)")
        .bind(asset_id.0)
        .bind("playback-test-asset")
        .bind(Uuid::new_v4())
        .bind("finalized")
        .execute(pool)
        .await
        .expect("insert asset");
    asset_id
}

async fn insert_source_artifact(pool: &PgPool, asset_id: AssetId) -> ArtifactRecord {
    let record = ArtifactRecord::new_original(
        asset_id,
        Uuid::new_v4(),
        format!("ingest/{}/source.mp4", asset_id),
        "video/mp4".into(),
        1_000_000,
        "deadbeef".into(),
    );
    artifact_repo::insert_artifact_record(pool, &record)
        .await
        .expect("insert source artifact");
    record
}

async fn insert_hls_manifest(pool: &PgPool, asset_id: AssetId, source_id: Uuid) {
    let manifest = DerivedArtifact::new(
        asset_id,
        source_id,
        ArtifactKind::HlsManifest,
        format!("prepared/{}/index.m3u8", asset_id),
        "application/vnd.apple.mpegurl".into(),
        512,
        "manifestchk".into(),
    );
    preparation_repo::insert_derived_artifact(pool, &manifest)
        .await
        .expect("insert hls manifest");
}

fn make_grant(
    asset_id: AssetId,
    org_id: OrgId,
    issued_at: OffsetDateTime,
    expires_at: OffsetDateTime,
) -> PlaybackGrant {
    PlaybackGrant::new(
        PlaybackGrantId::new(),
        asset_id,
        PlaybackScope::Review,
        GrantPrincipal {
            principal_id: Uuid::new_v4(),
            org_id,
            project_id: ProjectId(Uuid::new_v4()),
        },
        issued_at,
        expires_at,
    )
    .expect("valid grant")
}

// HP-1: issue → get_active → resolve returns HlsManifest artifact.
#[tokio::test]
async fn issue_and_resolve_active_grant_returns_hls_manifest() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let org_id = insert_org(&pool).await;
    let asset_id = insert_asset(&pool).await;
    let source = insert_source_artifact(&pool, asset_id).await;

    preparation_repo::upsert_preparation_status(&pool, asset_id, PreparationStatus::Ready, None)
        .await
        .expect("upsert ready");

    insert_hls_manifest(&pool, asset_id, source.id).await;

    let now = OffsetDateTime::now_utc();
    let grant = make_grant(asset_id, org_id, now, now + Duration::hours(1));
    let grant_id = grant.id;

    playback_repo::issue_grant(&pool, &grant)
        .await
        .expect("issue grant");

    let active = playback_repo::get_active_grant(&pool, grant_id)
        .await
        .expect("get active grant")
        .expect("should be Some");

    assert_eq!(active.id, grant_id);
    assert_eq!(active.asset_id, asset_id);

    let artifact = playback_repo::resolve_grant_target(&pool, grant_id)
        .await
        .expect("resolve target");

    assert_eq!(artifact.kind, ArtifactKind::HlsManifest);
    assert_eq!(artifact.asset_id, asset_id);
    assert_eq!(artifact.parent_artifact_id, source.id);
}

// HP-2: expire_grant → get_active_grant returns None.
#[tokio::test]
async fn expired_grant_is_not_returned_by_get_active() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let org_id = insert_org(&pool).await;
    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;

    let now = OffsetDateTime::now_utc();
    let grant = make_grant(asset_id, org_id, now, now + Duration::hours(1));
    let grant_id = grant.id;

    playback_repo::issue_grant(&pool, &grant)
        .await
        .expect("issue grant");

    playback_repo::expire_grant(&pool, grant_id)
        .await
        .expect("expire grant");

    let result = playback_repo::get_active_grant(&pool, grant_id)
        .await
        .expect("get active grant");

    assert!(result.is_none());
}

// EC-1: preparation status != Ready → resolve returns PlaybackDenial::NotReady.
#[tokio::test]
async fn resolve_denies_when_asset_not_ready() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let org_id = insert_org(&pool).await;
    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;

    preparation_repo::upsert_preparation_status(
        &pool,
        asset_id,
        PreparationStatus::InProgress,
        None,
    )
    .await
    .expect("upsert in_progress");

    let now = OffsetDateTime::now_utc();
    let grant = make_grant(asset_id, org_id, now, now + Duration::hours(1));
    let grant_id = grant.id;

    playback_repo::issue_grant(&pool, &grant)
        .await
        .expect("issue grant");

    let denial = playback_repo::resolve_grant_target(&pool, grant_id)
        .await
        .expect_err("should deny");

    assert_eq!(denial, PlaybackDenial::NotReady);
}

// EC-2: no HLS manifest row → resolve returns PlaybackDenial::MissingManifest.
#[tokio::test]
async fn resolve_denies_when_manifest_missing() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let org_id = insert_org(&pool).await;
    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;

    preparation_repo::upsert_preparation_status(&pool, asset_id, PreparationStatus::Ready, None)
        .await
        .expect("upsert ready");
    // intentionally no HLS manifest row inserted.

    let now = OffsetDateTime::now_utc();
    let grant = make_grant(asset_id, org_id, now, now + Duration::hours(1));
    let grant_id = grant.id;

    playback_repo::issue_grant(&pool, &grant)
        .await
        .expect("issue grant");

    let denial = playback_repo::resolve_grant_target(&pool, grant_id)
        .await
        .expect_err("should deny");

    assert_eq!(denial, PlaybackDenial::MissingManifest);
}

// EC-3: past expires_at (wall-clock) → get_active_grant returns None.
#[tokio::test]
async fn get_active_grant_returns_none_for_wall_clock_expired_grant() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let org_id = insert_org(&pool).await;
    let asset_id = insert_asset(&pool).await;
    let _source = insert_source_artifact(&pool, asset_id).await;

    // issued and expired both in the past (1 second window, already closed).
    let issued = OffsetDateTime::now_utc() - Duration::hours(2);
    let expires = issued + Duration::seconds(1);

    let grant = make_grant(asset_id, org_id, issued, expires);
    let grant_id = grant.id;

    playback_repo::issue_grant(&pool, &grant)
        .await
        .expect("issue grant");

    let result = playback_repo::get_active_grant(&pool, grant_id)
        .await
        .expect("get active grant");

    assert!(result.is_none());
}
