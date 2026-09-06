use dubbridge_db::{
    create_pool,
    error::DbError,
    p2p_publication_repo::{
        ensure_publication_with_outbox, get_outbox_for_publication, get_publication,
        list_outstanding_publication_work, record_external_confirmation,
        transition_publication_state,
    },
};
use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId, PublicationState},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../infra/migrations");

async fn test_pool() -> PgPool {
    let database_url = std::env::var("DUBBRIDGE_DATABASE_URL")
        .expect("DUBBRIDGE_DATABASE_URL must be set for DB integration tests");
    let pool = create_pool(&database_url).await.expect("connect test database");
    MIGRATOR.run(&pool).await.expect("run migrations");
    pool
}

async fn insert_asset(pool: &PgPool) -> AssetId {
    let asset_id = AssetId(Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO assets (id, title, uploader_id, status)
        VALUES ($1, 'P2P T1 test asset', $2, 'finalized')
        "#,
    )
    .bind(asset_id.0)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("insert asset fixture");
    asset_id
}

#[tokio::test]
async fn hp_t1_atomic_create_restart_reread_and_same_lineage_idempotency() {
    let pool = test_pool().await;
    let asset_id = insert_asset(&pool).await;
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();
    let outbox_id = Uuid::new_v4();

    let created = ensure_publication_with_outbox(
        &pool,
        asset_id,
        publication_id,
        lineage_id,
        outbox_id,
    )
    .await
    .expect("create publication and outbox");
    assert!(created.created);
    assert_eq!(created.publication.state, PublicationState::Building);
    assert_eq!(created.publication.lineage_id, lineage_id);
    assert_eq!(created.outbox.id, outbox_id);
    assert_eq!(created.outbox.delivery_state, "pending");

    drop(pool);
    let restarted_pool = test_pool().await;
    let reread = get_publication(&restarted_pool, publication_id)
        .await
        .expect("re-read publication")
        .expect("publication persists");
    let reread_outbox = get_outbox_for_publication(&restarted_pool, publication_id)
        .await
        .expect("re-read outbox")
        .expect("outbox persists");
    assert_eq!(reread.lineage_id, lineage_id);
    assert_eq!(reread_outbox.id, outbox_id);

    let duplicate = ensure_publication_with_outbox(
        &restarted_pool,
        asset_id,
        P2pPublicationId::new(),
        lineage_id,
        Uuid::new_v4(),
    )
    .await
    .expect("same-lineage ensure is idempotent");
    assert!(!duplicate.created);
    assert_eq!(duplicate.publication.id, publication_id);
    assert_eq!(duplicate.outbox.id, outbox_id);
}

#[tokio::test]
async fn ec_t1_conflicting_lineage_and_partial_atomic_create_fail_closed() {
    let pool = test_pool().await;
    let asset_id = insert_asset(&pool).await;
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();

    ensure_publication_with_outbox(
        &pool,
        asset_id,
        publication_id,
        lineage_id,
        Uuid::new_v4(),
    )
    .await
    .expect("initial ensure");

    let conflict = ensure_publication_with_outbox(
        &pool,
        asset_id,
        P2pPublicationId::new(),
        K1LineageId::new(),
        Uuid::new_v4(),
    )
    .await;
    assert!(matches!(conflict, Err(DbError::Conflict)));

    let missing_asset = AssetId(Uuid::new_v4());
    let orphan_publication = P2pPublicationId::new();
    let orphan_outbox = Uuid::new_v4();
    assert!(ensure_publication_with_outbox(
        &pool,
        missing_asset,
        orphan_publication,
        K1LineageId::new(),
        orphan_outbox,
    )
    .await
    .is_err());

    let publication_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM p2p_publications WHERE id = $1")
        .bind(orphan_publication.0)
        .fetch_one(&pool)
        .await
        .expect("count orphan publication");
    let outbox_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM p2p_publication_outbox WHERE id = $1")
        .bind(orphan_outbox)
        .fetch_one(&pool)
        .await
        .expect("count orphan outbox");
    assert_eq!(publication_count, 0);
    assert_eq!(outbox_count, 0);
}

#[tokio::test]
async fn hp_ec_t1_outstanding_read_and_ready_guard_require_same_lineage_confirmation() {
    let pool = test_pool().await;
    let asset_id = insert_asset(&pool).await;
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();

    ensure_publication_with_outbox(
        &pool,
        asset_id,
        publication_id,
        lineage_id,
        Uuid::new_v4(),
    )
    .await
    .expect("create publication");

    assert!(transition_publication_state(
        &pool,
        publication_id,
        PublicationState::Ready,
        None,
    )
    .await
    .is_err());

    transition_publication_state(&pool, publication_id, PublicationState::PublishPending, None)
        .await
        .expect("building -> publish_pending");
    let work = list_outstanding_publication_work(&pool, 10)
        .await
        .expect("list outstanding work");
    assert!(work.iter().any(|item| item.publication.id == publication_id));

    transition_publication_state(&pool, publication_id, PublicationState::Publishing, None)
        .await
        .expect("publish_pending -> publishing");
    assert!(record_external_confirmation(
        &pool,
        publication_id,
        K1LineageId::new(),
        "hyperdrive:wrong-lineage",
        OffsetDateTime::now_utc(),
    )
    .await
    .is_err());

    record_external_confirmation(
        &pool,
        publication_id,
        lineage_id,
        "hyperdrive:stable-publication",
        OffsetDateTime::now_utc(),
    )
    .await
    .expect("persist same-lineage confirmation");
    let ready = transition_publication_state(&pool, publication_id, PublicationState::Ready, None)
        .await
        .expect("confirmed publication -> ready");
    assert_eq!(ready.state, PublicationState::Ready);
    assert_eq!(ready.confirmed_lineage_id, Some(lineage_id));

    assert!(transition_publication_state(
        &pool,
        publication_id,
        PublicationState::Reconciling,
        None,
    )
    .await
    .is_err());
    let work_after_ready = list_outstanding_publication_work(&pool, 1_000)
        .await
        .expect("list after ready");
    assert!(!work_after_ready
        .iter()
        .any(|item| item.publication.id == publication_id));
}

#[tokio::test]
async fn ec_t1_schema_contains_no_forbidden_secret_fields() {
    let pool = test_pool().await;
    let columns: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT column_name
          FROM information_schema.columns
         WHERE table_schema = 'public'
           AND table_name IN ('p2p_publications', 'p2p_publication_outbox')
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("inspect P2P schema");

    let forbidden = [
        "plaintext_ck",
        "content_key",
        "kek",
        "invite_token",
        "mtls_private_key",
        "jwt_signing_secret",
    ];
    for field in forbidden {
        assert!(!columns.iter().any(|column| column == field), "forbidden secret field: {field}");
    }
}
