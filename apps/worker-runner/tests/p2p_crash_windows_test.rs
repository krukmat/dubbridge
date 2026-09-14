use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use dubbridge_connectors::p2p_availability::{
    AvailabilityPublicationError, AvailabilityPublicationEvidence, AvailabilityPublicationRequest,
    CONTRACT_VERSION,
};
use dubbridge_db::{
    create_pool,
    p2p_publication_claim_repo::claim_next_publication_work,
    p2p_publication_repo::{
        ensure_publication_with_outbox, get_publication, record_external_confirmation,
        record_sealed_k1, transition_publication_state,
    },
    p2p_ready_repo::{get_ready_descriptor_by_asset, persist_confirmed_manifest_digest},
};
use dubbridge_domain::{
    asset::AssetId,
    p2p_publication::{K1LineageId, P2pPublicationId, PublicationState},
};
use dubbridge_jobs::p2p_publication_job::{
    AvailabilityPublisher, DispatchTick, P2pPublicationDispatcher,
};
use dubbridge_p2p::manifest::{Manifest, canonical_json, manifest_sha256};
use sqlx::PgPool;
use tempfile::TempDir;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../infra/migrations");
const EXTERNAL_ID: &str = "hyperdrive:t6-crash-window";
const CONFIRMED_AT: &str = "2026-09-14T10:00:00Z";

#[derive(Clone, Copy)]
enum Outcome {
    Success,
    Ambiguous,
}

struct FakePublisher(Mutex<VecDeque<Outcome>>);

#[async_trait]
impl AvailabilityPublisher for FakePublisher {
    async fn publish(
        &self,
        request: &AvailabilityPublicationRequest,
    ) -> Result<AvailabilityPublicationEvidence, AvailabilityPublicationError> {
        match self
            .0
            .lock()
            .expect("outcomes lock")
            .pop_front()
            .unwrap_or(Outcome::Success)
        {
            Outcome::Success => Ok(AvailabilityPublicationEvidence {
                contract_version: CONTRACT_VERSION.to_string(),
                publication_id: request.publication_id.clone(),
                lineage_id: request.lineage_id.clone(),
                manifest_digest_sha256: request.manifest_digest_sha256.clone(),
                external_publication_id: EXTERNAL_ID.to_string(),
                evidence_id: "evidence:t6-crash-window".to_string(),
                confirmed_at: CONFIRMED_AT.to_string(),
            }),
            Outcome::Ambiguous => Err(AvailabilityPublicationError::AmbiguousOutcome),
        }
    }
}

struct Fixture {
    asset_id: AssetId,
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    outbox_id: Uuid,
    package_root: TempDir,
    manifest_digest: String,
}

async fn test_pool() -> PgPool {
    let url = std::env::var("DUBBRIDGE_DATABASE_URL").expect("DUBBRIDGE_DATABASE_URL");
    let pool = create_pool(&url).await.expect("connect database");
    MIGRATOR.run(&pool).await.expect("run migrations");
    pool
}

async fn fixture(pool: &PgPool) -> Fixture {
    let asset_id = AssetId::new();
    sqlx::query(
        "INSERT INTO assets (id, title, uploader_id, status) VALUES ($1, 'P2 T6 crash', $2, 'finalized')",
    )
    .bind(asset_id.0)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("insert asset");
    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();
    let outbox_id = Uuid::new_v4();
    ensure_publication_with_outbox(pool, asset_id, publication_id, lineage_id, outbox_id)
        .await
        .expect("create publication intent");

    let package_root = tempfile::tempdir().expect("package root");
    let package_dir = package_root.path().join(publication_id.to_string());
    tokio::fs::create_dir_all(&package_dir)
        .await
        .expect("package dir");
    let manifest = Manifest {
        asset_id: asset_id.to_string(),
        cipher: "AES-256-GCM".to_string(),
        digest: "SHA-256".to_string(),
        files: Vec::new(),
        lineage_id: lineage_id.to_string(),
        manifest_version: "p2p-manifest-v1".to_string(),
        publication_id: publication_id.to_string(),
    };
    let canonical = canonical_json(&manifest);
    let manifest_digest = manifest_sha256(&canonical);
    tokio::fs::write(package_dir.join("manifest.json"), canonical)
        .await
        .expect("manifest");
    Fixture {
        asset_id,
        publication_id,
        lineage_id,
        outbox_id,
        package_root,
        manifest_digest,
    }
}

async fn seal(pool: &PgPool, f: &Fixture) {
    record_sealed_k1(
        pool,
        f.publication_id,
        f.lineage_id,
        "t6-kek",
        1,
        &[7; 12],
        &[9; 48],
    )
    .await
    .expect("seal K1");
}

async fn pending(pool: &PgPool, f: &Fixture) {
    transition_publication_state(
        pool,
        f.publication_id,
        PublicationState::PublishPending,
        None,
    )
    .await
    .expect("publish pending");
}

fn dispatcher(
    pool: &PgPool,
    f: &Fixture,
    outcomes: impl IntoIterator<Item = Outcome>,
) -> P2pPublicationDispatcher {
    P2pPublicationDispatcher::new(
        pool.clone(),
        Arc::new(FakePublisher(Mutex::new(outcomes.into_iter().collect()))),
        f.package_root.path().to_path_buf(),
        Duration::minutes(5),
        Duration::ZERO,
        3,
    )
    .expect("dispatcher")
}

