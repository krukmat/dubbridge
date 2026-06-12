use dubbridge_auth::SharedTokenVerifier;
use dubbridge_storage::StorageAdapter;
use sqlx::PgPool;

use crate::workspace_service::{SharedWorkspaceService, pg_workspace_service};

pub struct AppState {
    pub pool: PgPool,
    pub storage: Box<dyn StorageAdapter + Send + Sync>,
    pub verifier: SharedTokenVerifier,
    pub config: dubbridge_config::AppConfig,
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
            workspace_service: pg_workspace_service(pool.clone()),
            pool,
            storage,
            verifier,
            config,
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
            pool,
            storage,
            verifier,
            config,
            workspace_service,
        }
    }
}
