use dubbridge_db::{
    create_pool,
    error::DbError,
    p2p_audience_repo::{
        claim_invitation, create_invitation, get_active_authorization, list_viewer_invitations,
        register_or_get_active_device,
    },
    p2p_envelope_repo::get_envelope_release_context,
};
use dubbridge_domain::asset::AssetId;
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

fn token_hash(seed: Uuid) -> [u8; 32] {
    let mut hash = [0_u8; 32];
    hash[..16].copy_from_slice(seed.as_bytes());
    hash[16..].copy_from_slice(seed.as_bytes());
    hash
}

async fn insert_ready_publication(pool: &PgPool, owner: Uuid) -> (AssetId, Uuid, Uuid) {
    let asset_id = AssetId(Uuid::new_v4());
    let publication_id = Uuid::new_v4();
    let lineage_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();

    sqlx::query(
        "INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, $2, $3, 'finalized')",
    )
    .bind(asset_id.0)
    .bind("P2P audience integration asset")
    .bind(owner)
    .execute(pool)
    .await
    .expect("insert asset");

    sqlx::query(
        r#"
        INSERT INTO p2p_publications (
            id, asset_id, lineage_id, state,
            external_publication_id, confirmed_lineage_id, external_confirmed_at,
            sealed_kek_id, sealed_kek_version, sealed_nonce, sealed_wrapped_ck, sealed_at,
            manifest_digest_sha256
        )
        VALUES (
            $1, $2, $3, 'ready',
            $4, $3, $5,
            'kek-test', 1, $6, $7, $5,
            $8
        )
        "#,
    )
    .bind(publication_id)
    .bind(asset_id.0)
    .bind(lineage_id)
    .bind(format!("hyperdrive:{publication_id}"))
    .bind(now)
    .bind(vec![7_u8; 12])
    .bind(vec![9_u8; 48])
    .bind("a".repeat(64))
    .execute(pool)
    .await
    .expect("insert ready publication");

    sqlx::query(
        r#"
        INSERT INTO p2p_publication_outbox (
            id, publication_id, lineage_id, delivery_state, delivered_at
        )
        VALUES ($1, $2, $3, 'delivered', $4)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(publication_id)
    .bind(lineage_id)
    .bind(now)
    .execute(pool)
    .await
    .expect("insert delivered outbox");

    (asset_id, publication_id, lineage_id)
}


#[derive(Clone, Copy)]
struct ReleaseFixture {
    viewer: Uuid,
    authorization_id: Uuid,
    invitation_id: Uuid,
    device_id: Uuid,
    publication_id: Uuid,
    lineage_id: Uuid,
    now: OffsetDateTime,
}

async fn setup_release_fixture(pool: &PgPool) -> ReleaseFixture {
    let owner = Uuid::new_v4();
    let viewer = Uuid::new_v4();
    let (asset_id, publication_id, lineage_id) = insert_ready_publication(pool, owner).await;
    let now = OffsetDateTime::now_utc();
    let hash = token_hash(Uuid::new_v4());
    let invitation = create_invitation(pool, owner, asset_id, &hash, now + Duration::hours(1))
        .await
        .expect("create release invitation");
    let device = register_or_get_active_device(
        pool,
        viewer,
        &format!("viewer-key-{}", Uuid::new_v4()),
        &[1_u8, 2, 3],
    )
    .await
    .expect("register release device");
    let claim = claim_invitation(pool, &hash, viewer, device.id, now)
        .await
        .expect("claim release invitation");

    ReleaseFixture {
        viewer,
        authorization_id: claim.authorization.id,
        invitation_id: invitation.id,
        device_id: device.id,
        publication_id,
        lineage_id,
        now,
    }
}

async fn assert_release_not_found(pool: &PgPool, fixture: ReleaseFixture) {
    assert!(matches!(
        get_envelope_release_context(
            pool,
            fixture.authorization_id,
            fixture.viewer,
            fixture.now + Duration::seconds(1),
        )
        .await,
        Err(DbError::NotFound)
    ));
}

async fn assert_invalid_invitation_creation(
    pool: &PgPool,
    owner: Uuid,
    asset_id: AssetId,
    now: OffsetDateTime,
) {
    assert!(matches!(
        create_invitation(
            pool,
            owner,
            asset_id,
            &token_hash(Uuid::new_v4()),
            now - Duration::seconds(1),
        )
        .await,
        Err(DbError::Conflict)
    ));
    assert!(matches!(
        create_invitation(
            pool,
            Uuid::new_v4(),
            asset_id,
            &token_hash(Uuid::new_v4()),
            now + Duration::hours(1),
        )
        .await,
        Err(DbError::NotFound)
    ));
}

#[tokio::test]
async fn device_registration_is_idempotent_and_rejects_conflicting_active_identity() {
    let pool = test_pool().await;
    let viewer = Uuid::new_v4();

    assert!(matches!(
        register_or_get_active_device(&pool, viewer, "", &[1_u8]).await,
        Err(DbError::Conflict)
    ));
    assert!(matches!(
        register_or_get_active_device(&pool, viewer, "device-key", &[]).await,
        Err(DbError::Conflict)
    ));

    let created = register_or_get_active_device(&pool, viewer, "device-key", &[1_u8, 2, 3])
        .await
        .expect("register device");
    let repeated = register_or_get_active_device(&pool, viewer, "device-key", &[1_u8, 2, 3])
        .await
        .expect("same active device is idempotent");

    assert_eq!(created.id, repeated.id);
    assert_eq!(created.subject_id, viewer);
    assert_eq!(created.key_id, "device-key");
    assert!(created.revoked_at.is_none());

    assert!(matches!(
        register_or_get_active_device(&pool, viewer, "different-key", &[4_u8, 5, 6]).await,
        Err(DbError::Conflict)
    ));
}

#[tokio::test]
async fn invitation_claim_is_owner_scoped_idempotent_and_exposes_only_active_authorization() {
    let pool = test_pool().await;
    let owner = Uuid::new_v4();
    let viewer = Uuid::new_v4();
    let foreign_viewer = Uuid::new_v4();
    let (asset_id, publication_id, lineage_id) = insert_ready_publication(&pool, owner).await;
    let now = OffsetDateTime::now_utc();

    assert_invalid_invitation_creation(&pool, owner, asset_id, now).await;

    let hash = token_hash(Uuid::new_v4());
    let invitation = create_invitation(&pool, owner, asset_id, &hash, now + Duration::hours(1))
        .await
        .expect("create invitation");
    assert_eq!(invitation.asset_id(), asset_id);
    assert_eq!(invitation.publication_id().0, publication_id);
    assert_eq!(invitation.lineage_id().0, lineage_id);

    let device = register_or_get_active_device(&pool, viewer, "viewer-key", &[1_u8, 2, 3])
        .await
        .expect("register viewer device");
    let claim = claim_invitation(&pool, &hash, viewer, device.id, now)
        .await
        .expect("claim invitation");

    assert_eq!(claim.invitation.claimed_by_subject_id, Some(viewer));
    assert_eq!(claim.invitation.claimed_device_id, Some(device.id));
    assert_eq!(claim.authorization.viewer_subject_id, viewer);
    assert_eq!(claim.authorization.device_id, device.id);

    let repeated = claim_invitation(&pool, &hash, viewer, device.id, now + Duration::seconds(1))
        .await
        .expect("same viewer/device claim is idempotent");
    assert_eq!(repeated.invitation.id, claim.invitation.id);
    assert_eq!(repeated.authorization.id, claim.authorization.id);

    let inbox = list_viewer_invitations(&pool, viewer)
        .await
        .expect("list viewer invitations");
    assert!(inbox.iter().any(|row| row.id == invitation.id));

    let active = get_active_authorization(
        &pool,
        claim.authorization.id,
        viewer,
        now + Duration::seconds(2),
    )
    .await
    .expect("active authorization");
    assert_eq!(active.id, claim.authorization.id);

    assert!(matches!(
        get_active_authorization(
            &pool,
            claim.authorization.id,
            foreign_viewer,
            now + Duration::seconds(2),
        )
        .await,
        Err(DbError::NotFound)
    ));

    let foreign_device =
        register_or_get_active_device(&pool, foreign_viewer, "foreign-key", &[4_u8, 5, 6])
            .await
            .expect("register foreign device");
    assert!(matches!(
        claim_invitation(
            &pool,
            &hash,
            foreign_viewer,
            foreign_device.id,
            now + Duration::seconds(2),
        )
        .await,
        Err(DbError::Conflict)
    ));
}

#[tokio::test]
async fn concurrent_different_viewer_claims_have_exactly_one_winner() {
    let pool = test_pool().await;
    let owner = Uuid::new_v4();
    let viewer_a = Uuid::new_v4();
    let viewer_b = Uuid::new_v4();
    let (asset_id, _, _) = insert_ready_publication(&pool, owner).await;
    let now = OffsetDateTime::now_utc();
    let hash = token_hash(Uuid::new_v4());

    let invitation = create_invitation(&pool, owner, asset_id, &hash, now + Duration::hours(1))
        .await
        .expect("create invitation");
    let device_a = register_or_get_active_device(&pool, viewer_a, "viewer-a-key", &[1_u8, 2, 3])
        .await
        .expect("register viewer A device");
    let device_b = register_or_get_active_device(&pool, viewer_b, "viewer-b-key", &[4_u8, 5, 6])
        .await
        .expect("register viewer B device");

    let pool_a = pool.clone();
    let pool_b = pool.clone();
    let claim_a = async { claim_invitation(&pool_a, &hash, viewer_a, device_a.id, now).await };
    let claim_b = async { claim_invitation(&pool_b, &hash, viewer_b, device_b.id, now).await };
    let (result_a, result_b) = tokio::join!(claim_a, claim_b);

    let winners = [&result_a, &result_b]
        .into_iter()
        .filter(|result| result.is_ok())
        .count();
    let conflicts = [&result_a, &result_b]
        .into_iter()
        .filter(|result| matches!(result, Err(DbError::Conflict)))
        .count();

    assert_eq!(winners, 1, "exactly one concurrent claim must win");
    assert_eq!(conflicts, 1, "the losing concurrent claim must fail closed");

    let authorization_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM p2p_audience_authorizations WHERE invitation_id = $1",
    )
    .bind(invitation.id)
    .fetch_one(&pool)
    .await
    .expect("count invitation authorizations");
    assert_eq!(authorization_count, 1);

    let winner = result_a
        .as_ref()
        .ok()
        .or_else(|| result_b.as_ref().ok())
        .expect("winner");
    let repeated = claim_invitation(
        &pool,
        &hash,
        winner.authorization.viewer_subject_id,
        winner.authorization.device_id,
        now + Duration::seconds(1),
    )
    .await
    .expect("winning viewer/device claim remains idempotent");
    assert_eq!(repeated.authorization.id, winner.authorization.id);
}

