use dubbridge_auth::SharedTokenVerifier;
use dubbridge_storage::StorageAdapter;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub storage: Box<dyn StorageAdapter + Send + Sync>,
    pub verifier: SharedTokenVerifier,
    pub config: dubbridge_config::AppConfig,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        storage: Box<dyn StorageAdapter + Send + Sync>,
        verifier: SharedTokenVerifier,
        config: dubbridge_config::AppConfig,
    ) -> Self {
        Self {
            pool,
            storage,
            verifier,
            config,
        }
    }
}
