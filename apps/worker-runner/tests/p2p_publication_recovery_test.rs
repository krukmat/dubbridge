use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use dubbridge_connectors::p2p_availability::{
    AvailabilityPublicationError, AvailabilityPublicationEvidence, AvailabilityPublicationRequest,
    CONTRACT_VERSION,
};
use dubbridge_db::p2p_publication_claim_repo::claim_next_publication_work;
use dubbridge_db::p2p_publication_repo::{
    ensure_publication_with_outbox, get_outbox_for_publication, get_publication,
    record_external_confirmation, transition_publication_state,
};
use dubbridge_db::{create_pool, error::DbError};
use dubbridge_domain::asset::AssetId;
use dubbridge_domain::p2p_publication::{K1LineageId, P2pPublicationId, PublicationState};
use dubbridge_jobs::p2p_publication_job::{
    AvailabilityPublisher, DispatchTick, P2pPublicationDispatcher,
};
use dubbridge_p2p::manifest::{Manifest, canonical_json};
use sqlx::PgPool;
use tempfile::TempDir;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../infra/migrations");
const EXTERNAL_PUBLICATION_ID: &str = "hyperdrive:recovery-test-stable-key";
const CONFIRMED_AT: &str = "2026-09-14T06:00:00Z";

#[derive(Clone, Copy)]
enum FakeOutcome {
    Success,
    Error(AvailabilityPublicationError),
}

struct FakePublisher {
    outcomes: Mutex<VecDeque<FakeOutcome>>,
    requests: Mutex<Vec<AvailabilityPublicationRequest>>,
}

