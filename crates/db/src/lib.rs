// T3: S1 db crate — pool factory and repository modules
// S-120-T2: preparation_repo added
pub mod artifact_repo;
pub mod asset_repo;
pub mod audit_repo;
pub mod consent_repo;
pub mod error;
pub mod notification_repo;
pub mod pending_ingestion_repo;
pub mod playback_repo;
pub mod preparation_repo;
pub mod review_repo;
pub mod rights_repo;
pub mod user_account;
pub mod workspace_repo;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::error::DbError;

pub async fn create_pool(database_url: &str) -> Result<PgPool, DbError> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(DbError::ConnectionFailed)
}
