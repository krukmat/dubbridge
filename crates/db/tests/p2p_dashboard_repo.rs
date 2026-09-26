use dubbridge_db::{
    create_pool,
    p2p_dashboard_repo::{list_owner_p2p_content, list_viewer_p2p_inbox},
};
use dubbridge_domain::p2p_publication::PublicationState;
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

async fn insert_asset_and_publication(
    pool: &PgPool,
    owner: Uuid,
    title: &str,
    state: &str,
) -> (Uuid, Uuid, Uuid) {
    let asset_id = Uuid::new_v4();
    let publication_id = Uuid::new_v4();
    let lineage_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, 'finalized')",
    )
    .bind(asset_id)
    .bind(title)
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert asset");
    sqlx::query(
        "INSERT INTO p2p_publications (id, asset_id, lineage_id, state) VALUES ($1, $2, $3, $4)",
    )
    .bind(publication_id)
    .bind(asset_id)
    .bind(lineage_id)
    .bind(state)
    .execute(pool)
    .await
    .expect("insert publication");
    (asset_id, publication_id, lineage_id)
}

async fn insert_claimed_invitation(
    pool: &PgPool,
    owner: Uuid,
    viewer: Uuid,
    asset_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
) -> (Uuid, Uuid) {
    let device_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO p2p_devices (id, subject_id, key_id, public_key_spki) VALUES ($1, $2, $3, $4)",
    )
    .bind(device_id)
    .bind(viewer)
    .bind(format!("key-{device_id}"))
    .bind(vec![1_u8, 2, 3])
    .execute(pool)
    .await
    .expect("insert device");

    let invitation_id = Uuid::new_v4();
    let mut token_hash = Vec::with_capacity(32);
    token_hash.extend_from_slice(invitation_id.as_bytes());
    token_hash.extend_from_slice(invitation_id.as_bytes());
    let now = OffsetDateTime::now_utc();
    let expires_at = now + Duration::hours(1);
    sqlx::query(
        r#"
        INSERT INTO p2p_invitations (
            id, asset_id, publication_id, lineage_id, owner_subject_id,
            token_hash, expires_at, claimed_by_subject_id, claimed_device_id, claimed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(invitation_id)
    .bind(asset_id)
    .bind(publication_id)
    .bind(lineage_id)
    .bind(owner)
    .bind(token_hash)
    .bind(expires_at)
    .bind(viewer)
    .bind(device_id)
    .bind(now)
    .execute(pool)
    .await
    .expect("insert invitation");

    let authorization_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO p2p_audience_authorizations (
            id, invitation_id, asset_id, publication_id, lineage_id,
            viewer_subject_id, device_id, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(authorization_id)
    .bind(invitation_id)
    .bind(asset_id)
    .bind(publication_id)
    .bind(lineage_id)
    .bind(viewer)
    .bind(device_id)
    .bind(expires_at)
    .execute(pool)
    .await
    .expect("insert authorization");

    (invitation_id, authorization_id)
}

#[tokio::test]
async fn owner_content_is_scoped_to_owned_assets_and_preserves_publication_state() {
    let pool = test_pool().await;
    let owner = Uuid::new_v4();
    let other_owner = Uuid::new_v4();
    let (owned_asset, owned_publication, _) =
        insert_asset_and_publication(&pool, owner, "Owned P2P asset", "failed").await;
    insert_asset_and_publication(&pool, other_owner, "Foreign P2P asset", "building").await;

    let rows = list_owner_p2p_content(&pool, owner)
        .await
        .expect("list owner content");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].asset_id.0, owned_asset);
    assert_eq!(rows[0].publication_id.0, owned_publication);
    assert_eq!(rows[0].asset_title, "Owned P2P asset");
    assert_eq!(rows[0].state, PublicationState::Failed);
}

#[tokio::test]
async fn viewer_inbox_returns_only_the_authenticated_viewers_claimed_authorization() {
    let pool = test_pool().await;
    let owner = Uuid::new_v4();
    let viewer = Uuid::new_v4();
    let other_viewer = Uuid::new_v4();
    let (asset_id, publication_id, lineage_id) =
        insert_asset_and_publication(&pool, owner, "Inbox asset", "building").await;
    let (expected_invitation, expected_authorization) =
        insert_claimed_invitation(&pool, owner, viewer, asset_id, publication_id, lineage_id).await;
    let (other_asset_id, other_publication_id, other_lineage_id) =
        insert_asset_and_publication(&pool, owner, "Other viewer asset", "building").await;
    insert_claimed_invitation(
        &pool,
        owner,
        other_viewer,
        other_asset_id,
        other_publication_id,
        other_lineage_id,
    )
    .await;

    let rows = list_viewer_p2p_inbox(&pool, viewer)
        .await
        .expect("list viewer inbox");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].invitation.id, expected_invitation);
    assert_eq!(rows[0].authorization.id, expected_authorization);
    assert_eq!(rows[0].authorization.viewer_subject_id, viewer);
}
