//! Proves a job enqueued through `AppState`'s Redis-backed preparation queue
//! (constructed exactly as `apps/api/src/main.rs` does at startup) is visible
//! to an independent Redis-backed consumer, not merely that `enqueue`
//! returned `Ok`. Requires `DUBBRIDGE_REDIS_URL`; run explicitly via
//! `cargo test -p dubbridge-api --test redis_preparation_queue_test -- --ignored`.

use std::sync::Arc;

use apalis::prelude::Storage;
use dubbridge_api::state::{AppState, SharedAuthService};
use dubbridge_auth::{
    AuthService, Hs256Issuer, Hs256TokenVerifier, PgAccountStore, SharedTokenVerifier,
};
use dubbridge_jobs::{PreparationJob, RedisPreparationJobQueue, SharedPreparationJobQueue};
use dubbridge_storage::{LocalFsAdapter, StorageAdapter};
use sqlx::PgPool;
use tempfile::TempDir;
use uuid::Uuid;

fn redis_url_for_test() -> String {
    std::env::var("DUBBRIDGE_REDIS_URL")
        .expect("DUBBRIDGE_REDIS_URL must be set when running ignored Redis integration tests")
}

fn test_config(redis_url: String) -> dubbridge_config::AppConfig {
    dubbridge_config::AppConfig {
        env: dubbridge_config::AppEnv::Local,
        api_port: 8080,
        database_url: "postgres://user:pass@db.example.com:5432/dubbridge".to_string(),
        redis_url,
        worker_concurrency: 4,
        storage: dubbridge_config::StorageSettings {
            backend: dubbridge_config::StorageBackend::S3,
            bucket: "dubbridge-test".to_string(),
            ..Default::default()
        },
        observability: dubbridge_config::ObsSettings {
            log_format: dubbridge_config::LogFormat::Json,
            filter: "info".to_string(),
        },
        auth: Some(dubbridge_config::AuthSettings {
            issuer: "https://issuer.example".to_string(),
            audience: "dubbridge-api".to_string(),
            rsa_public_key_path: "/tmp/public.pem".to_string(),
            jwt_secret: Some("test-secret".to_string()),
            jwt_expiry_hours: 24,
            clock_skew_leeway_seconds: 30,
        }),
        gateway: None,
    }
}

#[tokio::test]
#[ignore = "requires DUBBRIDGE_REDIS_URL; run explicitly via \
             `cargo test -p dubbridge-api --test redis_preparation_queue_test -- --ignored`"]
async fn job_enqueued_through_api_configured_queue_is_visible_to_redis_consumer() {
    let url = redis_url_for_test();

    let pool = PgPool::connect_lazy("postgres://user:pass@db.example.com:5432/dubbridge")
        .expect("lazy pool");
    let storage_dir = TempDir::new().expect("temp dir");
    let storage: Box<dyn StorageAdapter + Send + Sync> =
        Box::new(LocalFsAdapter::new(storage_dir.path()));
    let verifier: SharedTokenVerifier =
        Arc::new(Hs256TokenVerifier::new("test-secret").expect("verifier"));
    let config = test_config(url.clone());
    let auth_service: SharedAuthService = Arc::new(AuthService::new(
        PgAccountStore::new(pool.clone()),
        Hs256Issuer::new("test-secret", std::time::Duration::from_secs(3600)).expect("issuer"),
    ));

    // Build the queue exactly as apps/api/src/main.rs does at startup.
    let redis_queue = Arc::new(
        RedisPreparationJobQueue::connect(&url)
            .await
            .expect("redis connect"),
    );

    let state = Arc::new(AppState::with_auth_service_and_preparation_queue(
        pool,
        storage,
        verifier,
        config,
        auth_service,
        redis_queue.clone() as SharedPreparationJobQueue,
    ));

    let job = PreparationJob::new(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

    // redis_queue and state.preparation_queue are two Arc handles to the SAME
    // RedisPreparationJobQueue instance, exactly like AppState receives it in
    // production main.rs -- enqueue_with_id here is enqueueing through the
    // identical queue AppState's route handlers would use.
    let task_id = redis_queue
        .enqueue_with_id(job.clone())
        .await
        .expect("enqueue via API's configured queue");

    // Independent connection into the same namespace: this is exactly what an
    // apalis worker polls, so retrieval here proves the round trip through the
    // API's configured (not a test double's) queue.
    let conn = apalis_redis::connect(url.as_str())
        .await
        .expect("probe connect");
    let probe_config = apalis_redis::Config::default().set_namespace(PreparationJob::JOB_TYPE);
    let mut probe: apalis_redis::RedisStorage<PreparationJob> =
        apalis_redis::RedisStorage::new_with_config(conn, probe_config);

    let fetched = probe.fetch_by_id(&task_id).await.expect("fetch");
    let fetched = fetched.expect("job present in its own namespace");
    assert_eq!(fetched.args, job);

    // The trait-object path AppState actually exposes to route handlers also
    // works end to end.
    state
        .preparation_queue
        .enqueue(PreparationJob::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ))
        .await
        .expect("enqueue via AppState.preparation_queue trait object");
}