#[tokio::test]
async fn claim_fails_closed_when_ready_publication_drifts_after_invitation_creation() {
    let pool = test_pool().await;
    let owner = Uuid::new_v4();
    let viewer = Uuid::new_v4();
    let (asset_id, publication_id, _) = insert_ready_publication(&pool, owner).await;
    let now = OffsetDateTime::now_utc();
    let hash = token_hash(Uuid::new_v4());

    create_invitation(&pool, owner, asset_id, &hash, now + Duration::hours(1))
        .await
        .expect("create invitation");
    let device = register_or_get_active_device(&pool, viewer, "viewer-key", &[1_u8, 2, 3])
        .await
        .expect("register device");

    sqlx::query("UPDATE p2p_publications SET state = 'failed' WHERE id = $1")
        .bind(publication_id)
        .execute(&pool)
        .await
        .expect("drift publication state");

    assert!(matches!(
        claim_invitation(&pool, &hash, viewer, device.id, now).await,
        Err(DbError::Conflict)
    ));
}

#[tokio::test]
async fn envelope_release_requires_live_claim_device_and_ready_publication_evidence() {
    let pool = test_pool().await;
    let owner = Uuid::new_v4();
    let viewer = Uuid::new_v4();
    let (asset_id, publication_id, lineage_id) = insert_ready_publication(&pool, owner).await;
    let now = OffsetDateTime::now_utc();
    let hash = token_hash(Uuid::new_v4());

    let invitation = create_invitation(&pool, owner, asset_id, &hash, now + Duration::hours(1))
        .await
        .expect("create invitation");
    let device = register_or_get_active_device(&pool, viewer, "viewer-key", &[1_u8, 2, 3])
        .await
        .expect("register device");
    let claim = claim_invitation(&pool, &hash, viewer, device.id, now)
        .await
        .expect("claim invitation");

    let context = get_envelope_release_context(
        &pool,
        claim.authorization.id,
        viewer,
        now + Duration::seconds(1),
    )
    .await
    .expect("release context");

    assert_eq!(context.authorization_id, claim.authorization.id);
    assert_eq!(context.invitation_id, invitation.id);
    assert_eq!(context.asset_id, asset_id.0);
    assert_eq!(context.publication_id, publication_id);
    assert_eq!(context.lineage_id, lineage_id);
    assert_eq!(context.viewer_subject_id, viewer);
    assert_eq!(context.device_id, device.id);
    assert_eq!(context.device_key_id, "viewer-key");
    assert_eq!(context.device_public_key_spki, vec![1_u8, 2, 3]);
    assert_eq!(context.sealed_kek_id, "kek-test");
    assert_eq!(context.sealed_kek_version, 1);
    assert_eq!(context.sealed_nonce, vec![7_u8; 12]);
    assert_eq!(context.sealed_wrapped_ck, vec![9_u8; 48]);

    sqlx::query("UPDATE p2p_devices SET revoked_at = now() WHERE id = $1")
        .bind(device.id)
        .execute(&pool)
        .await
        .expect("revoke device");

    assert!(matches!(
        get_envelope_release_context(
            &pool,
            claim.authorization.id,
            viewer,
            now + Duration::seconds(2),
        )
        .await,
        Err(DbError::NotFound)
    ));
}


