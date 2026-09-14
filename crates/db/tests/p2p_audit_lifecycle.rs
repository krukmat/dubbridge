use dubbridge_db::{
    create_pool,
    p2p_package_seal_repo::persist_sealed_package_evidence,
    p2p_publication_repo::{ensure_publication_with_outbox, record_sealed_k1},
};
use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId},
};
use sqlx::PgPool;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../infra/migrations");
const DIGEST: &str = "b753ba52473d8b9f1ddc8444d43d6166c6b46eeb1214018e3a33503f56a021b4";

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
    let asset_id = AssetId::new();
    sqlx::query(
        "INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, 'P2 audit lifecycle', $2, 'finalized')",
    )
    .bind(asset_id.0)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("insert asset");
    asset_id
}

async fn event_count(pool: &PgPool, publication_id: P2pPublicationId, kind: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE publication_id = $1 AND event_kind = $2",
    )
    .bind(publication_id.0)
    .bind(kind)
    .fetch_one(pool)
    .await
    .expect("audit count")
}

#[tokio::test]
async fn intent_and_lineage_seal_are_correlated_and_idempotent() {
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
    assert_eq!(
        event_count(&pool, publication_id, "p2p_publication_intent_created").await,
        1
    );

    ensure_publication_with_outbox(
        &pool,
        asset_id,
        publication_id,
        lineage_id,
        Uuid::new_v4(),
    )
    .await
    .expect("idempotent publication replay");
    assert_eq!(
        event_count(&pool, publication_id, "p2p_publication_intent_created").await,
        1
    );

    record_sealed_k1(
        &pool,
        publication_id,
        lineage_id,
        "t6-kek",
        1,
        &[7_u8; 12],
        &[9_u8; 48],
    )
    .await
    .expect("persist K1 wrap");
    assert_eq!(event_count(&pool, publication_id, "p2p_lineage_sealed").await, 0);

    persist_sealed_package_evidence(
        &pool,
        publication_id,
        lineage_id,
        DIGEST,
        &publication_id.to_string(),
    )
    .await
    .expect("persist package seal evidence");
    persist_sealed_package_evidence(
        &pool,
        publication_id,
        lineage_id,
        DIGEST,
        &publication_id.to_string(),
    )
    .await
    .expect("idempotent package seal evidence replay");

    let row: (i64, Option<Uuid>, Option<Uuid>, Option<Uuid>, Option<Uuid>) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT,
               min(correlation_id),
               min(publication_id),
               min(lineage_id),
               min(ingest_token)
          FROM audit_events
         WHERE publication_id = $1
           AND event_kind = 'p2p_lineage_sealed'
        "#,
    )
    .bind(publication_id.0)
    .fetch_one(&pool)
    .await
    .expect("read lineage audit");

    assert_eq!(row.0, 1);
    assert_eq!(row.1, Some(publication_id.0));
    assert_eq!(row.2, Some(publication_id.0));
    assert_eq!(row.3, Some(lineage_id.0));
    assert_eq!(row.4, None);
}
