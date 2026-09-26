use dubbridge_db::{
    create_pool,
    p2p_publication_claim_repo::{ReadyFinalization, finalize_publication_ready},
    p2p_publication_repo::{
        ensure_publication_with_outbox, record_external_confirmation, record_sealed_k1,
        transition_publication_state,
    },
    p2p_ready_repo::{get_ready_descriptor_by_asset, persist_confirmed_manifest_digest},
    preparation_repo,
};
use dubbridge_domain::{
    artifact::PreparationStatus,
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId, PublicationState},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../infra/migrations");
const DIGEST: &str = "b753ba52473d8b9f1ddc8444d43d6166c6b46eeb1214018e3a33503f56a021b4";
const EXTERNAL_ID: &str = "hyperdrive:t5-ready";

struct Fixture {
    asset_id: AssetId,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    outbox_id: Uuid,
}

async fn test_pool() -> PgPool {
    let database_url = std::env::var("DUBBRIDGE_DATABASE_URL")
        .expect("DUBBRIDGE_DATABASE_URL must be set for DB integration tests");
    let pool = create_pool(&database_url)
        .await
        .expect("connect test database");
    MIGRATOR.run(&pool).await.expect("run migrations");
    pool
}

async fn insert_asset(pool: &PgPool, title: &str) -> AssetId {
    let asset_id = AssetId(Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO assets (id, title, uploader_id, status)
        VALUES ($1, $2, $3, 'finalized')
        "#,
    )
    .bind(asset_id.0)
    .bind(title)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("insert asset fixture");
    asset_id
}

async fn create_publishing_fixture(pool: &PgPool, title: &str) -> Fixture {
    let asset_id = insert_asset(pool, title).await;
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();
    let outbox_id = Uuid::new_v4();

    ensure_publication_with_outbox(pool, asset_id, publication_id, lineage_id, outbox_id)
        .await
        .expect("create publication and outbox");
    record_sealed_k1(
        pool,
        publication_id,
        lineage_id,
        "server-kek",
        1,
        &[7_u8; 12],
        &[9_u8; 48],
    )
    .await
    .expect("seal K1 metadata");
    transition_publication_state(pool, publication_id, PublicationState::PublishPending, None)
        .await
        .expect("building -> publish_pending");
    transition_publication_state(pool, publication_id, PublicationState::Publishing, None)
        .await
        .expect("publish_pending -> publishing");

    Fixture {
        asset_id,
        publication_id,
        lineage_id,
        outbox_id,
    }
}

async fn mark_outbox_delivered(pool: &PgPool, outbox_id: Uuid) {
    sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET delivery_state = 'delivered',
               delivered_at = now(),
               claim_token = NULL,
               lease_expires_at = NULL,
               updated_at = now()
         WHERE id = $1
        "#,
    )
    .bind(outbox_id)
    .execute(pool)
    .await
    .expect("mark outbox delivered");
}

#[tokio::test]
async fn hp_t5d_authoritative_ready_materializes_minimal_descriptor() {
    let pool = test_pool().await;
    let fixture = create_publishing_fixture(&pool, "T5d authoritative ready").await;

    persist_confirmed_manifest_digest(&pool, fixture.publication_id, fixture.lineage_id, DIGEST)
        .await
        .expect("persist manifest evidence");

    let claim_token = Uuid::new_v4();
    sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET delivery_state = 'claimed',
               attempt_count = attempt_count + 1,
               claimed_at = now(),
               claim_token = $2,
               lease_expires_at = now() + interval '5 minutes',
               updated_at = now()
         WHERE id = $1
        "#,
    )
    .bind(fixture.outbox_id)
    .bind(claim_token)
    .execute(&pool)
    .await
    .expect("claim fixture outbox");

    let now = OffsetDateTime::now_utc();
    finalize_publication_ready(
        &pool,
        ReadyFinalization {
            outbox_id: fixture.outbox_id,
            publication_id: fixture.publication_id,
            lineage_id: fixture.lineage_id,
            claim_token,
            external_publication_id: EXTERNAL_ID,
            confirmed_at: now,
            delivered_at: now,
        },
    )
    .await
    .expect("finalize ready");

    let descriptor = get_ready_descriptor_by_asset(&pool, fixture.asset_id)
        .await
        .expect("read ready descriptor")
        .expect("descriptor must exist");

    assert_eq!(descriptor.asset_id, fixture.asset_id);
    assert_eq!(descriptor.publication_id, fixture.publication_id);
    assert_eq!(descriptor.lineage_id, fixture.lineage_id);
    assert_eq!(descriptor.manifest_digest_sha256, DIGEST);
    assert_eq!(descriptor.external_publication_id, EXTERNAL_ID);
    assert_eq!(descriptor.kek_id, "server-kek");
    assert_eq!(descriptor.kek_version, 1);
    assert!(descriptor.ck_wrap_ref.starts_with("p2p-k1-wrap/"));
}

#[tokio::test]
async fn ec_t5d_ready_without_manifest_evidence_is_not_exposed() {
    let pool = test_pool().await;
    let fixture = create_publishing_fixture(&pool, "T5d missing digest").await;
    let now = OffsetDateTime::now_utc();

    record_external_confirmation(
        &pool,
        fixture.publication_id,
        fixture.lineage_id,
        EXTERNAL_ID,
        now,
    )
    .await
    .expect("persist confirmation");
    transition_publication_state(&pool, fixture.publication_id, PublicationState::Ready, None)
        .await
        .expect("publishing -> ready");
    mark_outbox_delivered(&pool, fixture.outbox_id).await;

    assert!(
        get_ready_descriptor_by_asset(&pool, fixture.asset_id)
            .await
            .expect("read descriptor")
            .is_none(),
        "Ready without durable manifest evidence must fail closed"
    );
}

#[tokio::test]
async fn ec_t5d_ready_with_undelivered_outbox_is_not_exposed() {
    let pool = test_pool().await;
    let fixture = create_publishing_fixture(&pool, "T5d undelivered outbox").await;
    let now = OffsetDateTime::now_utc();

    persist_confirmed_manifest_digest(&pool, fixture.publication_id, fixture.lineage_id, DIGEST)
        .await
        .expect("persist manifest evidence");
    record_external_confirmation(
        &pool,
        fixture.publication_id,
        fixture.lineage_id,
        EXTERNAL_ID,
        now,
    )
    .await
    .expect("persist confirmation");
    transition_publication_state(&pool, fixture.publication_id, PublicationState::Ready, None)
        .await
        .expect("publishing -> ready");

    assert!(
        get_ready_descriptor_by_asset(&pool, fixture.asset_id)
            .await
            .expect("read descriptor")
            .is_none(),
        "Ready with an outstanding delivery obligation must not become P2P_READY"
    );
}

#[tokio::test]
async fn ec_t5d_p2_failure_does_not_regress_s120_ready() {
    let pool = test_pool().await;
    let fixture = create_publishing_fixture(&pool, "T5d S120 isolation").await;

    preparation_repo::upsert_preparation_status(
        &pool,
        fixture.asset_id,
        PreparationStatus::Ready,
        None,
    )
    .await
    .expect("set S-120 ready");

    transition_publication_state(
        &pool,
        fixture.publication_id,
        PublicationState::Failed,
        Some("publication_unavailable"),
    )
    .await
    .expect("P2 terminal failure");

    let status = preparation_repo::get_preparation_status(&pool, fixture.asset_id)
        .await
        .expect("read preparation status")
        .expect("preparation row");
    assert_eq!(status.status, PreparationStatus::Ready);
}