#[tokio::test]
async fn envelope_release_denies_wrong_viewer_and_dead_authorization() {
    let pool = test_pool().await;

    let wrong_viewer = setup_release_fixture(&pool).await;
    assert!(matches!(
        get_envelope_release_context(
            &pool,
            wrong_viewer.authorization_id,
            Uuid::new_v4(),
            wrong_viewer.now + Duration::seconds(1),
        )
        .await,
        Err(DbError::NotFound)
    ));

    let revoked = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_audience_authorizations SET revoked_at = $1 WHERE id = $2")
        .bind(revoked.now)
        .bind(revoked.authorization_id)
        .execute(&pool)
        .await
        .expect("revoke authorization");
    assert_release_not_found(&pool, revoked).await;

    let expired = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_audience_authorizations SET expires_at = $1 WHERE id = $2")
        .bind(expired.now - Duration::seconds(1))
        .bind(expired.authorization_id)
        .execute(&pool)
        .await
        .expect("expire authorization");
    assert_release_not_found(&pool, expired).await;
}

#[tokio::test]
async fn envelope_release_denies_dead_invitation_or_device_binding_drift() {
    let pool = test_pool().await;

    let revoked_invitation = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_invitations SET revoked_at = $1 WHERE id = $2")
        .bind(revoked_invitation.now)
        .bind(revoked_invitation.invitation_id)
        .execute(&pool)
        .await
        .expect("revoke invitation");
    assert_release_not_found(&pool, revoked_invitation).await;

    let expired_invitation = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_invitations SET expires_at = $1 WHERE id = $2")
        .bind(expired_invitation.now - Duration::seconds(1))
        .bind(expired_invitation.invitation_id)
        .execute(&pool)
        .await
        .expect("expire invitation");
    assert_release_not_found(&pool, expired_invitation).await;

    let revoked_device = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_devices SET revoked_at = $1 WHERE id = $2")
        .bind(revoked_device.now)
        .bind(revoked_device.device_id)
        .execute(&pool)
        .await
        .expect("revoke device");
    assert_release_not_found(&pool, revoked_device).await;

    let mismatched_device = setup_release_fixture(&pool).await;
    let foreign_viewer = Uuid::new_v4();
    let foreign_device = register_or_get_active_device(
        &pool,
        foreign_viewer,
        &format!("foreign-key-{}", Uuid::new_v4()),
        &[4_u8, 5, 6],
    )
    .await
    .expect("register foreign device");
    sqlx::query("UPDATE p2p_audience_authorizations SET device_id = $1 WHERE id = $2")
        .bind(foreign_device.id)
        .bind(mismatched_device.authorization_id)
        .execute(&pool)
        .await
        .expect("drift authorization device");
    assert_release_not_found(&pool, mismatched_device).await;
}

