use dubbridge_db::{audit_repo::insert_audit_event, create_pool};
use dubbridge_domain::{
    asset::AssetId,
    audit::{AuditEvent, AuditEventKind},
};
use sqlx::PgPool;
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

async fn insert_asset_and_publication(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let asset_id = Uuid::new_v4();
    let publication_id = Uuid::new_v4();
    let lineage_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, 'P3 audit shape', $2, 'finalized')",
    )
    .bind(asset_id)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("insert asset");

    sqlx::query(
        "INSERT INTO p2p_publications (id, asset_id, lineage_id, state) VALUES ($1, $2, $3, 'building')",
    )
    .bind(publication_id)
    .bind(asset_id)
    .bind(lineage_id)
    .execute(pool)
    .await
    .expect("insert publication");

    (asset_id, publication_id, lineage_id)
}

#[tokio::test]
async fn p3_device_and_package_audit_shapes_persist_without_weakening_p2_contract() {
    let pool = test_pool().await;
    let device_id = Uuid::new_v4();
    let device_event = AuditEvent::new_p3_event(
        None,
        AuditEventKind::P2pDeviceRegistered,
        device_id,
        None,
        None,
        Some(r#"{"operation":"register_device"}"#.to_owned()),
    );
    insert_audit_event(&pool, &device_event)
        .await
        .expect("persist package-less device event");

    let (asset_id, publication_id, lineage_id) = insert_asset_and_publication(&pool).await;
    let authorization_id = Uuid::new_v4();
    let package_event = AuditEvent::new_p3_event(
        Some(AssetId(asset_id)),
        AuditEventKind::P2pAudienceAuthorizationIssued,
        authorization_id,
        Some(publication_id),
        Some(lineage_id),
        Some(r#"{"operation":"authorize"}"#.to_owned()),
    );
    insert_audit_event(&pool, &package_event)
        .await
        .expect("persist exact-package P3 event");

    let rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE id = ANY($1)",
    )
    .bind(vec![device_event.id, package_event.id])
    .fetch_one(&pool)
    .await
    .expect("count P3 audit rows");
    assert_eq!(rows, 2);
}

#[tokio::test]
async fn p3_package_event_without_complete_package_identity_is_rejected() {
    let pool = test_pool().await;
    let (asset_id, publication_id, _) = insert_asset_and_publication(&pool).await;
    let malformed = AuditEvent::new_p3_event(
        Some(AssetId(asset_id)),
        AuditEventKind::P2pInvitationCreated,
        Uuid::new_v4(),
        Some(publication_id),
        None,
        None,
    );

    assert!(insert_audit_event(&pool, &malformed).await.is_err());
}