async fn assert_not_ready(pool: &PgPool, f: &Fixture) {
    assert!(
        get_ready_descriptor_by_asset(pool, f.asset_id)
            .await
            .expect("descriptor")
            .is_none()
    );
}

async fn assert_single_lineage(pool: &PgPool, f: &Fixture) {
    let rows: Vec<(Uuid, Uuid)> =
        sqlx::query_as("SELECT id, lineage_id FROM p2p_publications WHERE asset_id = $1")
            .bind(f.asset_id.0)
            .fetch_all(pool)
            .await
            .expect("identities");
    assert_eq!(rows, vec![(f.publication_id.0, f.lineage_id.0)]);
}

#[tokio::test]
async fn window_1_restart_after_intent_reuses_publication_and_lineage() {
    let pool = test_pool().await;
    let f = fixture(&pool).await;
    let replay = ensure_publication_with_outbox(
        &pool,
        f.asset_id,
        f.publication_id,
        f.lineage_id,
        Uuid::new_v4(),
    )
    .await
    .expect("intent replay");
    assert_eq!(replay.publication.id, f.publication_id);
    assert_eq!(replay.publication.lineage_id, f.lineage_id);
    assert_not_ready(&pool, &f).await;
    assert_single_lineage(&pool, &f).await;
}

#[tokio::test]
async fn window_2_restart_after_k1_seal_reuses_sealed_lineage() {
    let pool = test_pool().await;
    let f = fixture(&pool).await;
    seal(&pool, &f).await;
    seal(&pool, &f).await;
    let publication = get_publication(&pool, f.publication_id)
        .await
        .expect("read")
        .expect("publication");
    assert_eq!(publication.state, PublicationState::Building);
    assert_eq!(publication.sealed_kek_id.as_deref(), Some("t6-kek"));
    assert_not_ready(&pool, &f).await;
    assert_single_lineage(&pool, &f).await;
}

#[tokio::test]
async fn window_3_restart_after_publish_pending_converges_ready() {
    let pool = test_pool().await;
    let f = fixture(&pool).await;
    seal(&pool, &f).await;
    pending(&pool, &f).await;
    assert_not_ready(&pool, &f).await;
    assert_eq!(
        dispatcher(&pool, &f, [Outcome::Success])
            .dispatch_once()
            .await
            .expect("dispatch"),
        DispatchTick::Ready(f.publication_id)
    );
    assert!(
        get_ready_descriptor_by_asset(&pool, f.asset_id)
            .await
            .expect("descriptor")
            .is_some()
    );
    assert_single_lineage(&pool, &f).await;
}

#[tokio::test]
async fn window_4_expired_claim_is_reclaimed_without_second_lineage() {
    let pool = test_pool().await;
    let f = fixture(&pool).await;
    seal(&pool, &f).await;
    pending(&pool, &f).await;
    let token = Uuid::new_v4();
    claim_next_publication_work(
        &pool,
        token,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await
    .expect("claim query")
    .expect("claim");
    sqlx::query("UPDATE p2p_publication_outbox SET lease_expires_at = now() - interval '1 minute' WHERE id = $1 AND claim_token = $2")
        .bind(f.outbox_id)
        .bind(token)
        .execute(&pool)
        .await
        .expect("expire lease");
    assert_not_ready(&pool, &f).await;
    assert_eq!(
        dispatcher(&pool, &f, [Outcome::Success])
            .dispatch_once()
            .await
            .expect("reclaim"),
        DispatchTick::Ready(f.publication_id)
    );
    assert_single_lineage(&pool, &f).await;
}

#[tokio::test]
async fn window_5_ambiguous_remote_outcome_replays_same_lineage() {
    let pool = test_pool().await;
    let f = fixture(&pool).await;
    seal(&pool, &f).await;
    pending(&pool, &f).await;
    let d = dispatcher(&pool, &f, [Outcome::Ambiguous, Outcome::Success]);
    assert_eq!(
        d.dispatch_once().await.expect("ambiguous"),
        DispatchTick::Retrying(f.publication_id)
    );
    assert_not_ready(&pool, &f).await;
    assert_eq!(
        d.dispatch_once().await.expect("replay"),
        DispatchTick::Ready(f.publication_id)
    );
    assert_single_lineage(&pool, &f).await;
}

#[tokio::test]
async fn window_6_persisted_external_evidence_recovers_without_false_ready() {
    let pool = test_pool().await;
    let f = fixture(&pool).await;
    seal(&pool, &f).await;
    pending(&pool, &f).await;
    transition_publication_state(&pool, f.publication_id, PublicationState::Publishing, None)
        .await
        .expect("publishing");
    persist_confirmed_manifest_digest(&pool, f.publication_id, f.lineage_id, &f.manifest_digest)
        .await
        .expect("digest");
    record_external_confirmation(
        &pool,
        f.publication_id,
        f.lineage_id,
        EXTERNAL_ID,
        OffsetDateTime::parse(CONFIRMED_AT, &Rfc3339).expect("confirmed at"),
    )
    .await
    .expect("external evidence");
    transition_publication_state(&pool, f.publication_id, PublicationState::Reconciling, None)
        .await
        .expect("reconciling");
    assert_not_ready(&pool, &f).await;
    assert_eq!(
        dispatcher(&pool, &f, [Outcome::Success])
            .dispatch_once()
            .await
            .expect("reconcile"),
        DispatchTick::Ready(f.publication_id)
    );
    assert_single_lineage(&pool, &f).await;
}