#[tokio::test]
async fn envelope_release_denies_publication_readiness_or_delivery_drift() {
    let pool = test_pool().await;

    let non_ready = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_publications SET state = 'failed' WHERE id = $1")
        .bind(non_ready.publication_id)
        .execute(&pool)
        .await
        .expect("mark publication failed");
    assert_release_not_found(&pool, non_ready).await;

    let lineage_drift = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_publications SET confirmed_lineage_id = $1 WHERE id = $2")
        .bind(Uuid::new_v4())
        .bind(lineage_drift.publication_id)
        .execute(&pool)
        .await
        .expect("drift confirmed lineage");
    assert_release_not_found(&pool, lineage_drift).await;

    let external_missing = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_publications SET external_publication_id = NULL WHERE id = $1")
        .bind(external_missing.publication_id)
        .execute(&pool)
        .await
        .expect("remove external publication");
    assert_release_not_found(&pool, external_missing).await;

    let sealed_missing = setup_release_fixture(&pool).await;
    sqlx::query("UPDATE p2p_publications SET sealed_kek_id = NULL WHERE id = $1")
        .bind(sealed_missing.publication_id)
        .execute(&pool)
        .await
        .expect("remove sealed K1 evidence");
    assert_release_not_found(&pool, sealed_missing).await;

    let undelivered = setup_release_fixture(&pool).await;
    sqlx::query(
        "UPDATE p2p_publication_outbox SET delivery_state = 'pending', delivered_at = NULL WHERE publication_id = $1 AND lineage_id = $2",
    )
    .bind(undelivered.publication_id)
    .bind(undelivered.lineage_id)
    .execute(&pool)
    .await
    .expect("remove durable delivery evidence");
    assert_release_not_found(&pool, undelivered).await;
}

