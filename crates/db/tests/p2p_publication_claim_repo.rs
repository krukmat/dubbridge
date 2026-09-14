use dubbridge_db::{
    create_pool,
    error::DbError,
    p2p_publication_claim_repo::{
        claim_next_publication_work, release_publication_claim,
    },
    p2p_publication_repo::{
        ensure_publication_with_outbox, transition_publication_state,
    },
};
use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId, PublicationState},
};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../infra/migrations");

async fn test_pool() -> PgPool {
    let database_url = std::env::var("DUBBRIDGE_DATABASE_URL")
        .expect("DUBBRIDGE_DATABASE_URL must be set for DB integration tests");
    let pool = create_pool(&database_url)
        .await
        .expect("connect test database");
    MIGRATOR.run(&pool).await.expect("run migrations");
    pool
}

async fn insert_asset(pool: &PgPool) -> AssetId {
    let asset_id = AssetId(Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO assets (id, title, uploader_id, status)
        VALUES ($1, 'P2P T4b claim test asset', $2, 'finalized')
        "#,
    )
    .bind(asset_id.0)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("insert asset fixture");
    asset_id
}

async fn create_claimable_work(pool: &PgPool) -> (P2pPublicationId, K1LineageId, Uuid) {
    let asset_id = insert_asset(pool).await;
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();
    let outbox_id = Uuid::new_v4();

    ensure_publication_with_outbox(pool, asset_id, publication_id, lineage_id, outbox_id)
        .await
        .expect("create publication and outbox");
    transition_publication_state(
        pool,
        publication_id,
        PublicationState::PublishPending,
        None,
    )
    .await
    .expect("building -> publish_pending");

    (publication_id, lineage_id, outbox_id)
}

#[tokio::test]
async fn hp_t4b_claim_is_single_owner_and_increments_attempt_count() {
    let pool = test_pool().await;
    let (publication_id, lineage_id, outbox_id) = create_claimable_work(&pool).await;
    let claim_token = Uuid::new_v4();
    let lease_expires_at = OffsetDateTime::now_utc() + Duration::minutes(5);

    let claim = claim_next_publication_work(&pool, claim_token, lease_expires_at)
        .await
        .expect("claim query")
        .expect("claimable work");

    assert_eq!(claim.outbox_id, outbox_id);
    assert_eq!(claim.publication_id, publication_id);
    assert_eq!(claim.lineage_id, lineage_id);
    assert_eq!(claim.claim_token, claim_token);
    assert_eq!(claim.attempt_count, 1);
    assert!(claim.lease_expires_at > claim.claimed_at);

    let second = claim_next_publication_work(
        &pool,
        Uuid::new_v4(),
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await
    .expect("second claim query");
    assert!(second.is_none(), "a live lease must not be stolen");
}

#[tokio::test]
async fn hp_t4b_expired_lease_is_reclaimable_with_new_owner() {
    let pool = test_pool().await;
    let (_, _, outbox_id) = create_claimable_work(&pool).await;
    let first_token = Uuid::new_v4();

    claim_next_publication_work(
        &pool,
        first_token,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await
    .expect("first claim query")
    .expect("first claim");

    sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET claimed_at = now() - interval '2 minutes',
               lease_expires_at = now() - interval '1 minute'
         WHERE id = $1
           AND claim_token = $2
        "#,
    )
    .bind(outbox_id)
    .bind(first_token)
    .execute(&pool)
    .await
    .expect("expire first lease");

    let second_token = Uuid::new_v4();
    let reclaimed = claim_next_publication_work(
        &pool,
        second_token,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await
    .expect("reclaim query")
    .expect("expired claim is reclaimable");

    assert_eq!(reclaimed.outbox_id, outbox_id);
    assert_eq!(reclaimed.claim_token, second_token);
    assert_eq!(reclaimed.attempt_count, 2);
}

#[tokio::test]
async fn ec_t4b_foreign_release_fails_closed_and_owner_release_requeues() {
    let pool = test_pool().await;
    let (_, _, outbox_id) = create_claimable_work(&pool).await;
    let claim_token = Uuid::new_v4();

    claim_next_publication_work(
        &pool,
        claim_token,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await
    .expect("claim query")
    .expect("claimable work");

    let foreign = release_publication_claim(
        &pool,
        outbox_id,
        Uuid::new_v4(),
        OffsetDateTime::now_utc(),
        Some("transient dispatch failure"),
    )
    .await;
    assert!(matches!(foreign, Err(DbError::Conflict)));

    release_publication_claim(
        &pool,
        outbox_id,
        claim_token,
        OffsetDateTime::now_utc(),
        Some("transient dispatch failure"),
    )
    .await
    .expect("owner releases claim");

    let row: (String, Option<Uuid>, Option<OffsetDateTime>, Option<String>) = sqlx::query_as(
        r#"
        SELECT delivery_state, claim_token, lease_expires_at, last_error
          FROM p2p_publication_outbox
         WHERE id = $1
        "#,
    )
    .bind(outbox_id)
    .fetch_one(&pool)
    .await
    .expect("read released outbox");

    assert_eq!(row.0, "pending");
    assert_eq!(row.1, None);
    assert_eq!(row.2, None);
    assert_eq!(row.3.as_deref(), Some("transient dispatch failure"));

    let stale_repeat = release_publication_claim(
        &pool,
        outbox_id,
        claim_token,
        OffsetDateTime::now_utc(),
        None,
    )
    .await;
    assert!(matches!(stale_repeat, Err(DbError::Conflict)));
}

#[tokio::test]
async fn ec_t4b_invalid_or_expired_requested_lease_fails_before_db_mutation() {
    let pool = test_pool().await;
    create_claimable_work(&pool).await;

    let nil_token = claim_next_publication_work(
        &pool,
        Uuid::nil(),
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await;
    assert!(matches!(nil_token, Err(DbError::Conflict)));

    let expired = claim_next_publication_work(
        &pool,
        Uuid::new_v4(),
        OffsetDateTime::now_utc() - Duration::seconds(1),
    )
    .await;
    assert!(matches!(expired, Err(DbError::Conflict)));

    let attempt_count: i32 = sqlx::query_scalar(
        "SELECT attempt_count FROM p2p_publication_outbox LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("read attempt count");
    assert_eq!(attempt_count, 0);
}