impl FakePublisher {
    fn new(outcomes: impl IntoIterator<Item = FakeOutcome>) -> Self {
        Self {
            outcomes: Mutex::new(outcomes.into_iter().collect()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<AvailabilityPublicationRequest> {
        self.requests.lock().expect("requests lock").clone()
    }
}

#[async_trait]
impl AvailabilityPublisher for FakePublisher {
    async fn publish(
        &self,
        request: &AvailabilityPublicationRequest,
    ) -> Result<AvailabilityPublicationEvidence, AvailabilityPublicationError> {
        self.requests
            .lock()
            .expect("requests lock")
            .push(request.clone());
        let outcome = self
            .outcomes
            .lock()
            .expect("outcomes lock")
            .pop_front()
            .unwrap_or(FakeOutcome::Success);

        match outcome {
            FakeOutcome::Success => Ok(success_evidence(request)),
            FakeOutcome::Error(error) => Err(error),
        }
    }
}

struct WorkFixture {
    publication_id: P2pPublicationId,
    lineage_id: K1LineageId,
    outbox_id: Uuid,
    package_root: TempDir,
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

async fn create_work(pool: &PgPool) -> WorkFixture {
    let asset_id = AssetId(Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO assets (id, title, uploader_id, status)
        VALUES ($1, 'P2P T4 recovery test asset', $2, 'finalized')
        "#,
    )
    .bind(asset_id.0)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("insert asset fixture");

    let publication_id = P2pPublicationId::new();
    let lineage_id = K1LineageId::new();
    let outbox_id = Uuid::new_v4();
    ensure_publication_with_outbox(pool, asset_id, publication_id, lineage_id, outbox_id)
        .await
        .expect("create publication and outbox");
    transition_publication_state(pool, publication_id, PublicationState::PublishPending, None)
        .await
        .expect("building -> publish_pending");

    let package_root = tempfile::tempdir().expect("package root");
    let package_dir = package_root.path().join(publication_id.to_string());
    tokio::fs::create_dir_all(&package_dir)
        .await
        .expect("create package dir");
    let manifest = Manifest {
        asset_id: asset_id.0.to_string(),
        cipher: "AES-256-GCM".to_string(),
        digest: "SHA-256".to_string(),
        files: Vec::new(),
        lineage_id: lineage_id.to_string(),
        manifest_version: "p2p-manifest-v1".to_string(),
        publication_id: publication_id.to_string(),
    };
    tokio::fs::write(package_dir.join("manifest.json"), canonical_json(&manifest))
        .await
        .expect("write manifest");

    WorkFixture {
        publication_id,
        lineage_id,
        outbox_id,
        package_root,
    }
}

fn dispatcher(
    pool: &PgPool,
    publisher: Arc<dyn AvailabilityPublisher>,
    fixture: &WorkFixture,
    max_attempts: u32,
) -> P2pPublicationDispatcher {
    P2pPublicationDispatcher::new(
        pool.clone(),
        publisher,
        fixture.package_root.path().to_path_buf(),
        Duration::minutes(5),
        Duration::ZERO,
        max_attempts,
    )
    .expect("dispatcher")
}

fn success_evidence(request: &AvailabilityPublicationRequest) -> AvailabilityPublicationEvidence {
    AvailabilityPublicationEvidence {
        contract_version: CONTRACT_VERSION.to_string(),
        publication_id: request.publication_id.clone(),
        lineage_id: request.lineage_id.clone(),
        manifest_digest_sha256: request.manifest_digest_sha256.clone(),
        external_publication_id: EXTERNAL_PUBLICATION_ID.to_string(),
        evidence_id: "availability-evidence-recovery-test".to_string(),
        confirmed_at: CONFIRMED_AT.to_string(),
    }
}

#[tokio::test]
async fn t4f_unknown_outcome_replays_same_lineage_and_converges_ready() {
    let pool = test_pool().await;
    let fixture = create_work(&pool).await;
    let publisher = Arc::new(FakePublisher::new([
        FakeOutcome::Error(AvailabilityPublicationError::AmbiguousOutcome),
        FakeOutcome::Success,
    ]));
    let dispatcher = dispatcher(&pool, publisher.clone(), &fixture, 3);

    assert_eq!(
        dispatcher.dispatch_once().await.expect("first dispatch"),
        DispatchTick::Retrying(fixture.publication_id)
    );
    assert_eq!(
        get_publication(&pool, fixture.publication_id)
            .await
            .expect("read publication")
            .expect("publication")
            .state,
        PublicationState::Reconciling
    );

    assert_eq!(
        dispatcher.dispatch_once().await.expect("replay dispatch"),
        DispatchTick::Ready(fixture.publication_id)
    );
    let requests = publisher.requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0], requests[1]);
    assert_eq!(requests[0].lineage_id, fixture.lineage_id.to_string());

    let outbox = get_outbox_for_publication(&pool, fixture.publication_id)
        .await
        .expect("read outbox")
        .expect("outbox");
    assert_eq!(outbox.delivery_state, "delivered");
    assert_eq!(outbox.attempt_count, 2);
}

#[tokio::test]
async fn t4f_stale_lease_is_reclaimed_without_exactly_once_assumption() {
    let pool = test_pool().await;
    let fixture = create_work(&pool).await;
    let first_token = Uuid::new_v4();
    claim_next_publication_work(
        &pool,
        first_token,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await
    .expect("claim query")
    .expect("first claim");
    sqlx::query(
        r#"
        UPDATE p2p_publication_outbox
           SET lease_expires_at = now() - interval '1 minute'
         WHERE id = $1 AND claim_token = $2
        "#,
    )
    .bind(fixture.outbox_id)
    .bind(first_token)
    .execute(&pool)
    .await
    .expect("expire lease");

    let publisher = Arc::new(FakePublisher::new([FakeOutcome::Success]));
    let dispatcher = dispatcher(&pool, publisher, &fixture, 3);
    assert_eq!(
        dispatcher
            .dispatch_once()
            .await
            .expect("reclaimed dispatch"),
        DispatchTick::Ready(fixture.publication_id)
    );

    let outbox = get_outbox_for_publication(&pool, fixture.publication_id)
        .await
        .expect("read outbox")
        .expect("outbox");
    assert_eq!(outbox.attempt_count, 2);
    assert_eq!(outbox.delivery_state, "delivered");
}

#[tokio::test]
async fn t4f_duplicate_after_ready_is_idle_and_does_not_redispatch() {
    let pool = test_pool().await;
    let fixture = create_work(&pool).await;
    let publisher = Arc::new(FakePublisher::new([FakeOutcome::Success]));
    let dispatcher = dispatcher(&pool, publisher.clone(), &fixture, 3);

    assert_eq!(
        dispatcher.dispatch_once().await.expect("initial dispatch"),
        DispatchTick::Ready(fixture.publication_id)
    );
    assert_eq!(
        dispatcher.dispatch_once().await.expect("duplicate scan"),
        DispatchTick::Idle
    );
    assert_eq!(publisher.requests().len(), 1);
}

#[tokio::test]
async fn t4f_persisted_remote_confirmation_survives_lost_ready_commit() {
    let pool = test_pool().await;
    let fixture = create_work(&pool).await;
    transition_publication_state(
        &pool,
        fixture.publication_id,
        PublicationState::Publishing,
        None,
    )
    .await
    .expect("publish_pending -> publishing");
    record_external_confirmation(
        &pool,
        fixture.publication_id,
        fixture.lineage_id,
        EXTERNAL_PUBLICATION_ID,
        OffsetDateTime::parse(CONFIRMED_AT, &time::format_description::well_known::Rfc3339)
            .expect("confirmed at"),
    )
    .await
    .expect("persist remote confirmation");
    transition_publication_state(
        &pool,
        fixture.publication_id,
        PublicationState::Reconciling,
        None,
    )
    .await
    .expect("publishing -> reconciling");

    let publisher = Arc::new(FakePublisher::new([FakeOutcome::Success]));
    let dispatcher = dispatcher(&pool, publisher, &fixture, 3);
    assert_eq!(
        dispatcher
            .dispatch_once()
            .await
            .expect("reconcile dispatch"),
        DispatchTick::Ready(fixture.publication_id)
    );

    let publication = get_publication(&pool, fixture.publication_id)
        .await
        .expect("read publication")
        .expect("publication");
    assert_eq!(publication.state, PublicationState::Ready);
    assert_eq!(
        publication.external_publication_id.as_deref(),
        Some(EXTERNAL_PUBLICATION_ID)
    );
}

#[tokio::test]
async fn t4f_retry_budget_exhaustion_persists_terminal_failure() {
    let pool = test_pool().await;
    let fixture = create_work(&pool).await;
    let publisher = Arc::new(FakePublisher::new([FakeOutcome::Error(
        AvailabilityPublicationError::AmbiguousOutcome,
    )]));
    let dispatcher = dispatcher(&pool, publisher, &fixture, 1);

    assert_eq!(
        dispatcher.dispatch_once().await.expect("bounded dispatch"),
        DispatchTick::Failed(fixture.publication_id)
    );
    let publication = get_publication(&pool, fixture.publication_id)
        .await
        .expect("read publication")
        .expect("publication");
    assert_eq!(publication.state, PublicationState::Failed);
    assert_eq!(
        publication.failure_detail.as_deref(),
        Some("publication_outcome_unknown")
    );

    let outbox = get_outbox_for_publication(&pool, fixture.publication_id)
        .await
        .expect("read terminal outbox")
        .expect("terminal outbox");
    assert_eq!(outbox.delivery_state, "pending");
    assert_eq!(
        outbox.last_error.as_deref(),
        Some("publication_outcome_unknown")
    );
    let claim_token: Option<Uuid> =
        sqlx::query_scalar("SELECT claim_token FROM p2p_publication_outbox WHERE id = $1")
            .bind(fixture.outbox_id)
            .fetch_one(&pool)
            .await
            .expect("read terminal claim token");
    assert_eq!(claim_token, None);
}

#[tokio::test]
async fn t4f_foreign_claim_completion_remains_fail_closed() {
    let pool = test_pool().await;
    let fixture = create_work(&pool).await;
    let token = Uuid::new_v4();
    let claim = claim_next_publication_work(
        &pool,
        token,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )
    .await
    .expect("claim query")
    .expect("claim");

    let result = dubbridge_db::p2p_publication_claim_repo::complete_publication_claim(
        &pool,
        claim.outbox_id,
        Uuid::new_v4(),
        OffsetDateTime::now_utc(),
    )
    .await;
    assert!(matches!(result, Err(DbError::Conflict)));
}
