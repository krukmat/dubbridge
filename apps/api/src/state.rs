use std::sync::Arc;

use dubbridge_auth::{AuthService, Hs256Issuer, PgAccountStore, SharedTokenVerifier};
use dubbridge_jobs::{InMemoryPreparationJobQueue, SharedPreparationJobQueue};
use dubbridge_storage::StorageAdapter;
use sqlx::PgPool;

use crate::workspace_service::{SharedWorkspaceService, pg_workspace_service};

pub type ApiAuthService = AuthService<PgAccountStore, Hs256Issuer>;
pub type SharedAuthService = Arc<ApiAuthService>;

pub struct AppState {
    pub pool: PgPool,
    pub storage: Box<dyn StorageAdapter + Send + Sync>,
    pub verifier: SharedTokenVerifier,
    pub config: dubbridge_config::AppConfig,
    pub preparation_queue: SharedPreparationJobQueue,
    pub auth_service: Option<SharedAuthService>,
    pub workspace_service: SharedWorkspaceService,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        storage: Box<dyn StorageAdapter + Send + Sync>,
        verifier: SharedTokenVerifier,
        config: dubbridge_config::AppConfig,
    ) -> Self {
        Self {
            preparation_queue: Arc::new(InMemoryPreparationJobQueue::default()),
            workspace_service: pg_workspace_service(pool.clone()),
            pool,
            storage,
            verifier,
            config,
            auth_service: None,
        }
    }

    pub fn with_auth_service(
        pool: PgPool,
        storage: Box<dyn StorageAdapter + Send + Sync>,
        verifier: SharedTokenVerifier,
        config: dubbridge_config::AppConfig,
        auth_service: SharedAuthService,
    ) -> Self {
        Self {
            preparation_queue: Arc::new(InMemoryPreparationJobQueue::default()),
            workspace_service: pg_workspace_service(pool.clone()),
            pool,
            storage,
            verifier,
            config,
            auth_service: Some(auth_service),
        }
    }

    pub fn with_workspace_service(
        pool: PgPool,
        storage: Box<dyn StorageAdapter + Send + Sync>,
        verifier: SharedTokenVerifier,
        config: dubbridge_config::AppConfig,
        workspace_service: SharedWorkspaceService,
    ) -> Self {
        Self {
            preparation_queue: Arc::new(InMemoryPreparationJobQueue::default()),
            pool,
            storage,
            verifier,
            config,
            auth_service: None,
            workspace_service,
        }
    }

    pub fn with_preparation_queue(
        pool: PgPool,
        storage: Box<dyn StorageAdapter + Send + Sync>,
        verifier: SharedTokenVerifier,
        config: dubbridge_config::AppConfig,
        preparation_queue: SharedPreparationJobQueue,
    ) -> Self {
        Self {
            pool: pool.clone(),
            storage,
            verifier,
            config,
            preparation_queue,
            auth_service: None,
            workspace_service: pg_workspace_service(pool),
        }
    }
}

impl AppState {
    pub fn with_auth_service_and_preparation_queue(
        pool: PgPool,
        storage: Box<dyn StorageAdapter + Send + Sync>,
        verifier: SharedTokenVerifier,
        config: dubbridge_config::AppConfig,
        auth_service: SharedAuthService,
        preparation_queue: SharedPreparationJobQueue,
    ) -> Self {
        Self {
            preparation_queue,
            workspace_service: pg_workspace_service(pool.clone()),
            pool,
            storage,
            verifier,
            config,
            auth_service: Some(auth_service),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dubbridge_jobs::{InMemoryPreparationJobQueue, PreparationJob};
    use std::sync::Arc;

    fn test_fixtures() -> (
        PgPool,
        Box<dyn StorageAdapter + Send + Sync>,
        SharedTokenVerifier,
        dubbridge_config::AppConfig,
        SharedAuthService,
        tempfile::TempDir,
    ) {
        let pool = PgPool::connect_lazy("postgres://user:pass@db.example.com:5432/dubbridge")
            .expect("lazy pool");
        let storage_dir = tempfile::TempDir::new().expect("temp dir");
        let storage: Box<dyn StorageAdapter + Send + Sync> =
            Box::new(dubbridge_storage::LocalFsAdapter::new(storage_dir.path()));
        let verifier: SharedTokenVerifier =
            Arc::new(dubbridge_auth::Hs256TokenVerifier::new("test-secret").expect("verifier"));
        let config = dubbridge_config::AppConfig {
            env: dubbridge_config::AppEnv::Local,
            api_port: 8080,
            database_url: "postgres://user:pass@db.example.com:5432/dubbridge".to_string(),
            redis_url: "redis://redis.example.com:6379".to_string(),
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
        };
        let auth_service: SharedAuthService = Arc::new(dubbridge_auth::AuthService::new(
            dubbridge_auth::PgAccountStore::new(pool.clone()),
            dubbridge_auth::Hs256Issuer::new("test-secret", std::time::Duration::from_secs(3600))
                .expect("issuer"),
        ));
        (pool, storage, verifier, config, auth_service, storage_dir)
    }

    #[tokio::test]
    async fn with_auth_service_and_preparation_queue_uses_passed_queue() {
        let (pool, storage, verifier, config, auth_service, _storage_dir) = test_fixtures();
        let inner_queue = Arc::new(InMemoryPreparationJobQueue::default());
        let state = AppState::with_auth_service_and_preparation_queue(
            pool,
            storage,
            verifier,
            config,
            auth_service,
            inner_queue.clone() as SharedPreparationJobQueue,
        );

        state
            .preparation_queue
            .enqueue(PreparationJob::new(
                uuid::Uuid::new_v4(),
                uuid::Uuid::new_v4(),
                uuid::Uuid::new_v4(),
            ))
            .await
            .expect("enqueue");

        assert_eq!(inner_queue.queued_jobs().len(), 1);
    }

    #[tokio::test]
    async fn with_auth_service_and_preparation_queue_sets_auth_service() {
        let (pool, storage, verifier, config, auth_service, _storage_dir) = test_fixtures();
        let inner_queue = Arc::new(InMemoryPreparationJobQueue::default());
        let state = AppState::with_auth_service_and_preparation_queue(
            pool,
            storage,
            verifier,
            config,
            auth_service,
            inner_queue.clone() as SharedPreparationJobQueue,
        );

        assert!(state.auth_service.is_some());
    }
}
