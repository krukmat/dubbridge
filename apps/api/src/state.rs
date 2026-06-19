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
